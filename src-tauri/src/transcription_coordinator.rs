use crate::actions::ACTION_MAP;
use crate::managers::audio::AudioRecordingManager;
use log::{debug, error, warn};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

const DEBOUNCE: Duration = Duration::from_millis(30);

/// Commands processed sequentially by the coordinator thread.
enum Command {
    Input {
        binding_id: String,
        hotkey_string: String,
        is_pressed: bool,
        push_to_talk: bool,
    },
    Cancel {
        recording_was_active: bool,
    },
    ProcessingFinished,
}

/// Pipeline lifecycle, owned exclusively by the coordinator thread.
enum Stage {
    Idle,
    Recording(String), // binding_id
    Processing,
}

/// Which producer currently owns the transcription engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Owner {
    None,
    /// Live dictation: recording or the async transcribe-paste pipeline.
    Dictation,
    /// An imported-file transcription job.
    File,
}

/// Mutual exclusion between the two producers of transcription work.
///
/// Live dictation and imported-file jobs both end up in
/// `TranscriptionManager::transcribe`, and the surrounding lifecycle UI (tray
/// icon, overlay, history) assumes a single job at a time. This is the single
/// lock both sides check, so neither can start while the other holds the
/// engine. The dictation side is driven from the coordinator thread alongside
/// its `Stage`; the file side is claimed and released by the import command.
pub struct TranscriptionActivity {
    owner: Mutex<Owner>,
}

impl TranscriptionActivity {
    pub fn new() -> Self {
        Self {
            owner: Mutex::new(Owner::None),
        }
    }

    /// Claim the engine for an imported-file job.
    /// Returns `false` when live dictation (or another job) already owns it.
    pub fn try_begin_file(&self) -> bool {
        let mut owner = self.owner.lock().unwrap();
        if *owner == Owner::None {
            *owner = Owner::File;
            true
        } else {
            false
        }
    }

    pub fn end_file(&self) {
        let mut owner = self.owner.lock().unwrap();
        if *owner == Owner::File {
            *owner = Owner::None;
        }
    }

    pub fn is_file_active(&self) -> bool {
        *self.owner.lock().unwrap() == Owner::File
    }

    /// Claim the engine for live dictation. Returns `false` when a file job
    /// holds it, in which case the caller must not start recording.
    fn try_begin_dictation(&self) -> bool {
        let mut owner = self.owner.lock().unwrap();
        if *owner == Owner::None {
            *owner = Owner::Dictation;
            true
        } else {
            false
        }
    }

    fn end_dictation(&self) {
        let mut owner = self.owner.lock().unwrap();
        if *owner == Owner::Dictation {
            *owner = Owner::None;
        }
    }
}

impl Default for TranscriptionActivity {
    fn default() -> Self {
        Self::new()
    }
}

/// Cancellation of the live dictation pipeline once recording has stopped
/// (#27). The overlay's cancel button and the tray's Cancel stay available
/// while the audio is transcribed and post-processed; cancelling then discards
/// the result: nothing is pasted or saved, and a running post-processing
/// request is dropped. The transcription engine call itself is not interrupted:
/// it finishes in the background and its text is thrown away.
///
/// Each pipeline takes a [`CancelToken`] when it starts; [`cancel`] cancels
/// every token taken so far.
///
/// [`cancel`]: DictationCancel::cancel
pub struct DictationCancel {
    epoch: tokio::sync::watch::Sender<u64>,
    in_flight: Arc<AtomicUsize>,
}

