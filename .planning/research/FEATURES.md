# Feature Landscape

**Domain:** Desktop app auto-updater + fork upstream sync (Dictus Desktop v1.1)
**Researched:** 2026-04-10
**Confidence:** HIGH (Tauri v2 updater is a first-party plugin with official docs; upstream sync patterns are GitHub-native and well-established)

---

## Table Stakes

Features users and maintainers expect. Missing any of these means the milestone is incomplete.

### Auto-Updater Table Stakes

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Ed25519 keypair generated and stored as repo secret | Required by Tauri v2 updater; no keypair = no signed updates = updater cannot function | LOW | `npm run tauri signer generate -- -w ~/.tauri/dictus.key`; pubkey goes in `tauri.conf.json`, private key as `TAURI_SIGNING_PRIVATE_KEY` secret |
| `tauri.conf.json` updater endpoint pointing to Dictus releases | Currently `pubkey: ""` and `endpoints: []`; updater silently does nothing without this | LOW | Set `pubkey` to generated pubkey content; set `endpoints` to `https://github.com/getdictus/dictus-desktop/releases/latest/download/latest.json` |
| `createUpdaterArtifacts: true` in bundle config | Currently `false`; without this, the build does not produce `.sig` files and `latest.json` — the update payload does not exist | LOW | Single field change in `tauri.conf.json`; confirmed required for tauri-plugin-updater 2.x |
| Release workflow `asset-prefix` fixed to "dictus" | Currently hardcoded `"handy"` in `release.yml`; uploaded artifacts will be named `handy_*` which breaks the `latest.json` URL references | LOW | One-line change in `release.yml`; affects all 7 platform jobs via the shared `build.yml` input |
| `tauri-action` configured with `includeUpdaterJson: true` | tauri-action must be told to generate and upload `latest.json` to the GitHub Release; without it no update manifest exists | LOW | Add `includeUpdaterJson: true` to the `tauri-apps/tauri-action@v0` step in `build.yml` |
| UpdateChecker.tsx portable-update dialog fixed | Currently hardcodes `https://github.com/cjpais/Handy/releases/latest` — users on Windows portable builds are sent to the wrong project | LOW | One-line URL change; already identified in milestone scope |
| Manual "Check for Updates" accessible to user | User must be able to trigger an update check from settings/footer — the mechanism already exists in UpdateChecker.tsx but only works if endpoint is configured | LOW | Zero new code; fixing the config makes this work automatically |
| Update check respects `update_checks_enabled` setting | Already implemented in UpdateChecker.tsx; must remain gated by the user setting so opt-out is honored | LOW | Already correct; just verify the setting key matches settings.rs |

### Upstream Sync Table Stakes

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Weekly GitHub Action to detect new upstream commits | Without automated detection, maintainer will miss upstream releases; 69 commits already accumulated before detection | LOW | Schedule cron; compare `upstream/main` HEAD vs tracked merge base |
| Upstream detection creates a GitHub Issue or PR | Passive notification (workflow summary only) is easy to miss; an issue/PR ensures the upstream delta is visible and tracked | LOW | Use `gh issue create` or `peter-evans/create-pull-request` action |
| First upstream merge: v0.8.0–v0.8.2 (69 commits) | The fork is 69 commits behind; shipping v1.1 without merging these means accumulated drift and growing conflict risk | HIGH | Highest complexity in the milestone; git rebase or cherry-pick onto a dedicated branch; conflict resolution required |
| Fork point documented (commit `85a8ed77`) | Future merges require knowing the divergence point; undocumented fork point means every future sync starts from ambiguity | LOW | One-time write to `UPSTREAM.md` or equivalent; already referenced in PROJECT.md |
| Merge strategy documented (rebase vs merge vs cherry-pick) | Without a documented strategy, each sync risks inconsistent history and repeated conflict resolution | LOW | Write the strategy once; reference it in the weekly action output |

---

## Differentiators

