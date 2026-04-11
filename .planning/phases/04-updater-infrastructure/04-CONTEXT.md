# Phase 4: Updater Infrastructure - Context

**Gathered:** 2026-04-11
**Status:** Ready for planning

<domain>
## Phase Boundary

Wire the Tauri v2 auto-updater end-to-end so Dictus can sign, publish, and deliver updates to users automatically. Specifically: generate the Ed25519 signing keypair, populate `tauri.conf.json` updater config, fix the release CI (asset-prefix + `includeUpdaterJson`), fix the hardcoded `cjpais/Handy` URL in `UpdateChecker.tsx`, and validate the whole chain with a real first public release (`v0.1.0`) whose `latest.json` must be reachable at the configured endpoint.

**Out of scope for this phase:** upstream sync work (Phase 5), binary rename `handy → dictus` (deferred TECH-03), model CDN migration (deferred INFR-01), code signing Apple Developer / Azure setup (deferred INFR-03), in-app updater-failure notification UX.

</domain>

<decisions>
## Implementation Decisions

### Ed25519 keypair — generation and custody

- **Generation**: Pierre runs `bunx tauri signer generate -w ~/.tauri/dictus.key` locally on this Mac, once. Strong passphrase (distinct from Bitwarden master password).
- **Primary storage (CI signing)**: GitHub Secrets on `getdictus/dictus-desktop`:
  - `TAURI_SIGNING_PRIVATE_KEY` ← raw contents of `~/.tauri/dictus.key`
  - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` ← the passphrase
  - These are already referenced at `build.yml:354-355` — no workflow change needed to plumb them, only the repo-level secret must exist before the first signed build.
- **Primary backup (recovery)**: Bitwarden — two separate secure notes under the `getdictus` folder:
  - Note 1: "Dictus Updater Signing Key" — contents of `~/.tauri/dictus.key`
  - Note 2: "Dictus Updater Signing Passphrase" — the passphrase
  - Separation of key material and passphrase is defense in depth against partial vault compromise.
- **Secondary backup (redundancy)**: Encrypted offline file using `age -R <Pierre's YubiKey age pubkey>`, stored on an external drive / USB stick. This is the designated use for Pierre's YubiKey pair in this project — the YubiKey itself cannot be used as the Tauri signing backend (see next section).
- **After setup**: delete `~/.tauri/dictus.key` from disk once all three storage locations are confirmed working. The pubkey stays in `tauri.conf.json` (committed, non-sensitive).

### Why not YubiKey-native signing

YubiKey was evaluated and explicitly rejected as the signing backend for three reasons that downstream agents should NOT revisit:

1. Tauri updater uses **Minisign** (Ed25519), not OpenPGP/SSH. YubiKey supports Ed25519 via OpenPGP but not via Minisign.
2. Signing happens inside GitHub Actions on every release. The CI runner has no physical access to the YubiKey. Alternatives (self-hosted runner on Pierre's Mac, or cloud HSM) are disproportionate scope for a solo-maintained open source project in 2026.
3. Neither `tauri signer` nor `tauri-apps/tauri-action@v0` support HSM/YubiKey backends as of 2026. Building a custom signing wrapper would explode scope of this phase.

YubiKey IS used in this project, but for its appropriate strengths: encrypting the offline backup file (via `age`), GitHub 2FA, and optionally Git commit signing — not for Tauri update signing itself.

### Recovery plan if key is lost

- **Accepted as CRITICAL unrecoverable risk.** If the private key is ever lost after the first release ships, every installed copy is permanently stranded on its current version and can never receive an update through the updater pipeline.
- **Fallback for users**: `UpdateChecker.tsx` already opens the GitHub Releases page when the updater flow fails (line 206, to be fixed in this phase to point at `getdictus/dictus-desktop`). That path continues to work independently of the Ed25519 key, so users always have a manual-reinstall escape hatch.
- **No in-app rotation mechanism is being built in v1.1.** Documented as a known limitation in the runbook. Revisit only if a real loss event ever occurs.

### Runbook (committed documentation)

- **Single file**: `docs/RUNBOOK-updater-signing.md` (new, created in this phase).
- **Content**: pointers only, zero secrets:
  - Where the private key lives (Bitwarden entry name, age-encrypted backup location)
  - How to rotate (regenerate + update pubkey in `tauri.conf.json` + update GitHub Secrets + re-release)
  - What happens if lost (reference the Recovery plan above)
  - **"Release procedure" section** — covers the version bump + release workflow run + post-release curl assertion so this file is the single entry point for shipping any Dictus Desktop release.

### `tauri.conf.json` updater config

- `bundle.createUpdaterArtifacts` → `true` (was `false` at `tauri.conf.json:28`)
- `plugins.updater.pubkey` → raw base64 pubkey string from `~/.tauri/dictus.key.pub` (NOT PEM-wrapped)
- `plugins.updater.endpoints` → `["https://github.com/getdictus/dictus-desktop/releases/latest/download/latest.json"]`

### CI workflow fixes

- **`.github/workflows/release.yml:77`**: change `asset-prefix: "handy"` → `asset-prefix: "dictus"`
- **`.github/workflows/build.yml`**: the reusable workflow is called by `release.yml` with `asset-prefix: "dictus"` — verify no hardcoded default elsewhere needs changing, and add `includeUpdaterJson: true` to the `tauri-apps/tauri-action@v0` `with:` block around line 365 (currently absent).
- **`.github/workflows/build.yml:380`**: the `otool -L "$APP/Contents/MacOS/handy"` line stays as-is (binary is still named `handy` per deferred TECH-03). Add a prominent comment above the `Verify macOS dylib bundling` step explaining "binary name is intentionally still `handy` per deferred TECH-03, do NOT change to `dictus` until TECH-03 is un-deferred". Do NOT drop the `|| true` — keep current tolerance.

### `UpdateChecker.tsx` URL fix

- `src/components/update-checker/UpdateChecker.tsx:206`: change `https://github.com/cjpais/Handy/releases/latest` → `https://github.com/getdictus/dictus-desktop/releases/latest`.

### Version policy

- **v0.1.0 is the first real public Dictus Desktop release** — the same release that contains the updater wiring IS the validation release for UPDT-10. No pre-bump to 0.1.1 this phase.
- **After v0.1.0**: semver patch bumps for bug fixes (`0.1.0 → 0.1.1 → 0.1.2`). Minor/major bumps only for real feature or breaking work. This contradicts nothing in the existing "GSD milestones ≠ app version" memory.
- **Atomicity**: any future version bump MUST update `Cargo.toml`, `src-tauri/tauri.conf.json`, and `package.json` in the same commit. Planner MUST include a post-task grep assertion: `grep '"version"' Cargo.toml src-tauri/tauri.conf.json package.json` returns three matching values. This applies starting with any change after v0.1.0 ships.

### Dry-run release strategy (UPDT-10 validation)

- **Approach**: ship real `v0.1.0` as the first Dictus Desktop public release. This IS the dry-run. Pierre triggers `release.yml` manually via GitHub Actions `workflow_dispatch`. The workflow creates a draft release; Pierre inspects artifacts and publishes it.
- **Pre-flight cleanup**: delete stale local tags `0.1.0` and `v0.1.0-dictus` (`git tag -d 0.1.0 v0.1.0-dictus`) before triggering the release so `release.yml`'s `v0.1.0` tag creation is clean.
- **Success assertion for UPDT-10**: curl-based check, committed as either a shell snippet in the runbook or a script under `scripts/`:
  ```bash
  curl -sfL https://github.com/getdictus/dictus-desktop/releases/latest/download/latest.json \
    | jq -e '.version and .platforms and (.platforms | keys | length > 0)'
  ```
  Must return exit code 0, print a parseable JSON with non-empty `platforms` object. Anything else (404, redirect-to-login, empty JSON) fails the phase.
- **Runner**: Pierre triggers manually from the GitHub Actions UI. Not automated from a plan task — planning should NOT spend CI minutes producing a real public release as a side effect.

### Deferred TECH-03 handling this phase

- Only touch `build.yml:380` with an explanatory comment. No code change to the binary name or to the `|| true` tolerance. Keep TECH-03 fully deferred; revisit post-v1.1 as a deferred idea.

### Claude's Discretion

- Exact wording of the explanatory comment above `build.yml:380`.
- Exact layout and prose of `docs/RUNBOOK-updater-signing.md` — as long as it covers the custody, recovery, runbook, and release-procedure content captured above.
- Whether the `curl | jq` assertion lives inline in the runbook, as `scripts/verify-updater-release.sh`, or both. Planner picks what's idiomatic for this repo.
- Exact placement of `includeUpdaterJson: true` within the existing `with:` block in `build.yml`.
- Whether `cleanup of stale local tags` (step `git tag -d 0.1.0 v0.1.0-dictus`) runs as its own plan task or is folded into the release-procedure section of the runbook.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements and scope
- `.planning/REQUIREMENTS.md` §Auto-Updater — UPDT-01 through UPDT-10 acceptance criteria
- `.planning/ROADMAP.md` §"Phase 4: Updater Infrastructure" — goal, dependencies, success criteria list
- `.planning/PROJECT.md` §"Current Milestone: v1.1 Auto-Update & Upstream Sync" — milestone framing

### Research inputs (read these before planning)
- `.planning/research/SUMMARY.md` — executive summary of the whole v1.1 milestone
- `.planning/research/ARCHITECTURE.md` §"What Must Change" — exact files and fields; §"Ed25519 Keypair Setup"; §"latest.json Format"
- `.planning/research/PITFALLS.md` §"Critical Pitfalls — Auto-Updater (v1.1)" — A1 through A6, all of which apply to this phase
- `.planning/research/PITFALLS.md` §"'Looks Done But Isn't' Checklist (v1.1)" §Auto-Updater bullets
- `.planning/research/FEATURES.md` — feature table stakes, complexity ratings

### External Tauri v2 docs (official)
- https://v2.tauri.app/plugin/updater/ — plugin config, endpoint format, `createUpdaterArtifacts` placement
- https://v2.tauri.app/distribute/pipelines/github/ — `tauri-apps/tauri-action` usage, `includeUpdaterJson`, release-upload flow
- https://github.com/tauri-apps/tauri-action — asset name patterns, release ID wiring

### Files to modify (grounded paths)
- `src-tauri/tauri.conf.json` — bundle + plugins.updater config
- `.github/workflows/release.yml` — line 77 asset-prefix
- `.github/workflows/build.yml` — tauri-action block ~line 365 (add `includeUpdaterJson: true`); line ~380 explanatory comment
- `src/components/update-checker/UpdateChecker.tsx` — line 206 URL
- `docs/RUNBOOK-updater-signing.md` — NEW file to create

### No external ADRs
This project has no `docs/decisions/` ADR directory. All context and decisions live in `.planning/` and inline in CONTEXT.md + the forthcoming RUNBOOK.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `tauri-plugin-updater = "2.10.0"` is already in `Cargo.toml` — no new dependency.
- `tauri_plugin_updater::Builder::new().build()` is already registered in `src-tauri/src/lib.rs:532` — no Rust code changes needed.
- `trigger_update_check` command already exists in `lib.rs:315-323` and is already wired to the tray "check_updates" handler — no Rust wiring needed.
- `UpdateChecker.tsx` React component already implements the full UX: check, download, progress, install, up-to-date, portable fallback. Only the line 206 URL needs fixing.
- `UpdateChecksToggle.tsx` already controls the `update_checks_enabled` Zustand setting via `tauri-plugin-store`.
- `build.yml:354-355` already forwards `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` from caller env to the tauri-action step. Once the repo secrets are set, signing "just works."
- `release.yml` already uses `workflow_dispatch` only (manual trigger, no tag-push), matching the "Pierre triggers manually" decision.
- `release.yml:25-39` already creates `draft: true` releases with `generate_release_notes: true`, giving Pierre a review step before publishing.

### Established Patterns
- **Single source of truth for version**: `release.yml:21` greps `src-tauri/tauri.conf.json` for the version string. That file is authoritative; `Cargo.toml` and `package.json` must be kept in sync manually (enforced by plan grep assertion).
- **Secret forwarding shape**: all sensitive CI env vars in `build.yml` follow `inputs.sign-binaries && secrets.X || ''` so they are empty strings for PR builds and populated only for signed release builds. No change needed here.
- **Settings default**: `settings.rs` `default_update_checks_enabled()` returns `true`. v1.0 installs (none in the wild on GitHub Releases yet — confirmed via `gh release list`) would silently 404 against an empty endpoint, but since no public release has shipped yet, there are no legacy users to worry about for v1.1.

### Integration Points
- `tauri.conf.json` bundle section ↔ macOS dylib bundling step in `build.yml` (the `Contents/MacOS/handy` otool line)
- GitHub Secrets ↔ `build.yml:354-355` env passthrough ↔ `tauri-action@v0` signing step
- Published release on GitHub ↔ `UpdateChecker.tsx` check() call ↔ `plugins.updater.endpoints` URL in `tauri.conf.json`

</code_context>

<specifics>
## Specific Ideas

- Pierre owns a YubiKey pair and wants to use them where it actually makes sense. Designated role in this phase: encrypt the offline backup of the private key file with `age -R <yubikey_age_pubkey>`. Not the Tauri signing backend.
- Pierre has Bitwarden as his password manager — use it as the primary backup location (two separate notes for key material + passphrase).
- Pierre has never set up a Tauri updater before — runbook prose should be slightly more verbose than "experts only" shorthand, so future-Pierre can follow it in six months without re-researching.
- Previous experience generating Apple iOS signing keys for `dictus-ios` exists — do NOT attempt to reuse those; they are RSA App Store certs, incompatible with Minisign Ed25519.

</specifics>

<deferred>
## Deferred Ideas

- **In-app "broken updater" notification UI** — detect repeated update-check failures, surface a non-dismissible banner telling the user to manually reinstall. Not built in v1.1. Revisit if a key-loss event ever occurs.
- **Full TECH-03 binary rename (`handy → dictus`)** — stays deferred. Revisit after v1.1 ships. The `build.yml:380` otool line is a tripwire to remember this.
- **Dropping `|| true` on the macOS dylib verify step** — worth doing for stricter CI, but out of scope for this phase.
- **`scripts/bump-version.sh` atomic version-bump helper** — considered but rejected. For v1.1 the plan task with grep assertion is enough. Revisit if version bumps become frequent.
- **Notarization / Apple Developer code signing (INFR-03)** — deferred milestone. Interacts with the updater because notarized dmg's ship alongside `.sig` files, but the current `signingIdentity: "-"` (ad-hoc) is acceptable for v1.1.
- **Model CDN migration away from `blob.handy.computer` (INFR-01)** — unrelated to updater, tracked separately.
- **Tag-push trigger on `release.yml`** — stays manual `workflow_dispatch` for v1.1. Could automate later once the process is battle-tested.
- **Release notes template / changelog discipline** — `generate_release_notes: true` is sufficient for now; formalize only if it becomes a pain point.

</deferred>

---

*Phase: 04-updater-infrastructure*
*Context gathered: 2026-04-11*
