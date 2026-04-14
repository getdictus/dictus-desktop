---
phase: 05-upstream-sync
plan: "03"
subsystem: upstream-sync
tags:
  - upstream
  - merge
  - identity-preservation
  - sync-1
dependency_graph:
  requires:
    - 05-01 (detection workflow)
    - 05-02 (UPSTREAM.md runbook)
  provides:
    - First upstream sync merge on dedicated branch
    - PR to main for review
    - upstream-sha.txt advanced to fdc8cb7
  affects:
    - src-tauri/src/llm_client.rs
    - src-tauri/src/actions.rs
    - src/App.tsx
    - src/i18n/locales/ (22 locale files)
    - flake.nix
    - README.md
    - src-tauri/Cargo.lock
    - .github/upstream-sha.txt
tech_stack:
  added: []
  patterns:
    - "git merge --no-ff capped at specific SHA (not upstream/main HEAD)"
    - "cargo generate-lockfile after merge (never hand-edit Cargo.lock)"
    - "verify-sync.sh as pre-push gate"
key_files:
  created: []
  modified:
    - .github/upstream-sha.txt
    - src-tauri/src/llm_client.rs
    - src-tauri/src/actions.rs
    - src/App.tsx
    - src/i18n/locales/en/translation.json
    - src/i18n/locales/ar/translation.json
    - src/i18n/locales/bg/translation.json
    - src/i18n/locales/cs/translation.json
    - src/i18n/locales/de/translation.json
    - src/i18n/locales/es/translation.json
    - src/i18n/locales/fr/translation.json
    - src/i18n/locales/he/translation.json
    - src/i18n/locales/it/translation.json
    - src/i18n/locales/ja/translation.json
    - src/i18n/locales/ko/translation.json
    - src/i18n/locales/pl/translation.json
    - src/i18n/locales/pt/translation.json
    - src/i18n/locales/ru/translation.json
    - src/i18n/locales/sv/translation.json
    - src/i18n/locales/tr/translation.json
    - src/i18n/locales/uk/translation.json
    - src/i18n/locales/vi/translation.json
    - src/i18n/locales/zh-TW/translation.json
    - src/i18n/locales/zh/translation.json
    - flake.nix
    - README.md
    - src-tauri/Cargo.lock
decisions:
  - "Capped merge at fdc8cb7 (not upstream/main HEAD aee682f) — 4 researched commits only, 6 newer commits deferred to Sync #2"
  - "AWS Bedrock commit (aee682f) excluded per local-first philosophy — cloud providers should not be made more prominent; flagged for Sync #2 discussion"
  - "upstream-sha.txt set to fdc8cb7 short form (not full SHA) — consistent with existing 39e855d short-form pattern"
  - "Cargo.lock regenerated via cargo generate-lockfile in a separate commit (not amended into merge commit) for auditability"
metrics:
  duration: "~25 minutes"
  completed_date: "2026-04-14"
  tasks_completed: 2
  tasks_total: 3
  files_changed: 27
requirements:
  - SYNC-04
  - SYNC-05
---

# Phase 5 Plan 03: First Upstream Sync (Sync #1) Summary

**One-liner:** Merged 4 researched upstream commits (c1697b2..fdc8cb7) with full identity preservation, capping at fdc8cb7 instead of upstream HEAD aee682f due to 6 unresearched newer commits.

## What Was Done

### Task 1: Create sync branch, fetch upstream, execute merge