impl DictationCancel {
    pub fn new() -> Self {
        Self {
            epoch: tokio::sync::watch::Sender::new(0),
            in_flight: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// A token for a pipeline starting now. The pipeline counts as in flight
    /// until the token and all its clones are dropped.
    pub fn token(&self) -> CancelToken {
        self.in_flight.fetch_add(1, Ordering::SeqCst);
        let rx = self.epoch.subscribe();
        let start = *rx.borrow();
        CancelToken {
            rx,
            start,
            _in_flight: Arc::new(InFlight(self.in_flight.clone())),
        }
    }

    /// Cancels every pipeline started so far. Returns whether one was running.
    pub fn cancel(&self) -> bool {
        self.epoch.send_modify(|e| *e += 1);
        self.in_flight.load(Ordering::SeqCst) > 0
    }
}

impl Default for DictationCancel {
    fn default() -> Self {
        Self::new()
    }
}

struct InFlight(Arc<AtomicUsize>);

impl Drop for InFlight {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

#[derive(Clone)]
pub struct CancelToken {
    rx: tokio::sync::watch::Receiver<u64>,
    start: u64,
    _in_flight: Arc<InFlight>,
}

impl CancelToken {
    pub fn is_cancelled(&self) -> bool {
        *self.rx.borrow() != self.start
    }

    /// Runs `work` unless the pipeline is cancelled first, in which case
    /// `work` is dropped (aborting it) and `None` is returned.
    pub async fn run<F: std::future::Future>(&self, work: F) -> Option<F::Output> {
        let mut rx = self.rx.clone();
        let start = self.start;
        tokio::select! {
            biased;
            // A closed channel (app shutting down) disables this branch.
            Ok(_) = rx.wait_for(|e| *e != start) => None,
            out = work => Some(out),
        }
    }
}

/// Serialises all transcription lifecycle events through a single thread
/// to eliminate race conditions between keyboard shortcuts, signals, and
/// the async transcribe-paste pipeline.
pub struct TranscriptionCoordinator {
    tx: Sender<Command>,
}

pub fn is_transcribe_binding(id: &str) -> bool {
    id == "transcribe" || id == "transcribe_with_post_process" || id.starts_with("smart_mode_")
}

impl TranscriptionCoordinator {
    pub fn new(app: AppHandle) -> Self {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut stage = Stage::Idle;
                let mut last_press: Option<Instant> = None;

                while let Ok(cmd) = rx.recv() {
                    match cmd {
                        Command::Input {
                            binding_id,
                            hotkey_string,
                            is_pressed,
                            push_to_talk,
                        } => {
                            // Debounce rapid-fire press events (key repeat / double-tap).
                            // Releases always pass through for push-to-talk.
                            if is_pressed {
                                let now = Instant::now();
                                if last_press.is_some_and(|t| now.duration_since(t) < DEBOUNCE) {
                                    debug!("Debounced press for '{binding_id}'");
                                    continue;
                                }
                                last_press = Some(now);
                            }

                            if push_to_talk {
                                if is_pressed && matches!(stage, Stage::Idle) {
                                    start(&app, &mut stage, &binding_id, &hotkey_string);
                                } else if !is_pressed
                                    && matches!(&stage, Stage::Recording(id) if id == &binding_id)
                                {
                                    stop(&app, &mut stage, &binding_id, &hotkey_string);
                                }
                            } else if is_pressed {
                                match &stage {
                                    Stage::Idle => {
                                        start(&app, &mut stage, &binding_id, &hotkey_string);
                                    }
                                    Stage::Recording(id) if id == &binding_id => {
                                        stop(&app, &mut stage, &binding_id, &hotkey_string);
                                    }
                                    _ => {
                                        debug!("Ignoring press for '{binding_id}': pipeline busy")
                                    }
                                }
                            }
                        }
                        Command::Cancel {
                            recording_was_active,
                        } => {
                            // Don't reset during processing — wait for the pipeline to finish.
                            if !matches!(stage, Stage::Processing)
                                && (recording_was_active || matches!(stage, Stage::Recording(_)))
                            {
                                stage = Stage::Idle;
                                release_dictation(&app);
                            }
                        }
                        Command::ProcessingFinished => {
                            stage = Stage::Idle;
                            release_dictation(&app);
                        }
                    }
                }
                debug!("Transcription coordinator exited");
            }));
            if let Err(e) = result {
                error!("Transcription coordinator panicked: {e:?}");
            }
        });

        Self { tx }
    }

    /// Send a keyboard/signal input event for a transcribe binding.
    /// For signal-based toggles, use `is_pressed: true` and `push_to_talk: false`.
    pub fn send_input(
        &self,
        binding_id: &str,
        hotkey_string: &str,
        is_pressed: bool,
        push_to_talk: bool,
    ) {
        if self
            .tx
            .send(Command::Input {
                binding_id: binding_id.to_string(),
                hotkey_string: hotkey_string.to_string(),
                is_pressed,
                push_to_talk,
            })
            .is_err()
        {
            warn!("Transcription coordinator channel closed");
        }
    }

    pub fn notify_cancel(&self, recording_was_active: bool) {
        if self
            .tx
            .send(Command::Cancel {
                recording_was_active,
            })
            .is_err()
        {
            warn!("Transcription coordinator channel closed");
        }
    }

    pub fn notify_processing_finished(&self) {
        if self.tx.send(Command::ProcessingFinished).is_err() {
            warn!("Transcription coordinator channel closed");
        }
    }
}

