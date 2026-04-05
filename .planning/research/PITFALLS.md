# Pitfalls Research

**Domain:** Desktop app rebranding — Tauri 2.x / Rust / React fork (Handy → Dictus Desktop)
**Researched:** 2026-04-05
**Confidence:** HIGH — based on direct codebase analysis, not hypothetical scenarios

---

## Critical Pitfalls

### Pitfall 1: Updater Endpoint Left Pointing at Upstream

**What goes wrong:**
The Tauri updater is configured in `tauri.conf.json` to pull from `https://github.com/cjpais/Handy/releases/latest/download/latest.json`. If this endpoint is not updated before the first Dictus release, users will either receive upstream Handy updates (installing a rebranded binary over their Dictus install) or the updater silently fails when the upstream version does not match the fork's version number.

**Why it happens:**
The updater config, the pubkey, and the endpoint are buried in `tauri.conf.json` and easy to overlook during a UI-focused rebrand. Developers visually rebrand the app but do not consider the background update mechanism.

**How to avoid:**
- Change `plugins.updater.endpoints` in `tauri.conf.json` to a Dictus-controlled URL before any release build.
- Generate a new signing keypair (`tauri signer generate`) and replace the `pubkey` field. Keep the private key outside the repo.
- Set up a Dictus releases endpoint (GitHub Releases on the dictus-desktop repo, or a self-hosted `latest.json`).
- Disable updater entirely (`createUpdaterArtifacts: false`) until the endpoint is ready, rather than leaving upstream endpoint active.

**Warning signs:**
- `tauri.conf.json` still references `cjpais/Handy` in the `endpoints` array.
- `pubkey` value is the original Handy key (starts with `dW50cnVzdGVk...`).
- `bundle.windows.signCommand` still contains `cjpais-dev` Azure signing identity.

**Phase to address:**
Identity and bundle rebrand phase (first phase). Must be resolved before any test build is distributed.

---

### Pitfall 2: Bundle Identifier Not Changed — Conflicting App Data on macOS/Windows

**What goes wrong:**
`tauri.conf.json` currently uses `identifier: "com.pais.handy"`. Tauri uses this identifier to determine where app data is stored (macOS: `~/Library/Application Support/com.pais.handy/`, Windows: `%APPDATA%/com.pais.handy/`). If the identifier is changed after a user has already installed the app, all their settings, history, and downloaded models will appear to be missing on next launch. The app will start as fresh with a new identifier directory. The old data remains but is no longer found.

**Why it happens:**
The identifier looks like a pure config detail. Developers focus on visible branding and assume this can be changed cleanly at any time.

**How to avoid:**
- Change the identifier to `com.dictus.desktop` in `tauri.conf.json` as part of the very first rebrand commit, before any Dictus builds are distributed.
- Since no Dictus users exist yet (this is a fresh fork), there is no existing data to migrate. Change it once and be done.
- Do NOT change the identifier after distributing any Dictus build without implementing a data migration.

**Warning signs:**
- `identifier` still reads `com.pais.handy` after rebrand work is considered complete.
- macOS app data path contains `handy` instead of `dictus`.

**Phase to address:**
Identity and bundle rebrand phase (first phase). Change once, never revisit.

---

### Pitfall 3: Model Download URLs Hard-Wired to `blob.handy.computer`

**What goes wrong:**
All 15+ model download URLs in `src-tauri/src/managers/model.rs` (lines 133–600) point to `https://blob.handy.computer/`. This is a CDN controlled by the upstream Handy project. If that CDN goes offline, changes access policies, or the upstream project stops hosting files, all model downloads in Dictus Desktop break silently — the download fails and users cannot get a working model. This is a hard runtime dependency on upstream infrastructure.

**Why it happens:**
CDN URLs are implementation details that look stable. Developers rebrand the UI without auditing the backend data dependencies.

