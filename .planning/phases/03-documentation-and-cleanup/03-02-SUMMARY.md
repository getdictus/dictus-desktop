---
phase: 03-documentation-and-cleanup
plan: 02
subsystem: ui
tags: [i18n, about-panel, locales, branding, dictus]

# Dependency graph
requires: []
provides:
  - About panel with Dictus URLs (donate, source code, privacy, ecosystem)
  - Privacy Policy section in About settings linking to getdictus.com/en/privacy
  - Ecosystem section in About settings linking to getdictus.com
  - i18n keys settings.about.privacy.* and settings.about.ecosystem.* in all 20 locales
affects: [03-documentation-and-cleanup]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "i18n key propagation: English fallback strings used for V1 in new locales"
    - "SettingContainer with Button pattern for About panel external links"

key-files:
  created: []
  modified:
    - src/components/settings/about/AboutSettings.tsx
    - src/i18n/locales/en/translation.json
    - src/i18n/locales/fr/translation.json
    - src/i18n/locales/ar/translation.json
    - src/i18n/locales/bg/translation.json
    - src/i18n/locales/cs/translation.json
    - src/i18n/locales/de/translation.json
    - src/i18n/locales/es/translation.json
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
  - "English fallback strings used for 18 locales for V1 — i18next fallback prevents missing-key warnings; proper translations deferred"
  - "Privacy and ecosystem sections added after sourceCode/supportDevelopment, before AppDataDirectory, matching existing panel layout pattern"

patterns-established:
  - "About panel external links: SettingContainer with Button variant=secondary size=md onClick openUrl()"
  - "New i18n keys for About panel placed between supportDevelopment and acknowledgments across all locale files"

requirements-completed: [DOCS-03]

# Metrics
duration: 2min
completed: 2026-04-09
---

# Phase 3 Plan 02: About Panel Rebrand Summary

**About panel rebranded with Dictus URLs plus new Privacy Policy and Ecosystem sections, with i18n keys propagated across all 20 locales**

## Performance

- **Duration:** 2 min
- **Started:** 2026-04-09T11:08:05Z
- **Completed:** 2026-04-09T11:10:46Z
- **Tasks:** 2
- **Files modified:** 21

## Accomplishments
- Replaced `handy.computer/donate` with `getdictus.com/donate` and `github.com/cjpais/Handy` with `github.com/getdictus/dictus-desktop` in AboutSettings.tsx — zero Handy URLs remain
- Added Privacy Policy section (links to getdictus.com/en/privacy) and Ecosystem section (links to getdictus.com) to the About panel with proper i18n keys
- Propagated `settings.about.privacy.*` and `settings.about.ecosystem.*` keys to all 20 locale files (EN/FR with translations, 18 others with English fallback text)

## Task Commits

Each task was committed atomically:

1. **Task 1: Update AboutSettings.tsx URLs and add privacy/ecosystem sections** - `e661570` (feat)
2. **Task 2: Propagate new i18n keys to remaining 18 locales** - `9a72fdf` (feat)
3. **Prettier formatting fix (AboutSettings.tsx)** - `d2e22e1` (chore)

**Plan metadata:** (pending final commit)

## Files Created/Modified
- `src/components/settings/about/AboutSettings.tsx` - Rebranded URLs, added Privacy Policy and Ecosystem SettingContainer sections
- `src/i18n/locales/en/translation.json` - Added privacy and ecosystem keys (English)
- `src/i18n/locales/fr/translation.json` - Added privacy and ecosystem keys (French with proper accented chars)
- `src/i18n/locales/[ar,bg,cs,de,es,he,it,ja,ko,pl,pt,ru,sv,tr,uk,vi,zh,zh-TW]/translation.json` - Added privacy and ecosystem keys (English fallback for V1)

## Decisions Made
- English fallback strings used for 18 locales: i18next falls back to EN anyway; having explicit keys prevents missing-key console warnings at runtime without requiring 18 manual translations for V1
- Sections inserted after sourceCode/supportDevelopment, before AppDataDirectory — this matches the existing panel visual flow and keeps related identity/legal content grouped

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- Pre-existing lint error in `src/components/icons/DictusLogo.tsx` (literal "Dictus" in SVG `<text>` element, `i18next/no-literal-string` rule). This error existed before this plan's changes and is out of scope. Confirmed via git stash test. Documented in deferred items.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- About panel is fully Dictus-branded with zero Handy references
- Privacy and Ecosystem sections are live in the UI with i18n-backed strings across all 20 locales
- Pre-existing lint error in DictusLogo.tsx needs resolution in a separate task before CI can enforce lint-clean builds

---
*Phase: 03-documentation-and-cleanup*
*Completed: 2026-04-09*