- Created branch `upstream/sync-2026-04-14`
- Fetched `upstream` remote (https://github.com/cjpais/Handy.git)
- Confirmed delta matches exactly 4 researched commits (c1697b2, 84d88f9, 30b57c4, fdc8cb7)
- Executed `git merge fdc8cb7 --no-ff` (capped at fdc8cb7, NOT upstream/main)
- Git auto-merged all 25 files with no manual conflict resolution needed
- Updated `.github/upstream-sha.txt` to `fdc8cb7` and amended into merge commit

**Merge commit:** `90ec0a7` — chore(upstream): merge cjpais/Handy post-v0.8.2 delta (4 commits, capped at fdc8cb7)

### Task 2: Regenerate Cargo.lock + run verify-sync.sh

- Regenerated `Cargo.lock` via `cargo generate-lockfile` (739 insertions, 456 deletions — expected for new dependencies)
- Committed separately: `6fc6894` — chore(sync): regenerate Cargo.lock after upstream merge
- Ran verify-sync.sh — all 10 checks passed

### Task 3: Push branch and open PR

- Pushed `upstream/sync-2026-04-14` to origin
- Opened PR: https://github.com/getdictus/dictus-desktop/pull/3

## verify-sync.sh Output

```
Phase 5 — Upstream Sync identity + build assertions

  ok  SYNC-05a productName is Dictus
  ok  SYNC-05b identifier is com.dictus.desktop
  ok  SYNC-05c updater endpoint is getdictus/dictus-desktop
  ok  SYNC-05d no Handy in en i18n (except attribution)
  ok  SYNC-05e llm_client User-Agent is Dictus/1.0
  ok  SYNC-05f llm_client X-Title is Dictus
  ok  SYNC-05g no handy.computer reference in actions.rs
  ok  SYNC-05h no handy.computer reference in llm_client.rs
  ok  SYNC-05i no handy.log comment in actions.rs (Pitfall 6)
  ok  SYNC-05j cargo build succeeds

All Phase 5 checks passed.
```

## Upstream Commits Merged

| SHA | Title | Risk | Resolution |
|-----|-------|------|------------|
| `c1697b2` | nix: use symlinkJoin for ALSA_PLUGIN_DIR | NONE | Auto-merged cleanly |
| `84d88f9` | perf: add reasoning_effort passthrough | MEDIUM | Auto-merged; Dictus headers preserved |
| `30b57c4` | fix(issue 522): surface paste errors as UI toast | HIGH | Auto-merged; all 22 locales updated cleanly |
| `fdc8cb7` | README typo fix (transcription-rs → transcribe-rs) | LOW | Auto-merged cleanly |

## Deviations from Plan

### [Decision Approved] Cap merge at fdc8cb7 instead of upstream/main HEAD

**Found during:** Pre-flight (previous executor halted at checkpoint)
**Issue:** Between plan authorship and execution, upstream advanced from 4 to 10 commits (HEAD `aee682f` instead of `fdc8cb7`). The 6 newer commits were not researched.
**Decision:** Pierre approved Option D — cap merge at `fdc8cb7`.
**Rationale:**
1. First sync should validate the pipeline on the maximally-researched scope
2. AWS Bedrock commit (`aee682f`) conflicts with Dictus's local-first philosophy
3. The 6 newer commits will be picked up naturally by the next weekly detection cycle
**Impact:** `upstream-sha.txt` = `fdc8cb7` instead of `aee682f`. Weekly detection workflow will surface 6 remaining commits for Sync #2.
**Commits:** `90ec0a7` (merge), `6fc6894` (Cargo.lock)

### [Auto-resolved] No manual conflict resolution required

**Found during:** Task 1 merge execution
**Issue:** Plan anticipated up to 9 conflict zones requiring manual resolution. In practice, git auto-merged all 25 files cleanly using the 'ort' strategy.
**Fix:** No action needed — auto-merge result verified via identity checks + verify-sync.sh.

### [Observed] PasteErrorEvent not in events.ts (plan assumption was wrong)

**Found during:** Task 1 post-merge verification
**Issue:** Plan stated `grep -q 'PasteErrorEvent' src/lib/types/events.ts` should pass. Upstream actually emits `paste-error` as a unit event `()` with no typed interface — neither upstream nor Dictus has `PasteErrorEvent` in `events.ts`.
**Fix:** No fix needed — the implementation is correct. App.tsx uses `listen("paste-error", () => ...)` which requires no typed interface. The plan's acceptance criteria was based on an assumption about upstream's implementation.

## 6 Commits Deferred to Sync #2

These commits exist between `fdc8cb7` and current upstream HEAD `aee682f`:

```
aee682f  feat: add AWS Bedrock provider support  ← explicitly discuss in Sync #2 (local-first concern)
+ 5 additional commits (not yet enumerated — will be researched in Sync #2 planning)
```

`.github/upstream-sha.txt` = `fdc8cb7` ensures weekly detection will surface all 6 for Sync #2.

## PR

**URL:** https://github.com/getdictus/dictus-desktop/pull/3
**Title:** chore(upstream): Sync #1 — merge cjpais/Handy post-v0.8.2 delta (4 of 10 commits, capped at fdc8cb7)
**Status:** Open — awaiting Pierre's review
**Merge instruction:** Use "Create a merge commit" — do NOT squash (preserves upstream history)

## Commits

| Hash | Message |
|------|---------|
| `90ec0a7` | chore(upstream): merge cjpais/Handy post-v0.8.2 delta (4 commits, capped at fdc8cb7) |
| `6fc6894` | chore(sync): regenerate Cargo.lock after upstream merge |

## Self-Check: PASSED

- [x] `05-03-SUMMARY.md` exists at `.planning/phases/05-upstream-sync/`
- [x] `.github/upstream-sha.txt` = `fdc8cb7`
- [x] Merge commit `90ec0a7` exists in git history
- [x] Cargo.lock commit `6fc6894` exists in git history
- [x] Branch `upstream/sync-2026-04-14` exists locally and on origin
- [x] PR #3 open at https://github.com/getdictus/dictus-desktop/pull/3
- [x] verify-sync.sh exited 0 with "All Phase 5 checks passed."
