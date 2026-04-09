# Architecture Research

**Domain:** Tauri 2.x desktop app rebranding (Handy → Dictus Desktop)
**Researched:** 2026-04-05
**Confidence:** HIGH (based on direct codebase analysis, not assumptions)

---

## Standard Architecture

### System Overview

```
┌──────────────────────────────────────────────────────────────────┐
│                     IDENTITY LAYER (Branding)                    │
│  tauri.conf.json · Cargo.toml · package.json · nsis/installer   │
│  icons/ · resources/handy.png · i18n JSON strings               │
├──────────────────────────────────────────────────────────────────┤
│                    FRONTEND (React/TypeScript)                    │
│  ┌─────────────┐  ┌──────────────────┐  ┌──────────────────┐    │
│  │  Sidebar.tsx│  │ Onboarding.tsx   │  │  AboutSettings   │    │
│  │ HandyHand   │  │ HandyTextLogo    │  │  (URLs/links)    │    │
│  │ HandyText   │  │ AccessOnboarding │  │                  │    │
│  └─────────────┘  └──────────────────┘  └──────────────────┘    │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │   HandyKeysShortcutInput.tsx / ShortcutInput.tsx        │    │
│  │   (listens to "handy-keys-event", calls handy_keys cmds)│    │
│  └─────────────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │   bindings.ts (auto-generated: startHandyKeysRecording) │    │
│  └─────────────────────────────────────────────────────────┘    │
├──────────────────────────────────────────────────────────────────┤
│              IPC BOUNDARY (Tauri commands + events)              │
│   "handy-keys-event" event · start_handy_keys_recording cmd      │
│   stop_handy_keys_recording cmd                                  │
├──────────────────────────────────────────────────────────────────┤
│                    BACKEND (Rust / src-tauri)                    │
│  ┌──────────────────────┐  ┌──────────────────────────────┐     │
│  │  shortcut/           │  │  lib.rs                      │     │
│  │  ├── handy_keys.rs   │  │  (crate: handy_app_lib)      │     │
│  │  │   HandyKeysState  │  │  window title "Handy"        │     │
│  │  │   "handy-keys-*"  │  │  log file name "handy"       │     │
│  │  └── mod.rs          │  │                              │     │
│  └──────────────────────┘  └──────────────────────────────┘     │
│  ┌──────────────────────┐  ┌──────────────────────────────┐     │
│  │  tray.rs             │  │  cli.rs                      │     │
│  │  "Handy v{}"         │  │  command(name = "handy")     │     │
│  │  resources/handy.png │  │  about = "Handy - STT"       │     │
│  │  "handy-1.wav"       │  │                              │     │
│  └──────────────────────┘  └──────────────────────────────┘     │
│  ┌──────────────────────┐  ┌──────────────────────────────┐     │
│  │  llm_client.rs       │  │  portable.rs                 │     │
│  │  Referer: cjpais/    │  │  "Handy Portable Mode"       │     │
│  │  User-Agent: Handy   │  │  handy_test_* (temp dirs)    │     │
│  │  X-Title: Handy      │  │                              │     │
│  └──────────────────────┘  └──────────────────────────────┘     │
│  ┌──────────────────────┐  ┌──────────────────────────────┐     │
│  │  settings.rs         │  │  managers/ · commands/       │     │
│  │  HandyKeys enum      │  │  (no Handy-branded symbols)  │     │
│  │  "handy_keys" key    │  │                              │     │
│  └──────────────────────┘  └──────────────────────────────┘     │
├──────────────────────────────────────────────────────────────────┤
│                    EXTERNAL DEPENDENCY                           │
│  Cargo.toml: handy-keys = "0.2.4" (third-party crate)           │
│  Cargo.toml: [patch.crates-io] → github.com/cjpais/tauri.git    │
│             branch: handy-2.10.2 (forked Tauri runtime)         │
└──────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities — Branding Dimension

| Component                                                          | Current Handy Identity                                                             | Rebrand Target                                                                                           | Boundary Type                                            |
| ------------------------------------------------------------------ | ---------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- | -------------------------------------------------------- |
| `tauri.conf.json`                                                  | `productName: "Handy"`, `identifier: "com.pais.handy"`, updater URL `cjpais/Handy` | `"Dictus Desktop"`, `"com.dictus.desktop"`, updater URL TBD                                              | Config file (hard change)                                |
| `src-tauri/Cargo.toml`                                             | `name = "handy"`, `default-run = "handy"`, `[lib] name = "handy_app_lib"`          | `"dictus-desktop"`, `"dictus_desktop_lib"`                                                               | Build config (requires code change in `main.rs` imports) |
| `package.json`                                                     | `"name": "handy-app"`                                                              | `"dictus-desktop"`                                                                                       | Config file (soft change)                                |
| `src/components/icons/HandyHand.tsx`                               | SVG icon — a hand graphic, Handy branded                                           | Replace with Dictus logo SVG component                                                                   | Component (rename + new content)                         |
| `src/components/icons/HandyTextLogo.tsx`                           | SVG wordmark — renders "Handy" text                                                | Replace with Dictus wordmark SVG                                                                         | Component (rename + new content)                         |
| `src/components/Sidebar.tsx`                                       | Uses `HandyHand`, `HandyTextLogo`                                                  | Use renamed icon components                                                                              | Component (rename imports)                               |
| `src/components/onboarding/Onboarding.tsx`                         | Uses `HandyTextLogo`, mentions Handy                                               | Dictus logo + Dictus copy                                                                                | Component (rename + copy change)                         |
| `src/components/onboarding/AccessibilityOnboarding.tsx`            | Uses `HandyTextLogo`, "Handy needs accessibility…"                                 | Dictus logo + Dictus copy                                                                                | Component (rename + copy change)                         |
| `src/i18n/locales/en/translation.json`                             | 12+ "Handy" occurrences in strings                                                 | "Dictus" everywhere                                                                                      | i18n source (all 4 languages)                            |
| `src/components/settings/about/AboutSettings.tsx`                  | Links to `handy.computer/donate`, `cjpais/Handy`                                   | Dictus URLs / attribution                                                                                | Component (URL update)                                   |
| `src/components/update-checker/UpdateChecker.tsx`                  | Links to `cjpais/Handy/releases`                                                   | Dictus releases URL                                                                                      | Component (URL update)                                   |
| `src/components/settings/HandyKeysShortcutInput.tsx`               | Component name, listens to `"handy-keys-event"`                                    | Rename file/component; event name is tied to `handy-keys` crate                                          | Internal API (Phase 2 rename)                            |
| `src/components/settings/debug/DebugPaths.tsx`                     | Shows `%APPDATA%/handy` paths                                                      | `%APPDATA%/com.dictus.desktop` or OS-derived                                                             | UI string (derived from bundle ID)                       |
| `src/components/settings/debug/KeyboardImplementationSelector.tsx` | Label `"Handy Keys"`                                                               | `"System Keys"` or keep as-is                                                                            | UI label                                                 |
| `src-tauri/src/shortcut/handy_keys.rs`                             | Module name, `HandyKeysState`, wraps `handy-keys` crate                            | Rename file to `ext_keys.rs` or similar; rename state struct                                             | Internal (Phase 2)                                       |
| `src-tauri/src/settings.rs`                                        | `KeyboardImplementation::HandyKeys`, serialized as `"handy_keys"`                  | Rename enum variant; `"handy_keys"` serialization key is persisted in user settings — migration required | Persisted data (migration risk)                          |
| `src-tauri/src/tray.rs`                                            | `"Handy v{}"` tray tooltip, `resources/handy.png`                                  | `"Dictus Desktop v{}"`, `resources/dictus.png`                                                           | Runtime display + asset                                  |
| `src-tauri/src/lib.rs`                                             | Window title `"Handy"`, log file `"handy"`, calls `handy_keys::*`                  | `"Dictus Desktop"`, `"dictus"`, updated module refs                                                      | Core bootstrap                                           |
| `src-tauri/src/cli.rs`                                             | Binary CLI name `"handy"`, about `"Handy - Speech to Text"`                        | `"dictus-desktop"`, updated description                                                                  | CLI identity                                             |
| `src-tauri/src/llm_client.rs`                                      | HTTP headers: `Referer`, `User-Agent`, `X-Title` all reference Handy               | Update to Dictus identity + repository URL                                                               | Network identity                                         |
| `src-tauri/src/portable.rs`                                        | Marker string `"Handy Portable Mode"`, temp dir names `handy_test_*`               | `"Dictus Portable Mode"` (breaks existing portable installs)                                             | Data migration risk                                      |
| `src-tauri/nsis/installer.nsi`                                     | Comments + strings reference Handy, `"Handy Portable Mode"` marker                 | Update strings; keep marker check backward-compat or handle migration                                    | Windows installer                                        |
| `src-tauri/resources/handy.png`                                    | Tray icon (colored, idle state)                                                    | Rename to `dictus.png`, replace content                                                                  | Asset file                                               |
| `src-tauri/icons/`                                                 | App icons (generic, not Handy-branded visually)                                    | Replace with Dictus brand icons                                                                          | Asset files                                              |
| `handy-keys` crate (external)                                      | Third-party crate name — not renameable                                            | Crate stays; internal wrappers can be renamed                                                            | External dependency                                      |
| `[patch.crates-io] handy-2.10.2`                                   | Forked Tauri runtime on `cjpais/handy-2.10.2` branch                               | Branch name stays (external); no action needed                                                           | External fork                                            |

---

## Recommended Project Structure (Post-Rebrand)

No structural reorganization is needed. The existing structure is sound. Only renaming within the structure:

```
src/
├── components/
│   ├── icons/
│   │   ├── DictusLogo.tsx          # was HandyTextLogo.tsx
│   │   ├── DictusIcon.tsx          # was HandyHand.tsx
│   │   └── ...
│   ├── settings/
│   │   ├── ExtKeysShortcutInput.tsx  # was HandyKeysShortcutInput.tsx (Phase 2)
│   │   └── ...
│   └── ...
├── i18n/locales/
│   ├── en/translation.json         # replace all "Handy" → "Dictus"
│   ├── es/translation.json         # same
│   ├── fr/translation.json         # same
│   └── vi/translation.json         # same

