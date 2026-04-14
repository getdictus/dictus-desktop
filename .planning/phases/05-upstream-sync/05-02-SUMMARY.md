---
phase: 05-upstream-sync
plan: 02
subsystem: infra
tags: [git, upstream-sync, runbook, documentation, fork, merge, tauri]

# Dependency graph
requires:
  - phase: 05-upstream-sync
    provides: Research (05-RESEARCH.md) with conflict zones, pitfalls, and delta analysis

provides:
  - UPSTREAM.md at repo root — copy-paste merge runbook documenting fork point, merge-base, step-by-step merge process, conflict resolution rules, and post-merge checklist

affects:
  - 05-03 (upstream merge execution follows this runbook)
  - future upstream sync cycles (Phase 6 AI pipeline will reference or supersede)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Upstream sync runbook pattern: copy-paste commands + explicit conflict rules per hot zone"
    - "Anti-patterns table: document what NOT to do alongside what to do"

key-files:
  created:
    - UPSTREAM.md
  modified: []

key-decisions:
  - "UPSTREAM.md placed at repo root (not docs/) for maximum visibility alongside README.md"
  - "All 4 post-v0.8.2 upstream SHAs listed in quick-reference table so the merge executor can validate the expected delta"
  - "Anti-patterns table uses three columns (anti-pattern, why wrong, what to do instead) for scan-ability"
  - "Section 4.4 explicitly notes that attribution text referencing Handy is intentional and must not be removed during i18n conflict resolution"

patterns-established:
  - "Pattern 1: Conflict resolution ordered by criticality — identity fields first, HTTP headers second, i18n third"
  - "Pattern 2: Verify-as-you-go bash commands embedded directly in each resolution section (no wait-until-end approach)"
  - "Pattern 3: Anti-patterns section mirrors RUNBOOK-updater-signing.md §Common Pitfalls structure"

requirements-completed: [SYNC-03]

# Metrics
duration: 1min
completed: 2026-04-14
---

# Phase 05 Plan 02: Upstream Sync Runbook Summary

**Copy-paste UPSTREAM.md runbook codifying fork point 85a8ed77, merge-base 39e855d, 9 conflict zone resolution rules, Pitfalls 1/2/6, and verify-sync.sh gate**

## Performance

- **Duration:** ~1 min
- **Started:** 2026-04-14T07:53:37Z
- **Completed:** 2026-04-14T07:54:43Z
- **Tasks:** 1 of 1
- **Files modified:** 1

## Accomplishments

- Created `UPSTREAM.md` (279 lines) at repo root — merge runbook future-Pierre and future AI agents can follow without re-researching
- Documents all 9 conflict hot zones with explicit Dictus-preserving resolution rules and inline verify commands
- Codifies Pitfall 1 (llm_client.rs HTTP header regression), Pitfall 2 (upstream-sha.txt timing), and Pitfall 6 (handy.log comment) from RESEARCH.md
- Anti-patterns section warns against cherry-pick, `git checkout --theirs tauri.conf.json`, manual Cargo.lock edits, and premature sha.txt updates

## Task Commits

1. **Task 1: Write UPSTREAM.md at repo root** - `0c8bfd3` (docs)

**Plan metadata:** _(to be added by final commit)_

## Files Created/Modified

- `/Users/pierreviviere/dev/dictus-desktop/UPSTREAM.md` — 279-line copy-paste merge runbook at repo root

## Decisions Made

- UPSTREAM.md placed at repo root (not `docs/`) for high visibility alongside README.md, matching the locked CONTEXT.md decision.
- i18n conflict section explicitly calls out that attribution text referencing Handy is intentional and must survive the "no Handy values" check — prevents false conflict during future merges.
- Quick Reference table added at end of document to allow scan-first reading without committing to the full runbook prose.

## Deviations from Plan

None — plan executed exactly as written. The template content from the plan's `<action>` block was used verbatim and expanded into full prose.

**UPSTREAM.md final line count:** 279 (minimum required: 120)

**Key sections included:**
- Fork Point and Current State (fork point SHA, merge-base SHA, upstream-sha.txt as source of truth, Pitfall 2 warning)
- Prerequisites (upstream remote, jq, clean tree, main pulled)
- Step-by-Step Merge Process (7 numbered steps: fetch/inspect, branch, merge, conflict resolution, finalize, verify, PR)
- Conflict Resolution (9 subsections: tauri.conf.json, llm_client.rs, actions.rs, i18n locales, App.tsx, events.ts, Cargo.lock, flake.nix, README.md)
- Anti-Patterns table (6 anti-patterns with rationale and correct alternative)
- Future: Phase 6 Automation
- Quick Reference table (all 9 hot zones with rule and risk level)

**Content deferred or simplified:** None — all sections from the template were fully implemented.

## Issues Encountered

None.

## User Setup Required

None — this is a documentation task with no external service configuration.

## Next Phase Readiness

- UPSTREAM.md at repo root is the reference document for Plan 05-03 (execute first merge)
- Plan 05-03 executor can follow UPSTREAM.md step-by-step without additional research
- The `verify-sync.sh` script referenced by UPSTREAM.md will be created as part of Plan 05-03 Wave 0

---
*Phase: 05-upstream-sync*
*Completed: 2026-04-14*
