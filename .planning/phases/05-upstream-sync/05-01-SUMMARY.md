---
phase: 05-upstream-sync
plan: 01
subsystem: infra
tags: [github-actions, upstream-sync, bash, idempotency, cron]

# Dependency graph
requires:
  - phase: 04-updater-infrastructure
    provides: validate.sh check()/FAIL pattern reused for verify-sync.sh
provides:
  - .github/upstream-sha.txt seeded with merge-base 39e855d
  - .github/workflows/upstream-sync.yml weekly cron detection workflow
  - .planning/phases/05-upstream-sync/scripts/verify-sync.sh post-merge identity validator
affects:
  - 05-upstream-sync/05-03-PLAN (uses verify-sync.sh in post-merge gate)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - SHA-based idempotent detection via committed upstream-sha.txt flat file
    - actions/github-script@v7 issue creation pattern (reused from release.yml)
    - check()/FAIL bash pattern for identity assertion scripts (reused from Phase 4)

key-files:
  created:
    - .github/upstream-sha.txt
    - .github/workflows/upstream-sync.yml
    - .planning/phases/05-upstream-sync/scripts/verify-sync.sh
  modified: []

key-decisions:
  - "upstream-sha.txt updated only on merge-to-main (not on issue creation) to avoid premature idempotency (Pitfall 2)"
  - "SYNC-05d i18n check excludes acknowledgments section via awk to allow legitimate Handy attribution"
  - "Detection workflow is read-only — no auto-commit, no auto-merge, no auto-PR"
  - "upstream-sync label must be created manually in repo before first workflow run (Pitfall 3)"

patterns-established:
  - "Pattern: SHA-based idempotency — read committed file, compare to upstream HEAD, skip if equal"
  - "Pattern: check()/FAIL helper for bash assertion scripts — reused from Phase 4 validate.sh"

requirements-completed:
  - SYNC-01
  - SYNC-02
  - SYNC-05

# Metrics
duration: 30min
completed: 2026-04-14
---

# Phase 5 Plan 01: Upstream Sync Detection Infrastructure Summary

**Weekly GitHub Actions cron detecting cjpais/Handy drift via SHA-based idempotency file, plus SYNC-05 post-merge identity validator script seeded green on current codebase**

## Performance

- **Duration:** ~30 min
- **Started:** 2026-04-14T07:49:11Z
- **Completed:** 2026-04-14T08:20:00Z
- **Tasks:** 3 of 3 complete
- **Files modified:** 3 created

## Accomplishments
- `.github/upstream-sha.txt` seeded with merge-base SHA `39e855d` (8 bytes, correct format)
- `verify-sync.sh` with 10 SYNC-05 assertions (a-j) passes green on current codebase including cargo build
- `upstream-sync.yml` detection workflow with weekly cron, idempotency guard, `upstream-sync` label, and `issues: write` permission

## Task Commits

Each task was committed atomically:

1. **Task 1: Create upstream-sha.txt seed and verify-sync.sh validator** - `f1b7b96` (chore)
2. **Task 2: Create upstream-sync.yml detection workflow** - `8b76ad0` (feat)
3. **Task 3: Verify upstream-sync label exists** - COMPLETE (label created 2026-04-14 via `gh label create`)

## Files Created/Modified
- `.github/upstream-sha.txt` - Idempotency seed file; stores last-seen upstream SHA (currently merge-base `39e855d`)
- `.github/workflows/upstream-sync.yml` - Weekly cron detection action; creates `upstream-sync`-labeled issue when SHA differs
- `.planning/phases/05-upstream-sync/scripts/verify-sync.sh` - Post-merge identity validator; SYNC-05a..j assertions; must be re-run after Plan 03 merge

## Decisions Made
- `upstream-sha.txt` updated only when sync branch merges to main, not on issue creation (prevents premature idempotency — upstream drift appears resolved before work is done)
- SYNC-05d i18n check uses `awk` to skip the `acknowledgments` block before grepping for `"Handy"` — the acknowledgments section legitimately credits the upstream Handy project
- Workflow is strictly detection-only: no `git commit`, no `git push`, no SHA file write
- `github-token: ${{ secrets.GITHUB_TOKEN }}` and `permissions: issues: write` explicit to prevent 403 (Pitfall 5)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] SYNC-05d check adjusted to exclude attribution section**
- **Found during:** Task 1 (verify-sync.sh creation + test run)
- **Issue:** Plan's simpler form `! grep '"Handy"' src/i18n/locales/en/translation.json` fails on current codebase because `acknowledgments.handy.title` contains `"Handy"` as legitimate upstream attribution
- **Fix:** Replaced with `awk` pipeline that skips the acknowledgments block before grepping, matching the plan's intent ("except attribution" in the label)
- **Files modified:** `.planning/phases/05-upstream-sync/scripts/verify-sync.sh`
- **Verification:** Script now exits 0 with "All Phase 5 checks passed." on current codebase
- **Committed in:** `f1b7b96` (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - bug in simpler check form)
**Impact on plan:** Fix necessary for correctness — script designed to be green on pre-merge codebase. No scope creep.

## Issues Encountered
- `pyyaml` not available in the default Python environment; installed inline to validate YAML. YAML validated as correct.

## Label Status

**Task 3 — `upstream-sync` label: CONFIRMED PRESENT**

Label created on 2026-04-14 via:
```
gh label create upstream-sync --repo getdictus/dictus-desktop --color fbca04 --description "Tracks new upstream Handy commits detected by weekly sync action"
```

- Name: `upstream-sync` (exact, case-sensitive)
- Color: `#fbca04`
- Description: `Tracks new upstream Handy commits detected by weekly sync action`

Smoke test (optional end-to-end workflow dispatch) skipped — requires unmerged upstream commits to be present on pushed main. User will trigger manually if desired before Plan 03 merge.

## Next Phase Readiness
- Detection infrastructure complete — all 3 tasks done
- `verify-sync.sh` ready for Plan 03 post-merge gate
- `upstream-sha.txt` will be updated to current upstream HEAD as part of Plan 03 merge commit
- `upstream-sync` label confirmed in repo — workflow can run without 422 error

---
*Phase: 05-upstream-sync*
*Completed: 2026-04-14*