/// Release the dictation claim on the shared engine, if this app has one.
fn release_dictation(app: &AppHandle) {
    if let Some(activity) = app.try_state::<Arc<TranscriptionActivity>>() {
        activity.end_dictation();
    }
}

fn start(app: &AppHandle, stage: &mut Stage, binding_id: &str, hotkey_string: &str) {
    // An imported-file job owns the engine for its whole run; starting a
    // dictation on top of it would fight over the same model and overlay.
    if let Some(activity) = app.try_state::<Arc<TranscriptionActivity>>() {
        if !activity.try_begin_dictation() {
            warn!("Ignoring start for '{binding_id}': a file transcription is running");
            return;
        }
    }

    let action: Arc<dyn crate::actions::ShortcutAction> = if binding_id.starts_with("smart_mode_") {
        let mode_id = binding_id.strip_prefix("smart_mode_").unwrap().to_string();
        Arc::new(crate::actions::SmartModeAction { mode_id })
    } else {
        let Some(a) = ACTION_MAP.get(binding_id) else {
            warn!("No action in ACTION_MAP for '{binding_id}'");
            release_dictation(app);
            return;
        };
        Arc::clone(a)
    };
    action.start(app, binding_id, hotkey_string);
    if app
        .try_state::<Arc<AudioRecordingManager>>()
        .is_some_and(|a| a.is_recording())
    {
        *stage = Stage::Recording(binding_id.to_string());
    } else {
        debug!("Start for '{binding_id}' did not begin recording; staying idle");
        release_dictation(app);
    }
}

fn stop(app: &AppHandle, stage: &mut Stage, binding_id: &str, hotkey_string: &str) {
    let action: Arc<dyn crate::actions::ShortcutAction> = if binding_id.starts_with("smart_mode_") {
        let mode_id = binding_id.strip_prefix("smart_mode_").unwrap().to_string();
        Arc::new(crate::actions::SmartModeAction { mode_id })
    } else {
        let Some(a) = ACTION_MAP.get(binding_id) else {
            warn!("No action in ACTION_MAP for '{binding_id}'");
            return;
        };
        Arc::clone(a)
    };
    action.stop(app, binding_id, hotkey_string);
    *stage = Stage::Processing;
}

#[cfg(test)]
mod cancel_tests {
    //! #27: cancelling while the dictation is transcribed or post-processed
    //! must keep its text from being pasted.
    use super::DictationCancel;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn cancel_reaches_a_running_pipeline_only() {
        let cancel = DictationCancel::new();
        assert!(!cancel.cancel(), "nothing in flight yet");

        let token = cancel.token();
        assert!(
            !token.is_cancelled(),
            "an earlier cancel must not reach a later pipeline"
        );
        assert!(cancel.cancel(), "a pipeline was in flight");
        assert!(token.is_cancelled());
        assert!(token.clone().is_cancelled());

        drop(token);
        assert!(!cancel.cancel(), "the pipeline has ended");
        assert!(!cancel.token().is_cancelled());
    }

