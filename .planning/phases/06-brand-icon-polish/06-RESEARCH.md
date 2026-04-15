# Phase 6: Brand & Icon Polish — Research

**Researched:** 2026-04-15
**Domain:** Tauri 2.x desktop branding, icon generation pipeline, Rust string literals, React/TypeScript frontend patterns, shell scripting
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **BRAND-01 (recording filename):** Change `format!("handy-{}.wav", …)` → `format!("dictus-{}.wav", …)` at `actions.rs:538`. Update test fixtures at `history.rs:689` and `tray.rs:273` from `"handy-{}.wav"` / `"handy-1.wav"` to `"dictus-{}.wav"` / `"dictus-1.wav"`. No migration of existing files on disk.
- **BRAND-02 (portable mode):** Straight string replace only — no dual-read, no legacy recognition. Changes at `portable.rs:5,30,98` and 6 test cases (lines 107-165). Rename test dir names to `dictus_test_*`. REQUIREMENTS.md BRAND-02 "still recognizing legacy" language is explicitly overridden.
- **BRAND-03 (DebugPaths):** Backend command returns one base dir (portable-aware), frontend joins `/models` and `/settings_store.json` suffixes in JSX. Remove `eslint-disable-next-line i18next/no-literal-string` directives. Existing i18n keys preserved.
- **BRAND-04 / SYNC-06 (verify-sync.sh):** Move from `.planning/phases/05-upstream-sync/scripts/verify-sync.sh` to `.github/scripts/verify-sync.sh` via `git mv`. Add four new assertions: BRAND-01a, BRAND-02a, BRAND-03a, ICON-02a. Update all UPSTREAM.md path references.
- **ICON-01/04 (icon source):** New 1024×1024 RGBA transparent-background square PNG added to `dictus-brand` repo (path TBD by Pierre). Do NOT reuse iOS `AppIcon-1024.png` (has baked-in rounded corners). Same waveform glyph, square canvas.
- **ICON-02 (Windows ICO):** Verify with `identify -format '%w\n' src-tauri/icons/icon.ico | sort -u` containing 16, 24, 32, 48, 64, 256. ImageMagick becomes a hard dep for `verify-sync.sh` (same pattern as `jq` check at line 18).
- **ICON-03 (tauri.conf.json):** Extend `bundle.icon` to include `icons/256x256.png` and `icons/512x512.png` explicitly.
- **Icon generation tool:** `bun run tauri icon <path-to-square-1024>.png` — accept default output set.
- **SYNC-06 pulled forward:** verify-sync.sh relocation absorbed into Phase 6. Phase 9 still owns SYNC-07/08/09/10/11.
- **Roadmap ripple:** After Phase 6, `/gsd:roadmap-update` should be run to reflect SYNC-06 completion. Planner must flag this in PLAN.md.

### Claude's Discretion

- Exact wording of commit comment when SYNC-06 move lands.
- Whether to introduce a helper function in `verify-sync.sh` for the icon check vs inlining — match existing script style.
- Exact React pattern in DebugPaths.tsx: dedicated `useAppDataDir()` hook vs inline `useEffect` — pick what's idiomatic with existing settings hooks.
- Whether new `get_app_data_dir_display` command lives in `commands/mod.rs` or a new `commands/debug.rs` — pick based on existing command organization.
- Exact path inside `dictus-brand/` for the new 1024 square PNG — Pierre decides when adding the file.

### Deferred Ideas (OUT OF SCOPE)

