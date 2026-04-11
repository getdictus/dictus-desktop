# Phase 4: Updater Infrastructure - Research

**Researched:** 2026-04-11
**Domain:** Tauri v2 auto-updater wiring — Ed25519 keypair, tauri.conf.json, CI workflow, URL fix, dry-run release
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **Ed25519 generation**: `bunx tauri signer generate -w ~/.tauri/dictus.key` locally, once. Strong passphrase distinct from Bitwarden master.
- **CI signing secrets**: `TAURI_SIGNING_PRIVATE_KEY` + `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` in GitHub Secrets on `getdictus/dictus-desktop`. Already referenced at `build.yml:354-355` — no workflow plumbing change needed once secrets exist.
- **Primary backup**: Bitwarden — two separate secure notes under `getdictus` folder: one for key material, one for passphrase.
- **Secondary backup**: `age -R <YubiKey age pubkey>` encrypted file on external drive. YubiKey is for backup encryption only, NOT for Tauri signing backend (explicitly rejected — see context for three reasons).
- **After setup**: delete `~/.tauri/dictus.key` from disk once all three storage locations confirmed.
- **Recovery**: accepted as CRITICAL unrecoverable risk. No in-app rotation in v1.1. `UpdateChecker.tsx` manual-reinstall fallback stays.
- **Runbook**: single file `docs/RUNBOOK-updater-signing.md` (new). Must cover custody, recovery, rotation, and release procedure.
- **tauri.conf.json**: `bundle.createUpdaterArtifacts` → `true`; `plugins.updater.pubkey` → raw base64 from `.key.pub`; `plugins.updater.endpoints` → `["https://github.com/getdictus/dictus-desktop/releases/latest/download/latest.json"]`.
- **release.yml:77**: `asset-prefix: "handy"` → `asset-prefix: "dictus"`.
- **build.yml**: add `includeUpdaterJson: true` to tauri-action `with:` block (~line 365); add explanatory comment above `Verify macOS dylib bundling` step at ~line 380 about deferred TECH-03.
- **build.yml:380**: `otool -L "$APP/Contents/MacOS/handy"` line stays as-is. Keep `|| true`. Do NOT rename.
- **UpdateChecker.tsx:206**: `https://github.com/cjpais/Handy/releases/latest` → `https://github.com/getdictus/dictus-desktop/releases/latest`.
- **Version policy**: v0.1.0 is the first real public Dictus Desktop release. No pre-bump. Future bumps must update Cargo.toml + tauri.conf.json + package.json atomically.
- **Dry-run release (UPDT-10)**: Pierre triggers `release.yml` manually via GitHub Actions UI. Creates draft. Pierre publishes. Not automated from plan task. Pre-flight: delete stale local tags `0.1.0` and `v0.1.0-dictus` before triggering.
- **UPDT-10 success assertion**:
  ```bash
  curl -sfL https://github.com/getdictus/dictus-desktop/releases/latest/download/latest.json \
    | jq -e '.version and .platforms and (.platforms | keys | length > 0)'
  ```
  Must return exit code 0 with non-empty `platforms`. Anything else fails the phase.
- **Deferred TECH-03**: Only add explanatory comment to `build.yml:380`. No code change to binary name.

### Claude's Discretion

- Exact wording of the explanatory comment above `build.yml:380`.
- Exact layout and prose of `docs/RUNBOOK-updater-signing.md` (must cover custody, recovery, rotation, release procedure).
- Whether the `curl | jq` assertion lives inline in the runbook, as `scripts/verify-updater-release.sh`, or both.
- Exact placement of `includeUpdaterJson: true` within the existing `with:` block in `build.yml`.
- Whether cleanup of stale local tags runs as its own plan task or is folded into the runbook release-procedure section.

### Deferred Ideas (OUT OF SCOPE)

