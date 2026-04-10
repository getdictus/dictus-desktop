# Architecture Patterns: Auto-Updater & Upstream Sync

**Domain:** Tauri 2.x auto-update + fork upstream sync integration
**Milestone:** v1.1 Auto-Update & Upstream Sync
**Researched:** 2026-04-10
**Confidence:** HIGH (based on direct codebase analysis + official Tauri v2 docs)

---

## System Overview

```
┌───────────────────────────────────────────────────────────────────────┐
│                        GITHUB INFRASTRUCTURE                          │
│                                                                       │
│  ┌───────────────────┐   ┌─────────────────────────────────────────┐ │
│  │  cjpais/Handy     │   │  dictus-desktop/dictus-desktop          │ │
│  │  upstream remote  │──▶│  GitHub Releases                        │ │
│  │  (69 commits      │   │  ├── Dictus_0.1.1_aarch64.dmg          │ │
│  │   ahead of fork)  │   │  ├── Dictus_0.1.1_aarch64.dmg.sig      │ │
│  └───────────────────┘   │  ├── Dictus_0.1.1_x64.msi              │ │
│           ▲              │  ├── Dictus_0.1.1_x64.msi.sig          │ │
│           │              │  ├── Dictus_0.1.1.AppImage              │ │
│           │              │  ├── Dictus_0.1.1.AppImage.sig          │ │
│  weekly   │              │  └── latest.json  ◀── updater endpoint  │ │
│  cron     │              └─────────────────────────────────────────┘ │
│  workflow─┘                                                           │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │  GitHub Actions                                                  │ │
│  │  ├── release.yml         (manual: build + sign + publish)       │ │
│  │  ├── build.yml           (reusable: 7-platform matrix)          │ │
│  │  └── upstream-sync.yml   (NEW: weekly cron, detect + alert)     │ │
│  └─────────────────────────────────────────────────────────────────┘ │
└───────────────────────────────────────────────────────────────────────┘
                                    │
                    latest.json at GitHub Releases URL
                                    │
                                    ▼
┌───────────────────────────────────────────────────────────────────────┐
│                     DICTUS DESKTOP (RUNTIME)                          │
│                                                                       │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │  FRONTEND (React / TypeScript)                                 │  │
│  │                                                                │  │
│  │  UpdateChecker.tsx          UpdateChecksToggle.tsx             │  │
│  │  ├── check() [plugin-updater]  └── settings.update_checks_enabled │
│  │  ├── downloadAndInstall()                                      │  │
│  │  ├── listens "check-for-updates" event                         │  │
│  │  └── openUrl() → github.com/dictus-desktop/dictus-desktop      │  │  
│  │      (CHANGE: fix hardcoded cjpais/Handy URL)                  │  │
│  │                                                                │  │
│  │  Footer.tsx → renders UpdateChecker.tsx                        │  │
│  └────────────────────────────┬───────────────────────────────────┘  │
│                               │ IPC commands + events                 │
│  ┌────────────────────────────▼───────────────────────────────────┐  │
│  │  BACKEND (Rust / src-tauri)                                    │  │
│  │                                                                │  │
│  │  lib.rs                                                        │  │
│  │  ├── .plugin(tauri_plugin_updater::Builder::new().build())     │  │
│  │  │   (already registered — NO change needed)                  │  │
│  │  ├── trigger_update_check command (already exists)            │  │
│  │  └── tray "check_updates" handler (already exists)            │  │
│  │                                                                │  │
│  │  tauri.conf.json  (CHANGES NEEDED)                            │  │
│  │  ├── bundle.createUpdaterArtifacts: true  (was false)         │  │
│  │  └── plugins.updater.pubkey: "<Ed25519 pubkey>"               │  │
│  │      plugins.updater.endpoints: ["<github releases url>"]     │  │
│  └────────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────────┘
```

---

## Component Responsibilities

### Existing Components (Modified)

| Component | File | Current State | Required Change |
|-----------|------|---------------|-----------------|
| updater config | `src-tauri/tauri.conf.json` | `createUpdaterArtifacts: false`, `pubkey: ""`, `endpoints: []` | Set `createUpdaterArtifacts: true`, add real pubkey, add GitHub Releases endpoint URL |
| UpdateChecker.tsx | `src/components/update-checker/UpdateChecker.tsx` | Hardcoded `openUrl("https://github.com/cjpais/Handy/releases/latest")` at line 206 | Change to `https://github.com/dictus-desktop/dictus-desktop/releases/latest` |
| release.yml | `.github/workflows/release.yml` | `asset-prefix: "handy"` (wrong), missing `TAURI_SIGNING_PRIVATE_KEY` env, missing `includeUpdaterJson` | Fix asset-prefix to `"dictus"`, add signing env vars, add `includeUpdaterJson: true` |
| build.yml | `.github/workflows/build.yml` | default `asset-prefix: "handy"`, missing signing key env passthrough | Accept signing key env vars from caller |

