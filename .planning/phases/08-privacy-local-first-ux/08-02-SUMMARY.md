---
phase: 08-privacy-local-first-ux
plan: "02"
subsystem: docs
tags: [privacy, network-surface, documentation, local-first, markdown]

# Dependency graph
requires: []
provides:
  - "docs/PRIVACY.md: complete outbound network surface audit (11 endpoint families, 16 CDN URLs)"
  - "README.md ## Privacy section linking to docs/PRIVACY.md"
  - "UPSTREAM.md ## Privacy / Network Surface Hook: grep-based maintenance gate"
affects:
  - "08-03 About panel (Plan 03): openUrl() to github.com/getdictus/dictus-desktop/blob/main/docs/PRIVACY.md"

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Network surface audit document: single source-of-truth markdown table for outbound endpoints"
    - "Upstream maintenance hook: grep command in UPSTREAM.md to catch undocumented endpoints before merging"

key-files:
  created:
    - docs/PRIVACY.md
  modified:
    - README.md
    - UPSTREAM.md

key-decisions:
  - "docs/PRIVACY.md is the single source of truth for network surface — no in-app Privacy page"
  - "blob.handy.computer CDN documented as-is (INFR-01 deferred); legacy Referer header flagged but not changed in Phase 8"
  - "Prettier applied to all three files to align with project format standards"

patterns-established:
  - "Privacy doc pattern: short prose preamble + endpoint table + legacy artifact notes + headers + non-contacted list + appendix"
  - "Upstream hook pattern: grep command in UPSTREAM.md to detect undocumented HTTP endpoints before merge"

requirements-completed:
  - PRIV-02

# Metrics
duration: 5min
completed: "2026-05-21"
---

# Phase 8 Plan 02: Privacy / Network Surface Documentation Summary

**`docs/PRIVACY.md` audit document listing all 11 outbound endpoint families and 16 model CDN URLs, linked from README.md and guarded by a grep-based maintenance hook in UPSTREAM.md**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-05-21T14:40:30Z
- **Completed:** 2026-05-21T14:45:42Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Created `docs/PRIVACY.md` (75 lines) documenting all 11 outbound endpoint families with a scannable table (endpoint, trigger, data sent, disable method, default state)
- Listed all 16 `blob.handy.computer` model CDN URLs in an appendix, flagging the INFR-01 migration and the legacy OpenRouter `Referer` header
- Added `## Privacy` section to README.md for discoverability
- Added `## Privacy / Network Surface Hook` to UPSTREAM.md with a grep-based check script to prevent silent endpoint drift in future upstream syncs

## Task Commits

Each task was committed atomically:

1. **Task 1: Author docs/PRIVACY.md** - `10e11b3` (docs)
2. **Task 2: Link PRIVACY.md from README and add UPSTREAM maintenance hook** - `b00146f` (docs)

## Files Created/Modified

- `docs/PRIVACY.md` — Network surface audit: 11 endpoint families in a table, 16-URL appendix, legacy artifact notes, common LLM headers, endpoints not contacted
- `README.md` — New `## Privacy` section (4 lines) after "## How it works", pointing to `docs/PRIVACY.md`
- `UPSTREAM.md` — New `## Privacy / Network Surface Hook` section at end of file with grep check command

## Confirmed endpoint families present in docs/PRIVACY.md

1. `https://github.com/getdictus/dictus-desktop/releases/latest/download/latest.json` (updater)
2. `https://blob.handy.computer/*` (model CDN, 16 URLs in appendix)
3. `https://github.com/getdictus/dictus-desktop/releases/latest` (browser navigation)
4. `https://api.openai.com/v1/{chat/completions,models}`
5. `https://api.z.ai/api/paas/v4/{chat/completions,models}`
6. `https://openrouter.ai/api/v1/{chat/completions,models}`
7. `https://api.anthropic.com/v1/{messages,models}`
8. `https://api.groq.com/openai/v1/{chat/completions,models}`
9. `https://api.cerebras.ai/v1/{chat/completions,models}`
10. `http://localhost:11434/v1/{chat/completions,models}` (local-only)
11. `apple-intelligence://local` (on-device)

## README `## Privacy` section location

Inserted between "## How it works" (line 47) and "## How Dictus compares" (line 57) in README.md.

## UPSTREAM `## Privacy / Network Surface Hook` section location

Appended at end of UPSTREAM.md after the existing "Key files:" bullet list in the `## Quick Reference` section (line 286 onward).

## Decisions Made

- `docs/PRIVACY.md` is the single source of truth for outbound network surface — no in-app Privacy settings page (plan decision, confirmed in CONTEXT.md)
- `blob.handy.computer` CDN documented as current state; INFR-01 migration deferred and noted honestly
- Legacy `Referer: https://github.com/cjpais/Handy` header documented and flagged as an upstream artifact to be scrubbed in a follow-up PR; not changed in Phase 8
- Prettier applied to all three files (`docs/PRIVACY.md`, `README.md`, `UPSTREAM.md`) to align with project format; pre-existing warnings in other project files are not caused by this plan

## Deviations from Plan

None - plan executed exactly as written. Prettier formatting applied as part of the commit (pre-existing format warnings in .planning/ and other files are out of scope for this plan).

## Issues Encountered

`bun run format:check` exits non-zero due to pre-existing Prettier warnings across 84 files in the project (`.planning/` files, other source files). The three files this plan touched (`docs/PRIVACY.md`, `README.md`, `UPSTREAM.md`) were individually formatted with `npx prettier --write` before the Task 2 commit; the pre-existing failures are not caused by this plan.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `docs/PRIVACY.md` is ready to be linked from the About panel in Plan 03 via `openUrl()` to `https://github.com/getdictus/dictus-desktop/blob/main/docs/PRIVACY.md`
- PRIV-02 requirement is complete
- No blockers for Phase 8 Plan 03

---
*Phase: 08-privacy-local-first-ux*
*Completed: 2026-05-21*