    #[test]
    fn in_flight_lasts_until_every_clone_is_dropped() {
        let cancel = DictationCancel::new();
        let token = cancel.token();
        let for_paste = token.clone();
        drop(token);
        assert!(cancel.cancel(), "the paste closure still holds a clone");
        drop(for_paste);
        assert!(!cancel.cancel());
    }

    #[tokio::test]
    async fn cancel_during_post_processing_drops_the_work() {
        let cancel = Arc::new(DictationCancel::new());
        let token = cancel.token();
        let pasted = Arc::new(AtomicBool::new(false));

        // A post-processing step that takes a while, then "pastes".
        let work = {
            let pasted = pasted.clone();
            async move {
                tokio::time::sleep(Duration::from_millis(300)).await;
                pasted.store(true, Ordering::SeqCst);
                "processed text"
            }
        };
        let canceller = {
            let cancel = cancel.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                cancel.cancel();
            })
        };

        assert_eq!(token.run(work).await, None);
        canceller.await.unwrap();
        tokio::time::sleep(Duration::from_millis(400)).await;
        assert!(
            !pasted.load(Ordering::SeqCst),
            "cancelled work must never paste"
        );
    }

    #[tokio::test]
    async fn cancel_before_post_processing_skips_it() {
        let cancel = DictationCancel::new();
        let token = cancel.token();
        cancel.cancel();
        let ran = AtomicBool::new(false);
        assert_eq!(
            token.run(async { ran.store(true, Ordering::SeqCst) }).await,
            None
        );
        assert!(!ran.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn uncancelled_work_completes() {
        let cancel = DictationCancel::new();
        let token = cancel.token();
        assert_eq!(token.run(async { 42 }).await, Some(42));
    }
}

#[cfg(test)]
mod coordinator_tests {
    use super::{is_transcribe_binding, TranscriptionActivity};

    #[test]
    fn file_job_and_dictation_cannot_hold_the_engine_together() {
        let activity = TranscriptionActivity::new();

        assert!(activity.try_begin_file(), "idle engine must be claimable");
        assert!(activity.is_file_active());
        assert!(
            !activity.try_begin_dictation(),
            "dictation must not start while a file job owns the engine"
        );

        activity.end_file();
        assert!(!activity.is_file_active());
        assert!(
            activity.try_begin_dictation(),
            "engine must be claimable again once the file job releases it"
        );
        assert!(
            !activity.try_begin_file(),
            "a file job must not start while dictation owns the engine"
        );

        activity.end_dictation();
        assert!(activity.try_begin_file());
    }

    #[test]
    fn releasing_a_claim_you_do_not_hold_is_a_no_op() {
        let activity = TranscriptionActivity::new();

        assert!(activity.try_begin_file());
        // A stray dictation release must not hand the engine away mid-import.
        activity.end_dictation();
        assert!(activity.is_file_active());
        assert!(!activity.try_begin_dictation());
    }

    #[test]
    fn is_transcribe_binding_smart_mode_prefix() {
        assert!(
            is_transcribe_binding("smart_mode_mode_abc"),
            "smart_mode_ prefix must be recognized"
        );
        assert!(
            is_transcribe_binding("smart_mode_"),
            "bare smart_mode_ prefix must be recognized"
        );
        assert!(
            is_transcribe_binding("transcribe"),
            "transcribe must still be recognized"
        );
        assert!(
            is_transcribe_binding("transcribe_with_post_process"),
            "transcribe_with_post_process must still be recognized"
        );
        assert!(
            !is_transcribe_binding("cancel"),
            "cancel must not be a transcribe binding"
        );
        assert!(
            !is_transcribe_binding("test"),
            "test must not be a transcribe binding"
        );
    }
}
