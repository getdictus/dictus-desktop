---
created: 2026-04-24T00:05:00.000Z
title: Backfill missing i18n keys across 18 non-English locales
area: i18n
files:
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
  - scripts/check-translations.ts
---

## Problem

The `code-quality.yml` CI workflow runs `bun run check:translations` which fails because **5 English keys are missing from all 18 other locales** (AR, BG, CS, DE, ES, FR, HE, IT, JA, KO, PL, PT, RU, SV, TR, UK, VI, ZH, ZH-TW). This is 90 missing JSON entries total.

**Failing keys:**
- `settings.debug.simulateUpdaterRestart.title`
- `settings.debug.simulateUpdaterRestart.description`
- `settings.debug.simulateUpdaterRestart.button`
- `onboarding.permissions.accessibility.restart`
- `onboarding.permissions.accessibility.restartHint`

**Root causes (two distinct):**
1. The `simulateUpdaterRestart.*` trio was added by Phase 7 Plan 07-01 (commit `722a5b2`). The plan summary explicitly called out "English-only per debug-panel precedent (see CLAUDE.md i18n notes)" — but the CI check does NOT honor that convention.
2. The `accessibility.restart*` duo is pre-existing — added to English at some earlier commit but never backfilled. The v0.1.1 release CI was probably failing already on these two keys.

**Impact:**
- `code-quality.yml` CI is red on every push to main and every PR
- NOT blocking for `release.yml` (which doesn't call this workflow)
- `v0.1.0`, `v0.1.1`, and `v0.1.2` were all shipped with red translations CI

## Solution

Two possible approaches — pick one:

### Option 1 (quick, recommended for soft-launch period)
Add the missing keys with **English values as fallback** to all 18 locales. i18next falls back to English at runtime anyway, so the user-visible behavior is identical. This unblocks CI in ~5 minutes with a script:

```bash
# Pseudo:
# 1. Read the 5 missing keys + English values from en/translation.json
# 2. For each non-English locale, merge the keys at the correct JSON path
# 3. Commit as "chore(i18n): backfill missing keys with English fallback"
```

### Option 2 (proper, for real launch)
Actually translate the 5 keys into each language. Either manually (slow, requires native speakers or reliable MT) or via a translation service / model pass. This is the correct long-term fix.

**Recommendation:** Option 1 now to unblock CI, then schedule Option 2 as part of a dedicated i18n phase when a broader translation pass is planned.

### Also consider
Update `scripts/check-translations.ts` to optionally allowlist debug-panel keys (English-only by design per CLAUDE.md convention) so future Phase 7-style additions don't re-break the check.

## Priority / timing

- **Blocks:** clean green CI on main — cosmetic but erodes trust in the CI signal over time
- **Does NOT block:** v0.1.2 release or subsequent soft-launch releases
- **Good candidate for a small drive-by fix** (~30 min for Option 1) or a dedicated i18n phase

## Definition of done

- `bun run check:translations` exits 0 locally
- `code-quality.yml` turns green on main
- If Option 1 chosen: follow-up todo created for proper translations
- If Option 2 chosen: all 5 keys have natural-language translations in all 18 locales
- `scripts/check-translations.ts` behavior vis-à-vis debug-panel keys documented