Features that go beyond minimum-viable and improve maintainability or user experience meaningfully.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Silent background update check on startup (non-blocking) | Users do not notice update checks unless an update is found; less disruptive than blocking-modal approach | LOW | Already implemented in UpdateChecker.tsx (async, non-blocking); works once endpoint is live |
| Download progress bar during update install | Users know the app has not frozen; 3-second silence at 0% progress is anxiety-inducing for large binaries | LOW | Already implemented in UpdateChecker.tsx with `ProgressBar`; works once endpoint is live |
| "Up to date" confirmation with 3-second auto-dismiss | Users who manually check want confirmation; persistent "up to date" is visual noise | LOW | Already implemented (3s `setTimeout` in UpdateChecker.tsx); works once endpoint is live |
| Windows NSIS `installMode: passive` | Windows updates install silently with a progress bar rather than prompting the user for confirmation each time | LOW | Add `"windows": { "installMode": "passive" }` to updater plugin config in `tauri.conf.json` |
| Upstream diff summary in Issue body | Weekly action posts a one-liner count of new commits and top 5 commit messages; maintainer can assess merge urgency without opening GitHub manually | LOW | `git log --oneline upstream/main ^<tracked-base> | head -5` in the action |
| Labeled upstream issue (`upstream-sync`, `maintenance`) | Issues without labels get lost in triage; a consistent label makes it easy to query all sync history | LOW | One `labels:` field in the issue creation step |
| Idempotent weekly action (no duplicate issues) | If no upstream changes exist, the action does nothing; if an open issue already exists, it does not create a second one | MEDIUM | Query open issues by label before creating; skip if found |

---

## Anti-Features

Features to explicitly NOT build in this milestone.

| Anti-Feature | Why It Seems Reasonable | Why to Avoid | What to Do Instead |
|--------------|------------------------|--------------|-------------------|
| Auto-merge upstream without human review | Reduces manual work | Dictus has 52 commits of divergence (branding, config, workflow changes); an auto-merge would silently overwrite Dictus identity with Handy defaults, breaking the rebrand | Always open a PR for review; never push upstream changes directly to main |
| Built-in update dialog/modal (replacing UpdateChecker.tsx) | Tauri v2 has a built-in dialog API for updates | The existing custom UpdateChecker.tsx already handles all UX states (checking, progress, up-to-date, available) with i18n in 20 locales; the built-in dialog is English-only with no customization | Keep UpdateChecker.tsx; just fix the endpoint and Handy URL reference |
| Delta/patch updates | Reduces download size for users | Not natively supported by tauri-plugin-updater v2; requires custom update server (CrabNebula Cloud or similar); the full installer approach is simpler and sufficient at this scale | Ship full installers; revisit if binary size becomes a user complaint |
| Code-signing on macOS/Windows in this milestone | Avoids Gatekeeper / SmartScreen warnings | Code signing requires Apple Developer Program enrollment (paid), Azure Trusted Signing (paid), or notarization pipeline work — scope is separate from updater mechanics; already deferred as INFR-03 | `signingIdentity: "-"` (ad-hoc) on macOS stays; Windows signing deferred to V2/INFR-03 |
| Semantic changelog auto-generation | Nice release notes in update dialog | Requires conventional commit tooling setup and a changelog template; not worth the setup time in this milestone | GitHub's built-in `generate_release_notes: true` in `release.yml` already provides this at zero cost |
| Automated upstream cherry-pick for specific commits only | Lets you merge just the bug fixes without the new features | Requires manual commit curation, high conflict risk, and no tooling supports it reliably; cherry-pick approach fragments history and makes future merges harder | Full branch merge/rebase per release; use feature flags or reverts if specific Handy features are unwanted |
| Self-hosted update server (e.g. CrabNebula Cloud) | Avoids GitHub rate limits, adds analytics | Zero evidence of GitHub rate limit issues at current scale; adds infra dependency, cost, and operational burden with no benefit at this stage | GitHub Releases + `latest.json` is the standard Tauri community pattern and is sufficient |

---

## Feature Dependencies

