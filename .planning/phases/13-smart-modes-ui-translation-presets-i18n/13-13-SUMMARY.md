---
phase: 13-smart-modes-ui-translation-presets-i18n
plan: 13
subsystem: shortcuts
tags: [rust, tauri, smart-modes, shortcuts, collision-detection, tdd]

# Dependency graph
requires:
  - phase: 13-smart-modes-ui-translation-presets-i18n
    plan: 12
    provides: suspend/resume all shortcuts on shortcut chip record start

provides:
  - find_conflicting_binding pure-logic helper (pub fn, shortcut/mod.rs)
  - Collision gate in set_smart_mode_binding returning BindingResponse{success:false} on duplicate combo
  - 5 unit tests covering all collision edge cases

affects:
  - SmartModeShortcutChip (frontend): existing inline conflict UI now fires at bind time
  - resume_all_shortcuts: can no longer hit "Hotkey already registered" for cross-mode collision
  - UAT test 7 / [B5]: now resolved

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Pre-delegation collision gate: check for conflicts at command layer before delegating to change_binding"
    - "Pure-logic helper extracted for unit-testability (no AppHandle needed)"
    - "TDD flow: 5 failing tests first, then implementation, then wire-in"

key-files:
  created: []
  modified:
    - src-tauri/src/shortcut/mod.rs

key-decisions:
  - "[13-13] find_conflicting_binding compares raw stored combo strings — no new normalization layer; frontend always sends canonical combo"
  - "[13-13] Collision gate scoped to set_smart_mode_binding only — change_binding left unchanged to preserve non-smart-mode shortcut editor semantics"
  - "[13-13] other_id != binding_id guard ensures idempotent same-mode re-bind still succeeds"

patterns-established:
  - "Collision check reads settings BEFORE the entry-insertion block so mode's own empty placeholder cannot cause false self-collision"

requirements-completed: [MODE-04, MODE-05]

# Metrics
duration: 2min
completed: 2026-06-05
---

# Phase 13 Plan 13: Cross-Binding Collision Detection Summary

**Backend collision gate in set_smart_mode_binding: returns success:false with inline error when a combo is already held by a different global binding (closes UAT test 7 / [B5])**

## Performance

- **Duration:** ~2 min
- **Started:** 2026-06-05T20:25:24Z
- **Completed:** 2026-06-05T20:27:49Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- Added `find_conflicting_binding` pure-logic helper: iterates `settings.bindings`, returns conflicting binding id when another entry holds the same combo, None otherwise; never conflicts with itself (idempotent re-bind safe)
- Wrote 5 TDD unit tests (RED then GREEN): `conflict_detected_for_different_binding`, `no_conflict_when_combo_unused`, `same_binding_id_is_not_a_conflict`, `empty_current_binding_ignored`, `cross_mode_conflict_returns_other_id`
- Wired collision gate into `set_smart_mode_binding` before the entry-insertion block; returns `BindingResponse{success:false, error:"This shortcut is already used by: {name}"}` so the frontend chip's existing inline conflict UI fires
- `change_binding` body unchanged — scope strictly limited to `set_smart_mode_binding`
- `cargo build` clean, all 115 lib tests pass

## Task Commits

Each task was committed atomically:

1. **Task 1: Add find_conflicting_binding pure-logic helper + unit tests** - `9bbb9e7` (test+feat TDD)
2. **Task 2: Gate set_smart_mode_binding on the collision check** - `91164c0` (feat)

**Plan metadata:** (docs commit below)

## Files Created/Modified
- `src-tauri/src/shortcut/mod.rs` - Added `find_conflicting_binding` helper, collision gate in `set_smart_mode_binding`, and 5 unit tests in `#[cfg(test)] mod tests`

## Decisions Made
- Collision check reads `settings.bindings` before the entry-insertion block so an empty placeholder for the current mode cannot trigger a false self-collision
- Error string is exactly `"This shortcut is already used by: {name}"` to match what the chip renders from `response.error`
- `change_binding` deliberately not touched — adding the check there would affect non-smart-mode shortcut editors which rely on register-failure semantics

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- `cargo test --lib find_conflicting -- --nocolor` failed with "Unrecognized option: 'nocolor'" (the test binary uses `--color` not `--nocolor`). Ran without the flag — all 5 tests confirmed passing via the full test suite output.

## Next Phase Readiness
- UAT gap [B5] (test 7) is now closed: cross-binding collision detected at bind time, frontend inline warning fires
- Remaining gaps: [C7] test 11 (engine modal shows Gemma when Apple Foundation active) and [D8] test 3 (seeded mode names show English in non-en app) — plans 13-14 and 13-15 respectively
- `resume_all_shortcuts` can no longer hit "Hotkey already registered" from two smart modes sharing a combo

---
*Phase: 13-smart-modes-ui-translation-presets-i18n*
*Completed: 2026-06-05*
