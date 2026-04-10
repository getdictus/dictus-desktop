# Research Summary: Dictus Desktop v1.1

**Milestone:** v1.1 Auto-Update & Upstream Sync
**Researched:** 2026-04-10
**Overall Confidence:** HIGH

## Executive Summary

v1.1 is almost entirely a configuration and CI wiring milestone, not a feature development milestone. The Tauri v2 updater plugin is already installed (`tauri-plugin-updater 2.10.0`), the React update UI (`UpdateChecker.tsx`) is already built with all UX states, and the plugin is already registered in `lib.rs`. What's missing: an Ed25519 keypair, three config values in `tauri.conf.json`, two workflow fixes in `release.yml`, and one URL fix in `UpdateChecker.tsx`.

The upstream merge scope is **significantly smaller than originally thought**. The 69-commit figure refers to total divergence from fork point `85a8ed77`; the actual outstanding delta is **4 commits** post-v0.8.2 (`git merge-base` is `39e855d`).

## Stack Additions

- **Zero new dependencies.** All tooling already installed.
- Keypair: `bunx tauri signer generate` (not OpenSSL or minisign — incompatible format)
- `tauri-action@v0` (v0.6.2) already in `build.yml`
- Upstream sync: native `git log` bash, no marketplace action needed

## Feature Table Stakes (P1)

| Feature | Complexity | Notes |
|---------|------------|-------|
| Ed25519 keypair + GitHub Secrets | LOW | `TAURI_SIGNING_PRIVATE_KEY` / `PASSWORD` already wired in build.yml |
| `tauri.conf.json` config (3 values) | LOW | `createUpdaterArtifacts`, `pubkey`, `endpoints` |
| `release.yml` fixes | LOW | `asset-prefix: "dictus"`, `includeUpdaterJson: true` |
| `UpdateChecker.tsx` line 206 URL | LOW | Single-line fix |
| Weekly upstream detection action | LOW | New `.github/workflows/upstream-sync.yml` |
| First upstream merge (4 commits) | MEDIUM | Dedicated branch, conflict zones: tauri.conf.json, i18n |
| `UPSTREAM.md` documentation | LOW | Fork point, merge strategy |

## Watch Out For

1. **Ed25519 private key loss is permanent** — back up to password manager + offline BEFORE first release. No recovery path.
2. **`tauri.conf.json` identity overwritten in merge** — `productName`, `identifier`, `version` must stay Dictus. Post-merge grep assertion mandatory.
3. **`createUpdaterArtifacts` placement** — must be in `bundle:` section, not `plugins.updater:`.
4. **Two locations for asset-prefix** — `release.yml` AND `build.yml` both have `"handy"`.
5. **v1.0 installs already hitting empty endpoint** — `default_update_checks_enabled()` returns `true`, silently failing on every launch.
6. **`Cargo.lock` after merge** — always regenerate with `cargo generate-lockfile`.

## Suggested Phase Structure

**Phase 1: Updater Infrastructure** — keypair, config, CI fixes, URL fix, dry-run release validation
**Phase 2: Upstream Sync** — detection action, UPSTREAM.md, 4-commit merge, post-merge checklist

## Open Questions

- Repo must be public for GitHub Releases endpoint — confirm before first release
- `upstream-sync` label must be created in repo before detection action runs
- ARM platform keys in `latest.json` need validation on first test run

---
*Synthesized: 2026-04-10 from STACK.md, FEATURES.md, ARCHITECTURE.md, PITFALLS.md*
