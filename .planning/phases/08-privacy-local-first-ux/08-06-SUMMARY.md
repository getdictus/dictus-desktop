---
phase: 08-privacy-local-first-ux
plan: "06"
subsystem: post-processing-settings
tags: [local-first, privacy, cloud-toggle, settings, i18n]
dependency_graph:
  requires: ["08-01", "08-02", "08-03", "08-04", "08-05"]
  provides:
    [
      "enable_cloud_providers setting",
      "cloud-gated ProviderPicker",
      "model-first PostProcessingSettings page",
    ]
  affects:
    [
      "src-tauri/src/settings.rs",
      "src/components/settings/post-processing/PostProcessingSettings.tsx",
      "src/components/settings/PostProcessingSettingsApi/ProviderPicker.tsx",
    ]
tech_stack:
  added: []
  patterns:
    [
      "OFF-by-default feature toggle",
      "i18next manual ternary pluralization",
      "non-destructive migration notice",
    ]
key_files:
  created: []
  modified:
    - src-tauri/src/settings.rs
    - src-tauri/src/shortcut/mod.rs
    - src-tauri/src/lib.rs
    - src/stores/settingsStore.ts
    - src/bindings.ts
    - src/components/settings/PostProcessingSettingsApi/ProviderPicker.tsx
    - src/components/settings/post-processing/PostProcessingSettings.tsx
    - src/i18n/locales/en/translation.json
decisions:
  - "Manually synced bindings.ts for changeEnableCloudProvidersSetting — specta auto-export only runs on debug builds (bun run tauri dev), not on cargo build alone"
  - "Used raw checkbox toggle in ProviderPicker instead of ToggleSwitch component — ToggleSwitch wraps SettingContainer which adds unwanted layout nesting in the inline cloud-toggle row"
  - "Used manual ternary for i18n pluralization (ready_one/ready_other) — no existing _one/_other suffix keys found in project, so i18next plural resolution was not verified as active"
  - "Library placeholder is intentional Coming-soon card, not scope creep — Local-First Models milestone explicitly committed to (user decision 2026-05-22); placeholder sets user expectation and frames current Ollama path as the bridge"
  - "Cloud-default migration is UI visibility filter only — does NOT mutate post_process_provider_id; non-destructive Alert shown when persisted provider is cloud and toggle is OFF"
metrics:
  duration: ~6 min
  completed: "2026-05-22"
  tasks: 3
  files: 8
---

# Phase 8 Plan 06: Cloud Providers OFF-by-default + Model-First Page Reframe Summary

Closed the UAT design gap on test 9: cloud providers are now hidden by default behind an OFF-by-default toggle; the post-processing page is reframed from "provider × model" to a model-first, privacy-first surface per `mockups/local-first-models-vision.png`.

## What Was Built

### Task 1 — Backend setting + Tauri command + Rust tests (commit `8e513ee`)

- Added `enable_cloud_providers: bool` field to `AppSettings` struct in `settings.rs` with `#[serde(default = "default_enable_cloud_providers")]`
- Added `default_enable_cloud_providers() -> bool { false }` function
- Wired field into `get_default_settings()` literal
- Added `change_enable_cloud_providers_setting` Tauri command in `shortcut/mod.rs` (mirrors `change_experimental_enabled_setting` pattern exactly)
- Registered command in `collect_commands!` in `lib.rs`
- Added 2 Rust unit tests: `default_enable_cloud_providers_is_false` and `default_settings_have_cloud_providers_disabled`
- All 7 `settings::tests` pass

### Task 2 — Frontend store wiring + ProviderPicker cloud-toggle gating (commit `4ee6e64`)

- Added `enable_cloud_providers` updater to `settingUpdaters` in `settingsStore.ts`
- Manually added `changeEnableCloudProvidersSetting` binding to `bindings.ts` (specta auto-export requires a full `bun run tauri dev` run; done manually for build-time correctness)
- Added `enable_cloud_providers?: boolean` to the `AppSettings` TypeScript type in `bindings.ts`
- Extended `ProviderPicker` with 3 new optional props: `enableCloudProviders`, `onToggleCloudProviders`, `isUpdatingCloudToggle`
- Cloud providers (External section) is hidden when `enableCloudProviders === false`
- Integrated inline cloud-toggle row always renders (so user can enable it even when section is hidden)
- Used a raw checkbox toggle (not `ToggleSwitch`) to avoid unwanted `SettingContainer` layout nesting
- Added 25 new English i18n keys:
  - `settings.postProcessing.cloudToggle.*` (2 keys)
  - `settings.postProcessing.modelsAndLocalProcessing.*` (23 keys)

