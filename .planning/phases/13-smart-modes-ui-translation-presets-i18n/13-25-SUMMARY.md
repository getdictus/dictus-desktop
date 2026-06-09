---
phase: 13-smart-modes-ui-translation-presets-i18n
plan: 25
subsystem: shortcut-conflict-localization
tags: [G16, gap-closure, i18n, localization, rust, typescript, shortcut]
requirements: [MODE-04, MODE-05, L10N-01]

dependency_graph:
  requires: [13-20, 13-21]
  provides: [G16-closed]
  affects:
    - src-tauri/src/shortcut/mod.rs
    - src/components/settings/post-processing/SmartModeCard.tsx
    - src/components/settings/post-processing/SmartModeShortcutChip.tsx

tech_stack:
  added: []
  patterns:
    - shared-localization-helper
    - payload-id-before-name
    - seeded-and-pristine-localization-rule

key_files:
  created: []
  modified:
    - src-tauri/src/shortcut/mod.rs
    - src/components/settings/post-processing/SmartModeCard.tsx
    - src/components/settings/post-processing/SmartModeShortcutChip.tsx

decisions:
  - SHORTCUT_CONFLICT payload now carries binding id in field[2], stored name in field[3], base in field[4] — field order locked by unit test
  - localizeSmartModeName is the single source of truth for seeded-and-pristine localization rule; shared between SmartModeCard and SmartModeShortcutChip
  - Core ids (transcribe/cancel) fall back to stored name verbatim — known and documented limitation

metrics:
  duration: 145s
  completed_date: "2026-06-09"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 3
---

# Phase 13 Plan 25: G16 Shortcut Conflict Localized Mode Name Summary

One-liner: Carried conflicting binding id in SHORTCUT_CONFLICT payload and added shared `localizeSmartModeName` helper so conflict messages interpolate the localized mode label (e.g. "Nettoyage" under FR) instead of the raw English seed name.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Carry conflicting binding id in SHORTCUT_CONFLICT payload (backend) | b4a679f | src-tauri/src/shortcut/mod.rs |
| 2 | Factor shared localizeSmartModeName helper and resolve localized name in chip | b5c54b2 | SmartModeCard.tsx, SmartModeShortcutChip.tsx |

## What Was Built

**[G16] Root cause fix:** The shortcut conflict message was interpolating the raw English seed mode name (e.g. "Clean Up") instead of the localized label (e.g. "Nettoyage" in FR). Two changes closed the gap:

### Backend (Task 1): New payload field order

Changed `set_smart_mode_binding` in `mod.rs` to carry the conflicting binding's stable id in field[2] BEFORE the stored name in field[3]:

- `SHORTCUT_CONFLICT|exact_duplicate|<id>|<name>` (4 fields, was 3)
- `SHORTCUT_CONFLICT|base_overlap|<id>|<name>|<base>` (5 fields, was 4)

Added unit test `conflict_payload_carries_binding_id_before_name` that asserts the exact format strings for both conflict kinds — field order is load-bearing and locked.

### Frontend (Task 2): Shared helper + chip resolution

1. **SmartModeCard.tsx**: Added `export function localizeSmartModeName(modeId, storedName, t)` — the single source of truth for the seeded-and-pristine rule. Refactored `displayName` to delegate (no behavioral change for the card).

2. **SmartModeShortcutChip.tsx**: Imported `localizeSmartModeName`; rewrote `localizeBindingError` to:
   - Parse the new 4/5-field payload format
   - Resolve the localized label for `smart_mode_*` ids via the shared helper (strips `smart_mode_` prefix → looks up `SEEDED_MODE_ID_TO_I18N_KEY`)
   - Fall back to stored name for core binding ids (transcribe/cancel — documented limitation)
   - Pass the localized `displayName` into both `shortcutConflict` and `shortcutConflictBase` t() calls

No new i18n keys added — `shortcutConflict` / `shortcutConflictBase` already existed in all 20 locales; only the interpolated `{{name}}` value changes.

## Deviations from Plan

None — plan executed exactly as written.

## Verification

- `cd src-tauri && cargo test --lib conflict_payload_carries_binding_id_before_name` → PASS (1/1)
- `cd src-tauri && cargo fmt --check` → PASS (clean)
- `bun run lint` → PASS (exit 0)
- [G16] Backend payload: `grep "SHORTCUT_CONFLICT|exact_duplicate|{}|{}"` and `grep "SHORTCUT_CONFLICT|base_overlap|{}|{}|{}"` both match new id-before-name format
- [G16] Frontend chip: `localizeSmartModeName` imported; `id.startsWith("smart_mode_")` resolves localized label before interpolation
- [G16-no-regress] Distinct localized sentences for exact-duplicate vs base-overlap preserved (shortcutConflict vs shortcutConflictBase)

## Self-Check: PASSED

- mod.rs: FOUND
- SmartModeCard.tsx: FOUND
- SmartModeShortcutChip.tsx: FOUND
- 13-25-SUMMARY.md: FOUND
- commit b4a679f: FOUND
- commit b5c54b2: FOUND
