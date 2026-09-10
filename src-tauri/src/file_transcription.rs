//! Imported-file transcription: the job that turns a file the user picked into
//! a history entry, using the same ASR engine and settings as live dictation.
//!
//! Everything here runs off the async runtime on a blocking worker. The command
//! wrappers live in `commands/file_transcription.rs`; this module owns the job
//! state, the lifecycle stages reported to the UI, and the cleanup rules.

use crate::audio_toolkit::{decode_to_mono_16k, save_wav_file, verify_wav_file, DecodeError};
use crate::managers::history::HistoryManager;
use crate::managers::transcription::TranscriptionManager;
use crate::settings::get_settings;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::AppHandle;
use tauri_specta::Event;

/// How often progress is pushed to the UI while decoding. Decoding a long file
/// produces thousands of packets; the UI only needs a readable cadence.
const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);

/// Which part of the import the job is currently in.
///
/// Only `Decoding` can report a meaningful fraction — inference is one opaque
/// call into the engine, so the UI shows an indeterminate state for it rather
/// than inventing a percentage.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FileTranscriptionStage {
    Decoding,
    LoadingModel,
    Transcribing,
    Saving,
}

/// Progress event for the active import job.
#[derive(Clone, Debug, Serialize, Deserialize, Type, tauri_specta::Event)]
pub struct FileTranscriptionProgress {
    pub job_id: String,
    pub stage: FileTranscriptionStage,
    /// Fraction in `0.0..=1.0`, or `None` when the stage cannot honestly
    /// report one.
    pub progress: Option<f64>,
}

/// Everything that can go wrong with an import, as distinct cases.
///
/// Each variant maps to exactly one localized message in the frontend
/// (`fileTranscription.errors.*`); `detail` carries the technical text for the
/// log and is never shown on its own.
#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[serde(tag = "kind", content = "detail", rename_all = "snake_case")]
pub enum FileTranscriptionError {
    /// Not a format we advertise, or the container couldn't be read.
    UnsupportedFormat,
    /// The container was fine but its codec has no decoder — Opus in `.ogg`,
    /// most commonly. `detail` names the codec so the message can too.
    UnsupportedCodec(String),
    /// The file could not be opened or read.
    UnreadableFile(String),
    /// A decoder matched but the stream is damaged.
    CorruptAudio(String),
    /// The file decoded to nothing at all.
    EmptyAudio,
    /// Longer than a single-pass decode will take on. The exact ceiling is in
    /// the log; the user's move is the same whatever it is — split the file.
    FileTooLong,
    /// Decoding worked but the engine found no speech.
    NoSpeech,
    /// No transcription model is selected, or the selected one failed to load.
    ModelUnavailable(String),
    /// The engine ran and failed.
    TranscriptionFailed(String),
    /// Writing the managed recording or the history row failed.
    StorageFailed(String),
    /// Live dictation or another import already owns the engine.
    Busy,
    /// The user cancelled.
    Cancelled,
}

impl From<DecodeError> for FileTranscriptionError {
    fn from(error: DecodeError) -> Self {
        match error {
            DecodeError::UnsupportedFormat => Self::UnsupportedFormat,
            DecodeError::UnsupportedCodec(name) => Self::UnsupportedCodec(name),
            DecodeError::Io(detail) => Self::UnreadableFile(detail),
            DecodeError::Corrupt(detail) => Self::CorruptAudio(detail),
            DecodeError::Empty => Self::EmptyAudio,
            DecodeError::TooLong => Self::FileTooLong,
            DecodeError::Cancelled => Self::Cancelled,
        }
    }
}

/// What the UI shows about a file before the user commits to transcribing it.
#[derive(Clone, Debug, Serialize, Deserialize, Type)]
pub struct AudioFileDetails {
    pub path: String,
    pub file_name: String,
    pub size_bytes: u64,
    /// `None` when the container declares no frame count.
    pub duration_ms: Option<u64>,
    /// `None` when the container declares no sample rate.
    pub sample_rate: Option<u32>,
    /// `None` when the container declares no channel layout, which MP4 does not
    /// for AAC. Absent here says nothing about whether the file will decode.
    pub channels: Option<u16>,
}

/// A finished import.
#[derive(Clone, Debug, Serialize, Deserialize, Type)]
pub struct FileTranscriptionResult {
    pub job_id: String,
    pub history_entry_id: i64,
    pub text: String,
    pub source_name: String,
    pub duration_ms: i64,
}