```
[Ed25519 keypair generated]
    └──required by──> [tauri.conf.json pubkey field]
    └──required by──> [TAURI_SIGNING_PRIVATE_KEY repo secret]
                           └──required by──> [build.yml signs artifacts]
                                                └──required by──> [latest.json contains valid signatures]
                                                                        └──required by──> [UpdateChecker.tsx can verify and install]

[createUpdaterArtifacts: true]
    └──required by──> [.sig files exist in GitHub Release]
    └──required by──> [latest.json references valid artifact URLs]

[asset-prefix: "dictus" in release.yml]
    └──required by──> [artifact filenames match latest.json URLs]
    └──required by──> [UpdateChecker.tsx portable dialog shows correct URL]

[includeUpdaterJson: true in tauri-action]
    └──required by──> [latest.json uploaded to GitHub Release]
    └──required by──> [updater endpoint returns valid manifest]

[All four above complete]
    └──enables──> [Auto-update end-to-end functional for users]

[Fork point documented (85a8ed77)]
    └──required by──> [First upstream merge (v0.8.0-v0.8.2)]
    └──required by──> [Weekly detection action knows the tracked base]

[Weekly detection action]
    └──independent of──> [auto-updater config] (can ship in either order)
    └──feeds into──> [First upstream merge workflow (human-triggered)]

[First upstream merge]
    └──depends on──> [Fork point documented]
    └──depends on──> [Merge strategy documented]
    └──risk of conflict with──> [Any Dictus-specific files: tauri.conf.json, release.yml, icons, i18n locales, UpdateChecker.tsx]
```

### Dependency Notes

- **Keypair → everything:** The keypair is the critical path gate for the entire auto-updater feature. It must be generated first, stored as a secret, and committed to `tauri.conf.json` before any other updater work can be verified end-to-end.
- **`createUpdaterArtifacts` and `asset-prefix` are build-time:** These changes only take effect on the next release build. They cannot be tested without running a full release workflow.
- **UpdateChecker.tsx fixes are decoupled:** The portable dialog URL fix and any future cosmetic changes to UpdateChecker.tsx are independent of the build pipeline changes. They can ship in any order.
- **Upstream merge is highest-risk:** The 69 upstream commits touch files that Dictus has already modified (tauri.conf.json, release.yml, i18n locales, possibly components). Conflicts are expected. The merge must happen on a dedicated branch with careful review before merging to main.
- **Weekly action is low-risk and independent:** It reads remote state and opens issues. It does not modify the codebase. It can be shipped in a standalone PR with no dependency on the updater config work.

---

## MVP Definition

### Must Ship for v1.1

Minimum for the milestone goals to be met: "the app can update itself and we don't fall further behind upstream."

- [ ] Ed25519 keypair generated; `TAURI_SIGNING_PRIVATE_KEY` stored as GitHub secret
- [ ] `tauri.conf.json`: `pubkey` populated, `endpoints` set to Dictus GitHub Releases `latest.json` URL, `createUpdaterArtifacts: true`
- [ ] `release.yml`: `asset-prefix` changed from `"handy"` to `"dictus"`
- [ ] `build.yml`: `includeUpdaterJson: true` added to tauri-action step
- [ ] UpdateChecker.tsx portable dialog URL changed from `cjpais/Handy` to `getdictus/dictus-desktop`
- [ ] First release cut with the above changes to validate the pipeline end-to-end
- [ ] First upstream merge (v0.8.0–v0.8.2) completed on a feature branch, reviewed, merged
- [ ] Weekly upstream detection GitHub Action live (scheduled + manual dispatch)
- [ ] `UPSTREAM.md` written: fork point, merge history, strategy

### Add After Core Ships

- [ ] Windows `installMode: passive` in updater config (low friction, can go in same PR as config changes)
- [ ] Idempotent check in weekly action (skip issue creation if open issue already exists)
- [ ] Upstream diff summary in issue body (commit count + top 5 commit titles)

### Future / Deferred

- [ ] macOS/Windows code-signing for Gatekeeper/SmartScreen bypass (INFR-03, V2)
- [ ] Delta/patch updates (requires self-hosted server, no current need)
- [ ] CDN migration for models away from `blob.handy.computer` (INFR-01, V2)

---

## Feature Prioritization Matrix

