# Sound feedback

Dictus plays a start sound when recording starts, at the key press with the overlay, and an end sound once the text has been inserted. On General › Sound: « Audio Feedback » (on by default for new installs since #73), « Sound Pack » (Wood, Synth, Minimal, each with a ▷ preview that plays its start then end sound), « Output Device » and « Volume » (50 % by default). The Debug page keeps « Sound Theme », a dropdown with every theme, including the legacy Marimba and Pop and, when its files exist, Custom. There is no haptic feedback on the desktop.

- **ID:** `feedback.sounds-haptics` (the iOS/Android ID; sounds only)
- **Desktop status:** partial: sounds only
- **Tiers:** 1 (settings), 3 (macOS playback), 4 (hearing the sounds)
- **Screens:** `app.settings.general`, `app.settings.debug` — see [screens.md](screens.md)

## Sub-features

- `feedback.sounds`: « Audio Feedback » toggle; Sound Pack, Output Device and Volume are disabled while it is off. — present
- `feedback.sound-pack` _(desktop-only)_: « Sound Pack » with a ▷ preview per pack. Marimba, Pop or Custom appear in it only while selected (existing installs). — present
- `feedback.volume`: « Volume » slider. — present
- `feedback.output-device` _(desktop-only)_: « Output Device ». — present
- `feedback.sound-theme` _(desktop-only)_: « Sound Theme » + ▷ (Debug page), every theme. — present
- `feedback.mute-while-recording` _(desktop-only)_: « Mute While Recording ». — present
- `feedback.haptics`: — mobile-only.

## Timing

- Start: the sound starts on its own thread at the press; the microphone opens at the same time. Microphone input is replaced by silence until the sound has finished plus 150 ms (`START_SOUND_OUTPUT_LATENCY` in `actions.rs`, `InputMask` in `recorder.rs`), so the sound never reaches VAD or the model. The app log has `Played Start sound of <Theme> (<path>, <n> ms, volume <v>)`.
- End: played after `utils::paste` returns `Ok`, never on an empty transcript or a failed paste. Log line `Played End sound of …`.

## How to get to it (user POV)

1. Sidebar → « General » → « Sound ».
2. Ctrl+Shift+D → « Debug » → « Sound Theme ».

## Driving it with x.py

Not yet run end to end: coordinates come from the tier-1 captures in `../screens/`; the first run that drives this recipe confirms or fixes them.

Preconditions: `READY`, `journey-onboarding.sh` → `PASS`.

- **Defaults.** `helpers/nav.sh general`; capture: Audio Feedback on, Sound Pack « Wood » selected, Volume 50 %; `<name>.state.txt` shows `audio_feedback = true`, `sound_theme = bois`.
- **Pick a pack.** Click « Synth » in Sound Pack; capture; the state shows `sound_theme = synthe`.
- **Turn it off.** Click the Audio Feedback toggle; capture: Sound Pack, Output Device and Volume greyed.
- **Hear it.** No audio device on tier 1: the log shows `Failed to play sound` for a preview. Tier 3 or 4.
- **Proof.** Before/after captures and the setting.

## Gotchas

- The debug shortcut toggles `debug_mode` in the isolated settings; it never reaches Pierre's.
- Sound Pack, Volume and Output Device are greyed while Audio Feedback is off: capture the « on » state before asserting them.
- An existing settings file keeps its `audio_feedback`, `audio_feedback_volume` and `sound_theme`. A file without the last two gets 1.0 and Marimba (what it played before), not the new-install values.
