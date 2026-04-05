# Phase 1: Bundle Identity - Research

**Researched:** 2026-04-05
**Domain:** Tauri 2.x bundle configuration, Cargo.toml package metadata
**Confidence:** HIGH

## Summary

Phase 1 is a pure configuration edit with no new dependencies and no code changes. All five requirements (BNDL-01 through BNDL-05) resolve to targeted field edits across two files: `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml`. The current state of both files is fully known — they contain Handy/cjpais identity at every relevant field.

The updater strategy (set `createUpdaterArtifacts: false`, leave `plugins.updater: {}`) is validated: when artifact generation is disabled, Tauri does not enforce the presence of a valid `pubkey` or `endpoints` at build time. The `tauri-plugin-updater` Cargo dependency may stay — it imposes no build-time requirement, and removing it would block easy re-enable later.

`productName` in `tauri.conf.json` controls the `.app` bundle name, CFBundleName (dock/Spotlight), tray menu label, and window title in production (release) builds. During `tauri dev`, the binary name from `Cargo.toml` `name` field is used instead — this is a known Tauri 2 dev/build difference. Since the binary name (`handy`) is deferred to V2 (TECH-03), the dock in dev mode will show "handy" after this phase; release builds will correctly show "Dictus".

**Primary recommendation:** Edit the two config files as specified in CONTEXT.md decisions. No library research, no dependency changes, no code refactoring needed.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **Version number:** Reset to 0.1.0 in both `tauri.conf.json` and `Cargo.toml`
- **Updater handling:** Set `createUpdaterArtifacts: false`; remove `endpoints` array and `pubkey` from updater plugin config (leave empty object `{}`); keep `tauri-plugin-updater` dependency in Cargo.toml
- **Windows signing:** Remove `signCommand` field entirely from `bundle.windows` config — no placeholder
- **Cargo.toml authorship:** `authors = ["Dictus", "cjpais"]`; update `description` to describe Dictus Desktop; do NOT touch `name`, `default-run`, or `lib.name`

### Claude's Discretion

- Exact wording of `Cargo.toml` `description` field
- Whether to add a `repository` field pointing to the Dictus repo
- Order of fields in `tauri.conf.json` after edits

### Deferred Ideas (OUT OF SCOPE)

- `name = "handy"` and `default-run = "handy"` in Cargo.toml (TECH-03, V2)
- `lib.name = "handy_app_lib"` in Cargo.toml (TECH-01, V2)
- `[patch.crates-io]` references to `cjpais/tauri.git` (TECH-04, V2)
- `handy-keys` dependency (TECH-01, V2)
- Any Rust source code renaming
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| BNDL-01 | `productName` changed to "Dictus" in `tauri.conf.json` | Confirmed: `productName` is the top-level field controlling app bundle name, dock display (release builds), tray label, and window title in Tauri 2.x |
| BNDL-02 | `identifier` changed to `"com.dictus.desktop"` | Confirmed: `identifier` field controls OS-level bundle ID and determines app data directory path; clean-slate install decided (no migration needed) |
| BNDL-03 | Cargo.toml metadata updated (version, description, authors) — NOT name/default-run/lib.name | Confirmed: `version`, `description`, `authors` are safe to edit; `name`, `default-run`, `lib.name` deliberately excluded per V2 deferral |
| BNDL-04 | Auto-updater endpoint disabled — no reference to upstream Handy releases | Confirmed: `createUpdaterArtifacts: false` + empty `plugins.updater: {}` disables artifact generation without requiring valid pubkey/endpoints; plugin dependency stays |
| BNDL-05 | Upstream Windows code signing reference removed | Confirmed: `bundle.windows.signCommand` field removed entirely; clean removal with no placeholder |
</phase_requirements>

---

## Standard Stack

### Core (files being edited)
| File | Current Identity | Target Identity |
|------|-----------------|----------------|
| `src-tauri/tauri.conf.json` | `productName: "Handy"`, `identifier: "com.pais.handy"`, `version: "0.8.2"`, `createUpdaterArtifacts: true`, `plugins.updater` with cjpais endpoint, `bundle.windows.signCommand` pointing to Azure/cjpais | `productName: "Dictus"`, `identifier: "com.dictus.desktop"`, `version: "0.1.0"`, `createUpdaterArtifacts: false`, `plugins.updater: {}`, no `signCommand` |
| `src-tauri/Cargo.toml` | `version: "0.8.2"`, `description: "Handy"`, `authors: ["cjpais"]` | `version: "0.1.0"`, `description: <Dictus wording>`, `authors: ["Dictus", "cjpais"]` |

### No new dependencies
This phase installs nothing. All changes are field edits.

## Architecture Patterns

### Tauri 2.x config structure — field locations

The current `tauri.conf.json` uses this structure (verified from file):

