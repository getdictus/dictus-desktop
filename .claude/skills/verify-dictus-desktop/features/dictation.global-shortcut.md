# Dictation from a global shortcut

The desktop's way to dictate: a global shortcut (default Ctrl+Space on Linux and Windows, Alt+Space on macOS) starts recording from any app, a second press (or release, in Push To Talk) stops it, the active model transcribes and the text is pasted into the focused app. A cancel shortcut drops the recording. The recording overlay shows the state. The tray icon and the CLI (`--toggle-transcription`, `--cancel`) are the other entry points.

- **ID:** `dictation.global-shortcut` _(desktop-only; the desktop counterpart of the mobile `dictation.keyboard` / `dictation.in-app`)_
- **Desktop status:** present
- **Tiers:** 1 (settings, shortcut editing, and a real dictation through the CLI and a virtual microphone: journey `dictation.corpus`), 3 (macOS paste and accessibility), 4 (real voice, paste feel)
- **Screens:** `app.settings.general`, `app.recording-overlay` (not captured) — see [screens.md](screens.md)

## Sub-features

- `dictation.start` / `dictation.stop`: the Transcribe Shortcut toggles recording. — present
- `dictation.cancel`: the Cancel Shortcut, active while recording. — present
- `dictation.push-to-talk` _(desktop-only)_: « Push To Talk » (on by default) records while the keys are held. — present
- `dictation.shortcut-edit` _(desktop-only)_: click the shortcut chip, « Press keys… », the new chord is saved; the ↻ button resets it. — present
- `dictation.status`: the recording overlay (« Overlay Position » in Advanced; off by default on Linux). — present
- `dictation.paste` _(desktop-only)_: « Paste Method », « Typing Tool », « Clipboard Handling », « Auto Submit » in Advanced → Output. — present
- `dictation.cli` _(desktop-only)_: `--toggle-transcription`, `--toggle-post-process`, `--cancel` on a running instance. — present
- `dictation.transcribe` _(desktop-only)_: the recording goes through VAD and the active model, the text lands in History. — present

## How to get to it (user POV)

1. Press Ctrl+Space (Alt+Space on macOS) anywhere, speak, release (Push To Talk) or press again.
2. Change the shortcut: General → « Transcribe Shortcut » chip.
3. From a terminal: `dictus --toggle-transcription`.

## Driving it with x.py

**Real dictation, journey `dictation.corpus`** (proven 2026-11-18 on `main`, see `../SKILL.md` → Dictation with the test-voices corpus). It drives `dictation.start`, `dictation.stop`, `dictation.cli` and `dictation.transcribe`; not the shortcut keys, not paste.

Preconditions: network for `gh` (private repo `getdictus/dictus-test-voices`) and, the first time, for the model cache; nothing else, `run.sh` launches with `--model parakeet-tdt-0.6b-v3 --mic`.

- **One clip, or every French clip.** `helpers/run.sh dictation --no-build -- fr-03-shopping` or `-- all-fr`: launch, Doctor, journey, Cleanup in one process.
- **Start.** `helpers/cli.sh --toggle-transcription` (a second `dictus` in the run's isolated environment, handed to the running one over the private D-Bus); log `TranscribeAction::start called`; `helpers/mic.py <scratch> link` connects the virtual source to the app's capture stream.
- **Speak.** `helpers/mic.py <scratch> play <clip.wav>` writes the clip into the virtual source in real time.
- **Stop.** `helpers/cli.sh --toggle-transcription` again; a new row in the isolated `history.db`.
- **Proof.** One line per clip, `JOURNEY dictation <id> PASS wer=… wer_sp=… … gate=… stop_to_text_ms=… transcribe_ms=…` (a `short` clip is gated on WER ≤ 0.25, every clip on a complete capture; the long code-switched clip reports its WER only, see `../SKILL.md`), the summary `JOURNEY dictation all-fr PASS 5/5 …`, `LEAK CHECK OK` in `report.txt`, and `20-dictation-ready.png` (General page, the seeded model in the footer, before any dictation).

**Settings and the shortcut.** Not yet run end to end (2026-11-17): coordinates come from the tier-1 captures in `../screens/`; the first run that drives this recipe confirms or fixes them.

Preconditions: `READY`, then `journey-onboarding.sh` → `PASS` (a model is needed). Log `Shortcuts initialized successfully`.

- **See the binding.** `helpers/nav.sh general`; `helpers/evidence.sh 20-shortcut-general`: « Transcribe Shortcut » shows « Ctrl + Space », « Push To Talk » on.
- **Edit the shortcut.** `helpers/x.py click 561 59` (the chip) → « Press keys… »; `helpers/x.py key ctrl+alt+d`; the chip shows the new chord and `settings_store.json` (isolated) changes its `bindings.transcribe`. Reset with `helpers/x.py click 634 59`.
- **Start a recording with the shortcut.** `helpers/x.py key ctrl+space` reaches the app's global shortcut on the Xvfb display (inferred from the X11 grab, not yet observed). Not driven yet: the journey above starts recordings through the CLI.
- **Proof.** Before/after captures of the chip and the `bindings` value in `<name>.state.txt`.

## Gotchas

- Without `--mic` the isolated session has no PipeWire: recording fails with no input device, and the microphone list shows « Default » only. With `--mic` the app records from ALSA `default`, which reaches the run's private PipeWire.
- Never capture the History page, the clipboard or the window after a corpus dictation: the transcript of a private clip would land in the evidence. The app logs the transcript at debug level in its own log (scratch dir, deleted by Cleanup); never copy that log into the run dir.
- The first clip of a run waits for the model to load (Parakeet V3: a few seconds, `MODEL LOADED (took Nms)` in `report.txt`), so its `stop_to_text_ms` is not inflated by the load.
- Never send the shortcut to a display other than the run's: on Pierre's desktop it would start his real Dictus.
- Paste needs a target app; there is none on the Xvfb display.