### New Components

| Component | File | Purpose |
|-----------|------|---------|
| upstream-sync.yml | `.github/workflows/upstream-sync.yml` | Weekly cron: check for new commits on `cjpais/Handy` since fork point, open a GitHub issue or post a summary if detected |
| Ed25519 keypair | GitHub Secrets: `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Signing artifacts for verified updates — generated once, private key stored only in GitHub Secrets |

---

## Data Flow: Auto-Update Check

```
App startup
    │
    ▼
UpdateChecker.tsx useEffect
    │ settings loaded & update_checks_enabled = true
    ▼
check() from @tauri-apps/plugin-updater
    │ calls tauri_plugin_updater internally
    ▼
HTTP GET https://github.com/dictus-desktop/dictus-desktop/releases/latest/download/latest.json
    │
    ├── 200 OK → latest.json with { version, platforms.[target].url, platforms.[target].signature }
    │   │
    │   ├── version > current? → setUpdateAvailable(true)
    │   └── version <= current? → setShowUpToDate(true) [if manual check]
    │
    └── network error → console.error, no UI change
```

## Data Flow: Update Install

```
User clicks "Update Available" button
    │
    ▼
installUpdate() in UpdateChecker.tsx
    │
    ├── commands.isPortable() → true? → show portable dialog → openUrl(GitHub releases)
    │
    └── false? →
        ▼
        update.downloadAndInstall(progressCallback)
            │ streams from platforms.[target].url
            │ verifies Ed25519 signature from platforms.[target].signature
            │ against pubkey embedded in tauri.conf.json
            ▼
        Progress events → setDownloadProgress()
            │
            ▼
        Install complete → relaunch()
```

## Data Flow: Release Build + Signing

```
Developer triggers release.yml (workflow_dispatch)
    │
    ▼
create-release job
    ├── reads version from tauri.conf.json
    └── creates draft GitHub Release with tag v{version}
    │
    ▼
publish-tauri job (7-platform matrix, parallel)
    │
    ├── each platform: tauri-apps/tauri-action@v0
    │   ├── reads TAURI_SIGNING_PRIVATE_KEY from GitHub Secret
    │   ├── bun run tauri build (with createUpdaterArtifacts: true)
    │   │   └── produces: Dictus_0.1.1_aarch64.dmg + .sig file
    │   ├── uploads artifacts to draft release
    │   └── (last platform) generates latest.json from all .sig files
    │
    ▼
Developer reviews draft → publishes release
    │
    ▼
latest.json available at:
    https://github.com/dictus-desktop/dictus-desktop/releases/latest/download/latest.json
```

## Data Flow: Upstream Sync Detection

```
Weekly cron (Monday 08:00 UTC) OR workflow_dispatch
    │
    ▼
upstream-sync.yml
    ├── git fetch upstream (cjpais/Handy)
    ├── git log upstream/main..HEAD --oneline → count new upstream commits
    │
    ├── new commits found?
    │   ├── YES → create GitHub issue "Upstream sync available: N commits since v0.8.2"
    │   │         with commit list + affected files summary
    │   └── NO  → workflow exits cleanly (no noise)
    │
    └── outputs: upstream_version, commit_count, has_changes