- In-app "broken updater" notification UI.
- Full TECH-03 binary rename (`handy → dictus`).
- Dropping `|| true` on macOS dylib verify step.
- `scripts/bump-version.sh` atomic version-bump helper.
- Notarization / Apple Developer code signing (INFR-03).
- Model CDN migration (INFR-01).
- Tag-push trigger on `release.yml`.
- Release notes template / changelog discipline.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| UPDT-01 | Developer can generate Ed25519 keypair using `bunx tauri signer generate` and back up private key securely | Keypair generation command verified; backup strategy (Bitwarden + age) documented in CONTEXT.md and grounded in pitfall A1 |
| UPDT-02 | Developer can add `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` to GitHub Secrets | `build.yml:354-355` already has env passthrough — secrets just need to be added in GitHub repo settings UI; no workflow change needed |
| UPDT-03 | `tauri.conf.json` configured with Ed25519 public key in `plugins.updater.pubkey` | Current file confirmed: `"pubkey": ""` at line 68. Must be raw base64 from `.key.pub`, NOT PEM-wrapped |
| UPDT-04 | `tauri.conf.json` configured with GitHub Releases `latest.json` endpoint in `plugins.updater.endpoints` | Current: `"endpoints": []`. Target URL confirmed: `https://github.com/getdictus/dictus-desktop/releases/latest/download/latest.json` |
| UPDT-05 | `tauri.conf.json` has `createUpdaterArtifacts: true` in `bundle` section | Current: `"createUpdaterArtifacts": false` at line 28. Must be in `bundle`, NOT in `plugins.updater` |
| UPDT-06 | `release.yml` asset-prefix changed from `"handy"` to `"dictus"` | Confirmed at `release.yml:77`: `asset-prefix: "handy"` — single-line change |
| UPDT-07 | `build.yml` asset-prefix changed from `"handy"` to `"dictus"` | Confirmed at `build.yml:22`: `default: "handy"` — single-line change; fixes the default for all callers |
| UPDT-08 | `build.yml` has `includeUpdaterJson: true` in tauri-action config | Currently absent from `with:` block at ~line 365. Must be added alongside existing `tagName`, `releaseId`, etc. |
| UPDT-09 | `UpdateChecker.tsx` fallback URL points to `getdictus/dictus-desktop` releases | Confirmed at line 206: `https://github.com/cjpais/Handy/releases/latest` — single-line change |
| UPDT-10 | Dry-run release validates that `latest.json` is generated and accessible | Depends on UPDT-01 through UPDT-08 all complete. Manual: Pierre triggers `release.yml` → draft release → publishes → curl assertion. Stale tag cleanup required first. |
</phase_requirements>

---

## Summary

Phase 4 wires the Tauri v2 auto-updater end-to-end for the first time in Dictus Desktop. The infrastructure is almost entirely pre-built: `tauri-plugin-updater` is registered in `lib.rs`, `build.yml` already forwards signing secrets to `tauri-action`, and `UpdateChecker.tsx` already implements the full UX. What is missing is configuration plumbing — an Ed25519 keypair, three `tauri.conf.json` fields, two CI workflow tweaks (asset-prefix + `includeUpdaterJson`), and one hardcoded URL fix.

The most critical sequential constraint is: keypair generation must happen first, because the pubkey goes into `tauri.conf.json` and the private key must be in GitHub Secrets before any signed build can run. Everything else (URL fix, config changes, CI edits) is independent and can be batched. UPDT-10 (dry-run release) is the integration gate — it depends on all prior requirements being complete and proves the chain works end-to-end.

The key operational risk is Ed25519 private key loss: there is no recovery path once a signed release ships. The runbook (`docs/RUNBOOK-updater-signing.md`) serves as the single source of truth for custody, rotation, and the release procedure.

**Primary recommendation:** Execute tasks in this order: (1) keypair generation + backup + GitHub Secrets, (2) `tauri.conf.json` config, (3) CI workflow fixes + URL fix, (4) runbook creation, (5) UPDT-10 dry-run validation by Pierre.

---

## Standard Stack

