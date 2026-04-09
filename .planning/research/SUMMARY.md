# Project Research Summary

**Project:** Dictus Desktop V1 — Handy Fork Rebrand
**Domain:** Tauri 2.x desktop app rebranding (identity, assets, bundle config, design tokens)
**Researched:** 2026-04-05
**Confidence:** HIGH

## Executive Summary

Dictus Desktop V1 is a controlled rebranding milestone, not a greenfield product. The underlying Tauri 2.x + Rust + React + TypeScript + Tailwind v4 stack is fixed and sound. The work is scoped to four layers: bundle identity configuration, icon and asset replacement, design token injection, and i18n string updates across all locales. Because the codebase is a fork of an actively maintained upstream project (Handy by cjpais), the rebrand carries inherited dependencies — a patched Tauri runtime, an external `handy-keys` crate, all model download URLs on a CDN controlled by the upstream maintainer, and an auto-updater pointing to the upstream repository. These are not blockers for V1 but represent known technical debt that must be explicitly documented and tracked.

The recommended approach follows a three-pass pattern: visible identity first (user-facing text, icons, config fields), bundle identity second (Cargo.toml binary naming, package.json), and internal symbol renaming third (Rust struct names, persisted enum keys, IPC event names). This ordering allows the visible rebrand to ship quickly with low risk, while deferring the changes that carry the highest risk of silent breakage — specifically the `KeyboardImplementation::HandyKeys` enum key serialized in user settings, and the portable mode marker string. The most critical decision gate before any code ships is whether to implement a data migration for the app identifier change (`com.pais.handy` → `com.dictus.desktop`). Since Dictus Desktop has no existing user base, the recommended choice is to accept a clean-slate install and change the identifier immediately in Phase 1.

The top risks are: (1) the auto-updater must be pointed away from the upstream `cjpais/Handy` releases endpoint before any build is distributed — leaving it active would deliver upstream Handy updates to Dictus users; (2) the Windows code signing identity in `tauri.conf.json` references the upstream maintainer's Azure signing account and must be cleared or replaced before any Windows CI release build; and (3) all 20+ locale files (not just English) contain "Handy" in translation values and must all be updated as part of the visible rebrand phase. These three pitfalls are the most likely to be missed and the most damaging if they ship.

## Key Findings

### Recommended Stack

The stack is not a decision — it is inherited from the Handy codebase. The research task was to identify the correct tooling for each layer of rebrand work. All findings are HIGH confidence, based on official documentation and direct codebase inspection.

**Core technologies:**

- `tauri.conf.json` `productName` + `identifier` fields: single source of truth for app identity; changing `identifier` cascades to all OS-level paths automatically — change once, never revisit
- `bun run tauri icon` (Tauri CLI 2.10.0): generates all required icon sizes from a single 1024x1024 PNG source — the only correct approach, no manual resizing
- Tailwind v4 `@theme` directive (already installed at 4.1.16): CSS-first design token system; defines Dictus brand colors as CSS custom properties that auto-generate both utility classes and CSS variables simultaneously
- `i18next-parser` (add as dev dep): scans all source files to flag missing/orphaned translation keys across all 21 locale files — essential for catching non-English locale gaps that visual testing misses

**Version constraints:**

- `@tauri-apps/cli 2.10.0` must match `tauri 2.10.2` crate version — always run matching versions
- `tailwindcss 4.1.16` and `@tailwindcss/vite 4.1.16` must remain pinned together
- The JS `tailwind.config.js` pattern is deprecated in v4; use only the CSS `@theme` directive

### Expected Features

The feature research defines a clear MVP boundary. Nearly all user-facing STT features (hotkey, overlay, paste, language selection, history, model management) already exist and are not in scope. V1 is exclusively a rebrand milestone.

**Must have (table stakes for a legitimate rebrand):**

