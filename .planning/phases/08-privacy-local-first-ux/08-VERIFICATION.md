---
phase: 08-privacy-local-first-ux
verified: 2026-05-22T21:00:00Z
status: passed
score: 11/11 must-haves verified
re_verification:
  previous_status: gaps_found
  previous_score: 5/11 must-haves verified
  gaps_closed:
    - "Apple Intelligence unavailability banner appears inline with the Apple Intelligence provider card"
    - "The Ollama link in the Custom provider row is visually identifiable as a link without hovering"
    - "The API key field for the Custom (local) provider communicates it is optional"
    - "The provider area uses a tabs pattern (Local / Cloud) to separate local and cloud providers structurally"
    - "The three-pillar privacy marketing block (Confidentialite / Controle / Experience) is removed from the post-processing page"
    - "The local model library coming-soon placeholder block appears at the top of the post-processing page"
  gaps_remaining: []
  regressions: []
---

# Phase 8: Privacy / Local-First UX Verification Report (Re-Verification)

**Phase Goal:** The settings UI and onboarding flow communicate clearly that Dictus is a local-first app — local post-process providers appear before external ones, the network surface is documented, and onboarding copy presents cloud as opt-in.

**Verified:** 2026-05-22T21:00:00Z
**Status:** passed
**Re-verification:** Yes — second pass after gap-closure work in Plans 08-08, 08-09, 08-10

**Source of evidence:** Direct code inspection (post-gap-closure state) + automated quality gates (`bun run check:translations`, `bun run build`, `cargo test --lib settings::tests`).

---

## Re-Verification Summary

The 6 gaps surfaced by the live UAT on 2026-05-22 (Pierre Viviere, macOS Apple Silicon, dev mode) and recorded in the first verification pass have all been closed in source code. Closure traces:

| Gap | Truth                                                       | Closure path                                                                                | Commit(s)                                  |
| --- | ----------------------------------------------------------- | ------------------------------------------------------------------------------------------- | ------------------------------------------ |
| 1a  | Apple Intelligence Alert inline with apple_intelligence row | Plan 08-08 — `renderRowExtras` invoked for every row + Alert branched on apple_intelligence | `47688cc`, `bbe82db`                       |
| 2a  | Ollama link underlined at rest                              | Plan 08-08 — className `underline underline-offset-2`                                       | `bbe82db`                                  |
| 2b  | API key field hidden when Custom (local) is selected        | Plan 08-08 — `state.selectedProvider?.id !== "custom"` gate                                 | `bbe82db`                                  |
| 5   | Local / Cloud tabs replace cloud toggle                     | Plan 08-09 — `ProviderPicker` rewritten with tabs control + 08-10 locale propagation        | `dd45f4f`, `3433f9e`, `4750bd2`, `54b9a85` |
| 6   | Three-pillar marketing block removed                        | Plan 08-09 — JSX deleted + EN keys removed + Plan 08-10 propagated removal to 19 locales    | `3433f9e`, `4750bd2`, `54b9a85`            |
| 7   | Library coming-soon teaser hoisted to top                   | Plan 08-09 — JSX reordered to first child after page header                                 | `3433f9e`                                  |

All 6 gap-closure commits verified in `git log feat/phase-08-local-first-ux ^main`. No regressions detected in previously-verified truths 1-5.

---

## Goal Achievement

### Observable Truths

