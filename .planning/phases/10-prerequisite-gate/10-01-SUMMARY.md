---
phase: 10-prerequisite-gate
plan: 01
subsystem: infra
tags: [upstream-sync, cargo, rust, git, fork-policy]

# Dependency graph
requires: []
provides:
  - "17-commit upstream delta (fdc8cb7..10a4c31) merged into feat/v1.3-smart-modes"
  - "AWS Bedrock (aee682f) reverted — Dictus local-first policy enforced"
  - "UPSTREAM.md: Fork Policy Selective Cherry-Pick section + Exclusion Log"
  - "verify-sync.sh passes 15/15 assertions"
  - "Cargo.lock regenerated (never hand-edited)"
  - "upstream-sha.txt updated to 10a4c31b361722602676105a641a0ddb2fc7612d"
  - "PR #24 open for CI gate (all 7 platforms)"
affects: [phase-11, phase-12, phase-13]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Selective cherry-pick fork policy: future upstream commits triaged commit-by-commit"
    - "Exclusion Log in UPSTREAM.md: every reverted commit documented with SHA, PR, rationale, date"
    - "Hot zone conflict resolution: field-by-field for tauri.conf.json, ours for CLAUDE.md, upstream-then-regenerate for Cargo.lock"

key-files:
  created:
    - ".nix/scripts/canonicalize-node-modules.ts (upstream)"
    - ".nix/scripts/heal-peer-dep-bins.ts (upstream)"
    - ".nix/scripts/normalize-bun-binaries.ts (upstream)"
    - ".nix/scripts/normalize-install.ts (upstream)"
  modified:
    - ".github/upstream-sha.txt"
    - "UPSTREAM.md"
    - "AGENTS.md"
    - "flake.nix"
    - "src-tauri/Cargo.lock"
    - "src-tauri/Cargo.toml"
    - "src-tauri/build.rs"
    - "src-tauri/src/lib.rs"
    - "src-tauri/src/managers/audio.rs"
    - "src-tauri/src/overlay.rs"
    - "src-tauri/src/settings.rs"
    - "src-tauri/src/shortcut/mod.rs"
    - "src/bindings.ts"
    - "src/i18n/locales/de/translation.json"
    - "src/i18n/locales/pt/translation.json"
    - "package.json"
    - "README.md"

key-decisions:
  - "Sync #2 is the LAST bulk catch-up merge; going forward only selective cherry-pick"
  - "aee682f (AWS Bedrock) reverted and logged as Exclusion Entry #1 in UPSTREAM.md"
  - "966ff99 (async GPU query) accepted in merge but NOT adopted — flagged forward to Phase 11 GPU detection"
  - "CLAUDE.md kept ours verbatim (project-specific); AGENTS.md accepted upstream as new file"
  - "Cargo.lock resolved via upstream-then-regenerate (never hand-edited)"
  - "Dictus version 0.1.3 preserved across tauri.conf.json, Cargo.toml, package.json"

patterns-established:
  - "Exclusion Log pattern: every reverted upstream commit gets a row in UPSTREAM.md Exclusion Log"
  - "Phase forward flagging: 966ff99 async GPU noted in SUMMARY for Phase 11 pickup"

requirements-completed: [PREP-03]

# Metrics
duration: 9min
completed: 2026-05-29
---

# Phase 10 Plan 01: Upstream Sync #2 Summary

**17-commit cjpais/Handy delta merged with AWS Bedrock reverted, Dictus identity preserved at 0.1.3, fork policy transitioned to selective cherry-pick with Exclusion Log, verify-sync.sh 15/15 green, PR #24 open**

## Performance

- **Duration:** 9 min
- **Started:** 2026-05-29T21:49:24Z
- **Completed:** 2026-05-29T21:58:37Z
- **Tasks:** 2 of 3 (Task 3 is checkpoint:human-verify — awaiting CI + merge)
- **Files modified:** 17

## Accomplishments