/// The single in-flight import job, if any.
///
/// Only one import runs at a time (the engine is exclusive anyway), so this is
/// one slot rather than a map.
#[derive(Default)]
pub struct FileTranscriptionState {
    active: Mutex<Option<ActiveJob>>,
}

struct ActiveJob {
    id: String,
    cancel: Arc<AtomicBool>,
}

impl FileTranscriptionState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a job and hand back its cancellation flag.
    ///
    /// Called before the worker is spawned so that a cancel arriving in the
    /// first instants of a job always finds it.
    pub fn begin(&self, id: String) -> Arc<AtomicBool> {
        let cancel = Arc::new(AtomicBool::new(false));
        *self.active.lock().unwrap() = Some(ActiveJob {
            id,
            cancel: Arc::clone(&cancel),
        });
        cancel
    }

    /// Clear the slot, but only if it still holds the job with this id — a late
    /// finish must not wipe a job that started after it.
    fn end(&self, id: &str) {
        let mut active = self.active.lock().unwrap();
        if active.as_ref().is_some_and(|job| job.id == id) {
            *active = None;
        }
    }

    /// Ask the running job to stop. Returns the job id when there was one.
    pub fn request_cancel(&self) -> Option<String> {
        let active = self.active.lock().unwrap();
        active.as_ref().map(|job| {
            job.cancel.store(true, Ordering::Relaxed);
            job.id.clone()
        })
    }
}

/// Deletes a file when dropped, unless it has been disarmed.
///
/// The managed WAV is written before inference so the history entry has audio
/// to point at, which means every failure path after that point has to remove
/// it again. Doing that with a guard rather than by hand is what keeps
/// cancellation and errors from leaving orphans in the recordings directory.
pub struct ManagedFileGuard {
    path: PathBuf,
    armed: bool,
}

impl ManagedFileGuard {
    pub fn new(path: PathBuf) -> Self {
        Self { path, armed: true }
    }

    /// Keep the file — the entry that owns it has been committed.
    pub fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for ManagedFileGuard {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        if self.path.exists() {
            match std::fs::remove_file(&self.path) {
                Ok(()) => debug!("Removed orphaned import recording {:?}", self.path),
                Err(e) => error!(
                    "Failed to remove orphaned import recording {:?}: {}",
                    self.path, e
                ),
            }
        }
    }
}

/// Releases the shared-engine claim when the job ends, however it ends.
struct ActivityGuard {
    activity: Arc<crate::TranscriptionActivity>,
}

impl Drop for ActivityGuard {
    fn drop(&mut self) {
        self.activity.end_file();
    }
}

/// Identifier for one import job, unique enough to tell a late finish from the
/// job that replaced it.
pub fn new_job_id() -> String {
    format!("import-{}", chrono::Utc::now().timestamp_millis())
}

/// Milliseconds of audio represented by `sample_count` samples at 16 kHz mono.
pub fn duration_ms_for_samples(sample_count: usize) -> i64 {
    (sample_count as f64 * 1000.0 / crate::audio_toolkit::TARGET_SAMPLE_RATE as f64).round() as i64
}

/// Managed recording file name for an import started at `timestamp_ms`.
///
/// The `dictus-import-` prefix keeps imports visually distinct from dictation
/// recordings (`dictus-<ts>.wav`) in the recordings folder, while staying in
/// the same directory so retention and deletion treat them identically.
pub fn managed_file_name(timestamp_ms: i64) -> String {
    format!("dictus-import-{}.wav", timestamp_ms)
}