src-tauri/
├── src/
│   ├── shortcut/
│   │   ├── ext_keys.rs             # was handy_keys.rs (Phase 2)
│   │   └── mod.rs                  # updated references
│   ├── lib.rs                      # window title, log name, crate ref
│   ├── tray.rs                     # tray tooltip, resource path
│   ├── cli.rs                      # binary name, description
│   ├── llm_client.rs               # HTTP headers
│   └── portable.rs                 # marker string (migration-sensitive)
├── resources/
│   ├── dictus.png                  # was handy.png (tray colored idle)
│   └── ...
├── icons/                          # replace all PNGs + .icns + .ico
├── Cargo.toml                      # crate name + lib name
└── tauri.conf.json                 # productName, identifier, updater endpoint
```

---

## Architectural Patterns

### Pattern 1: Two-Pass Rebrand (Visible First, Internal Second)

**What:** Separate the rebrand into two distinct passes — user-visible identity changes first, then internal code symbol renaming.

**When to use:** When internal renaming carries non-trivial risk (persisted data keys, IPC event names, generated bindings) and the visible rebrand has independent value.

**Trade-offs:**

- Pro: Visible rebrand ships quickly without risk of build breakage from Cargo crate rename
- Pro: Settings migration can be scoped to a dedicated phase with a tested migration path
- Con: Codebase temporarily has mismatched names (e.g., `HandyKeysShortcutInput` in a Dictus-branded UI)
- Con: Two PRs, slightly longer overall timeline

**Implementation:**

```
Pass 1 (visible):
  tauri.conf.json → productName, identifier
  Cargo.toml      → description only (keep name="handy" to defer binary rename risk)
  icons/          → replace all PNGs
  resources/      → rename handy.png → dictus.png + update tray.rs reference
  src/icons/      → rename/replace HandyTextLogo, HandyHand
  src/i18n/       → all 4 language files
  src/components/ → Sidebar, Onboarding, About, UpdateChecker
  lib.rs          → window title, log file name

