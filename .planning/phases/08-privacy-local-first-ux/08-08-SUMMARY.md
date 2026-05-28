---
phase: 08-privacy-local-first-ux
plan: "08"
subsystem: ui
tags:
  - post-processing
  - provider-picker
  - apple-intelligence
  - ollama
  - local-first
  - ux-bugfix
  - gap-closure

requires:
  - phase: 08-privacy-local-first-ux
    provides:
      - "renderRowExtras prop pattern on ProviderPicker (established in 08-03)"
      - "Apple Intelligence platform-gated default + appleIntelligenceUnavailable runtime probe (established in 08-03)"
      - "08-VERIFICATION.md UAT partial-pass entry with gaps 1a/2a/2b filed against PostProcessingSettings.tsx"
provides:
  - "Apple Intelligence unavailability Alert rendered inline within the apple_intelligence radio row"
  - "Ollama link visually identifiable at rest (underline + offset, no hover required)"
  - "API key field hidden for Custom (local) provider (Option A — no misleading sk-... placeholder for local-only path)"
  - "ProviderPicker.renderRowExtras now invoked for every row (caller controls null-vs-content)"
affects:
  - "08-09 (tabs restructure of post-processing page — must merge after 08-08)"
  - "08-10 (locale propagation sweep — no i18n change from 08-08, but tabs+pillar-removal copy lands in 08-10)"

tech-stack:
  added: []
  patterns:
    - "renderRowExtras callback invoked for every row in ProviderPicker; caller decides null-vs-content per row id"
    - "Local-first hide-pattern for inputs that don't apply to selected provider (vs greying out)"

key-files:
  created: []
  modified:
    - "src/components/settings/PostProcessingSettingsApi/ProviderPicker.tsx"
    - "src/components/settings/post-processing/PostProcessingSettings.tsx"

key-decisions:
  - "Gap 2b resolved via Option A (hide API key field for Custom (local) provider) — rejected Option B (optional placeholder) because it requires new i18n keys across 19 locales for marginal benefit and hiding aligns with the local-first 'trust users, no paternalistic UI' principle from 08-CONTEXT.md."
  - "renderRowExtras gate moved from `checked &&` to caller-controlled — single decision-maker is PostProcessingSettings.tsx; ProviderPicker stays dumb about row semantics."
  - "Apple Intelligence Alert rendered via renderRowExtras (not adjacent to the radio) so it sits within the row's pl-7 column, matching gap 1a visual spec."

patterns-established:
  - "Render-row-extras for every row: ProviderPicker invokes the callback for every radio option; the caller returns null when no extras apply, content otherwise. Eliminates need for per-row props in the picker for Apple Intelligence, Custom, or any future provider with row-scoped UI."
  - "Hide-vs-grey-out for irrelevant fields: when a provider doesn't need a field (e.g., Custom doesn't need an API key), the SettingContainer is omitted entirely. This keeps the form scannable and avoids the implication that the field is required."

requirements-completed: [PRIV-01]

duration: 2 min
completed: 2026-05-22
---

# Phase 8 Plan 8: Post-Processing UI Gap-Closure Summary

**Inline Apple Intelligence Alert inside the apple_intelligence radio row, underlined-at-rest Ollama link, and hidden API key field for Custom (local) — closing gaps 1a / 2a / 2b from the 08-05 UAT partial-pass.**

## Performance

