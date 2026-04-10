# Pitfalls Research

**Domain:** Desktop app rebranding — Tauri 2.x / Rust / React fork (Handy → Dictus Desktop)
**Researched:** 2026-04-10 (updated for v1.1: Auto-Update + Upstream Sync milestone)
**Confidence:** HIGH — based on direct codebase analysis and Tauri v2 official documentation

---

## v1.1 Scope

This file covers pitfalls for two new workstreams added in the v1.1 milestone:

1. **Tauri v2 auto-updater** — enabling the updater that was disabled in v1.0 (no Dictus endpoint existed)
2. **First upstream merge** — merging Handy v0.8.0-v0.8.2 changes (69 commits from fork point `85a8ed77`)

Pitfalls from v1.0 rebrand are preserved below in their own section for reference.

---

## Critical Pitfalls — Auto-Updater (v1.1)

### Pitfall A1: Ed25519 Keypair Lost or Never Generated

**What goes wrong:**
The Tauri updater uses a Minisign Ed25519 keypair. The public key goes in `tauri.conf.json`; the private key signs every release artifact. If the private key is ever lost after even one Dictus release is published, all existing installed copies can never receive an update. Users on those installs are permanently stranded — no workaround exists. The app's updater will keep checking and failing forever.

**Why it happens:**
The keypair is generated once with `tauri signer generate`. Developers generate it locally, add the pubkey to config, add the privkey to CI secrets, and never write the privkey down anywhere else. When CI secrets are rotated or a new repo is created, the key is gone.

**Consequences:**
Permanent loss of update delivery to any user who installed before the key was lost.

**Prevention:**
- Generate the keypair once with `bun run tauri signer generate -w ~/.tauri/dictus.key` before the first release build.
- Store the private key in at minimum two places: a password manager (Bitwarden/1Password entry) AND an encrypted backup file outside the repo.
- Add the public key to `tauri.conf.json` `plugins.updater.pubkey`.
- Add the private key and password as GitHub Actions secrets `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` before running `release.yml`.
- Document the key's location in a private note; never commit it.

**Detection (warning signs):**
- `tauri.conf.json` `plugins.updater.pubkey` is still `""` (current state).
- `TAURI_SIGNING_PRIVATE_KEY` is not set as a GitHub Actions secret.
- Build logs show `TAURI_SIGNING_PRIVATE_KEY` warning: `Signing key not set, skipping artifact signing`.

**Phase to address:** Auto-updater setup, before any release build that users will install.

---

### Pitfall A2: Asset Prefix Still "handy" — Updater URLs Point to Wrong Files

**What goes wrong:**
`release.yml` passes `asset-prefix: "handy"` to `build.yml`. The `tauri-action` uses this prefix to name release artifacts: `handy_0.1.0_aarch64.dmg`, `handy_0.1.0_x64-setup.exe`, etc. The Tauri updater then generates a `latest.json` with download URLs pointing to these `handy_*.dmg` filenames. If the pubkey is valid and the endpoint is correct but the asset prefix is wrong, updates will check for a version correctly but then try to download `handy_*.dmg` — which might not exist if a future rename happens, or which installs a binary named `handy` onto a machine where the user expects `Dictus`.

The build.yml `Verify macOS dylib bundling` step at line 380–392 also hard-references `Contents/MacOS/handy` — if the binary is ever renamed, this step will fail silently (the `|| true` at the end suppresses errors).

**Why it happens:**
`asset-prefix: "handy"` was the original Handy release workflow value. It was never updated during the v1.0 rebrand because `release.yml` was not exercised (no Dictus releases were made).

**Consequences:**
Release artifacts named `handy_*` instead of `dictus_*`. Users downloading manually from GitHub Releases get a confusingly named file. The updater `latest.json` URLs will contain `handy` in the filename, which may confuse debugging. Does not break updater functionality as long as the filename is consistent.

**Prevention:**
- Change `asset-prefix: "handy"` to `asset-prefix: "dictus"` in `release.yml` before the first Dictus release.
- Change the `asset-prefix` default in `build.yml` from `"handy"` to `"dictus"`.
- Fix the `Verify macOS dylib bundling` step to reference `Contents/MacOS/dictus` — or keep `handy` until TECH-03 (binary rename, deferred to V3+) is done; document the inconsistency.

