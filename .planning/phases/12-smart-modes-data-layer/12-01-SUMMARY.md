---
phase: 12-smart-modes-data-layer
plan: "01"
subsystem: settings
tags: [smart-modes, data-model, migration, rust]
dependency_graph:
  requires: []
  provides: [SmartMode, SmartModeKind, TargetLanguage, default_smart_modes, migrate_settings_if_needed, CLEAN_UP_MODE_ID]
  affects: [settings.rs, bindings.ts]
tech_stack:
  added: []
  patterns: [version-guarded migration, atomic commit migration pattern, TDD red-green]
key_files:
  created: []
  modified:
    - src-tauri/src/settings.rs
    - src/bindings.ts
decisions:
  - "Pristine 'Improve Transcriptions' (exact text match) replaced by Clean Up on migration — no near-duplicate retained"
  - "Edited Improve Transcriptions (different text) preserved as Rewrite SmartMode with original id"
  - "migration_v12_to_v13 is atomic: new_modes built separately, assigned at end (Pitfall 5)"
  - "transcribe_with_post_process binding retired, combo transferred to smart_mode_{active_id}"
  - "pre-existing drop_non_drop clippy error in llm.rs is out of scope (pre-dates this plan)"
metrics:
  duration_seconds: 233
  completed_date: "2026-06-03"
  tasks_completed: 3
  files_modified: 2
---

# Phase 12 Plan 01: Smart Modes Data Layer — Types, Defaults, and Migration Summary

SmartMode/SmartModeKind/TargetLanguage types added to settings.rs with 10 curated default modes (6 Rewrite + 4 Translation), version-guarded idempotent v1.2→v1.3 migration, and 19 green tests.

## Tasks Completed

| Task | Description | Commit | Files |
|------|-------------|--------|-------|
| 1 | Declare SmartMode types and AppSettings fields | a421a61 | settings.rs, bindings.ts |
| 2 | Seed 10 default Smart Modes | a421a61 | settings.rs |
| 3 | Implement and test v1.2→v1.3 migration | a421a61 | settings.rs |

Note: Tasks 1 and 2 were implemented as a unit (required by plan — `default_smart_modes` referenced by `get_default_settings` struct literal; they cannot compile independently). Task 3 committed in the same pass since it adds to the same file with all tests.

## What Was Built

### New Types (src-tauri/src/settings.rs)

- `SmartModeKind` enum: `Rewrite` (default) | `Translation`, serde snake_case
- `TargetLanguage` struct: `code: String`, `label: String`
- `SmartMode` struct: `id`, `name`, `kind`, `prompt`, `target_language: Option<TargetLanguage>`

### New AppSettings Fields

- `settings_schema_version: u32` — 0 on v1.2 deserialization, 1 after migration/fresh install
- `smart_modes: Vec<SmartMode>` — serde default = `default_smart_modes()`
- `smart_mode_active_id: Option<String>` — serde default = None

### Default Smart Modes (10 total)

Rewrite (indices 0-5): Clean Up, Make Formal, Make Casual, Write as Email, Bullet Points, Summarize

Translation (indices 6-9): Translate → English (`en`), Spanish (`es`), French (`fr`), Chinese (`zh`)

### Migration Logic (`migrate_settings_if_needed`)

- Version-guarded: returns false if `settings_schema_version >= 1` (idempotent)
- Starts from `default_smart_modes()` (10 seeded modes always present)
- Pristine "Improve Transcriptions" (exact text match) → replaced by Clean Up
- Edited "Improve Transcriptions" (different text) → preserved with original id + Clean Up kept
- Any other custom LLMPrompt → converted to Rewrite SmartMode (id/name/prompt preserved)
- Active id: resolved from `post_process_selected_prompt_id` with pristine→Clean Up remap
- Combo transfer: `transcribe_with_post_process` binding removed, current_binding moved to `smart_mode_{active_id}` key
- Hooked into `load_or_create_app_settings` and `get_settings` (both load paths)

## Test Results

19 tests pass in `settings::tests`:

- `default_smart_modes_count_is_ten`
- `default_smart_modes_order`
- `default_translation_modes_have_target_language`
- `smart_mode_kind_serializes_snake_case`
- `v12_settings_deserializes_without_smart_modes_fields`
- `migration_v12_to_v13_preserves_custom_prompt`
- `migration_v12_pristine_improve_transcriptions_replaced_by_clean_up`
- `migration_v12_edited_improve_transcriptions_kept_plus_clean_up`
- `migration_idempotent`
- `migration_stamps_version`
- `migration_active_selection`
- `migration_transfers_post_process_combo`
- + 7 pre-existing tests (all still green)

## Deviations from Plan

### Auto-fixed Issues

None — plan executed as written.

### Out-of-Scope Discovery

**[Pre-existing] drop_non_drop clippy error in llm.rs:1221**
- Exists before this plan (confirmed via git stash check)
- Not introduced by this plan's changes
- Logged to deferred-items per scope boundary rule
- New code in settings.rs is clippy-clean

## Self-Check: PASSED

- settings.rs: FOUND
- bindings.ts: FOUND
- commit a421a61: FOUND
- All 19 tests green: CONFIRMED