- Renaming existing `handy-*.wav` files on disk.
- Dropping `handy-keys` external crate dependency (TECH-01).
- Migrating on-disk app data dir path (DATA-01).
- Dedicated `verify-icons.sh` script separate from `verify-sync.sh`.
- Automating `tauri icon` regeneration in CI.
- Binary rename `handy → dictus` (TECH-03).
- Renaming `handy_keys` module (TECH-01).
- SYNC-07 assertions (updater pubkey/endpoint) in verify-sync.sh.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| BRAND-01 | New recordings named `dictus-{timestamp}.wav` | Exact line confirmed at `actions.rs:538`; test fixtures at `history.rs:689` and `tray.rs:273` confirmed |
| BRAND-02 | Portable mode uses `"Dictus Portable Mode"` string (no dual-read) | All 4 change sites confirmed in `portable.rs` (lines 5, 30, 98, 107-165); test dir names use `handy_test_*` |
| BRAND-03 | DebugPaths shows real path from Tauri API, not hardcoded `%APPDATA%/handy` | Confirmed `get_app_dir_path` command already exists; `AppDataDirectory.tsx` shows the established pattern to reuse |
| BRAND-04 | `verify-sync.sh` gains assertions for BRAND-01/02/03 surfaces | Existing 11 assertions (`check` function pattern) provide the template; 4 new assertions specified in CONTEXT.md |
| ICON-01 | Linux package ships square 256×256 PNG with transparent background | Current `tauri.conf.json` lacks `256x256.png`; `128x128@2x.png` is 256×256 but not explicitly listed for Linux bundles |
| ICON-02 | Windows `.exe` embeds ICO with layers at 16, 24, 32, 48, 64, 256 px | CONFIRMED: ICO already has all 6 layers (verified via `identify`). ICON-02 may already pass — verify-sync assertion still needed |
| ICON-03 | `tauri.conf.json > bundle.icon` lists 256×256 and 512×512 explicitly | Currently lists only 5 entries; `128x128@2x.png` (256×256) and `icon.png` (512×512) exist but are not in the list |
| ICON-04 | Single 1024×1024 square RGBA PNG as source-of-truth in `dictus-brand` | `logo.png` (1024×1024) exists but has transparent corner pixels (macOS-rounded source); new square version needed in `dictus-brand` |
| SYNC-06 | `verify-sync.sh` relocated to `.github/scripts/verify-sync.sh` | `.github/scripts/` directory does not exist yet; must be created. 4 UPSTREAM.md references confirmed at lines 216, 239, 254, 284 |
</phase_requirements>

---

## Summary

Phase 6 makes Dictus identity complete across the last three user-visible leak surfaces (recording filenames, portable mode marker, debug paths display), fixes the Linux icon artifact by regenerating from a square source PNG, and relocates the identity gate script to its permanent home at `.github/scripts/`.

The research reveals that the existing codebase is already well-positioned for this work. A `get_app_dir_path` backend command already exists and is already used by `AppDataDirectory.tsx` — the `DebugPaths.tsx` component simply needs to be rewritten to follow that same established pattern instead of using hardcoded strings. The Windows ICO already contains all required layers (16, 24, 32, 48, 64, 256) so ICON-02 is likely already passing — the ICON-02a verify-sync assertion will confirm this. The biggest dependency is an external one: Pierre must add a square 1024×1024 PNG to the `dictus-brand` repo before icon regeneration can happen.

The `verify-sync.sh` move is a `git mv` plus extension — no logic changes to existing assertions, just new ones appended. The SYNC-05d awk-skip of the acknowledgments block must be preserved verbatim.

**Primary recommendation:** Implement in five independent work units: (1) BRAND-01/02 Rust string replacements, (2) BRAND-03 DebugPaths rewrite, (3) ICON-03/04/01 icon pipeline (blocked on dictus-brand asset), (4) SYNC-06 script relocation + BRAND-04 assertions, (5) UPSTREAM.md path updates.

---

## Standard Stack

### Core (already in project)

| Tool/Library | Version | Purpose | Status |
|---|---|---|---|
| `tauri icon` CLI | Tauri 2.x | Generate all platform icon variants from 1024px source | Already used in Phase 2 |
| ImageMagick `identify` | system | ICO layer introspection, PNG dimension verification | Must be installed to run verify-sync.sh |
| `portable::app_data_dir()` | in-project | Portable-aware path resolution in Rust backend | Confirmed at `portable.rs:60-66` |
| `get_app_dir_path` command | in-project | Tauri command exposing app data dir to frontend | Confirmed in `commands/mod.rs:25-30` and `bindings.ts:440` |
| `tauri_specta` | in-project | Auto-generates TypeScript bindings from `#[tauri::command]` fns | Used in `lib.rs:361`; `collect_commands!` macro |

### No New Dependencies Required

All required tooling is either already in the project or available as system tools. ImageMagick is a dev-env dependency only (for `verify-sync.sh`), not a build dependency.

---

## Architecture Patterns

### Pattern 1: Rust String Replacement (BRAND-01, BRAND-02)

