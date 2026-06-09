use crate::settings::get_settings;
use crate::TranscriptionCoordinator;
#[cfg(unix)]
use log::debug;
use log::warn;
use tauri::{AppHandle, Manager};

/// Legacy combined transcribe + post-process trigger id, predating Smart Modes.
const LEGACY_POST_PROCESS_ID: &str = "transcribe_with_post_process";

#[cfg(unix)]
use signal_hook::consts::{SIGUSR1, SIGUSR2};
#[cfg(unix)]
use signal_hook::iterator::Signals;
#[cfg(unix)]
use std::thread;

/// Resolve the effective coordinator binding id for an external (CLI / signal) trigger.
///
/// `LEGACY_POST_PROCESS_ID` predates Smart Modes. After the v1.2→v1.3 migration it is no
/// longer a registered binding, and its prompt source (`post_process_prompts`) is a frozen
/// snapshot that never tracks Smart Mode edits. Re-route it to the active Smart Mode so a
/// CLI/signal post-process trigger runs the same pipeline as pressing the active mode's
/// shortcut. All other ids (e.g. plain `transcribe`) pass through unchanged. Falls back to
/// the legacy id when no active Smart Mode is configured.
fn resolve_effective_binding(binding_id: &str, active_mode_id: Option<&str>) -> String {
    match (binding_id, active_mode_id) {
        (LEGACY_POST_PROCESS_ID, Some(id)) => format!("smart_mode_{id}"),
        _ => binding_id.to_string(),
    }
}

/// Send a transcription input to the coordinator.
/// Used by signal handlers, CLI flags, and any other external trigger.
pub fn send_transcription_input(app: &AppHandle, binding_id: &str, source: &str) {
    // Only read settings for the legacy post-process id; everything else passes through.
    let active_mode_id = if binding_id == LEGACY_POST_PROCESS_ID {
        get_settings(app).smart_mode_active_id
    } else {
        None
    };
    let effective_id = resolve_effective_binding(binding_id, active_mode_id.as_deref());

    if let Some(c) = app.try_state::<TranscriptionCoordinator>() {
        c.send_input(&effective_id, source, true, false);
    } else {
        warn!("TranscriptionCoordinator not initialized");
    }
}

#[cfg(unix)]
pub fn setup_signal_handler(app_handle: AppHandle, mut signals: Signals) {
    debug!("Signal handlers registered (SIGUSR1, SIGUSR2)");
    thread::spawn(move || {
        for sig in signals.forever() {
            let (binding_id, signal_name) = match sig {
                SIGUSR1 => ("transcribe_with_post_process", "SIGUSR1"),
                SIGUSR2 => ("transcribe", "SIGUSR2"),
                _ => continue,
            };
            debug!("Received {signal_name}");
            send_transcription_input(&app_handle, binding_id, signal_name);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_post_process_routes_to_active_smart_mode() {
        assert_eq!(
            resolve_effective_binding(LEGACY_POST_PROCESS_ID, Some("clean_up")),
            "smart_mode_clean_up"
        );
    }

    #[test]
    fn legacy_post_process_without_active_mode_keeps_legacy_id() {
        assert_eq!(
            resolve_effective_binding(LEGACY_POST_PROCESS_ID, None),
            LEGACY_POST_PROCESS_ID
        );
    }

    #[test]
    fn plain_transcribe_is_never_rewritten() {
        assert_eq!(
            resolve_effective_binding("transcribe", Some("clean_up")),
            "transcribe"
        );
        assert_eq!(resolve_effective_binding("transcribe", None), "transcribe");
    }
}
