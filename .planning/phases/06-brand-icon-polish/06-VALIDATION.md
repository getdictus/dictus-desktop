---
phase: 6
slug: brand-icon-polish
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-15
---

# Phase 6 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | shell (verify-sync.sh) + cargo check + bun run lint |
| **Config file** | `.github/scripts/verify-sync.sh` (after SYNC-06 move) |
| **Quick run command** | `bash .github/scripts/verify-sync.sh` |
| **Full suite command** | `bash .github/scripts/verify-sync.sh && cargo check --manifest-path src-tauri/Cargo.toml && bun run lint` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `bash .github/scripts/verify-sync.sh`
- **After every plan wave:** Run full suite command
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| TBD | TBD | TBD | BRAND-01 | grep | `bash .github/scripts/verify-sync.sh` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | BRAND-02 | grep | `bash .github/scripts/verify-sync.sh` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | BRAND-03 | grep | `bash .github/scripts/verify-sync.sh` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | BRAND-04 | shell exit code | `bash .github/scripts/verify-sync.sh` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | ICON-01 | file inspection | `identify src-tauri/icons/icon.ico` | ✅ | ⬜ pending |
| TBD | TBD | TBD | ICON-02 | grep layers | `bash .github/scripts/verify-sync.sh` (ICON-02a) | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | ICON-03 | config check | `grep -c 'png' src-tauri/tauri.conf.json` | ✅ | ⬜ pending |
| TBD | TBD | TBD | ICON-04 | file exists | `test -f ../dictus-brand/logo/square-1024.png` | ⚠️ external | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

*Note: Task IDs will be assigned by the planner. The requirement → verification command mapping is the contract; planner must ensure each requirement has a task that triggers the mapped command.*

---

## Wave 0 Requirements

- [ ] `.github/scripts/verify-sync.sh` — move from repo root (SYNC-06), add BRAND-01/02/03 grep assertions, add ICON-02a layer assertion
- [ ] Update UPSTREAM.md path references (4 locations: lines 216, 239, 254, 284) to point to new verify-sync.sh location
- [ ] `../dictus-brand/logo/square-1024.png` — EXTERNAL dependency (ICON-04); must exist before icon tasks can run

*Rationale: verify-sync.sh is the primary automated verification surface for BRAND-01..04. It must be in place (and moved to `.github/scripts/`) before task commits can be sampled against it.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Linux app launcher shows Dictus icon with no black corner artifact | ICON-01 | Requires actual Linux desktop environment render | Install .deb/.AppImage on Linux, open file manager / activities, inspect icon at 32/48/64/128px — no black square in any corner |
| Windows taskbar and executable icon shows Dictus logo at every scale | ICON-02 | Requires Windows shell render at multiple DPIs | Install .msi on Windows, pin to taskbar, inspect at 100%, 125%, 150%, 200% scaling; right-click executable → Properties to confirm embedded icon |
| DebugPaths panel shows real OS path (not `%APPDATA%/handy`) | BRAND-03 | UI behavior requires app to be running | Launch app, open Settings → Debug Paths, verify data dir shows `~/Library/Application Support/com.dictus.desktop` (macOS) or platform equivalent |
| History panel shows `dictus-*.wav` filenames (not `handy-*.wav`) | BRAND-01 | Requires recording a new clip in the running app | Record a transcription, open History panel, confirm filename starts with `dictus-` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (verify-sync.sh move + assertions)
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