**How to avoid:**
- Decide immediately: will Dictus host its own models or continue relying on `blob.handy.computer`?
- For V1, the pragmatic path is to keep the URLs but document the dependency explicitly. Add a fallback error message that names the dependency so debugging is straightforward.
- For V2+, consider mirroring models to a Dictus-controlled CDN or using Hugging Face as a neutral host.
- The VAD model (`silero_vad_v4.onnx`) is also fetched from this CDN in all dev setup docs (`CLAUDE.md`, `CRUSH.md`, `CONTRIBUTING.md`, `AGENTS.md`). Update all docs to reflect the chosen source.

**Warning signs:**
- `model.rs` still contains only `blob.handy.computer` URLs after V1 ships.
- Dev setup docs still instruct contributors to `curl` from `blob.handy.computer` without a note about the dependency.

**Phase to address:**
Document the dependency in V1. Resolve the CDN dependency ownership in V2.

---

### Pitfall 4: i18n Translation Files Contain "Handy" in Every Language

**What goes wrong:**
The English `translation.json` contains "Handy" as a literal string in at least 10 keys (onboarding descriptions, tray descriptions, about page). All other language files (20+ locales including tr, pt, vi, ru, zh, etc.) have their own translated forms of "Handy" embedded. A rebrand that only touches the English file leaves every non-English user seeing "Handy" in the UI.

**Why it happens:**
Developers run a visible string search in the UI, find the strings to change, edit `en/translation.json`, and consider the job done. The 20 other locale files are not checked.

**How to avoid:**
- Run `grep -rl "Handy" src/i18n/locales/` before marking any rebrand phase done — the search confirms 20+ locale files are affected.
- Treat translation key *values* (not keys) as content to audit. Use a script or grep to generate an exhaustive list across all locales.
- For V1, change the values in all locale files to "Dictus" (the name is a proper noun and does not require localization — keep existing translations for surrounding text but replace only the product name).
- Do not rely on visual review of the running app to catch this — non-English locales are not typically tested during development.

**Warning signs:**
- `grep -r '"Handy"' src/i18n/locales/` still returns results after rebrand.
- The onboarding screen shown in a non-English locale still reads "Handy".

**Phase to address:**
UI rebrand phase. Must cover all 20+ locale files, not just English.

---

### Pitfall 5: Patched Tauri Runtime Tied to `handy-2.10.2` Branch

**What goes wrong:**
`Cargo.toml` patches three core Tauri crates (`tauri-runtime`, `tauri-runtime-wry`, `tauri-utils`) with a custom fork at `https://github.com/cjpais/tauri.git` branch `handy-2.10.2`. This is the upstream maintainer's personal fork. If Tauri releases a security patch or breaking change in `2.10.x`, this fork branch is unlikely to be updated. Dictus is locked to a modified Tauri version on infrastructure it does not control.

**Why it happens:**
The patch was added to support custom behavior (likely the overlay or macOS private API features). Developers inherit the patch without evaluating whether the dependency risk is acceptable long-term.

**How to avoid:**
- Audit what the patch actually changes: `git diff tauri-v2.10.1..handy-2.10.2` on that repo.
- Determine whether the patch is still needed or has been upstreamed to official Tauri.
- If needed, fork the branch to a Dictus-controlled repository (`github.com/dictus-app/tauri` or similar) so Dictus controls its Tauri dependency.
- Document the patch clearly in `CONTRIBUTING.md` so future maintainers understand why the override exists.

**Warning signs:**
- `[patch.crates-io]` in `Cargo.toml` still references `cjpais/tauri.git` after V1.
- No documentation explaining why the patch exists.

**Phase to address:**
Technical dependency audit phase (or early in V1 infrastructure work). Does not block visual rebrand but blocks long-term stability.

---

### Pitfall 6: `handy-keys` Crate as a Runtime Dependency

**What goes wrong:**
The crate `handy-keys = "0.2.4"` is a published crate authored by the upstream maintainer (`cjpais`). It is used as the preferred keyboard implementation for recording shortcuts. If the crate is yanked, abandoned, or not updated for a new platform/Rust edition, Dictus's shortcut implementation degrades silently to the fallback Tauri implementation (which the code already handles, but the fallback is persisted to settings without warning the user).

**Why it happens:**
Third-party crates with names tied to upstream project branding are easy to miss during rebrand audits because they are in `Cargo.toml`, not in the visible UI.

