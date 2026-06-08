---
phase: 13-smart-modes-ui-translation-presets-i18n
plan: 18
subsystem: shortcut-conflict-detection
tags: [gap-closure, G11, backend, tdd, rust, shortcut]
dependency_graph:
  requires: [13-13]
  provides: [prefix-base-key-overlap-detection]
  affects: [set_smart_mode_binding, find_conflicting_binding]
tech_stack:
  added: []
  patterns: [TDD red-green-refactor, combo_base helper, is_modifier_token helper]
key_files:
  created: []
  modified:
    - src-tauri/src/shortcut/mod.rs
decisions:
  - combo_base uses rfind('+') on the last '+' after confirming the left-side prefix is all-modifier tokens; modifier-only combos return whole string
  - is_modifier_token lowercases only for comparison (accepts both handy_keys and tauri casing) but the main find_conflicting_binding comparison is NOT lowercased (preserves 13-13 parity)
  - Redundant closure |seg| is_modifier_token(seg) replaced by is_modifier_token directly to satisfy clippy -D warnings
  - set_smart_mode_binding unchanged — already maps Some(other_id) to BindingResponse{success:false, error}; chip renders it inline
metrics:
  duration: "3 minutes"
  completed: "2026-06-08T16:58:26Z"
  tasks_completed: 2
  files_modified: 1
---

# Phase 13 Plan 18: Prefix/Base-Key Overlap Conflict Detection Summary

**One-liner:** Extend `find_conflicting_binding` with prefix/base-key overlap detection so combos that can physically never fire are blocked at bind time, not silently accepted.

## What Was Built

Gap [G11] closure: when a user tries to bind `command_left+digit1` but `command_left` is already a shortcut (or vice versa), the OS fires the base-key action before the trailing key registers, making the combo unreachable. Previously `find_conflicting_binding` only did full-string equality, so the overlap went undetected.

Added two private helpers and extended the conflict-detection closure:

- `is_modifier_token(token: &str) -> bool` — recognises all handy_keys and tauri modifier key names
- `combo_base(combo: &str) -> &str` — returns the modifier-only prefix of a combo (everything before the last `+` when that prefix is all-modifier); modifier-only combos return the whole string
- `find_conflicting_binding` extended with two new branches after the existing exact-equality check:
  - (2) `other == candidate_base` — candidate's base IS an existing full binding
  - (3) `combo_base(other) == target` — candidate IS the base of an existing full combo

Two distinct full combos sharing only a modifier prefix (e.g. `command_left+digit1` vs `command_left+digit2`) do NOT trigger — correctly identified as non-overlapping.

Zero frontend changes: `SmartModeShortcutChip.tsx` already renders `response.error` inline.

## Tasks

| # | Name | Status | Commit |
|---|------|--------|--------|
| 1 | Add prefix/base-key overlap rule to find_conflicting_binding + unit tests (TDD) | DONE | 1e45a9b (RED), e562e37 (GREEN) |
| 2 | Build + clippy + fmt gate | DONE | 63887c5 |

## Test Coverage

16 unit tests pass in `shortcut::tests` (all in `src-tauri/src/shortcut/mod.rs`):

- 5 pre-existing `find_conflicting_binding` tests (13-13 behaviour preserved)
- 4 new overlap tests:
  - `base_prefix_collision_blocks` — existing `command_left` blocks `command_left+digit1`
  - `symmetric_base_collision_blocks` — existing `command_left+digit1` blocks `command_left`
  - `no_collision_when_bases_differ` — `command_right` does not block `command_left+digit1`
  - `distinct_full_combos_no_base_overlap` — `command_left+digit2` does not block `command_left+digit1`
- 7 other pre-existing shortcut tests

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Clippy redundant closure warning**
- **Found during:** Task 2
- **Issue:** `prefix.split('+').all(|seg| is_modifier_token(seg))` triggers `clippy::redundant_closure` under `-D warnings`
- **Fix:** Replaced with `prefix.split('+').all(is_modifier_token)` — function reference directly
- **Files modified:** src-tauri/src/shortcut/mod.rs
- **Commit:** 63887c5

**2. [Rule 2 - Formatting] cargo fmt reformatting**
- **Found during:** Task 2
- **Issue:** Multi-line chain expression and assert_eq! not matching rustfmt style
- **Fix:** `cargo fmt` applied inline
- **Files modified:** src-tauri/src/shortcut/mod.rs
- **Commit:** 63887c5

## Self-Check: PASSED

- src-tauri/src/shortcut/mod.rs: FOUND
- 13-18-SUMMARY.md: FOUND
- commit 1e45a9b (RED): FOUND
- commit e562e37 (GREEN): FOUND
- commit 63887c5 (quality gate): FOUND