| #   | Truth                                                                                    | Status   | Evidence                                                                                                                                                                             |
| --- | ---------------------------------------------------------------------------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1   | Platform-aware default provider: Apple Intelligence on macOS ARM64, Custom elsewhere     | VERIFIED | `settings.rs:526-529`; test `default_post_process_provider_id_returns_apple_on_macos_arm64` passes                                                                                   |
| 2   | Custom provider relabeled to "Custom (local)" with stable `id="custom"`                  | VERIFIED | `settings.rs:608` `label: "Custom (local)"`; test `default_post_process_providers_includes_custom_with_stable_id` passes                                                             |
| 3   | `docs/PRIVACY.md` exists listing all outbound endpoints                                  | VERIFIED | File present, 75 lines; README `## Privacy` section at line 53                                                                                                                       |
| 4   | Onboarding presents local transcription as primary; no cloud post-processing surface     | VERIFIED | `grep -E "post_process\|cloud\|api_key" src/components/onboarding/*.tsx` returns 0 matches                                                                                           |
| 5   | ProviderPicker and TestConnectionButton components exist and are wired                   | VERIFIED | `ProviderPicker.tsx` (123 lines), imported at `PostProcessingSettings.tsx:17`; `TestConnectionButton` imported at line 18, used inside `renderRowExtras` callback line 168           |
| 6   | Apple Intelligence unavailability Alert renders inline within its provider card (Gap 1a) | VERIFIED | `PostProcessingSettings.tsx:128-137` — `option.value === "apple_intelligence"` branch inside `renderRowExtras` returns Alert when `state.appleIntelligenceUnavailable`               |
| 7   | Ollama link is visually identifiable as a link at rest (Gap 2a)                          | VERIFIED | `PostProcessingSettings.tsx:149` — className contains `underline underline-offset-2 hover:opacity-80`                                                                                |
| 8   | API key field hidden for Custom (local) provider (Gap 2b)                                | VERIFIED | `PostProcessingSettings.tsx:199` — `{state.selectedProvider?.id !== "custom" && (...)}` gating around `ApiKeyField` SettingContainer                                                 |
| 9   | Provider area uses Local / Cloud tabs (Gap 5)                                            | VERIFIED | `ProviderPicker.tsx:71-102` — `role="tablist"` with 2 `role="tab"` buttons; `aria-selected` per tab; section render gated on `activeTab === "local"` / `activeTab === "cloud"`       |
| 10  | Three-pillar marketing block removed from post-processing page (Gap 6)                   | VERIFIED | `grep -c "modelsAndLocalProcessing.pillars\|grid grid-cols-1 md:grid-cols-3" PostProcessingSettings.tsx` returns 0; EN i18n `pillars` key removed; all 19 sibling locales propagated |
| 11  | Library coming-soon placeholder is at top of page (Gap 7)                                | VERIFIED | `PostProcessingSettings.tsx:614-638` Library SettingsGroup appears at line 614 BEFORE Hotkey SettingsGroup at line 641 (file-order check)                                            |

**Score:** 11/11 truths verified (up from 5/11 in initial pass)

---

### Required Artifacts

| Artifact                                                                                          | Expected                                                             | Status   | Details                                                                                                                                                             |
| ------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `src-tauri/src/settings.rs`                                                                       | Platform-aware default + Custom (local) label + unit tests           | VERIFIED | Lines 526, 608, 995, 1012; 7/7 settings tests pass (including vestigial `default_enable_cloud_providers_is_false`)                                                  |
| `docs/PRIVACY.md`                                                                                 | Network surface documentation                                        | VERIFIED | 75 lines, exists; UPSTREAM hook present                                                                                                                             |
| `README.md`                                                                                       | `## Privacy` section with link                                       | VERIFIED | Section header at line 53                                                                                                                                           |
| `src/components/settings/PostProcessingSettingsApi/ProviderPicker.tsx`                            | Tabs control replacing toggle; renderRowExtras invoked for every row | VERIFIED | 123 lines; `activeTab` prop, `role="tablist"`, `role="tab"` (×2), `aria-selected` (×2); `renderRowExtras` invoked at line 59 unconditionally                        |
| `src/components/settings/PostProcessingSettingsApi/TestConnectionButton.tsx`                      | Success/error alert on connection test                               | VERIFIED | Component present and used inside Custom renderRowExtras branch                                                                                                     |
| `src/components/settings/post-processing/PostProcessingSettings.tsx`                              | Apple Intelligence Alert via renderRowExtras                         | VERIFIED | Lines 128-137 — branch returns Alert in `renderRowExtras` callback when `option.value === "apple_intelligence"` and `appleIntelligenceUnavailable`                  |
| `src/components/settings/post-processing/PostProcessingSettings.tsx`                              | Ollama link rest-state underline                                     | VERIFIED | Line 149: `text-logo-primary underline underline-offset-2 hover:opacity-80 cursor-pointer`                                                                          |
| `src/components/settings/post-processing/PostProcessingSettings.tsx`                              | API key field gated for Custom                                       | VERIFIED | Line 199: `{state.selectedProvider?.id !== "custom" && (...)}`                                                                                                      |
| `src/components/settings/post-processing/PostProcessingSettings.tsx`                              | Pillars block removed                                                | VERIFIED | 0 matches for `modelsAndLocalProcessing.pillars`, 0 matches for `grid grid-cols-1 md:grid-cols-3`                                                                   |
| `src/components/settings/post-processing/PostProcessingSettings.tsx`                              | Library at top of page                                               | VERIFIED | Library SettingsGroup at line 614; Hotkey SettingsGroup at line 641; correct order                                                                                  |
| `src/components/settings/post-processing/PostProcessingSettings.tsx`                              | Tab state managed via useState/useEffect                             | VERIFIED | Lines 39-51: `useState<"local" \| "cloud">(initialTab)` + `useEffect` syncing tab to `state.selectedProviderId`                                                     |
| `src/i18n/locales/en/translation.json`                                                            | tabs.local + tabs.cloud added; pillars + cloudToggle removed         | VERIFIED | `tabs.local = "Local"`, `tabs.cloud = "Cloud"`, `cloudToggle === undefined`, `pillars === undefined`, `cloudSelectedNotice` references `« Cloud »` tab              |
| `src/i18n/locales/{ar,bg,cs,de,es,fr,he,it,ja,ko,pl,pt,ru,sv,tr,uk,vi,zh,zh-TW}/translation.json` | Same EN structure with locale-translated values                      | VERIFIED | All 19 locales: tabs.local + tabs.cloud present, cloudToggle absent, pillars absent, cloudSelectedNotice contains the locale's `tabs.cloud` value (script-verified) |

