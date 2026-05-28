---
phase: 08-privacy-local-first-ux
plan: "09"
subsystem: ui
tags:
  - post-processing
  - provider-picker
  - tabs
  - layout-restructure
  - i18n
  - gap-closure
  - local-first

requires:
  - phase: 08-privacy-local-first-ux
    provides:
      - "renderRowExtras pattern invoked for every row (established in 08-08 — survives this tabs restructure)"
      - "Apple Intelligence Alert inlined via renderRowExtras (established in 08-08)"
      - "Ollama link rest-state underline (established in 08-08)"
      - "API key field hidden for Custom (local) (established in 08-08)"
      - "Platform-aware default provider (apple_intelligence on macOS ARM64, custom elsewhere — established in 08-03)"
provides:
  - "Local/Cloud tabs control in ProviderPicker (replaces opt-in cloud toggle)"
  - "Tab state managed in PostProcessingSettingsApiComponent via useState, auto-synced to selectedProviderId"
  - "Library 'coming-soon' placeholder hoisted to first content block under page header"
  - "Three-pillar marketing grid removed entirely (no JSX, no English i18n keys)"
  - "i18n keys settings.postProcessing.tabs.local / tabs.cloud in en/translation.json"
  - "Vestigial enable_cloud_providers Rust field documented (kept for persisted-settings backward compatibility)"
affects:
  - "08-10 (locale propagation sweep — must add tabs.* and remove cloudToggle.* + pillars.* across 19 sibling locales)"
  - "08-VERIFICATION.md (gaps 5, 6, 7 closed pending UAT re-run after 08-10)"

tech-stack:
  added: []
  patterns:
    - "Controlled tabs component pattern: parent owns activeTab state, child receives activeTab + onTabChange props, renders one of two mutually-exclusive sections"
    - "useEffect-driven tab sync: when persisted provider id changes (e.g., user clicks a radio in a tab), the tab follows the selection automatically"

key-files:
  created: []
  modified:
    - "src/components/settings/PostProcessingSettingsApi/ProviderPicker.tsx"
    - "src/components/settings/post-processing/PostProcessingSettings.tsx"
    - "src/i18n/locales/en/translation.json"

key-decisions:
  - "enable_cloud_providers Rust field KEPT as vestigial (no migration risk; field no longer read or written by frontend UI; removing it would break deserialization for users who already persisted it in 08-06)."
  - "Tab default derived from persisted provider: if selectedProviderId is a cloud provider (not in LOCAL_PROVIDER_IDS_SET), default tab is 'cloud'; otherwise 'local'. Keeps users where they expect to be on page reload."
  - "useEffect auto-syncs tab to selection: clicking a radio in the Cloud tab keeps you on Cloud; programmatic switch to a local provider switches tab to Local. Prevents the cloudSelectedNotice from flashing on every selection."
  - "cloudSelectedNotice now fires only when activeTab === 'local' AND selectedIsCloud — i.e., 'you have a cloud provider selected but you're looking at the Local tab; switch tabs to see it.' Matches the new tabs mental model."
  - "Library SettingsGroup hoisted to top so the coming-soon teaser is the first thing users see (per Pierre's direct UAT direction in 08-05). Hotkey, API, and Prompts groups retain their relative order below."
  - "Three-pillar grid removed in full (Gap 6); en/translation.json `pillars.*` keys removed alongside the JSX. No replacement block — page is leaner."
  - "cloudToggle.* English i18n keys removed; replaced by tabs.local / tabs.cloud. Insertion position chosen to match cloudToggle's original location (between api and modelsAndLocalProcessing) so 08-10's locale sweep can mirror the structure without ordering drift."

patterns-established:
  - "Controlled-tabs-over-sequential-toggle: when a UI choice creates two mutually-exclusive content sections, prefer a tabs control over a toggle. Tabs make the distinction structural rather than sequential, and users don't have to discover the second section by flipping a switch."
  - "Vestigial-backend-when-removal-costs-migration: when a backend setting field is no longer used by the UI, keep the field if removing it would require a settings.json migration. Document as vestigial in SUMMARY. Defer cleanup to a dedicated migration plan if desired."
  - "Tab state initialized from persisted state, not hard-coded default: derive the initial active tab from whatever the user last selected; don't drag them back to a default tab on reload."