- Product name "Dictus Desktop" everywhere visible (window title, tray menu, about panel, onboarding)
- Bundle identifier `com.dictus.desktop` in `tauri.conf.json` and `Cargo.toml`
- Dictus app icon across all required sizes (macOS `.icns`, Windows `.ico`, Linux `.png`)
- All "Handy" user-visible text replaced in UI, onboarding, and settings
- i18n strings updated across all 4 primary locales (en/es/fr/vi); all 20+ locale files audited
- Settings sections renamed: "General" → "Dictation", "Postprocessing" → "Smart Modes"
- README rewritten as Dictus Desktop (fork attribution, privacy-first positioning)
- Critical internal renames that surface in logs or filesystem (binary name, log file name)

**Should have (deliver before first external user):**

- Internal Rust symbol renames (handy_app_lib → dictus_app_lib, function names)
- Full Dictus iOS palette alignment via Tailwind `@theme` tokens
- About panel with privacy-first and open-source copy
- Build and contributor documentation updated

**Defer to V2+:**

- Mobile ↔ desktop sync (no architecture defined)
- Windows/Linux code signing pipeline
- Model recommendation tiers (requires benchmark data)
- Additional LLM providers
- Real-time streaming transcription (requires pipeline rewrite)
- User accounts or auth (contradicts privacy-first positioning)

### Architecture Approach

The existing architecture requires no structural reorganization. The rebrand is a naming exercise across a well-understood three-layer stack: configuration files (tauri.conf.json, Cargo.toml, package.json), React/TypeScript frontend components and i18n JSON, and Rust backend source files. The dependency graph between these layers determines the safe execution order; the critical constraint is that the Cargo.toml library name change (`handy_app_lib` → `dictus_app_lib`) must be committed in the same change as the `main.rs` import update, or the build breaks.

**Major components and rebrand scope:**

1. **Bundle identity layer** (`tauri.conf.json`, `Cargo.toml`, `package.json`) — config file changes; highest downstream impact (OS paths, binary name, update mechanism)
2. **Frontend visual identity** (`src/components/icons/HandyHand.tsx`, `HandyTextLogo.tsx`, `src/i18n/locales/`) — component rename + SVG replacement + 20+ JSON files; user-visible; low compile risk
3. **Rust runtime identity** (`lib.rs`, `tray.rs`, `cli.rs`, `llm_client.rs`, `shortcut/handy_keys.rs`, `settings.rs`) — requires care around persisted serialized keys and IPC event names; highest internal risk
4. **Asset pipeline** (`src-tauri/icons/`, `src-tauri/resources/`) — binary asset replacement; no code changes required; driven by `tauri icon` CLI
5. **External dependencies** (`handy-keys` crate, `cjpais/tauri.git` patch, `blob.handy.computer` CDN, `cjpais/Handy` updater endpoint) — inherited upstream infrastructure; V1 documents the risk, V2+ resolves it

### Critical Pitfalls

1. **Updater endpoint pointing to upstream Handy releases** — must be disabled or redirected before any distributed build; leaving it active delivers upstream Handy updates to Dictus users, or leaks the upstream signing key as a trusted source
2. **Windows code signing identity referencing `cjpais-dev` Azure account** — `signCommand` in `tauri.conf.json` will either fail in CI or (if credentials are present via secrets inheritance) sign Dictus releases under the wrong identity; clear or replace before any Windows CI release
3. **i18n "Handy" across 20+ non-English locale files** — `grep -rl "Handy" src/i18n/locales/` finds 20+ affected files beyond `en/translation.json`; visual testing in English misses this entirely
4. **`KeyboardImplementation::HandyKeys` serialized as `"handy_keys"` in user settings** — renaming the enum variant without a `#[serde(rename = "handy_keys")]` attribute silently resets any user whose settings file contains this key; use serde rename to keep wire format stable
5. **Portable mode marker string `"Handy Portable Mode"`** — changing this string without backward-compat check breaks existing portable installs; implement dual-string acceptance (`"Handy Portable Mode"` and `"Dictus Portable Mode"`) before shipping the internal rename

## Implications for Roadmap

Based on combined research, the rebrand naturally separates into three phases matching risk profile and dependency order.

