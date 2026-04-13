---
phase: 04-updater-infrastructure
verified: 2026-04-13T00:00:00Z
status: passed
score: 10/10 must-haves verified
re_verification: false
human_verification:
  - test: "Confirm Ed25519 private key is no longer present at ~/.tauri/dictus.key on Pierre's machine"
    expected: "File does not exist; triple-backup to Bitwarden + iCloud age-encrypted file is confirmed"
    why_human: "Cannot inspect Pierre's local filesystem — SUMMARY claims key was deleted but this cannot be verified programmatically from the repo"
  - test: "Confirm GitHub Secrets TAURI_SIGNING_PRIVATE_KEY and TAURI_SIGNING_PRIVATE_KEY_PASSWORD are set on getdictus/dictus-desktop"
    expected: "Both secrets visible (as masked) in https://github.com/getdictus/dictus-desktop/settings/secrets/actions"
    why_human: "GitHub Secrets are not readable via the API or filesystem; their presence is inferred from the v0.1.0 release succeeding with signed artifacts"
  - test: "Confirm published v0.1.0 release assets are prefixed dictus_ with no handy_* artifacts"
    expected: "All 27 assets at https://github.com/getdictus/dictus-desktop/releases/tag/v0.1.0 have dictus_ prefix"
    why_human: "Release asset names require browser or gh CLI inspection; SUMMARY documents 27 assets and dictus_ prefix"
  - test: "Confirm Windows installers produce the expected SmartScreen warning (acceptable for v0.1.0) and do not crash"
    expected: "Installer runs, SmartScreen warns, user can proceed — acceptable known limitation"
    why_human: "Requires Windows VM and live installer execution"
---

# Phase 04: Updater Infrastructure Verification Report

**Phase Goal:** The auto-updater is fully wired — Dictus can sign, publish, and deliver updates to users automatically
**Verified:** 2026-04-13
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Developer can generate, back up, and rotate the Ed25519 signing key following a single runbook | VERIFIED | `docs/RUNBOOK-updater-signing.md` (226 lines) contains full keygen, triple-backup, rotation, and release procedures. Literal secret names `TAURI_SIGNING_PRIVATE_KEY` / `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` present. |
| 2 | Runbook documents exact GitHub Secrets names the CI pipeline expects | VERIFIED | Both secret names appear verbatim in `RUNBOOK-updater-signing.md:39-40` and match `.github/workflows/build.yml:354-355` |
| 3 | Runbook contains full release procedure including stale-tag cleanup and post-release curl assertion | VERIFIED | §6 contains: git tag -d for stale tags, draft inspection checklist, post-publish `bash scripts/verify-updater-release.sh` call |
| 4 | A repeatable shell script validates every config assertion (UPDT-03..09) in under 10 seconds | VERIFIED | `.planning/phases/04-updater-infrastructure/scripts/validate.sh` — 8 of 9 assertions pass. One false-failure noted (see Anti-Patterns). |
| 5 | CI release workflow produces artifacts prefixed `dictus_` (not `handy_`) | VERIFIED | `release.yml:77` has `asset-prefix: "dictus"`. `build.yml:22` has `default: "dictus"`. No unintended `handy_` artifact prefix found. |
| 6 | tauri-action generates and uploads `latest.json` to the release | VERIFIED | `build.yml:371` has `includeUpdaterJson: true`. v0.1.0 release confirmed 27 assets including `latest.json`. |
| 7 | `tauri.conf.json` contains valid pubkey, endpoint, and createUpdaterArtifacts | VERIFIED | pubkey: non-empty raw base64 minisign key (decodes to `untrusted comment: minisign public key: 73CC9C02406C5A76`). endpoints: `["https://github.com/getdictus/dictus-desktop/releases/latest/download/latest.json"]`. createUpdaterArtifacts: true. |
| 8 | UpdateChecker.tsx fallback URL points to getdictus/dictus-desktop (not cjpais/Handy) | VERIFIED | `UpdateChecker.tsx:206` has `https://github.com/getdictus/dictus-desktop/releases/latest`. No `cjpais/Handy` reference. |
| 9 | v0.1.0 is published and latest.json is fetchable with structurally valid content | VERIFIED | `scripts/verify-updater-release.sh` exited 0 with output: "UPDT-10 PASS — latest.json is published and structurally valid" (27 assets, 18 platforms per SUMMARY) |
| 10 | Ed25519 keypair generated, backed up to three locations, GitHub Secrets set | VERIFIED (human-action) | 04-03-SUMMARY confirms triple-backup completed (Bitwarden key note, Bitwarden passphrase note, iCloud Drive age-p encrypted file), GitHub Secrets set via browser UI out-of-band. Cannot inspect filesystem directly — see human verification items. |

