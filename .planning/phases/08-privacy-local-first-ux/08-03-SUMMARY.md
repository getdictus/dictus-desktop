---
phase: 08-privacy-local-first-ux
plan: "03"
subsystem: ui
tags: [react, i18n, tailwind, tauri, post-processing, privacy, local-first]

# Dependency graph
requires:
  - phase: 08-privacy-local-first-ux
    provides: i18n keys for all new UI copy (plan 03 adds English source; plan 04 propagates to other locales)
provides:
  - ProviderPicker component: stacked two-section radio picker (local above external)
  - TestConnectionButton component: test Ollama/custom endpoint with Alert feedback
  - groupedProviderOptions in usePostProcessProviderState hook
  - PostProcessingToggle promoted to its own SettingsGroup in Advanced
  - Network surface link in About panel
  - All new English i18n keys for Phase 8 UI
affects:
  - 08-04-PLAN (copies new i18n keys to 19 sibling locales)
  - 08-05-PLAN (validation plan will verify this UI surface)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - renderRowExtras prop pattern for injecting extras inside selected radio row
    - groupedProviderOptions memo splitting providers by LOCAL_PROVIDER_IDS Set

key-files:
  created:
    - src/components/settings/PostProcessingSettingsApi/ProviderPicker.tsx
    - src/components/settings/PostProcessingSettingsApi/TestConnectionButton.tsx
  modified:
    - src/components/settings/PostProcessingSettingsApi/usePostProcessProviderState.ts
    - src/components/settings/post-processing/PostProcessingSettings.tsx
    - src/components/settings/advanced/AdvancedSettings.tsx
    - src/components/settings/about/AboutSettings.tsx
    - src/i18n/locales/en/translation.json

key-decisions:
  - "ProviderSelect.tsx left in place as unused export — deletion deferred to avoid unrelated changes"
  - "PRIV-03 satisfied by existing state — Onboarding.tsx has zero post_process references; no code change made or required"
  - "Pre-existing lint error in DictusLogo.tsx (i18next/no-literal-string on SVG brand text) is out of scope; not caused by this plan"

patterns-established:
  - "renderRowExtras prop: parent injects row-level extras (tips, buttons) inside selected radio card"
  - "LOCAL_PROVIDER_IDS Set: frontend-only constant defining which provider IDs are on-device (apple_intelligence, custom)"

requirements-completed:
  - PRIV-01
  - PRIV-02
  - PRIV-03

# Metrics
duration: ~25min
completed: 2026-05-21
---

# Phase 8 Plan 3: Privacy / Local-First UX — Frontend Implementation Summary

**ProviderPicker (two-section radio picker), TestConnectionButton (Ollama test with Alert), Advanced toggle promotion, and About panel network surface link — full Phase 8 frontend surface**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-05-21T15:30:42Z
- **Completed:** 2026-05-21T15:56:10Z
- **Tasks:** 3
- **Files modified:** 7 (2 created, 5 modified)

## Accomplishments

- Built ProviderPicker (82 lines): stacked `<fieldset>`/`<legend>` radio sections; local providers above external; selected row gets `border-logo-primary bg-logo-primary/10` accent; `renderRowExtras` prop injects Ollama tip + TestConnectionButton inside the Custom (local) selected row
- Built TestConnectionButton (81 lines): calls `fetchPostProcessModels("custom")`, shows `Alert variant="success"` with model count or `Alert variant="error"` with friendly heading + collapsible `<details>` for raw error
- Extended `usePostProcessProviderState` with `groupedProviderOptions` memo (LOCAL_PROVIDER_IDS Set, apple_intelligence sorts first, preserves settings.rs order for external)
- Wired ProviderPicker into PostProcessingSettings replacing flat `<ProviderSelect>` dropdown; Ollama tip uses `<Trans>` with accessible link (role=link, tabIndex, onKeyDown)
- Promoted PostProcessingToggle from Experimental SettingsGroup to its own dedicated SettingsGroup in AdvancedSettings
- Added Network surface SettingContainer to About panel linking to `docs/PRIVACY.md` on GitHub
- Added all new English i18n keys to `src/i18n/locales/en/translation.json` (providers section, custom.ollamaTip, testConnection._, networkSurface._, advanced.groups.postProcessing)

