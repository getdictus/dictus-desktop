---
phase: 11-llm-runtime-foundation
plan: 02
subsystem: llm
tags: [tauri, rust, llama-cpp-2, tauri-specta, post-processing, embedded-llm]

# Dependency graph
requires:
  - phase: 11-llm-runtime-foundation
    plan: 01
    provides: "LlmManager with download/load/inference/shutdown API and LlmModelInfo/LlmDownloadProgress types"

provides:
  - "7 Tauri commands for LLM model management callable from frontend (get, download, cancel, delete, set-active, get-active, import-custom)"
  - "LlmManager registered in Tauri managed state with idle-watcher shutdown on app exit"
  - "LlmDownloadProgress as a typed tauri-specta event for typed TS bindings"
  - "Embedded provider branch in post_process_transcription routing to LlmManager.run_inference"
  - "On-demand model loading when embedded provider is selected and model not yet in memory"

affects: [11-03, 12-smart-modes, frontend-settings, bindings.ts]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "LlmManager follows same Arc<Manager> + app_handle.manage() pattern as ModelManager, TranscriptionManager"
    - "Embedded provider uses provider_id string sentinel checked before HTTP provider lookup — no entry needed in post_process_providers"
    - "flush_and_exit() as the single shared quit path for LLM shutdown (covers both tray quit and no-tray CloseRequested)"

key-files:
  created:
    - src-tauri/src/commands/llm.rs
  modified:
    - src-tauri/src/commands/mod.rs
    - src-tauri/src/lib.rs
    - src-tauri/src/actions.rs
    - src-tauri/src/managers/llm.rs

key-decisions:
  - "LlmManager shutdown wired into flush_and_exit() (shared helper) rather than on_window_event directly — ensures coverage of both tray quit and no-tray CloseRequested paths symmetrically"
  - "Embedded provider detected by checking settings.post_process_provider_id == 'embedded' BEFORE active_post_process_provider() lookup — avoids None early-return since embedded has no entry in post_process_providers list"
  - "import_custom_llm_model takes _app_handle (unused, kept for future use) to match the API contract described in plan"

patterns-established:
  - "Embedded provider sentinel: check provider_id string before HTTP provider lookup, return early from run_inference path before any api_key/base_url logic"
  - "On-demand LLM model load: load active model only when inference is requested, not at startup"

requirements-completed: [LLM-01, LLM-04, MDL-02, MDL-03, MDL-04]

# Metrics
duration: 25min
completed: 2026-06-01
---

# Phase 11 Plan 02: LLM Command Bridge & Embedded Provider Routing Summary

**7 Tauri commands bridging LlmManager to frontend via tauri-specta, with embedded post-processing provider routing to on-demand llama.cpp inference instead of HTTP**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-06-01T~15:00:00Z
- **Completed:** 2026-06-01T~15:25:00Z
- **Tasks:** 2
- **Files modified:** 5 (1 created, 4 modified)

## Accomplishments

- All 7 LLM commands (#[tauri::command] #[specta::specta]) created and registered in collect_commands
- LlmManager initialized in initialize_core_logic and added to Tauri managed state
- LlmDownloadProgress added to collect_events with tauri_specta::Event derive for typed TS bindings
- LlmManager.shutdown() wired into flush_and_exit() covering both quit paths
- Embedded provider branch added to post_process_transcription before HTTP logic; loads active GGUF on demand, routes through run_inference, requires no api_key/base_url/model entry

## Task Commits

Each task was committed atomically:

1. **Task 1: Create commands/llm.rs and register LlmManager + commands + events** - `7359c84` (feat)
2. **Task 2: Route "embedded" provider through LlmManager.run_inference in actions.rs** - `64240cf` (feat)

## Files Created/Modified

- `src-tauri/src/commands/llm.rs` - 7 Tauri commands for LLM model library + active-model selection
- `src-tauri/src/commands/mod.rs` - Added `pub mod llm;`
- `src-tauri/src/lib.rs` - LlmManager init/manage, 7 commands registered, LlmDownloadProgress event, shutdown in flush_and_exit
- `src-tauri/src/actions.rs` - post_process_transcription takes app: &AppHandle, embedded provider branch
- `src-tauri/src/managers/llm.rs` - tauri_specta::Event derive on LlmDownloadProgress

## Decisions Made

- Wired `LlmManager.shutdown()` into the shared `flush_and_exit()` helper rather than directly in `on_window_event` — this ensures both the tray "quit" arm and the no-tray `CloseRequested` branch both shut down the idle watcher thread (symmetric with the existing SHUT-02 pattern).
- Detected embedded provider by checking `settings.post_process_provider_id == "embedded"` before calling `active_post_process_provider()`. The embedded provider intentionally has no entry in `post_process_providers` (no HTTP base_url), so the existing lookup returns `None` and would cause an early return — the early sentinel check avoids that.
- Kept `_app_handle` as unused parameter in `import_custom_llm_model` for future extensibility and to match the command signature contract described in the plan.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None — both tasks compiled on first attempt. Existing 82 unit tests all pass. Zero clippy errors.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 03 can now consume regenerated `bindings.ts` (tauri-specta exports on next `cargo build` / `tauri dev` run) to wire up the frontend LLM model management UI
- Embedded provider backend is complete; Plan 12 (Smart Modes) can set `post_process_provider_id = "embedded"` and the pipeline will route through llama.cpp automatically
- `bindings.ts` regeneration will happen on next dev/build run — Plan 03 will trigger this

---
*Phase: 11-llm-runtime-foundation*
*Completed: 2026-06-01*
