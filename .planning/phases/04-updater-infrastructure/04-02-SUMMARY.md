---
phase: 04-updater-infrastructure
plan: "02"
subsystem: infra
tags: [ci, github-actions, tauri, updater, asset-prefix]

# Dependency graph
requires:
  - phase: 04-updater-infrastructure
    provides: Research and context for CI workflow fixes and updater infrastructure decisions
provides:
  - CI workflows produce dictus_*-prefixed artifacts (UPDT-06, UPDT-07)
  - tauri-action generates and uploads latest.json alongside .sig files per release (UPDT-08)
  - UpdateChecker.tsx portable-update fallback links to getdictus/dictus-desktop (UPDT-09)
  - Inline TECH-03 deferral comment in build.yml protects against accidental binary rename
affects: [04-03, 04-04, CI releases, auto-updater endpoint]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Asset-prefix input propagation: release.yml passes 'dictus' to build.yml reusable workflow"
    - "includeUpdaterJson in tauri-action with-block: enables automatic latest.json generation per release"
    - "Inline deferral comment pattern: TECH-03 comment guards binary rename with explicit un-defer instructions"

key-files:
  created: []
  modified:
    - .github/workflows/release.yml
    - .github/workflows/build.yml
    - src/components/update-checker/UpdateChecker.tsx

key-decisions:
  - "TECH-03 (binary rename Contents/MacOS/handy) remains deferred post-v1.1 — protected by inline comment"
  - "includeUpdaterJson: true added to tauri-action so latest.json is auto-uploaded without extra CI steps"
  - "Portable-update fallback URL corrected to getdictus/dictus-desktop — gives users a manual reinstall path independent of the auto-updater"

patterns-established:
  - "Deferral comment pattern: insert a dated comment above deferred step explaining why + when to un-defer"

requirements-completed: [UPDT-06, UPDT-07, UPDT-08, UPDT-09]

# Metrics
duration: 1min
completed: 2026-04-11
---

# Phase 4 Plan 02: CI Asset Prefix + UpdateChecker URL Fix Summary

**CI workflows now produce `dictus_*`-prefixed artifacts with auto-generated `latest.json`, and the portable-update fallback URL points to `getdictus/dictus-desktop/releases/latest`**

## Performance

- **Duration:** ~1 min
- **Started:** 2026-04-11T14:11:01Z
- **Completed:** 2026-04-11T14:12:00Z
- **Tasks:** 2 of 2
- **Files modified:** 3

## Accomplishments

- `release.yml` asset-prefix changed from `"handy"` to `"dictus"` — CI release artifacts will be named `dictus_*` (UPDT-06)
- `build.yml` default asset-prefix changed to `"dictus"` and `includeUpdaterJson: true` added to tauri-action with-block — `latest.json` auto-generated and uploaded per release (UPDT-07, UPDT-08)
- Inline TECH-03 deferral comment inserted above `Verify macOS dylib bundling` step — prevents accidental rename of `Contents/MacOS/handy` until TECH-03 is explicitly un-deferred
- `UpdateChecker.tsx` portable-update fallback URL corrected to `https://github.com/getdictus/dictus-desktop/releases/latest` (UPDT-09)

## Task Commits

1. **Task 1: Fix asset-prefix in both workflow files + add includeUpdaterJson + TECH-03 comment** - `7105735` (feat)
2. **Task 2: Fix UpdateChecker.tsx portable-update fallback URL** - `fb30fb0` (feat)

## Files Created/Modified

- `.github/workflows/release.yml` — Asset-prefix changed from `"handy"` to `"dictus"` (line 77)
- `.github/workflows/build.yml` — Default asset-prefix changed to `"dictus"` (line 22), `includeUpdaterJson: true` added to tauri-action with-block, TECH-03 deferral comment inserted above Verify macOS dylib bundling step
- `src/components/update-checker/UpdateChecker.tsx` — Portable-update fallback URL updated from `cjpais/Handy` to `getdictus/dictus-desktop` (line 206)

## Decisions Made

- TECH-03 (binary rename `Contents/MacOS/handy` → `Contents/MacOS/dictus`) remains deferred post-v1.1 per 04-CONTEXT.md. The inline comment serves as a tripwire so future maintainers don't accidentally rename the binary before the underlying bundle ID changes are coordinated.
- `includeUpdaterJson: true` is a tauri-action input that auto-generates and uploads `latest.json` alongside the release artifacts — no separate upload step needed.

## Deviations from Plan

None - plan executed exactly as written. All four edits (two in workflow files, two more in build.yml, one in UpdateChecker.tsx) matched the exact before/after values specified in the plan interfaces block.

## Issues Encountered

- Pre-existing lint error in `DictusLogo.tsx` (hardcoded "Dictus" text in SVG, i18next/no-literal-string) — not introduced by this plan, not in scope, logged for awareness. The plan's acceptance criteria for lint applied only to UpdateChecker.tsx, which is clean.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- UPDT-06, UPDT-07, UPDT-08, UPDT-09 satisfied
- Plan 03 (keypair generation + tauri.conf.json updater config) can proceed — it depends on these CI fixes being in place
- UPDT-03, UPDT-04, UPDT-05 still fail until Plan 03 (keypair + endpoints config)

---
*Phase: 04-updater-infrastructure*
*Completed: 2026-04-11*