- **Duration:** 2 min
- **Started:** 2026-05-22T20:04:49Z
- **Completed:** 2026-05-22T20:07:31Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- **Gap 1a closed:** Apple Intelligence "feature unavailable" Alert relocated from the page-bottom (where it visually orphaned itself far below the apple_intelligence radio it described) into the apple_intelligence row itself via `ProviderPicker.renderRowExtras`. The Alert now renders inline whenever `state.appleIntelligenceUnavailable` is true, regardless of whether the row is checked — so users on macOS Apple Silicon hardware that doesn't meet the Apple Intelligence OS gate see the explanation directly under the radio they're looking at.
- **Gap 2a closed:** Ollama anchor in the Custom (local) row hint text switched from `text-logo-primary hover:underline cursor-pointer` (invisible at rest — color shift swallowed by the small hint text) to `text-logo-primary underline underline-offset-2 hover:opacity-80 cursor-pointer`. The link is now identifiable as a link without hovering.
- **Gap 2b closed:** API Key SettingContainer gated on `state.selectedProvider?.id !== "custom"` — when Custom (local) is selected the misleading `sk-...` placeholder is hidden entirely. Local-only path no longer implies a required API key.
- **Foundation lift:** `ProviderPicker` now invokes `renderRowExtras` for every row (removed the `checked &&` gate). PostProcessingSettings.tsx is the single decision-maker for what renders per row — apple_intelligence shows the Alert when unavailable, custom shows the Ollama tip + Test connection button when selected, others return null.

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend ProviderPicker to invoke renderRowExtras for every row** — `47688cc` (fix)
2. **Task 2: Inline Apple Intelligence Alert via renderRowExtras + Ollama link rest-state underline + hide API key for Custom (local)** — `bbe82db` (fix)

**Plan metadata:** _(committed after this SUMMARY lands via gsd-tools commit)_

## Files Created/Modified

- `src/components/settings/PostProcessingSettingsApi/ProviderPicker.tsx` — removed the `checked &&` gate on `renderRowExtras` so the callback fires for every row (1 line changed). Preserves the existing `<div className="pl-7 mt-1">` alignment wrapper. No prop, type, or contract change.
- `src/components/settings/post-processing/PostProcessingSettings.tsx` — three discrete edits inside the existing `PostProcessingSettingsApiComponent`:
  1. **renderRowExtras callback** — added an `option.value === "apple_intelligence"` branch returning the unavailability Alert when `state.appleIntelligenceUnavailable` is true, before the existing `"custom"` branch.
  2. **Ollama anchor className** — `text-logo-primary hover:underline cursor-pointer` → `text-logo-primary underline underline-offset-2 hover:opacity-80 cursor-pointer`.
  3. **API key gating + Alert removal** — collapsed the old `{state.isAppleProvider ? (Alert or null) : (<>BaseUrl, ApiKey</>)}` ternary into `{!state.isAppleProvider && (<>{custom ? BaseUrl}{!custom ? ApiKey}</>)}`; the old page-bottom Alert is gone (it now lives inside renderRowExtras).
  4. **Pre-existing Prettier drift fixed** — lines 543-588 had pre-existing format drift untouched by these edits; Prettier-normalized as part of the same commit to satisfy the plan's `bun run format:check` gate (see Deviations).

## Decisions Made

- **Option A for gap 2b (hide field, not placeholder):** Chosen because (a) Option B would require new i18n keys propagated across 19 locales for marginal benefit, (b) hiding aligns with the local-first "trust users, avoid paternalistic UI" principle from 08-CONTEXT.md, and (c) the field is _functionally_ optional for Custom — the call site never sends `sk-...` for local providers anyway. No `en/translation.json` change needed.
- **renderRowExtras gate moved out of ProviderPicker:** The picker no longer makes assumptions about when extras apply. This is a one-line behavior change that makes the picker truly generic and pushes the row-semantics decision to the only place that has the context to make it (PostProcessingSettings.tsx).
- **Apple Intelligence Alert rendered via renderRowExtras (not adjacent):** The Alert sits inside the row's `pl-7 mt-1` column so it visually nests under the radio it describes, matching the gap 1a visual spec.
- **No regression to "En savoir plus" header link:** The page-header "Learn more" anchor also uses the old `hover:underline` style. Plan explicitly scopes only the Ollama link inside renderRowExtras; the header anchor is out of scope.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Pre-existing Prettier drift in PostProcessingSettings.tsx**

- **Found during:** Task 2 (after Edit 1+2+3, `bun run format:check` flagged lines 543-544 and 586-588 — neither of which I touched in this plan).
- **Issue:** `getSetting("post_process_providers") as PostProcessProvider[] | undefined` (line 543) and the `t("settings.postProcessing.modelsAndLocalProcessing.learnMore")` call (line 586) had pre-existing Prettier-violation formatting from a prior commit. The plan's verification gate requires `bun run format:check` to exit 0, but those drifts would have caused failure even though they're unrelated to gap-closure work.
- **Fix:** Ran `bunx prettier --write src/components/settings/post-processing/PostProcessingSettings.tsx` to normalize the file as a whole, which fixed those lines as a side effect of formatting the file. No semantic change.
- **Files modified:** `src/components/settings/post-processing/PostProcessingSettings.tsx` (5 lines reformatted, unrelated to gap-closure edits).
- **Verification:** `bunx prettier --check <file>` exits 0 post-fix.
- **Committed in:** `bbe82db` (Task 2 commit) — folded into the same commit as the gap-closure edits because it's the same file.

