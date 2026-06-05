---
phase: 13-smart-modes-ui-translation-presets-i18n
plan: 12
subsystem: ui
tags: [react, typescript, tauri, smart-modes, shortcuts, i18n]

# Dependency graph
requires:
  - phase: 13-09
    provides: suspendAllShortcuts / resumeAllShortcuts backend commands
  - phase: 13-11
    provides: serialized en/translation.json edits (picker dedup + output hint)
provides:
  - SmartModeShortcutChip wired to suspend ALL global shortcuts on record start and resume on every exit path
  - TranslationEngineChoiceModal active badge driven by engineChoice + activeIsRecommended props (not GGUF activeModelId)
  - SmartModesSection computes activeEngineName and activeIsRecommended; passes both to the modal
affects: [13-13, 13-14, 13-15]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Global shortcut suspend-all/resume-all pattern: suspendAllShortcuts() on capture start, resumeAllShortcuts() on every exit path (commit success, commit conflict, click-outside, effect cleanup)"
    - "Engine modal driven by persisted engineChoice + activeIsRecommended props, not GGUF activeModelId — enables non-GGUF providers (Apple Intelligence) to show active badge"

key-files:
  created: []
  modified:
    - src/components/settings/post-processing/SmartModeShortcutChip.tsx
    - src/components/settings/post-processing/TranslationEngineChoiceModal.tsx
    - src/components/settings/post-processing/SmartModesSection.tsx

key-decisions:
  - "TranslationEngineChoiceModal active badge driven by engineChoice + activeIsRecommended props (not GGUF activeModelId) — covers Apple Intelligence and non-GGUF providers"
  - "SmartModeShortcutChip: module-level resumeAll() helper + suspendAllShortcuts on record start; resumeAll on commitCombo (success+conflict), handleClickOutside, and effect cleanup — every exit path covered"

patterns-established:
  - "resumeAll helper: module-level const at top of chip file, shared across commitCombo branches, handleClickOutside, and effect cleanup — single definition, consistent idempotent semantics"

requirements-completed: [MODE-05, TRANS-02]

# Metrics
duration: ~20min (tasks 1-2 only; task 3 is human-verify checkpoint)
completed: 2026-06-05
---

# Phase 13 Plan 12: Chip Suspend-All/Resume-All + Engine Modal Active Badge Summary

**Global shortcut suspend-all/resume-all wiring on SmartModeShortcutChip + engine-modal active badge driven by persisted engineChoice+provider instead of GGUF-only activeModelId**

## Performance

- **Duration:** ~20 min (tasks 1-2 automated; task 3 is human-verify checkpoint)
- **Started:** 2026-06-05T20:30:12Z
- **Completed:** 2026-06-05 (tasks 1-2; task 3 pending human-verify)
- **Tasks:** 2 of 3 (task 3 is checkpoint:human-verify)
- **Files modified:** 3

## Accomplishments

- SmartModeShortcutChip now calls `suspendAllShortcuts()` on record start and `resumeAllShortcuts()` on every exit path (commit success, commit conflict, click-outside, and effect cleanup unmount path), preventing already-bound combos from firing their action during capture (UAT test 7)
- TranslationEngineChoiceModal active badge derived from `engineChoice` (persisted setting) + `activeIsRecommended` prop instead of the GGUF-only `activeModelId`, so Apple Intelligence and other non-GGUF providers correctly show "use current model" as active (UAT test 11 display)
- SmartModesSection computes `activeEngineName` (GGUF model name when embedded active, else provider label) and `activeIsRecommended` (only true for embedded+gemma-3-4b), passing both to the modal

## Task Commits

Each task was committed atomically:

1. **Task 1: Chip suspends ALL global shortcuts during capture** - `262576b` (fix)
2. **Task 2: Engine modal active badge from persisted choice + active provider** - `89e17d6` (fix)

**Note:** Task 3 is a `checkpoint:human-verify` — awaiting live-build verification of conflict capture and engine modal display.

## Files Created/Modified

- `src/components/settings/post-processing/SmartModeShortcutChip.tsx` — Module-level `resumeAll()` helper; `suspendAllShortcuts()` on record start; `resumeAll()` on all exit paths
- `src/components/settings/post-processing/TranslationEngineChoiceModal.tsx` — Props `engineChoice`, `activeEngineName`, `activeIsRecommended` added; `recActive`/`currentActiveSelected` derived from props; "use current" button gated on `activeEngineName != null` not `activeDownloaded`
- `src/components/settings/post-processing/SmartModesSection.tsx` — `isEmbeddedActive` guard, `activeEngineName`, `activeIsRecommended` computed and passed to modal

## Decisions Made

- `resumeAll` defined as a module-level constant (not inside the component) — avoids stale closure issues and ensures the same function reference is reused across all exit paths
- "Use current model" button enabled whenever `activeEngineName != null` (not gated on `activeDownloaded`) — Apple Intelligence has a label but no GGUF model; the `activeDownloaded` gate was GGUF-specific and wrong for non-GGUF providers
- `activeIsRecommended` flagged false for all non-embedded providers regardless of `activeModelId` — `activeModelId` lingers non-null after switching away from embedded GGUF, which was the root cause of the modal showing Gemma active when Apple Foundation was selected

## Deviations from Plan

None — plan executed exactly as written. Note that `isEmbeddedActive` gating was added in the subsequent 13-14 plan (gap-closure round 2), which built on the props introduced here.

## Issues Encountered

None during tasks 1-2. Task 3 (live human-verify) is the pending gate before plan closure.

**Known follow-up (documented in plan):** `handy_keys` background-focus capture — after suspend-all, webview keydown capture requires the settings window to be focused. For the UAT scenario the window IS focused when the user clicks the chip; unfocused background capture remains a known follow-up if a later UAT shows capture failing while unfocused.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Chip suspend-all/resume-all is implemented and lint/translation checks pass
- Modal prop interface is established for 13-14 to gate `activeIsRecommended` on `isEmbeddedActive`
- Task 3 (human-verify checkpoint) must be approved before this plan is marked complete

---
*Phase: 13-smart-modes-ui-translation-presets-i18n*
*Completed: 2026-06-05*
