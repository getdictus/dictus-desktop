---
phase: 1
slug: bundle-identity
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-05
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | None needed — all checks are grep-based smoke tests |
| **Config file** | none |
| **Quick run command** | `grep -rn "Handy\|cjpais\|com.pais\|0\.8\.2\|signCommand" src-tauri/tauri.conf.json src-tauri/Cargo.toml` |
| **Full suite command** | `bun run format:check && bun run lint` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run the smoke grep for the specific field edited
- **After every plan wave:** Run all 5 greps + `bun run format:check`
- **Before `/gsd:verify-work`:** Full suite must be green + `bun run tauri build` succeeds
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 1 | BNDL-01 | smoke (grep) | `grep '"productName": "Dictus"' src-tauri/tauri.conf.json` | ✅ | ⬜ pending |
| 01-01-02 | 01 | 1 | BNDL-02 | smoke (grep) | `grep '"identifier": "com.dictus.desktop"' src-tauri/tauri.conf.json` | ✅ | ⬜ pending |
| 01-01-03 | 01 | 1 | BNDL-03 | smoke (grep) | `grep -E 'version = "0\.1\.0"' src-tauri/Cargo.toml && grep 'Dictus' src-tauri/Cargo.toml` | ✅ | ⬜ pending |
| 01-01-04 | 01 | 1 | BNDL-04 | smoke (grep) | `grep -c "cjpais\|github.com/cjpais" src-tauri/tauri.conf.json` (expect 0) | ✅ | ⬜ pending |
| 01-01-05 | 01 | 1 | BNDL-05 | smoke (grep) | `grep -c "signCommand" src-tauri/tauri.conf.json` (expect 0) | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

*Existing infrastructure covers all phase requirements.*

No test framework setup required — all verifications are grep-based smoke checks against the two config files.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| macOS dock shows "Dictus" in release build | BNDL-01 | Requires `tauri build` + visual inspection of .app | 1. Run `bun run tauri build` 2. Open the built .app from `src-tauri/target/release/bundle/` 3. Verify dock shows "Dictus" |
| Dev mode dock shows "handy" (expected) | BNDL-01 | Known Tauri 2 dev/build difference — informational only | Run `tauri dev`, confirm dock shows "handy" — this is expected and not a bug |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