**How to avoid:**
- Accept the dependency for V1 — the fallback path is already implemented and tested.
- Track the crate's health on crates.io. If `handy-keys` is yanked or stops compiling, the Tauri fallback already works.
- Document that `handy-keys` is an upstream crate and Dictus should evaluate publishing its own fork if abandonment becomes a risk.

**Warning signs:**
- `handy-keys` is marked deprecated or yanked on crates.io.
- The shortcut implementation selector UI shows "Tauri" as the only option (fallback has engaged).

**Phase to address:**
Track in V2 risk backlog. Not a V1 blocker given the existing fallback.

---

### Pitfall 7: Windows NSIS Installer Contains Upstream Code Signing Identity

**What goes wrong:**
`tauri.conf.json` contains `signCommand: "trusted-signing-cli -e https://eus.codesigning.azure.net/ -a CJ-Signing -c cjpais-dev -d Handy %1"`. This references a specific Azure code signing account (`CJ-Signing`, `cjpais-dev`) that belongs to the upstream maintainer. Any Windows build that runs this command will attempt to sign with credentials it does not have access to, causing the build to fail — or worse, if credentials are leaked via CI secrets inheritance, it could sign under the wrong identity.

**Why it happens:**
Windows code signing config is buried in the JSON bundle config and rarely exercised during development (developers build unsigned on their machines). It becomes visible only when the CI release pipeline runs.

**How to avoid:**
- Replace the `signCommand` with Dictus's own signing identity, or remove it entirely if Windows code signing is not a V1 requirement.
- Audit CI pipeline secrets to ensure no `cjpais` signing credentials are inherited.
- If unsigned builds are acceptable for V1, set `signCommand: ""` or remove the `windows.signCommand` key entirely.

**Warning signs:**
- `signCommand` still contains `CJ-Signing` or `cjpais-dev`.
- CI release pipeline fails with a signing error on the Windows target.

**Phase to address:**
Identity and bundle rebrand phase. Fix before any CI-driven Windows release build.

---

### Pitfall 8: Portable Mode Marker File Contains "Handy Portable Mode" String

**What goes wrong:**
`src-tauri/src/portable.rs` creates and reads a marker file using the string `"Handy Portable Mode"` as both a write value and a validation check. The `is_valid_portable_marker()` function checks that the file content starts with `"Handy Portable Mode"`. If this string is changed to `"Dictus Portable Mode"` without migrating existing marker files, users with existing portable installations will find that their portable mode is no longer detected — the app falls back to system app data, and their data appears to be missing.

**Why it happens:**
The marker string is a magic value embedded in a Rust source file. It is not visible in the UI, does not surface in translation files, and is missed by grep patterns that only search for "Handy" in visible strings.

**How to avoid:**
- When changing the magic string, preserve backward compatibility by accepting both `"Handy Portable Mode"` (legacy) and `"Dictus Portable Mode"` (new) in `is_valid_portable_marker()`.
- The existing `portable.rs` already has a legacy migration path (from empty marker to magic string) — extend this pattern.
- New marker files written post-rebrand should use the new string. Old files should be upgraded on first read.

**Warning signs:**
- `portable.rs` contains `"Handy Portable Mode"` as a hard-coded string after rebrand.
- A portable mode test (`handy_test_valid` and similar) still uses the old string.