/// Run one import end to end. Blocking; call from a blocking worker.
///
/// `activity` must already have been claimed by the caller via
/// `try_begin_file` — it is released when this function returns.
#[allow(clippy::too_many_arguments)]
pub fn run_job(
    app: AppHandle,
    activity: Arc<crate::TranscriptionActivity>,
    state: Arc<FileTranscriptionState>,
    history_manager: Arc<HistoryManager>,
    transcription_manager: Arc<TranscriptionManager>,
    path: String,
    job_id: String,
    cancel: Arc<AtomicBool>,
) -> Result<FileTranscriptionResult, FileTranscriptionError> {
    let _activity_guard = ActivityGuard { activity };
    let _job_slot = JobSlotGuard {
        state: Arc::clone(&state),
        job_id: job_id.clone(),
    };

    let source_path = PathBuf::from(&path);
    let source_name = file_name_of(&source_path);
    info!(
        "Starting file transcription job {} for {}",
        job_id, source_name
    );

    // Fail before decoding a long file if there is nothing to transcribe with.
    let settings = get_settings(&app);
    if settings.selected_model.trim().is_empty() {
        return Err(FileTranscriptionError::ModelUnavailable(
            "no transcription model is selected".to_string(),
        ));
    }

    // --- Decode -----------------------------------------------------------
    emit_progress(&app, &job_id, FileTranscriptionStage::Decoding, Some(0.0));
    let mut last_emit = Instant::now();
    let samples = decode_to_mono_16k(&source_path, &cancel, &mut |fraction| {
        if last_emit.elapsed() >= PROGRESS_INTERVAL {
            last_emit = Instant::now();
            emit_progress(&app, &job_id, FileTranscriptionStage::Decoding, fraction);
        }
    })?;
    emit_progress(&app, &job_id, FileTranscriptionStage::Decoding, Some(1.0));
    check_cancelled(&cancel)?;

    let duration_ms = duration_ms_for_samples(samples.len());
    debug!(
        "Decoded {} to {} samples ({} ms)",
        source_name,
        samples.len(),
        duration_ms
    );

    // --- Load model -------------------------------------------------------
    emit_progress(&app, &job_id, FileTranscriptionStage::LoadingModel, None);
    transcription_manager.initiate_model_load();

    // --- Persist the normalised copy -------------------------------------
    // Written before inference so the history row always has audio behind it;
    // the guard removes it again on every failure and on cancellation.
    let sample_count = samples.len();
    let file_name = managed_file_name(chrono::Utc::now().timestamp_millis());
    let wav_path = history_manager.recordings_dir().join(&file_name);
    let mut managed = ManagedFileGuard::new(wav_path.clone());
    save_wav_file(&wav_path, &samples)
        .map_err(|e| FileTranscriptionError::StorageFailed(e.to_string()))?;
    verify_wav_file(&wav_path, sample_count)
        .map_err(|e| FileTranscriptionError::StorageFailed(e.to_string()))?;
    check_cancelled(&cancel)?;

    // --- Transcribe -------------------------------------------------------
    emit_progress(&app, &job_id, FileTranscriptionStage::Transcribing, None);
    let text = transcription_manager.transcribe(samples).map_err(|e| {
        // A model that never loaded is a different problem for the user than an
        // engine that ran and failed, so tell the two apart on the way out.
        if transcription_manager.is_model_loaded() {
            FileTranscriptionError::TranscriptionFailed(e.to_string())
        } else {
            FileTranscriptionError::ModelUnavailable(e.to_string())
        }
    })?;

    // Inference is a single opaque call and cannot be interrupted, so a cancel
    // raised while it was running lands here: throw the result away rather than
    // saving something the user asked us to abandon.
    check_cancelled(&cancel)?;

    if text.trim().is_empty() {
        warn!("File transcription job {} produced no speech", job_id);
        return Err(FileTranscriptionError::NoSpeech);
    }

    // --- Save -------------------------------------------------------------
    emit_progress(&app, &job_id, FileTranscriptionStage::Saving, None);
    let entry = history_manager
        .save_imported_entry(
            file_name,
            text.clone(),
            source_name.clone(),
            Some(duration_ms),
        )
        .map_err(|e| FileTranscriptionError::StorageFailed(e.to_string()))?;
    managed.disarm();

    info!(
        "File transcription job {} saved as history entry {}",
        job_id, entry.id
    );

    Ok(FileTranscriptionResult {
        job_id,
        history_entry_id: entry.id,
        text,
        source_name,
        duration_ms,
    })
}

/// Clears the job slot when the job ends, however it ends.
struct JobSlotGuard {
    state: Arc<FileTranscriptionState>,
    job_id: String,
}

impl Drop for JobSlotGuard {
    fn drop(&mut self) {
        self.state.end(&self.job_id);
    }
}

fn check_cancelled(cancel: &AtomicBool) -> Result<(), FileTranscriptionError> {
    if cancel.load(Ordering::Relaxed) {
        Err(FileTranscriptionError::Cancelled)
    } else {
        Ok(())
    }
}

fn emit_progress(
    app: &AppHandle,
    job_id: &str,
    stage: FileTranscriptionStage,
    progress: Option<f64>,
) {
    if let Err(e) = (FileTranscriptionProgress {
        job_id: job_id.to_string(),
        stage,
        progress,
    })
    .emit(app)
    {
        error!("Failed to emit file transcription progress: {}", e);
    }
}

