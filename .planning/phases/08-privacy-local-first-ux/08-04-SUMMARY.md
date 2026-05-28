---
phase: 08-privacy-local-first-ux
plan: "04"
subsystem: i18n
tags: [i18n, localization, react, privacy, local-first]

# Dependency graph
requires:
  - phase: 08-privacy-local-first-ux
    plan: "03"
    provides: New English i18n keys added in Plan 03
provides:
  - Full i18n parity: all 20 locale files contain the 14 new Phase 8 keys
  - bun run check:translations exits 0
affects:
  - 08-05-PLAN (validation can now verify i18n-dependent UI strings in all locales)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - English fallback pattern for technical debug strings (simulateUpdaterRestart)
    - Brand names preserved verbatim across all locales per CONTRIBUTING_TRANSLATIONS.md
    - JSX component placeholders (<link>, <code>) and interpolation ({{count}}, {{baseUrl}}) preserved verbatim

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
  - "simulateUpdaterRestart keys kept in English (technical debug feature — English fallback is acceptable per CONTRIBUTING_TRANSLATIONS.md for technical strings)"
  - "Pre-existing missing keys (restartHint, restart, simulateUpdaterRestart.*) backfilled as part of achieving full parity — deviation Rule 2 (auto-add missing critical functionality)"

# Metrics
duration: ~4min
completed: 2026-05-21
---

# Phase 8 Plan 4: Privacy / Local-First UX — i18n Locale Propagation Summary

**Full i18n parity achieved for 14 new Phase 8 keys across all 19 sibling locales; 5 pre-existing missing keys also backfilled**

## Performance

- **Duration:** ~4 min
- **Started:** 2026-05-21T15:38:55Z
- **Completed:** 2026-05-21T15:42:28Z
- **Tasks:** 1
- **Files modified:** 19

## Accomplishments

- Added all 14 Phase 8 keys from Plan 03 to all 19 non-English locale files (ar, bg, cs, de, es, fr, he, it, ja, ko, pl, pt, ru, sv, tr, uk, vi, zh, zh-TW)
- Translated all user-facing strings using the vetted translation table from the plan; brand names (Ollama, Apple Intelligence, macOS Apple Silicon) preserved verbatim
- JSX placeholders (`<link>Ollama</link>`, `<code>ollama serve</code>`) and interpolation tokens (`{{count}}`, `{{baseUrl}}`) preserved verbatim in all 19 locales
- Key insertion order matches EN structure: `providers` after `provider`, `custom` after `providers`, `networkSurface` after `privacy`, `postProcessing` inside `groups` after `transcription`
- `bun run check:translations` exits 0 (all 19 languages pass, 413/413 keys)
- `bun run build` exits 0 (Vite + TypeScript build green)
- Prettier check passes for all locale JSON files

## Task Commits

1. **Task 1: Replicate keys across 19 sibling locales** - `c5c95e0` (feat)

## 19 Locale Files Modified

| Locale | sectionLocal (translated) | groups.postProcessing |
| ------ | ------------------------- | --------------------- |
| de     | Auf deinem Gerät          | Nachbearbeitung       |
| fr     | Sur votre appareil        | Post-traitement       |
| es     | En tu dispositivo         | Postprocesamiento     |
| it     | Sul tuo dispositivo       | Post-elaborazione     |
| pt     | No seu dispositivo        | Pós-processamento     |
| ja     | お使いのデバイスで        | 後処理                |
| ko     | 내 기기에서               | 후처리                |
| zh     | 在你的设备上              | 后处理                |
| zh-TW  | 在你的裝置上              | 後處理                |
| ru     | На вашем устройстве       | Постобработка         |
| uk     | На вашому пристрої        | Постобробка           |
| pl     | Na twoim urządzeniu       | Przetwarzanie końcowe |
| cs     | Na vašem zařízení         | Následné zpracování   |
| bg     | На вашето устройство      | Последваща обработка  |
| sv     | På din enhet              | Efterbehandling       |
| tr     | Cihazında                 | Son işleme            |
| vi     | Trên thiết bị của bạn     | Hậu xử lý             |
| ar     | على جهازك                 | المعالجة اللاحقة      |
| he     | במכשיר שלך                | עיבוד בדיעבד          |

## check:translations Output

```
✓ AR: All keys present
✓ BG: All keys present
✓ CS: All keys present
✓ DE: All keys present
✓ ES: All keys present
✓ FR: All keys present
✓ HE: All keys present
✓ IT: All keys present
✓ JA: All keys present
✓ KO: All keys present
✓ PL: All keys present
✓ PT: All keys present
✓ RU: All keys present
✓ SV: All keys present
✓ TR: All keys present
✓ UK: All keys present
✓ VI: All keys present
✓ ZH: All keys present
✓ ZH-TW: All keys present
✓ All 19 languages have complete translations!
```

## Decisions Made

- **simulateUpdaterRestart in English:** This is a technical debug feature (`settings.debug.simulateUpdaterRestart`) describing Tauri internals (SHUT-03 validation). English fallback is acceptable per CONTRIBUTING_TRANSLATIONS.md for strings where machine translation would be unsafe for technical UI. Applied to all 19 locales.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical Functionality] Backfilled 5 pre-existing missing keys**

- **Found during:** Task 1 — `bun run check:translations` failed with 5 missing keys per locale (before Phase 8 keys were added)
- **Missing keys:** `onboarding.permissions.accessibility.restartHint`, `onboarding.permissions.accessibility.restart`, `settings.debug.simulateUpdaterRestart.title`, `settings.debug.simulateUpdaterRestart.description`, `settings.debug.simulateUpdaterRestart.button`
- **Fix:** Added translated `restartHint` / `restart` (user-facing accessibility strings) and English-fallback `simulateUpdaterRestart.*` (technical debug) to all 19 locales
- **Files modified:** All 19 locale files (included in the same commit)
- **Commit:** c5c95e0

These were pre-existing gaps that prevented `check:translations` from passing. Fixing them was required to satisfy the plan's acceptance criteria (`bun run check:translations` exits 0).

### Out of Scope (Not Fixed)

- Pre-existing ESLint `i18next/no-literal-string` error in `src/components/icons/DictusLogo.tsx` (SVG brand text "Dictus") — documented in Phase 8 Plan 03 SUMMARY. Not caused by this plan, not fixed here.
- Pre-existing Prettier format issue in `.planning/phases/08-privacy-local-first-ux/08-03-SUMMARY.md` — planning document from Plan 03, not a source file. Not caused by this plan.

## Next Phase Readiness

- All 20 locale files have full parity for all 413 keys including the 14 new Phase 8 keys
- Plan 05 (validation) can verify the full Phase 8 UI surface including i18n rendering across locales
- No remaining i18n gaps known in the 20 managed locale files

---

_Phase: 08-privacy-local-first-ux_
_Completed: 2026-05-21_