## Task Commits

1. **Task 1: Extend hook with groupedProviderOptions + add new i18n keys** - `4f4b262` (feat)
2. **Task 2: ProviderPicker + TestConnectionButton components** - `356574a` (feat)
3. **Task 3: Wire ProviderPicker + promote toggle + About link** - `2fd0648` (feat)

## Files Created/Modified

- `src/components/settings/PostProcessingSettingsApi/ProviderPicker.tsx` (NEW, 82 lines) — Stacked two-section radio picker for post-process providers
- `src/components/settings/PostProcessingSettingsApi/TestConnectionButton.tsx` (NEW, 81 lines) — Test connection button + Alert feedback for Custom (local) provider
- `src/components/settings/PostProcessingSettingsApi/usePostProcessProviderState.ts` — Extended with `GroupedProviderOption` interface, `LOCAL_PROVIDER_IDS` constant, `groupedProviderOptions` memo, `useTranslation` import
- `src/components/settings/post-processing/PostProcessingSettings.tsx` — Replaced `<ProviderSelect>` with `<ProviderPicker>`; added Ollama tip `<Trans>` + `<TestConnectionButton>` via `renderRowExtras`
- `src/components/settings/advanced/AdvancedSettings.tsx` — Moved `<PostProcessingToggle>` to new dedicated SettingsGroup (`settings.advanced.groups.postProcessing`); removed from Experimental gate
- `src/components/settings/about/AboutSettings.tsx` — Added Network surface SettingContainer after marketing privacy row
- `src/i18n/locales/en/translation.json` — Added: `settings.postProcessing.api.providers.*`, `settings.postProcessing.api.custom.*`, `settings.about.networkSurface.*`, `settings.advanced.groups.postProcessing`

## New i18n Key Paths Added to en/translation.json

- `settings.postProcessing.api.providers.sectionLocal`
- `settings.postProcessing.api.providers.sectionExternal`
- `settings.postProcessing.api.providers.labels.custom`
- `settings.postProcessing.api.providers.descriptions.apple_intelligence`
- `settings.postProcessing.api.providers.descriptions.custom`
- `settings.postProcessing.api.custom.ollamaTip`
- `settings.postProcessing.api.custom.testConnection.button`
- `settings.postProcessing.api.custom.testConnection.success`
- `settings.postProcessing.api.custom.testConnection.errorTitle`
- `settings.postProcessing.api.custom.testConnection.errorDetails`
- `settings.about.networkSurface.title`
- `settings.about.networkSurface.description`
- `settings.about.networkSurface.button`
- `settings.advanced.groups.postProcessing`

## Decisions Made

- **ProviderSelect.tsx left in place:** The file has no callers after this plan but deleting it was deferred to keep the diff minimal and avoid any unintended breakage from removing an exported module. Deletion can be done in a follow-up cleanup.
- **PRIV-03 satisfied without code change:** `src/components/onboarding/Onboarding.tsx` contains zero `post_process` references. Onboarding already satisfies the local-first-primary requirement (model download first, no post-process surface). No code modification made or required.

## Deviations from Plan

None — plan executed exactly as written. The Prettier format run after Task 3 was expected (new files need formatting) and is part of normal workflow.

## Issues Encountered

- Pre-existing `i18next/no-literal-string` ESLint error in `src/components/icons/DictusLogo.tsx` (SVG brand text "Dictus") was present before this plan and is out of scope. All files introduced in this plan pass lint cleanly.

## PRIV-03 Status

**Satisfied by existing state.** `grep -rn 'post_process' src/components/onboarding/` returns 0 matches. The onboarding flow (model download → permissions → done) has no post-processing surface. This satisfies PRIV-03: first-run experience is local-first with no cloud/API prompting. Documented here; no code change made or required.

## Next Phase Readiness

- All English i18n keys ready for Plan 04 to copy across 19 sibling locales
- ProviderPicker, TestConnectionButton wired and functional
- Plan 05 (validation) can verify the full UI surface
- `ProviderSelect.tsx` remains in repo as unused export; consider deleting in a future cleanup plan

---

_Phase: 08-privacy-local-first-ux_
_Completed: 2026-05-21_