```

---

## Integration Points with Existing Architecture

### What Does NOT Change

- `tauri_plugin_updater` is already registered in `lib.rs` line 532: `.plugin(tauri_plugin_updater::Builder::new().build())` — no Rust code changes needed
- `trigger_update_check` command exists in `lib.rs` lines 315-323 — already wired to tray "check_updates" event
- `UpdateChecker.tsx` already uses `@tauri-apps/plugin-updater`'s `check()` and `downloadAndInstall()` correctly
- `UpdateChecksToggle.tsx` already toggles `update_checks_enabled` setting via Zustand/tauri-plugin-store
- All 7 build platforms already in release matrix

### What Must Change

**tauri.conf.json** — three fields:
```json
"bundle": {
  "createUpdaterArtifacts": true  // was false
},
"plugins": {
  "updater": {
    "pubkey": "<content of ~/.tauri/dictus.key.pub>",  // was ""
    "endpoints": [
      "https://github.com/dictus-desktop/dictus-desktop/releases/latest/download/latest.json"
    ]  // was []
  }
}
```

**UpdateChecker.tsx** — line 206 only:
```typescript
// Change:
openUrl("https://github.com/cjpais/Handy/releases/latest");
// To:
openUrl("https://github.com/dictus-desktop/dictus-desktop/releases/latest");
```

**release.yml** — three changes:
1. `asset-prefix: "handy"` → `asset-prefix: "dictus"`
2. Add `includeUpdaterJson: true` to tauri-action `with:` block
3. `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` are already passed via `secrets: inherit` — no change if secrets are set in repo

**build.yml** — verify `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` are forwarded from caller env to the `tauri-apps/tauri-action@v0` step (currently they are in the `env:` block at lines 344-345 — this is correct, no change needed if secrets exist)

---

## latest.json Format

`tauri-action` generates this file and uploads it to the release. Structure:

```json
{
  "version": "0.1.1",
  "notes": "Release notes from GitHub",
  "pub_date": "2026-04-15T10:00:00Z",
  "platforms": {
    "darwin-aarch64": {
      "signature": "<content of .sig file>",
      "url": "https://github.com/dictus-desktop/dictus-desktop/releases/download/v0.1.1/Dictus_0.1.1_aarch64.dmg"
    },
    "darwin-x86_64": {
      "signature": "<content of .sig file>",
      "url": "https://github.com/dictus-desktop/dictus-desktop/releases/download/v0.1.1/Dictus_0.1.1_x64.dmg"
    },
    "linux-x86_64": {
      "signature": "<content of .sig file>",
      "url": "https://github.com/dictus-desktop/dictus-desktop/releases/download/v0.1.1/Dictus_0.1.1_amd64.AppImage"
    },
    "windows-x86_64": {
      "signature": "<content of .sig file>",
      "url": "https://github.com/dictus-desktop/dictus-desktop/releases/download/v0.1.1/Dictus_0.1.1_x64-setup.exe"
    }
  }
}
```

Platform keys follow `{os}-{arch}` where os is one of `darwin/linux/windows` and arch is `x86_64/aarch64`.

---

## Ed25519 Keypair Setup (One-Time)

```bash
# Generate keypair (run locally, once)
bunx tauri signer generate -w ~/.tauri/dictus.key
# Prompts for password (store password in password manager)
# Creates: ~/.tauri/dictus.key (private) + ~/.tauri/dictus.key.pub (public)

# Copy pubkey content to tauri.conf.json plugins.updater.pubkey
cat ~/.tauri/dictus.key.pub

# Add to GitHub repository secrets:
# TAURI_SIGNING_PRIVATE_KEY = content of ~/.tauri/dictus.key
# TAURI_SIGNING_PRIVATE_KEY_PASSWORD = the password you chose
```

**Critical constraint:** If the private key is lost, future updates cannot be signed and users will be stuck. Store the key and password in a password manager with offline backup.

---

## Upstream Sync Workflow Design

```yaml
# .github/workflows/upstream-sync.yml (new file)
name: Upstream Sync Check

on:
  schedule:
    - cron: '0 8 * * 1'  # Every Monday at 08:00 UTC
  workflow_dispatch:

jobs:
  check-upstream:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Add upstream remote
        run: |
          git remote add upstream https://github.com/cjpais/Handy.git
          git fetch upstream

      - name: Check for new upstream commits
        id: check
        run: |
          FORK_POINT="85a8ed77"
          NEW_COMMITS=$(git log ${FORK_POINT}..upstream/main --oneline | wc -l | tr -d ' ')
          LATEST_TAG=$(git describe --tags upstream/main --abbrev=0 2>/dev/null || echo "unknown")
          echo "commit_count=${NEW_COMMITS}" >> $GITHUB_OUTPUT
          echo "latest_tag=${LATEST_TAG}" >> $GITHUB_OUTPUT
          echo "has_changes=$([ $NEW_COMMITS -gt 0 ] && echo 'true' || echo 'false')" >> $GITHUB_OUTPUT

      - name: Open tracking issue if new commits found
        if: steps.check.outputs.has_changes == 'true'
        uses: actions/github-script@v7
        with:
          script: |
            // Check if open issue with same title already exists
            const title = `Upstream sync: ${${{ steps.check.outputs.commit_count }}} new commits (${${{ steps.check.outputs.latest_tag }}})`;
            const existing = await github.rest.issues.listForRepo({
              owner: context.repo.owner, repo: context.repo.repo,
              state: 'open', labels: 'upstream-sync'
            });
            if (!existing.data.some(i => i.title.startsWith('Upstream sync:'))) {
              await github.rest.issues.create({
                owner: context.repo.owner, repo: context.repo.repo,
                title, labels: ['upstream-sync'],
                body: `## Upstream Sync Available\n\n**${${{ steps.check.outputs.commit_count }}} commits** available from cjpais/Handy since fork point \`85a8ed77\`.\n\nLatest upstream tag: \`${${{ steps.check.outputs.latest_tag }}}\`\n\nReview and cherry-pick or merge as appropriate per the sync strategy in issue #1.`
              });
            }
