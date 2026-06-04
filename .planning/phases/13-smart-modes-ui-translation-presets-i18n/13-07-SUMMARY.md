---
phase: 13-smart-modes-ui-translation-presets-i18n
plan: "07"
subsystem: ui
tags: [react, typescript, i18n, smart-modes, tailwind]

requires:
  - phase: 13-smart-modes-ui-translation-presets-i18n
    plan: "05"
    provides: smart_mode_templates Rust command, seeded mode defaults, clean-up-only first-run seed
  - phase: 13-smart-modes-ui-translation-presets-i18n
    plan: "06"
    provides: SmartModeShortcutChip component (chip placement is in-header per this plan)

provides:
  - edit-aware displayName in SmartModeCard (localizes only pristine seeded modes)
  - smartModes.card.nameLabel + promptLabel i18n keys replacing legacy settings.postProcessing paths
  - SmartModeTemplatePicker component with modal-overlay style, template list, Custom path
  - create buttons open picker instead of inline blank card
  - always-available change-translation-engine affordance in Translation section header
  - 6 new i18n keys (nameLabel, promptLabel, picker.*, translation.changeEngine/currentEngine/engineGemma/engineGeneric) propagated to 20 locales
  - smartModeTemplates() binding added to bindings.ts

affects: [13-08-PLAN, future smart-modes UAT, i18n locales]

tech-stack:
  added: []
  patterns:
    - "Pristine-seeded detection: compare mode.name to SEEDED_MODE_DEFAULT_NAME before localizing"
    - "SEEDED_MODE_ID_TO_I18N_KEY exported from SmartModeCard for reuse in picker"
    - "SmartModeTemplatePicker: addSmartMode creates new id (no seeded id reuse) so delete+readd works"

key-files:
  created:
    - src/components/settings/post-processing/SmartModeTemplatePicker.tsx
  modified:
    - src/components/settings/post-processing/SmartModeCard.tsx
    - src/components/settings/post-processing/SmartModesSection.tsx
    - src/bindings.ts
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
  - "SEEDED_MODE_ID_TO_I18N_KEY exported from SmartModeCard so SmartModeTemplatePicker can localize template names without duplicating the map"
  - "SmartModeTemplatePicker uses addSmartMode (not update) so picking a template always creates a new mode_timestamp id — prevents id collision when user deleted a default and recreates it"
  - "smartModeTemplates() added manually to bindings.ts (returns SmartMode[] not Result) since tauri-specta regeneration not run; follows the same infallible-command pattern as getAvailableTypingTools()"
  - "Picker insertion in non-EN locales done between card and shortcut sections using precise regex on promptLabel close pattern"

requirements-completed: [MODE-03, MODE-06, TRANS-01, TRANS-02, L10N-01]

duration: ~45min
completed: 2026-06-04
---

# Phase 13 Plan 07: Smart Mode Card + Picker + Engine Toggle Summary

**Edit-aware card naming, SmartModeTemplatePicker overlay with predefined templates + Custom path, always-available translation engine switcher, and 6 new i18n keys across 20 locales**

## Performance

- **Duration:** ~45 min
- **Started:** 2026-06-04T09:15:00Z
- **Completed:** 2026-06-04T10:03:30Z
- **Tasks:** 4
- **Files modified:** 24 (1 created, 23 modified)

## Accomplishments
- Seeded mode cards now localize only when pristine (name === seed default); edited names show the literal stored name
- SmartModeTemplatePicker created: fixed-overlay modal reusing TranslationEngineChoiceModal style, showing 10 predefined templates filtered by kind, with "Added" hint and Custom fallback path
- Create buttons now open the picker (single Plus icon, no textual "+"); Custom routes to existing inline blank-card flow
- Translation section header shows current engine label + "Change engine" button when translation is enabled, making engine re-selection available anytime in both directions

## Task Commits

