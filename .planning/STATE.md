---
gsd_state_version: 1.0
milestone: v1.3
milestone_name: Smart Modes & Local LLM
status: completed
stopped_at: Phase 11 context gathered
last_updated: "2026-06-01T13:37:50.487Z"
last_activity: 2026-06-01 — 10-03 closed; PREP-01 confirmed; ggml coexistence proven 6/7 CI platforms; Windows-x64 vulkan deferred to Phase 11
progress:
  total_phases: 4
  completed_phases: 1
  total_plans: 3
  completed_plans: 3
  percent: 8
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-29 after starting milestone v1.3)

**Core value:** Local-first as a visible, powerful default — embedded LLM runtime + Smart Modes + multi-target translation, all in-process, all platforms.
**Current focus:** v1.3 Smart Modes & Local LLM — roadmap ready, Phase 10 next.

## Current Position

Phase: 10 of 13 (Prerequisite Gate) — COMPLETE (3/3 plans)
Plan: — (Phase 10 done; Phase 11 next)
Status: Phase 10 complete; Phase 11 (LLM Runtime Foundation) is the next phase
Last activity: 2026-06-01 — 10-03 closed; PREP-01 confirmed; ggml coexistence proven 6/7 CI platforms; Windows-x64 vulkan deferred to Phase 11

Progress: [██░░░░░░░░] ~8%

## Performance Metrics

**Velocity (v1.2 reference):**
- Total v1.2 plans completed: 16
- Median plan duration: ~5-30 min (range 2 min to multi-day validation)

**By Phase (v1.2):**

| Phase | Plans | Notes |
|-------|-------|-------|
| 6. Brand & Icon Polish | 4 | Icon rasterization regression fixed post-close |
| 7. macOS Clean Shutdown | 1 | Multi-day validation window |
| 8. Privacy / Local-First UX | 10 | UAT iteration cycle included |
| 9. Audit Gap Closure | 1 | 14 min |
| Phase 10 P01 | 9 | 2 tasks | 17 files |
| Phase 10-prerequisite-gate P02 | 2 | 2 tasks | 3 files |
| Phase 10-prerequisite-gate P03 | multi-day | 3 tasks | 2 files |

## Accumulated Context

### Decisions

- v1.3: Phase 14 (polish) is NOT a standalone phase — essentials absorbed into Phase 11 (disk size before download, MDL-01) and Phase 13 (shortcut conflict detection, MODE-05). Fit badge, quantization labels, GPU badge deferred to Future Requirements (MDL-F2/F3/F4).
- v1.3: Within Phase 10, execution order: PREP-03 (Sync #2) first → PREP-02 (TECH-04 refactor) → PREP-01 (ggml spike). Rationale: sync may touch llm_client.rs; refactor on up-to-date base.
- v1.3: MODE-03 spans Phase 12 (backend) and Phase 13 (UI). Requirement assigned to Phase 13 (first phase where user can observe the full capability).
- v1.3: All platforms ship together (no macOS-first staging). Vulkan SDK added to Windows/Linux CI in Phase 11.
- v1.3: Cloud stays explicit opt-in. Embedded is primary local option. No silent cloud fallback ever.
- [Phase 10]: Sync #2 is the LAST bulk catch-up merge; selective cherry-pick policy going forward per UPSTREAM.md Fork Policy section
- [Phase 10]: aee682f (AWS Bedrock) reverted and logged as Exclusion Entry #1 in UPSTREAM.md Exclusion Log (local-first principle)
- [Phase 10]: 966ff99 (async GPU query) accepted in Sync #2 merge but NOT adopted — flagged forward to Phase 11 GPU detection
- [Phase 10-02]: Kept send_chat_completion thin wrapper rather than inlining — lowest-risk, call site 2 unchanged
- [Phase 10-02]: Added Default derive to PostProcessProvider (all fields String/bool/Option — empty default safe for internal use)
- [Phase 10-03]: PREP-01 spike: ggml duplicate-symbol conflict did NOT manifest; llama-cpp-2 0.1.146 + transcribe-rs coexist (two static ggml builds tolerated by linkers) on 6/7 CI platforms; llama-cpp-2 confirmed as in-process engine; candle/mistral.rs fallback not needed; Windows-x64 vulkan-shaders-gen MSVC build failure deferred to Phase 11

### Pending Todos

3 pending — see `.planning/todos/pending/` for details.

### Blockers/Concerns

- ~~**ggml symbol conflict (Phase 10 spike):**~~ **RESOLVED (10-03):** Conflict did not manifest; llama-cpp-2 + transcribe-rs coexist with two separate static ggml builds (0.9.5 + 0.9.11) tolerated by all tested linkers; no resolution path needed. 6/7 CI platforms green.
- **Windows x64 + llama-cpp-2 vulkan-shaders-gen MSVC build failure (Phase 11 gate):** CI run 26749959299 — vulkan-shaders-gen ExternalProject cmake_install.cmake not generated when MSBuild runs install step on MSVC x64 toolchain; NOT a ggml conflict; NOT a missing Vulkan SDK; must be fixed before Phase 11 Windows-x64 embedded-LLM build ships. Windows ARM64 builds fine.
- **Metal bundle resources (Phase 11 gate):** `.metallib` paths from `llama-cpp-2` OUT_DIR not documented; verify via `tauri build` release smoke test on macOS before Phase 12 opens.
- **AMD Vulkan driver (Phase 11 risk):** AMD driver 25.11.1 has known crash with Vulkan SDK 1.4.328.1 (May 2026). Monitor llama.cpp issue #17432 before Windows beta.
- Carried from v1.2: `blob.handy.computer` CDN for onnxruntime (INFR-01); Windows unsigned builds (INFR-03); Nyquist VALIDATION.md drafts for phases 5-9.

## Session Continuity

Last session: 2026-06-01T13:37:50.480Z
Stopped at: Phase 11 context gathered
Resume file: .planning/phases/11-llm-runtime-foundation/11-CONTEXT.md
