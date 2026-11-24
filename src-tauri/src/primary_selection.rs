//! The X11 PRIMARY selection: the text last highlighted, pasted with a middle
//! click. Shift+Insert reads it instead of the clipboard in terminals (xterm,
//! VTE, kitty, Alacritty), while GUI apps read the clipboard, so a Shift+Insert
//! paste puts the text on both.

use arboard::{ClearExtLinux, Clipboard, GetExtLinux, LinuxClipboardKind, SetExtLinux};
use log::warn;
use std::sync::Mutex;

/// arboard serves an X11 selection only while a `Clipboard` lives, so one is
/// kept for the life of the process.
static ARBOARD: Mutex<Option<Clipboard>> = Mutex::new(None);

fn with_arboard<T>(
    f: impl FnOnce(&mut Clipboard) -> Result<T, arboard::Error>,
) -> Result<T, String> {
    let mut guard = ARBOARD.lock().unwrap_or_else(|e| e.into_inner());
    if guard.is_none() {
        *guard = Some(Clipboard::new().map_err(|e| format!("Failed to open clipboard: {e}"))?);
    }
    match guard.as_mut() {
        Some(clipboard) => f(clipboard).map_err(|e| e.to_string()),
        None => Err("Clipboard not initialized".into()),
    }
}

/// The current PRIMARY text, or `None` when it is empty, not text, or unreadable.
pub fn read() -> Option<String> {
    with_arboard(|c| c.get().clipboard(LinuxClipboardKind::Primary).text())
        .ok()
        .filter(|t| !t.is_empty())
}

pub fn write(text: &str) -> Result<(), String> {
    with_arboard(|c| c.set().clipboard(LinuxClipboardKind::Primary).text(text))
}

/// Puts back what [`read`] returned, or empties PRIMARY if there was nothing.
pub fn restore(saved: Option<String>) {
    let result = match saved {
        Some(text) => write(&text),
        None => with_arboard(|c| c.clear_with().clipboard(LinuxClipboardKind::Primary)),
    };
    if let Err(e) = result {
        warn!("Failed to restore the primary selection: {e}");
    }
}