| Feature | User Value | Maintainer Value | Implementation Cost | Priority |
|---------|-----------|-----------------|--------------------:|---------|
| Ed25519 keypair + tauri.conf.json config | HIGH (enables all updates) | HIGH | LOW | P1 |
| `createUpdaterArtifacts: true` | HIGH (without it, no update payload) | HIGH | LOW | P1 |
| `asset-prefix: "dictus"` in release.yml | HIGH (broken otherwise) | HIGH | LOW | P1 |
| `includeUpdaterJson: true` in build.yml | HIGH (no manifest without it) | HIGH | LOW | P1 |
| UpdateChecker.tsx portable URL fix | MEDIUM (affects portable users) | MEDIUM | LOW | P1 |
| First upstream merge v0.8.0-v0.8.2 | MEDIUM (bug fixes, new features) | HIGH (reduces drift) | HIGH | P1 |
| Weekly upstream detection action | LOW (invisible to users) | HIGH (prevents future 69-commit drift) | LOW | P1 |
| `UPSTREAM.md` fork documentation | LOW (users) | HIGH (maintainers) | LOW | P1 |
| Windows `installMode: passive` | MEDIUM (Windows UX) | LOW | LOW | P2 |
| Idempotent weekly action | LOW | MEDIUM | MEDIUM | P2 |
| Upstream issue diff summary | LOW | MEDIUM | LOW | P2 |

**Priority key:**
- P1: Required for milestone to be complete
- P2: Meaningfully improves quality; ship if time allows in same milestone
- P3: Nice to have; background work, no urgency

---

## Known Conflict Zones for Upstream Merge

The upstream v0.8.0–v0.8.2 merge will encounter conflicts in files Dictus has intentionally modified. These are not bugs — they require deliberate resolution.

| File | Dictus Change | Upstream Likely Change | Resolution Strategy |
|------|--------------|----------------------|---------------------|
| `src-tauri/tauri.conf.json` | `productName: "Dictus"`, `identifier: "com.dictus.desktop"`, `signingIdentity: "-"`, updater config | Version bump, possible new plugin config | Keep Dictus values; apply upstream config structure changes |
| `.github/workflows/release.yml` | `asset-prefix: "handy"` (to be fixed to "dictus") | Possible runner changes, new signing steps | Keep Dictus asset-prefix; apply upstream CI improvements |
| `.github/workflows/build.yml` | Minor Dictus additions | Possible new build matrix entries, ONNX updates | Merge carefully; upstream build improvements are valuable |
| `src/i18n/locales/*/translation.json` | All strings rebranded Handy→Dictus | New translation keys from new features | Add upstream new keys; do not revert Dictus brand strings |
| `src/components/update-checker/UpdateChecker.tsx` | Will have Handy URL fixed | Possible upstream UX changes | Keep Dictus URL; evaluate upstream UX changes individually |
| `src-tauri/src-tauri/Cargo.toml` | `tauri-plugin-updater = "2.10.0"` (existing) | Possible version bumps | Take upstream version bumps |

---

## Sources

- [Tauri v2 Updater Plugin documentation](https://v2.tauri.app/plugin/updater/) — official, HIGH confidence
- [Tauri v2 GitHub Releases pipeline](https://v2.tauri.app/distribute/pipelines/github/) — official, HIGH confidence
- [tauri-action v0.5.24 release notes — `includeUpdaterJson`](https://github.com/tauri-apps/tauri-action/releases) — HIGH confidence
- [Tauri updater with GitHub Releases — community guide](https://thatgurjot.com/til/tauri-auto-updater/) — MEDIUM confidence
- [aormsby/Fork-Sync-With-Upstream-action](https://github.com/aormsby/Fork-Sync-With-Upstream-action) — MEDIUM confidence
- [GitHub Actions fork sync best practices discussion](https://github.com/orgs/community/discussions/153608) — MEDIUM confidence
- Verified directly in codebase: `src-tauri/tauri.conf.json`, `.github/workflows/release.yml`, `.github/workflows/build.yml`, `src/components/update-checker/UpdateChecker.tsx`, `src-tauri/Cargo.toml`

---

_Feature research for: Dictus Desktop v1.1 — Auto-Update & Upstream Sync_
_Researched: 2026-04-10_