requirements-completed: [PRIV-01, PRIV-02, PRIV-03]

duration: 3 min 29 s
completed: 2026-05-22
---

# Phase 8 Plan 9: Post-Processing UI Tabs Restructure & Layout Cleanup Summary

**Replaced cloud opt-in toggle with Local/Cloud tabs in ProviderPicker, hoisted local-models library teaser to first-block-under-header position, removed three-pillar marketing grid entirely, and updated English i18n source (tabs._ added; cloudToggle._ + pillars.\* removed; cloudSelectedNotice copy aligned to the tabs mental model) — closing gaps 5/6/7 from the 08-05 UAT partial-pass.**

## Performance

- **Duration:** 3 min 29 s
- **Started:** 2026-05-22T20:11:49Z
- **Completed:** 2026-05-22T20:15:18Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments

- **Gap 5 closed:** `ProviderPicker.tsx` no longer has a cloud opt-in toggle. The component now renders a segmented Local/Cloud tabs control (role=tablist, role=tab, aria-selected on each tab) at the top, and renders ONLY the active section's provider rows beneath. The two cloud-toggle paths (toggle row markup + `enableCloudProviders &&` conditional) are gone. The component's prop surface is now strictly `localOptions`, `externalOptions`, `value`, `onChange`, `disabled?`, `renderRowExtras?`, `activeTab`, `onTabChange`.
- **Gap 6 closed:** `PostProcessingSettings.tsx` no longer renders the three-pillar marketing grid (Confidentialité par défaut / Contrôle utilisateur / Expérience simplifiée). The entire `<div className="grid grid-cols-1 md:grid-cols-3 gap-3 pt-2">…</div>` block is deleted. `en/translation.json` no longer carries the `modelsAndLocalProcessing.pillars` key block.
- **Gap 7 closed:** The Library SettingsGroup ("Bibliothèque de modèles locaux" — coming-soon teaser for the local-models milestone) is now the FIRST content block under the page header, above the Hotkey, API, and Prompts groups. File-order grep confirms `modelsAndLocalProcessing.library.title` appears before `postProcessing.hotkey.title`.
- **Tab state initialization & sync:** `PostProcessingSettingsApiComponent` derives its initial `activeTab` from the persisted `selectedProviderId` (cloud if the provider isn't in `LOCAL_PROVIDER_IDS_SET`, else local) and uses a `useEffect` to keep the tab in sync whenever the selection changes. This means clicking OpenAI from the Cloud tab keeps you on Cloud; programmatic switches to a local provider snap the tab to Local.
- **`cloudSelectedNotice` wiring updated:** The Alert that fires when a user has a cloud provider selected but is currently looking at the Local tab now derives from `activeTab === "local"` (formerly `!enableCloudProviders`) and its copy refers to "the Cloud tab below" instead of "enable cloud models below."
- **Vestigial backend documented:** The `enable_cloud_providers` field in `src-tauri/src/settings.rs` is intentionally kept (no schema migration), but the frontend UI no longer reads or writes it. The setting becomes a no-op for all UI flows; if a user has it persisted from 08-06, deserialization still works.

## Task Commits

Each task was committed atomically:

1. **Task 1: Restructure ProviderPicker — Local/Cloud tabs replace cloud toggle** — `dd45f4f` (feat)
2. **Task 2: Hoist library to top, remove pillars, wire tab state in PostProcessingSettings** — `3433f9e` (refactor)
3. **Task 3: Update en/translation.json — add tabs._, drop pillars._ + cloudToggle.\*, update cloudSelectedNotice** — `4750bd2` (chore)

**Plan metadata:** _(committed after this SUMMARY lands via gsd-tools commit)_

## Files Created/Modified

- `src/components/settings/PostProcessingSettingsApi/ProviderPicker.tsx` — rewrite (41 insertions / 33 deletions; net +8 lines). Prop interface: dropped `enableCloudProviders` / `onToggleCloudProviders` / `isUpdatingCloudToggle`; added `activeTab: "local" | "cloud"` and `onTabChange: (tab) => void`. Render body: removed the toggle row markup (the 22-line `<div className="flex items-start justify-between gap-3 p-3 rounded-md border border-mid-gray/20 bg-mid-gray/5">…</div>` block) and replaced the section-render conditional with `activeTab === "local"` / `activeTab === "cloud"` branches. Added a segmented tablist (inline-flex container, two buttons with aria-selected, neutral hover/active styling per UI-SPEC §Color so it doesn't compete with the selected radio's logo-primary accent). `renderSection` helper preserved verbatim. `ProviderPicker.displayName` preserved.

- `src/components/settings/post-processing/PostProcessingSettings.tsx` — four discrete edits in `PostProcessingSettingsApiComponent` and the outer `PostProcessingSettings`:
  1. **Removed cloud-toggle wiring**: dropped the `enableCloudProviders` / `onToggleCloudProviders` / `isUpdatingCloudToggle` derivations and the `getSetting("enable_cloud_providers")` read. Removed `updateSetting, isUpdating` from the `useSettings()` destructure in this component (they were only used by the toggle).
  2. **Added tab state**: `useState<"local" | "cloud">(initialTab)` where `initialTab` is computed from `selectedProviderId` membership in `LOCAL_PROVIDER_IDS_SET`. Added `useEffect` to keep `activeTab` synced with `state.selectedProviderId`.
  3. **Updated `showCloudSelectedNotice`**: now `selectedIsCloud && activeTab === "local"` (formerly `selectedIsCloud && !enableCloudProviders`).
  4. **Updated `<ProviderPicker />` JSX**: dropped the three toggle props, added `activeTab={activeTab}` and `onTabChange={setActiveTab}`. The `renderRowExtras` body is preserved verbatim from 08-08 (Apple Intelligence Alert branch + Ollama tip + TestConnectionButton for Custom).
  5. **Hoisted Library SettingsGroup** to be the first child after the header `<div>` (Gap 7). Order is now: Header → Library → Hotkey → API → Prompts. Deleted the entire three-pillar grid block (Gap 6) — no comment placeholder.

- `src/i18n/locales/en/translation.json` — four discrete changes (4 insertions / 18 deletions; net −14 lines):
  1. **Added** `settings.postProcessing.tabs.local: "Local"` and `tabs.cloud: "Cloud"`, inserted at the position formerly occupied by `cloudToggle` (between `api` and `modelsAndLocalProcessing`) so 08-10's locale sweep preserves insertion order.
  2. **Removed** `settings.postProcessing.cloudToggle` (entire object — `.label` + `.description`).
  3. **Removed** `settings.postProcessing.modelsAndLocalProcessing.pillars` (entire object — `.privacy` / `.control` / `.simplicity`, each with `.title` + `.body`). `library` is now the last key under `modelsAndLocalProcessing`; trailing comma fixed.
  4. **Updated** `settings.postProcessing.modelsAndLocalProcessing.selectedModel.cloudSelectedNotice`: copy now references "Cliquez sur l'onglet « Cloud » ci-dessous" instead of "Activez les modèles cloud ci-dessous". French canonical voice preserved per the 08-06 source-locale decision.

## Decisions Made

- **Tabs over toggle for provider section choice (Gap 5):** Pierre's direct UAT direction (2026-05-22). Tabs make the local/cloud distinction structural rather than sequential — users see both options at all times and don't have to discover the cloud section by flipping a switch they may not want to flip. The semantic accessibility is also better (role=tablist with aria-selected) than a toggle that gates visibility.
- **Library hoisted to top (Gap 7):** Pierre's direct UAT direction. The coming-soon teaser for local-models v1.3 should be the first thing users see on the page so they understand where Dictus is heading. Below-the-fold placement (where it sat after 08-03) made it feel like an afterthought.
- **Three-pillar grid removed (Gap 6):** Pierre's direct UAT direction. The grid was a marketing-style block reasserting privacy/control/simplicity values that the page header already conveys via the "En savoir plus" link to PRIVACY.md. Removing it cleans up visual noise without losing information.
- **Default tab derived from persisted provider, not hard-coded:** Defaulting to "local" always would force users on Cloud providers to switch tabs every time they reopen settings. Deriving from persisted state respects their last choice.
- **useEffect syncs tab to selection:** Otherwise selecting a cloud provider from the Cloud tab would leave the tab on Cloud (correct), but selecting a local provider via some other UI path (e.g., the Onboarding default) would leave the tab on Cloud while the local provider is now selected — confusing. The effect keeps tab and selection coherent.
- **`enable_cloud_providers` Rust field kept vestigial:** Removing it would require a settings.json migration for users who persisted the field in 08-06. The cost-benefit doesn't justify a schema change for a UI cleanup. The field is now a no-op; if a future cleanup is desired, file a dedicated migration plan. `settings::tests` (7 tests including `default_settings_have_cloud_providers_disabled`) still pass with the field intact.
- **Tabs styling neutral, not accent:** Tabs use `bg-background text-text shadow-sm` for the active tab and `text-mid-gray` for inactive — NOT `logo-primary`. Per UI-SPEC §Color (60/30/10), the accent color is reserved for the selected provider radio inside the tab content; tabs at a higher visual layer should be neutral so they don't compete.
- **`cloudSelectedNotice` semantic kept, copy updated:** The notice's role is unchanged — it tells the user "you have a cloud provider persisted but the section you're looking at doesn't show it." Only the suggested action changes from "enable the toggle" to "switch tabs."

## Deviations from Plan

### Auto-fixed Issues

None during execution. The plan was followed exactly as written.

### Out-of-Scope Discoveries (Logged, Not Fixed)

The plan's verification block calls for repo-wide `bun run lint` and `bun run format:check` to exit 0. Both have pre-existing failures that 08-08-SUMMARY.md already documented:

- **Repo-wide `bun run lint`** fails on `src/components/icons/DictusLogo.tsx` (i18next/no-literal-string on the hardcoded "Dictus" SVG `<text>` content). Unrelated to this plan; pre-existing since 08-08. Scoped `bunx eslint` on the three plan-modified files exits 0.
- **Repo-wide `bun run format:check`** fails on 16 pre-existing `.planning/*.md` markdown drift files (none of which are this plan's three modified files). Scoped `bunx prettier --check` on the three plan-modified files exits 0.
- **`bun run check:translations`** fails after Task 3 with exactly the predicted pattern: 19 sibling locales each have 2 missing keys (`tabs.local`, `tabs.cloud`) and 8 extra keys (`cloudToggle.{label,description}` + `pillars.{privacy,control,simplicity}.{title,body}`). This failure is documented in the plan and resolved by Plan 08-10 (locale propagation).

These pre-existing failures are tracked separately in deferred work and were not introduced by this plan.

---

**Total deviations:** 0 auto-fixed inside scope. 3 pre-existing repo-wide failures noted as out-of-scope (documented above).
**Impact on plan:** Zero scope creep. The plan landed exactly as written; check:translations failure is by design and explicitly called out in the plan's acceptance criteria.

## Issues Encountered

- **`check:translations` red after Task 3 (expected):** 19 locales × 10 affected keys = 190 issues. Plan 08-10 will close this by mirroring the English JSON's key set across `es`, `fr`, `it`, `de`, `pt`, `pt-BR`, `nl`, `ja`, `ko`, `pl`, `ru`, `vi`, `tr`, `id`, `ar`, `hi`, `hu`, `zh`, `zh-TW`.
- **No regression on Rust settings:** `cargo test --lib settings::tests` runs all 7 tests successfully, including `default_settings_have_cloud_providers_disabled` and `default_enable_cloud_providers_is_false`. The vestigial field still deserializes correctly.

## Authentication Gates

None.

## User Setup Required

None — no external service configuration.

## Next Phase Readiness

- **Ready for 08-10 (locale propagation):** The English source-locale changes that need to propagate are tightly scoped:
  - ADD `settings.postProcessing.tabs.local: "Local"` and `tabs.cloud: "Cloud"` to all 19 sibling locales (translate `"Local"` and `"Cloud"` per target language — typically the same word for most, but some Asian locales may transliterate).
  - REMOVE `settings.postProcessing.cloudToggle` (entire object) from all 19 sibling locales.
  - REMOVE `settings.postProcessing.modelsAndLocalProcessing.pillars` (entire object) from all 19 sibling locales.
  - UPDATE `settings.postProcessing.modelsAndLocalProcessing.selectedModel.cloudSelectedNotice` to align with the new tabs mental model (target language).
- **Merge order:** 08-09 commits land on `feat/phase-08-local-first-ux` at `4750bd2`. 08-10 builds on top with no conflict.
- **UAT re-run pending:** After 08-10 lands, the full 08-05 UAT script should be re-run to confirm gaps 5/6/7 are visually closed across at least one locale (likely French, per the canonical-French source decision from 08-06). Gaps 1a/2a/2b were closed by 08-08 and verified by the same UAT framework.
- **No backend impact:** Settings store, Tauri commands, providers list all untouched. Vestigial `enable_cloud_providers` field documented but kept.

---

## Self-Check: PASSED

**Files verified on disk:**

- FOUND: `src/components/settings/PostProcessingSettingsApi/ProviderPicker.tsx` (Task 1 edit)
- FOUND: `src/components/settings/post-processing/PostProcessingSettings.tsx` (Task 2 edit)
- FOUND: `src/i18n/locales/en/translation.json` (Task 3 edit)

**Commits verified in git log:**

- FOUND: `dd45f4f` (Task 1 — `feat(08-09): replace cloud toggle with Local/Cloud tabs in ProviderPicker`)
- FOUND: `3433f9e` (Task 2 — `refactor(08-09): hoist library to top, remove pillars, wire tab state in PostProcessingSettings`)
- FOUND: `4750bd2` (Task 3 — `chore(08-09): update en/translation.json — add tabs.*, drop pillars.* + cloudToggle.*, update cloudSelectedNotice`)

**Acceptance criteria spot-checks:**

Task 1 (ProviderPicker.tsx):

- `grep -c "enableCloudProviders\|onToggleCloudProviders\|isUpdatingCloudToggle"` → `0`
- `grep -c "activeTab"` → `8` (≥4 required)
- `grep -c "settings.postProcessing.tabs.local"` → `1`
- `grep -c "settings.postProcessing.tabs.cloud"` → `1`
- `grep -c "settings.postProcessing.cloudToggle"` → `0`
- `grep -c 'role="tablist"'` → `1`
- `grep -c 'role="tab"'` → `2`
- `grep -c "aria-selected"` → `2`
- `grep -c "renderSection"` → `3` (helper preserved)
- `grep -c "ProviderPicker.displayName"` → `1`
- Scoped `bunx eslint` + `bunx prettier --check` → both exit 0

Task 2 (PostProcessingSettings.tsx):

- `grep -c "enable_cloud_providers"` → `0`
- `grep -c "enableCloudProviders\|onToggleCloudProviders\|isUpdatingCloudToggle"` → `0`
- `grep -c 'useState<"local" | "cloud">'` → `1`
- `grep -c "activeTab"` → `3`
- `grep -c "setActiveTab"` → `3`
- `grep -c "modelsAndLocalProcessing.pillars"` → `0`
- `grep -c "grid grid-cols-1 md:grid-cols-3 gap-3"` → `0`
- `grep -c '"privacy", "control", "simplicity"'` → `0`
- Library-before-Hotkey order check → `LIBRARY_FIRST`
- `bun run build` → exits 0
- Scoped `bunx eslint` + `bunx prettier --check` → both exit 0

Task 3 (en/translation.json):

- `JSON.parse(...)` → `JSON_OK`
- `t.settings.postProcessing.cloudToggle === undefined` → `true`
- `t.settings.postProcessing.modelsAndLocalProcessing.pillars === undefined` → `true`
- `t.settings.postProcessing.tabs.local === 'Local' && tabs.cloud === 'Cloud'` → `true`
- `cloudSelectedNotice` includes `'onglet'` and `'Cloud'` → `true`
- `grep -c "pillars"` → `0`
- `grep -c "cloudToggle"` → `0`
- `bun run build` → exits 0
- `bun run check:translations` → exits 1 (BY DESIGN — sibling locale parity restored in 08-10)
- Scoped `bunx prettier --check` → exits 0

Rust (vestigial-field validation):

- `cargo fmt -- --check` → exits 0
- `cargo test --lib settings::tests` → 7/7 pass (including `default_enable_cloud_providers_is_false` and `default_settings_have_cloud_providers_disabled`)

---

_Phase: 08-privacy-local-first-ux_
_Completed: 2026-05-22_
