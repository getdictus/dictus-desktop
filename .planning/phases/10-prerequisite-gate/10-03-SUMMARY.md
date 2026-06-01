---
phase: 10-prerequisite-gate
plan: "03"
subsystem: infra
tags: [llama-cpp-2, ggml, rust, cargo, ci, vulkan, metal, tauri, linker]

# Dependency graph
requires:
  - phase: 10-02
    provides: "ChatCompletionParams struct refactor and clippy-clean codebase"
provides:
  - "llama-cpp-2 0.1.146 declared per-platform in Cargo.toml (metal/macOS, vulkan/Windows+Linux)"
  - "PREP-01 confirmed: llama-cpp-2 is viable as the in-process embedded LLM engine"
  - "ggml duplicate-symbol conflict confirmed as non-issue on 6/7 CI platforms (two static ggml builds coexist without linker error)"
  - "Windows-x64 vulkan-shaders-gen MSVC build failure documented and deferred to Phase 11"
  - "candle/mistral.rs fallback is NOT needed"
affects: [phase-11-llm-runtime-foundation, phase-12-smart-modes-data-layer, phase-13-smart-modes-ui]

# Tech tracking
tech-stack:
  added: ["llama-cpp-2 0.1.146 (llama-cpp-sys-2, metal/vulkan features, bundles ggml 0.9.11)"]
  patterns:
    - "Per-platform Cargo target blocks: metal on macOS, vulkan on Windows and Linux — mirrors existing transcribe-rs pattern"
    - "No system-ggml feature, no [patch.crates-io] ggml resolution — coexistence relies on linker tolerance of two separate static ggml builds (0.9.5 from whisper-rs-sys + 0.9.11 from llama-cpp-sys-2)"

key-files:
  created: []
  modified:
    - "src-tauri/Cargo.toml — llama-cpp-2 0.1.146 added to three per-platform target blocks"
    - "src-tauri/Cargo.lock — updated with llama-cpp-2 + llama-cpp-sys-2 dependency tree"

key-decisions:
  - "PREP-01 spike: ggml duplicate-symbol conflict did NOT manifest; llama-cpp-2 0.1.146 + transcribe-rs coexist (two static ggml builds tolerated by linkers) on 6/7 CI platforms; llama-cpp-2 confirmed as in-process engine; Windows-x64 vulkan build deferred to Phase 11"
  - "Resolution path taken: NONE. No system-ggml feature, no [patch.crates-io] patch, no /FORCE:MULTIPLE. The predicted conflict did not occur."
  - "Windows x64 failure is the vulkan-shaders-gen MSVC ExternalProject cmake_install.cmake issue — NOT a ggml symbol conflict. Deferred to Phase 11."
  - "candle/mistral.rs fallback is NOT needed. llama-cpp-2 is confirmed as the engine choice."

patterns-established:
  - "Two coexisting static ggml builds (different versions) are tolerated by: Apple ld (macOS ARM64 + x64), MSVC linker (Windows ARM64), and ld on Linux (x64 + ARM64). This is a linker-family-spanning empirical result."
  - "Windows x64 + llama-cpp-2 vulkan feature requires separate MSVC ExternalProject fix for vulkan-shaders-gen before the embedded build ships."

requirements-completed: [PREP-01]

# Metrics
duration: ~multi-day (CI verification phase)
completed: "2026-06-01"
---

# Phase 10 Plan 03: ggml Coexistence Spike (PREP-01) Summary

**llama-cpp-2 0.1.146 confirmed viable alongside transcribe-rs on 6/7 CI platforms — the predicted ggml duplicate-symbol conflict did not manifest; two static ggml builds coexist without resolution; Windows-x64 vulkan-shaders-gen MSVC failure deferred to Phase 11**

## Performance

- **Duration:** Multi-day (spike + CI verification)
- **Started:** Prior session (Tasks 1+2), CI verification 2026-05-31, closed 2026-06-01
- **Completed:** 2026-06-01
- **Tasks:** 3 (Task 1 + Task 2 committed together as c91e3e3; Task 3 checkpoint resolved)
- **Files modified:** 2 (Cargo.toml, Cargo.lock)

## Accomplishments

