# Sound feedback

Dictus can play a sound when recording starts and stops: « Audio Feedback » (off by default) with « Output Device » and « Volume » on General, and « Sound Theme » (Marimba, Pop, …) with a ▷ preview on the Debug page. There is no haptic feedback on the desktop.

- **ID:** `feedback.sounds-haptics` (the iOS/Android ID; sounds only)
- **Desktop status:** partial: sounds only, theme picker on the Debug page
- **Tiers:** 1 (settings), 4 (hearing the sounds)
- **Screens:** `app.settings.general`, `app.settings.debug` — see [screens.md](screens.md)

## Sub-features

- `feedback.sounds`: « Audio Feedback » toggle; the Output Device and Volume rows are disabled while it is off. — present
- `feedback.volume`: « Volume » slider. — present
- `feedback.output-device` _(desktop-only)_: « Output Device ». — present
- `feedback.sound-theme`: « Sound Theme » + ▷ preview (Debug page). — present
- `feedback.mute-while-recording` _(desktop-only)_: « Mute While Recording ». — present
- `feedback.haptics`: — mobile-only.

## How to get to it (user POV)

1. Sidebar → « General » → « Sound ».
2. Ctrl+Shift+D → « Debug » → « Sound Theme ».

## Driving it with x.py

Not yet run end to end (2026-11-17): coordinates come from the tier-1 captures in `../screens/`; the first run that drives this recipe confirms or fixes them.

Preconditions: `READY`, `journey-onboarding.sh` → `PASS`.

- **Turn it on.** `helpers/nav.sh general`; capture; `helpers/x.py click 628 284` (Audio Feedback toggle); capture: Output Device and Volume become enabled; `<name>.state.txt` shows `audio_feedback = true`.
- **Theme.** `helpers/x.py key ctrl+shift+d`; `helpers/nav.sh debug`; capture « Sound Theme / Marimba ».
- **Hear it.** No audio device on tier 1: tier 4.
- **Proof.** Before/after captures and the setting.

## Gotchas

- The debug shortcut toggles `debug_mode` in the isolated settings; it never reaches Pierre's.
- Volume and output device are greyed while Audio Feedback is off: capture the « on » state before asserting them.