```
{
  "$schema": "https://schema.tauri.app/config/2",   ← keep as-is
  "productName": "...",                              ← BNDL-01
  "version": "...",                                 ← BNDL-03 (version)
  "identifier": "...",                              ← BNDL-02
  "build": { ... },                                 ← no change
  "app": { ... },                                   ← no change
  "bundle": {
    "createUpdaterArtifacts": true,                 ← BNDL-04 (set false)
    "windows": {
      "signCommand": "...",                         ← BNDL-05 (remove field)
      "nsis": { ... }                               ← no change
    },
    ...                                             ← no change
  },
  "plugins": {
    "updater": {
      "pubkey": "...",                              ← BNDL-04 (remove)
      "endpoints": [ ... ]                          ← BNDL-04 (remove)
    }
  }
}
```

After edits, `plugins.updater` becomes `{}`.

### Cargo.toml package section — field locations

```toml
[package]
name = "handy"          ← DO NOT TOUCH (V2/TECH-03)
version = "0.8.2"       ← change to "0.1.0"
description = "Handy"   ← change to Dictus wording
authors = ["cjpais"]    ← change to ["Dictus", "cjpais"]
edition = "2021"        ← no change
license = "MIT"         ← no change
default-run = "handy"   ← DO NOT TOUCH (V2/TECH-03)

[lib]
name = "handy_app_lib"  ← DO NOT TOUCH (V2/TECH-01)
```

### Anti-Patterns to Avoid

- **Touching deferred fields:** `name`, `default-run`, `lib.name` in Cargo.toml, and `[patch.crates-io]` section are V2 scope — do not edit them.
- **Adding a placeholder `signCommand`:** Remove the field entirely; no empty string, no TODO comment.
- **Removing `tauri-plugin-updater` from Cargo.toml:** Keep the dependency; it imposes no build-time requirement when `createUpdaterArtifacts: false`.
- **Leaving the old `pubkey` or `endpoints` as empty strings:** Remove the keys entirely; `plugins.updater: {}` is the target state.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead |
|---------|-------------|-------------|
| Verifying bundle identifier change took effect | Custom build script | `tauri build` output — the .app bundle path will contain the new identifier |
| Checking OS data path changed | Runtime logging | Use `tauri_plugin_fs` `app_data_dir()` in a test run, or inspect macOS `~/Library/Application Support/` |

## Common Pitfalls

### Pitfall 1: Dev mode still shows "handy" in dock
**What goes wrong:** After changing `productName`, running `tauri dev` still shows "handy" in the macOS dock title and process name.
**Why it happens:** `tauri dev` launches the raw binary directly (binary name = Cargo.toml `name` = "handy"). The `.app` bundle Info.plist that carries `productName` is only assembled during `tauri build`.
**How to avoid:** This is expected and acceptable for Phase 1. The dock name in release builds will correctly show "Dictus". Document this in the plan so the implementer doesn't chase a non-bug.
**Warning signs:** Only a concern if reviewer tests exclusively in `tauri dev`.

### Pitfall 2: Build fails after removing updater pubkey
**What goes wrong:** `tauri build` errors with a complaint about missing updater configuration.
**Why it happens:** If `createUpdaterArtifacts` is still `true` when `pubkey`/`endpoints` are removed, the build will fail — the updater build step requires signing keys.
**How to avoid:** Set `createUpdaterArtifacts: false` in the same edit pass as removing `pubkey`/`endpoints`. Verify the boolean is `false`, not omitted or `"false"` (string).
**Warning signs:** Build error mentioning "updater" or "pubkey" during `tauri build`.

### Pitfall 3: App data directory migration confusion
**What goes wrong:** Existing Handy installation data (at `~/Library/Application Support/com.pais.handy`) is not accessible to the renamed app.
**Why it happens:** Identifier change means a new data path.
**How to avoid:** This is an accepted decision (clean-slate install, no migration needed). No existing Dictus user base. Document it and proceed.

### Pitfall 4: Windows NSIS template references old name
**What goes wrong:** The NSIS installer template at `nsis/installer.nsi` might hardcode "Handy" strings.
**Why it happens:** Custom NSIS templates can embed product name literals.
**How to avoid:** After the config edit, grep the NSIS template for "Handy" to check. If found, those references are outside BNDL-01 scope but should be noted for Phase 2/3 cleanup (DOCS-02).
**Warning signs:** Installer shows "Handy" in dialog titles despite config change.

## Code Examples

### Target state: tauri.conf.json (changed fields only)

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Dictus",
  "version": "0.1.0",
  "identifier": "com.dictus.desktop",
  ...
  "bundle": {
    "active": true,
    "createUpdaterArtifacts": false,
    ...
    "windows": {
      "nsis": {
        "template": "nsis/installer.nsi"
      }
    }
  },
  "plugins": {
    "updater": {}
  }
}
```

Note: `bundle.windows.signCommand` is removed entirely. The `macOS.signingIdentity: "-"` (ad-hoc signing for local dev) stays unchanged — it is not Handy-specific.

### Target state: Cargo.toml [package] section

```toml
[package]
name = "handy"
version = "0.1.0"
description = "Open-source speech-to-text desktop app — Dictus Desktop"
authors = ["Dictus", "cjpais"]
edition = "2021"
license = "MIT"
default-run = "handy"
```

The `description` wording is Claude's discretion per CONTEXT.md. The example above is a reasonable default; planner may adjust.

### Verification commands

```bash
# Check no Handy identity in modified fields
grep -n "Handy\|cjpais\|com.pais\|0\.8\.2\|signCommand\|eus.codesigning\|github.com/cjpais" src-tauri/tauri.conf.json src-tauri/Cargo.toml