Simple literal string change. No structural changes needed. The functions that need updating are small and isolated.

**BRAND-01 — `actions.rs:538`:**
```rust
// Before:
let file_name = format!("handy-{}.wav", chrono::Utc::now().timestamp());
// After:
let file_name = format!("dictus-{}.wav", chrono::Utc::now().timestamp());
```

**BRAND-02 — `portable.rs:30`:**
```rust
// Before:
let _ = std::fs::write(&marker_path, "Handy Portable Mode");
// After:
let _ = std::fs::write(&marker_path, "Dictus Portable Mode");
```

**BRAND-02 — `portable.rs:98`:**
```rust
// Before:
.map(|s| s.trim().starts_with("Handy Portable Mode"))
// After:
.map(|s| s.trim().starts_with("Dictus Portable Mode"))
```

**BRAND-02 — `portable.rs:5` doc comment:**
```rust
// Before:
/// Portable mode support for Handy.
// After:
/// Portable mode support for Dictus.
```

**Test fixtures (history.rs:689, tray.rs:273):** String replace `"handy-"` → `"dictus-"` in the test helper's `format!` and hardcoded `file_name` value.

**Test dir names (portable.rs:107-165):** Replace `handy_test_*` → `dictus_test_*` in all 6 test function temp dir names.

### Pattern 2: DebugPaths Rewrite (BRAND-03)

The established pattern for fetching backend paths is already present in `AppDataDirectory.tsx`. `DebugPaths.tsx` must be rewritten to follow the same pattern: `useEffect` → `commands.getAppDirPath()` → state → render.

**Key insight from code review:** `get_app_dir_path` command in `commands/mod.rs` already returns `portable::app_data_dir(&app)` as a string — this is exactly the portable-aware base dir needed. No new backend command is needed. The CONTEXT.md suggestion of a "new `get_app_data_dir_display` command" is superseded by the discovery that `get_app_dir_path` already exists and does exactly this.

**Established pattern (from `AppDataDirectory.tsx`):**
```tsx
const [appDirPath, setAppDirPath] = useState<string>("");
const [loading, setLoading] = useState(true);

useEffect(() => {
  const load = async () => {
    const result = await commands.getAppDirPath();
    if (result.status === "ok") setAppDirPath(result.data);
    setLoading(false);
  };
  load();
}, []);

// In render:
<span className="font-mono text-xs select-text">{appDirPath}</span>
<span className="font-mono text-xs select-text">{appDirPath}/models</span>
<span className="font-mono text-xs select-text">{appDirPath}/settings_store.json</span>
```

The three hardcoded `%APPDATA%/handy*` spans each become a dynamic span. The `eslint-disable-next-line i18next/no-literal-string` directives are removed — a backend-provided value (not a string literal) does not trigger the ESLint rule.

**No new Tauri command needed.** No `bindings.ts` regeneration needed. No new `commands/debug.rs` file needed.

### Pattern 3: Icon Pipeline (ICON-01, ICON-03, ICON-04)

1. Pierre adds `appicon-desktop-1024.png` (1024×1024 RGBA, square corners, transparent background) to `dictus-brand` repo.
2. Run: `bun run tauri icon ../dictus-brand/<path>/appicon-desktop-1024.png`
3. This regenerates all files in `src-tauri/icons/` including `icon.icns`, `icon.ico`, all PNG sizes.
4. After generation, add `icons/256x256.png` and `icons/512x512.png` to `tauri.conf.json bundle.icon` list.

**Current `bundle.icon` array (must be extended):**
```json
"icon": [
  "icons/32x32.png",
  "icons/128x128.png",
  "icons/128x128@2x.png",
  "icons/icon.icns",
  "icons/icon.ico",
  "icons/256x256.png",
  "icons/512x512.png"
]
```

Note: After running `tauri icon`, verify `256x256.png` and `512x512.png` are actually generated. Based on current icon inventory, `128x128@2x.png` is 256×256 but named differently. Tauri's CLI output may vary; check actual generated filenames and match them in the config.

### Pattern 4: verify-sync.sh Move + Extension (BRAND-04, SYNC-06)

```bash
mkdir -p .github/scripts
git mv .planning/phases/05-upstream-sync/scripts/verify-sync.sh .github/scripts/verify-sync.sh
```