Pass 2 (internal):
  Cargo.toml      → name, lib name (binary rename)
  main.rs         → update handy_app_lib import
  shortcut/handy_keys.rs → rename file + struct
  settings.rs     → KeyboardImplementation::HandyKeys variant rename + migration
  bindings.ts     → regenerated automatically from Rust changes
  portable.rs     → marker string (migration logic for existing installs)
  nsis/           → template strings
```

### Pattern 2: Config-Driven Identity (Tauri)

**What:** Tauri derives many identity strings from `tauri.conf.json`. Changing `productName` and `identifier` propagates to:

- OS app registration (macOS bundle ID, Windows registry key, Linux .desktop file)
- App data directory path (OS derives this from the identifier)
- Auto-updater artifact lookup

**When to use:** Always change `tauri.conf.json` first in Pass 1 — it is the canonical identity source.

**Critical implication:** Changing `identifier` from `com.pais.handy` to `com.dictus.desktop` changes the OS-level app data directory path. Existing user data (settings, history, models) stored at `%APPDATA%/com.pais.handy` will no longer be found. This is a **data migration concern** — either:

- Accept data loss for V1 (fresh install experience)
- Implement a one-time migration in `lib.rs::run()` that moves data at first launch with new identifier

### Pattern 3: Persisted Key Stability (Settings Migration)

**What:** The `KeyboardImplementation` enum variant `HandyKeys` serializes as `"handy_keys"` in `settings_store.json`. If renamed to e.g. `ExtKeys` serializing as `"ext_keys"`, existing users get a deserialization failure and fall back to defaults — their keyboard implementation choice is reset.

**When to use:** Any time a Rust enum variant used in persistent settings is renamed.

**Trade-offs:**

- Pro of renaming: Internal code is fully debranded
- Con: Silent setting reset for existing users, hard to debug

**Safe implementation:**

```rust
#[serde(rename = "handy_keys")]  // keep wire format stable
ExtKeys,                          // rename the Rust symbol
```

This is the lowest-risk path and is fully supported by serde.

---

## Data Flow

### Rebrand Touch Point Dependencies

```
tauri.conf.json (identifier change)
    ↓ affects
