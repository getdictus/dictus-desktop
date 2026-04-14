---
phase: 05
slug: upstream-sync
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-14
---

# Phase 05 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | bash scripts (shell-based validators; project has no formal test framework for infrastructure phases) |
| **Config file** | none — Wave 0 creates `verify-sync.sh` |
| **Quick run command** | `bash .planning/phases/05-upstream-sync/scripts/verify-sync.sh` |
| **Full suite command** | `bash .planning/phases/05-upstream-sync/scripts/verify-sync.sh && cargo build --manifest-path src-tauri/Cargo.toml` |
| **Estimated runtime** | ~60 seconds (verify-sync < 5s; cargo build dominates) |

---

## Sampling Rate

- **After every task commit:** Run `grep -q '"productName": "Dictus"' src-tauri/tauri.conf.json && grep -q '"identifier": "com.dictus.desktop"' src-tauri/tauri.conf.json`
- **After every plan wave:** Run `bash .planning/phases/05-upstream-sync/scripts/verify-sync.sh`
- **Before `/gsd:verify-work`:** Full suite must be green AND PR opened
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 05-01-01 | 01 | 1 | SYNC-01, SYNC-02 | smoke (yaml validity) | `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/upstream-sync.yml'))"` | ❌ W0 | ⬜ pending |
| 05-01-02 | 01 | 1 | SYNC-02 | smoke (file presence) | `test -f .github/upstream-sha.txt && grep -q '39e855d' .github/upstream-sha.txt` | ❌ W0 | ⬜ pending |
| 05-01-03 | 01 | 1 | SYNC-01 | manual | Manual pre-flight: create `upstream-sync` label in GitHub repo settings | manual-only | ⬜ pending |
| 05-02-01 | 02 | 1 | SYNC-03 | smoke (content check) | `grep -q '85a8ed77' UPSTREAM.md && grep -q '39e855d' UPSTREAM.md` | ❌ W0 | ⬜ pending |
| 05-02-02 | 02 | 1 | SYNC-03 | smoke (runbook steps) | `grep -qE 'git (fetch|merge|checkout)' UPSTREAM.md` | ❌ W0 | ⬜ pending |
| 05-03-01 | 03 | 2 | SYNC-05 | unit (identity) | `bash .planning/phases/05-upstream-sync/scripts/verify-sync.sh` | ❌ W0 | ⬜ pending |
| 05-03-02 | 03 | 2 | SYNC-04 | smoke (merge branch) | `git log upstream/sync-* --oneline \| grep -E "c1697b2\|84d88f9\|30b57c4\|fdc8cb7"` | ❌ W0 | ⬜ pending |
| 05-03-03 | 03 | 2 | SYNC-05 | unit (build) | `cargo build --manifest-path src-tauri/Cargo.toml` | ❌ W0 | ⬜ pending |
| 05-03-04 | 03 | 2 | SYNC-04 | manual | Open PR from `upstream/sync-YYYY-MM-DD` to `main`; verify PR URL returned | manual-only | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `.planning/phases/05-upstream-sync/scripts/verify-sync.sh` — covers SYNC-05 identity + build assertions (check()/FAIL pattern from Phase 4 `validate.sh`)
- [ ] `.github/upstream-sha.txt` — seed value `39e855d` (merge-base / v0.8.2)
- [ ] `upstream-sync` label created in GitHub repo settings (manual pre-flight, blocks workflow first run)

*Wave 0 must be complete before Wave 1 tasks can verify.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Workflow creates an issue on detected drift | SYNC-01 | Requires GitHub Actions runner + repo token + live upstream state | After merge: trigger `workflow_dispatch` with stored SHA = merge-base; confirm one new issue with `upstream-sync` label, correct title, commit list in body |
| Workflow is idempotent (no duplicate issue on second run) | SYNC-02 | Requires two sequential runs with same upstream state | Trigger `workflow_dispatch` twice in quick succession; confirm only one issue exists with `upstream-sync` label |
| `upstream-sync` label exists in repo | SYNC-01 | GitHub repo settings UI; not script-accessible | Navigate to repo → Labels; confirm `upstream-sync` exists (any color) |
| PR opened from merge branch | SYNC-04 | Requires Pierre's review in GitHub UI | Open PR from `upstream/sync-YYYY-MM-DD` → `main`; confirm PR URL, CI runs green |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (`verify-sync.sh`, `upstream-sha.txt`, label)
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