**Detection:**
- `grep "asset-prefix" .github/workflows/release.yml` returns `"handy"`.
- First test release shows `handy_0.1.0_aarch64.dmg` in GitHub Releases assets.

**Phase to address:** Auto-updater setup / CI fix phase.

---

### Pitfall A3: `createUpdaterArtifacts` Not Set — No `.sig` Files Generated

**What goes wrong:**
The `tauri.conf.json` `bundle` section currently has no `createUpdaterArtifacts` key. Without this, `tauri build` does not generate `.sig` signature files alongside the installers. The `tauri-action` workflow will upload the unsigned installers but the updater has nothing to verify. When a user runs the update check, it will either fail with a signature error or (worse) appear to download but refuse to install.

The current `tauri.conf.json` updater config is:
```json
"updater": {
  "pubkey": "",
  "endpoints": []
}
```
Both fields are empty and `createUpdaterArtifacts` is absent.

**Why it happens:**
Tauri v2 separates artifact generation (`bundle.createUpdaterArtifacts`) from updater runtime config (`plugins.updater`). Developers configure one without the other.

**Consequences:**
Update check succeeds (version comparison works), download starts, installation blocked with `InvalidSignature` or missing artifact.

**Prevention:**
Add `"createUpdaterArtifacts": true` to the `bundle` section of `tauri.conf.json`. This tells the Tauri bundler to produce `.tar.gz` + `.sig` for macOS/Linux and `.nsis.zip` + `.sig` for Windows alongside the normal installer outputs.

```json
"bundle": {
  "createUpdaterArtifacts": true,
  ...
}
```

**Detection:**
- Build output contains no `.sig` files in `src-tauri/target/*/bundle/`.
- `latest.json` generated by tauri-action has empty `signature` fields.

**Phase to address:** Auto-updater setup.

---

### Pitfall A4: Endpoint URL Returns 404 or Wrong JSON — Updater Fails Silently

**What goes wrong:**
The Tauri updater calls the endpoint from `plugins.updater.endpoints`, parses the JSON, and compares versions. The recommended GitHub Releases endpoint is:

```
https://github.com/[owner]/[repo]/releases/latest/download/latest.json
```

Common failure modes in the Dictus context:
1. **Repo not public** — GitHub Releases assets are not accessible without auth; the updater request gets a redirect to a login page, not JSON.
2. **No release published yet** — endpoint 404s until the first release is created.
3. **Wrong repo path** — if someone copies the endpoint from upstream and still has `cjpais/Handy`, the check succeeds but offers Handy updates.
4. **`latest.json` not uploaded** — `tauri-action` only uploads `latest.json` if `createUpdaterArtifacts: true` is set AND the release upload step runs with a valid `releaseId`.

**Why it happens:**
Developers test the updater check before publishing a real release, or publish to a private repo, or forget to wire `release-id` through the workflow.

**Prevention:**
- Endpoint should be `https://github.com/[dictus-org]/dictus-desktop/releases/latest/download/latest.json`.
- Test by hitting the URL in a browser before enabling the updater in production.
- Keep `update_checks_enabled` defaulting to `true` in `settings.rs` (current behavior: line 455-456) — but pair it with a guard: if endpoint is empty string, disable silently.
- In the `release.yml`, confirm `releaseId` is always passed to `tauri-action`; this is what triggers `latest.json` upload.

**Detection:**
- `curl -I https://github.com/[owner]/dictus-desktop/releases/latest/download/latest.json` returns 404.
- UpdateChecker shows "Checking..." indefinitely and logs `Failed to check for updates` to console.

**Phase to address:** Auto-updater setup + first release dry-run.

---

### Pitfall A5: Version Mismatch Across Three Files — Updates Never Detected

**What goes wrong:**
The Tauri updater compares the currently installed version against the version in `latest.json`. The installed version comes from `tauri.conf.json` `version` field. The `latest.json` version comes from the release workflow. If `Cargo.toml` version, `tauri.conf.json` version, and `package.json` version are out of sync, update detection misbehaves:

- If `Cargo.toml` is `0.1.0` but `tauri.conf.json` is `0.1.1`, the bundle is stamped `0.1.1` but the Rust binary self-identifies as `0.1.0` in logs — confusing but not broken.
- If `tauri.conf.json` is `0.1.0` and `latest.json` is `v0.1.0` (with `v` prefix), version comparison may fail in some implementations.