OS app data directory path
    ↓ requires
Migration logic in lib.rs (copy old dir → new dir on first launch)

Cargo.toml (lib name: handy_app_lib → dictus_desktop_lib)
    ↓ requires
main.rs updated import: use dictus_desktop_lib::CliArgs

shortcut/handy_keys.rs (rename file + struct)
    ↓ requires
shortcut/mod.rs updated references
lib.rs updated references (handy_keys::start_handy_keys_recording)
    ↓ triggers
Rebuild of bindings.ts (auto-generated via tauri-specta on debug build)
    ↓ requires
src/components/settings/HandyKeysShortcutInput.tsx updated imports

icons/ replacement
    ↓ no code changes required
    (tauri.conf.json already references icons/ by generic names: 32x32.png, icon.icns)

resources/handy.png → resources/dictus.png
    ↓ requires
tray.rs: update path string at line 59
```

### Settings Flow (Unchanged by Rebrand)

```
useSettings() hook
    ↓
settingsStore (Zustand)
    ↓ initialized from
commands.getAppSettings()
    ↓
settings_store.json (path derived from identifier via Tauri)
```

The settings flow itself does not change — only the filesystem path where `settings_store.json` lives changes when the identifier changes.

---

## Scaling Considerations

This is a desktop app rebrand, not a scale concern. The applicable "scaling" dimension is across three platforms (macOS, Windows, Linux) and four locales (en, es, fr, vi).

| Platform | Rebrand-Specific Notes                                                                                                                                              |
| -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| macOS    | Bundle ID change affects LaunchAgent (autostart), Keychain entries. `signingIdentity` in `tauri.conf.json` may need update for production distribution.             |
| Windows  | NSIS installer uses `"Handy Portable Mode"` marker string — needs update. Registry path changes with `identifier`. `signCommand` references `"Handy"` display name. |
| Linux    | `.desktop` file generated from `productName` — no manual action beyond `tauri.conf.json`. AppImage and RPM package names derive from `productName`.                 |

---

## Anti-Patterns

### Anti-Pattern 1: Renaming the `handy-keys` Crate Name

**What people do:** Attempt to alias or rename the `handy-keys` external crate in `Cargo.toml` to remove the Handy reference.

**Why it's wrong:** The crate is a third-party dependency (`handy-keys = "0.2.4"`). Its name cannot be changed. Attempting `handy-keys = { package = "handy-keys", ... }` with a local rename changes the Rust import path but adds confusion. The crate is an implementation detail — users never see the crate name.

**Do this instead:** Rename only the internal module file (`handy_keys.rs` → `ext_keys.rs`) and the internal struct (`HandyKeysState` → `ExtKeysState`). The extern crate remains as-is with `use handy_keys::...` inside the module.

### Anti-Pattern 2: Renaming the Settings Key Wire Format

**What people do:** Rename `KeyboardImplementation::HandyKeys` including its serde serialization from `"handy_keys"` to `"ext_keys"`.

**Why it's wrong:** `settings_store.json` on existing user machines contains `"keyboard_implementation": "handy_keys"`. After the rename, deserialization silently falls back to the default, resetting the user's setting without any warning.

**Do this instead:** Use `#[serde(rename = "handy_keys")]` on the renamed variant to keep the wire format stable, or write an explicit migration that reads the old value and writes the new one on first launch.

