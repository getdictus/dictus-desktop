# Post Process and Smart Modes

Post-processing rewrites a transcription with a language model before it is pasted: Smart Modes (prompt templates with their own shortcut chips), a local model library (GGUF models, drop zone) or an OpenAI-compatible API (provider, base URL, API key, model, « Test connection »). The « Post Process » sidebar section appears only once Advanced → Post Processing is on. A dedicated hotkey (`transcribe_with_post_process`, CLI `--toggle-post-process`) dictates through the active Smart Mode.

- **ID:** `smart-modes` (the iOS ID; the desktop calls it Post Process)
- **Desktop status:** partial: hidden until the Advanced toggle is on (PR #64 changes this)
- **Tiers:** 1 (settings, toggle, mode list), 4 (rewrite quality on real dictation)
- **Screens:** `app.settings.advanced`, `app.settings.post-process` (not captured yet) — see [screens.md](screens.md)

## Sub-features

- `smart-modes.toggle`: Advanced → « Post Processing » group toggle; adds « Post Process » to the sidebar. — present
- `smart-modes.list`: the Smart Modes section, with templates (`SmartModeTemplatePicker`). — present
- `smart-modes.shortcut` _(desktop-only)_: a shortcut chip per mode. — present
- `smart-modes.local-model` _(desktop-only)_: « Local Model Library », custom GGUF drop zone. — present
- `smart-modes.api` _(desktop-only)_: « API (OpenAI Compatible) », « Test connection ». — present
- `smart-modes.fan`, `smart-modes.pin`: — mobile-only.

## How to get to it (user POV)

1. Sidebar → « Advanced » → « Post Processing » → toggle on.
2. Sidebar → « Post Process ».

## Driving it with x.py

Not yet run end to end (2026-11-17): coordinates come from the tier-1 captures in `../screens/`; the first run that drives this recipe confirms or fixes them.

Preconditions: `READY`, `journey-onboarding.sh` → `PASS`.

- **Turn it on.** `helpers/nav.sh advanced`; scroll to « Post Processing » (`helpers/x.py scroll down 5 --at 415 300`), capture, click its toggle (read its position from the capture). `settings_store.json` (isolated) → `post_process_enabled = true` in `<name>.state.txt`.
- **Open the section.** `helpers/nav.sh post-process` (row 6, About moves to 362: `DICTUS_ABOUT_Y=362`). Capture with `helpers/screen.sh app.settings.post-process` when the screen changes.
- **Rewrite.** Needs a dictation (no microphone on tier 1) and a model or an API key: tier 4. Never type a real API key into a test run.
- **Proof.** The toggle before/after, the setting, the section capture.

## Gotchas

- Not yet driven by this skill: positions below the fold must be read from a capture after scrolling.
- Downloading a local LLM is large (hundreds of MB): only when the change needs it.