### Phase 1: Bundle Identity and Updater Safety

**Rationale:** The updater endpoint and Windows signing identity are security/trust boundaries that must be resolved before anything else is distributed. The bundle identifier (`com.dictus.desktop`) must be set once and never revisited — doing it first means no data migration is needed since no Dictus users exist yet. This phase has the highest downstream impact with the lowest implementation complexity.

**Delivers:** A correctly-identified Tauri application that cannot accidentally push upstream Handy updates to users; a build that won't fail or misbehave in CI due to inherited signing identity.

**Addresses:** Product name, bundle identifier, updater endpoint, Windows signing command, Cargo.toml description

**Avoids:** Pitfalls 1 (updater), 2 (bundle ID post-distribution), 7 (Windows signing identity)

**Research flag:** Standard patterns — well-documented Tauri config fields, no deeper research needed.

---

### Phase 2: Visual Rebrand (UI, Icons, i18n)

**Rationale:** The visible rebrand is what users and stakeholders see. It has no compile-time risk (no Rust changes) and can be executed in parallel across components, icons, and translation files. The icon pipeline (`tauri icon` CLI) is the single authoritative path — no manual resizing. The i18n update must cover all 20+ locale files, not just English; this is the most commonly missed step.

**Delivers:** An app that looks and reads as Dictus Desktop in all supported languages, with Dictus icons across all platforms and correct brand color tokens in Tailwind.

**Addresses:** App icon replacement, all i18n locale files, HandyHand/HandyTextLogo component rename, Sidebar/Onboarding/About/UpdateChecker copy, Tailwind `@theme` brand tokens, settings section renames (Dictation, Smart Modes), README rewrite

**Avoids:** Pitfall 4 (i18n only partially updated), Pitfall 5 (icon sizes incomplete)

**Stack note:** Run `bun run tauri icon` from a 1024x1024 source PNG after Dictus brand assets are provided; add `i18next-parser` as dev dep to audit translation coverage.

**Research flag:** Standard patterns — well-established Tauri and i18next tooling. No deeper research needed.

---

### Phase 3: Internal Symbol Rename and Technical Cleanup

**Rationale:** Internal renames (Rust struct names, Cargo binary name, IPC event names, persisted enum keys) have the highest risk of silent breakage and the lowest user-visible impact. Deferring them to Phase 3 means the visible product is complete before introducing compile-time changes. This phase requires care around three specific items: the Cargo.toml library name change (must be co-committed with `main.rs` import update), the `KeyboardImplementation::HandyKeys` serde key (use `#[serde(rename)]` to preserve wire format), and the portable mode marker string (implement dual-string acceptance).

**Delivers:** A fully debranded codebase where no internal symbol or log output references "Handy"; `bindings.ts` regenerated and committed; GitHub Actions workflow artifact names updated.

**Addresses:** `handy_app_lib` → `dictus_app_lib`, `handy_keys.rs` → `ext_keys.rs`, log file name, CLI binary name, `llm_client.rs` HTTP headers, portable marker string, NSIS installer template, CI workflow artifact prefixes

**Avoids:** Pitfall 8 (portable mode marker), anti-pattern: serde wire format rename, anti-pattern: bindings.ts not regenerated

**Research flag:** The `[patch.crates-io]` Tauri runtime fork (branch `handy-2.10.2` on `cjpais/tauri.git`) warrants a targeted investigation during this phase: determine what the patch actually changes and whether Dictus should fork it to a controlled repository. This is low urgency but should be resolved before V2.

---

### Phase Ordering Rationale

- Phase 1 must come first because the bundle identifier must be set before any build is distributed (changing it after breaks data paths), and the updater/signing configuration are safety concerns that cannot be deferred
- Phase 2 is independent of Phase 3 — visible changes require no Rust compilation changes and can be executed quickly; this lets stakeholders see a branded product early
- Phase 3 is last because internal renames have build-breaking dependencies (Cargo.toml + main.rs must change together) and data-migration risks (serde serialization, portable marker) that need focused attention after the visible rebrand is stable
- The `blob.handy.computer` CDN dependency is a documented risk accepted for V1; it does not block any phase but must be resolved in V2 before Dictus can claim full infrastructure independence

