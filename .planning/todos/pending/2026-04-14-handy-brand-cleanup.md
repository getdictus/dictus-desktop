---
created: 2026-04-14
title: Handy → Dictus brand cleanup (remaining leaks)
area: general
files:
  - src-tauri/src/actions.rs:538
  - src-tauri/src/managers/history.rs:689
  - src-tauri/src/tray.rs:273
  - src-tauri/src/portable.rs:30
  - src-tauri/src/portable.rs:98
  - src/components/settings/debug/DebugPaths.tsx:29-46
---

## Problem

The handy.log → dictus.log rename (committed on branch `upstream/sync-2026-04-14`) fixed the logger. But a review during Phase 5 Sync #1 surfaced several other places where the Handy brand still leaks into user-visible surfaces:

1. **Recording file naming** — `actions.rs:538`, `history.rs:689`, `tray.rs:273` all produce WAV files named `handy-{timestamp}.wav`. Users see these filenames when exporting recordings or browsing history.
2. **Portable mode magic string** — `portable.rs:30` and `portable.rs:98` use the literal string `"Handy Portable Mode"` to detect/label portable installs. User-facing + affects portable-mode detection logic.
3. **Windows user-data path display** — `DebugPaths.tsx:29-46` displays `%APPDATA%/handy` to users in the debug settings panel. Misleading: the actual path on disk may or may not still be `handy` depending on underlying config.

**DO NOT TOUCH** — false positives that look similar but must stay:
- `src-tauri/src/shortcut/handy_keys.rs` and the `handy_keys` module name — this is the name of the upstream `handy-keys` **crate** (external library dep). Renaming would break the build.

## Solution

TBD — should be a dedicated rebrand phase (not tacked onto a sync PR). Scope each item carefully:

1. **Recording files** — decide naming convention (`dictus-{timestamp}.wav`?) and whether to migrate existing history entries. History DB schema may reference old filenames.
2. **Portable mode** — change magic string, but verify no existing portable installs break. Likely needs a fallback check for the old string during a migration window.
3. **Debug paths display** — update display string AND verify actual on-disk path. May require data-dir migration (bigger scope than a cosmetic fix).

Each change should extend `verify-sync.sh` with a corresponding check so future upstream syncs don't silently reintroduce the old naming.

Also worth doing: a broader `grep -rn -i 'handy' src/ src-tauri/src/ --exclude-dir=target` to catch anything else before a formal rebrand phase. Exclude `handy_keys` / `handy-keys` hits (external crate).
