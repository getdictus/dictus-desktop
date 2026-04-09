---
phase: 02-visual-rebrand
plan: 05
subsystem: i18n
tags: [branding, i18n, locale, rebrand, gap-closure]
requirements: [VISU-04, LANG-01]

dependency_graph:
  requires: [02-03-SUMMARY.md]
  provides:
    [zero-handy-refs-all-20-locales, appLanguage-auto-clarification-all-locales]
  affects: [i18next runtime, all language settings UI]

tech_stack:
  added: []
  patterns: [direct JSON string replacement, no transliteration of proper noun]

key_files:
  created: []
  modified:
    - src/i18n/locales/pl/translation.json
    - src/i18n/locales/de/translation.json
    - src/i18n/locales/ja/translation.json
    - src/i18n/locales/it/translation.json
    - src/i18n/locales/ko/translation.json
    - src/i18n/locales/ru/translation.json
    - src/i18n/locales/zh/translation.json
    - src/i18n/locales/zh-TW/translation.json
    - src/i18n/locales/sv/translation.json
    - src/i18n/locales/he/translation.json
    - src/i18n/locales/ar/translation.json
    - src/i18n/locales/pt/translation.json
    - src/i18n/locales/cs/translation.json
    - src/i18n/locales/uk/translation.json
    - src/i18n/locales/bg/translation.json
    - src/i18n/locales/tr/translation.json

decisions:
  - "Dictus kept as Latin-script proper noun in all 16 locales including RTL (Hebrew, Arabic) and CJK scripts"
  - "Ukrainian shortcut.title already localized (no Handy reference) — 10 replacements instead of 11 in that file"
  - "appLanguage.description Auto clarification phrased naturally in each target language rather than literal translation"

metrics:
  duration: ~25min
  completed: "2026-04-08"
  tasks: 2
  files_modified: 16
---

# Phase 02 Plan 05: i18n Gap Closure — 16 Remaining Locales Summary

**One-liner:** Replaced all "Handy" references with "Dictus" in 16 locale files (pl, de, ja, it, ko, ru, zh, zh-TW, sv, he, ar, pt, cs, uk, bg, tr), satisfying VISU-04 and LANG-01 across all 20 locale files.

## What Was Built

Gap-closure plan that completed the Handy→Dictus rebrand for all non-primary locale files. Plan 02-03 had covered en/es/fr/vi. This plan covered the remaining 16 locales in two batches of 8.

Each file received:

- Replacement of "Handy" with "Dictus" at all occurrences across 11 key paths
- Updated `appLanguage.description` with both Dictus branding and an Auto/system-language clarification in the target language (LANG-01)

All 20 locale files now have zero "Handy" occurrences. VISU-04 is fully satisfied.

## Tasks Completed

| Task | Description                                               | Commit  | Files          |
| ---- | --------------------------------------------------------- | ------- | -------------- |
| 1    | Replace Handy→Dictus in pl, de, ja, it, ko, ru, zh, zh-TW | 8ae594e | 8 locale files |
| 2    | Replace Handy→Dictus in sv, he, ar, pt, cs, uk, bg, tr    | 4e7ddce | 8 locale files |

## Key Changes Per Locale

**Task 1 locales (8):** Polish, German, Japanese, Italian, Korean, Russian, Chinese Simplified, Chinese Traditional

**Task 2 locales (8):** Swedish, Hebrew, Arabic, Portuguese, Czech, Ukrainian, Bulgarian, Turkish

**Key paths updated in each file:**

1. `onboarding.permissions.description`
2. `settings.shortcuts.title` (except uk — already localized)
3. `settings.general.launchAtLogin.description`
4. `settings.general.showTrayIcon.description`
5. `settings.updates.autoCheck.description`
6. `settings.about.version.description`
7. `settings.about.dataLocation.description`
8. `settings.about.supportDevelopment.description`
9. `settings.about.supportDevelopment.credits.details`
10. `accessibilityOnboarding.permissionsDescription`
11. `appLanguage.description` (Dictus + Auto clarification)

## Verification

```
All 20 locales — Handy count: 0
en: 0, es: 0, fr: 0, vi: 0, pl: 0, de: 0, ja: 0, it: 0
ko: 0, ru: 0, zh: 0, zh-TW: 0, sv: 0, he: 0, ar: 0, pt: 0
cs: 0, uk: 0, bg: 0, tr: 0

All 16 gap-closure locale files: valid JSON (python3 json.load)
```

## Deviations from Plan

**1. [Rule 1 - Observation] Ukrainian shortcut.title already localized**

- **Found during:** Task 2
- **Issue:** `uk/translation.json` shortcut.title reads "Комбінації клавіш" (no "Handy") — already properly localized in a prior edit outside this plan's scope
- **Fix:** No action needed; 10 replacements made instead of 11
- **Files modified:** None extra
- **Commit:** N/A (not a deviation requiring a fix)

## Requirements Closed

- **VISU-04:** All 20+ locale files updated — fully satisfied
- **LANG-01:** `appLanguage.description` in all non-English locales includes Auto/system-language clarification — fully satisfied

## Self-Check: PASSED

- SUMMARY.md: FOUND at .planning/phases/02-visual-rebrand/02-05-SUMMARY.md
- Commit 8ae594e: FOUND (Task 1 — pl, de, ja, it, ko, ru, zh, zh-TW)
- Commit 4e7ddce: FOUND (Task 2 — sv, he, ar, pt, cs, uk, bg, tr)
- All 20 locales: 0 Handy occurrences confirmed
- All 16 gap-closure files: valid JSON confirmed
