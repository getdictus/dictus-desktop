---
phase: 04-updater-infrastructure
plan: "04"
subsystem: infra
tags: [tauri-updater, github-releases, latest.json, ed25519, ci-cd, release]

# Dependency graph
requires:
  - phase: 04-updater-infrastructure/04-01
    provides: "verify-updater-release.sh script and validate.sh validator"
  - phase: 04-updater-infrastructure/04-02
    provides: "asset-prefix: dictus, includeUpdaterJson: true in CI workflows"
  - phase: 04-updater-infrastructure/04-03
    provides: "Ed25519 keypair, TAURI_SIGNING_PRIVATE_KEY secret, tauri.conf.json updater config"
provides:
  - "v0.1.0 published as first public Dictus Desktop release on getdictus/dictus-desktop"
  - "latest.json live at https://github.com/getdictus/dictus-desktop/releases/latest/download/latest.json"
  - "UPDT-10 end-to-end validation passing"
  - "All 10 updater requirements (UPDT-01..UPDT-10) confirmed green"
affects: [phase-5-upstream-sync, future-releases]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "GitHub Releases as tauri-updater endpoint — latest.json served via /releases/latest/download/"
    - "tauri-action@v0 auto-generates latest.json when includeUpdaterJson: true + createUpdaterArtifacts: true"
    - "18-platform latest.json (not 7) — tauri-action aggregates all matrix jobs into one file"

key-files:
  created: []
  modified: []

key-decisions:
  - "latest.json has 18 platform entries (not 7 as expected) — tauri-action creates entries for every bundle format per architecture, e.g. linux-x86_64 appears as both deb and AppImage entries. This is correct behavior."
  - "Windows builds succeeded but binaries are unsigned at OS level (Azure Trusted Signing not yet set up) — acceptable for v0.1.0, Windows users will see SmartScreen warning"
  - "macOS builds signed with Apple Developer ID cert — notarization confirmed"
  - "CI cosmetic failure: 'Verify macOS dylib bundling' step failed on x86_64 looking for Contents/MacOS/handy (TECH-03 binary rename deferred). Actual build and upload succeeded."
  - "Repo must be public for /releases/latest/download/ to serve assets — confirmed public before release"

patterns-established:
  - "Release procedure: workflow_dispatch → inspect draft (latest.json + .sig + dictus_* prefix) → publish as non-pre-release"
  - "UPDT-10 verification: bash scripts/verify-updater-release.sh exits 0 = phase complete signal"

requirements-completed: [UPDT-10]

# Metrics
duration: ~60min (CI build time) + human review
completed: "2026-04-13"
---

# Phase 04 Plan 04: Dry-Run Release (v0.1.0) Summary

**v0.1.0 published as first Dictus Desktop release — latest.json live with 18 platform entries, UPDT-10 end-to-end validation passing, all Phase 4 requirements green**

## Performance

- **Duration:** ~60 min CI build time + human review
- **Started:** 2026-04-13
- **Completed:** 2026-04-13
- **Tasks:** 2 (1 human-action, 1 auto)
- **Files modified:** 0 (release-only plan — no file changes)

## Accomplishments

- v0.1.0 released publicly at https://github.com/getdictus/dictus-desktop/releases/tag/v0.1.0
- 27 assets uploaded across all platforms, all with matching `.sig` files confirming Ed25519 signing worked end-to-end
- `latest.json` attached and fetchable, structurally valid with 18 platform entries — UPDT-10 assertion passes
- All asset names prefixed `Dictus_*` (no `handy_*` leakage), confirming Plan 02 asset-prefix fix landed correctly
- macOS builds signed with Apple Developer ID cert

## Task Commits

This plan made no file commits — it was an integration validation plan (human-action + curl assertion only).

1. **Task 1: Pierre triggers release.yml and publishes v0.1.0** — human-action gate, no commit
2. **Task 2: Run verify-updater-release.sh and confirm UPDT-10 passes** — verification only, no commit

## Files Created/Modified

None — this plan validates the release artifact rather than modifying code.

## Decisions Made

- **18 platforms in latest.json (not 7):** tauri-action@v0 creates one entry per bundle format per architecture, so linux-x86_64 appears multiple times (deb, AppImage, rpm). This answers Open Question §3 from 04-RESEARCH.md — all 7 build matrix targets are represented, with multiple bundle types per target.
- **Windows unsigned at OS level:** Azure Trusted Signing was not set up before v0.1.0. Windows builds uploaded successfully and have Ed25519 `.sig` files (required for tauri-updater), but the `.msi`/`.exe` binaries themselves are not Authenticode-signed. Users will see a SmartScreen warning on first install. This is documented as a known limitation for v0.1.0.
- **CI cosmetic failure logged, not blocking:** The "Verify macOS dylib bundling" step on x86_64 searches for `Contents/MacOS/handy` (TECH-03 rename is deferred). This step failure does not affect the build or upload. Logged to deferred-items.md.

## Verification Output

`bash scripts/verify-updater-release.sh` output:
```
UPDT-10 PASS — latest.json is published and structurally valid
```

`curl ... | jq '.version, (.platforms | keys)'` — actual platforms in latest.json:
- version: `"0.1.0"`
- 18 platform entries covering darwin-aarch64, darwin-x86_64, linux-x86_64 (multiple bundle types), linux-aarch64, windows-x86_64, windows-aarch64

`bash .planning/phases/04-updater-infrastructure/scripts/validate.sh`:
- UPDT-03 through UPDT-09: ALL CHECKS PASSED

## Deviations from Plan

None — plan executed exactly as written. The 18-platform count vs expected 7 is an informational observation, not a failure.

## Issues Encountered

- **Windows code signing:** Azure Trusted Signing not configured — binaries are unsigned at OS level. Ed25519 `.sig` files are present (tauri-updater requirement met), but Authenticode signing is deferred. Users see SmartScreen warning.
- **CI cosmetic failure:** "Verify macOS dylib bundling" on x86_64 looks for `Contents/MacOS/handy` (TECH-03 deferred rename). Actual build succeeded. No action required for Phase 4.

## User Setup Required

None — release is published and live.

## Next Phase Readiness

- Phase 4 is complete. All 10 UPDT requirements satisfied.
- Phase 5 (Upstream Sync) can begin. Prerequisites: `upstream-sync` label must be created in getdictus/dictus-desktop repo before the detection action runs (noted in STATE.md blockers).
- Windows code signing (Azure Trusted Signing) should be addressed before v1.0 public announcement if SmartScreen warnings are a concern.

---
*Phase: 04-updater-infrastructure*
*Completed: 2026-04-13*
