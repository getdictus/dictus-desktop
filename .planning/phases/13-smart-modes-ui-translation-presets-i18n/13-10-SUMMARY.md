---
phase: 13-smart-modes-ui-translation-presets-i18n
plan: 10
subsystem: api
tags: [apple-intelligence, foundation-models, swift, smart-modes, translation, post-processing]

# Dependency graph
requires:
  - phase: 12-smart-modes-data-layer
    provides: Smart Mode kinds (rewrite/translate/bullets) routed through process_transcription_output
  - phase: 13-smart-modes-ui-translation-presets-i18n
    provides: Smart Mode UI + translation engine wiring (13-01..13-07) and prompt tuning
provides:
  - Task-agnostic Apple Intelligence post-process path — free-text session.respond(to:) carrying the Smart Mode prompt as instructions
  - Removal of the hardcoded @Generable CleanedTranscript schema that biased every Apple task toward transcript cleanup
  - Neutralized word-count truncation on the Apple smart-mode path (maxTokens=0)
affects: [apple-intelligence, smart-modes, translation, post-processing]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Apple FoundationModels free-text generation: pass the user/task prompt via LanguageModelSession instructions, call session.respond(to:) without a @Generable schema, so the model follows the actual task"

key-files:
  created: []
  modified:
    - src-tauri/swift/apple_intelligence.swift
    - src-tauri/src/actions.rs

key-decisions:
  - "Apple Intelligence Smart Mode output is generated free-text (no @Generable schema). The single-field CleanedTranscript struct was the source of translate/bullets failures — it framed every task as transcript cleanup."
  - "token_limit hardcoded to 0 on the Apple branch in actions.rs — the per-provider 'model' string ('Apple Intelligence') is a label, not a numeric word cap; parsing it was a latent footgun."

patterns-established:
  - "Free-text Apple FoundationModels path: instructions carry the task, session.respond(to:) returns arbitrary task output — mirrors the embedded LLM behavior."

requirements-completed: [TRANS-01, TRANS-02, MODE-06]

# Metrics
duration: ~25min (code) + user live-verify window
completed: 2026-06-05
---

# Phase 13 Plan 10: Apple Intelligence Task-Agnostic Smart Mode Path Summary

**Apple Intelligence post-process now uses free-text `session.respond(to:)` driven by the Smart Mode prompt (no hardcoded CleanedTranscript schema), so translate/bullets/clean-up all produce correct, untruncated output.**

## Performance

- **Duration:** ~25 min code execution + live macOS verification window
- **Started:** 2026-06-05T14:58:19Z
- **Completed:** 2026-06-05 (after user live-verify approval)
- **Tasks:** 3 (2 code + 1 human-verify checkpoint)
- **Files modified:** 2

## Accomplishments

- Removed the hardcoded `@Generable struct CleanedTranscript` from `apple_intelligence.swift`. This schema forced every Apple Intelligence task through a "clean a transcript" frame, so translate/bullets/formal prompts yielded near-unchanged or terse output.
- Replaced the structured `session.respond(to:generating:)` + catch-fallback block with a single plain free-text `session.respond(to:)` call. The Smart Mode prompt is carried via the session `instructions`, so the model follows the actual task.
- Neutralized the word-count truncation on the Apple smart-mode path: `actions.rs` now passes `token_limit = 0` explicitly instead of parsing the "Apple Intelligence" label string as an integer word cap. Swift's `truncatedText()` guard (`limit > 0`) makes this a guaranteed no-op.
- Verified live on macOS Apple Silicon (user-confirmed): Spanish translation works, Bullet Points produces a full multi-line list (no ~26-char fragment), Clean Up still cleans correctly.

## Task Commits

Each task was committed atomically:

1. **Task 1: Make the Apple Swift path task-agnostic (free-text generation)** - `0be3f89` (fix)
2. **Task 2: Stop word-count truncation for the Apple smart-mode path** - `b47c499` (fix)
3. **Task 3: Verify Apple Intelligence Smart Mode output on macOS** - human-verify checkpoint, approved live by user (no code change)

**Plan metadata:** see final docs commit.

## Files Created/Modified

- `src-tauri/swift/apple_intelligence.swift` - Removed `@Generable CleanedTranscript` struct; `processTextWithSystemPrompt` now calls plain `session.respond(to:)`; `truncatedText()` retained (no-op at limit 0).
- `src-tauri/src/actions.rs` - Apple branch passes `token_limit: i32 = 0` (was a dynamic parse of the model label string).

## Decisions Made

- **Free-text generation over schema:** The single-field `CleanedTranscript` schema was the root cause behind UAT test 10 ("translation does not happen") and the truncation half of UAT test 11 ("Apple bullet output cut to 26 chars"). Switching to instructions-driven free-text generation mirrors how the working embedded LLM path behaves.
- **Hardcoded `token_limit = 0`:** The Apple provider "model" string is a UI label, not a numeric word cap. It parsed to 0 today (so no live truncation), but interpreting it as a cap was fragile; hardcoding 0 removes the latent footgun without touching the non-Apple structured-output branch.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- **Pre-existing unrelated changes in working tree:** `src-tauri/src/shortcut/mod.rs` had uncommitted modifications (a `resolve_seeded_id` helper from other 13-xx work) and an untracked `dictus-postproc-mockup.png`. These were NOT part of Plan 10 and were left out of the task commits — only the two plan-scoped files were staged individually per task.
- **SourceKit false-positive (informational, not a regression):** `apple_intelligence.swift` shows a SourceKit warning "Cannot find type 'AppleLLMResponse'". That type is the `#[repr(C)]` struct from `src-tauri/src/apple_intelligence.rs`, resolved at Cargo build time via C interop. `cargo build` succeeds on both task commits. Confirmed by the user as a known false-positive, not introduced by this plan.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Apple Intelligence path is now task-agnostic and verified live. This closes the shared root cause behind UAT tests 10 and 11.
- Remaining 13-xx gap-closure plans (13-11, 13-12) handle the `${output}` UX concern and engine-modal active-badge fixes respectively; this plan deliberately left `build_system_prompt`/`user_content` as-is per the plan boundary.

---
*Phase: 13-smart-modes-ui-translation-presets-i18n*
*Completed: 2026-06-05*

## Self-Check: PASSED

- FOUND: src-tauri/swift/apple_intelligence.swift
- FOUND: src-tauri/src/actions.rs
- FOUND: .planning/phases/13-smart-modes-ui-translation-presets-i18n/13-10-SUMMARY.md
- FOUND: commit 0be3f89 (Task 1)
- FOUND: commit b47c499 (Task 2)
