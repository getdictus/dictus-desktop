---
phase: 13-smart-modes-ui-translation-presets-i18n
plan: 14
subsystem: ui
tags: [react, typescript, smart-modes, translation, llm, apple-intelligence]

# Dependency graph
requires:
  - phase: 13-smart-modes-ui-translation-presets-i18n
    provides: TranslationEngineChoiceModal consuming activeEngineName/activeIsRecommended props (13-12)
  - phase: 13-smart-modes-ui-translation-presets-i18n
    provides: Apple Intelligence task-agnostic Smart Mode path, providerId=embedded canonical check (Phase 11/13-10)
provides:
  - SmartModesSection engine-name resolution gated on post_process_provider_id === "embedded"
  - activeEngineName resolves to provider label (e.g. "Apple Intelligence") for non-embedded providers
  - activeIsRecommended is false for non-embedded providers
affects: [13-15, translation-engine-modal, smart-modes-section]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Gate GGUF model-name lookups on providerId === 'embedded' — activeModelId lingers non-null across provider switches"

key-files:
  created: []
  modified:
    - src/components/settings/post-processing/SmartModesSection.tsx

key-decisions:
  - "[13-14] activeModelName gated on isEmbeddedActive (providerId === 'embedded') — prevents lingering GGUF activeModelId from short-circuiting ?? chain to Gemma when Apple Foundation is active"
  - "[13-14] activeIsRecommended = isEmbeddedActive && activeModelId === 'gemma-3-4b' — recommended badge only shown for embedded provider with Gemma active"

patterns-established:
  - "Engine-name resolution: always gate GGUF store lookups on embedded-provider check before using ?? fallback chain"

requirements-completed: [TRANS-01, TRANS-02]

# Metrics
duration: 2min
completed: 2026-06-05
---

# Phase 13 Plan 14: Smart Modes — Engine Name Gate Summary

**SmartModesSection.tsx activeEngineName gated on embedded provider so Apple Intelligence / cloud providers show their own label instead of lingering Gemma model name**

## Performance

- **Duration:** ~2 min
- **Started:** 2026-06-05T20:25:27Z
- **Completed:** 2026-06-05T20:26:36Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments
- Added `isEmbeddedActive = providerId === "embedded"` gate before deriving `activeModelName`
- `activeModelName` is now null for any non-embedded provider, so the `??` chain correctly falls through to the active provider's display label
- `activeIsRecommended` now gated: `isEmbeddedActive && activeModelId === "gemma-3-4b"` — false for Apple Intelligence / cloud providers
- Section-header `currentEngine` label unchanged — already references the gated `activeModelName` variable
- TranslationEngineChoiceModal untouched — consumes `activeEngineName`/`activeIsRecommended` props correctly (13-12)
- Closes gap [C7] / UAT test 11 (TRANS-01/TRANS-02)

## Task Commits

Each task was committed atomically:

1. **Task 1: Gate activeEngineName + activeIsRecommended on the embedded provider** - `98b65d6` (fix)

**Plan metadata:** _(pending final commit)_

## Files Created/Modified
- `src/components/settings/post-processing/SmartModesSection.tsx` - Added `isEmbeddedActive` gate; `activeModelName` now null for non-embedded providers; `activeIsRecommended` gated on embedded + gemma-3-4b

## Decisions Made
None - plan specified exact replacement block; executed as written.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Gap [C7] closed: translation engine modal + section header will show correct provider label for non-embedded providers
- Remaining gaps before Phase 13 closure: [B5] duplicate shortcut backend rejection, [D8] seeded mode name i18n
- No regressions: embedded provider path (Qwen/Gemma name + recommended badge) unchanged

---
*Phase: 13-smart-modes-ui-translation-presets-i18n*
*Completed: 2026-06-05*
