# Dictus Desktop feature map

Every user-facing feature of Dictus Desktop, one file each for the main ones, with **the same IDs as the iOS and Android maps** (`getdictus/dictus-ios` and `getdictus/dictus-android`, `.claude/skills/verify-dictus-*/features/`) wherever the feature exists on the desktop. An ID names what the user gets, never a module or a component. An ID never changes once published; a feature that goes away keeps its ID retired. A feature only the desktop has gets a new ID in the same format, marked _desktop-only_.

Screens are mapped in [screens.md](screens.md), captures in [`../screens/`](../screens/).

## Baseline preconditions

- Tier 1 (default): `helpers/launch.sh` printed `READY`, `helpers/doctor.sh` printed `DOCTOR OK`. The app runs from the extracted debug .deb on a private Xvfb, with an empty isolated HOME: it starts on onboarding, with no model.
- A journey that needs the main window first runs `helpers/journey-onboarding.sh` (Moonshine V2 Tiny, 31 MB, into the isolated dir), or launches with `--model <id>`, which seeds a model from the shared cache so the app starts on the main window.
- A journey that records launches with `--mic`: a private PipeWire with a virtual source, in the run's scratch dir only.
- English labels (`LANG=en_US.UTF-8`), window 680x570 at `+0+0`.
- Pierre's real data is never read or written; Doctor and Cleanup check its fingerprint.

## Driving conventions

- `helpers/x.py` is the only driver (XTEST on the run's display). Coordinates are window-relative and come from a fresh capture.
- `helpers/nav.sh <section>` opens a sidebar section; it is the only place the sidebar coordinates are written.
- Assert from a log mark: `M=$(helpers/log.sh mark)`, then `helpers/log.sh wait '<regex>' <s> "$M"`.
- The isolated session has no sound card. With `--mic` it has one virtual source, fed by `helpers/mic.py play`; nothing is ever played to a speaker, so sound feedback stays undrivable.
- No destructive command inside `bash -c`/`sh -c` strings or loops; one journey per run; Cleanup right after.

## Proof and skip reporting

- Capture the action and the resulting state: a screenshot before and after (`helpers/evidence.sh <NN>-<feature-id>-<step>`), the log line that proves the action, and the side effect (files in the isolated data dir, the setting in `settings_store.json`; `evidence.sh` records both in `<name>.state.txt`).
- An unreachable path is reported with the command attempted and the unmet precondition. A skipped entry point is never reported as verified through another path.
- Evidence lives in `/home/pierre/OpenBot/Shared/pilotage/dictus-desktop/preuves/verify-dictus-desktop/<run-id>/`, never in the repository. Only screen-map captures go in `../screens/`.

## Feature entry contract

Each feature file starts with an H1 title and one paragraph describing the user-visible behaviour, then a short metadata list (ID, desktop status, tiers, screens), then exactly four H2 in this order:

1. `Sub-features`: short IDs with one line each, ending with the desktop status (present / partial: what is missing / absent / mobile-only); desktop-only ones are marked _(desktop-only)_.
2. `How to get to it (user POV)`: every user entry point.
3. `Driving it with x.py`: starts with `Preconditions:`, then labelled bullets pairing each user action with an exact helper call and its observable result, ending with a **Proof.** bullet.
4. `Gotchas`: traps that waste or invalidate a run, including what tier 1 cannot show.

## Features

| ID                                                          | Feature                                                                             | Desktop status                                                | Drivable on tier 1                                                                                                                                                                                                                                                              |
| ----------------------------------------------------------- | ----------------------------------------------------------------------------------- | ------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`onboarding`](onboarding.md)                               | First run: model choice and download                                                | present (Linux: model step only)                              | **Yes, proven 2026-11-17** (`journey-onboarding.sh`)                                                                                                                                                                                                                            |
| [`dictation.global-shortcut`](dictation.global-shortcut.md) | Dictation from a global shortcut, anywhere                                          | present, _desktop-only_                                       | **Recording + transcription yes, proven 2026-11-18** (journey `dictation.corpus`, `journey-dictation.sh`, started by the CLI); **paste on X11 yes, proven 2026-11-24** (journey `dictation.paste`, `journey-paste.sh`); settings and shortcut editing yes; the shortcut keys no |
| [`models.transcription`](models.transcription.md)           | Transcription models                                                                | present                                                       | Yes (Moonshine V2 Tiny downloaded; Parakeet V3 seeded from the cache and loaded by `dictation.corpus`; Parakeet V3, Whisper Turbo and Moonshine Base loaded and timed by `bench.sh`)                                                                                            |
| [`smart-modes`](smart-modes.md)                             | Post Process and Smart Modes (LLM rewrite)                                          | partial: behind the Post Processing toggle                    | Settings yes; a rewrite needs a dictation                                                                                                                                                                                                                                       |
| [`history`](history.md)                                     | Transcription history                                                               | present                                                       | Empty state yes; entries written by `dictation.corpus` (read from `history.db`; the page is not captured: it would show private text)                                                                                                                                           |
| [`feedback.sounds-haptics`](feedback.sounds-haptics.md)     | Sound feedback                                                                      | partial: sounds only, no haptics                              | Settings yes; no sound heard                                                                                                                                                                                                                                                    |
| `transcribe-file`                                           | Transcribe an audio file                                                            | present, _desktop-only_                                       | Screen yes; dropping a file needs a drag source (no recipe yet)                                                                                                                                                                                                                 |
| `vocabulary`                                                | Custom words                                                                        | present (Advanced → Custom Words)                             | Yes, not yet written as a recipe                                                                                                                                                                                                                                                |
| `diagnostics.logs`                                          | Log level, log transcripts (off by default, #83), log directory, debug page         | present (Debug page with Ctrl+Shift+D, About → Log Directory) | Yes (`x.py key ctrl+shift+d`, `nav.sh debug`)                                                                                                                                                                                                                                   |
| `appearance`                                                | Interface language                                                                  | partial: language only, the theme follows the system          | Yes (About → Application Language)                                                                                                                                                                                                                                              |
| `dictation.cli`                                             | `--toggle-transcription`, `--toggle-post-process`, `--cancel` on a running instance | present, _desktop-only_                                       | `--toggle-transcription` yes (`helpers/cli.sh`, inside `dictation.corpus`); the other flags not yet                                                                                                                                                                             |
| `voice-notes`                                               | Voice notes                                                                         | absent                                                        | —                                                                                                                                                                                                                                                                               |
| `pro.subscription`                                          | Pro purchase                                                                        | absent (Dictus Desktop is free)                               | —                                                                                                                                                                                                                                                                               |

Not in the map as files, on purpose: the mobile-only IDs (`dictation.keyboard`, `dictation.in-app`, `dictation.return-to-app`, `polish`, `live-activity`, the `kbd.*` screens). A desktop dictation always starts from a shortcut, the tray or the CLI, so `dictation.in-app` maps to `dictation.global-shortcut`.

TODO: write recipe files for `transcribe-file`, `vocabulary`, `diagnostics.logs`, `appearance` and `dictation.cli` the first time a PR touches them. Sound feedback stays undrivable on tier 1: the virtual device is a source, there is no sink.
