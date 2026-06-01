---
phase: 11
slug: llm-runtime-foundation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-01
---

# Phase 11 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` (backend) + ESLint/tsc (frontend) |
| **Config file** | `src-tauri/Cargo.toml` (test targets) |
| **Quick run command** | `cd src-tauri && cargo test --lib` |
| **Full suite command** | `cd src-tauri && cargo test && cd .. && bun run lint` |
| **Estimated runtime** | ~60 seconds (excludes cross-platform build smoke gate) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run full suite (`cargo test && bun run lint`)
- **Before `/gsd:verify-work`:** Full suite must be green + cross-platform `tauri build` smoke gate passed
- **Max feedback latency:** ~60 seconds

---

## Per-Task Verification Map

> Populated by gsd-planner from the Validation Architecture section in 11-RESEARCH.md.
> Each plan task maps to a requirement and an automated command (or Wave 0 stub / manual gate).

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| TBD | — | — | LLM-01..04, MDL-01..05 | — | — | — | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

> Populated during planning. Likely test stubs for `LlmManager` catalogue parsing,
> SHA256 verification, and settings serialization.

- [ ] Catalogue metadata unit tests (filename/URL/SHA256 fields present)
- [ ] SHA256 verification test (reuse model.rs test pattern)
- [ ] Settings serde round-trip test for new `active_llm_model_id` / `llm_unload_timeout` fields

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| GPU acceleration (Metal/Vulkan) + CPU fallback | LLM-02 | Requires real GPU hardware per platform | Run on macOS/Windows/Linux; confirm GPU layers offloaded in logs, CPU fallback when no GPU |
| End-to-end download → load → inference → streaming tokens | LLM-01..04, MDL-01..05 | Requires running app + real model download per OS | Cross-platform `tauri build` smoke gate on macOS/Windows/Linux |
| Overlay/shortcuts non-blocking during inference | LLM-03 | Requires interactive UI observation | Trigger inference, confirm overlay + shortcuts remain responsive |
| Idle-timeout model unload without affecting transcription model | LLM-04 | Requires timing observation + memory inspection | Load model, idle past timeout, confirm unload via logs; transcription still works |
| Drag/drop custom GGUF import | MDL-04 | Requires interactive file drop | Drop a GGUF file onto library; confirm it appears alongside catalogue |

*macOS `.metallib` bundle-resource discovery is a build-time gate, resolved via `tauri build` (not `tauri dev`).*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