### Core (already present, no installation needed)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `tauri-plugin-updater` | 2.10.0 | Tauri v2 auto-update plugin | Official Tauri updater; already in `Cargo.toml` and registered in `lib.rs:532` |
| `tauri-apps/tauri-action` | v0 | GitHub Action for building + signing + uploading | Official Tauri CI action; already in `build.yml:341` |
| `@tauri-apps/plugin-updater` | (JS counterpart) | Frontend JS bindings for updater plugin | Already used in `UpdateChecker.tsx` |
| `bunx tauri signer` | (tauri-cli) | Ed25519 keypair generation + artifact signing | Official Tauri tool; Minisign-based, not OpenPGP |

**No new dependencies.** All required libraries are already installed.

### Supporting

| Tool | Purpose | When to Use |
|------|---------|-------------|
| `age` | Encrypt offline backup of private key | One-time: after generating keypair, before deleting local copy |
| `jq` | Validate `latest.json` structure in curl assertion | UPDT-10 validation step |
| GitHub Secrets UI | Store `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | One-time setup |
| Bitwarden | Primary backup of key material and passphrase | Immediately after keygen |

---

## Architecture Patterns

### Tauri v2 Updater Config Split

Tauri v2 separates artifact generation from runtime config across two different sections of `tauri.conf.json`. This is not obvious and is the most common misconfiguration (Pitfall A3):

```json
// BUNDLE section controls artifact GENERATION at build time
"bundle": {
  "createUpdaterArtifacts": true   // produces .sig files alongside installers
},

// PLUGINS section controls runtime UPDATE CHECK behavior
"plugins": {
  "updater": {
    "pubkey": "<raw base64 content of .key.pub>",
    "endpoints": [
      "https://github.com/getdictus/dictus-desktop/releases/latest/download/latest.json"
    ]
  }
}
```

**Critical:** `pubkey` must be the raw base64 string, NOT PEM-wrapped (no `-----BEGIN...-----` headers). Paste the content of `~/.tauri/dictus.key.pub` directly.

### tauri-action `with:` Block — Where `includeUpdaterJson` Goes

Current `build.yml` tauri-action block (lines 365-371):

```yaml
with:
  tagName: ${{ inputs.release-id && format('v{0}', steps.get-version.outputs.version) || '' }}
  releaseName: ${{ inputs.release-id && format('v{0}', steps.get-version.outputs.version) || '' }}
  releaseId: ${{ inputs.release-id }}
  assetNamePattern: ${{ steps.patch-release-name.outputs.platform }}
  args: ${{ inputs.build-args }}
