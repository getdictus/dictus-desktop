# Dictation from a global shortcut

The desktop's way to dictate: a global shortcut (default Ctrl+Space on Linux and Windows, Alt+Space on macOS) starts recording from any app, a second press (or release, in Push To Talk) stops it, the active model transcribes and the text is pasted into the focused app. A cancel shortcut drops the recording. The recording overlay shows the state. The tray icon and the CLI (`--toggle-transcription`, `--cancel`) are the other entry points.

- **ID:** `dictation.global-shortcut` _(desktop-only; the desktop counterpart of the mobile `dictation.keyboard` / `dictation.in-app`)_
- **Desktop status:** present
- **Tiers:** 1 (settings and shortcut editing only), 3 (macOS paste and accessibility), 4 (real voice, paste feel)
- **Screens:** `app.settings.general`, `app.recording-overlay` (not captured) — see [screens.md](screens.md)

## Sub-features

- `dictation.start` / `dictation.stop`: the Transcribe Shortcut toggles recording. — present
- `dictation.cancel`: the Cancel Shortcut, active while recording. — present
- `dictation.push-to-talk` _(desktop-only)_: « Push To Talk » (on by default) records while the keys are held. — present
- `dictation.shortcut-edit` _(desktop-only)_: click the shortcut chip, « Press keys… », the new chord is saved; the ↻ button resets it. — present
- `dictation.status`: the recording overlay (« Overlay Position » in Advanced; off by default on Linux). — present
- `dictation.paste` _(desktop-only)_: « Paste Method », « Typing Tool », « Clipboard Handling », « Auto Submit » in Advanced → Output. — present
- `dictation.cli` _(desktop-only)_: `--toggle-transcription`, `--toggle-post-process`, `--cancel` on a running instance. — present

## How to get to it (user POV)

1. Press Ctrl+Space (Alt+Space on macOS) anywhere, speak, release (Push To Talk) or press again.
2. Change the shortcut: General → « Transcribe Shortcut » chip.
3. From a terminal: `dictus --toggle-transcription`.

## Driving it with x.py

Not yet run end to end (2026-11-17): coordinates come from the tier-1 captures in `../screens/`; the first run that drives this recipe confirms or fixes them.

Preconditions: `READY`, then `journey-onboarding.sh` → `PASS` (a model is needed). Log `Shortcuts initialized successfully`.

- **See the binding.** `helpers/nav.sh general`; `helpers/evidence.sh 20-shortcut-general`: « Transcribe Shortcut » shows « Ctrl + Space », « Push To Talk » on.
- **Edit the shortcut.** `helpers/x.py click 561 59` (the chip) → « Press keys… »; `helpers/x.py key ctrl+alt+d`; the chip shows the new chord and `settings_store.json` (isolated) changes its `bindings.transcribe`. Reset with `helpers/x.py click 634 59`.
- **Start a recording.** `helpers/x.py key ctrl+space` reaches the app's global shortcut on the Xvfb display (inferred from the X11 grab, not yet observed). With no audio device in the isolated session the recording cannot capture sound: report `dictation.start` as not driven on tier 1 until a virtual microphone exists.
- **Proof.** Before/after captures of the chip and the `bindings` value in `<name>.state.txt`; for recording, tier 4.

## Gotchas

- The isolated session has no PulseAudio/PipeWire: the microphone list shows « Default » only.
- Never send the shortcut to a display other than the run's: on Pierre's desktop it would start his real Dictus.
- Paste needs a target app; there is none on the Xvfb display.
