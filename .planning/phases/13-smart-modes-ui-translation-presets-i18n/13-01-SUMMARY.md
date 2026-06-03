---
phase: 13-smart-modes-ui-translation-presets-i18n
plan: "01"
subsystem: translation-backend
tags: [translation, llm, settings, commands, rust]
dependency_graph:
  requires: []
  provides: [TranslationEngineChoice, setTranslationEngineChoice, translate-gemma-4b-catalogue, run_translation]
  affects: [actions.rs, settings.rs, commands/llm.rs, lib.rs, managers/llm.rs, bindings.ts, LlmLibrarySection.tsx]
tech_stack:
  added: []
  patterns: [run_translation helper, engine-choice dispatch, catalogue entry pattern]
key_files:
  created: []
  modified:
    - src-tauri/src/settings.rs
    - src-tauri/src/commands/llm.rs
    - src-tauri/src/lib.rs
    - src-tauri/src/managers/llm.rs
    - src-tauri/src/actions.rs
    - src/bindings.ts
    - src/components/settings/post-processing/LlmLibrarySection.tsx
decisions:
  - "TranslationEngineChoice uses snake_case serde (not_chosen/translate_gemma/generic_model) matching SmartModeKind pattern"
  - "TranslateGemma SHA256 computed from real file download (7f7357c14abd9da4eb200b38b05da502cd6e10d7e1d403fbc9f78c19f3209b72)"
  - "run_translation is a standalone async helper, not inlined in process_transcription_output, for testability"
  - "resolve_mode_prompt doc comment updated — Translation now routed via process_transcription_output, not stub"
metrics:
  duration_min: 126
  completed_date: "2026-06-03"
  tasks_completed: 3
  files_modified: 7
---

# Phase 13 Plan 01: Translation Backend Wire-Up Summary

**One-liner:** TranslateGemma 4B catalogue entry + TranslationEngineChoice persisted setting + real run_translation inference replacing Phase 12 stub

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add TranslationEngineChoice setting + setter command | 9923d65 | settings.rs, commands/llm.rs, lib.rs |
| 2 | Add TranslateGemma catalogue entry | d5aa378 | managers/llm.rs, LlmLibrarySection.tsx |
| 2b | Update TranslateGemma SHA256 to real value | 275497d | managers/llm.rs |
| 3 | Wire Translation execution path in actions.rs | 7f19ae1 | actions.rs, bindings.ts |

## What Was Built

**Task 1 — TranslationEngineChoice setting:**
- Added `TranslationEngineChoice` enum to `settings.rs` with variants `NotChosen` (default), `TranslateGemma`, `GenericModel`; serde serializes as snake_case
- Added `translation_engine_choice: TranslationEngineChoice` field to `AppSettings` with `#[serde(default)]`
- Added explicit field to `get_default_settings()` (NotChosen)
- Added `set_translation_engine_choice` command in `commands/llm.rs` mirroring `set_active_llm_model` pattern (writes setting, no model load)
- Registered command in `collect_commands!` in `lib.rs`

**Task 2 — TranslateGemma catalogue entry:**
- Added `translate-gemma-4b` to `catalogue()` in `managers/llm.rs` with real HuggingFace URL (bullerwins/translategemma-4b-it-GGUF), real computed SHA256 (`7f7357c14abd9da4eb200b38b05da502cd6e10d7e1d403fbc9f78c19f3209b72`), size_mb: 2490
- Updated `test_catalogue_has_four_models` to `test_catalogue_has_five_models` (101 → 101 tests passing, count corrected)
- Added `"translate-gemma-4b": "translateGemma4b"` to `MODEL_ID_TO_I18N_KEY` in `LlmLibrarySection.tsx`

**Task 3 — Translation execution path:**
- Replaced Phase 12 stub (`"translation engine is not yet wired"` warn) with real dispatch in `process_transcription_output`
- Added `run_translation` async helper that dispatches per `TranslationEngineChoice`:
  - `NotChosen`: log warn, return None (safe no-op)
  - `TranslateGemma`: load `translate-gemma-4b` on demand, pass `"{text}\n{target.code}"` (native template format)
  - `GenericModel`: load `active_llm_model_id`, build explicit `"Translate the following text to {label}. Output only the translation..."` prompt
- Updated `resolve_mode_prompt` doc comment — Translation is no longer a stub
- Added `generic_model_translation_prompt_contains_target_language` unit test (pure-logic, no AppHandle required)
- `bindings.ts` auto-regenerated with `TranslationEngineChoice` type and `setTranslationEngineChoice` command

## Verification

- `cargo test`: 102 passed, 0 failed
- `cargo clippy --all-targets -- -D warnings`: no errors (pre-existing `drop_non_drop` in llm.rs test also fixed)
- `grep TranslationEngineChoice src/bindings.ts`: found
- `grep setTranslationEngineChoice src/bindings.ts`: found

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed test_catalogue_has_four_models count assertion**
- **Found during:** Task 2
- **Issue:** Adding the 5th catalogue entry broke the existing count assertion (`assert_eq!(catalogue.len(), 4)`)
- **Fix:** Updated test name to `test_catalogue_has_five_models` and assertion to `5`
- **Files modified:** src-tauri/src/managers/llm.rs
- **Commit:** d5aa378

**2. [Rule 1 - Bug] Fixed pre-existing drop_non_drop clippy error**
- **Found during:** Overall verification (clippy)
- **Issue:** `drop(before)` on `AtomicU64` (which doesn't implement `Drop`) caused clippy error with `-D warnings`
- **Fix:** Changed `drop(before)` to `let _ = before` in llm.rs test
- **Files modified:** src-tauri/src/managers/llm.rs
- **Commit:** 275497d

**3. TranslateGemma SHA256 computed from real download**
- **Note:** The plan allowed `sha256: None` if download couldn't complete. The 2.49 GB file downloaded successfully during task execution (HuggingFace URL valid, SHA256 = `7f7357c14abd9da4eb200b38b05da502cd6e10d7e1d403fbc9f78c19f3209b72`)
- Initial commit used `None` (d5aa378), then updated to real hash in follow-up commit (275497d)

## Self-Check: PASSED
