---
phase: 10
slug: prerequisite-gate
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-29
---

# Phase 10 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo` build/clippy/test) + ESLint/TypeScript (frontend) + bash identity script |
| **Config file** | `src-tauri/Cargo.toml` (Rust workspace); `.github/scripts/verify-sync.sh` (identity) |
| **Quick run command** | `cargo build --manifest-path src-tauri/Cargo.toml` |
| **Full suite command** | `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings && cargo build --manifest-path src-tauri/Cargo.toml && bun run lint && bash .github/scripts/verify-sync.sh` |
| **Estimated runtime** | ~120–600 seconds (cold ggml/whisper compile dominates; verify-sync.sh < 5s) |

---

## Sampling Rate

- **After every task commit:** Run `cargo build --manifest-path src-tauri/Cargo.toml`
- **After every plan wave:** Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings && bun run lint && bash .github/scripts/verify-sync.sh`
- **Before `/gsd:verify-work`:** Full suite green locally + all 7 CI platforms green in `main-build.yml`
- **Max feedback latency:** ~600 seconds (cold compile worst case; incremental builds far faster)

---

## Per-Task Verification Map

> Task IDs are placeholders until plans are finalized; the planner maps each requirement to concrete task IDs. Phase 10 validation is build/lint/script-driven (no new unit tests — these are blocker-clearing work items).

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 10-01-* | 01 (PREP-03) | 1 | PREP-03 | identity script | `bash .github/scripts/verify-sync.sh` | ✅ | ⬜ pending |
| 10-01-* | 01 (PREP-03) | 1 | PREP-03 | build smoke | `cargo build --manifest-path src-tauri/Cargo.toml` | ✅ | ⬜ pending |
| 10-02-* | 02 (PREP-02) | 2 | PREP-02 | lint | `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | ✅ | ⬜ pending |
| 10-02-* | 02 (PREP-02) | 2 | PREP-02 | build | `cargo build --manifest-path src-tauri/Cargo.toml` | ✅ | ⬜ pending |
| 10-03-* | 03 (PREP-01) | 3 | PREP-01 | build (coexistence) | `cargo build --manifest-path src-tauri/Cargo.toml` | ❌ W0 | ⬜ pending |
| 10-03-* | 03 (PREP-01) | 3 | PREP-01 | dep graph | `cargo tree --manifest-path src-tauri/Cargo.toml \| grep ggml` | ❌ W0 | ⬜ pending |
| 10-03-* | 03 (PREP-01) | 3 | PREP-01 | CI matrix | All 7 targets green in `main-build.yml` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/Cargo.toml` — add `llama-cpp-2` dependency entry (enables PREP-01 coexistence build + `cargo tree | grep ggml` commands)

*PREP-02 and PREP-03 need no Wave 0 setup — existing toolchain (`cargo`, `clippy`, `verify-sync.sh`, CI matrix) covers all behavior.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Per-commit upstream triage table is accurate & complete | PREP-03 | Human judgment — triage notes summarize intent of 16 kept commits | Review PR body triage table against `git log fdc8cb712..upstream/main`; confirm each commit categorized |
| Fork Policy + Exclusion Log section reads correctly | PREP-03 | Prose/policy quality, not machine-checkable | Read new `UPSTREAM.md` section; confirm Bedrock `aee682f` exclusion entry present with rationale |
| ggml spike conclusion (pass / patch-applied / fallback) | PREP-01 | Outcome depends on live CI results across 7 platforms | Confirm spike writeup states resolution path taken and CI is green |

*Identity, clippy, and coexistence build behaviors all have automated verification above.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (llama-cpp-2 dep)
- [ ] No watch-mode flags
- [ ] Feedback latency < 600s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