Currently all three files are `0.1.0`. The risk appears when cutting the first `v1.1.0` release.

**Prevention:**
- When bumping version, update all three files atomically: `Cargo.toml`, `tauri.conf.json`, `package.json`.
- The `latest.json` `name` field should be `"v1.1.0"` (with `v` prefix, per Tauri convention) while the config files store `"1.1.0"` (without `v`).
- Do not use the release workflow's `get-version` step (grep on `tauri.conf.json`) as the single source of truth — verify all three files agree before triggering release.

**Detection:**
- `grep '"version"' src-tauri/tauri.conf.json Cargo.toml package.json` shows different values.
- Updater reports "up to date" immediately after a release that should be newer.

**Phase to address:** Version management, before first release.

---

### Pitfall A6: `update_checks_enabled` Defaults to `true` With No Valid Endpoint

**What goes wrong:**
`settings.rs` line 455–456 shows `default_update_checks_enabled()` returns `true`. With `pubkey: ""` and `endpoints: []` in `tauri.conf.json`, the app currently tries to check for updates on every launch (based on `lib.rs` line 229 and `useEffect` in `UpdateChecker.tsx`). With an empty endpoints array, the Tauri updater throws an error that is currently silently swallowed in `catch (error)` on line 88 of `UpdateChecker.tsx`.

This means: every user of the current v1.0 build launches the app, the updater fires, fails with an internal error, and the component silently resets to `isChecking: false`. No user-facing harm, but the error is happening in production.

**Prevention:**
- When wiring up the updater for real, confirm the guard at `lib.rs` line 317 (`if !settings.update_checks_enabled`) properly gates the Rust-side check.
- For v1.1, as soon as `endpoints` is populated with a real URL, existing v1.0 users will have `update_checks_enabled: true` in their persisted settings and will start hitting the endpoint — make sure the endpoint exists before the release.

**Detection:**
- Browser console on v1.0 shows `Failed to check for updates` on each launch.
- Tauri Rust logs show updater error at startup.

**Phase to address:** Awareness during auto-updater setup; not a blocker but ensure endpoint is live before `endpoints` is populated in `tauri.conf.json`.

---

## Critical Pitfalls — Upstream Merge (v1.1)

### Pitfall B1: Conflict in Files Modified by Both Sides — Tauri Custom Patch Fork

**What goes wrong:**
`Cargo.toml` has a `[patch.crates-io]` section pointing to `https://github.com/cjpais/tauri.git` branch `handy-2.10.2`. The upstream Handy repo may have updated to a different Tauri version or a different patch branch (e.g., `handy-2.11.x`). The current state shows 4 upstream commits post-v0.8.2, but future syncs will accumulate more. If upstream bumps the Tauri patch branch and the Dictus `Cargo.toml` is merged naively, the patch will either be duplicated (two `[patch.crates-io]` sections) or overridden with upstream's branch — which may not compile with Dictus-specific changes.

**Prevention:**
- Before any `git merge upstream/main`, run `git diff HEAD upstream/main -- src-tauri/Cargo.toml` to preview the `Cargo.toml` delta.
- After merge, explicitly verify `[patch.crates-io]` still references the correct branch and that `cargo build` succeeds.
- If upstream changes the Tauri patch branch (e.g., `handy-2.11.0`), evaluate whether to follow (inherit their patch) or stay on `handy-2.10.2` (known working).

**Detection:**
- `cargo build` fails after merge with `failed to resolve patches for crates.io`.
- Two `[patch.crates-io]` sections appear in `Cargo.toml` after a merge.

**Phase to address:** Upstream sync phase, pre-merge checklist step.

---

### Pitfall B2: Upstream Changes `tauri.conf.json` — Overwrites Dictus Identity Fields

**What goes wrong:**
`tauri.conf.json` is the most conflict-prone file in this fork. The Dictus version has `productName: "Dictus"`, `identifier: "com.dictus.desktop"`, `version: "0.1.0"`, `pubkey: ""`, `endpoints: []`. Upstream Handy has `productName: "Handy"`, `identifier: "com.handy.desktop"` (or `com.pais.handy`), their version number, and their own pubkey/endpoints.

When upstream makes any change to `tauri.conf.json` (e.g., adding a new bundle option, changing macOS entitlements, adding a new platform), `git merge` will produce a conflict on the entire file. A careless resolution that accepts upstream's changes wholesale will silently revert the bundle identifier to `com.handy.desktop`. Users who already installed `com.dictus.desktop` will have their settings orphaned on next launch — the app sees itself as a new install.

