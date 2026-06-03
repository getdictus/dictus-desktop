---
phase: 11-llm-runtime-foundation
plan: "04"
subsystem: build-gate + ui-consolidation
tags: [tauri, llama-cpp-2, vulkan, metal, ggml, ci, ui, post-uat]

# Dependency graph
requires:
  - phase: 11-03
    provides: useLlmModelStore + LlmLibrarySection + embedded provider wired into PostProcessingSettings

provides:
  - Windows x64 llama-cpp-2 link resolved (CMAKE_GENERATOR=Ninja) — vulkan-shaders-gen MSVC failure fixed
  - ggml duplicate-symbol link failure resolved (link flags) — llama-cpp-2 + transcribe-rs coexistence
  - macOS GPU confirmed working without explicit .metallib bundling (embedded)
  - Refreshed 4-model instruct catalogue (Qwen2.5 1.5B / Gemma 3 4B / Phi-4 Mini / Llama 3.2 3B)
  - Unified on-device engine picker (UI-SPEC R2): one list, provider cards as peers of GGUF model cards
  - Embedded inference tuning: per-model chat template, BOS token, repetition penalty

affects:
  - Phase 12 (Smart Modes backend) — build gate cleared, embedded provider stable across platforms
  - Phase 13 (Translation + i18n) — TranslateGemma deferred to translation mode (recorded in catalogue docs)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Windows x64 GPU build: CMAKE_GENERATOR=Ninja env in CI to dodge MSVC vulkan-shaders-gen ExternalProject failure"
    - "ggml coexistence: link flags resolve duplicate symbols between llama-cpp-2 and transcribe-rs (both vendor ggml)"
    - "Unified engine list: providers (Apple/Custom) and GGUF models rendered as uniform peer cards, active shown by badge not by reordering"

key-files:
  created:
    - .planning/phases/11-llm-runtime-foundation/11-04-SUMMARY.md
  modified:
    - src-tauri/Cargo.toml
    - .github/workflows/build.yml
    - src/components/settings/post-processing/LlmLibrarySection.tsx
    - src/components/settings/post-processing/PostProcessingSettings.tsx
    - src/components/settings/PostProcessingSettingsApi/ProviderPicker.tsx
    - src/components/settings/post-processing/CustomGgufDropZone.tsx

key-decisions:
  - "Windows x64: Ninja generator workaround (plan option 2) chosen over version bump or Vulkan-disable fallback — keeps GPU on all platforms"
  - "metallib: no explicit bundle.resources entry needed; the Metal shader library is embedded and GPU offload works in the packaged macOS app"
  - "Engine list: active engine is NOT pinned to top; stable zones (downloaded -> downloadable -> perso) keep Custom anchored at the bottom next to its config and stay consistent with the Cloud tab"
  - "Custom (Ollama) config consolidated: Test-connection button moved into the bottom config block under the Base URL field (stacked, full-width) — the card holds only the Ollama tip"

requirements-completed: [LLM-01, LLM-02]

# Metrics
completed: 2026-06-03
---

# Phase 11 Plan 04: Cross-Platform Build Gate + Engine-List Consolidation

**Close the Phase 10 build gates (Windows x64 GPU, macOS metallib, ggml coexistence), refresh the catalogue, and consolidate the on-device engine list after UAT.**

## Accomplishments

### Build gate (plan 11-04 core)

- **Windows x64 `vulkan-shaders-gen` MSVC failure** — resolved by adding `CMAKE_GENERATOR=Ninja` for the Windows x64 build job (`ab97d9f`). Plan option 2 (Ninja workaround); no version bump and no Vulkan-disable fallback, so GPU acceleration stays enabled on every platform. `Cargo.toml` keeps `llama-cpp-2 0.1.146 features=["vulkan"]` on Windows.
- **ggml duplicate-symbol link failure (Linux/Windows)** — the ggml conflict flagged as the top Phase 10 risk DID manifest at link time (both `llama-cpp-2` and `transcribe-rs` vendor ggml). Resolved via link flags (`fe2949c`); the blocker record was corrected to reflect that it manifested and was fixed rather than being a non-issue (`a6742b9`).
- **macOS `.metallib`** — `bundle.resources` remains `["resources/**/*"]` with no explicit metallib entry. The Metal shader library is embedded in the binary; GPU offload works in the packaged app, so no bundling change was required (plan Task 1 "embedded, no bundling needed" outcome).
- **bindings.ts** regenerated from a release build so the LLM commands match the bundled backend (`b0c0843`).