---

### Key Link Verification

| From                                 | To                                       | Via                                                 | Status | Details                                                                     |
| ------------------------------------ | ---------------------------------------- | --------------------------------------------------- | ------ | --------------------------------------------------------------------------- |
| `default_post_process_provider_id()` | `APPLE_INTELLIGENCE_PROVIDER_ID`         | `cfg(all(target_os="macos",target_arch="aarch64"))` | WIRED  | settings.rs:526-529                                                         |
| `ProviderPicker`                     | `PostProcessingSettingsApi`              | import + JSX usage                                  | WIRED  | PostProcessingSettings.tsx:17, 120                                          |
| `TestConnectionButton`               | renderRowExtras callback (custom branch) | import + JSX usage                                  | WIRED  | PostProcessingSettings.tsx:18, 168                                          |
| Apple Intelligence Alert             | apple_intelligence row                   | `renderRowExtras` callback branch                   | WIRED  | PostProcessingSettings.tsx:128-137                                          |
| Ollama link visibility               | rest-state underline                     | className `underline underline-offset-2`            | WIRED  | PostProcessingSettings.tsx:149                                              |
| API key field gating                 | Custom provider                          | conditional render `id !== "custom"`                | WIRED  | PostProcessingSettings.tsx:199                                              |
| ProviderPicker tabs state            | activeTab/onTabChange                    | parent useState + props                             | WIRED  | PostProcessingSettings.tsx:44-50, 125-126; ProviderPicker.tsx:12-13, 79-100 |
| ProviderPicker section visibility    | active tab                               | conditional render on `activeTab === "local/cloud"` | WIRED  | ProviderPicker.tsx:105-117                                                  |
| Tab state sync                       | persisted provider                       | useEffect on selectedProviderId                     | WIRED  | PostProcessingSettings.tsx:47-51                                            |
| cloudSelectedNotice                  | activeTab === "local" && selectedIsCloud | derived from tab state                              | WIRED  | PostProcessingSettings.tsx:53                                               |
| Library placeholder                  | top of page                              | first SettingsGroup after header div                | WIRED  | PostProcessingSettings.tsx:614 (Library) before line 641 (Hotkey)           |
| Sibling locale parity                | `en/translation.json` source             | `bun run check:translations` script                 | WIRED  | All 19 languages pass; "All 19 languages have complete translations!"       |

---

### Requirements Coverage

| Requirement | Source plans                                           | Description                                                                                                                                               | Status    | Evidence                                                                                                                                                                                                                                                                       |
| ----------- | ------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| PRIV-01     | 08-01, 08-03, 08-04, 08-05, 08-06, 08-07, 08-08, 08-09 | Local providers visible primary (Apple Intelligence + Custom (local)); cloud providers gated behind opt-in (now tabs, not toggle); platform-aware default | SATISFIED | Backend default + relabel (settings.rs:526, 608); tabs structurally separate local/cloud (ProviderPicker.tsx); Apple Intelligence Alert inline (gap 1a closed); Ollama link visible (gap 2a closed); API key hidden for Custom (gap 2b closed); pillars removed (gap 6 closed) |
| PRIV-02     | 08-02, 08-03, 08-04, 08-05, 08-06, 08-07, 08-09        | Network surface documented in `docs/PRIVACY.md`; README linked; About panel link wired                                                                    | SATISFIED | PRIVACY.md (75 lines) + README §Privacy at line 53 + "En savoir plus" link in page header opens PRIVACY.md GitHub URL                                                                                                                                                          |
| PRIV-03     | 08-03, 08-05, 08-06, 08-07, 08-09                      | Onboarding presents local transcription as primary; no cloud post-process surface                                                                         | SATISFIED | `grep` confirms no `post_process` / `cloud` / `api_key` references in `src/components/onboarding/*.tsx`; documented in 08-03 SUMMARY                                                                                                                                           |

