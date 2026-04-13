# Phase 5: Upstream Sync - Context

**Gathered:** 2026-04-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Detect new upstream Handy commits automatically each week, document the fork/sync strategy in UPSTREAM.md, merge the current 4-commit post-v0.8.2 delta into a dedicated branch, and verify identity integrity post-merge. This phase delivers the **foundations** for upstream sync — detection, documentation, and first proven merge. Full IA-driven automation (Claude Code agents doing merges/reviews) is deferred to Phase 6.

**Out of scope for this phase:** AI-automated merge pipeline (Phase 6), binary rename handy->dictus (deferred TECH-03), code signing (deferred INFR-03), develop branch (not needed for solo project).

</domain>

<decisions>
## Implementation Decisions

### Detection action design
- **Workflow file**: `.github/workflows/upstream-sync.yml`
- **Triggers**: Weekly cron (Monday 08:00 UTC) + manual `workflow_dispatch`
- **Issue content**: Commit list only — title: "Upstream: N new commits on cjpais/Handy", body: commit hashes + one-line messages + link to upstream compare view. Minimal, scannable.
- **Idempotency**: Store last-seen upstream SHA in `.github/upstream-sha.txt` committed to repo. Action compares current upstream HEAD against stored SHA before creating issue. No issue if unchanged.
- **Label**: `upstream-sync` label must be created in repo before first action run

### Merge strategy
- **Method**: Single `git merge upstream/main` on a dedicated branch — one merge commit preserving upstream history
- **Branch naming**: `upstream/sync-YYYY-MM-DD` (date-stamped, under upstream/ namespace)
- **Landing**: PR to main — review checkpoint, CI runs, clean merge record
- **No develop branch** — not needed for solo project. PR to main is the gate.
- **Post-merge checks (SYNC-05)**: grep `productName`, `identifier`, and i18n strings for Dictus (not Handy), plus `cargo build` succeeds. Matches Phase 4 identity verification pattern.
- **First merge executor**: Claude Code executes the merge in a plan task (creates branch, merges, resolves conflicts, verifies identity, opens PR). Pierre reviews and merges.

### UPSTREAM.md structure
- **Location**: Root of repo (`UPSTREAM.md`) alongside README.md — high visibility
- **Detail level**: Copy-paste git commands — future-Pierre can follow in 6 months without re-researching (same philosophy as `docs/RUNBOOK-updater-signing.md`)
- **Content sections**:
  - Fork point info (`85a8ed77`, merge-base `39e855d`)
  - Step-by-step merge process (fetch, branch, merge, check, PR)
  - Conflict resolution guide for known hot zones (tauri.conf.json, i18n files, Cargo.toml — always keep Dictus values)
  - Post-merge checklist (identity grep, cargo build, handy.computer scan)
- **SHA tracking**: References `.github/upstream-sha.txt` as the source of truth for "where are we"

### Future sync process (Phase 5 scope)
- After Phase 5 ships, ongoing syncs are **manual via UPSTREAM.md** process
- Weekly action creates notification issues — Pierre decides if/when to merge
- Full AI-driven pipeline (Claude Code agents doing analysis, merge, PR, review) is deferred to Phase 6

### Claude's Discretion
- Exact cron expression for Monday 08:00 UTC
- Issue body template formatting (markdown)
- UPSTREAM.md prose and section ordering
- Whether post-merge identity checks are a script or inline commands in UPSTREAM.md
- Conflict resolution order if multiple files conflict simultaneously
- Whether to regenerate Cargo.lock as part of merge or as separate step

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements and scope
- `.planning/REQUIREMENTS.md` §Upstream Sync — SYNC-01 through SYNC-05 acceptance criteria
- `.planning/ROADMAP.md` §"Phase 5: Upstream Sync" — goal, dependencies, success criteria list
- `.planning/PROJECT.md` §"Current Milestone: v1.1 Auto-Update & Upstream Sync" — milestone framing
- `.planning/PROJECT.md` §"Origine" — fork point `85a8ed77`, remote `upstream` configured

### Research inputs
- `.planning/research/SUMMARY.md` — executive summary, upstream delta is 4 commits not 69
- `.planning/research/ARCHITECTURE.md` — merge-base info, upstream delta analysis
- `.planning/research/PITFALLS.md` — pitfalls relevant to upstream merge (identity overwrite, Cargo.lock)

### Prior phase context
- `.planning/phases/04-updater-infrastructure/04-CONTEXT.md` — Phase 4 decisions (identity verification pattern, TECH-03 deferral, version policy)
- `.planning/STATE.md` — accumulated decisions including "upstream delta is 4 commits" and "upstream-sync label must be created"

### Existing sync-related files
- `.planning/phases/04-updater-infrastructure/deferred-items.md` — deferred items from Phase 4 (validate.sh fix)

### No external ADRs
This project has no `docs/decisions/` ADR directory. All context and decisions live in `.planning/` and inline in CONTEXT.md.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `upstream` git remote already configured pointing to `https://github.com/cjpais/Handy.git`
- Phase 4's identity verification pattern (grep for productName, identifier, i18n) — reuse same approach for SYNC-05
- `.planning/phases/04-updater-infrastructure/scripts/validate.sh` — reference for writing post-merge check scripts

### Established Patterns
- **GitHub Actions**: 9 existing workflows in `.github/workflows/` — follow same YAML conventions
- **Manual trigger**: `release.yml` uses `workflow_dispatch` — same pattern for upstream-sync manual trigger
- **Documentation**: `docs/RUNBOOK-updater-signing.md` sets the copy-paste commands style for UPSTREAM.md
- **Commit conventions**: `feat:`, `fix:`, `docs:`, `chore:` — upstream merge commits should use `chore(upstream):` or similar

### Integration Points
- `.github/upstream-sha.txt` (new) — tracked by the detection action, committed to repo
- `upstream-sync` label (new) — must be created before first action run
- GitHub Issues API — action creates issues programmatically
- `upstream/main` remote branch — action fetches and compares against stored SHA

</code_context>

<specifics>
## Specific Ideas

- Pierre's full vision for upstream sync is an AI-driven pipeline: Claude Code agent #1 analyzes commits + adapts code + opens PR, Claude Code agent #2 reviews the PR + generates test list, Pierre merges and tests for several days, then releases. This is the Phase 6 target.
- The UPSTREAM.md should be written so that both a human (Pierre) AND a future AI agent can follow the steps — explicit, unambiguous, command-based.
- No develop branch — solo project, PR to main is sufficient. Revisit if team grows.

</specifics>

<deferred>
## Deferred Ideas

- **Phase 6: AI-driven upstream sync pipeline** — Claude Code scheduled agent analyzes upstream commits for relevance, adapts code (identity preservation), opens PR. Second agent reviews PR and generates manual test checklist. Pierre approves, tests, releases. Full automation of the merge workflow.
- **SYNC-A1: AI-assisted cherry-pick triage** — already in REQUIREMENTS.md as future requirement. Subsumed by Phase 6 vision.
- **Semi-automatic PR creation in the action** — action could auto-create branch + merge + PR on detection. Intermediate step between manual and full AI pipeline. Could be Phase 6 wave 1.
- **Develop branch** — not needed now, revisit if team grows or release cadence demands a staging buffer.

</deferred>

---

*Phase: 05-upstream-sync*
*Context gathered: 2026-04-14*