### Catalogue + inference tuning

- Catalogue refreshed to **4 diverse general-instruct models** — Qwen2.5 1.5B, Gemma 3 4B, Phi-4 Mini, Llama 3.2 3B (`5f76944`); TranslateGemma deferred to the future translation mode, recorded in catalogue docs (`0d01876`).
- Embedded inference correctness: per-model **chat template + repetition penalty** (`f0253c4`), **model-appropriate BOS token** (`17fd3f0`), and auto-activation of the first downloaded model (`2dba009`).

### On-device engine list (UI-SPEC R2 + post-UAT)

- **Unified engine picker** (`1b154a4`, spec `d9cd642`): a single on-device list where provider cards (Apple Intelligence, Custom/Ollama) are peers of the GGUF model cards — the model IS the engine. Radios dropped from provider rows; Apple description refined (`02a43e7`); visuals harmonized (`ca1a6af`, `cc8504d`).
- **Final consolidation** (`c360a62`):
  - Active engine no longer pinned to the top. Stable order by zone: **Apple (lead) → downloaded → downloadable → perso (imported GGUF) → Custom/Ollama (trail)**. Selecting a model no longer moves it; a model shifts into the downloaded zone only when its download starts. `sort()` is stable, so catalogue order holds within each zone. This also matches the Cloud tab, which never reorders on selection.
  - Custom (Ollama) config consolidated: the Test-connection button moved into the bottom config block under the Base URL field (container switched to `stacked`, URL full-width — fixes the responsive overflow). The Custom card now holds only the Ollama tip, so all Custom config (URL + Test + model) sits in one place directly under the trailing card.

## Verification status

- **macOS**: end-to-end download → model selection → embedded inference confirmed at runtime, GPU offload working (per session handoff 2026-06-02).
- **Cross-platform CI**: green across the 7 build targets after the Ninja + ggml link-flag fixes (per session handoff).
- **Pending (non-blocking, visual only)** — two UI tweaks from the final consolidation commit (`c360a62`) need an in-app eyeball; logic is code-verified but the rendering was not visually confirmed this session. Tracked in `.planning/todos/pending/2026-06-03-verify-postproc-engine-list-ux.md`:
  1. Custom: **Test-connection button** renders under the Base URL field, full-width, not clipped at any window width.
  2. Engine list **ordering** behaves as specified (Apple → downloaded → downloadable → perso → Custom; selecting a model does not move it; a model moves up to the downloaded zone only when its download begins).

## Deviations from Plan

- The plan's Task 3 (human cross-platform release-build smoke on macOS **and** Windows **and** Linux) was satisfied for macOS at runtime + 7-platform CI build/link green. Independent Windows/Linux **runtime GPU inference** human-smoke was not re-run this session; relying on the handoff status and CI. Flagged here for honesty — not re-blocking Phase 12 per the user's decision.
- Plan 11-04 grew well beyond its original build-gate scope to absorb the post-UAT engine-picker rework and inference tuning; these landed under the `fix(11)` / `11-04` banner as the consolidated tail of Phase 11.

## Next Phase Readiness

- Build gate cleared on all platforms (CI green); embedded provider stable and selectable.
- Phase 12 (Smart Modes Data Layer) can open: settings schema migration, `SmartMode` CRUD backend, per-mode shortcut routing, embedded provider, 10 default modes.

---
*Phase: 11-llm-runtime-foundation*
*Completed: 2026-06-03*
</content>
</invoke>