```

Add `includeUpdaterJson: true` as a new line in this `with:` block. This tells `tauri-action` to generate and upload `latest.json` to the release when all platforms are done.

### Asset Name Pattern Flow

The `assetNamePattern` output from `patch-release-name` step combines `asset-prefix` with platform. Changing the default in `build.yml:22` from `"handy"` to `"dictus"` fixes the pattern for all callers. `release.yml:77` explicitly passes `asset-prefix: "handy"` — that line must ALSO change to `"dictus"`. Both must be changed in the same commit or the release workflow will still produce `handy_*` prefixed artifacts.

### Secret Forwarding Pattern (Already Correct)

`build.yml:354-355` already uses the established project pattern:
```yaml
TAURI_SIGNING_PRIVATE_KEY: ${{ inputs.sign-binaries && secrets.TAURI_SIGNING_PRIVATE_KEY || '' }}
TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ inputs.sign-binaries && secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD || '' }}
```
This means secrets are empty for PR builds and populated only for signed release builds (`sign-binaries: true`). No workflow change needed — only set the GitHub Secrets at the repo level.

### latest.json Format (Generated by tauri-action)

```json
{
  "version": "0.1.0",
  "notes": "Release notes from GitHub",
  "pub_date": "2026-04-11T10:00:00Z",
  "platforms": {
    "darwin-aarch64": {
      "signature": "<content of .sig file>",
      "url": "https://github.com/getdictus/dictus-desktop/releases/download/v0.1.0/Dictus_0.1.0_aarch64.dmg"
    },
    "darwin-x86_64": { "signature": "...", "url": "..." },
    "linux-x86_64": { "signature": "...", "url": "..." },
    "windows-x86_64": { "signature": "...", "url": "..." }
  }
}
```

Platform keys follow `{os}-{arch}` convention. `tauri-action` generates this automatically from the `.sig` files uploaded by each platform job.

### Anti-Patterns to Avoid

- **Setting `createUpdaterArtifacts: true` in `plugins.updater` instead of `bundle`** — wrong section, has no effect; sig files will not be generated.
- **Using PEM-wrapped pubkey** — paste raw base64 only, or the signature verification will fail with `UnexpectedKeyId`.
- **Using the GitHub Releases page URL** — `endpoints` must use the direct asset download URL (`.../releases/latest/download/latest.json`), not the HTML page URL.
- **Setting `TAURI_SIGNING_PRIVATE_KEY` as plaintext in the YAML** — it must be a GitHub Actions encrypted secret; plaintext leaks in logs.
- **Triggering a release before the endpoint is live** — `update_checks_enabled` defaults to `true`; once `endpoints` is populated, v0.1.0 installs will start checking. The endpoint must exist before publish.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Ed25519 signing for updates | Custom signing script | `bunx tauri signer` | Minisign format required; custom tools produce incompatible `.sig` files |
| `latest.json` generation | Manual JSON construction | `tauri-action` with `includeUpdaterJson: true` | tauri-action handles multi-platform aggregation, signature embedding, and release upload automatically |
| Version detection in release workflow | Custom grep/parse | Existing `get-version` step in `build.yml:71-77` | Already implemented; grepping `tauri.conf.json` is the established pattern |
| Offline key backup | gpg/openssl custom | `age -R <yubikey_age_pubkey>` | Simple, modern, YubiKey-compatible; no key management overhead |

---

## Common Pitfalls

### Pitfall 1: Ed25519 Private Key Lost Post-Release (A1)
**What goes wrong:** Key lost after first signed release ships → all installed copies permanently stranded, can never update.
**Why it happens:** Key generated locally, added to CI secrets, local file deleted without backup.
**How to avoid:** Generate → Bitwarden (two notes: key + passphrase) → `age` encrypted offline file → GitHub Secrets → delete local copy only after all three backups confirmed.
**Warning signs:** `TAURI_SIGNING_PRIVATE_KEY` secret absent from repo settings; `plugins.updater.pubkey` is `""`.

### Pitfall 2: Wrong Config Section for `createUpdaterArtifacts` (A3)
**What goes wrong:** `.sig` files not generated → update download fails with `InvalidSignature` or missing artifact.
**Why it happens:** Developers put `createUpdaterArtifacts` in `plugins.updater` instead of `bundle`.
**How to avoid:** `bundle.createUpdaterArtifacts: true` — not `plugins.updater.createUpdaterArtifacts`.
**Warning signs:** Build output has no `.sig` files; `latest.json` has empty `signature` fields.

### Pitfall 3: PEM-Wrapped Pubkey in `tauri.conf.json` (Integration Gotcha)
**What goes wrong:** `UnexpectedKeyId` error when user attempts update download.
**Why it happens:** Developer copies the entire `.key.pub` file contents including PEM header/footer.
**How to avoid:** The `.key.pub` file from `tauri signer generate` contains only the raw base64 string (no PEM armor). Paste that string as-is.
**Warning signs:** `pubkey` value in `tauri.conf.json` starts with `-----BEGIN`.

### Pitfall 4: Both Asset-Prefix Locations Must Change (A2)
**What goes wrong:** Release still produces `handy_*` artifacts if only one location is updated.
**Why it happens:** Two places — `release.yml:77` (explicit pass) and `build.yml:22` (default) — both have `"handy"`.
**How to avoid:** Change both in the same commit. Grep assertion: `grep "asset-prefix" .github/workflows/release.yml .github/workflows/build.yml` should show `"dictus"` in both.
**Warning signs:** First test release shows `handy_0.1.0_aarch64.dmg` in GitHub Releases assets.

### Pitfall 5: Endpoint Returns 404 Until First Release Published (A4)
**What goes wrong:** Once `endpoints` is populated in `tauri.conf.json`, the app starts checking for updates. Before the first release is published, the URL 404s. This causes a silent error per launch (currently happening with empty endpoints, but won't affect users since no installs exist yet).
**Why it happens:** The endpoint URL only becomes valid after `release.yml` completes and Pierre publishes the draft.
**How to avoid:** Sequence matters — populate `endpoints` in `tauri.conf.json`, trigger release, publish draft, then do the curl assertion. Don't expect the endpoint to be live until the release is published.
**Warning signs:** `curl -I https://github.com/getdictus/dictus-desktop/releases/latest/download/latest.json` returns 404.

