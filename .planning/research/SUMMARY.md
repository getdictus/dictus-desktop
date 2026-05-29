# Project Research Summary

**Project:** Dictus Desktop v1.3 — Smart Modes & Local LLM
**Domain:** Embedded LLM runtime + per-mode shortcut binding + multi-target translation in a shipping Tauri 2.x desktop app
**Researched:** 2026-05-29
**Confidence:** MEDIUM-HIGH overall (HIGH on codebase reuse paths, build system, and settings migration; MEDIUM on runtime stability, Vulkan maturity, and translation quality)

---

## Executive Summary

Dictus Desktop v1.3 is an extension milestone, not a rewrite. All four research threads independently confirmed that ~80% of the implementation reuses existing infrastructure: the `ModelManager` download pipeline, the `ModelUnloadTimeout` lifecycle mechanism, the dynamic shortcut register/unregister system, and the `PostProcessProvider` abstraction are all directly extensible. The core deliverable is three new components wired into the existing architecture: an in-process GGUF inference engine (`LlmRuntimeManager`), a parallel model catalogue/downloader (`LlmModelManager`), and a settings schema extension that promotes `LLMPrompt` to `SmartMode` with per-mode shortcut bindings. The recommended LLM runtime is `llama-cpp-2` (Rust bindings over llama.cpp) with Metal auto-enabled on macOS and Vulkan feature-gated on Windows/Linux.

The single most critical architectural constraint is also the single highest-risk item: `llama-cpp-2` and the existing `transcribe-rs` crate both vendor `ggml` internally. A linker symbol conflict (`duplicate definition of ggml_init`) is a realistic build failure on first integration. This is not a blocker — it has documented resolution paths — but it must be treated as a feasibility spike before any other v1.3 code is written. The second highest-risk item is settings migration: the `post_process_prompts` + `post_process_selected_prompt_id` schema must be explicitly migrated to `smart_modes` + `post_process_selected_mode_id`, or existing users silently lose their configured prompts and shortcuts on upgrade. Both risks are fully preventable with the preparation steps described in the research.

The recommended build order that all four research files independently converged on: (1) feasibility spike on `llama-cpp-2` build alongside `transcribe-rs` + TECH-04 `llm_client.rs` refactor + upstream Sync #2 as prerequisites; (2) runtime foundation — `LlmModelManager`, `LlmRuntimeManager`, CI Vulkan SDK setup; (3) Smart Modes data layer — settings migration, `SmartMode` type, per-mode shortcut registration, embedded provider branch in `llm_client.rs`; (4) Smart Modes UI + translation presets + i18n; (5) polish — memory advisory, conflict feedback, quantization labels. Whisper's translate task is English-output only, so multi-target translation is only possible through the LLM post-processing step — this makes the embedded runtime directly enabling for the offline translation use case.

---

## Key Findings

### Recommended Stack

The core stack is unchanged (Tauri 2.x, Rust, React 18/TypeScript, Tailwind, Zustand, Vite). The only new Cargo dependency is `llama-cpp-2 = "0.1"` with conditional GPU feature gates following the existing `transcribe-rs` pattern: Metal is automatic on macOS (no feature flag needed — unconditional framework link in `llama-cpp-sys-2/build.rs`), Vulkan is feature-gated on Windows/Linux, CUDA is deferred to v1.4. CI runners need Vulkan SDK added for Windows and Linux jobs (`humbletim/install-vulkan-sdk@v1.2.0` on Windows; `libvulkan-dev + spirv-headers` on Linux). `SPIRV-Headers` must be installed separately — it is NOT pulled in by `libvulkan-dev` alone.

Candle (pure Rust, HuggingFace) and mistral.rs were evaluated and rejected. Candle's GGUF support is partial and unreliable for arbitrary community models; it has no Vulkan backend; it requires per-architecture inference code. Mistral.rs inherits candle's GGUF limitations, its Vulkan quality is unverified (single source, LOW confidence), and its server-first design requires excessive embedding glue. `llama-cpp-2` is the only option that provides full GGUF compatibility, automatic Metal, Vulkan support, and a minimal integration surface.