### Anti-Pattern 3: Changing the App Identifier Without a Data Migration

**What people do:** Change `identifier` in `tauri.conf.json` from `com.pais.handy` to `com.dictus.desktop` and ship, treating it as a config-only change.

**Why it's wrong:** Tauri derives the OS app data directory path from the identifier. On macOS, data lives at `~/Library/Application Support/com.pais.handy/`. After the change, the app reads from `~/Library/Application Support/com.dictus.desktop/` — a new empty directory. All user settings, history, and model downloads become invisible to the app.

**Do this instead:** For V1, either explicitly accept this as a "fresh install" (acceptable for a fork → rebrand transition) and document it, or implement a one-time migration in `lib.rs::run()` early in the setup closure that detects and copies the old data directory.

### Anti-Pattern 4: Regenerating bindings.ts Without Committing

**What people do:** Rename Rust command functions (e.g., `start_handy_keys_recording` → `start_ext_keys_recording`), rebuild, but forget to commit the regenerated `bindings.ts`.

**Why it's wrong:** `bindings.ts` is committed to git (per the codebase conventions). If it's not updated, TypeScript code still calls the old command name, breaking at runtime. Other contributors won't get the update.

**Do this instead:** After any Rust command rename, run `bun run tauri dev` once to regenerate `bindings.ts`, then commit both the Rust changes and the updated `bindings.ts` in the same commit.

### Anti-Pattern 5: Replacing Icons Without All Required Sizes

**What people do:** Create a new `icon.png` and replace only `icons/icon.png`, `icons/icon.icns`, `icons/icon.ico`.

**Why it's wrong:** Tauri bundles multiple icon sizes. `tauri.conf.json` explicitly lists: `32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.icns`, `icon.ico`. Windows Store packaging requires Square\* variants. The tray uses `resources/handy.png` (separate from the app icons).

**Do this instead:** Generate all required sizes using `tauri icon <source>` from a single high-resolution source (1024×1024 PNG). Also replace `resources/handy.png` → `resources/dictus.png` and update the reference in `tray.rs`.

---

## Integration Points

### External Services

| Service                    | Integration Pattern                             | Branding Touch Point                                                                                                                                      |
| -------------------------- | ----------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| GitHub Releases (updater)  | `tauri.conf.json` → `plugins.updater.endpoints` | URL hardcoded to `cjpais/Handy/releases/latest` — must point to Dictus releases. Also requires new signing key pair if updater public key changes.        |
| OpenRouter / LLM providers | `llm_client.rs` HTTP headers                    | `Referer`, `User-Agent`, `X-Title` all set to Handy. Change to Dictus identity.                                                                           |
| Windows Code Signing       | `tauri.conf.json` `bundle.windows.signCommand`  | References `"Handy"` display name in signing CLI. Update to Dictus.                                                                                       |
| macOS autostart            | `tauri-plugin-autostart` via LaunchAgent        | LaunchAgent plist name derives from `identifier`. Changing identifier changes plist name; existing autostart entries from old identifier become orphaned. |

### Internal Boundaries