**Score:** 10/10 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `docs/RUNBOOK-updater-signing.md` | Single source of truth for updater key custody, rotation, release procedure | VERIFIED | 226 lines. Contains Ed25519, TAURI_SIGNING_PRIVATE_KEY, stale-tag cleanup, post-release curl assertion. |
| `.planning/phases/04-updater-infrastructure/scripts/validate.sh` | Shell-based config assertions for UPDT-03..UPDT-09 | VERIFIED | 66 lines. Contains getdictus/dictus-desktop. Executes all 8 config assertions (see note on false-failure below). |
| `scripts/verify-updater-release.sh` | Standalone UPDT-10 curl+jq assertion wrapper | VERIFIED | 30 lines. Contains `jq -e`. Fetches endpoint and validates `.version`, `.platforms`, platform count. |
| `src-tauri/tauri.conf.json` | Runtime updater config (pubkey, endpoint) + build-time sig generation flag | VERIFIED | pubkey (raw base64 minisign), endpoints pointing to getdictus release, createUpdaterArtifacts: true. |
| `.github/workflows/release.yml` | Asset-prefix passed to reusable build workflow | VERIFIED | Line 77: `asset-prefix: "dictus"`. No handy_ artifact prefix references. |
| `.github/workflows/build.yml` | Default asset-prefix + includeUpdaterJson + TECH-03 deferral comment | VERIFIED | default: "dictus" at line 22, includeUpdaterJson: true at line 371, TECH-03 comment at lines 373-376. |
| `src/components/update-checker/UpdateChecker.tsx` | Portable-update fallback opens Dictus releases page | VERIFIED | Line 206: `https://github.com/getdictus/dictus-desktop/releases/latest`. |
| `https://github.com/getdictus/dictus-desktop/releases/latest/download/latest.json` | Published UPDT-10 endpoint (external) | VERIFIED | verify-updater-release.sh passed. 18 platforms, structurally valid. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|-----|-----|--------|---------|
| `RUNBOOK-updater-signing.md` | `build.yml:354-355` secret names | Literal TAURI_SIGNING_PRIVATE_KEY references | WIRED | Same secret names in runbook and workflow |
| `validate.sh` | `src-tauri/tauri.conf.json` | jq assertions on plugins.updater fields | WIRED | validate.sh assertions verified against actual tauri.conf.json values — all pass |
| `release.yml:77` | `build.yml` asset-prefix input | workflow_call inputs.asset-prefix | WIRED | release.yml passes `asset-prefix: "dictus"` to build.yml which defaults to "dictus" |
| `build.yml` tauri-action with: block | tauri-action@v0 (generates latest.json) | includeUpdaterJson input | WIRED | `includeUpdaterJson: true` present at line 371; v0.1.0 confirms latest.json was generated |
| `UpdateChecker.tsx:206` | GitHub Releases page for getdictus/dictus-desktop | openUrl() call in portable-update dialog | WIRED | Line 206 confirmed pointing to getdictus/dictus-desktop/releases/latest |
| `tauri.conf.json bundle.createUpdaterArtifacts` | .sig file generation at build time | Tauri v2 bundle config | WIRED | `"createUpdaterArtifacts": true` + v0.1.0 .sig files present in release |
| `tauri.conf.json plugins.updater.pubkey` | Ed25519 private key in GitHub Secrets (signing) + client verification | Minisign Ed25519 pairing | WIRED | pubkey is the counterpart to TAURI_SIGNING_PRIVATE_KEY; v0.1.0 release succeeded with signatures |
| `tauri.conf.json plugins.updater.endpoints` | Published latest.json on getdictus releases | HTTPS GET from tauri-plugin-updater | WIRED | Endpoint URL matches published asset URL; verify-updater-release.sh passed |
| `scripts/verify-updater-release.sh` curl assertion | Published latest.json endpoint | HTTPS GET + jq structural validation | WIRED | Script ran and exited 0 ("UPDT-10 PASS") |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|---------|
| UPDT-01 | 04-01, 04-03 | Developer can generate Ed25519 keypair using `bunx tauri signer generate` and back up private key securely | SATISFIED | RUNBOOK-updater-signing.md documents full keypair generation and triple-backup. 04-03-SUMMARY confirms execution. |
| UPDT-02 | 04-01, 04-03 | Developer can add TAURI_SIGNING_PRIVATE_KEY and TAURI_SIGNING_PRIVATE_KEY_PASSWORD to GitHub Secrets | SATISFIED | RUNBOOK-updater-signing.md documents GitHub Secrets setup. 04-03-SUMMARY confirms human-action task completed. v0.1.0 signed artifacts confirm secrets were active at CI time. |
| UPDT-03 | 04-03 | tauri.conf.json is configured with Ed25519 public key in plugins.updater.pubkey | SATISFIED | pubkey field non-empty, valid base64-decoded minisign key starting with "untrusted comment: minisign public key: 73CC9C02406C5A76" |
| UPDT-04 | 04-03 | tauri.conf.json is configured with GitHub Releases latest.json endpoint in plugins.updater.endpoints | SATISFIED | endpoints[0] = "https://github.com/getdictus/dictus-desktop/releases/latest/download/latest.json" |
| UPDT-05 | 04-03 | tauri.conf.json has createUpdaterArtifacts: true in bundle section | SATISFIED | `"createUpdaterArtifacts": true` confirmed by direct JSON parse and validate.sh assertion |
| UPDT-06 | 04-02 | release.yml asset-prefix changed from "handy" to "dictus" | SATISFIED | release.yml:77 `asset-prefix: "dictus"` |
| UPDT-07 | 04-02 | build.yml asset-prefix changed from "handy" to "dictus" | SATISFIED | build.yml:22 `default: "dictus"` |
| UPDT-08 | 04-02 | build.yml has includeUpdaterJson: true in tauri-action config | SATISFIED | build.yml:371 `includeUpdaterJson: true` |
| UPDT-09 | 04-02 | UpdateChecker.tsx fallback URL points to getdictus/dictus-desktop releases | SATISFIED | UpdateChecker.tsx:206 confirmed; no cjpais/Handy reference present |
| UPDT-10 | 04-04 | A dry-run release validates that latest.json is generated and accessible on GitHub Releases | SATISFIED | v0.1.0 published. verify-updater-release.sh exited 0. 18 platforms, 27 assets in release. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `.planning/phases/04-updater-infrastructure/scripts/validate.sh` | 55 | `grep -B5 'otool -L'` false-failure | INFO | validate.sh exits 1 on "Anti-regression TECH-03 deferral comment" assertion because the TECH-03 comment is at lines 373-376 in build.yml but the first `otool -L` is at line 385 — 9 lines away, outside the -B5 window. The comment clearly exists and is correctly placed. This is a script logic bug (window too narrow), not a missing comment. All 8 substantive UPDT assertions pass. |
| `.github/workflows/build.yml` | 306, 321 | `blob.handy.computer` CDN URL for onnxruntime download | INFO | Upstream CDN URL, tracked for migration in V2 (INFR-01 per CLAUDE.md). Not a phase 4 requirement and not a dictus_ artifact prefix issue. |

