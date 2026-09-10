use crate::audio_toolkit::{has_supported_extension, probe_audio_file, SUPPORTED_EXTENSIONS};
use crate::file_transcription::{
    file_name_of, new_job_id, run_job, AudioFileDetails, FileTranscriptionError,
    FileTranscriptionResult, FileTranscriptionState,
};
use crate::managers::history::HistoryManager;
use crate::managers::transcription::TranscriptionManager;
use crate::TranscriptionActivity;
use log::info;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, State};

/// The extensions the file picker should offer. Sourced from the decoder so the
/// two can't drift apart.
#[tauri::command]
#[specta::specta]
pub fn supported_audio_extensions() -> Vec<String> {
    SUPPORTED_EXTENSIONS.iter().map(|e| e.to_string()).collect()
}

/// Read a file's headers so the workspace can show what the user picked before
/// committing to a transcription. No audio is decoded here.
#[tauri::command]
#[specta::specta]
pub async fn inspect_audio_file(path: String) -> Result<AudioFileDetails, FileTranscriptionError> {
    // Every step below hits the filesystem — stat, open, demux headers, build a
    // decoder — so none of it belongs on an async worker. It also sits on the
    // critical path of every drop now that picking a file starts the job.
    tauri::async_runtime::spawn_blocking(move || {
        let file_path = PathBuf::from(&path);

        if !has_supported_extension(&file_path) {
            return Err(FileTranscriptionError::UnsupportedFormat);
        }

        let size_bytes = std::fs::metadata(&file_path)
            .map_err(|e| FileTranscriptionError::UnreadableFile(e.to_string()))?
            .len();

        let info = probe_audio_file(&file_path)?;

        Ok(AudioFileDetails {
            file_name: file_name_of(&file_path),
            path,
            size_bytes,
            duration_ms: info.duration_ms,
            sample_rate: info.sample_rate,
            channels: info.channels,
        })
    })
    .await
    .map_err(|e| FileTranscriptionError::UnreadableFile(format!("inspection panicked: {}", e)))?
}

/// Transcribe an imported audio file with the currently selected model and ASR
/// settings, then save it to History.
#[tauri::command]
#[specta::specta]
pub async fn transcribe_audio_file(
    app: AppHandle,
    activity: State<'_, Arc<TranscriptionActivity>>,
    state: State<'_, Arc<FileTranscriptionState>>,
    history_manager: State<'_, Arc<HistoryManager>>,
    transcription_manager: State<'_, Arc<TranscriptionManager>>,
    path: String,
) -> Result<FileTranscriptionResult, FileTranscriptionError> {
    // Claim the shared engine up front so live dictation can't start underneath
    // us; `run_job` releases it on every exit path.
    let activity = Arc::clone(&activity);
    if !activity.try_begin_file() {
        return Err(FileTranscriptionError::Busy);
    }

    let state = Arc::clone(&state);
    let history_manager = Arc::clone(&history_manager);
    let transcription_manager = Arc::clone(&transcription_manager);

    // Register the job before the worker exists. Doing it inside `run_job` left
    // a window where a cancel arriving between spawn and registration found no
    // active job and was dropped — and with the job now starting on drop, that
    // is exactly when someone who grabbed the wrong file hits Cancel.
    let job_id = new_job_id();
    let cancel = state.begin(job_id.clone());

    tauri::async_runtime::spawn_blocking(move || {
        run_job(
            app,
            activity,
            state,
            history_manager,
            transcription_manager,
            path,
            job_id,
            cancel,
        )
    })
    .await
    .map_err(|e| FileTranscriptionError::TranscriptionFailed(format!("job panicked: {}", e)))?
}

/// Ask the running import to stop. Safe to call when nothing is running.
#[tauri::command]
#[specta::specta]
pub fn cancel_file_transcription(state: State<'_, Arc<FileTranscriptionState>>) {
    match state.request_cancel() {
        Some(job_id) => info!(
            "Cancellation requested for file transcription job {}",
            job_id
        ),
        None => info!("Cancellation requested but no file transcription is running"),
    }
}