**Phase to address:**
Technical rename phase (second pass). Low urgency for V1 since portable mode is a Windows feature affecting a subset of users, but must not break existing portable installs.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Keep `blob.handy.computer` for model downloads | Zero V1 effort | CDN dependency on upstream infrastructure the fork does not control | V1 only, with documented risk |
| Keep `handy-keys` crate without forking | Works today with fallback path | Potential break if upstream yanks crate; name inconsistency | V1 acceptable; review in V2 |
| Skip internal Rust symbol renames (`handy_app_lib`, `startHandyKeysRecording`) | Saves compile time | Developer confusion when internal names contradict external branding | Acceptable in V1 if a V2 ticket exists |
| Disable updater before Dictus endpoint is ready | Ships faster | Users have no update path, no communication about new releases | Never ship an enabled updater pointing to upstream |
| Remove Windows `signCommand` rather than fixing it | Unblocks builds | Unsigned Windows builds trigger security warnings on install | Acceptable for V1 if targeted at technical early adopters |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Tauri updater | Change `productName` but not `endpoints` or `pubkey` | All three must change together: productName, endpoints, pubkey |
| macOS entitlements | Forget to update `signingIdentity` in `macOS.signingIdentity` | Currently set to `"-"` (ad-hoc) which is fine for local builds; only matters if distribution signing is added |
| tauri-specta bindings | Rename Rust commands without regenerating `src/bindings.ts` | `bindings.ts` is auto-generated — re-run the specta export after any Rust command rename |
| i18next translations | Rebrand English source, forget to run translation validation | All 20+ locale files must be searched independently; ESLint i18next rule catches hardcoded strings in JSX but not wrong values in JSON files |
| NSIS installer template | Update `tauri.conf.json` but not the custom NSIS template in `nsis/installer.nsi` | The custom template uses Tauri template variables (`{{product_name}}` etc.) so it should inherit correctly, but the Windows `signCommand` is a direct string |

---

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| SHA256 model verification on UI thread | UI freezes for 5-30 seconds after model download | Move to background thread (already flagged in CONCERNS.md) | Every time a large model (1GB+) is downloaded |
| Linux clipboard tool availability checks | Slow paste on Linux; multiple process spawns per transcription | Cache `which` results on first call via `OnceLock` (already flagged in CONCERNS.md) | Every transcription on Linux |
| Model directory scan at startup | Startup delay with many custom models | Load custom models lazily; cache discovery results | Users with 10+ custom models |

These are pre-existing issues (not introduced by rebrand) but worth noting since any rebrand phase that touches model management could accidentally worsen them.

---

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Inheriting upstream CI secrets | CI build could use `cjpais` Azure signing credentials for Dictus releases | Audit all repo secrets; remove any inherited from upstream before enabling CI |
| Keeping upstream updater `pubkey` | Attacker with upstream private key could push updates to Dictus installs | Generate new keypair immediately; treat this as a security boundary, not a config detail |
| Custom model SHA256 bypass | Users can load unverified arbitrary models | Existing behavior — acceptable, but add UI warning that custom models are unverified |

---

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Updater still checks `cjpais/Handy` releases | Users see "Handy v0.9.0 available" after installing Dictus | Disable updater before V1 ships; enable when Dictus endpoint is ready |
| About page links to `handy.computer` and `github.com/cjpais/Handy` | Clicking "Source Code" or "Donate" goes to upstream project | Update all URLs in `AboutSettings.tsx` and `UpdateChecker.tsx` to Dictus equivalents |
| Onboarding shows `HandyTextLogo` SVG | First-run experience shows competitor branding | Replace SVG with Dictus logo before any user-facing V1 build |
| Sidebar icon is `HandyHand` component | Every session the user sees upstream branding in navigation | Replace icon component (it is a single SVG component, low effort) |
| Settings section names unchanged | Users from Dictus iOS expect "Dictation" naming; find "General" instead | Rename settings sections as planned: General → Dictation, Postprocessing → Smart Modes |

---

## "Looks Done But Isn't" Checklist

