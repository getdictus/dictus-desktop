---
phase: 13-smart-modes-ui-translation-presets-i18n
plan: 19
subsystem: ui
tags: [react, typescript, tauri, i18n, translation, llm, rust]

# Dependency graph
requires:
  - phase: 13-smart-modes-ui-translation-presets-i18n
    provides: 13-17 provider-switch commands (set_translation_engine_to_embedded, restore_translation_engine_provider) being reverted
provides:
  - Recommendation-only translation modal: no global-model mutation, single enable CTA that persists generic_model choice
  - Backend provider-switch surface removed (commands, settings field, lib.rs registrations, bindings.ts)
  - 4 new i18n keys across all 20 locales with English fallback
affects:
  - translation settings UI
  - SmartModesSection translation header
  - bindings.ts consumers

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Translation enable = setTranslationEngineChoice('generic_model') only — no provider mutation"
    - "Modal is informational recommendation panel, not a model switcher"
    - "English-fallback precedent maintained for new i18n keys across 19 non-en locales"

key-files:
  created: []
  modified:
    - src-tauri/src/commands/llm.rs
    - src-tauri/src/settings.rs
    - src-tauri/src/lib.rs
    - src/bindings.ts
    - src/components/settings/post-processing/TranslationEngineChoiceModal.tsx
    - src/components/settings/post-processing/SmartModesSection.tsx
    - src/i18n/locales/en/translation.json
    - src/i18n/locales/ar/translation.json
    - src/i18n/locales/bg/translation.json
    - src/i18n/locales/cs/translation.json
    - src/i18n/locales/de/translation.json
    - src/i18n/locales/es/translation.json
    - src/i18n/locales/fr/translation.json
    - src/i18n/locales/he/translation.json
    - src/i18n/locales/it/translation.json
    - src/i18n/locales/ja/translation.json
    - src/i18n/locales/ko/translation.json
    - src/i18n/locales/pl/translation.json
    - src/i18n/locales/pt/translation.json
    - src/i18n/locales/ru/translation.json
    - src/i18n/locales/sv/translation.json
    - src/i18n/locales/tr/translation.json
    - src/i18n/locales/uk/translation.json
    - src/i18n/locales/vi/translation.json
    - src/i18n/locales/zh/translation.json
    - src/i18n/locales/zh-TW/translation.json

key-decisions:
  - "[G12] Translation engine = recommendation-only. ONE active model chosen in main selector; translation runs through it; modal informs without mutating provider/model state."
  - "set_translation_engine_to_embedded + restore_translation_engine_provider + previous_post_process_provider_id fully removed; set_translation_engine_choice retained (enable flag only)"
  - "bindings.ts regenerated automatically by cargo build (specta export runs in debug builds)"

patterns-established:
  - "Translation enable CTA calls setTranslationEngineChoice('generic_model') only — zero side effects on active model or provider"

requirements-completed: [TRANS-01, TRANS-02]

# Metrics
duration: 15min
completed: 2026-06-08
---

# Phase 13 Plan 19: [G12] Translation Engine Recommendation-Only Summary

**Reverted 13-17's global-model-switch backend surface and converted the translation modal to an informational recommendation panel with a single enable CTA that persists `generic_model` without mutating the active provider**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-06-08T~16:40Z
- **Completed:** 2026-06-08T~16:55Z
- **Tasks:** 3
- **Files modified:** 24

## Accomplishments

- Removed `set_translation_engine_to_embedded`, `restore_translation_engine_provider`, and `previous_post_process_provider_id` from Rust backend (commands/llm.rs, settings.rs, lib.rs); bindings.ts regenerated clean
- Converted `TranslationEngineChoiceModal` from a two-option model-switcher to an informational recommendation panel (Gemma 3 4B) with optional download UI and a single "Enable offline translation" CTA that only persists the enable flag
- Added 4 new i18n keys (`recommendNote`, `modal.recommendOnlyHeading`, `modal.recommendOnlyBody`, `modal.enableCta`) across all 20 locales; `bun run check:translations` passes

## Task Commits

1. **Task 1: Revert 13-17 backend provider-switch + regenerate bindings.ts** - `829ce52` (feat)
2. **Task 2: Convert TranslationEngineChoiceModal to recommendation-only** - `9ab61fd` (feat)
3. **Task 3: Add recommendation i18n keys across en + 19 non-en locales** - `c3a2b29` (feat)

## Files Created/Modified

- `src-tauri/src/commands/llm.rs` - Removed set_translation_engine_to_embedded + restore_translation_engine_provider
- `src-tauri/src/settings.rs` - Removed previous_post_process_provider_id field and its default constructor entry
- `src-tauri/src/lib.rs` - Removed the two commands from invoke_handler registration
- `src/bindings.ts` - Regenerated: removed symbols gone, setTranslationEngineChoice retained, previous_post_process_provider_id gone from AppSettings type
- `src/components/settings/post-processing/TranslationEngineChoiceModal.tsx` - Full rewrite: recommendation-only panel, no model-switch buttons
- `src/components/settings/post-processing/SmartModesSection.tsx` - Removed unused LlmModelStore subscriptions + derived state; replaced "Change engine" button with recommendNote span; dropped removed modal props
- `src/i18n/locales/*/translation.json` - 4 new keys added to all 20 locales (en + 19 non-en with English fallback)

## Decisions Made

- Translation engine = recommendation-only per design decision 2026-06-08 (UAT test 11): the modal recommends Gemma 3 4B and points to the main model selector, but does NOT switch the active model or provider
- Existing keys (`modal.title`, `modal.downloadRecommended`, `modal.footnote`, `modal.downloadingPercent`, `modal.verifying`, `modal.downloadBackgroundNote`) retained — still referenced by the reworked modal; no key deletion needed
- `modal.applying` and `modal.applyFailed` retained (used by the enable CTA path)

## Deviations from Plan

None - plan executed exactly as written. The bindings.ts regeneration occurred automatically via the `cargo build` step (specta exports on debug build startup — triggered by cargo compile step which invoked the build script).

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- [G12] closed: translation modal/section is recommendation-only, no global model mutation
- [G11] (combo prefix/base-key collision) remains open — separate plan 13-18
- Phase 13 will be complete once [G11] is also closed
- cargo build clean, lint clean, check:translations green, TypeScript clean

---
*Phase: 13-smart-modes-ui-translation-presets-i18n*
*Completed: 2026-06-08*