**Why it happens:**
`tauri.conf.json` is not just config — it is identity. Developers resolving merge conflicts focus on the functional change (e.g., new entitlements) and do not notice the identity fields being reverted.

**Prevention:**
- Mark `tauri.conf.json` as "always use ours for identity fields" in a merge driver or document explicit rules:
  - `productName`, `identifier`, `version`, `plugins.updater.pubkey`, `plugins.updater.endpoints` → always keep Dictus values
  - All other keys → evaluate upstream changes
- After every upstream merge, run: `grep -E '"identifier"|"productName"' src-tauri/tauri.conf.json` and verify values are Dictus, not Handy.
- Consider a post-merge smoke test: `tauri info` should show `com.dictus.desktop`.

**Detection:**
- `grep "com.handy\|com.pais\|Handy" src-tauri/tauri.conf.json` returns results after merge.
- App data directory on test machine shows `com.handy.desktop` after merge build.

**Phase to address:** Every upstream sync, as a mandatory post-merge verification step.

---

### Pitfall B3: Upstream Adds New Translation Keys — Dictus Locales Go Out of Sync

**What goes wrong:**
The upstream Handy project has 20+ PR contributors adding translation keys in every version (the 69 commits include multiple translation PRs). When merging, new keys added in `src/i18n/locales/en/translation.json` upstream will merge cleanly if they don't conflict. However, any key whose value references "Handy" will be added with that value. Separately, if upstream adds a key that the Dictus fork has already added (perhaps with a different structure), the merge will conflict.

**Consequences:**
- New features from upstream (v0.8.x) ship with "Handy" branding in user-facing strings.
- If a key conflict exists, the merge fails — developers resolve it manually and may introduce the wrong key structure.

**Prevention:**
- After merge, always run: `grep -r '"Handy"' src/i18n/locales/` — any hit is a rebrand miss.
- For keys added by upstream that contain "Handy", perform a targeted search-and-replace of the value (not the key name).
- If Dictus has modified a key's value that upstream also modified: keep Dictus's value unless upstream's change adds meaningful new content (e.g., a correction); never accept upstream's "Handy" product name in values.

**Detection:**
- Post-merge `grep -r '"Handy"' src/i18n/` returns results.
- A newly added feature from upstream shows "Handy" in the UI.

**Phase to address:** Upstream sync — post-merge i18n audit is mandatory.

---

### Pitfall B4: `Cargo.lock` Merge Conflict Resolved Incorrectly

**What goes wrong:**
`Cargo.lock` is a generated file that records the exact resolved versions of every Rust dependency. When merging upstream, both sides will have modified `Cargo.lock` (upstream bumped dependencies in their releases; Dictus may have added dependencies). `git merge` will produce a conflict in this file. Manually resolving `Cargo.lock` conflicts is extremely error-prone — the file format is not designed for human editing.

**Why it happens:**
Developers see a Cargo.lock conflict, pick one side with `git checkout --ours Cargo.lock` or `--theirs`, and move on without verifying the result is consistent with `Cargo.toml`.

**Prevention:**
- Never resolve `Cargo.lock` conflicts by manually editing — always regenerate.
- The correct procedure: resolve all `Cargo.toml` conflicts first, then run `cargo generate-lockfile` to regenerate `Cargo.lock` from scratch.
- Alternatively: `git checkout --theirs src-tauri/Cargo.lock && cargo update` — accept upstream's lock, then let Cargo re-resolve to include Dictus-specific dependencies.
- Add `.gitattributes` rule: `src-tauri/Cargo.lock merge=union` (merges non-conflicting changes automatically) to reduce conflict frequency.

**Detection:**
- `cargo build` fails after merge with version conflict errors citing dependencies not in `Cargo.toml`.
- `Cargo.lock` still has conflict markers (`<<<<<<<`) after resolution attempt.

**Phase to address:** Upstream sync — pre-merge, add `.gitattributes`; during merge, follow the regeneration procedure.

---

### Pitfall B5: New Upstream Features Reference `handy.computer` Infrastructure

**What goes wrong:**
The 4 current post-v0.8.2 upstream commits and any future ones may add new features that reference `blob.handy.computer` URLs (new models), `handy.computer` API endpoints, or `cjpais/Handy` GitHub links. These will merge cleanly (no conflict) but introduce new Handy-branded dependencies into Dictus.