- [ ] **Visual rebrand:** Check the app with a non-English locale active — 20+ locale files contain "Handy" in translation values, not just `en/translation.json`.
- [ ] **Bundle identity:** Verify `~Library/Application Support/` on macOS shows `com.dictus.desktop` (or equivalent), not `com.pais.handy`.
- [ ] **Updater:** Confirm the updater endpoint in `tauri.conf.json` does not point to `cjpais/Handy`. Disable updater until a Dictus endpoint exists.
- [ ] **Signing identity:** Windows `signCommand` in `tauri.conf.json` must not reference `cjpais-dev` or `CJ-Signing`.
- [ ] **Rust crate name:** `Cargo.toml` `[package] name` and `default-run` still say `handy`. Binary will be named `handy` until this changes.
- [ ] **Tauri runtime patch:** `[patch.crates-io]` still references `cjpais/tauri.git`. Either accept and document, or fork to Dictus-controlled repo.
- [ ] **Model CDN:** `blob.handy.computer` URLs in `model.rs` are present and operational — decide whether to keep (acceptable for V1) or mirror.
- [ ] **Portable marker:** `portable.rs` magic string `"Handy Portable Mode"` — if changing, implement backward compatibility before breaking existing installs.
- [ ] **Bindings regenerated:** After any Rust command rename, `src/bindings.ts` must be regenerated by running the app in debug mode (specta auto-generates on debug build).
- [ ] **GitHub Actions workflows:** `build.yml` asset prefix is `handy`, `release.yml` asset prefix is `handy` — release artifacts will be named `handy-*.dmg` etc. until changed.

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Updater pointing to upstream | LOW (config change) | Update `tauri.conf.json` endpoints + pubkey; rebuild and redistribute |
| Bundle ID changed after distribution | HIGH (data migration required) | Write a migration shim that copies data from old path to new path on first launch |
| i18n missed in non-English locales | LOW (JSON edits) | `grep -rl "Handy" src/i18n/locales/` to find all files; batch-edit values |
| Portable mode magic string changed without backward compatibility | MEDIUM | Release a patch that reads both old and new magic strings; auto-upgrades on read |
| Patched Tauri branch abandoned | MEDIUM-HIGH | Fork `cjpais/tauri.git` to Dictus-owned repo; update `Cargo.toml` patch URL |
| Rust binary still named `handy` | LOW (Cargo.toml edit + rebuild) | Change `[package] name`, `default-run`, and `[lib] name`; test Windows (lib name collision caveat applies) |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Updater endpoint pointing to upstream | Phase 1: Bundle identity | `grep -r "cjpais/Handy" tauri.conf.json` returns empty |
| Bundle identifier not changed | Phase 1: Bundle identity | App data path contains `dictus` not `handy` |
| Windows signing identity from upstream | Phase 1: Bundle identity | `signCommand` contains no `cjpais` references |
| i18n "Handy" across 20+ locale files | Phase 2: UI rebrand | `grep -rl '"Handy"' src/i18n/locales/` returns empty |
| Visual components (HandyTextLogo, HandyHand) | Phase 2: UI rebrand | No `HandyTextLogo` or `HandyHand` imports in the codebase |
| Hardcoded URLs in About/UpdateChecker | Phase 2: UI rebrand | `grep -r "handy.computer\|cjpais/Handy" src/` returns empty |
| Model CDN dependency on `blob.handy.computer` | Phase 1 (document) / Phase 3+ (resolve) | Risk documented in README; decision recorded in PROJECT.md |
| Patched Tauri runtime tied to upstream branch | Phase 2 or V2 | `[patch.crates-io]` references Dictus-owned repo or is removed |
| `handy-keys` crate dependency | V2 risk review | Crate health checked; fork decision documented |
| Portable marker string | Phase 3: Technical renames | `grep "Handy Portable Mode" src-tauri/src/portable.rs` returns empty; backward compatibility code present |
| Rust binary name still `handy` | Phase 3: Technical renames | `cargo build` produces `dictus-desktop` binary; Windows lib name collision tested |

---

## Sources

- Direct codebase analysis: `tauri.conf.json`, `Cargo.toml`, `src-tauri/src/portable.rs`, `src-tauri/src/managers/model.rs`, `src-tauri/src/shortcut/mod.rs`
- Translation files: all 20+ locale files in `src/i18n/locales/`
- Component audit: `src/components/icons/`, `src/components/settings/about/AboutSettings.tsx`, `src/components/update-checker/UpdateChecker.tsx`
- CI workflow audit: `.github/workflows/build.yml`, `.github/workflows/release.yml`
- Known concerns: `.planning/codebase/CONCERNS.md`
- Project scope: `.planning/PROJECT.md`

---
*Pitfalls research for: Tauri 2.x desktop app rebranding (Handy → Dictus Desktop)*
*Researched: 2026-04-05*
