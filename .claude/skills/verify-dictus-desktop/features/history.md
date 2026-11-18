# Transcription history

Every transcription is saved with its audio: the History page lists entries newest first, each with copy, save (star), re-transcribe and delete, an audio player, and « Open Recordings Folder ». Imported files (Transcribe File) appear as « Imported from <name> ». Advanced → History sets the limit and how long recordings are kept.

- **ID:** `history`
- **Desktop status:** present (free)
- **Tiers:** 1 (empty state; entries once a virtual microphone or a file journey exists), 4
- **Screens:** `app.settings.history` — see [screens.md](screens.md)

## Sub-features

- `history.list`: entries with text, time, audio player. — present
- `history.delete`: trash icon per entry. — present
- `history.copy` _(desktop-only)_: copy icon. — present
- `history.save` _(desktop-only)_: star to keep an entry past the retention limit. — present
- `history.retranscribe` _(desktop-only)_: ↻, re-runs the active model on the saved audio. — present
- `history.open-folder` _(desktop-only)_: « Open Recordings Folder ». — present
- `history.retention` _(desktop-only)_: Advanced → History limit and recording retention. — present
- `history.search`, `history.clear`, `history.detail`: — absent.

## How to get to it (user POV)

Sidebar → « History ».

## Driving it with x.py

Not yet run end to end (2026-11-17): coordinates come from the tier-1 captures in `../screens/`; the first run that drives this recipe confirms or fixes them.

Preconditions: `READY`, `journey-onboarding.sh` → `PASS`.

- **Empty state.** `helpers/nav.sh history`; `helpers/evidence.sh 40-history-empty`: « No transcriptions yet. Start recording to build your history! » and « Open Recordings Folder ».
- **With entries.** Needs a transcription: not drivable on tier 1 yet (no microphone; Transcribe File needs a file drop). Report as not driven with that precondition.
- **Proof.** The capture, and `history.db` in the isolated dir (never Pierre's).

## Gotchas

- The real `~/.local/share/com.dictus.desktop/history.db` is the file the fingerprint protects: a debug build run without the isolated HOME migrates it (v4 → v7, reported in PR #75).
- « Open Recordings Folder » calls the desktop's file manager: nothing opens on Xvfb.