1. **Task 1: Fix edit-aware title + Prompt/Name labels + chip in header** - `e360f5c` (fix)
2. **Task 2: SmartModeTemplatePicker + wire create buttons** - `77b722f` (feat)
3. **Task 3: Always-available change-translation-engine affordance** - `615b86f` (feat)
4. **Task 4: Propagate i18n keys to 20 locales** - `eb9ce29` (feat)

## Files Created/Modified
- `src/components/settings/post-processing/SmartModeCard.tsx` - Add SEEDED_MODE_DEFAULT_NAME map, edit-aware displayName, export SEEDED_MODE_ID_TO_I18N_KEY, new label keys, chip moved into header row
- `src/components/settings/post-processing/SmartModeTemplatePicker.tsx` - New: picker overlay with template list + Custom entry
- `src/components/settings/post-processing/SmartModesSection.tsx` - Wire picker (pickerOpen state), change-engine affordance in Translation header, mount two SmartModeTemplatePicker instances
- `src/bindings.ts` - Add smartModeTemplates() returning SmartMode[] directly
- `src/i18n/locales/en/translation.json` - Add card.nameLabel, card.promptLabel, picker.*, translation.changeEngine/currentEngine/engineGemma/engineGeneric; remove leading "+" from createRewrite/createTranslation
- `src/i18n/locales/*/translation.json` (19 locales) - Same additions with English fallbacks

## Decisions Made
- Exported `SEEDED_MODE_ID_TO_I18N_KEY` from SmartModeCard so the picker can localize template names without duplicating the map
- `SmartModeTemplatePicker` always calls `addSmartMode` (creates new id) even for predefined templates — prevents id collision if user deleted a default and recreates it
- `smartModeTemplates()` added manually to bindings.ts since it returns `Vec<SmartMode>` (no `Result`), matching the `getAvailableTypingTools()` infallible-command pattern
- Picker section insertion in non-EN locales used a regex matching `"promptLabel": "Prompt"\n    },\n    "shortcut":` to precisely target the smartModes card close

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added smartModeTemplates binding to bindings.ts**
- **Found during:** Task 2 (SmartModeTemplatePicker)
- **Issue:** 13-05 added the Rust command `smart_mode_templates()` but bindings.ts was not regenerated; the binding was absent, blocking the picker
- **Fix:** Manually added `async smartModeTemplates(): Promise<SmartMode[]>` to bindings.ts following the infallible-command pattern
- **Files modified:** src/bindings.ts
- **Verification:** TypeScript compiles, lint passes
- **Committed in:** 77b722f (part of Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Necessary for the picker to call the backend. No scope creep.

## Issues Encountered
- The node regex for inserting the `picker` section into non-EN locales initially targeted the first `"shortcut":` in the file (which was in the `general` section, not `smartModes`). Detected immediately by inspecting fr/translation.json line numbers; resolved by using a more precise two-step approach: remove wrong insertion first, then insert at the exact `"promptLabel" close + "shortcut" start` pattern within smartModes.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All 4 UAT gaps (1, 2, 7, 8b/c) addressed in this plan; ready for 13-08 final UAT checkpoint
- SmartModeTemplatePicker uses `commands.smartModeTemplates()` — the backend command must be running (13-05 prerequisite)
- bun run lint exits 0 and bun run check:translations exits 0

## Self-Check

### Files exist:
- `src/components/settings/post-processing/SmartModeTemplatePicker.tsx` - FOUND
- `src/components/settings/post-processing/SmartModeCard.tsx` - FOUND (modified)
- `src/components/settings/post-processing/SmartModesSection.tsx` - FOUND (modified)

### Commits exist:
- e360f5c - FOUND
- 77b722f - FOUND
- 615b86f - FOUND
- eb9ce29 - FOUND

## Self-Check: PASSED

---
*Phase: 13-smart-modes-ui-translation-presets-i18n*
*Completed: 2026-06-04*