**Core new technologies:**
- `llama-cpp-2 = "0.1"` (crates.io): in-process GGUF inference — canonical Rust binding for llama.cpp, stays current with every llama.cpp release, Apache-2.0/MIT
- HuggingFace CDN (direct): model distribution — free, supports HTTP range requests, globally available; explicitly NOT `blob.handy.computer` (INFR-01 hard line)
- `tauri-plugin-global-shortcut = "2.3.1"` (already installed): per-mode shortcut binding — existing API, no changes needed
- `sysinfo` crate (optional, P2): hardware fit badge — total RAM detection for model size recommendation

**Existing infrastructure reused without modification:**
- `managers/model.rs`: SHA256 verify, partial-resume download, tar.gz extraction, cancel flags, progress events
- `settings.rs` `ModelUnloadTimeout` enum: idle-watcher lifecycle for both Whisper and LLM managers
- `shortcut/tauri_impl.rs` `register_shortcut` / `unregister_shortcut`: dynamic registration proven working

### Expected Features

**Must have (table stakes):**
- In-app model downloader with progress bar, cancel, and delete — Whisper picker set this expectation; users will not accept a separate download process
- Show disk size before download — Jan/LM Studio both do this; 2.5 GB surprises are unacceptable
- Hardware fit badge ("Fits" / "May be slow" / "Insufficient memory") — Jan pioneered this; critical on 8 GB machines
- Per-mode shortcut binding — this is the core missing link; without it users must open settings to switch modes
- Visual mode list (cards, not dropdown) — dropdown does not communicate "mode library"
- 7-10 pre-loaded default Smart Modes including translation presets — blank slate is intimidating; "Clean Up" should be the safe default and first mode
- Settings schema migration — existing users must not lose their configured prompts or shortcuts

**Should have (competitive):**
- Quantization tier labels (Small / Balanced / Quality) mapping to Q4_K_M / Q5_K_M / Q8_0 — avoids exposing GGUF suffixes to non-expert users
- Shortcut conflict detection with inline warning — Superwhisper only fixed silent failures in v2.13.1; proactive detection is a clear differentiator
- GPU/CPU status badge in model picker — users need to understand performance expectations
- Translation modes visually grouped as a sub-category — elevates translation as first-class concept

**Defer to v1.x post-validation:**
- Auto-quantization recommendation based on detected RAM
- Mode enable/disable toggle without deletion
- Export/import modes as JSON

**Defer to v2+:**
- Per-mode provider selection (different LLM per mode)
- Auto-activation by active app (accessibility permissions required on macOS)
- Community mode gallery / templates
- Open HuggingFace catalog integration
- Advanced GPU layer configuration (n_gpu_layers slider)

**Anti-features to explicitly avoid:**
- Running the LLM as a separate llama-server process — contradicts "no external process" pitch
- Cloud provider as fallback when no local model is downloaded — violates local-first philosophy
- GGUF model hosting on `blob.handy.computer` — availability and cost risk not owned by Dictus
- Silent settings migration that drops existing prompts — data loss on upgrade

### Architecture Approach

The architecture follows the established Manager pattern strictly. Two new managers are added alongside the existing four: `LlmModelManager` (catalogue + downloader, mirrors `model.rs` patterns exactly with separate `<app_data>/llm_models/` directory) and `LlmRuntimeManager` (GGUF inference engine, mirrors `TranscriptionManager` lifecycle). The inference call runs on `tokio::task::spawn_blocking` — the same pattern `TranscriptionManager` already uses for Whisper inference — to avoid blocking the async executor. Token streaming uses `app_handle.emit("llm-token", ...)` events. A new `"embedded"` provider branch is added to `llm_client.rs` that calls `LlmRuntimeManager` instead of the HTTP path; all existing HTTP paths are unchanged. `actions.rs` gains a `resolve_mode_id()` helper that routes `smart_mode_{id}` binding IDs to the correct `SmartMode` prompt. `TranscriptionCoordinator.is_transcribe_binding()` is extended to accept the `smart_mode_` prefix. The entire shortcut infrastructure handles Smart Mode shortcuts transparently via `smart_mode_{id}` key naming.