---

**Total deviations:** 1 auto-fixed (1 blocking — pre-existing format drift in the same file).
**Impact on plan:** Zero scope creep. The deviation is mechanical Prettier normalization on lines I didn't otherwise touch; no behavior change. Documented for traceability so a future reader doesn't wonder why Task 2's diff is slightly larger than the three gap-closure edits alone.

## Issues Encountered

- **Repo-wide `bun run lint` has a pre-existing failure** in `src/components/icons/DictusLogo.tsx` (i18next/no-literal-string on hardcoded "Dictus" SVG text — unrelated to this plan). Scoped lint (`bunx eslint <plan files>`) passes cleanly. Out of scope per deviation rules.
- **Repo-wide `bun run format:check` has pre-existing failures** in `.planning/` markdown files (untouched by this plan). Scoped Prettier check on plan-modified `.tsx` files passes cleanly.

These pre-existing repo-wide lint/format failures are tracked separately and will be addressed in dedicated cleanup work. No new issues introduced by this plan.

## Authentication Gates

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- **Ready for 08-09 (tabs restructure):** All three bugs are fixed in a way that survives the upcoming Local / Cloud tabs refactor in 08-09. The renderRowExtras pattern in particular is reusable in the tabbed layout; the Alert simply stays attached to the apple_intelligence row regardless of which tab contains it.
- **Merge order:** 08-08 must commit before 08-09 (both touch `PostProcessingSettings.tsx`). With 08-08 landed on `feat/phase-08-local-first-ux` at `bbe82db`, 08-09 can build on top with no conflict.
- **No i18n debt:** Option A for gap 2b avoided any new locale keys. 08-10's locale propagation sweep has nothing to carry from this plan.
- **No backend impact:** Settings store, Tauri commands, and providers list are untouched. No migration required.

---

## Self-Check: PASSED

**Files verified on disk:**

- FOUND: `src/components/settings/PostProcessingSettingsApi/ProviderPicker.tsx` (Task 1 edit)
- FOUND: `src/components/settings/post-processing/PostProcessingSettings.tsx` (Task 2 edit)

**Commits verified in git log:**

- FOUND: `47688cc` (Task 1 — `fix(08-08): make ProviderPicker invoke renderRowExtras for every row`)
- FOUND: `bbe82db` (Task 2 — `fix(08-08): inline Apple Intelligence Alert, fix Ollama link visibility, hide API key for Custom (local)`)

**Acceptance criteria spot-checks:**

- `grep -c "checked && renderRowExtras" src/components/settings/PostProcessingSettingsApi/ProviderPicker.tsx` → `0`
- `grep -cE "renderRowExtras \\? \\(" src/components/settings/PostProcessingSettingsApi/ProviderPicker.tsx` → `1`
- `grep -c "option.value === \"apple_intelligence\"" src/components/settings/post-processing/PostProcessingSettings.tsx` → `1`
- `grep -c "text-logo-primary underline underline-offset-2" src/components/settings/post-processing/PostProcessingSettings.tsx` → `1`
- `grep -c "state.selectedProvider?.id !== \"custom\"" src/components/settings/post-processing/PostProcessingSettings.tsx` → `1`
- `grep -c "{state.isAppleProvider ? (" src/components/settings/post-processing/PostProcessingSettings.tsx` → `0`
- `bun run build` → exits 0
- `bun run check:translations` → exits 0 (all 19 languages complete)
- `cargo fmt --check` → exits 0
- Scoped `bunx eslint` + `bunx prettier --check` on the two plan files → both exit 0

---

_Phase: 08-privacy-local-first-ux_
_Completed: 2026-05-22_
