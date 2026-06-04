---
phase: 13-smart-modes-ui-translation-presets-i18n
plan: "06"
subsystem: shortcut-capture-ux
tags: [shortcut, keyboard, i18n, gap-closure, macos-fix]
dependency_graph:
  requires: [13-05]
  provides: [keydown-commit-capture, clear-binding-affordance, compact-chip-layout, shortcut-i18n-keys]
  affects: [SmartModeShortcutChip, all-20-locales, bindings.ts]
tech_stack:
  added: []
  patterns: [keydown-commit-pattern, inline-flex-chip-cluster, lucide-icon-button]
key_files:
  created: []
  modified:
    - src/components/settings/post-processing/SmartModeShortcutChip.tsx
    - src/bindings.ts
    - src/i18n/locales/en/translation.json
    - src/i18n/locales/ar/translation.json
    - src/i18n/locales/bg/translation.json
    - src/i18n/locales/cs/translation.json
    - src/i18n/locales/de/translation.json
    - src/i18n/locales/es/translation.json
    - src/i18n/locales/fr/translation.json
    - src/i18n/locales/he/translation.json
    - src/i18n/locales/it/translation.json
    - src/i18n/locales/ja/translation.json
    - src/i18n/locales/ko/translation.json
    - src/i18n/locales/pl/translation.json
    - src/i18n/locales/pt/translation.json
    - src/i18n/locales/ru/translation.json
    - src/i18n/locales/sv/translation.json
    - src/i18n/locales/tr/translation.json
    - src/i18n/locales/uk/translation.json
    - src/i18n/locales/vi/translation.json
    - src/i18n/locales/zh/translation.json
    - src/i18n/locales/zh-TW/translation.json
decisions:
  - "Commit on first non-modifier keydown (not keyup drain) — fixes macOS Cmd+key swallow where OS never fires keyup for non-modifier while Cmd held"
  - "heldModifiersRef (useRef, not useState) tracks modifier set to avoid stale closure issues in keydown/keyup handlers"
  - "clearSmartModeBinding manually added to bindings.ts following tauri-specta pattern (Rust command existed from 13-05 but specta hadn't regenerated the file yet)"
  - "English fallback values used verbatim in all 19 non-English locales for new shortcut.* keys — real translations deferred (Phase 11/13 precedent)"
metrics:
  duration_minutes: 15
  completed_date: "2026-06-04"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 22
---

# Phase 13 Plan 06: Shortcut Capture Fix + Clear Affordance + i18n Summary

**One-liner:** Keydown-commit shortcut capture (fixes macOS Cmd+1 swallow), clear binding affordance via lucide X button, compact inline-flex chip layout, 3 new i18n keys across 20 locales.

## Tasks Completed

| # | Task | Commit | Key Files |
|---|------|--------|-----------|
| 1 | Rewrite shortcut capture to commit on keydown + add clear affordance | 5dbe4b1 | SmartModeShortcutChip.tsx, bindings.ts |
| 2 | Propagate 3 new shortcut i18n keys to all 20 locales | 2bceb43 | en + 19 locale translation.json files |

## What Was Built

### Task 1: SmartModeShortcutChip rewrite (GAPs 3, 4, 6)

**GAP 3 (BLOCKER) — keydown-commit fix:** The old chip used the keyup-drain pattern (`updatedPressed.length === 0`), which breaks on macOS when Cmd (meta) is held because the OS swallows keyup events for non-modifier keys while a meta modifier is active. The combo was never committed. Fixed by switching to keydown-commit: when a non-modifier key is pressed while modifiers are held, the combo is built and committed immediately on that keydown event. Bare single keys (e.g. "1", "F1") also commit immediately on their keydown.

**GAP 4 (MAJOR) — clear affordance:** Added a lucide `<X>` button to the right of the bound chip, mirroring the SmartModeCard icon-button pattern (`p-0.5 rounded focus:outline-none`, `w-4 h-4 text-mid-gray/50 hover:text-red-400`). Calls `clearSmartModeBinding(modeId)` on click.

**GAP 6 (MINOR) — compact layout:** Chip+clear button wrapped in `<span className="inline-flex items-center gap-1">`. No `w-full` anywhere. Reuses existing app chip classes from GlobalShortcutInput: `px-2 py-1 text-sm font-medium` with bound/recording states. `dir="ltr"` retained on chip for RTL safety.

**bindings.ts:** `clearSmartModeBinding` was missing (Rust command from 13-05 hadn't triggered specta regeneration). Added manually following the tauri-specta pattern. The build process subsequently regenerated bindings.ts and confirmed the entry was correct.

### Task 2: i18n keys

Added `smartModes.shortcut` sub-object under the `card` entry in all 20 locales:
- `clear`: "Clear"
- `clearAriaLabel`: "Clear shortcut {{combo}}"
- `recording`: "Press keys…"

English values used verbatim as fallbacks in the 19 non-English locales (Phase 11/13 precedent).

## Verification

- `bun run lint` — exits 0
- `bun run check:translations` — exits 0 (all 19 languages complete)
- All acceptance criteria grep checks pass

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] bindings.ts missing clearSmartModeBinding**
- **Found during:** Task 1 read phase
- **Issue:** `clearSmartModeBinding` command existed in Rust (added in 13-05) but was not in `src/bindings.ts`. The plan anticipated this and instructed to add it if missing.
- **Fix:** Added `clearSmartModeBinding` entry manually following the tauri-specta pattern. The build process later regenerated bindings.ts (adding also `smartModeTemplates` from 13-05), confirming the entry.
- **Files modified:** src/bindings.ts
- **Commit:** 5dbe4b1, 6abd0c9

**2. [Rule 3 - Blocking] smartModeTemplates missing from committed bindings.ts**
- **Found during:** Post-task git status
- **Issue:** `smartModeTemplates` (added in 13-05) was also missing from the committed bindings.ts. Discovered when the build regenerated bindings.ts.
- **Fix:** Committed the regenerated bindings.ts.
- **Files modified:** src/bindings.ts
- **Commit:** 6abd0c9

## Self-Check: PASSED

Files exist:
- src/components/settings/post-processing/SmartModeShortcutChip.tsx — FOUND
- src/bindings.ts (clearSmartModeBinding at line 326) — FOUND
- src/i18n/locales/en/translation.json (shortcut sub-object) — FOUND

Commits exist:
- 5dbe4b1 feat(13-06): rewrite shortcut capture — FOUND
- 2bceb43 feat(13-06): propagate 3 new shortcut i18n keys — FOUND
- 6abd0c9 fix(13-06): add missing smartModeTemplates to bindings.ts — FOUND