**New files:**
1. `managers/llm_model.rs` — GGUF catalogue + downloader, mirrors `model.rs` exactly
2. `managers/llm_runtime.rs` — in-process inference, `spawn_blocking`, idle watcher, token streaming
3. `commands/llm_models.rs` — Tauri command handlers for LLM model management
4. `commands/smart_modes.rs` — Tauri command handlers for SmartMode CRUD + shortcut binding
5. `components/smart-modes/` — list + edit SmartModes, shortcut binding per mode
6. `components/llm-model-picker/` — LLM model download/select UI (mirrors model-selector)

**Modified files (summary):**
- `settings.rs`: `SmartMode` type, 4 new `#[serde(default)]` fields, migration function in loader
- `llm_client.rs`: `"embedded"` branch + TECH-04 struct refactor
- `actions.rs`: `mode_id: Option<&str>` param, `resolve_mode_id()` helper
- `transcription_coordinator.rs`: `is_transcribe_binding()` extended for `smart_mode_` prefix
- `shortcut/mod.rs`: `register_all_smart_mode_shortcuts()`, conflict detection
- `lib.rs`: `.manage()` two new managers, call smart mode registration at init

### Critical Pitfalls

1. **ggml symbol conflict between `transcribe-rs` and `llama-cpp-2`** — Both vendor ggml internally. Linker `duplicate definition of ggml_init` is a realistic first-build failure. Run `cargo tree | grep ggml` before and after adding the dep; use `[patch.crates-io]` to force a single ggml source if conflict occurs. Treat as a feasibility spike — verify clean build before any feature code.

2. **Settings migration data loss** — `post_process_prompts` to `smart_modes` rename is a structural schema change. `#[serde(default)]` only handles additive additions; renamed/restructured fields are silently dropped. Prevention: explicit `settings_schema_version: u32` field, migration function in `load_or_create_app_settings()`, and a test loading a v1.2 settings JSON to verify prompts + shortcut are preserved.

3. **TECH-04 collision with v1.3 LLM work** — `llm_client.rs:137 send_chat_completion_with_schema` has 8 arguments, suppressed by `#[allow(clippy::too_many_arguments)]`. Every new Smart Modes call site written before the refactor triggers the warning. Resolution: resolve TECH-04 as the first task of v1.3, before any Smart Modes code is written.

4. **Inference blocks the Tauri main thread** — llama.cpp inference is a synchronous blocking loop. Without `spawn_blocking`, inference freezes the overlay, tray menu, and shortcut responsiveness for 5-30 seconds on CPU. Prevention: always `tokio::task::spawn_blocking` for inference; implement an `Arc<AtomicBool>` cancellation token checked between tokens; stream tokens via `app_handle.emit("llm-token", ...)`.

5. **Upstream Sync #2 merge complexity grows if deferred** — v1.3 adds substantial new LOC to files in the upstream conflict zone (`llm_client.rs`, `managers/`, `settings.rs`). Execute Sync #2 before v1.3 feature work begins as a prerequisite gate.

6. **Metal shaders missing from macOS release bundle** — `llama-cpp-2` compiles Metal shaders. Tauri does not auto-include resources from Cargo dependency source trees. The release `.app` bundle will silently fall back to CPU inference on macOS unless `.metallib` files are added to `tauri.conf.json` `bundle.resources`. Test with `tauri build` (not `tauri dev`) as part of Phase 11 verification gate.

7. **GGUF on blob.handy.computer is a hard line** — Multi-GB GGUF weights must never be hosted on `blob.handy.computer`. Use HuggingFace CDN URLs directly. INFR-01 resolution is a prerequisite for v1.3, not a deferred cleanup.

---

## Implications for Roadmap