Then append new assertions to `.github/scripts/verify-sync.sh`, following the existing `check "<label>" "<shell expr>"` pattern. The `imagemagick` dependency check goes at the top alongside the `jq` check:

```bash
command -v identify >/dev/null 2>&1 || { echo "imagemagick required — brew install imagemagick"; exit 2; }
```

The SYNC-05d awk command must be preserved verbatim (it skips the acknowledgments block):
```bash
"! awk '/\"acknowledgments\"/{skip=1} /^\s*},$/{if(skip)skip=0} !skip' src/i18n/locales/en/translation.json | grep -q '\"Handy\"'"
```

**New assertions to append:**
```bash
check "BRAND-01a no handy- filename prefix in source (except handy_keys)" \
  "! grep -rn 'handy-' src-tauri/src/ | grep -v handy_keys"

check "BRAND-02a no legacy Handy Portable Mode string" \
  "! grep -q '\"Handy Portable Mode\"' src-tauri/src/portable.rs"

check "BRAND-03a no hardcoded handy path in DebugPaths" \
  "! grep -q '%APPDATA%/handy' src/components/settings/debug/DebugPaths.tsx"

check "ICON-02a ICO contains all required layer sizes" \
  "identify -format '%w\n' src-tauri/icons/icon.ico | sort -u | grep -qE '^(16|24|32|48|64|256)$' && [ \$(identify -format '%w\n' src-tauri/icons/icon.ico | sort -u | wc -l | tr -d ' ') -ge 6 ]"
```

**Script header update:** Change `"Phase 5 — Upstream Sync identity + build assertions"` echo to `"Phase 5+6 — Upstream Sync + Brand identity assertions"` (or similar) and update the final pass/fail message to match.

### Recommended Work Order

1. BRAND-01 + BRAND-02 Rust string replacements (no deps, safe to do first — `cargo test` validates immediately)
2. BRAND-03 DebugPaths.tsx rewrite (no new backend needed — uses existing `get_app_dir_path`)
3. SYNC-06 `git mv` + BRAND-04 assertions + UPSTREAM.md update (independent, no blocking dep)
4. ICON-04 (blocked on Pierre adding the asset to dictus-brand) → then ICON-01 + ICON-03 icon regeneration
5. Post-phase: run `/gsd:roadmap-update` to reflect SYNC-06 completion

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead |
|---|---|---|
| Platform icon generation | Manual PNG resizing, ICO construction | `bun run tauri icon <1024px-source.png>` |
| ICO layer verification | Custom binary parser | `identify -format '%w\n' icon.ico` (ImageMagick) |
| TypeScript bindings for new Tauri commands | Manual binding declarations | `tauri-specta` auto-generates on debug build |
| Portable-aware path resolution | Custom path logic in frontend | `get_app_dir_path` backend command (already exists) |

**Key insight:** A new backend command for DebugPaths is NOT needed — `get_app_dir_path` already returns the portable-aware base dir. The planner must NOT create a new `get_app_data_dir_display` command.

---

## Common Pitfalls

### Pitfall 1: Creating a New Backend Command When One Already Exists
**What goes wrong:** CONTEXT.md describes a "new `get_app_data_dir_display` command" but `get_app_dir_path` in `commands/mod.rs:25-30` already does exactly this (calls `portable::app_data_dir(&app)`, returns `String`).
**Why it happens:** CONTEXT.md was written before code review confirmed the existing command.
**How to avoid:** Use `commands.getAppDirPath()` in the DebugPaths rewrite. No new command, no `bindings.ts` regeneration step needed.
**Warning signs:** Any task that says "add new Tauri command for app data dir" — this is unnecessary.

### Pitfall 2: ICON-02 May Already Pass
**What goes wrong:** Implementing icon generation before checking whether ICON-02 is already satisfied, wasting effort.
**Current state:** `identify` on the current `icon.ico` shows layers at 16, 24, 32, 48, 64, 256. All six required sizes are present.
**How to avoid:** The ICON-02a verify-sync assertion will confirm this. If assertion passes before icon regeneration, ICON-02 is already done. The icon regeneration work (ICON-01/03/04) is still needed for the Linux transparent-corner fix and explicit bundle config.

