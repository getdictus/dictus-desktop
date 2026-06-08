---
phase: 13-smart-modes-ui-translation-presets-i18n
plan: 20
subsystem: shortcut-conflict-detection
tags: [gap-closure, G13, backend, tdd, rust, shortcut, structured-error]
dependency_graph:
  requires: [13-18]
  provides: [ConflictKind-enum, structured-conflict-error-payload]
  affects: [find_conflicting_binding, set_smart_mode_binding, BindingResponse.error]
tech_stack:
  added: []
  patterns: [structured-error-code, pipe-delimited-payload, find_map-with-kind]
key_files:
  created: []
  modified:
    - src-tauri/src/shortcut/mod.rs
decisions:
  - ConflictKind enum uses serde snake_case matching SmartModeKind pattern; derives Serialize+Type for specta bindings even though it is not yet a direct field in BindingResponse (carried in the error string payload for now)
  - Both CandidateBaseIsExisting and CandidateIsBaseOfExisting collapse to base_overlap code in the SHORTCUT_CONFLICT payload — frontend needs only two distinct messages, not three
  - SHORTCUT_CONFLICT|<code>|<name>[|<base>] format: leading token is the stable discriminator for frontend split(); pipe delimiter avoids JSON overhead and keeps BindingResponse.error: Option<String> shape unchanged
  - combo_base reused from 13-18 helper for computing {base} field in base_overlap payload
metrics:
  duration: "~3 minutes"
  completed: "2026-06-08T19:03:21Z"
  tasks_completed: 2
  files_modified: 1
---

# Phase 13 Plan 20: Structured Conflict Error Backend Summary

**One-liner:** `find_conflicting_binding` now returns `(id, ConflictKind)` and `set_smart_mode_binding` emits a machine-parseable `SHORTCUT_CONFLICT|<code>|<name>[|<base>]` payload instead of hardcoded English prose, establishing the backend side of the G13 translatable-error pattern.

## What Was Built

Gap [G13] backend closure: the shortcut conflict error that crosses the backend→frontend boundary is now a structured, stable, localizable payload rather than hardcoded English prose.

### ConflictKind enum (Task 1)

Added above `find_conflicting_binding` in `src-tauri/src/shortcut/mod.rs`:

```rust
#[derive(Serialize, Type, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConflictKind {
    ExactDuplicate,
    CandidateBaseIsExisting,
    CandidateIsBaseOfExisting,
}
```

Changed `find_conflicting_binding` return type from `Option<String>` to `Option<(String, ConflictKind)>`. Replaced `.find(...).map(...)` with `.find_map(...)` computing the kind via priority-ordered if/else (rule 1 ExactDuplicate → rule 2 CandidateBaseIsExisting → rule 3 CandidateIsBaseOfExisting). All guards (skip same id, skip empty binding) preserved verbatim.

### Structured conflict payload (Task 2)

In `set_smart_mode_binding`, replaced the old conflict block that produced `"This shortcut is already used by: {name}"` with a structured payload:

- `exact_duplicate` → `SHORTCUT_CONFLICT|exact_duplicate|{other_name}`
- `base_overlap` (both CandidateBase* variants) → `SHORTCUT_CONFLICT|base_overlap|{other_name}|{base}`

`{base}` computed via the existing `combo_base(&binding)` helper from 13-18. `BindingResponse.error` shape unchanged (`Option<String>`), so no specta bindings or frontend back-compat change needed.

### Unit tests

All 16 existing tests updated: `Some(id)` assertions changed to `Some((id, ConflictKind::...))` with the correct variant per rule:
- `conflict_detected_for_different_binding` → `ExactDuplicate`
- `cross_mode_conflict_returns_other_id` → `ExactDuplicate`
- `base_prefix_collision_blocks` → `CandidateBaseIsExisting`
- `symmetric_base_collision_blocks` → `CandidateIsBaseOfExisting`
- `None` assertions unchanged

## Tasks

| # | Name | Status | Commit |
|---|------|--------|--------|
| 1 | Add ConflictKind and make find_conflicting_binding return it | DONE | aebed5b |
| 2 | Emit structured conflict error from set_smart_mode_binding | DONE | aebed5b |

Note: Both tasks were committed together in aebed5b since the return-type change in Task 1 requires the Task 2 callsite update to compile.

## Test Coverage

16 unit tests pass in `shortcut::tests`:
- All 5 pre-existing `find_conflicting_binding` tests updated with ConflictKind assertions
- All 4 prefix/base-key overlap tests (13-18) updated with CandidateBaseIsExisting / CandidateIsBaseOfExisting
- 7 other shortcut tests unaffected

## Deviations from Plan

None — plan executed exactly as written. Tasks 1 and 2 were committed as a single atomic commit because the return-type change in Task 1 requires the Task 2 consumer update to compile (splitting would produce a broken intermediate state).

## Self-Check: PASSED

- src-tauri/src/shortcut/mod.rs: FOUND
- 13-20-SUMMARY.md: FOUND (this file)
- commit aebed5b: FOUND (`git log --oneline -1` confirms)
- `grep -n "enum ConflictKind" src-tauri/src/shortcut/mod.rs` → line 1186: PASS
- `grep -n "Option<(String, ConflictKind)>" src-tauri/src/shortcut/mod.rs` → line 1217: PASS
- `grep -c "SHORTCUT_CONFLICT" src-tauri/src/shortcut/mod.rs` → 3: PASS
- `grep -c "This shortcut is already used by" src-tauri/src/shortcut/mod.rs` → 0: PASS
- `cargo test --lib shortcut::` → 16/16 green: PASS
- `cargo build` → exit 0: PASS
- `cargo clippy --all-targets -- -D warnings` → exit 0: PASS