Based on combined research, the following phase structure is recommended. Phases 10-14 continue from v1.2 which ended at Phase 9.

### Phase 10: Prerequisite Gate

**Rationale:** Three hard blockers must be resolved before any v1.3 feature code is written. The feasibility spike on `llama-cpp-2` build compatibility with `transcribe-rs` is the single highest-risk item in the milestone — discovering a ggml symbol conflict at Phase 13 would require architectural rework. TECH-04 must be resolved first because every new `llm_client.rs` call site written in later phases should use the clean struct API. Sync #2 must happen before the codebase diverges further.

**Delivers:** Green CI confirming `llama-cpp-2` compiles alongside `transcribe-rs` on all 7 platforms; `send_chat_completion_with_schema` refactored to `ChatCompletionRequest` struct; `cargo clippy --all-targets -- -D warnings` passes clean; upstream Sync #2 merged.

**Avoids:** V3-B1 (ggml symbol conflict discovered late), V3-B5 (tauri-runtime patch conflict), V3-L3 (TECH-04 collision), V3-L2 (upstream merge complexity doubling)

**Needs research-phase:** No — tasks are well-defined.

---

### Phase 11: LLM Runtime Foundation

**Rationale:** Dependency root. Smart Modes, model picker UI, and translation presets all require a working inference engine and model downloader before end-to-end validation is possible. Building the runtime first surfaces CI/packaging issues (Vulkan SDK setup, Metal bundle resources) before UI work depends on them.

**Delivers:** Working end-to-end path: download a GGUF, load it, run inference from a test Tauri command, receive streaming token events in frontend console. Model picker stub UI. Verification gate: full round-trip confirmed on all 3 platforms before Phase 12 opens.

**Implements:** `LlmModelManager`, `LlmRuntimeManager`, `commands/llm_models.rs`, `components/llm-model-picker/` (download/progress/delete), `lib.rs` wiring, CI Vulkan SDK steps.

**Models for initial catalog:** `qwen3-4b` Q4_K_M (~2.5 GB) as primary; `qwen2.5-1.5b` Q4_K_M (~1 GB) as low-RAM fallback; `translategemma-4b-it` Q4_K_M (~2.49 GB) as optional translation-specific. All Apache-2.0 licensed. All URLs from HuggingFace CDN.

**Avoids:** V3-R1 (inference blocking UI — `spawn_blocking` required), V3-R2 (Whisper + LLM OOM — sequential pipeline), V3-R3 (in-process crash — GGUF validation before load), V3-B2 (Metal shaders missing — release build smoke test), V3-D1 (download not resumable — range-request resume required), V3-D4 (GGUF on blob.handy.computer)

**Needs research-phase:** No — architecture fully specified in ARCHITECTURE.md.

---

### Phase 12: Smart Modes Data Layer + Settings Migration

**Rationale:** The data model must be established and migration-tested before any UI depends on it. Shipping a settings migration bug is irreversible without a hotfix. The embedded provider branch in `llm_client.rs` belongs here because it closes the loop between the runtime (Phase 11) and Smart Modes routing logic.

**Delivers:** `SmartMode` struct and schema migration (`post_process_prompts` to `smart_modes`) with migration test; `"embedded"` provider branch in `llm_client.rs`; per-mode shortcut binding infrastructure (`register_all_smart_mode_shortcuts()`, `is_transcribe_binding()` extension, `resolve_mode_id()` helper); 10 built-in default Smart Modes (Clean Up as recommended default, Make Formal, Make Casual, Write as Email, Write as SMS, Bullet Points, Summarize, Translate to English, Translate to Spanish, Translate to French).

**Avoids:** V3-S1 (settings migration data loss), V3-S2 (silent shortcut registration failure — emit registration result event), V3-L1 (cloud made prominent — embedded runtime is first in provider list)

**Needs research-phase:** No — migration pattern and shortcut routing fully specified.

---

### Phase 13: Smart Modes UI + Translation Presets + i18n