### Pitfall 3: `256x256.png` May Not Be Generated by `tauri icon`
**What goes wrong:** Adding `icons/256x256.png` to `tauri.conf.json` before confirming `tauri icon` generates a file by that exact name.
**Why it happens:** Current icon directory has `128x128@2x.png` (which is 256×256) but no `256x256.png`. Tauri's `icon` command output filenames are determined by the CLI version.
**How to avoid:** Run `bun run tauri icon` against the new source PNG, then `ls src-tauri/icons/` to confirm which files were generated before updating `tauri.conf.json`.

### Pitfall 4: BRAND-01a grep catches `handy_keys` references
**What goes wrong:** `grep -rn 'handy-' src-tauri/src/` hits `handy_keys` usage if not excluded.
**Current state:** `handy_keys` is an external crate and legitimately appears in source (`lib.rs:411-412` has `shortcut::handy_keys::*` commands).
**How to avoid:** The `BRAND-01a` assertion already excludes this: `grep -rn 'handy-' src-tauri/src/ | grep -v handy_keys`. Note: `handy_keys` uses underscore not hyphen, so `handy-` grep may not even match it — but the exclusion is belt-and-suspenders.

### Pitfall 5: verify-sync.sh Move Breaks the Existing Phase 5 Checks
**What goes wrong:** Moving the script changes its path but not its content — the script still works. But if the shebang or relative paths inside the script depend on CWD, they could break.
**Current state:** Script uses relative paths like `src-tauri/tauri.conf.json`, `src/i18n/...` — these require running from the repo root. This is unchanged by the `git mv`.
**How to avoid:** Run `bash .github/scripts/verify-sync.sh` from repo root after the move and before committing.

### Pitfall 6: `eslint-disable-next-line` Removal Causes Lint Failure
**What goes wrong:** Removing the disable comments from `DebugPaths.tsx` before the literal strings are replaced with dynamic values causes an ESLint `i18next/no-literal-string` error.
**How to avoid:** The rewrite replaces hardcoded strings with `{appDirPath}`, `{appDirPath}/models`, `{appDirPath}/settings_store.json` before removing the disable directives. Since these are template expressions (not literals), the ESLint rule is satisfied.

### Pitfall 7: logo.png Has Transparent Corners (Confirmed Root Cause of ICON-01)
**What goes wrong:** Using the existing `logo.png` (1024×1024) as the `tauri icon` source for the new generation. Its corner pixel alpha is 0 — it already has transparent corners — wait, that means transparent corners... let me re-examine.
**Clarification:** `identify -format "%[fx:p{0,0}.a]"` returned `0` for `logo.png`. Alpha=0 means transparent. A transparent-corner 1024px PNG is actually correct for Linux (transparent bg, square canvas). The issue is whether the waveform glyph itself is the correct Dictus one or the legacy Handy one. The CONTEXT.md decision is to create a new source from `dictus-brand` to ensure provenance is correct, regardless of whether `logo.png` is technically usable.
**How to avoid:** Use the new `dictus-brand` source PNG as required by ICON-04. Do not reuse `logo.png` even if its corner alpha appears correct.

---

## Code Examples

### DebugPaths.tsx Rewrite Pattern
```tsx
// Source: AppDataDirectory.tsx (established pattern in this codebase)
import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { commands } from "@/bindings";
import { SettingContainer } from "../../ui/SettingContainer";

export const DebugPaths: React.FC<DebugPathsProps> = ({ descriptionMode = "inline", grouped = false }) => {
  const { t } = useTranslation();
  const [appDirPath, setAppDirPath] = useState<string>("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const load = async () => {
      const result = await commands.getAppDirPath();
      if (result.status === "ok") setAppDirPath(result.data);
      setLoading(false);
    };
    load();
  }, []);

  if (loading) return null; // or a skeleton

  return (
    <SettingContainer title="Debug Paths" description="..." descriptionMode={descriptionMode} grouped={grouped}>
      <div className="text-sm text-gray-600 space-y-2">
        <div>
          <span className="font-medium">{t("settings.debug.paths.appData")}</span>{" "}
          <span className="font-mono text-xs select-text">{appDirPath}</span>
        </div>
        <div>
          <span className="font-medium">{t("settings.debug.paths.models")}</span>{" "}
          <span className="font-mono text-xs select-text">{appDirPath}/models</span>
        </div>
        <div>
          <span className="font-medium">{t("settings.debug.paths.settings")}</span>{" "}
          <span className="font-mono text-xs select-text">{appDirPath}/settings_store.json</span>
        </div>
      </div>
    </SettingContainer>
  );
};
```