### Research Flags

Phases that can proceed without additional research:

- **Phase 1:** Tauri config fields are fully documented, behavior is deterministic
- **Phase 2:** `tauri icon` CLI, Tailwind `@theme`, and `i18next-parser` are all well-documented tools; icon pipeline is straightforward

Phase that needs targeted investigation during execution:

- **Phase 3 (Tauri patch audit):** The `cjpais/tauri.git` `handy-2.10.2` patch is undocumented in the codebase. Before Phase 3 closes, run `git diff` between the upstream Tauri tag and the patch branch to determine what behavior the patch enables. Decide whether to fork to a Dictus-controlled repository. This affects long-term security and maintainability.

## Confidence Assessment

| Area         | Confidence | Notes                                                                                                                |
| ------------ | ---------- | -------------------------------------------------------------------------------------------------------------------- |
| Stack        | HIGH       | All tools verified via official Tauri and Tailwind docs; version constraints confirmed against package.json          |
| Features     | HIGH       | Rebrand scope well-defined; STT feature landscape confirmed via competitor research; codebase verified               |
| Architecture | HIGH       | Based on direct codebase inspection of all relevant files; dependency graph confirmed by reading source              |
| Pitfalls     | HIGH       | All critical pitfalls verified directly in source files (`tauri.conf.json`, `portable.rs`, `model.rs`, locale files) |

**Overall confidence:** HIGH

### Gaps to Address

- **Tauri runtime patch scope:** What does `cjpais/tauri.git` branch `handy-2.10.2` actually change vs. upstream Tauri 2.10.x? This is unknown without inspecting the fork diff. Address during Phase 3 planning.
- **Dictus brand assets:** No Dictus logo, wordmark, or iOS palette reference was available during research. Phase 2 cannot complete until a 1024x1024 app icon source and the Dictus iOS color palette are provided. This is a dependency on design, not engineering.
- **Model CDN decision:** Will Dictus mirror `blob.handy.computer` model files, or continue relying on the upstream CDN for V1? This decision should be recorded in a `PROJECT.md` note before V2 planning.
- **Updater endpoint readiness:** Phase 1 should disable the updater (`createUpdaterArtifacts: false`) until a Dictus-controlled GitHub releases workflow exists. The timeline for that workflow is not defined in current research.

## Sources

### Primary (HIGH confidence)

- Tauri v2 configuration reference — `v2.tauri.app/reference/config/` — `productName`, `identifier`, `bundle`, `plugins.updater` fields
- Tauri v2 App Icons documentation — `v2.tauri.app/develop/icons/` — icon generation pipeline, required sizes
- Tailwind CSS v4 Theme Variables — `tailwindcss.com/docs/theme` — `@theme` directive, CSS variable generation
- Tauri Configuration Files reference — `v2.tauri.app/develop/configuration-files/` — platform-specific config merge behavior
- Direct codebase analysis: `tauri.conf.json`, `Cargo.toml`, all Rust source files in `src-tauri/src/`, all TypeScript/TSX source files, all locale files, `nsis/installer.nsi`, `.github/workflows/`

### Secondary (MEDIUM confidence)

- whisper.cpp language detection issue #1831 — forced language override edge case with multilingual models
- GitHub issue tauri-apps/tauri #13874 — `productName` vs Cargo `name` divergence on macOS
- OpenWhispr, Superwhisper, Voibe competitor feature analysis — table stakes STT feature set (confirms all required features already exist in codebase)
- Speechify Windows app TechCrunch 2026 — market context for privacy-first local STT positioning

### Tertiary (LOW confidence / needs validation)

- `cjpais/tauri.git` patch diff — not inspected; patch purpose is undocumented; must be audited in Phase 3

---

_Research completed: 2026-04-05_
_Ready for roadmap: yes_