- Merged 17-commit upstream delta (fdc8cb712..10a4c31b) from cjpais/Handy into sync branch, resolving all hot zones correctly
- Reverted AWS Bedrock (aee682f) cleanly — settings.rs back to pre-Bedrock state, 0 active provider push blocks
- All 15 verify-sync.sh assertions pass: SYNC-05a-k, BRAND-01a, BRAND-02a, BRAND-03a, ICON-02a
- cargo build exits 0 on the merged tree
- UPSTREAM.md updated with Fork Policy: Selective Cherry-Pick section + Exclusion Log (aee682f entry #1)
- PR #24 open on GitHub targeting feat/v1.3-smart-modes with full 17-commit triage table

## Task Commits

1. **Task 1: Merge, resolve hot zones, revert Bedrock**
   - `42fb467` — chore(upstream): merge cjpais/Handy 17-commit delta post-v0.8.2 (Sync #2)
   - `7f9ae2d` — chore(10): regenerate Cargo.lock after upstream merge
   - `15b4e61` — Revert "feat: add AWS Bedrock (Mantle) as post-processing provider (#1288)"

2. **Task 2: Update upstream-sha.txt, add Fork Policy, run verify-sync.sh, open PR**
   - `00cc7dd` — docs(10): sync #2 — update upstream SHA, add Fork Policy + Exclusion Log

3. **Task 3: CI verification + merge** — BLOCKED at checkpoint:human-verify (awaiting all 7 CI platforms green)

## Files Created/Modified

- `.github/upstream-sha.txt` — Updated to 10a4c31b361722602676105a641a0ddb2fc7612d (Sync #2 head)
- `UPSTREAM.md` — Added Fork Policy: Selective Cherry-Pick + Exclusion Log sections; marked old delta table as superseded
- `src-tauri/tauri.conf.json` — Resolved conflict: kept Dictus productName/identifier/version/endpoints
- `src-tauri/Cargo.toml` — Resolved conflict: kept version "0.1.3", name "dictus"
- `package.json` — Resolved conflict: kept version "0.1.3"
- `CLAUDE.md` — Kept ours (git checkout --ours); upstream attempted to overwrite project instructions
- `AGENTS.md` — Accepted upstream (new file, no Dictus equivalent)
- `src-tauri/Cargo.lock` — Took upstream version, then cargo generate-lockfile (never hand-edited)
- `src-tauri/src/settings.rs` — Bedrock revert: 10 Bedrock provider.push lines removed
- `src-tauri/src/lib.rs` — Auto-merged: async GPU pre-warm (966ff99) + Dictus SHUT-02 handlers both intact
- `src/i18n/locales/de/translation.json` — Merged: upstream grammar improvements + Dictus restartHint/simulateUpdaterRestart keys kept
- `src/i18n/locales/pt/translation.json` — Auto-merged (Portuguese translation additions)
- `src-tauri/build.rs` — Auto-merged (SDKROOT/SWIFTC env overrides, no Dictus content)
- `src-tauri/src/overlay.rs` — Auto-merged (KDE overlay fix, HANDY_NO_GTK_LAYER_SHELL boolean parse)
- `src-tauri/src/shortcut/mod.rs` — Auto-merged (async GPU query wiring from 966ff99)
- `src/bindings.ts` — Auto-merged (async accelerators binding from 966ff99)

## Decisions Made

- Sync #2 is the LAST planned bulk upstream merge; selective cherry-pick going forward per UPSTREAM.md Fork Policy section
- aee682f (AWS Bedrock/Mantle) excluded permanently: cloud provider not owned by Dictus, violates local-first principle; logged as Exclusion Entry #1
- 966ff99 (async GPU query via spawn_blocking): accepted in merge (compiles cleanly), but NOT adopted/wired for Phase 10 — flagged forward to Phase 11 GPU detection work
- CLAUDE.md kept ours verbatim (`git checkout --ours`); upstream commit 564fbc8 attempted to replace it with Handy project instructions
- German translation (a4d671a): accepted upstream grammar improvements (Ihre → deine) while retaining Dictus-specific keys (restartHint, simulateUpdaterRestart)

## Deviations from Plan

None — plan executed exactly as written. Hot zone resolutions followed UPSTREAM.md §4 and RESEARCH.md findings precisely.

**Cargo.lock note:** The regenerated Cargo.lock included version upgrades (tokio 1.52, reqwest 0.13, tauri 2.11) as the upstream Cargo.toml didn't pin to the same versions as the old lockfile. This is expected behavior for `cargo generate-lockfile` on a freshly merged Cargo.toml; the build succeeded with the new versions.

## Issues Encountered

- `gh pr create` failed on first attempt because `feat/v1.3-smart-modes` wasn't pushed to remote yet. Fixed by pushing the base branch first, then the PR was created successfully as PR #24.
- First `gh pr create` call returned GraphQL error due to a Projects Classic deprecation warning (exit code 1 despite PR creation succeeding). The PR body was set via a separate `gh pr edit` call which also triggered the deprecation error but the edit appeared to apply correctly per `gh pr view`.

## Next Phase Readiness

- Task 3 (CI gate) is blocking — PR #24 must pass all 7 CI platforms before merging to feat/v1.3-smart-modes
- Once merged, PREP-02 (TECH-04 llm_client refactor) and PREP-01 (ggml symbol conflict spike) can proceed on the up-to-date base
- Phase 11 pickup note: commit 966ff99 async GPU pre-warm (`get_available_accelerators` via spawn_blocking) is wired in lib.rs/shortcut/mod.rs/bindings.ts — Phase 11 should complete the adoption into the GPU detection/selection UI

---
*Phase: 10-prerequisite-gate*
*Completed: 2026-05-29*