### verify-sync.sh dependency check pattern (from existing script)
```bash
# Existing pattern (line 18):
command -v jq >/dev/null 2>&1 || { echo "jq required — brew install jq"; exit 2; }

# New addition (same pattern):
command -v identify >/dev/null 2>&1 || { echo "imagemagick required — brew install imagemagick"; exit 2; }
```

### Tauri command registration pattern (for reference — no new command needed)
```rust
// Source: src-tauri/src/commands/mod.rs — existing get_app_dir_path
#[tauri::command]
#[specta::specta]
pub fn get_app_dir_path(app: AppHandle) -> Result<String, String> {
    let app_data_dir = crate::portable::app_data_dir(&app)
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    Ok(app_data_dir.to_string_lossy().to_string())
}
```

---

## State of the Art

| Surface | Current State | After Phase 6 |
|---|---|---|
| Recording filenames | `handy-{ts}.wav` | `dictus-{ts}.wav` |
| Portable marker string | `"Handy Portable Mode"` | `"Dictus Portable Mode"` |
| DebugPaths display | Hardcoded `%APPDATA%/handy` | Real runtime path via `getAppDirPath()` |
| verify-sync.sh location | `.planning/phases/05-upstream-sync/scripts/` | `.github/scripts/` |
| verify-sync.sh assertions | 11 (SYNC-05a through SYNC-05k) | 15 (+ BRAND-01a, 02a, 03a, ICON-02a) |
| `tauri.conf.json bundle.icon` | 5 entries (no 256×256 or 512×512) | 7 entries |
| Windows ICO layers | 16, 24, 32, 48, 64, 256 (already correct) | Verified by assertion |
| Linux icon source | Possibly iOS-rounded PNG | Square transparent-bg PNG from dictus-brand |

---

## Open Questions

1. **Does `tauri icon` generate `256x256.png` and `512x512.png` by those exact names?**
   - What we know: Current icon directory has `128x128@2x.png` (256×256) and `icon.png` (512×512) but no `256x256.png` or `512x512.png`.
   - What's unclear: Whether the Tauri 2.x `icon` CLI generates files named `256x256.png` and `512x512.png`, or uses different naming.
   - Recommendation: Run `bun run tauri icon` against any 1024px PNG in a test context, observe output filenames, then update `tauri.conf.json` to match actual names. Do not assume.

2. **Is dictus-brand repo accessible at `../dictus-brand/` relative to dictus-desktop?**
   - What we know: Phase 2 used `../dictus-brand/` as the reference path.
   - What's unclear: Whether Pierre has cloned dictus-brand alongside dictus-desktop in the current dev environment.
   - Recommendation: Planner should note that the icon task requires Pierre to (a) clone dictus-brand if not present, (b) add the square PNG, before the `tauri icon` step can execute.

3. **Should DebugPaths show a loading state or render nothing while fetching?**
   - What we know: AppDataDirectory.tsx uses an animated skeleton div while loading. DebugPaths is a debug panel — less critical.
   - Recommendation: Match AppDataDirectory.tsx pattern (animate-pulse skeleton) for consistency, or simply return null — Claude's discretion per CONTEXT.md.

---

## Validation Architecture

### Test Framework

