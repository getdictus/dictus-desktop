---
phase: 13-smart-modes-ui-translation-presets-i18n
plan: 17
subsystem: ui
tags: [react, typescript, smart-modes, translation, llm, embedded, apple-intelligence, rust, tauri]

# Dependency graph
requires:
  - phase: 13-smart-modes-ui-translation-presets-i18n
    provides: SmartModesSection badge gate isEmbeddedActive = providerId === 'embedded' (13-14)
  - phase: 13-smart-modes-ui-translation-presets-i18n
    provides: TranslationEngineChoiceModal active-badge props (13-12)
provides:
  - Backend commands set_translation_engine_to_embedded / restore_translation_engine_provider (flip post_process_provider_id to/from 'embedded', remembering prior provider)
  - settings.previous_post_process_provider_id field
  - Modal buttons wired to the provider-switch commands; bindings.ts regenerated
affects: [translation-engine-modal, smart-modes-section]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Provider-switch command pattern: remember previous_post_process_provider_id before forcing 'embedded', restore on reverse"

key-files:
  created: []
  modified:
    - src-tauri/src/settings.rs
    - src-tauri/src/commands/llm.rs
    - src-tauri/src/lib.rs
    - src/bindings.ts
    - src/components/settings/post-processing/TranslationEngineChoiceModal.tsx

key-decisions:
  - "[13-17] set_translation_engine_to_embedded flips post_process_provider_id='embedded' + active_llm_model_id='gemma-3-4b' and stashes the prior provider in previous_post_process_provider_id; restore_translation_engine_provider reverses it"
  - "[13-17 SUPERSEDED] Live UAT rejected the global-mutation UX — switching the translation engine to Gemma re-pointed ALL Smart Modes at Gemma. Decision 2026-06-08: translation engine becomes recommendation-only (single active model chosen via the main selector); the provider-switch approach is replaced by gap [G12]."

patterns-established: []

requirements-completed: [TRANS-02]

# Metrics
duration: ~3min
completed: 2026-06-08
---

# Phase 13 Plan 17: Translation Engine Provider Switch — Built, Then Superseded

**Backend provider-switch commands + modal wiring make 'Use Gemma 3 4B' flip post_process_provider_id to 'embedded' and persist; live UAT confirmed it works but rejected the UX (it re-points every Smart Mode at Gemma) → replaced by a recommendation-only redesign [G12]**

## Performance

- **Duration:** ~3 min (code tasks); live verification + design decision by user
- **Completed:** 2026-06-08
- **Tasks:** 2 auto + 1 human-verify checkpoint
- **Files modified:** 5

## Accomplishments
- `settings.rs`: added `previous_post_process_provider_id: Option<String>` (`#[serde(default)]`) + default constructor entry
- `commands/llm.rs`: `set_translation_engine_to_embedded(model_id)` loads the GGUF, stashes the prior provider, sets `post_process_provider_id="embedded"` + `active_llm_model_id="gemma-3-4b"` + `translation_engine_choice=GenericModel`; `restore_translation_engine_provider()` restores the stashed provider
- Both commands registered in `lib.rs`; `bindings.ts` regenerated (`setTranslationEngineToEmbedded` @812, `restoreTranslationEngineProvider` @820)
- `TranslationEngineChoiceModal.tsx`: Gemma button → `setTranslationEngineToEmbedded`, current-model button → `restoreTranslationEngineProvider`; `setActiveModel`-only Gemma path removed

## Task Commits

1. **Task 1: Backend provider-switch commands + bindings regen** - `ef8e559` (feat)
2. **Task 2: Frontend modal button wiring** - `30b6078` (feat)

## Files Created/Modified
- `src-tauri/src/settings.rs` - `previous_post_process_provider_id` field
- `src-tauri/src/commands/llm.rs` - two provider-switch commands
- `src-tauri/src/lib.rs` - command registration
- `src/bindings.ts` - regenerated bindings
- `src/components/settings/post-processing/TranslationEngineChoiceModal.tsx` - buttons wired to new commands

## Decisions Made
See SUPERSEDED decision in frontmatter. The implementation is mechanically correct and verified live, but the product direction changed during verification.

## Deviations from Plan
None during execution.

## Live Verification (Task 3 checkpoint)
**Result: WORKS MECHANICALLY, UX REJECTED → SUPERSEDED.** (user, macOS, 2026-06-08)

Choosing "Use Gemma 3 4B" under Apple Intelligence did flip the provider to embedded Gemma and persist; "Use your current model" restored Apple. **But** the user observed this switches the active model for **every** Smart Mode — not just translation — because the app has a single active post-process model, and the button mutates that global state.

**Design decision (user, 2026-06-08):** Do not ship a second model-switcher inside the translation modal. The translation engine becomes **recommendation-only**: one active model chosen via the main selector; the modal shows an informational recommendation ("Gemma 3 4B recommended for translation"). Two-models-in-RAM routing rejected for v1.3 (memory cost + contradicts the established "translation routes through the active LLM model" decision). Recorded as gap **[G12]**, which will revert the global-mutation path from this plan and convert the modal to informational.

## User Setup Required
None.

## Next Phase Readiness
- Gap [F10]'s mechanical "switch + persist" is achievable but **intentionally not shipped** as built.
- Follow-up gap [G12] (translation engine → recommendation-only; revert 13-17 global mutation) queued for the next gap-closure round.

---
*Phase: 13-smart-modes-ui-translation-presets-i18n*
*Completed: 2026-06-08*
