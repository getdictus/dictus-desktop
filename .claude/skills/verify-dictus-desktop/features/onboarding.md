# Onboarding

A first launch with no model on disk walks the user to a working setup. On macOS and Windows a permissions step comes first (microphone, and accessibility on macOS); Linux skips it. Then « To get started, choose a transcription model »: the recommended model (Parakeet V3, 456 MB) on top, every other model below by size. A click on a card downloads it, checks its SHA256, extracts it, selects it and opens the main window on General. A returning user (any model on disk) skips the model step.

- **ID:** `onboarding`
- **Desktop status:** present; Linux has the model step only
- **Tiers:** 1 (model step, proven), 2 (Windows microphone step), 3 (macOS permissions), 4 (first real dictation)
- **Screens:** `app.onboarding.permissions`, `app.onboarding.model-pick`, `app.onboarding.model-download`, then `app.settings.general` — see [screens.md](screens.md)

## Sub-features

- `onboarding.welcome`: — absent (no intro pages).
- `onboarding.language`: — absent (the interface language is in About).
- `onboarding.microphone`: macOS and Windows permission step « Permissions Required » with « Grant Permission »; skipped on Linux. — partial
- `onboarding.accessibility` _(desktop-only)_: macOS « Accessibility Access » with « Restart Dictus » hint. — present (macOS)
- `onboarding.model-download`: the model list, download with progress, « Verifying », « Extracting… », then selection. — present
- `onboarding.completion`: lands on General with the model in the footer (« Moonshine V2 Ti… » with a green dot). — present (no dedicated screen)
- `onboarding.resume`: a model on disk at launch skips the model step. — present
- `onboarding.keyboard-setup`, `onboarding.first-dictation`, `onboarding.apple-intelligence`, `onboarding.stop-standby`, `onboarding.whats-new`: — mobile-only / absent.

## How to get to it (user POV)

1. Install Dictus and open it with no model downloaded (fresh install, or an empty data dir).
2. macOS / Windows: grant the permissions. Linux: nothing to do.
3. Click a model card; wait for the download; the main window opens.

## Driving it with x.py

Preconditions: a run that printed `READY` from an empty isolated data dir (every tier-1 launch is one). `doctor.sh` → `DOCTOR OK`.

- **Picker.** `helpers/evidence.sh 10-onboarding-picker`. The capture shows « To get started, choose a transcription model », Parakeet V3 « Recommended », Moonshine V2 Tiny « 31.0 MB » second.
- **Choose Moonshine V2 Tiny.** `M=$(helpers/log.sh mark)`; `helpers/x.py click 340 320` (centre of the second card). Log: `Starting fresh download of model moonshine-tiny-streaming-en`. The other cards grey out.
- **Download.** `helpers/log.sh wait 'SHA256 verified for model moonshine-tiny-streaming-en' 180 "$M"`, then `Successfully downloaded model moonshine-tiny-streaming-en`. Capture in between shows « Extracting… » under a full bar.
- **Lands on General.** `settings_store.json` (isolated) has `selected_model = moonshine-tiny-streaming-en`; log `Successfully loaded transcription model: moonshine-tiny-streaming-en`.
- **Proof.** `helpers/journey-onboarding.sh` does all of the above and prints `JOURNEY onboarding PASS`; `12-onboarding-done.png` shows General with « Moonshine V2 Ti… » in the footer, `12-onboarding-done.state.txt` lists the model files (`encoder.ort`, `decoder_kv.ort`, …) and the setting, `12-onboarding-done.log` the log lines.

## Gotchas

- Never click Parakeet V3 on tier 1 unless the change is about it: 456 MB.
- The download comes from `blob.handy.computer` (the upstream Handy CDN): no network, no onboarding.
- The extracted archive contains macOS `._*` resource-fork files next to the model files; they are harmless.
- The log's timestamps are UTC; `report.txt` is local time.
- The macOS permission step needs tier 3; the Windows microphone step tier 2.
