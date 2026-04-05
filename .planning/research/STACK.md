# Stack Research

**Domain:** Tauri 2.x desktop app rebranding (identity, assets, bundle config, design tokens)
**Researched:** 2026-04-05
**Confidence:** HIGH (Tauri config/icons verified via official docs; Tailwind v4 tokens verified via official docs; migration pitfalls verified via codebase inspection)

---

## Context: What This Research Covers

This is not a greenfield stack decision. The Tauri 2.x + React + TypeScript + Tailwind + Rust stack is fixed. This research answers: **what tools and approaches govern a safe, complete rebrand** of this specific app from Handy to Dictus Desktop?

The four work layers in scope:

1. **Bundle identity** — `tauri.conf.json`, `Cargo.toml`, platform-specific outputs
2. **Icon/asset pipeline** — generating all required icon formats from a single source
3. **Design tokens** — Tailwind v4 `@theme` for palette/typography alignment with Dictus iOS
4. **i18n string updates** — 21 locale files, ESLint-enforced translation coverage

---

## Recommended Stack

### Core Technologies

| Technology | Version (current) | Purpose | Why This Approach |
|------------|-------------------|---------|-------------------|
| `tauri.conf.json` fields: `productName`, `identifier` | Tauri 2.10.2 | Controls user-visible app name and all system paths (bundle ID, app data dir, webview data dir, update endpoint path) | Single source of truth for Tauri. Changing `identifier` cascades to ALL platform-specific outputs automatically — no manual plist/registry edits needed. **HIGH confidence** — official docs confirm. |
| `Cargo.toml` package metadata | Rust stable | Controls binary name (`default-run`), crate name, description | Must stay in sync with `tauri.conf.json`. Cargo binary name affects CLI invocation and macOS `.app` bundle internals. Divergence between Cargo name and Tauri `productName` is a known source of macOS bundle confusion (GitHub issue #13874). |
| `tauri icon` CLI command | @tauri-apps/cli 2.10.0 | Generates all required icon sizes from a single 1024x1024 PNG or SVG source | One command, zero manual resizing. Outputs: `32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.icns`, `icon.ico` into `src-tauri/icons/`. No external tooling needed. **HIGH confidence** — verified via official Tauri docs. |
| Tailwind v4 `@theme` directive | Tailwind CSS 4.1.16 (already installed) | CSS-first design token system; defines brand colors, typography, spacing as CSS custom properties | v4 `@theme` replaces the JS config pattern entirely. Every `--color-*` token in `@theme` auto-generates utility classes AND becomes a CSS variable on `:root`. This is the correct approach for Dictus palette injection without touching any JS config. **HIGH confidence** — verified via tailwindcss.com/docs/theme. |
| `i18next-parser` | ^9.x (add as dev dep) | Scans all source files, extracts `t('key')` calls, flags missing/orphaned keys across all 21 locale files | Prevents silent regressions during rebrand (old keys left in JSON, new keys missing from non-EN locales). The existing `eslint-plugin-i18next` enforces usage in JSX but does not detect orphaned or untranslated keys. |

### Supporting Libraries / Tools

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `sharp` (Node.js) or `Inkscape` CLI | Latest | Pre-processing source icon to exact 1024x1024 PNG with transparency before feeding to `tauri icon` | Use when the Dictus brand SVG/logo needs compositing, padding, or background removal. `tauri icon` accepts SVG directly but PNG gives more predictable output for complex brand marks. |
| macOS `iconutil` (system tool) | Built-in | Inspect generated `.icns` for correct sizes | Use after `tauri icon` to sanity-check the `.icns` bundle, especially for macOS 26+ template icon requirements (see Pitfalls). |
| `cargo fmt` + `cargo clippy` | Rust stable | Validate Rust code after internal symbol renames (`HandyKeys` → `DictusKeys`, etc.) | Run after any Rust identifier changes to catch missed references and enforce style. |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| `bun run tauri icon` | Single-command icon generation | Run from repo root. Source file: `./app-icon.png` by default. Place a 1024x1024 PNG at that path before running. |
| `tauri.macos.conf.json` | macOS-specific config overrides | Merge cleanly over `tauri.conf.json`. Use for macOS-specific signing identity, entitlements path, minimum OS version — do not duplicate shared fields. |
| `tauri.windows.conf.json` | Windows-specific config overrides | Use to update the `signCommand` with the new Dictus signing identity. The current value references `Handy` in the `-d` description flag. |
| Text search + replace (Grep/sed) | Bulk string replacement across translation JSON files | 21 locale files all contain "Handy" strings. Automated search-replace is safer than manual editing for strings that appear 11 times per locale file. |

---

## Installation

No new runtime dependencies are required. The rebrand is achieved entirely through configuration changes and asset replacement.

```bash
# Add i18next-parser as a dev dependency for key auditing
bun add -D i18next-parser

# Run icon generation after placing app-icon.png at repo root
bun run tauri icon

# After Rust identifier renames, validate the codebase
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml
```

---

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| `tauri icon` CLI for all icon generation | Manual creation of each PNG size + separate `.icns`/`.ico` tools | Never for this project — `tauri icon` is the official, maintained path. Manual creation risks missing required sizes. |
| Tailwind v4 `@theme` directive in CSS | CSS custom properties in `:root` directly | Only if you need to share tokens with non-Tailwind stylesheets. For this project, `@theme` is strictly better — it auto-generates utility classes AND CSS variables simultaneously. |
| `i18next-parser` for key audit | Manual cross-check of 21 JSON files | Only for tiny projects (<3 locales). At 21 locales, manual is error-prone and does not scale. |
| `portable.rs` magic string update in place | Creating a migration command | In-place update is simpler and the existing file already has the right abstraction (`is_valid_portable_marker`). Just update the string constant and add backward-compat check. |

---

## What NOT to Do

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| Changing `identifier` without a first-run migration | `identifier` determines the app data directory path on all platforms (`${dataDir}/${identifier}`). Changing from `com.pais.handy` to `com.dictus.desktop` will orphan existing users' settings, history DB, model downloads, and `tauri-plugin-store` file. This affects `~/.local/share/com.pais.handy/` (Linux), `%APPDATA%\com.pais.handy\` (Windows), `~/Library/Application Support/com.pais.handy/` (macOS). | Implement a first-run migration in Rust that checks for the old path and copies data to the new path before Tauri initializes the store. The `portable.rs` abstraction pattern is a good model. |
| Renaming Rust symbols (`HandyKeys`, `handy_app_lib`, `default-run = "handy"`) before confirming they are not serialized | `KeyboardImplementation::HandyKeys` is serialized as `"handy_keys"` via `#[serde(rename_all = "snake_case")]`. If you rename the enum variant, existing settings JSON files break on deserialization. | Add a `#[serde(rename = "handy_keys")]` attribute to preserve the serialized form, OR implement a settings migration that maps the old value to the new one. This is the V1 "progressive renaming" strategy from the project plan. |
| Editing `.icns` and `.ico` files by hand | These are binary formats with embedded size variants. Manual edits corrupt the file. | Always regenerate from source via `bun run tauri icon`. Keep the source `app-icon.png` in the repo. |
| Using a `tailwind.config.js` file for design tokens | Tailwind v4 (already installed at 4.1.16) has deprecated the JS config approach for theme values. Using it creates a dual-config situation that causes confusing precedence behavior. | Define all brand tokens in the CSS entry point via `@theme`. The project already uses the Vite integration (`@tailwindcss/vite`) which expects the CSS-first approach. |
| Updating the updater `pubkey` and `endpoints` without a new release | The updater plugin verifies release signatures against the public key. Changing the endpoint to a Dictus GitHub org requires updating the key pair and re-signing releases. | For V1 internal/test builds, keep the updater pointing to the existing endpoint or disable it. Update the endpoint + key only when a real Dictus GitHub releases workflow is ready. |
| Adding "Handy Portable Mode" string removal to V1 scope | `portable.rs` line 30 writes `"Handy Portable Mode"` as a magic marker to disk. This is a portable install detection mechanism. Changing the string is a **breaking change for existing portable installs** and requires a backward-compat migration path. | Defer this to V2 when a broader portable mode migration strategy can be implemented. The visible UI impact of this string is zero (it is written to a marker file, not displayed). |

---

## Stack Patterns by Layer

**Layer 1 — Bundle Identity (tauri.conf.json + Cargo.toml):**
- Change `productName: "Handy"` → `"Dictus Desktop"`
- Change `identifier: "com.pais.handy"` → `"com.dictus.desktop"`
- Change `Cargo.toml` `name`, `description`, `default-run` → `dictus-desktop`
- Change `Cargo.toml` `[lib] name` → `dictus_app_lib`
- Update `windows.signCommand` `-d` flag value from `Handy` to `Dictus Desktop`
- Update updater `endpoints` URL when Dictus GitHub org releases are configured
- **Prerequisite:** Implement data migration before this change ships to real users

**Layer 2 — Icon/Asset Pipeline:**
- Place 1024x1024 `app-icon.png` (Dictus branding, transparency, square) at repo root
- Run `bun run tauri icon` — this replaces all files in `src-tauri/icons/`
- Replace tray icons in `src-tauri/resources/`: `tray_idle.png`, `tray_idle_dark.png`, `tray_recording.png`, `tray_recording_dark.png`, `tray_transcribing.png`, `tray_transcribing_dark.png`
- Replace `src-tauri/resources/recording.png`, `transcribing.png`, `handy.png`
- Replace `src-tauri/icons/logo.png`
- Tray icons are 22x22px (macOS) / 16x16px (Windows) — design them as template images (monochrome) for macOS so the system handles dark/light mode automatically via `iconAsTemplate: true`

**Layer 3 — Design Tokens (Tailwind v4 CSS):**
- Locate the CSS entry point (typically `src/index.css` or similar) where `@import "tailwindcss"` lives
- Add a `@theme` block with Dictus brand colors from the iOS reference palette
- Example structure:
  ```css
  @theme {
    --color-dictus-primary: oklch(...);   /* main brand color */
    --color-dictus-surface: oklch(...);   /* card/panel background */
    --color-dictus-text: oklch(...);      /* primary text */
    /* add typography, radius tokens as needed */
  }
  ```
- These tokens auto-generate utility classes (`bg-dictus-primary`, `text-dictus-text`) AND CSS variables (`var(--color-dictus-primary)`)
- Do NOT remove existing color tokens in V1 — add Dictus tokens alongside them, then migrate usage incrementally

**Layer 4 — i18n String Updates:**
- All 21 locale files contain "Handy" references (verified: 258 total occurrences across 33 files)
- English source (`en/translation.json`) is the canonical file — update it first
- The component `HandyTextLogo.tsx` and `HandyHand.tsx` contain hardcoded references — these components need renaming
- `eslint-plugin-i18next` is already configured and will enforce that any JSX string changes go through `t()` calls
- After updating EN, use `i18next-parser` to flag which keys in non-EN locales are stale or missing
- For V1 visible rebrand, only EN + FR are critical to validate manually (other locales can ship with EN fallback temporarily)

---

## Key Configuration File: What Changes Where

| File | Field(s) to Change | Current Value | Target Value |
|------|-------------------|---------------|--------------|
| `src-tauri/tauri.conf.json` | `productName` | `"Handy"` | `"Dictus Desktop"` |
| `src-tauri/tauri.conf.json` | `identifier` | `"com.pais.handy"` | `"com.dictus.desktop"` |
| `src-tauri/tauri.conf.json` | `bundle.windows.signCommand` | `"... -d Handy %1"` | `"... -d \"Dictus Desktop\" %1"` |
| `src-tauri/tauri.conf.json` | `plugins.updater.endpoints` | GitHub cjpais/Handy | Dictus GitHub releases URL |
| `src-tauri/tauri.conf.json` | `plugins.updater.pubkey` | cjpais key | New Dictus key pair |
| `src-tauri/Cargo.toml` | `[package] name` | `"handy"` | `"dictus-desktop"` |
| `src-tauri/Cargo.toml` | `[package] description` | `"Handy"` | `"Dictus Desktop"` |
| `src-tauri/Cargo.toml` | `[package] authors` | `["cjpais"]` | Dictus team author(s) |
| `src-tauri/Cargo.toml` | `[package] default-run` | `"handy"` | `"dictus-desktop"` |
| `src-tauri/Cargo.toml` | `[lib] name` | `"handy_app_lib"` | `"dictus_app_lib"` |
| `src-tauri/nsis/installer.nsi` | Header comment + any `Handy` literal strings | `"Handy"` | `"Dictus Desktop"` |

---

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| `@tauri-apps/cli 2.10.0` | `tauri 2.10.2` | `tauri icon` command is stable. Always run CLI version matching the Tauri crate version. |
| `Tailwind CSS 4.1.16` | `@tailwindcss/vite 4.1.16` | These must match exactly — the Vite plugin and Tailwind core share the version. The project already has them pinned together. |
| `i18next 25.7.2` | TypeScript 5.x only | Project uses TypeScript ~5.6.3, no compatibility issue. i18next v25 dropped TS 4.x support. |
| `eslint-plugin-i18next 6.1.3` | ESLint 9.39.1 | Plugin is already configured. No changes needed for rebrand. |

---

## Sources

- [Tauri App Icons — v2.tauri.app/develop/icons/](https://v2.tauri.app/develop/icons/) — Icon pipeline, source format, generated output list. HIGH confidence.
- [Tauri Config Reference — v2.tauri.app/reference/config/](https://v2.tauri.app/reference/config/) — `productName`, `identifier`, bundle fields, platform mappings. HIGH confidence.
- [Tailwind CSS Theme Variables — tailwindcss.com/docs/theme](https://tailwindcss.com/docs/theme) — `@theme` directive, CSS variable generation, dark mode integration. HIGH confidence.
- [Tauri Configuration Files — v2.tauri.app/develop/configuration-files/](https://v2.tauri.app/develop/configuration-files/) — Platform-specific config merge strategy. HIGH confidence.
- GitHub issue tauri-apps/tauri #13874 — macOS `productName` vs Cargo `name` divergence bug. Informs the recommendation to keep them synchronized.
- Codebase inspection (`portable.rs`, `settings.rs`, `tauri.conf.json`, `Cargo.toml`) — Identifies exact serialized values, migration risks, and scope of "Handy" references (258 occurrences, 33 files). HIGH confidence — direct source of truth.

---

*Stack research for: Tauri 2.x desktop app rebranding (Handy → Dictus Desktop)*
*Researched: 2026-04-05*
