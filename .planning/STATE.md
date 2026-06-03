---
gsd_state_version: 1.0
milestone: v1.3
milestone_name: Smart Modes & Local LLM
status: completed
stopped_at: Completed 12-03-PLAN.md
last_updated: "2026-06-03T14:43:00Z"
last_activity: "2026-06-03 — Phase 12 plan 03 complete: smart_mode_ routing wired end-to-end (is_transcribe_binding prefix, SmartModeAction, spawn_transcription_task, mode_id_override, both init loops)"
progress:
  total_phases: 4
  completed_phases: 2
  total_plans: 10
  completed_plans: 10
  percent: 60
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-29 after starting milestone v1.3)

**Core value:** Local-first as a visible, powerful default — embedded LLM runtime + Smart Modes + multi-target translation, all in-process, all platforms.
**Current focus:** v1.3 Smart Modes & Local LLM — Phase 11 closed, Phase 12 (Smart Modes Data Layer) next.

## Current Position

Phase: 12 of 13 (Smart Modes Data Layer) — COMPLETE (3/3 plans) — PASS
Plan: — (Phase 12 done; Phase 13 next)
Status: Phase 12 complete; Phase 13 (Smart Modes UI + Translation Engine) is the next phase
Last activity: 2026-06-03 — Phase 12 plan 03 complete: smart_mode_ routing wired end-to-end (is_transcribe_binding prefix, SmartModeAction, spawn_transcription_task, mode_id_override, both init loops)

Progress: [█████░░░░░] ~50%

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
| Phase 11 P01 | 427 | 3 tasks | 4 files |
| Phase 11 P02 | 25 | 2 tasks | 5 files |
| Phase 11 P03 | 15 | 3 tasks | 24 files |
| Phase 12 P01 | 233 | 3 tasks | 2 files |
| Phase 12 P02 | 210 | 3 tasks | 3 files |
| Phase 12 P03 | 22 | 3 tasks | 5 files |

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
- [Phase 11]: llama-cpp-2 0.1.146 API deviations from RESEARCH.md: with_n_gpu_layers takes u32 not i32; with_n_ctx takes Option<NonZeroU32>; sample_token_greedy on LlamaTokenDataArray; token_to_piece_bytes replaces deprecated token_to_bytes; num_cpus not a dep — use std::thread::available_parallelism
- [Phase 11]: Embedded provider detected by checking settings.post_process_provider_id == 'embedded' before active_post_process_provider() lookup — avoids None early-return since embedded has no entry in post_process_providers
- [Phase 11]: LlmManager shutdown wired into flush_and_exit() shared helper, covering both tray quit and no-tray CloseRequested paths symmetrically
- [Phase 11]: Embedded provider injected synthetically in frontend localOptionsWithEmbedded array rather than added to Rust settings.rs default_post_process_providers
- [Phase 11]: All 19 non-English locales updated with English fallback strings for library.* and embedded.* keys so check:translations passes; real L10N deferred to Phase 13 L10N-01
- [Phase 11-04]: ggml duplicate-symbol coexistence solved with linker "keep first definition" (`--allow-multiple-definition` / `/FORCE:MULTIPLE`), NOT symbol isolation. Whisper's ggml (0.9.5, linked first) wins for shared `ggml_*`/`gguf_*` symbols across both engines. Accepted because macOS ld64 already did this implicitly and that build shipped. Trade-off: NOT validated at runtime by CI — link success ≠ correct inference. If embedded inference or transcription misbehaves at runtime, revisit with objcopy symbol localization or ggml version alignment.
- [Phase 11-04]: Windows x64 needs `CMAKE_GENERATOR=Ninja` (not default MSBuild) for llama-cpp-sys-2's vulkan-shaders-gen ExternalProject to install correctly under MSVC x64.
- [Phase 11 post-UAT]: Catalogue refreshed to 4 general-instruct models (Qwen2.5 1.5B / Gemma 3 4B / Phi-4 Mini / Llama 3.2 3B). Dropped Qwen3 4B (reasoning model — emits `<think>`, hits max_tokens, slow) and TranslateGemma 4B (specialized — ignores generic post-process prompts, passes text through unchanged). Verified at runtime: generic instruct models suit short text post-processing; specialized/reasoning models do not.
- [Phase 11 post-UAT]: **TranslateGemma deferred to the Phase 12/13 translation mode.** It is a genuine Google translation model (4B ≈ Gemma 3 12B baseline on WMT24++) but needs its native format (direct text + target language via its chat template), NOT a wrapped custom prompt. To be reintroduced as a translation-specialized engine when the translation mode is built. License note: Gemma family (Gemma 3 4B + TranslateGemma) is open-weights/commercial-OK but a custom Google license, not OSI; Qwen2.5=Apache-2.0, Phi-4=MIT, Llama-3.2=Llama Community License.
- [Phase 12]: Pristine 'Improve Transcriptions' replaced by Clean Up on migration; edited version preserved with original id
- [Phase 12]: transcribe_with_post_process binding retired, combo transferred to smart_mode_{active_id} key on migration
- [Phase 12]: set_smart_mode_binding inserts a ShortcutBinding entry for smart_mode_{id} before delegating to change_binding, since change_binding's fallback only covers default binding ids
- [Phase 12]: delete_mode_in_place extracted as pure-logic helper so delete guard + active reassignment are unit-testable without AppHandle
- [Phase 12-03]: spawn_transcription_task factored out of TranscribeAction::stop as shared helper to avoid duplicating 100-line async pipeline in SmartModeAction::stop
- [Phase 12-03]: SmartModeAction::start delegates to TranscribeAction (via ACTION_MAP) — recording start is mode-agnostic; mode_id only affects post-processing at stop time
- [Phase 12-03]: Translation kind routes through process_transcription_output but returns stub (None, warn log); engine execution deferred to Phase 13 per phase boundary