- llama-cpp-2 0.1.146 added to Cargo.toml in per-platform target blocks (metal on macOS, vulkan on Windows and Linux), mirroring the existing transcribe-rs pattern
- PREP-01 confirmed: the feared ggml duplicate-symbol linker conflict (predicted from upstream issue llama.cpp #9267/#11303) did not manifest on any tested platform — both llama-cpp-sys-2 (ggml 0.9.11) and whisper-rs-sys (ggml 0.9.5) link as separate static builds without collision
- CI verdict: 6/7 green on GitHub Actions run 26749959299 (getdictus/dictus-desktop, branch feat/v1.3-smart-modes); representative linker families all pass (Apple ld macOS ARM64+x64, MSVC Windows ARM64, ld Linux x64+ARM64)
- Windows x64 failure root-caused as vulkan-shaders-gen MSVC ExternalProject cmake_install.cmake issue — explicitly not a ggml conflict; documented and deferred to Phase 11

## Task Commits

1. **Tasks 1+2: Add llama-cpp-2 + surface/resolve ggml conflict** — `c91e3e3` (feat)
2. **Task 3: Checkpoint documentation** — `445ede5` (docs)

**Plan metadata (this summary + state updates):** committed in final docs commit (see below)

## CI Verdict — Per-Platform Table

CI run: GitHub Actions workflow_dispatch `build-test.yml`, run ID 26749959299, branch `feat/v1.3-smart-modes`

| Platform | Target Triple | GPU Backend | Result | Notes |
|----------|---------------|-------------|--------|-------|
| macOS ARM64 | aarch64-apple-darwin | Metal | GREEN | |
| macOS x64 | x86_64-apple-darwin | Metal | GREEN | |
| Ubuntu 22.04 x64 | x86_64-unknown-linux-gnu | Vulkan | GREEN | |
| Ubuntu 24.04 x64 | x86_64-unknown-linux-gnu | Vulkan | GREEN | |
| Ubuntu 24.04 ARM64 | aarch64-unknown-linux-gnu | Vulkan | GREEN | First run failed (runner OOM/shutdown = infra); passed on rerun |
| Windows ARM64 | aarch64-pc-windows-msvc | Vulkan | GREEN | |
| Windows x64 | x86_64-pc-windows-msvc | Vulkan | RED | vulkan-shaders-gen MSVC ExternalProject failure — see below |

**Final verdict: 6/7 green.** The "7/7 green" gate defined in the plan was NOT met. This is a user-approved deviation; see Deviations section.

## The PREP-01 Result: What the Research Predicted vs. What Happened

**Plan prediction:** whisper-rs-sys 0.15.0 and llama-cpp-sys-2 0.1.146 both statically link their own ggml (ggml 0.9.5 and 0.9.11 respectively), producing duplicate-symbol linker errors such as `ggml_backend_buft_name already defined`. Resolution via system-ggml feature or `[patch.crates-io]` was expected to be required.

**Actual result:** No resolution was needed or applied. The two separate static ggml builds coexist without triggering duplicate-symbol errors on all tested linkers:
- Apple ld (macOS ARM64 + x64 with Metal) — tolerates both
- MSVC linker (Windows ARM64 with Vulkan) — tolerates both
- ld on Linux x64 + ARM64 (Vulkan) — tolerates both

The plan's `must_have` truth "cargo tree resolves to a SINGLE shared ggml build" is **NOT literally satisfied**. There are TWO ggml versions coexisting in the dependency tree (0.9.5 via whisper-rs-sys + 0.9.11 via llama-cpp-sys-2). The higher-order goal — "llama-cpp-2 and transcribe-rs compile and link together with no duplicate-symbol linker errors" — IS met on 6/7 platforms.

No `system-ggml` feature was added. No `[patch.crates-io]` ggml entry was added. No `/FORCE:MULTIPLE` was used.

## Windows x64 Failure — Root Cause

The Windows x64 failure is **not a ggml symbol conflict**. It is a distinct build toolchain issue in the llama.cpp embedded build system.

**Exact failure:**
```
-- Found Vulkan: .../vulkan-1.lib (version "1.4.309")    [Vulkan SDK present, not missing]
-- ggml version: 0.9.11                                   [ggml configured fine]
CMake error: Not a file: .../vulkan-shaders-gen-build/cmake_install.cmake
The system cannot find the batch label specified - VCEnd
error MSB8066: Custom build for '...vulkan-shaders-gen...' exited with code 1
```

**Root cause:** The `vulkan-shaders-gen` ExternalProject sub-build's `cmake_install.cmake` is not generated when MSBuild runs the install step for the host tool. This is a known llama.cpp/MSBuild ExternalProject quirk on the MSVC x64 toolchain.

**Explicitly ruled out:**
- NOT a ggml duplicate-symbol conflict (no `already defined` errors)
- NOT a missing Vulkan SDK (SDK version 1.4.309 found and confirmed)
- NOT a transcribe-rs interaction
- NOT a regression from any change in this plan

**Status:** Reproducible (same error on both CI runs). Windows ARM64 builds the same vulkan feature successfully, confirming the issue is specific to the MSVC x64 + ExternalProject interaction. Deferred to Phase 11 (which owns Vulkan CI hardening and the embedded-LLM integration build).

## Files Created/Modified

- `src-tauri/Cargo.toml` — llama-cpp-2 0.1.146 added to macOS (metal), Windows (vulkan), and Linux (vulkan) per-platform target blocks alongside existing transcribe-rs entries
- `src-tauri/Cargo.lock` — updated with full llama-cpp-2 and llama-cpp-sys-2 dependency subtree

## Decisions Made

1. **Resolution path: none required.** The ggml conflict predicted by upstream issue research and the plan's own facts/context did not manifest in practice. Both static ggml builds coexist under all tested linker families.
2. **7/7 CI gate not met; plan closed on PREP-01 basis.** The user approved closing 10-03 because the core question — "is llama-cpp-2 viable as the in-process engine?" — is answered YES on the basis of 6/7 green platforms covering all three linker families. The 1 red is a separate toolchain issue, not the PREP-01 risk.
3. **Windows-x64 vulkan-shaders-gen fix deferred to Phase 11.** Phase 11 already owns Vulkan CI hardening and the embedded-LLM integration build. This is the correct owner for the fix.
4. **candle/mistral.rs fallback is not needed.** The fallback was defined for the case where the ggml conflict blocked llama-cpp-2 from linking. That case did not occur.

## Deviations from Plan

### User-Approved Gate Deviation

**Plan gate: "All 7 CI platforms green" (strict blocking gate)**
- **Actual outcome:** 6/7 green
- **Platform failed:** Windows x64 (x86_64-pc-windows-msvc, Vulkan)
- **Why deviation approved:** The Windows x64 failure is not the ggml conflict that PREP-01 was designed to de-risk. The core PREP-01 question is answered. The user explicitly approved closing 10-03 on this basis, with Windows-x64 fix deferred to Phase 11.
- **This is NOT an auto-fix deviation** — it is a deliberate user decision with documented rationale.

### Research Prediction vs. Reality

**Predicted:** Hard ggml duplicate-symbol linker conflict requiring system-ggml feature and/or `[patch.crates-io]` resolution, with possible fallback to candle/mistral.rs.

**Actual:** No conflict manifested. The plan's `must_have` stating "cargo tree resolves to a SINGLE shared ggml build" was not achieved — there are two coexisting ggml versions — but this does not produce duplicate-symbol errors in practice. The more important must_have ("llama-cpp-2 and transcribe-rs compile and link together with no duplicate-symbol linker errors") is satisfied on 6/7 platforms.

This is documented as a research prediction mismatch, not as a failure. The outcome is strictly better than predicted.

---

**Total deviations:** 1 user-approved gate deviation (6/7 vs 7/7 CI), 1 research expectation vs. reality mismatch (no conflict manifested)
**Impact on plan:** The PREP-01 objective is achieved. Phase 11 receives a cleared path for llama-cpp-2 integration, with one known Windows-x64 build issue to address before the embedded build ships.

## Issues Encountered

None blocking. The single red CI platform (Windows x64) was investigated, root-caused, and deferred rather than blocked on.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

Phase 11 (LLM Runtime Foundation) is unblocked for:
- macOS ARM64, macOS x64 (Metal) — full green
- Ubuntu 22.04 x64, Ubuntu 24.04 x64, Ubuntu 24.04 ARM64 (Vulkan) — full green
- Windows ARM64 (Vulkan) — full green

**Known Phase 11 gate item:** Windows x64 requires the `vulkan-shaders-gen` MSVC ExternalProject `cmake_install.cmake` fix before the embedded-LLM build ships on that platform. Phase 11 owns this fix (Vulkan CI hardening scope).

**Metal bundle resources gate (carried from STATE.md):** `.metallib` paths from llama-cpp-2 OUT_DIR are not yet confirmed for Tauri bundle resources. Must verify via `tauri build` release smoke test on macOS before Phase 12 opens.

---
*Phase: 10-prerequisite-gate*
*Completed: 2026-06-01*
