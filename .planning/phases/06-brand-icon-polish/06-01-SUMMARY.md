---
phase: 06-brand-icon-polish
plan: 01
subsystem: infra
tags: [bash, verify-sync, imagemagick, brand, icon, upstream-sync]

# Dependency graph
requires:
  - phase: 05-upstream-sync
    provides: verify-sync.sh script with 11 SYNC-05* assertions
provides:
  - verify-sync.sh at permanent location .github/scripts/ with 15 assertions
  - imagemagick dep check with install hint
  - BRAND-01a/02a/03a assertions (infrastructure for Plans 06-02/03 to verify against)
  - ICON-02a assertion (icon.ico 6-layer size check, passes immediately)
affects: [06-brand-icon-polish, 06-02, 06-03, 06-04, 09-automation]

# Tech tracking
tech-stack:
  added: []
  patterns: [check() shell function for assertion-style script testing, git mv for history-preserving relocation]

key-files:
  created:
    - .github/scripts/verify-sync.sh
  modified:
    - UPSTREAM.md

key-decisions:
  - "SYNC-06 completed in Phase 6 Plan 01, ahead of its original Phase 9 schedule — script needed at .github/scripts/ before Phase 9 CI gate goes live"
  - "BRAND-01a/02a/03a intentionally fail until Plans 06-02 and 06-03 land — expected state at end of Plan 01"
  - "ICON-02a passes immediately — icon.ico already contains all 6 required layer sizes (16,24,32,48,64,256)"
  - "git mv used for file relocation to preserve Phase 5 commit history in git log --follow"

patterns-established:
  - "Pattern 1: verify-sync.sh as assertion-style bash script for identity regression testing"
  - "Pattern 2: dep checks (jq, identify) at script top with early exit and install hints"

requirements-completed: [BRAND-04, ICON-02, SYNC-06]

# Metrics
duration: 3min
completed: 2026-04-16
---

# Phase 6 Plan 01: verify-sync.sh Relocation + Brand/Icon Assertions Summary

**verify-sync.sh relocated from .planning/phases/05-upstream-sync/scripts/ to permanent home at .github/scripts/ with 15 total assertions: 11 existing SYNC-05* preserved + 4 new guards (BRAND-01a, BRAND-02a, BRAND-03a, ICON-02a) and imagemagick dep check**

## Performance

- **Duration:** 3 min
- **Started:** 2026-04-16T05:24:46Z
- **Completed:** 2026-04-16T05:27:05Z
- **Tasks:** 3
- **Files modified:** 2

## Accomplishments
- Relocated verify-sync.sh to .github/scripts/ using git mv, preserving full Phase 5 commit history
- Extended script from 11 to 15 assertions covering BRAND-01/02/03 and ICON-02 surfaces
- Added imagemagick (identify) dep check with early exit and install hint
- Updated all UPSTREAM.md path references — no stale .planning/phases/05-upstream-sync/scripts/ paths remain
- ICON-02a passes immediately (icon.ico already has all 6 required layer sizes)
- SYNC-06 completed ahead of original Phase 9 schedule

## Task Commits

Each task was committed atomically:

1. **Task 1: Relocate verify-sync.sh to .github/scripts/ (SYNC-06)** - `24d8e61` (chore)
2. **Task 2: Extend verify-sync.sh with BRAND/ICON assertions + imagemagick dep check** - `7150ff5` (feat)
3. **Task 3: Update UPSTREAM.md path references to new script location** - `d2cf229` (docs)

**Plan metadata:** (docs commit follows)

## Files Created/Modified
- `.github/scripts/verify-sync.sh` - verify-sync.sh at permanent location; 15 assertions covering SYNC-05a-k + BRAND-01a/02a/03a + ICON-02a
- `UPSTREAM.md` - path references updated from .planning/phases/05-upstream-sync/scripts/ to .github/scripts/

## Decisions Made
- SYNC-06 is complete ahead of its original Phase 9 schedule. Plans 06-02/03/04 can now invoke `bash .github/scripts/verify-sync.sh` as their per-task feedback command. After this phase ships, run `/gsd:roadmap-update` to reflect Phase 9 scope (SYNC-07/08/09/10/11 only — SYNC-06 done).
- BRAND-01a/02a/03a are expected to fail until Plans 06-02 and 06-03 complete their source changes. This is correct and desired state at end of Plan 01.
- ICON-02a passes immediately since the current icon.ico already has all 6 required layer sizes (16, 24, 32, 48, 64, 256), as indicated in the research notes (RESEARCH.md Pitfall 2 note).

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Plans 06-02, 06-03, 06-04 can now use `bash .github/scripts/verify-sync.sh` as their per-task verification command
- BRAND-01a/02a/03a checks are in place and will turn green as Plans 06-02/03 fix the handy- prefixes, portable.rs string, and DebugPaths.tsx path
- ICON-02a is already green — no icon.ico work needed for Plan 06-04 to pass this check
- Post-phase: run `/gsd:roadmap-update` to reflect SYNC-06 completion (Phase 9 SYNC scope is now SYNC-07/08/09/10/11 only)

---
*Phase: 06-brand-icon-polish*
*Completed: 2026-04-16*