/// The file name a user would recognise, falling back to the whole path when
/// the OS gives us something without one.
pub fn file_name_of(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_file_guard_removes_the_file_when_the_job_fails() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("dictus-import-1.wav");
        std::fs::write(&path, b"audio").expect("write file");

        {
            let _guard = ManagedFileGuard::new(path.clone());
        }

        assert!(
            !path.exists(),
            "a job that never committed must not leave a managed recording behind"
        );
    }

    #[test]
    fn managed_file_guard_keeps_the_file_once_disarmed() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("dictus-import-2.wav");
        std::fs::write(&path, b"audio").expect("write file");

        {
            let mut guard = ManagedFileGuard::new(path.clone());
            guard.disarm();
        }

        assert!(
            path.exists(),
            "a committed history entry must keep its recording"
        );
    }

    #[test]
    fn managed_file_guard_tolerates_a_missing_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("never-written.wav");

        // Decode failing before the WAV is written is a normal path.
        drop(ManagedFileGuard::new(path.clone()));

        assert!(!path.exists());
    }

    #[test]
    fn cancelling_an_idle_state_reports_no_job() {
        let state = FileTranscriptionState::new();
        assert!(state.request_cancel().is_none());
    }

    #[test]
    fn cancelling_a_running_job_sets_its_flag() {
        let state = FileTranscriptionState::new();
        let cancel = state.begin("import-1".to_string());

        let cancelled_id = state.request_cancel();

        assert_eq!(cancelled_id.as_deref(), Some("import-1"));
        assert!(cancel.load(Ordering::Relaxed));
        assert!(matches!(
            check_cancelled(&cancel),
            Err(FileTranscriptionError::Cancelled)
        ));
    }

    #[test]
    fn a_finished_job_does_not_clear_a_newer_one() {
        let state = FileTranscriptionState::new();
        state.begin("import-old".to_string());
        state.begin("import-new".to_string());

        state.end("import-old");

        assert_eq!(
            state.request_cancel().as_deref(),
            Some("import-new"),
            "the newer job must still be cancellable"
        );
    }

    #[test]
    fn job_slot_is_cleared_when_the_job_ends() {
        let state = Arc::new(FileTranscriptionState::new());
        state.begin("import-3".to_string());

        {
            let _guard = JobSlotGuard {
                state: Arc::clone(&state),
                job_id: "import-3".to_string(),
            };
        }

        assert!(
            state.request_cancel().is_none(),
            "no job may stay registered once it has finished"
        );
    }

    #[test]
    fn decode_errors_map_to_distinct_user_facing_cases() {
        assert!(matches!(
            FileTranscriptionError::from(DecodeError::UnsupportedFormat),
            FileTranscriptionError::UnsupportedFormat
        ));
        match FileTranscriptionError::from(DecodeError::UnsupportedCodec("Opus".into())) {
            FileTranscriptionError::UnsupportedCodec(name) => assert_eq!(name, "Opus"),
            other => panic!("expected UnsupportedCodec, got {other:?}"),
        }
        assert!(matches!(
            FileTranscriptionError::from(DecodeError::Io("nope".into())),
            FileTranscriptionError::UnreadableFile(_)
        ));
        assert!(matches!(
            FileTranscriptionError::from(DecodeError::Corrupt("bad".into())),
            FileTranscriptionError::CorruptAudio(_)
        ));
        assert!(matches!(
            FileTranscriptionError::from(DecodeError::Empty),
            FileTranscriptionError::EmptyAudio
        ));
        assert!(matches!(
            FileTranscriptionError::from(DecodeError::TooLong),
            FileTranscriptionError::FileTooLong
        ));
        assert!(matches!(
            FileTranscriptionError::from(DecodeError::Cancelled),
            FileTranscriptionError::Cancelled
        ));
    }

    #[test]
    fn duration_is_derived_from_the_normalised_sample_count() {
        assert_eq!(duration_ms_for_samples(16_000), 1_000);
        assert_eq!(duration_ms_for_samples(8_000), 500);
        assert_eq!(duration_ms_for_samples(0), 0);
    }

    #[test]
    fn managed_recordings_are_named_distinctly_from_dictation() {
        let name = managed_file_name(1_700_000_000_000);
        assert!(name.starts_with("dictus-import-"));
        assert!(name.ends_with(".wav"));
    }

    #[test]
    fn source_name_is_the_file_name_not_the_whole_path() {
        assert_eq!(
            file_name_of(Path::new("/Users/someone/Voice Memos/interview.m4a")),
            "interview.m4a"
        );
    }
}