| Property | Value |
|---|---|
| Framework | Rust `cargo test` (unit tests in-module) + ESLint (`bun run lint`) |
| Config file | `src-tauri/Cargo.toml` (Rust); `.eslintrc` / project root (ESLint) |
| Quick run command | `cargo test --manifest-path src-tauri/Cargo.toml 2>&1 \| grep -E "test result\|FAILED\|error"` |
| Full suite command | `bun run lint && cargo test --manifest-path src-tauri/Cargo.toml` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| BRAND-01 | New recordings use `dictus-` prefix | unit (Rust) | `cargo test -p dictus-desktop-lib --test '*' 2>&1` | ✅ `history.rs` tests |
| BRAND-02 | Portable marker uses `"Dictus Portable Mode"` | unit (Rust) | `cargo test -p dictus-desktop-lib portable 2>&1` | ✅ `portable.rs` tests (6 cases) |
| BRAND-03 | DebugPaths renders backend path, no hardcoded string | lint | `bun run lint src/components/settings/debug/DebugPaths.tsx` | ✅ file exists |
| BRAND-04 | verify-sync.sh assertions pass | shell | `bash .github/scripts/verify-sync.sh` | ❌ Wave 0 (create `.github/scripts/`) |
| ICON-01 | Linux icon has transparent square corners | manual | Visual inspection of deb/AppImage on Linux; or pixel check via `identify` | N/A |
| ICON-02 | ICO has all 6 required layer sizes | shell | `identify -format '%w\n' src-tauri/icons/icon.ico \| sort -u` | ✅ already passing |
| ICON-03 | bundle.icon includes 256×256 and 512×512 | shell | `grep -q '256x256' src-tauri/tauri.conf.json` | ✅ file exists |
| ICON-04 | Square 1024px PNG exists in dictus-brand | manual | Verify `../dictus-brand/<path>/appicon-desktop-1024.png` exists | ❌ requires Pierre action |
| SYNC-06 | verify-sync.sh at `.github/scripts/` | shell | `ls .github/scripts/verify-sync.sh` | ❌ Wave 0 (move needed) |

### Sampling Rate

- **Per task commit:** `cargo test --manifest-path src-tauri/Cargo.toml -q 2>&1 | tail -5`
- **Per wave merge:** `bun run lint && cargo test --manifest-path src-tauri/Cargo.toml && bash .github/scripts/verify-sync.sh`
- **Phase gate:** Full suite green + `verify-sync.sh` passes all 15 assertions before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `.github/scripts/` directory — created by `git mv` of verify-sync.sh (SYNC-06 task)
- [ ] `src-tauri/icons/256x256.png` and `src-tauri/icons/512x512.png` — generated by `tauri icon` run (ICON-01/03 task)
- [ ] `../dictus-brand/<path>/appicon-desktop-1024.png` — requires Pierre to add to dictus-brand repo (external dependency, blocks icon tasks)

---

## Sources

### Primary (HIGH confidence)

- Direct code inspection: `src-tauri/src/actions.rs:538`, `src-tauri/src/portable.rs`, `src/components/settings/debug/DebugPaths.tsx`, `src-tauri/src/commands/mod.rs`, `src-tauri/tauri.conf.json`, `.planning/phases/05-upstream-sync/scripts/verify-sync.sh`
- `identify` tool output on `src-tauri/icons/icon.ico` — confirmed all 6 ICO layers present
- `identify` output on all `src-tauri/icons/*.png` — confirmed size inventory
- `src/components/settings/AppDataDirectory.tsx` — confirmed established pattern for backend path fetching
- `src/bindings.ts:440-442` — confirmed `getAppDirPath` already in TypeScript bindings

### Secondary (MEDIUM confidence)

- Tauri 2.x `bundle.icon` configuration: [https://v2.tauri.app/reference/cli/#icon](https://v2.tauri.app/reference/cli/#icon) — referenced in CONTEXT.md as canonical ref
- ImageMagick `identify`: [https://imagemagick.org/script/identify.php](https://imagemagick.org/script/identify.php) — referenced in CONTEXT.md

---

## Metadata

**Confidence breakdown:**
- Rust string replacements (BRAND-01, BRAND-02): HIGH — exact line numbers confirmed by direct code read
- DebugPaths rewrite (BRAND-03): HIGH — pattern confirmed from AppDataDirectory.tsx; `get_app_dir_path` confirmed in bindings; NO new backend command needed
- verify-sync.sh move + assertions (BRAND-04, SYNC-06): HIGH — existing script structure and patterns confirmed; UPSTREAM.md reference lines confirmed
- Icon pipeline (ICON-01, 02, 03, 04): MEDIUM — ICO already has correct layers (HIGH); `tauri icon` CLI output filenames uncertain (MEDIUM); dictus-brand asset is external dependency (blocks)
- `tauri icon` output filenames: LOW — not verified against running Tauri 2.x CLI; planner must account for this uncertainty

**Research date:** 2026-04-15
**Valid until:** 2026-05-15 (stable domain — Tauri 2.x, Rust, bash scripting)
