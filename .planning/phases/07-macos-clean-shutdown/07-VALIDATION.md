---
phase: 7
slug: macos-clean-shutdown
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-16
---

# Phase 7 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Static analysis only — no Tauri integration test framework in repo |
| **Config file** | `src-tauri/Cargo.toml`, `eslint.config.js` |
| **Quick run command** | `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` |
| **Full suite command** | `bun run lint && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings && cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` |
| **Estimated runtime** | ~45 seconds (clippy incremental) |

---

## Sampling Rate

- **After every task commit:** Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- **After every plan wave:** Run `bun run lint && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- **Before `/gsd:verify-work`:** Full suite must be green AND 3 consecutive clean tray-quits observed on fresh `bun run tauri build` install
- **Max feedback latency:** 60 seconds (static checks); human-in-the-loop for crash-dialog observation

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 07-01-01 | 01 | 0 | SHUT-01 | manual + doc inspection | `grep -q "## Diagnosis" .planning/phases/07-macos-clean-shutdown/07-01-PLAN.md` | ✅ | ⬜ pending |
| 07-01-02 | 01 | 1 | SHUT-02 | static | `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | ✅ | ⬜ pending |
| 07-01-03 | 01 | 1 | SHUT-02 | static + grep | `grep -n "log::logger().flush()" src-tauri/src/lib.rs` (expect ≥2 hits) | ✅ | ⬜ pending |
| 07-01-04 | 01 | 2 | SHUT-03 | manual observation | `bun run tauri build` + 3× tray-quit + 1× restart-simulation | ❌ manual | ⬜ pending |
| 07-01-05 | 01 | 2 | SHUT-03 | static + grep | `grep -n "simulate_updater_restart\|app.restart()" src-tauri/src/` (expect debug-gated handler) | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `07-01-PLAN.md` contains `## Diagnosis` section at top, committed before any `src-tauri/` code change (SHUT-01 gate)
- [ ] No new test framework installation required — static checks (clippy, lint) are pre-existing

*No new test files to create; SHUT-01 is documentation-gated, SHUT-02 is static-verifiable, SHUT-03 is human-observation by design.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `.ips` crash report generated and read | SHUT-01 | Requires real macOS crash event; no way to synthesize | 1. Fresh `main` checkout, `bun run tauri build`; 2. Install `.app` from `src-tauri/target/release/bundle/macos/`; 3. Launch, click tray → "Quit Dictus"; 4. Open Console.app → User Reports → latest Dictus-*.ips; 5. Identify `"triggered": true` thread; 6. Paste summary into `07-01-PLAN.md` `## Diagnosis`. |
| Clean tray-quit (no crash dialog) | SHUT-03 | OS-level crash dialog is not programmatically detectable | 1. Fresh `bun run tauri build` install; 2. Launch; 3. Tray → Quit Dictus; 4. Observe: no "Dictus quit unexpectedly" dialog; 5. Repeat 3× total — all must pass. |
| Clean restart-simulation (no crash dialog) | SHUT-03 | Same OS-level observation required | 1. Fresh build; 2. In devtools console: `window.__TAURI__.core.invoke('simulate_updater_restart')`; 3. Observe: app relaunches without crash dialog appearing at any point. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies OR documented manual-only
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify (static clippy covers all code tasks)
- [ ] Wave 0 covers all MISSING references (none — gate is documentation-only)
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s for static; human-in-the-loop for crash observation
- [ ] `nyquist_compliant: true` set in frontmatter after planner fills per-task map

**Approval:** pending
