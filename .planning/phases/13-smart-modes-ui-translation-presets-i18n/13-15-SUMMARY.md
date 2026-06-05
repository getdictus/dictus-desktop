---
phase: 13-smart-modes-ui-translation-presets-i18n
plan: 15
subsystem: ui
tags: [i18n, localization, smart-modes, translation-json]

requires:
  - phase: 13-smart-modes-ui-translation-presets-i18n
    provides: "SmartModeCard.tsx SEEDED_MODE_ID_TO_I18N_KEY map and t() lookup (13-07); smartModes.defaultModes.* keys with English fallback values (13-02)"

provides:
  - "Native-language values for the 10 smartModes.defaultModes.* name keys in all 19 non-en locales"
  - "Closes gap [D8] (UAT test 3): seeded Smart Mode names render in the active app language"

affects:
  - "13-UAT.md — test 3 [D8] can be marked resolved"
  - "SmartModeCard.tsx — no code change needed; i18n data layer was the only gap"

tech-stack:
  added: []
  patterns:
    - "Names-only scope: only smartModes.defaultModes.* values changed; prompt/description bodies remain in English by explicit user decision"

key-files:
  created: []
  modified:
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
  - "Names-only scope confirmed: only smartModes.defaultModes.* values translated; prompt/description bodies remain English by user decision (2026-06-05)"
  - "He writeAsEmail uses quote-free form (כתוב כאימייל) to avoid inner gershayim double-quote breaking JSON"

patterns-established:
  - "i18n data patches: use a temp .cjs script that loads JSON, mutates only the target subtree, and writes back with JSON.stringify(obj, null, 2) + newline — guarantees no structural drift; script deleted after use"

requirements-completed: [MODE-06, L10N-01]

duration: 2min
completed: 2026-06-05
---

# Phase 13 Plan 15: Smart Mode Names i18n (Gap [D8]) Summary

**Native-language seeded Smart Mode name strings for all 19 non-English locales — closes UAT gap [D8] with zero code changes (architecture was already correct)**

## Performance

- **Duration:** ~2 min
- **Started:** 2026-06-05T20:25:28Z
- **Completed:** 2026-06-05T20:27:08Z
- **Tasks:** 1
- **Files modified:** 19

## Accomplishments

- Replaced English fallback values with native translations for the 10 `smartModes.defaultModes.*` name keys in all 19 non-en locales (ar, bg, cs, de, es, fr, he, it, ja, ko, pl, pt, ru, sv, tr, uk, vi, zh, zh-TW)
- `bun run check:translations` exits 0 at full 519-key count — no keys added or removed
- Spot checks confirmed: `fr.cleanUp` = "Nettoyage", `es.summarize` = "Resumir", `en.cleanUp` = "Clean Up" (unchanged)
- Hebrew `writeAsEmail` uses quote-free form `כתוב כאימייל` to avoid JSON-breaking gershayim character

## Task Commits

1. **Task 1: Translate 10 seeded mode NAME keys across all 19 non-en locales** - `5d9a956` (feat)

## Files Created/Modified

- `src/i18n/locales/{ar,bg,cs,de,es,fr,he,it,ja,ko,pl,pt,ru,sv,tr,uk,vi,zh,zh-TW}/translation.json` — `smartModes.defaultModes.*` values replaced with native-language strings; all other keys untouched

## Decisions Made

- Names-only scope: prompt/description bodies remain in English by explicit user decision (2026-06-05). Only the 10 `smartModes.defaultModes.*` name values were changed.
- Hebrew writeAsEmail uses `כתוב כאימייל` (quote-free) rather than `כתוב כדוא"ל` to avoid the inner gershayim double-quote breaking JSON without escaping.

## Deviations from Plan

None - plan executed exactly as written. A temp `.cjs` script was used for the bulk mutation and deleted immediately after (workspace hygiene as instructed by the plan).

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Gap [D8] (UAT test 3) is resolved: seeded Smart Mode names now render in the active app language
- Two gaps remain before Phase 13 closure: [B5] duplicate shortcut rejection (13-13) and [C7] engine modal provider-gated display (13-14)

---
*Phase: 13-smart-modes-ui-translation-presets-i18n*
*Completed: 2026-06-05*