### Pitfall 6: Stale Local Tags Blocking Release Workflow (Pre-flight)
**What goes wrong:** `release.yml` tries to create tag `v0.1.0` but it already exists locally → tag creation fails.
**Why it happens:** Old test runs created stale local tags `0.1.0` and `v0.1.0-dictus`.
**How to avoid:** Pre-flight: `git tag -d 0.1.0 v0.1.0-dictus` before triggering `release.yml`. Also verify no remote tags exist: `git ls-remote --tags origin`.
**Warning signs:** `release.yml` fails at the "Create Draft Release" step with tag conflict error.

---

## Code Examples

### Current State vs Required State: tauri.conf.json

```json
// BEFORE (current — lines 28, 67-70):
"bundle": {
  "createUpdaterArtifacts": false,
  ...
},
"plugins": {
  "updater": {
    "pubkey": "",
    "endpoints": []
  }
}

// AFTER:
"bundle": {
  "createUpdaterArtifacts": true,
  ...
},
"plugins": {
  "updater": {
    "pubkey": "<raw base64 from ~/.tauri/dictus.key.pub>",
    "endpoints": [
      "https://github.com/getdictus/dictus-desktop/releases/latest/download/latest.json"
    ]
  }
}
```

### Current State vs Required State: release.yml

```yaml
# BEFORE (line 77):
asset-prefix: "handy"

# AFTER:
asset-prefix: "dictus"
```

### Current State vs Required State: build.yml (two changes)

```yaml
# CHANGE 1 — line 22 (default value):
# BEFORE:
asset-prefix:
  required: false
  type: string
  default: "handy"
# AFTER:
  default: "dictus"

# CHANGE 2 — add to with: block (~line 365):
with:
  tagName: ${{ inputs.release-id && format('v{0}', steps.get-version.outputs.version) || '' }}
  releaseName: ${{ inputs.release-id && format('v{0}', steps.get-version.outputs.version) || '' }}
  releaseId: ${{ inputs.release-id }}
  assetNamePattern: ${{ steps.patch-release-name.outputs.platform }}
  args: ${{ inputs.build-args }}
  includeUpdaterJson: true   # ADD THIS LINE

# CHANGE 3 — add comment above "Verify macOS dylib bundling" step (~line 372):
# NOTE: The binary is named 'handy' (not 'dictus') because TECH-03 (binary rename)
# is deferred. Do NOT change 'Contents/MacOS/handy' to 'Contents/MacOS/dictus' here
# until TECH-03 is explicitly un-deferred. Keep '|| true' tolerance as-is.
```

### Current State vs Required State: UpdateChecker.tsx

```typescript
// BEFORE (line 206):
openUrl("https://github.com/cjpais/Handy/releases/latest");

// AFTER:
openUrl("https://github.com/getdictus/dictus-desktop/releases/latest");
```

### Keypair Generation (One-Time, Run Locally)