**Rationale:** UI work can only proceed after the data layer is stable. Translation presets are built-in `SmartMode` entries requiring no additional code beyond Phase 12. Main work is the visual redesign (cards replacing dropdown), the shortcut binding component with conflict detection, and i18n propagation across 20 locales.

**Delivers:** `components/smart-modes/` list + edit + shortcut-bind UI; translation modes grouped with visual label; `useSmartModes.ts` hook; settings UI Local tab entry for embedded provider + LLM model picker; all new i18n keys propagated to 20 sibling locales. Verification gate: full end-to-end flow — record, transcribe, Smart Mode fires correct prompt, embedded LLM responds, output pasted.

**Avoids:** V3-S3 (i18n of default mode names — propagate in same PR as mode definitions), V3-S4 (translation quality misrepresented — quality label in UI), V3-S5 (prompt regression on small models — test each default prompt against curated local models), V3-L1 (cloud made prominent — local runtime is default option)

**Needs research-phase:** No — competitive patterns documented in FEATURES.md; i18n is mechanical.

---

### Phase 14: Polish + Memory Advisory + Conflict Feedback

**Rationale:** Polish phase addresses UX gaps that only become apparent with a working system. These are lower-risk and can be iterated after initial beta feedback.

**Delivers:** Hardware fit badge (green/yellow/red) based on system RAM vs model RAM requirement; `llm_model_unload_timeout` setting exposed in UI; shortcut conflict detection with inline warning badge; quantization tier labels (Small / Balanced / Quality); GPU/CPU status badge.

**Avoids:** V3-D2 (Windows antivirus guidance on first download), V3-D3 (model license displayed — Apache/MIT-only in initial library)

**Needs research-phase:** No — `sysinfo` crate usage is straightforward; all patterns documented.

---

### Phase Ordering Rationale

- Phase 10 (prerequisites) must come before all feature work — the ggml feasibility spike and TECH-04 refactor define the integration constraints for everything that follows
- Phase 11 (runtime) must come before Phase 12 (data layer) — the settings migration introduces an `"embedded"` provider ID that must resolve to a working runtime at integration testing time
- Phase 12 (data layer) must come before Phase 13 (UI) — React components depend on Tauri command signatures and `SmartMode` type bindings auto-generated by tauri-specta
- Phase 14 (polish) can be parallelized with Phase 13 testing in practice but logically depends on Phases 11-13 being stable
- Sync #2 is gated before Phase 11 to minimize merge surface for `llm_client.rs` and `managers/`

### Research Flags

Phases requiring deeper research or spikes during planning:
- **Phase 10 (feasibility spike):** The `llama-cpp-2` + `transcribe-rs` ggml conflict resolution path cannot be determined until `cargo tree | grep ggml` is run against the actual Cargo.lock. The exact resolution approach (shared ggml via `[patch.crates-io]`, separate feature flags, or alternative crate) depends on what the spike reveals.
- **Phase 11 (Metal bundle resources):** The exact `.metallib` files that `llama-cpp-2` places in `OUT_DIR` and which ones must be copied to `bundle.resources` are not confirmed in any documentation. First release build on macOS is the verification gate.

Phases with standard patterns (skip research-phase):
- **Phase 12:** Settings migration and shortcut routing fully specified with code examples in ARCHITECTURE.md. The `settings_schema_version` + migration function pattern is unambiguous.
- **Phase 13:** All UI patterns mirror existing components (model-selector, shortcut input). i18n propagation is a mechanical process.
- **Phase 14:** `sysinfo` crate usage is well-documented; hardware fit badge logic is arithmetic.

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | `llama-cpp-2` verified against official build.rs source; GPU feature flags confirmed against actual codebase patterns; model sizes verified from HuggingFace model cards |
| Features | MEDIUM-HIGH | Competitive landscape verified via live Superwhisper/Jan/LM Studio sources; Whisper translate English-only constraint confirmed from OpenAI docs |
| Architecture | HIGH | Grounded in actual codebase reading; new component designs directly mirror existing verified patterns; all integration points confirmed working |
| Pitfalls | HIGH (risk identification) / MEDIUM (mitigations) | Build/signing/settings pitfalls from direct codebase analysis; Vulkan stability on Windows/Linux has contradictory signals; translation quality degradation from academic sources |

