---
phase: 12
plan: "03"
subsystem: smart-modes-routing
tags: [smart-modes, routing, shortcut, post-processing, tdd]
dependency_graph:
  requires: ["12-01"]
  provides: ["MODE-04-routing"]
  affects: ["transcription_coordinator", "actions", "shortcut-init"]
tech_stack:
  added: []
  patterns: ["SmartModeAction", "spawn_transcription_task shared helper", "post_process_with_prompt helper", "resolve_mode_prompt pure helper"]
key_files:
  created: []
  modified:
    - src-tauri/src/transcription_coordinator.rs
    - src-tauri/src/actions.rs
    - src-tauri/src/commands/history.rs
    - src-tauri/src/shortcut/tauri_impl.rs
    - src-tauri/src/shortcut/handy_keys.rs
decisions:
  - "Factored TranscribeAction::stop body into spawn_transcription_task(app, binding_id, post_process, mode_id_override: Option<String>) to avoid duplicating ~100-line async pipeline in SmartModeAction::stop"
  - "SmartModeAction::start delegates to ACTION_MAP['transcribe_with_post_process'].start — recording start logic is mode-agnostic; mode_id only matters at stop time"
  - "resolve_mode_prompt returns None for Translation kind (stub path), which maps to the Phase 13 warn log in process_transcription_output — clean signal without inference"
  - "Test module moved to end of file in transcription_coordinator.rs to satisfy clippy items_after_test_module lint"
  - "Pre-existing drop_non_drop in llm.rs left in place (out of scope per SCOPE BOUNDARY rule)"
metrics:
  duration_minutes: 22
  completed_date: "2026-06-03"
  tasks_completed: 3
  files_modified: 5
---

# Phase 12 Plan 03: Smart Modes Routing Summary

Wire `smart_mode_{id}` shortcuts end-to-end: prefix match in `is_transcribe_binding`, `SmartModeAction` driving the record→transcribe→mode-prompt pipeline, `process_transcription_output` accepting a `mode_id_override` that branches on `SmartModeKind` (Rewrite live, Translation stubbed for Phase 13), and both keyboard init paths registering bound smart modes.

## Tasks Completed

| # | Name | Commit | Key Files |
|---|------|--------|-----------|
| 1 | Extend is_transcribe_binding + refactor process_transcription_output | 86f7ef5 | transcription_coordinator.rs, actions.rs, commands/history.rs |
| 2 | SmartModeAction + coordinator start/stop dispatch | cbd5e30 | actions.rs, transcription_coordinator.rs |
| 3 | Register bound smart modes at init (both keyboard impls) | ec19cd7 | shortcut/tauri_impl.rs, shortcut/handy_keys.rs |

## What Was Built

### is_transcribe_binding extended (`transcription_coordinator.rs`)
Added `|| id.starts_with("smart_mode_")` to the existing two-case check. Keeps the coordinator's routing logic centralized.

### process_transcription_output refactored (`actions.rs`)
- Added `mode_id_override: Option<&str>` parameter
- Extracts `post_process_with_prompt(app, settings, transcription, prompt)` helper containing the full inference pipeline (embedded + HTTP + structured output + Apple Intelligence + legacy fallback) — accepts an explicit prompt string rather than resolving from settings
- Refactors `post_process_transcription` to resolve active selected prompt then delegate to `post_process_with_prompt` (existing behavior preserved exactly)
- Adds `resolve_mode_prompt(settings, mode_id) -> Option<(&SmartMode, String)>` pure helper: returns `Some((mode, prompt))` for Rewrite, `None` for Translation
- When `mode_id_override = Some(id)`: calls `resolve_mode_prompt`, branches on kind: Rewrite runs inference, Translation logs warn and returns stub
- Both legacy call sites (`TranscribeAction::stop`, `history.rs`) pass `None` — behavior identical to before

### SmartModeAction + spawn_transcription_task (`actions.rs`)
- `pub(crate) struct SmartModeAction { pub mode_id: String }` with `impl ShortcutAction`
- `start` delegates to `ACTION_MAP["transcribe_with_post_process"].start` (recording is mode-agnostic)
- `stop` calls `spawn_transcription_task(app, binding_id, true, Some(self.mode_id.clone()))`
- `spawn_transcription_task(app, binding_id, post_process, mode_id_override: Option<String>)` is the factored-out async pipeline body (previously duplicated in `TranscribeAction::stop`). Both actions now share this single codepath.

### Coordinator dispatch updated (`transcription_coordinator.rs`)
Both `start` and `stop` now check `binding_id.starts_with("smart_mode_")` first: if so, strip prefix and build `SmartModeAction { mode_id }` dynamically. Otherwise, fall through to `ACTION_MAP.get()` as before.

### Smart mode init loops (`shortcut/tauri_impl.rs`, `shortcut/handy_keys.rs`)
Added second loop after the default-bindings loop in both `init_shortcuts` implementations. Iterates `user_settings.smart_modes`, looks up `smart_mode_{id}` in `user_settings.bindings`, and registers only entries with non-empty `current_binding`. Defaults ship unbound so this fires only for migrated or user-configured modes.

## Tests Added

| Test | File | Assertion |
|------|------|-----------|
| `mode_routing_rewrite_uses_correct_prompt` | actions.rs | resolve_mode_prompt returns mode's own prompt for Rewrite kind |
| `mode_routing_translation_returns_none` | actions.rs | resolve_mode_prompt returns None for Translation kind (stub signal) |
| `mode_routing_unknown_id_returns_none` | actions.rs | resolve_mode_prompt returns None for unknown id |
| `is_transcribe_binding_smart_mode_prefix` | transcription_coordinator.rs | prefix recognized; cancel/test not recognized |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] clippy items_after_test_module in transcription_coordinator.rs**
- **Found during:** Overall verification (cargo clippy)
- **Issue:** Test module placed before `fn start` and `fn stop` triggered `items_after_test_module` lint
- **Fix:** Moved `#[cfg(test)] mod coordinator_tests` to end of file
- **Files modified:** src-tauri/src/transcription_coordinator.rs
- **Commit:** 40b26c7

### Out-of-scope (pre-existing)

- `drop_non_drop` in `src-tauri/src/managers/llm.rs:1221` — pre-existing from Phase 11; logged to deferred-items, not fixed

## Self-Check: PASSED

- `src-tauri/src/transcription_coordinator.rs` — modified, contains `starts_with("smart_mode_")` and coordinator dispatch
- `src-tauri/src/actions.rs` — modified, contains `SmartModeAction`, `spawn_transcription_task`, `post_process_with_prompt`, `mode_id_override`, `resolve_mode_prompt`, test module
- `src-tauri/src/commands/history.rs` — modified, passes `None` to `process_transcription_output`
- `src-tauri/src/shortcut/tauri_impl.rs` — modified, contains smart modes init loop
- `src-tauri/src/shortcut/handy_keys.rs` — modified, contains smart modes init loop
- Commits: 86f7ef5, cbd5e30, ec19cd7, 40b26c7 — all present
- `cargo test`: 101 passed, 0 failed
- `cargo build`: clean