| Boundary                          | Communication                             | Branding Notes                                                                                                                                                                                                       |
| --------------------------------- | ----------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Frontend ↔ Backend (IPC events)  | Tauri events by string name               | `"handy-keys-event"` event name is internal to the handy-keys wrapper. Changing it requires coordinated update in both `handy_keys.rs` and `HandyKeysShortcutInput.tsx`. Low priority — users never see event names. |
| Rust binary ↔ Rust lib           | `use handy_app_lib::CliArgs` in `main.rs` | When `[lib] name` changes in `Cargo.toml`, this import must be updated. Compile-time error if missed.                                                                                                                |
| tauri-specta code generation      | Debug build generates `src/bindings.ts`   | Any renamed Rust command function propagates to TypeScript. Must commit updated `bindings.ts`.                                                                                                                       |
| Portable mode ↔ settings storage | Marker file `Handy Portable Mode`         | Windows portable installs check for this exact string. Changing it breaks detection of existing portable installations. Requires backward-compatible string check in `portable.rs`.                                  |

---

## Build Order for Rebrand Phases

The dependency graph determines safe execution order:

```
Phase 1 — Visual Identity (no build risk)
  1a. icons/ → replace all PNGs + .icns + .ico   (asset swap, no code)
  1b. resources/handy.png → resources/dictus.png  (asset rename)
  1c. src/components/icons/ → rename+replace SVG components
  1d. src/i18n/ → update all 4 language files     (no compile step needed)
  1e. tauri.conf.json → productName, identifier    (DECISION GATE: accept data loss or write migration first)
      → if migration: implement in lib.rs setup closure first, test, then change identifier
  1f. src/components/ → Sidebar, Onboarding, About, UpdateChecker

Phase 2 — Bundle Identity (build config changes)
  2a. package.json → name                          (low risk, npm name only)
  2b. src-tauri/Cargo.toml → description           (safe, display only)
  2c. src-tauri/Cargo.toml → name, default-run, [lib] name  (compile break if main.rs not updated simultaneously)
      → must update main.rs import in same commit
  2d. lib.rs → window title, log file name
  2e. cli.rs → command name, about string
  2f. llm_client.rs → HTTP headers
  2g. tray.rs → tray tooltip string, resource path  (after 1b asset rename)

Phase 3 — Internal Symbol Rename (highest risk, lowest visible impact)
  3a. settings.rs → rename KeyboardImplementation::HandyKeys  (use serde rename to keep wire format)
  3b. shortcut/handy_keys.rs → rename file to ext_keys.rs, rename HandyKeysState
  3c. shortcut/mod.rs → update all references
  3d. lib.rs → update handy_keys:: references
  3e. bun run tauri dev → regenerate bindings.ts
  3f. src/components/settings/HandyKeysShortcutInput.tsx → rename file + component
  3g. portable.rs → update marker string  (migration: keep backward-compat check for old string)
  3h. nsis/installer.nsi → update Handy references + portable marker string
```

**Critical dependency:** Phase 2c (Cargo.toml lib name change) MUST be in the same commit as the `main.rs` import update. Splitting them produces a broken build.

**Decision gate before Phase 1e:** Determine whether existing user data migration is required. This decision gates the identifier change and should be resolved in planning before coding starts.

---

## Sources

- Direct analysis of `/Users/pierreviviere/dev/dictus-desktop/src-tauri/Cargo.toml` (crate identity)
- Direct analysis of `/Users/pierreviviere/dev/dictus-desktop/src-tauri/tauri.conf.json` (bundle config)
- Direct analysis of `/Users/pierreviviere/dev/dictus-desktop/src-tauri/src/` (all Rust files with Handy references)
- Direct analysis of `/Users/pierreviviere/dev/dictus-desktop/src/` (all TypeScript/TSX files with Handy references)
- Direct analysis of `/Users/pierreviviere/dev/dictus-desktop/src-tauri/nsis/installer.nsi` (Windows installer)
- Direct analysis of `/Users/pierreviviere/dev/dictus-desktop/src/i18n/locales/en/translation.json` (i18n strings)
- Tauri 2.x documentation on bundle identifier and app data paths (HIGH confidence — well-established Tauri behavior)

---

_Architecture research for: Tauri 2.x app rebranding (Handy → Dictus Desktop)_
_Researched: 2026-04-05_