**Overall confidence:** MEDIUM-HIGH

### Gaps to Address

- **ggml symbol conflict resolution path** — Cannot be determined without running `cargo build` with both crates active. Treat Phase 10 as a feasibility spike; have the `[patch.crates-io]` approach ready as fallback. If a `candle`-based alternative becomes necessary, it changes Cargo.toml additions significantly.
- **Metal bundle resource requirements** — The exact `.metallib` paths that `llama-cpp-2` places in `OUT_DIR` and which ones must be in `bundle.resources` are not confirmed. First `tauri build` release smoke test on macOS is the only way to verify. Explicit Phase 11 verification gate.
- **Vulkan quality on Windows with AMD GPUs** — AMD driver 25.11.1 has a known crash with Vulkan SDK 1.4.328.1 (May 2026). Monitor `llama-cpp-2` issue tracker before shipping Windows beta.
- **TranslateGemma 4B vs. Qwen3-4B translation quality** — No head-to-head benchmark found for the target language pairs. Recommendation: ship Qwen3-4B as default (already downloaded for other modes), offer TranslateGemma as optional download. Validate after first beta.
- **Upstream Sync #2 scope** — The AWS Bedrock commit (`aee682f`) excluded from Sync #1 per local-first philosophy needs a decision before Sync #2 merging.

---

## Sources

### Primary (HIGH confidence)
- Codebase direct reading — `managers/model.rs`, `managers/transcription.rs`, `llm_client.rs`, `settings.rs`, `actions.rs`, `shortcut/mod.rs`, `shortcut/handler.rs`, `transcription_coordinator.rs`, `lib.rs`, `Cargo.toml`
- `llama-cpp-sys-2/build.rs` at docs.rs — Metal auto-detection (unconditional framework link), Vulkan feature flag, GGML_VULKAN=ON, VULKAN_SDK env var requirement on Windows
- `utilityai/llama-cpp-rs` GitHub — Apache-2.0/MIT, 575 stars, 146+ releases, actively maintained
- `Qwen/Qwen3-4B-GGUF` on HuggingFace — Q4_K_M 2.5 GB, 100+ language support confirmed
- `Qwen/Qwen2.5-1.5B-Instruct-GGUF` on HuggingFace — Q4_K_M ~1 GB, 29+ languages confirmed
- tauri-plugin-global-shortcut docs.rs — `GlobalShortcutExt` trait, runtime register/unregister confirmed against working codebase implementation
- OpenAI Whisper paper — translate task English-output-only constraint confirmed

### Secondary (MEDIUM confidence)
- `mradermacher/translategemma-4b-it-GGUF` on HuggingFace — Q4_K_M 2.49 GB; translation quality vs. general models unverified via head-to-head benchmark
- TranslateGemma Technical Report (arxiv 2601.09012) — 55 language pairs, WMT25/WMT24++ benchmarks
- Superwhisper changelog — modes since v1.19, per-mode shortcuts since v2.12.0, conflict fix in v2.13.1
- Jan AI model hub UX documentation — fit badge, quantization tier, cancel/delete patterns
- GGUF quantization translation quality — arxiv 2508.20893, arxiv 2511.09748
- tauri-plugin-global-shortcut silent failure — issues #2540, #2646 in tauri-apps/plugins-workspace
- AMD Vulkan driver crash — llama.cpp issue #17432 (May 2026)

### Tertiary (LOW confidence)
- Vulkan quality on Windows/AMD beyond the AMD driver crash report — production readiness unclear
- mistral.rs Vulkan backend — claimed in README, no high-quality benchmarks found
- KissAPI blog April 2026 — OAuth vs API key billing for claude-code-action (single non-official source)

---
*Research completed: 2026-05-29*
*Ready for roadmap: yes*