```

---

## Build Order for v1.1

Dependency graph drives this order — each step is a prerequisite for the next:

### Step 1: Ed25519 Keypair Generation (blocker for everything)
Generate the keypair locally. Add public key to `tauri.conf.json`. Add private key and password to GitHub repository secrets. Without this, the build cannot sign artifacts and `createUpdaterArtifacts: true` will produce unsigned (and therefore rejected) updates.

**No code dependencies. Do this first.**

### Step 2: tauri.conf.json Updater Configuration
Set `createUpdaterArtifacts: true` and populate `pubkey` + `endpoints`. This is a config-only change with no frontend or backend code impact.

**Depends on:** Step 1 (pubkey content)

### Step 3: Fix UpdateChecker.tsx Hardcoded URL
Change the one hardcoded `cjpais/Handy` URL in the portable update dialog. Single-line change. No logic changes, no API changes.

**Depends on:** Nothing (independent, can be done any time)

### Step 4: Fix release.yml asset-prefix + includeUpdaterJson
Change `asset-prefix: "handy"` to `asset-prefix: "dictus"`. Add `includeUpdaterJson: true` to tauri-action invocation. This ensures assets are named `Dictus_*` instead of `handy_*` and that `latest.json` is generated and uploaded.

**Depends on:** Step 1 (secrets must exist for signing to work on first test run)

### Step 5: Test Release (dry-run)
Trigger `release.yml` manually (workflow_dispatch). Verify: artifacts are named `Dictus_*`, `.sig` files are present, `latest.json` is uploaded to the release, pubkey verification passes.

**Depends on:** Steps 1–4

### Step 6: Upstream Sync Workflow
Add `.github/workflows/upstream-sync.yml`. This is independent of the auto-updater work and can be done in parallel with steps 3–5.

**Depends on:** Nothing (independent of auto-updater)

### Step 7: Upstream Merge (v0.8.0–v0.8.2)
Fetch upstream remote, review 69 commits, cherry-pick or merge selectively. This is the highest-risk step: merge conflicts with the Handy→Dictus rebrand are expected in:
- `src-tauri/tauri.conf.json` (productName, identifier, updater config)
- `src/` files where Dictus branding was applied
- Any new features upstream added to settings, managers, or commands

**Depends on:** Step 6 (workflow already exists to detect future upstream commits after this merge)

### Step 8: Documentation
Document fork point, sync strategy, and update process. Update `CLAUDE.md` with new workflows.

**Depends on:** Steps 6–7

---

## Scalability Considerations

| Concern | Current (v1.1) | Future |
|---------|---------------|--------|
| Update endpoint | GitHub Releases direct (`latest.json`) | If rate-limited, migrate to Dictus CDN (INFR-01 deferred) |
| Code signing | Self-signed macOS (`signingIdentity: "-"`) | Full Apple Developer + Notarization (INFR-03 deferred) |
| Windows signing | Azure Trusted Signing in CI | Already configured in build.yml, requires secrets |
| Upstream sync | Weekly detection issue | Could add auto-PR creation, diff summaries, conflict prediction |
| Update channel | Single release channel | Could add beta/nightly channels via different endpoint URLs |

---

## Sources

- Tauri v2 Updater plugin docs: https://v2.tauri.app/plugin/updater/
- tauri-action GitHub Action: https://github.com/tauri-apps/tauri-action
- Tauri v2 GitHub pipeline guide: https://v2.tauri.app/distribute/pipelines/github/
- `tauri_plugin_updater` in Cargo.toml: `tauri-plugin-updater = "2.10.0"` (already present)
- Upstream sync actions: https://github.com/marketplace/actions/upstream-sync