The current merge base already has model URLs pointing to `blob.handy.computer`. The upstream commit `upgrade transcribe rs to 0.3.5` (in the 69-commit delta) likely added new model entries. Each such addition is a silent Handy dependency.

**Prevention:**
- Post-merge, run: `grep -r "handy.computer\|cjpais/Handy\|cjpais/Handy" src-tauri/src/` — any hit in newly-merged files is a foreign dependency.
- For new model URLs, decide: keep as-is (acceptable for v1.1, document the dependency) or redirect to Dictus CDN (INFR-01, deferred to V3+).
- For new UI/frontend references to Handy, replace before shipping.

**Detection:**
- Post-merge `grep -r "blob.handy.computer" src-tauri/src/managers/model.rs` shows new entries not present before merge.
- A new settings panel or feature shows "Handy" branding.

**Phase to address:** Upstream sync — post-merge foreign dependency scan.

---

### Pitfall B6: Upstream Bumps Version Number — Dictus Version Policy Confused

**What goes wrong:**
Upstream Handy is at v0.8.2. Dictus Desktop is at v0.1.0. These version namespaces are completely different (Dictus follows its own versioning). After merging upstream, `tauri.conf.json` on the upstream side has `version: "0.8.2"`. If this is accepted during conflict resolution, Dictus's internal version jumps from `0.1.0` to `0.8.2` — breaking the Dictus versioning strategy documented in `MEMORY.md` ("App is 0.1.0, GSD milestones ≠ app version, semver 0.x.y").

Additionally, the Tauri updater uses the version in `tauri.conf.json` to detect updates. If the version jumps to `0.8.2`, existing `0.1.0` users will immediately be offered an "update" to `0.8.2` even if no new Dictus release was published — or if the endpoint doesn't have a `0.8.2` release, update checks will silently fail.

**Prevention:**
- Always keep `version` in `tauri.conf.json` under Dictus's own versioning scheme.
- In conflict resolution for `tauri.conf.json`, upstream's `version` field is always discarded.
- Dictus version is incremented only intentionally when cutting a Dictus release (not as a side effect of upstream merges).

**Detection:**
- `grep '"version"' src-tauri/tauri.conf.json` shows `0.8.x` after merge.
- Tauri updater starts looking for a release that doesn't exist.

**Phase to address:** Upstream sync — `tauri.conf.json` post-merge verification.

---

### Pitfall B7: Weekly Upstream Detection Action Creates False Alarm Noise

**What goes wrong:**
The planned GitHub Action to detect new upstream commits will trigger on any commit to `upstream/main`, including dependency bumps, typo fixes, and translation PRs. If it creates a GitHub issue or sends a notification for every upstream commit, the signal-to-noise ratio becomes poor and maintainers start ignoring the notifications — defeating the purpose.

