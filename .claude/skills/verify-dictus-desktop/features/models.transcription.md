# Transcription models

The Models page lists the downloaded models (the active one marked « Active ») and those available to download, with accuracy and speed bars, size, languages (« English Only », « Multi-language », « Translate to English »), a language filter, and Delete. Clicking a downloaded model makes it active; clicking an available one downloads it. The footer shows the active model and lets the user switch.

- **ID:** `models.transcription`
- **Desktop status:** present
- **Tiers:** 1 (catalogue, download of Moonshine V2 Tiny, select, delete; load time, WER and latency per model with `helpers/bench.sh`, see `../SKILL.md` → Dictation benchmark), 4 (quality, speed on real voice)
- **Screens:** `app.settings.models`, `app.settings.general` (footer) — see [screens.md](screens.md)

## Sub-features

- `models.catalogue`: « Downloaded Models » and « Available to Download ». — present
- `models.recommended`: « Recommended » badge (onboarding only). — present
- `models.download`: progress, verifying, extracting. — present
- `models.select`: click a downloaded card, or the footer menu. — present
- `models.delete`: « Delete » on a downloaded card. — present
- `models.languages`: « All Languages » filter. — present
- `models.preparation`: the model loads on selection (`Successfully loaded transcription model`). — present
- `models.custom` _(desktop-only)_: models not in the catalogue show « Not officially supported ». — present

## How to get to it (user POV)

1. Sidebar → « Models ».
2. Footer → the model name → pick another.

## Driving it with x.py

Not yet run end to end (2026-11-17): coordinates come from the tier-1 captures in `../screens/`; the first run that drives this recipe confirms or fixes them.

Preconditions: `READY`, `journey-onboarding.sh` → `PASS` (Moonshine V2 Tiny active).

- **Open.** `helpers/nav.sh models`; `helpers/evidence.sh 30-models`: « Moonshine V2 Tiny » with « Active » under « Downloaded Models ».
- **Filter.** Click « All Languages » (`helpers/x.py click 574 114`) and capture the menu; close with `helpers/x.py key Escape`.
- **Delete.** Not on the active model. To prove delete, download a second small model first (Moonshine Base, 55 MB) only when the change is about models.
- **Proof.** Capture plus `<name>.state.txt` listing `models/` in the isolated dir.

## Gotchas

- Every download is real and comes from `blob.handy.computer`. Stay with Moonshine V2 Tiny unless the change needs another model.
- Whisper models use whisper.cpp with Vulkan on Linux. On this machine the user `pierre` cannot open `/dev/dri/renderD128` (group `render`), so the app logs `ggml_vulkan: No devices found.` and Whisper runs on the CPU (measured 2026-11-18 by `bench.sh`). A Whisper timing from tier 1 is a CPU timing.
- With several models on disk and none selected, the app auto-selects the first downloaded one of a `HashMap`: not deterministic. Seed one model per launch to choose it (`launch.sh --model`).