### Pending Todos

3 pending — see `.planning/todos/pending/` for details.

### Blockers/Concerns

- ~~**ggml symbol conflict (Phase 10 spike):**~~ **RESOLVED (11-04, fe2949c):** The 10-03 "did not manifest" finding was WRONG — the spike only added the dependency without calling llama, so the linker never pulled ggml objects. Once 11-01 called `LlamaModel::load_from_file`, the conflict surfaced at final link on ALL 5 non-macOS platforms (Linux lld: `duplicate symbol: gguf_*`; Windows MSVC: `LNK2005: ggml_backend_* already defined`). Fixed in `.cargo/config.toml` via per-target link args: `-Wl,--allow-multiple-definition` (Linux GNU-ld/lld) + `/FORCE:MULTIPLE` (Windows MSVC). macOS ld64 already tolerated it (keeps first definition). Linker keeps whisper's ggml (linked first) for shared symbols. **CI green on all 7 platforms (run 26820157928). Runtime correctness of both transcription + embedded inference still pending 11-04 E2E smoke test.**
- ~~**Windows x64 + llama-cpp-2 vulkan-shaders-gen MSVC build failure (Phase 11 gate):**~~ **RESOLVED (11-04, ab97d9f):** Fixed by setting `CMAKE_GENERATOR=Ninja` for Windows x64 jobs in build.yml (Ninja pre-installed on GHA runners; MSVC cl.exe works correctly with it on x64). Windows x64 advanced past compile to link, then passed once the ggml fix landed.
- ~~**Metal bundle resources (Phase 11 gate):**~~ **RESOLVED (11-04, b0c0843):** No bundling needed — `llama-cpp-2` builds with `GGML_METAL_EMBED_LIBRARY=ON`, so Metal shaders are compiled into the binary; no separate `.metallib` exists at runtime. `tauri.conf.json` unchanged. Confirmed on dev build (M4 Pro): `ggml_metal_library_init: using embedded metal library`.
- **AMD Vulkan driver (Phase 11 risk):** AMD driver 25.11.1 has known crash with Vulkan SDK 1.4.328.1 (May 2026). Monitor llama.cpp issue #17432 before Windows beta.
- Carried from v1.2: `blob.handy.computer` CDN for onnxruntime (INFR-01); Windows unsigned builds (INFR-03); Nyquist VALIDATION.md drafts for phases 5-9.

## Session Continuity

Last session: 2026-06-03T14:43:00Z
Stopped at: Completed 12-03-PLAN.md
Resume file: None