### Task 3 — PostProcessingSettings page reframe (commit `800f2e5`)

- Added module-level constants: `LOCAL_PROVIDER_IDS_SET` and `RECOMMENDED_PROVIDER_ID`
- `PostProcessingSettingsApiComponent`: prepends a selected-model card with privacy tags (Local, Privé, Aucune donnée envoyée, Fonctionne hors ligne) and a "Recommandé" badge for Apple Intelligence; shows non-destructive `Alert variant="error"` when persisted provider is cloud but toggle is OFF; wires `enableCloudProviders` / `onToggleCloudProviders` props through to `ProviderPicker`
- `PostProcessingSettings`: full page reframe with:
  - Header: h1 "Modèles et traitement local" (top-left) + status badge "N modèle(s) prêt(s)" (top-right, data-driven count of local providers)
  - Subtitle with "En savoir plus" link opening `docs/PRIVACY.md` on GitHub
  - Hotkey group (unchanged)
  - API group (now includes selected-model card)
  - Library placeholder group (coming-soon card with "Bientôt" badge previewing Local-First Models milestone)
  - Prompts group (unchanged)
  - Three-pillar grid: Confidentialité par défaut / Contrôle utilisateur / Expérience simplifiée
- Build: `tsc && vite build` exits 0

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Manual bindings.ts sync required**

- **Found during:** Task 2
- **Issue:** `changeEnableCloudProvidersSetting` was not yet in `bindings.ts` — specta only auto-exports during `bun run tauri dev` (debug_assertions build), not during `cargo test` or `cargo build` alone
- **Fix:** Manually added the binding following the exact pattern of adjacent `changeExperimentalEnabledSetting` binding; also added `enable_cloud_providers?: boolean` to the TypeScript `AppSettings` type
- **Files modified:** `src/bindings.ts`
- **Commit:** `4ee6e64`

**2. [Rule 1 - Bug] TypeScript type error in updateSetting call**

- **Found during:** Task 3 — `bun run build` reported `Argument of type 'string' is not assignable to parameter of type 'boolean | undefined'`
- **Issue:** Initial code used `value as unknown as string` cast which violated the generic constraint
- **Fix:** Removed cast; used `void updateSetting("enable_cloud_providers", value)` directly since `enable_cloud_providers` is now properly typed as `boolean | undefined` in `AppSettings`
- **Files modified:** `src/components/settings/post-processing/PostProcessingSettings.tsx`
- **Commit:** `800f2e5`

**3. [Rule 2 - Design] Used raw checkbox toggle in ProviderPicker instead of ToggleSwitch component**

- **Found during:** Task 2 — read `ToggleSwitch.tsx` which wraps into `SettingContainer`
- **Issue:** `ToggleSwitch` adds `SettingContainer` layout (full-width row with padding/border) which would create nested container layout in ProviderPicker's inline cloud-toggle row
- **Fix:** Replicated the raw toggle markup from `ToggleSwitch` (same Tailwind classes, same `peer-checked` pattern) without the `SettingContainer` wrapper
- **Files modified:** `src/components/settings/PostProcessingSettingsApi/ProviderPicker.tsx`
- **Commit:** `4ee6e64`

## Plan 08-07 Dependency

This plan adds 25 new English i18n keys. Plan 08-07 will propagate these keys verbatim to all 19 sibling locales (es, fr, vi, and 16 others). Until 08-07 runs, the sibling locales will fall back to English for these keys.

## Self-Check: PASSED

All 6 modified files exist on disk. All 3 per-task commits confirmed in git log:

- `8e513ee` feat(08-06): backend setting
- `4ee6e64` feat(08-06): store wiring + ProviderPicker
- `800f2e5` feat(08-06): page reframe