```bash
# Generate keypair
bunx tauri signer generate -w ~/.tauri/dictus.key
# Enter strong passphrase when prompted (distinct from Bitwarden master password)

# View pubkey to copy into tauri.conf.json
cat ~/.tauri/dictus.key.pub
# Paste raw base64 output (no headers) into plugins.updater.pubkey

# After Bitwarden + age backup confirmed:
rm ~/.tauri/dictus.key
```

### UPDT-10 Validation Command

```bash
curl -sfL https://github.com/getdictus/dictus-desktop/releases/latest/download/latest.json \
  | jq -e '.version and .platforms and (.platforms | keys | length > 0)'
# Must exit 0. Non-zero or JSON parse failure = phase not complete.
```

### Version Consistency Assertion (Post-Bump Verification)

```bash
grep '"version"' src-tauri/tauri.conf.json Cargo.toml package.json
# All three lines must show the same version value
```

### Stale Tag Cleanup (Pre-Flight for UPDT-10)

```bash
git tag -d 0.1.0 v0.1.0-dictus 2>/dev/null || true
git ls-remote --tags origin  # verify no conflicting remote tags for v0.1.0
```

---

## State of the Art

| Old Approach | Current Approach | Impact |
|--------------|------------------|--------|
| Tauri v1 `updater.active: true` in root config | Tauri v2: split between `bundle.createUpdaterArtifacts` + `plugins.updater` | Must know the v2 split; old docs/examples are misleading |
| `TAURI_PRIVATE_KEY` env var (Tauri v1) | `TAURI_SIGNING_PRIVATE_KEY` (Tauri v2) | Different variable name; v1 examples in StackOverflow will mislead |
| `pubkey` set to PEM-wrapped string | Raw base64 only (Minisign format) | PEM-wrapped causes `UnexpectedKeyId` runtime error |

---

## Open Questions

1. **Is the `getdictus/dictus-desktop` repo public?**
   - What we know: Pitfall A4 states GitHub Releases assets are not accessible without auth on private repos.
   - What's unclear: Current visibility of the repo has not been verified in this research session.
   - Recommendation: Confirm repo is public before triggering UPDT-10. If private, `latest.json` endpoint will redirect to login page, not return JSON.

2. **Does `tauri-action@v0` require a minimum version for `includeUpdaterJson`?**
   - What we know: `includeUpdaterJson` is documented in the official tauri-action README and used in official pipeline docs.
   - What's unclear: Whether `@v0` is pinned to a version that supports it.
   - Recommendation: If the first test release does not produce `latest.json`, check if upgrading to a specific tauri-action SHA is needed.

3. **Do any of the 7 build matrix platforms produce artifacts with different naming conventions that could break `latest.json` URL construction?**
   - What we know: ARM Linux and ARM Windows use non-standard platform strings.
   - What's unclear: How `tauri-action` constructs download URLs in `latest.json` for `aarch64-unknown-linux-gnu` and `aarch64-pc-windows-msvc` targets.
   - Recommendation: In UPDT-10, verify the `platforms` object in `latest.json` covers all expected targets, not just x86_64.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | None detected (no jest.config, vitest.config, or test directories present) |