Separately, if the action runs `git fetch upstream` but the upstream remote is not configured in the CI environment (it's only configured locally), the action will fail.

**Prevention:**
- The action should report upstream changes as a single summary (e.g., "N new commits since last sync, latest: {commit message}") rather than per-commit issues.
- Configure a threshold: only notify when N commits or a new release tag exists upstream.
- The action must explicitly add the upstream remote before fetching: `git remote add upstream https://github.com/cjpais/Handy.git && git fetch upstream`.
- Store the last-synced upstream SHA in the repo (e.g., `.upstream-sync-marker`) so the action can compute delta cleanly.

**Detection:**
- GitHub Issues flooded with one-per-commit upstream notifications.
- Action fails with `fatal: 'upstream' does not appear to be a git repository`.

**Phase to address:** Upstream sync automation design.

---

## Technical Debt Patterns (v1.1 additions)

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|---|---|---|---|
| Keep `asset-prefix: "handy"` in release workflow | No CI changes | Release artifacts named `handy_*.dmg`; confusing but functional | Not acceptable — fix in v1.1 before first release |
| Merge upstream without auditing new model URLs | Fast merge | New `blob.handy.computer` entries added silently | Acceptable with documented list of new dependencies |
| Accept upstream `Cargo.lock` wholesale | Fast resolution | May miss Dictus-specific dependency resolution | Acceptable if followed by `cargo build` + full test pass |
| Keep `update_checks_enabled: true` default with no endpoint | No setting change needed | Invisible errors on every launch; wasted HTTP attempts | Fix by populating endpoint before shipping v1.1 |
| Use `tauri-action@v0` without pinning to specific version | Always latest action | Breaking change in tauri-action could silently break release CI | Pin to a specific SHA or tag after verifying it works |

---

## Integration Gotchas (v1.1 additions)

| Integration | Common Mistake | Correct Approach |
|---|---|---|
| Tauri updater `pubkey` | Add pubkey from key file but include the full PEM armor | `pubkey` must be the raw base64 public key string only, not PEM format |
| `endpoints` array | Use the GitHub Releases page URL | Must be the direct asset URL: `.../releases/latest/download/latest.json` (raw download, not page) |
| `createUpdaterArtifacts` | Set in wrong section (`plugins.updater` instead of `bundle`) | Must be in `bundle: { createUpdaterArtifacts: true }` |
| `TAURI_SIGNING_PRIVATE_KEY` in CI | Set as regular env var in workflow | Must be a GitHub Actions secret (encrypted); if set as plaintext in yml, it leaks in logs |
| Upstream merge + `bindings.ts` | Upstream adds new Tauri commands; `bindings.ts` is not regenerated | After merge, if `src-tauri/src/commands/` changed, run a debug build to regenerate `src/bindings.ts` via tauri-specta |
| Upstream merge + macOS entitlements | Upstream may change `entitlements.plist`; merge keeps old entitlements | Post-merge, `diff src-tauri/entitlements.plist <(git show upstream/main:src-tauri/entitlements.plist)` to check |

---

## "Looks Done But Isn't" Checklist (v1.1)

**Auto-Updater:**
- [ ] `tauri.conf.json` `bundle.createUpdaterArtifacts` is `true`
- [ ] `tauri.conf.json` `plugins.updater.pubkey` contains the Dictus Ed25519 public key (not empty string, not Handy's key)
- [ ] `tauri.conf.json` `plugins.updater.endpoints` points to `https://github.com/[dictus-org]/dictus-desktop/releases/latest/download/latest.json`
- [ ] `release.yml` `asset-prefix` is `"dictus"` not `"handy"`
- [ ] `build.yml` default `asset-prefix` is `"dictus"` not `"handy"`
- [ ] `UpdateChecker.tsx` line 206 portable update button URL points to Dictus releases, not `cjpais/Handy/releases/latest`
- [ ] GitHub Actions secret `TAURI_SIGNING_PRIVATE_KEY` is set in the repo settings
- [ ] Private key is backed up outside GitHub (password manager / encrypted file)
- [ ] A test release was dry-run: `latest.json` is accessible at the configured endpoint URL
- [ ] Release artifacts include `.sig` files alongside installer files

**Upstream Merge:**
- [ ] `git diff HEAD upstream/main -- src-tauri/tauri.conf.json` reviewed before merge; Dictus identity fields are NOT overwritten
- [ ] Post-merge: `grep '"identifier"\|"productName"' src-tauri/tauri.conf.json` shows `com.dictus.desktop` / `Dictus`
- [ ] Post-merge: `grep '"version"' src-tauri/tauri.conf.json` shows Dictus version (not `0.8.x`)
- [ ] Post-merge: `grep -r '"Handy"' src/i18n/locales/` returns empty (or only attribution/acknowledgment uses)
- [ ] Post-merge: `grep -r "cjpais/Handy" src/` returns empty (or only attribution links, not functional endpoints)
- [ ] `cargo build` succeeds after resolving `Cargo.toml` and `Cargo.lock` conflicts
- [ ] `[patch.crates-io]` in `Cargo.toml` references intended Tauri fork branch; no duplicate section
- [ ] `src/bindings.ts` regenerated if any Rust commands changed in upstream merge
- [ ] All new model URLs from upstream documented (even if still pointing to `blob.handy.computer`)

---

## Recovery Strategies (v1.1)

| Pitfall | Recovery Cost | Recovery Steps |
|---|---|---|
| Ed25519 private key lost after first release | CRITICAL — no recovery | Must generate new keypair; existing installs permanently cannot update. Mitigate: include fallback "manual download" link in the update UI |
| `asset-prefix` wrong in first release | LOW | Change prefix in workflow for next release; existing release artifacts keep old name (no impact on updater if endpoint URL is consistent) |
| `tauri.conf.json` identity reverted after upstream merge | HIGH if already shipped | Roll back via git revert; if shipped, next release must re-establish `com.dictus.desktop` identifier (data migration required for anyone who received the wrong identifier build) |
| Upstream version number accepted in merge | MEDIUM | Increment to next intended Dictus version (e.g., `0.2.0`), rebuild, republish; existing installs on wrong version will get "update available" notification |
| New "Handy" strings slipped in via upstream translation merge | LOW | JSON edit + patch release |
| `Cargo.lock` regenerated incorrectly | LOW if caught before release | `cargo generate-lockfile` followed by `cargo build` |

---

## Phase-Specific Warnings (v1.1)

| Phase Topic | Likely Pitfall | Mitigation |
|---|---|---|
| Ed25519 keypair generation | Generating but not backing up the private key | Generate → immediately store in password manager → add to CI secrets → verify CI build signs artifacts |
| `tauri.conf.json` updater config | Enabling updater before endpoint is live | Populate `endpoints` only after the first release is published and `latest.json` is accessible |
| `release.yml` asset prefix fix | Missing the `asset-prefix` default in `build.yml` (there are two places) | Fix both `release.yml` and `build.yml` in the same commit |
| First upstream merge | Conflict resolution accepting wrong values for identity fields | Write a post-merge checklist; use `grep` assertions as verification |
| Upstream `bindings.ts` changes | Upstream may have regenerated bindings with new commands; merge may conflict | If `bindings.ts` conflicts, delete it and regenerate with a debug build |
| Weekly upstream detection action | Upstream remote not configured in CI | Add `git remote add upstream` step explicitly; test action with `workflow_dispatch` before enabling schedule |
| `Cargo.lock` post-merge | Manual edit of Cargo.lock introduces subtle version conflicts | Never edit Cargo.lock manually; always regenerate |

---

## v1.0 Rebrand Pitfalls (preserved for reference)

The following pitfalls were identified during the v1.0 milestone and are resolved or deferred. Kept here as context.

### Resolved in v1.0
- **Bundle Identifier:** Changed from `com.pais.handy` to `com.dictus.desktop` before any Dictus distribution.
- **i18n "Handy" in all 20 locales:** All locale files updated as part of the rebrand.
- **Visual components (HandyTextLogo, HandyHand):** Replaced with Dictus SVGs.
- **Windows NSIS signing identity (`cjpais-dev`):** Removed/updated.
- **About panel URLs:** Updated to Dictus references.

### Still Active / Deferred
- **Updater endpoint:** Was disabled (empty endpoints). v1.1 must wire this up.
- **`blob.handy.computer` model CDN:** Still in use, documented as deferred to V3+ (INFR-01).
- **`cjpais/tauri.git` Cargo patch:** Still active (`handy-2.10.2` branch), documented, deferred.
- **`handy-keys` crate:** Still used, fallback exists, deferred to V2 risk review.
- **Binary name `handy`:** Still `Contents/MacOS/handy`, deferred to V3+ (TECH-03).
- **Portable mode magic string `"Handy Portable Mode"`:** Still present in `portable.rs`, deferred.
- **CLI `about = "Handy - Speech to Text"`** in `src-tauri/src/cli.rs` line 4: still `handy`, deferred.

---

## Sources

- Direct codebase analysis: `tauri.conf.json`, `Cargo.toml`, `release.yml`, `build.yml`, `UpdateChecker.tsx`, `settings.rs`, `lib.rs`, `portable.rs`
- Tauri v2 updater official docs: https://v2.tauri.app/plugin/updater/
- Tauri v2 GitHub pipeline docs: https://v2.tauri.app/distribute/pipelines/github/
- Tauri issue #9565 (InvalidSignature root causes): https://github.com/tauri-apps/tauri/issues/9565
- Tauri issue #10316 (UnexpectedKeyId root causes): https://github.com/tauri-apps/tauri/issues/10316
- tauri-action assetNamePattern docs: https://github.com/tauri-apps/tauri-action
- Git upstream sync state: `git merge-base HEAD upstream/main` = `39e855d` (v0.8.2 release commit); 4 commits ahead on upstream/main, 55 commits ahead on Dictus main

---

_Pitfalls research for: Tauri v2 auto-updater + upstream sync (Handy → Dictus Desktop v1.1)_
_Researched: 2026-04-10_
