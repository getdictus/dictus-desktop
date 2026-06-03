---
phase: 13-smart-modes-ui-translation-presets-i18n
plan: 03
subsystem: ui
tags: [react, typescript, tauri, i18n, smart-modes, translation]

# Dependency graph
requires:
  - phase: 13-01
    provides: setSmartModeBinding, setTranslationEngineChoice backend commands, TranslationEngineChoice type
  - phase: 13-02
    provides: all smartModes.* i18n keys in en + 19 non-English locales
  - phase: 12
    provides: addSmartMode, updateSmartMode, deleteSmartMode, listSmartModes, SmartMode CRUD backend

provides:
  - SmartModeShortcutChip: per-card key capture routing through setSmartModeBinding with inline conflict (role=alert)
  - SmartModeCard: collapsed + inline-expanded card for create/edit/delete across Rewrite and Translation kinds
  - TranslationEngineChoiceModal: in-page modal for TranslateGemma download vs generic engine choice
  - SmartModesSection: top-level layout with Rewrite + Translation sections, create buttons, pre-enable CTA

affects:
  - 13-04 (mounts SmartModesSection into PostProcessingSettings; removes old Prompts group)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "SmartModeShortcutChip adapts GlobalShortcutInput mechanics (keydown/keyup) but routes through setSmartModeBinding, not updateBinding"
    - "Seeded mode IDs mapped to i18n keys at display time only — SEEDED_MODE_ID_TO_I18N_KEY constant in SmartModeCard"
    - "Translation section disabled state: aria-disabled + opacity-60 on card div, not HTML disabled attr"
    - "Language lists inlined in SmartModesSection: TRANSLATE_GEMMA_LANGUAGES (~40) + GENERIC_LANGUAGES (broader)"

key-files:
  created:
    - src/components/settings/post-processing/SmartModeShortcutChip.tsx
    - src/components/settings/post-processing/SmartModeCard.tsx
    - src/components/settings/post-processing/TranslationEngineChoiceModal.tsx
    - src/components/settings/post-processing/SmartModesSection.tsx
  modified: []

key-decisions:
  - "SmartModeShortcutChip does NOT call updateBinding or useSettings; all binding state goes through commands.setSmartModeBinding"
  - "suspend/resume binding is best-effort (catch(() => {})) because smart_mode_{id} entry only exists after first bind"
  - "dir=ltr on shortcut chip element prevents RTL locales from reversing key combo display"
  - "TranslationEngineChoiceModal uses fixed positioning overlay (not inline panel) — simpler and unambiguous; does not navigate away"
  - "SmartModesSection renders custom h2 instead of SettingsGroup to support dynamic text-amber-500 color on Translation header"
  - "Language lists defined inline in SmartModesSection — no external file needed; minimal but real (40+ codes)"
  - "Close button on modal uses common.close i18n key and lucide X icon to avoid hardcoded string ESLint violation"

patterns-established:
  - "Shortcut chip pattern: isRecording local state + keydown/keyup window listeners + setSmartModeBinding commit on key release"
  - "Card expand pattern: isEditing local state, always-expanded when mode===null (create)"
  - "Conflict display pattern: role=alert p element below chip, cleared on new bind attempt"

requirements-completed: [MODE-03, MODE-04, MODE-05, MODE-06, TRANS-01]

# Metrics
duration: 15min
completed: 2026-06-03
---

# Phase 13 Plan 03: Smart Modes UI Components Summary

**Four React components delivering the full Smart Modes UI surface: per-card shortcut chip (setSmartModeBinding + inline conflict), inline-editable card, TranslateGemma/generic engine choice modal, and Rewrite + Translation section layout with pre-enable amber CTA**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-06-03T19:44:00Z
- **Completed:** 2026-06-03T19:59:24Z
- **Tasks:** 3
- **Files modified:** 4 (all created)

## Accomplishments

- SmartModeShortcutChip: four visual states (unbound/bound/recording/conflict), routes through setSmartModeBinding never updateBinding, dir=ltr for RTL safety, role=alert conflict text
- SmartModeCard: collapsed + inline-expanded edit form for both Rewrite (name+prompt) and Translation (name+language select), full CRUD via Phase 12 commands, seeded name localization via SEEDED_MODE_ID_TO_I18N_KEY, ask() delete dialog
- TranslationEngineChoiceModal: TranslateGemma download-with-progress vs generic model option, persists via setTranslationEngineChoice, uses common.close i18n key for close button
- SmartModesSection: Rewrite + Translation groups, create buttons, amber section header when translation not configured, enable CTA box, language lists (TRANSLATE_GEMMA_LANGUAGES ~40 + GENERIC_LANGUAGES broader)

## Task Commits

Each task was committed atomically:

1. **Task 1: SmartModeShortcutChip** - `570afe8` (feat)
2. **Task 2: SmartModeCard** - `05874c2` (feat)
3. **Task 3: TranslationEngineChoiceModal + SmartModesSection** - `cff4195` (feat)

## Files Created/Modified

- `src/components/settings/post-processing/SmartModeShortcutChip.tsx` - Inline shortcut capture chip, 213 lines
- `src/components/settings/post-processing/SmartModeCard.tsx` - Collapsed + expanded card with CRUD, 338 lines
- `src/components/settings/post-processing/TranslationEngineChoiceModal.tsx` - Engine choice modal with download progress, 134 lines
- `src/components/settings/post-processing/SmartModesSection.tsx` - Top-level section layout, 238 lines

## Decisions Made

- SmartModeShortcutChip adapts GlobalShortcutInput mechanics but routes through setSmartModeBinding (not updateBinding) per plan requirement
- suspend/resume binding is best-effort since smart_mode_{id} entry only exists after first bind; skipping on unbound modes avoids spurious errors
- dir=ltr on the chip element prevents RTL locales from reversing key combo display
- TranslationEngineChoiceModal uses fixed overlay positioning (not inline panel) — simplest approach that satisfies "must not navigate away" constraint
- SmartModesSection renders custom h2 elements instead of SettingsGroup wrapper to support dynamic amber/mid-gray color on Translation header (SettingsGroup hardcodes text-mid-gray)
- Close button on modal uses lucide X icon with t("common.close") aria-label to satisfy ESLint i18next/no-literal-string rule

## Deviations from Plan

None - plan executed exactly as written. Minor implementation decisions (modal positioning style, custom h2 vs SettingsGroup) were within Claude's Discretion scope defined in the plan.

## Issues Encountered

- ESLint `react-hooks/exhaustive-deps` rule not installed in project — removed the inline disable comment and used full deps array instead
- ESLint `i18next/no-literal-string` caught hardcoded `✕` and `"Close"` aria-label in modal close button — fixed by switching to lucide X icon + `t("common.close")`

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- All 4 components ready to mount into PostProcessingSettings.tsx (plan 13-04)
- SmartModesSection mounts below Engine group, replacing old Prompts group
- transcribe_with_post_process shortcut row removal from PostProcessingSettings is a 13-04 task
- Translation section fully functional once user triggers modal and chooses engine

---
*Phase: 13-smart-modes-ui-translation-presets-i18n*
*Completed: 2026-06-03*