| Config file | None — see Wave 0 |
| Quick run command | N/A — all validations are configuration assertions or curl checks |
| Full suite command | N/A |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| UPDT-01 | `~/.tauri/dictus.key.pub` exists after generation | manual-only | N/A — local file, Pierre runs once | N/A |
| UPDT-02 | GitHub Secrets set in repo settings | manual-only | N/A — GitHub UI action | N/A |
| UPDT-03 | `tauri.conf.json` pubkey is non-empty base64 | smoke | `python3 -c "import json,base64,sys; d=json.load(open('src-tauri/tauri.conf.json')); k=d['plugins']['updater']['pubkey']; base64.b64decode(k); print('pubkey ok')"` | ❌ Wave 0 |
| UPDT-04 | `tauri.conf.json` endpoint points to correct URL | smoke | `grep 'getdictus/dictus-desktop' src-tauri/tauri.conf.json` | ❌ Wave 0 |
| UPDT-05 | `tauri.conf.json` bundle.createUpdaterArtifacts is true | smoke | `python3 -c "import json; d=json.load(open('src-tauri/tauri.conf.json')); assert d['bundle']['createUpdaterArtifacts'] == True, 'not true'; print('ok')"` | ❌ Wave 0 |
| UPDT-06 | `release.yml` asset-prefix is `dictus` | smoke | `grep 'asset-prefix: "dictus"' .github/workflows/release.yml` | ❌ Wave 0 |
| UPDT-07 | `build.yml` default asset-prefix is `dictus` | smoke | `grep 'default: "dictus"' .github/workflows/build.yml` | ❌ Wave 0 |
| UPDT-08 | `build.yml` has `includeUpdaterJson: true` | smoke | `grep 'includeUpdaterJson: true' .github/workflows/build.yml` | ❌ Wave 0 |
| UPDT-09 | `UpdateChecker.tsx` URL points to `getdictus/dictus-desktop` | smoke | `grep 'getdictus/dictus-desktop' src/components/update-checker/UpdateChecker.tsx` | ❌ Wave 0 |
| UPDT-10 | `latest.json` accessible and structurally valid | integration | `curl -sfL https://github.com/getdictus/dictus-desktop/releases/latest/download/latest.json \| jq -e '.version and .platforms and (.platforms \| keys \| length > 0)'` | ❌ Wave 0 — manual trigger by Pierre |

### Sampling Rate

- **Per task commit:** Run the smoke grep/python assertion for the specific file modified
- **Per wave merge:** Run all UPDT-03 through UPDT-09 smoke assertions together
- **Phase gate:** UPDT-10 curl assertion green (requires Pierre to publish release) before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `scripts/verify-updater-release.sh` — wraps UPDT-10 curl + jq assertion for easy re-run
- [ ] No test framework exists: smoke tests are all shell one-liners, no test runner needed for this phase
- [ ] `docs/RUNBOOK-updater-signing.md` — new file, not a test file but required before phase gate

*(No test framework installation needed — this phase is configuration changes and a CI trigger, not Rust or TypeScript code that needs unit tests.)*

---

## Sources

### Primary (HIGH confidence)

- Direct codebase analysis — `src-tauri/tauri.conf.json`, `.github/workflows/release.yml`, `.github/workflows/build.yml`, `src/components/update-checker/UpdateChecker.tsx` — all read in this session
- `.planning/research/ARCHITECTURE.md` — full data flow diagrams, component table, Ed25519 setup steps (researched 2026-04-10)
- `.planning/research/PITFALLS.md` — Pitfalls A1-A6 for auto-updater (researched 2026-04-10, codebase-grounded)
- `.planning/phases/04-updater-infrastructure/04-CONTEXT.md` — all locked decisions from Pierre (2026-04-11)

### Secondary (MEDIUM confidence)

- https://v2.tauri.app/plugin/updater/ — plugin config, endpoint format, `createUpdaterArtifacts` placement
- https://v2.tauri.app/distribute/pipelines/github/ — `tauri-apps/tauri-action` usage, `includeUpdaterJson`, release-upload flow
- https://github.com/tauri-apps/tauri-action — asset name patterns, `includeUpdaterJson` parameter

### Tertiary (LOW confidence — not individually re-verified in this session)

- Tauri issue #9565 (InvalidSignature root causes) — cited in PITFALLS.md
- Tauri issue #10316 (UnexpectedKeyId root causes) — cited in PITFALLS.md

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries confirmed present in Cargo.toml and lib.rs; no new dependencies
- Architecture: HIGH — grounded in direct file reads of all five files to be modified
- Pitfalls: HIGH — derived from existing codebase analysis (PITFALLS.md) and confirmed with current file state

**Research date:** 2026-04-11
**Valid until:** 2026-05-11 (30 days — stable Tauri v2 API)
