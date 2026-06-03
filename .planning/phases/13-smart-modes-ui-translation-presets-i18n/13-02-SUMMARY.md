---
phase: 13-smart-modes-ui-translation-presets-i18n
plan: "02"
subsystem: ui
tags: [i18n, localization, react, smart-modes, translation]

# Dependency graph
requires: []
provides:
  - smartModes.* i18n namespace in all 20 locales (en + 19 non-English) with English fallback values
  - translateGemma4b.description in library.models across all 20 locales
  - L10N-01 gate: bun run check:translations passes at 490 keys per locale
affects:
  - 13-03 (SmartModesSection UI — uses t('smartModes.*') keys defined here)
  - 13-04 (translation engine UI — uses t('smartModes.translation.*') keys)

# Tech tracking
tech-stack:
  added: []
  patterns: [English-fallback for new i18n namespaces — propagate en values verbatim to all locales; real translations deferred]

key-files:
  created: []
  modified:
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
  - "English fallback pattern: all 19 non-English locales receive en values verbatim for new keys — real translations deferred (Phase 11 precedent, L10N deferred)"
  - "smartModes namespace appended at top level after overlay key — consistent with existing namespace ordering"
  - "translateGemma4b.description added as sibling of qwen25_1b5/gemma3_4b/phi4_mini/llama32_3b in library.models"

patterns-established:
  - "New i18n namespaces: add to en first, then mirror exact key structure to all 19 locales using English values as placeholders"

requirements-completed: [L10N-01]

# Metrics
duration: 5min
completed: "2026-06-03"
---

# Phase 13 Plan 02: i18n Keys for Smart Modes UI Summary

**50 new smartModes.* i18n keys + translateGemma4b catalogue description added to en and propagated to all 19 non-English locales with English fallback values; bun run check:translations passes at 490 keys**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-06-03T19:06:13Z
- **Completed:** 2026-06-03T19:11:00Z
- **Tasks:** 2
- **Files modified:** 20

## Accomplishments

- Added `smartModes` namespace to en/translation.json with all Phase 13 UI strings: sections, card CTAs (saveCta, discardCta, createCta, discardNewCta, addShortcut, shortcutConflict, targetLanguage), kind badges, translation enable/engine modal strings, delete dialog, noModel empty state, and 10 defaultModes entries
- Added `translateGemma4b.description` under `settings.postProcessing.modelsAndLocalProcessing.library.models` alongside existing qwen25_1b5/gemma3_4b/phi4_mini/llama32_3b entries
- Propagated the identical key structure (English values as fallback) to all 19 non-English locales (ar, bg, cs, de, es, fr, he, it, ja, ko, pl, pt, ru, sv, tr, uk, vi, zh, zh-TW); check:translations green at 490 keys per locale

## Task Commits

1. **Task 1: Add all smartModes.* keys to en/translation.json** - `59764a8` (feat)
2. **Task 2: Propagate identical keys to all 19 non-English locales** - `7faf6d9` (feat)

## Files Created/Modified

- `src/i18n/locales/en/translation.json` - Added smartModes namespace (50 keys) + translateGemma4b.description
- `src/i18n/locales/{ar,bg,cs,de,es,fr,he,it,ja,ko,pl,pt,ru,sv,tr,uk,vi,zh,zh-TW}/translation.json` - Mirrored smartModes namespace + translateGemma4b.description with English fallback values

## Decisions Made

- English fallback values used verbatim in all 19 non-English locales — consistent with Phase 11 precedent (real translations deferred, L10N-01 gate requires structural parity only)
- `smartModes` namespace placed at the end of en/translation.json after `overlay` — follows the existing pattern of appending new top-level namespaces rather than inserting alphabetically mid-file
- Used a one-off Node.js script (`/tmp/merge-locales.cjs`) to merge keys reliably; script placed in /tmp (not in the repo) and not committed — Workspace Hygiene compliant

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- All Phase 13 i18n keys are available for plans 13-03 and 13-04 to reference via `t('smartModes.*')` without inventing or duplicating keys
- `bun run check:translations` and `bun run lint` both exit 0; no regressions introduced
- L10N-01 requirement satisfied

---
*Phase: 13-smart-modes-ui-translation-presets-i18n*
*Completed: 2026-06-03*
