---
phase: 13-smart-modes-ui-translation-presets-i18n
plan: "05"
subsystem: shortcut
tags: [rust, tauri, tauri-specta, smart-modes, keyboard-shortcuts, settings-migration]

# Dependency graph
requires:
  - phase: 12-smart-modes-data-layer
    provides: "set_smart_mode_binding, SmartMode types, migration v12->v13"
  - phase: 13-smart-modes-ui-translation-presets-i18n
    provides: "13-04: SmartModesSection mounted, legacy prompt UI removed"
provides:
  - "clear_smart_mode_binding Tauri command (removes binding entry + unregisters OS shortcut)"
  - "smart_mode_templates Tauri command (10-mode catalogue for create-picker)"
  - "transcribe_with_post_process retired as global shortcut (CLI/signal action preserved)"
  - "First-run seeds only Clean Up; full 10-template catalogue queryable"
affects: [13-06, 13-07]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Clear-is-delete: clearing a binding removes the entry entirely rather than setting empty string"
    - "Seed vs catalogue: default_smart_modes() = first-run seed (1 mode); smart_mode_templates() = full catalogue (10 modes)"

key-files:
  created: []
  modified:
    - src-tauri/src/shortcut/mod.rs
    - src-tauri/src/shortcut/tauri_impl.rs
    - src-tauri/src/shortcut/handy_keys.rs
    - src-tauri/src/settings.rs
    - src-tauri/src/lib.rs

key-decisions:
  - "clear_smart_mode_binding deletes the binding entry entirely (not empty-set) so init_shortcuts never re-registers it"
  - "transcribe_with_post_process retired only as a global shortcut; runtime action string preserved in coordinator/actions/signal/CLI"
  - "smart_mode_templates() is the 10-mode catalogue source; default_smart_modes() seeds only Clean Up (single-mode first-run)"

patterns-established:
  - "Binding clear = entry deletion: unregister OS shortcut if bound, then bindings.remove()"
  - "Template catalogue pattern: pub fn returns all N templates; seed fn filters to desired subset"

requirements-completed: [MODE-04, MODE-05, MODE-02]

# Metrics
duration: 4min
completed: 2026-06-04
---

# Phase 13 Plan 05: Backend Gap Closure — Shortcut Clear, Legacy Removal, Seeding Summary

**clear_smart_mode_binding command added, transcribe_with_post_process global shortcut retired from all four init/registration paths, and first-run seeding narrowed to Clean Up only with a 10-mode template catalogue exposed via smart_mode_templates command**

## Performance

- **Duration:** 4 min
- **Started:** 2026-06-04T19:30:01Z
- **Completed:** 2026-06-04T19:34:00Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments
- Added `clear_smart_mode_binding` Tauri command: unregisters OS shortcut if bound, removes binding entry so init never re-registers
- Retired `transcribe_with_post_process` as a global shortcut: removed default seed (bindings.insert + 4 cfg lets), removed 3 init/register guards; `change_post_process_enabled_setting` command kept but its shortcut bookkeeping branch removed; CLI/signal action paths untouched
- Renamed full 10-mode function to `pub smart_mode_templates()`, added new `default_smart_modes()` returning only Clean Up; migration also now seeds only Clean Up for upgraders; added `smart_mode_templates` Tauri command; updated all affected tests (103 pass)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add clear_smart_mode_binding command** - `c277a01` (feat)
2. **Task 2: Stop registering legacy transcribe_with_post_process global shortcut** - `6e7eefe` (fix)
3. **Task 3: Seed only Clean Up + expose template catalogue** - `5243d60` (feat)

## Files Created/Modified
- `src-tauri/src/shortcut/mod.rs` - Added clear_smart_mode_binding, smart_mode_templates commands; removed register guards; pruned change_post_process_enabled_setting shortcut branch
- `src-tauri/src/shortcut/tauri_impl.rs` - Removed transcribe_with_post_process init guard
- `src-tauri/src/shortcut/handy_keys.rs` - Removed transcribe_with_post_process init guard
- `src-tauri/src/settings.rs` - Renamed default_smart_modes->smart_mode_templates (pub), added new default_smart_modes (seed only), removed transcribe_with_post_process default seed; updated tests
- `src-tauri/src/lib.rs` - Registered clear_smart_mode_binding and smart_mode_templates in invoke_handler

## Decisions Made
- "clear" is a delete-the-entry operation, not an empty-set operation — binding entry removed entirely so init_shortcuts never sees it again
- transcribe_with_post_process retired only as a persisted global shortcut; the action string deliberately kept in coordinator/actions/signal/CLI for `--toggle-post-process` CLI flag
- Migration `migrate_settings_if_needed` starts with `default_smart_modes()` which now returns only Clean Up — upgraders get Clean Up + their preserved custom prompts, never the full 10

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None. cargo fmt reformatted two files after the edit pass; fmt --check and clippy both clean.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Backend commands `clearSmartModeBinding` and `smartModeTemplates` will be available in bindings.ts after next `bun run tauri dev` (tauri-specta regen)
- 13-06 (shortcut UX fixes) and 13-07 (create-picker / template catalogue UI) can now consume these commands
- No blockers

---
*Phase: 13-smart-modes-ui-translation-presets-i18n*
*Completed: 2026-06-04*