# Confirm build succeeds (no updater error)
bun run tauri build 2>&1 | grep -i "error\|warning.*updater"
```

## State of the Art

| Old Approach | Current Approach | Impact |
|--------------|------------------|--------|
| Tauri 1.x `tauri.conf.json` had updater under `tauri.updater` | Tauri 2.x uses `plugins.updater` and `bundle.createUpdaterArtifacts` | These files use Tauri 2.x schema — correct path confirmed |
| `createUpdaterArtifacts` defaulted to `true` in early Tauri 2 migration output | Default is `false`; existing config has it explicitly `true` | Set to `false` is safe and idiomatic |

## Open Questions

1. **`repository` field in Cargo.toml**
   - What we know: CONTEXT.md leaves this to Claude's discretion
   - What's unclear: Whether the Dictus repo URL is known/public yet
   - Recommendation: Planner should add `repository = "https://github.com/dictus-desktop/dictus-desktop"` as a discretionary task, but mark it as optional if the repo URL isn't confirmed

2. **NSIS template Handy references**
   - What we know: `nsis/installer.nsi` exists (referenced in config); content unknown
   - What's unclear: Whether it hardcodes "Handy" — this is outside BNDL-01-05 scope
   - Recommendation: Add a grep check as a verification step; log findings for Phase 3 (DOCS-02)

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | None detected — Rust: no test runner config; Frontend: no jest/vitest config found |
| Config file | None |
| Quick run command | `grep -rn "Handy\|cjpais\|com.pais\|0\.8\.2" src-tauri/tauri.conf.json src-tauri/Cargo.toml` |
| Full suite command | `bun run format:check && bun run lint` |

### Phase Requirements to Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| BNDL-01 | `productName` equals "Dictus" | smoke (grep) | `grep '"productName": "Dictus"' src-tauri/tauri.conf.json` | ✅ |
| BNDL-02 | `identifier` equals "com.dictus.desktop" | smoke (grep) | `grep '"identifier": "com.dictus.desktop"' src-tauri/tauri.conf.json` | ✅ |
| BNDL-03 | Cargo version 0.1.0, authors includes "Dictus", description updated | smoke (grep) | `grep -E 'version = "0\.1\.0"' src-tauri/Cargo.toml && grep 'Dictus' src-tauri/Cargo.toml` | ✅ |
| BNDL-04 | No updater endpoint pointing to cjpais/Handy; `createUpdaterArtifacts: false` | smoke (grep) | `grep -c "cjpais\|github.com/cjpais" src-tauri/tauri.conf.json` (expect 0) | ✅ |
| BNDL-05 | No `signCommand` in bundle.windows | smoke (grep) | `grep -c "signCommand" src-tauri/tauri.conf.json` (expect 0) | ✅ |

All checks are grep-based (no test framework needed). They are runnable in < 5 seconds.

### Sampling Rate

- **Per task commit:** Run the smoke grep for the specific field edited
- **Per wave merge:** Run all 5 greps + `bun run format:check`
- **Phase gate:** All 5 greps return expected output + `bun run tauri build` succeeds (no updater error)

### Wave 0 Gaps

None — existing files are sufficient. No test infrastructure setup required.

## Sources

### Primary (HIGH confidence)
- `src-tauri/tauri.conf.json` — Current field values directly inspected
- `src-tauri/Cargo.toml` — Current field values directly inspected
- [Tauri 2 Configuration Reference](https://v2.tauri.app/reference/config/) — `createUpdaterArtifacts`, `identifier`, `productName` field definitions
- [Tauri 2 Updater Plugin](https://v2.tauri.app/plugin/updater/) — `pubkey`/`endpoints` requirement; confirmed that these are only enforced when artifact generation is active

### Secondary (MEDIUM confidence)
- [Tauri Issue #13874](https://github.com/tauri-apps/tauri/issues/13874) — Confirmed that `productName` controls release build dock/menu name; `tauri dev` uses binary name (known limitation, closed as not planned)
- [Tauri Issue #10508](https://github.com/tauri-apps/tauri/issues/10508) — Confirmed that `createUpdaterArtifacts` config can be removed or set to `false` without build errors when updater is unused

### Tertiary (LOW confidence)
- None — all claims verified with official sources or direct file inspection

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — both files directly inspected; exact field names confirmed
- Architecture: HIGH — Tauri 2.x config structure verified from official schema reference
- Pitfalls: HIGH (dev/build dock difference confirmed from GitHub issue); MEDIUM (NSIS template — not yet inspected)

**Research date:** 2026-04-05
**Valid until:** 2026-07-05 (Tauri config schema is stable; 90-day validity reasonable)