**Orphan check:** REQUIREMENTS.md maps PRIV-01, PRIV-02, PRIV-03 to Phase 8 only. All three IDs are claimed by at least one PLAN's `requirements` frontmatter. No orphaned requirements.

---

### Anti-Patterns Found

| File                                                                 | Line | Pattern                                                        | Severity | Impact                                                                                                                                                                                                                      |
| -------------------------------------------------------------------- | ---- | -------------------------------------------------------------- | -------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `src/components/settings/post-processing/PostProcessingSettings.tsx` | 584  | Header "En savoir plus" link still uses `hover:underline` only | Info     | Out of scope for Phase 8 gap-closure (gap 2a was scoped to the Ollama link inside renderRowExtras); the header link is visible enough in larger paragraph text and was explicitly excluded by Plan 08-08. Not a regression. |

No blocker or warning anti-patterns. No `TODO` / `FIXME` / `XXX` / `HACK` / `PLACEHOLDER` markers found in the two modified component files.

**Note on vestigial backend:** Plan 08-09 intentionally kept the `enable_cloud_providers` Rust field for persisted-settings backward compatibility (rationale documented in 08-09-SUMMARY.md). The frontend no longer reads or writes it; the field is a no-op. `cargo test --lib settings::tests` confirms the field still deserializes correctly (test `default_enable_cloud_providers_is_false` passes).

---

### Human Verification Required

None for the verification verdict. The 6 previously-failing truths are now code-confirmed and survived the automated quality gates (`bun run check:translations`, `bun run build`, `cargo test --lib settings::tests` all green).

**Optional re-UAT (recommended before merging to main):** A short visual confirmation in `bun run tauri dev` on macOS Apple Silicon, spot-checking 2-3 locales (French canonical + de + ja or ar), would close the loop on the partial-pass UAT outcome from 2026-05-22. Plan 08-10 SUMMARY ("Phase 8 Ship Readiness" section) recommends this exact check. Not required for the verification verdict — the code state matches what the gaps demanded.

---

### Gaps Summary

**0 gaps remaining.**

All 6 gaps from the initial verification pass are closed in source:

- Gap 1a — Apple Intelligence Alert relocated into `renderRowExtras` callback, branched on `option.value === "apple_intelligence"`. The Alert now sits inside the row's `pl-7 mt-1` column directly below the apple_intelligence radio. Commit `bbe82db`.
- Gap 2a — Ollama anchor className changed from `text-logo-primary hover:underline cursor-pointer` to `text-logo-primary underline underline-offset-2 hover:opacity-80 cursor-pointer`. Underline visible at rest. Commit `bbe82db`.
- Gap 2b — API key SettingContainer wrapped in `{state.selectedProvider?.id !== "custom" && (...)}`. Field hidden entirely for the Custom (local) path. Commit `bbe82db`.
- Gap 5 — Cloud opt-in toggle removed from `ProviderPicker.tsx`; replaced with a Local/Cloud segmented tabs control (role=tablist, role=tab, aria-selected). Tab state managed in parent, initialized from persisted provider, auto-synced via useEffect. Commits `dd45f4f`, `3433f9e`, `4750bd2`, `54b9a85`.
- Gap 6 — Three-pillar marketing grid JSX removed entirely; EN `pillars.*` keys deleted; 19 sibling locales propagated. Commits `3433f9e`, `4750bd2`, `54b9a85`.
- Gap 7 — Library SettingsGroup hoisted to first child after the page header div (line 614, before Hotkey at line 641). Commit `3433f9e`.

**Build / test posture:**

- `bun run check:translations` → "All 19 languages have complete translations!" (exit 0)
- `bun run build` → built in 1.95s (exit 0)
- `cargo test --lib settings::tests` → 7/7 passed (exit 0)

**Pre-existing failures (out of scope, documented in 08-08 / 08-09 / 08-10 SUMMARYs):**

- `bun run lint` repo-wide still fails on `src/components/icons/DictusLogo.tsx` (i18next/no-literal-string on hardcoded "Dictus" SVG `<text>`). Pre-existing; not introduced by this phase.
- `bun run format:check` has pre-existing failures on 16 `.planning/*.md` markdown files. Scoped Prettier on the Phase 8 modified `.tsx` and `.json` files exits 0.

Neither pre-existing failure affects the verification verdict — they pre-date Phase 8 and have no functional impact on the goal.

---

_Verified: 2026-05-22T21:00:00Z (re-verification pass)_
_Verifier: Claude (gsd-verifier)_
_Previous pass: 2026-05-22T00:00:00Z — gaps_found, 5/11_
_This pass: 11/11 must-haves verified, all 6 gaps closed in source, ready to ship pending optional re-UAT_
