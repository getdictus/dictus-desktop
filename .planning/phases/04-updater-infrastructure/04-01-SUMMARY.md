---
phase: 04-updater-infrastructure
plan: 01
subsystem: infra
tags: [tauri, updater, ed25519, signing, ci, shell, runbook]

# Dependency graph
requires: []
provides:
  - docs/RUNBOOK-updater-signing.md — canonical signing key custody, rotation, and release procedure
  - .planning/phases/04-updater-infrastructure/scripts/validate.sh — config assertion feedback loop for Plans 02 and 03
  - scripts/verify-updater-release.sh — UPDT-10 post-release curl+jq assertion wrapper
affects:
  - 04-02-CI-and-URL-fixes (uses validate.sh as per-commit feedback loop)
  - 04-03-keypair-and-config (references RUNBOOK for generation procedure; validate.sh is its acceptance check)
  - 04-04-release (references verify-updater-release.sh as UPDT-10 check)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Config-grep validation: all UPDT-0x assertions are shell one-liners in a single validate.sh script"
    - "Wave 0 infrastructure: validators and runbooks created before any code changes so downstream plans have a feedback loop from their first commit"

key-files:
  created:
    - docs/RUNBOOK-updater-signing.md
    - .planning/phases/04-updater-infrastructure/scripts/validate.sh
    - scripts/verify-updater-release.sh
  modified: []

key-decisions:
  - "Wave 0 pattern: validate.sh is intentionally expected to FAIL against the current codebase; Plans 02 and 03 make it green"
  - "validate.sh TECH-03 anti-regression check uses simple grep -B5 form (no awk) for debuggability"
  - "verify-updater-release.sh is NOT run during Plan 01 — endpoint 404s until Plan 04 publishes the release"

patterns-established:
  - "Runbook-first: documentation that Pierre must follow to safely execute risky one-time operations is committed before the operations themselves"
  - "Shell validator as phase feedback loop: all config assertions live in one script with clear PASS/FAIL labels"

requirements-completed: [UPDT-01, UPDT-02]

# Metrics
duration: 15min
completed: 2026-04-11
---

# Phase 4 Plan 01: Updater Infrastructure Wave 0 — Runbook and Validators Summary

**Ed25519 signing runbook with 7-section custody/release procedure, config assertion shell validator covering UPDT-03 through UPDT-09, and UPDT-10 curl+jq release verification wrapper**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-04-11T14:11:00Z
- **Completed:** 2026-04-11T14:26:00Z
- **Tasks:** 3
- **Files created:** 3

## Accomplishments
- Created `docs/RUNBOOK-updater-signing.md` — the single entry point for all future Dictus Desktop releases, covering Ed25519 key generation, 3-location custody, recovery plan, rotation procedure, and step-by-step release procedure with pre-flight, artifact verification, and post-release assertion
- Created `.planning/phases/04-updater-infrastructure/scripts/validate.sh` — shell-based config assertion script covering UPDT-03 through UPDT-09 plus a TECH-03 anti-regression check and version consistency check; serves as the per-commit feedback loop for Plans 02 and 03
- Created `scripts/verify-updater-release.sh` — UPDT-10 post-release assertion wrapper implementing the exact `curl -sfL ... | jq -e '.version and .platforms ...'` command from 04-CONTEXT.md

## Requirement Coverage

| Requirement | Covered By | Type |
|-------------|-----------|------|
| UPDT-01 (keypair generation) | RUNBOOK §3 — step-by-step generation + backup procedure | Manual/runbook |
| UPDT-02 (GitHub Secrets) | RUNBOOK §2 — exact secret names, UI link | Manual/runbook |
| UPDT-03 (pubkey in tauri.conf.json) | validate.sh — jq base64 length assertion | Automated config-grep |
| UPDT-04 (endpoints in tauri.conf.json) | validate.sh — jq URL test assertion | Automated config-grep |
| UPDT-05 (createUpdaterArtifacts true) | validate.sh — jq boolean equality | Automated config-grep |
| UPDT-06 (release.yml asset-prefix) | validate.sh — grep assertion | Automated config-grep |
| UPDT-07 (build.yml default asset-prefix) | validate.sh — grep assertion | Automated config-grep |
| UPDT-08 (includeUpdaterJson in build.yml) | validate.sh — grep assertion | Automated config-grep |
| UPDT-09 (UpdateChecker.tsx URL) | validate.sh — grep + negation assertion | Automated config-grep |
| UPDT-10 (curl assertion on published release) | verify-updater-release.sh | Manual trigger by Pierre |

## Validator Expected State

`bash .planning/phases/04-updater-infrastructure/scripts/validate.sh` is **expected to FAIL** against the current codebase. This is correct and intentional:

- UPDT-03: pubkey is still `""` (Plan 03 populates this)
- UPDT-04: endpoints is still `[]` (Plan 03 populates this)
- UPDT-05: createUpdaterArtifacts is still `false` (Plan 03 changes this)
- UPDT-06: release.yml still has `asset-prefix: "handy"` (Plan 02 changes this)
- UPDT-07: build.yml still has `default: "handy"` (Plan 02 changes this)
- UPDT-08: includeUpdaterJson absent from build.yml (Plan 02 adds this)
- UPDT-09: UpdateChecker.tsx still has `cjpais/Handy` URL (Plan 02 changes this)
- TECH-03: TECH-03 comment not yet added to build.yml (Plan 02 adds this)

The validator becomes green after Plans 02 and 03 complete.

## Task Commits

1. **Task 1: Create RUNBOOK-updater-signing.md** - `2b22785` (docs)
2. **Task 2: Create validate.sh** - `28f8255` (chore)
3. **Task 3: Create verify-updater-release.sh** - `00e4916` (chore)

## Files Created/Modified
- `docs/RUNBOOK-updater-signing.md` — 7-section runbook: Overview (Ed25519 + YubiKey rationale), Key Custody, First-Time Keypair Generation, Recovery Plan, Key Rotation, Release Procedure (§6.1–6.6), Common Pitfalls
- `.planning/phases/04-updater-infrastructure/scripts/validate.sh` — Phase 4 config assertion script with UPDT-03 through UPDT-09 checks, TECH-03 anti-regression, version consistency check
- `scripts/verify-updater-release.sh` — UPDT-10 standalone curl+jq assertion; sibling to existing `check-nix-deps.ts` and `check-translations.ts`

## Decisions Made
- validate.sh TECH-03 check uses `grep -B5 'otool -L' ... | grep -q 'TECH-03'` (simple grep form, no awk) for easy debuggability as specified in the plan
- verify-updater-release.sh is not run during this plan — endpoint is expected to 404 until Plan 04 publishes the first release
- RUNBOOK prose is intentionally verbose (future-Pierre has never set up a Tauri updater before)

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required in this plan. The RUNBOOK documents what Pierre must do manually (key generation, Bitwarden backup, GitHub Secrets setup), but those are Plan 03 and Plan 04 operations.

## Next Phase Readiness

- Plans 02 and 03 can now run `bash .planning/phases/04-updater-infrastructure/scripts/validate.sh` as their per-commit feedback loop
- Plan 04 can reference `bash scripts/verify-updater-release.sh` as the UPDT-10 post-release check
- RUNBOOK §3 is the instructions for Plan 03 keypair generation
- All three files pass `bash -n` syntax validation

---
*Phase: 04-updater-infrastructure*
*Completed: 2026-04-11*