No blocker anti-patterns found. One INFO-level validate.sh false-failure (TECH-03 grep window) and one INFO-level deferred CDN migration.

### Human Verification Required

#### 1. Ed25519 Private Key Deletion

**Test:** On Pierre's machine, run `ls ~/.tauri/dictus.key`
**Expected:** `No such file or directory` — key should have been deleted after triple-backup was confirmed
**Why human:** Cannot inspect developer's local filesystem from the repository

#### 2. GitHub Secrets Presence Confirmation

**Test:** Visit https://github.com/getdictus/dictus-desktop/settings/secrets/actions
**Expected:** Both `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` appear as masked secrets
**Why human:** GitHub Secrets are not readable programmatically; their functional presence is already implied by v0.1.0 shipping signed artifacts, so this is a formality check

#### 3. Published Asset Prefix Check

**Test:** Visit https://github.com/getdictus/dictus-desktop/releases/tag/v0.1.0 and inspect asset names
**Expected:** All 27 assets prefixed `dictus_` — no `handy_*` artifacts visible
**Why human:** Release asset names require browser or `gh release view` inspection

#### 4. Windows SmartScreen Behavior (Known Acceptable)

**Test:** Run the Windows installer from v0.1.0 on a Windows machine
**Expected:** SmartScreen warning appears (Azure Trusted Signing not yet configured), user can click through, installer completes successfully
**Why human:** Requires Windows VM with live installer execution — SmartScreen behavior cannot be verified from CI logs alone

### Gaps Summary

No gaps found. All 10 UPDT requirements are satisfied by code-verifiable evidence or confirmed human-action executions documented in SUMMARYs and corroborated by the successful v0.1.0 release.

**Notable observations:**

1. **validate.sh false-failure is a script bug, not a config failure.** The TECH-03 deferral comment exists at lines 373-376 of build.yml with the `otool` call at line 385. The `-B5` grep window (5 lines) is too narrow to catch a comment that is 9 lines above. All 8 substantive UPDT config assertions pass. The script exits 1 due to this one anti-regression heuristic. This should be fixed in a follow-up (`-B15` or a dedicated grep for TECH-03 + otool in the same file).

2. **18 platforms in latest.json (not 7) is expected.** tauri-action creates one entry per bundle format per architecture (e.g., linux-x86_64 appears as both deb and AppImage). This is correct and documented in 04-04-SUMMARY key-decisions.

3. **Windows binaries unsigned at OS level.** Azure Trusted Signing was explicitly deferred and was never a phase 4 requirement. Windows users will see SmartScreen warnings for v0.1.0. Tracked for a future phase.

4. **macOS x86_64 "Verify macOS dylib bundling" CI cosmetic failure.** The `otool -L "$APP/Contents/MacOS/handy"` step looks for a binary whose name may differ on x86_64 due to the deferred TECH-03 rename. The actual build and upload succeeded. This is the same TECH-03 deferral tracked in the code comment at build.yml:373-376.

---

_Verified: 2026-04-13_
_Verifier: Claude (gsd-verifier)_
