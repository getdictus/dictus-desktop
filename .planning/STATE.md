---
gsd_state_version: 1.0
milestone: v1.3
milestone_name: Smart Modes & Local LLM
status: planning
stopped_at: Completed 13-14-PLAN.md (engine-name gate for non-embedded providers)
last_updated: "2026-06-05T20:27:28.186Z"
progress:
  total_phases: 4
  completed_phases: 3
  total_plans: 25
  completed_plans: 21
  percent: 95
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-29 after starting milestone v1.3)

**Core value:** Local-first as a visible, powerful default — embedded LLM runtime + Smart Modes + multi-target translation, all in-process, all platforms.
**Current focus:** v1.3 Smart Modes & Local LLM — Phase 11 closed, Phase 12 (Smart Modes Data Layer) next.

## Current Position

Phase: 13 of 13 (Smart Modes UI + Translation Engine + Presets i18n) — IN PROGRESS (gap-closure round 2 needed)
Plan: 13-01..13-11 have SUMMARYs. Gap-closure run 2026-06-05 executed 13-09 (backend binding/identity dedup + suspend/resume), 13-10 (Apple Intelligence task-agnostic path — live-verified), 13-11 (picker dedup + ${output} hint). 13-12 (frontend conflict-capture + engine modal) committed tasks 1-2 (262576b, 89e17d6) but its Task-3 live verify FAILED. 13-08 (re-verify) Task-1 gates green (64deeeb) but Task-2 live re-test found 3 remaining gaps. No 13-08/13-12 SUMMARY (incomplete). Phase NOT marked complete.
Status: Live re-test resolved 4 of 6 prior gaps + truncation. 3 gaps remain → see .planning/phases/13-.../13-UAT.md ## Gaps (status: failed).

### Remaining gaps (→ /gsd:plan-phase 13 --gaps; user chose formal cycle 2026-06-05)
- **[B5] test 7** — duplicate shortcut not blocked/warned. Capture/suspend works (13-12); BACKEND `set_smart_mode_binding`/`change_binding` must reject a combo already bound to a different mode (return success:false so the chip shows the existing conflict UI). Logs: `resume_all_shortcuts: Hotkey already registered: Cmd+2`.
- **[C7] test 11** — engine modal shows Gemma when Apple Foundation active. FRONTEND `SmartModesSection.tsx:111-114` precedence `activeModelName ?? providerLabel` prefers lingering GGUF id; gate on `post_process_provider_id` (model name only when embedded provider active, else provider label; activeIsRecommended false for non-GGUF).
- **[D8] test 3** — seeded mode names show English in non-en app. i18n DATA only: translate ~10 `smartModes.defaultModes.*` NAME keys across all 20 locales (names only — user decision; descriptions stay English). Architecture already correct (SmartModeCard.tsx:73-75).

### Next session (post /clear) — separate NEW WORK (not gap closure)
- **Improve/adapt the default Smart Mode prompts.** User finds output quality on the default prompts not quite matching intent; refine wording. Fresh task.

Progress: [█████████░] ~95% (Phase 13: 3 gaps remain before closure)

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
| Phase 13-smart-modes-ui-translation-presets-i18n P02 | 5 | 2 tasks | 20 files |
| Phase 13 P01 | 126 | 3 tasks | 7 files |
| Phase 13-smart-modes-ui-translation-presets-i18n P03 | 15 | 3 tasks | 4 files |
| Phase 13 P05 (gap-closure backend) | 4 | 3 tasks | 5 files |
| Phase 13 P06 | 15 | 2 tasks | 22 files |
| Phase 13 P07 | 45 | 4 tasks | 24 files |
| Phase 13-smart-modes-ui-translation-presets-i18n P09 | 5 | 3 tasks | 4 files |
| Phase 13-smart-modes-ui-translation-presets-i18n P11 | 2 | 2 tasks | 21 files |
| Phase 13 P10 | 25 | 3 tasks | 2 files |
| Phase 13-smart-modes-ui-translation-presets-i18n P14 | 2 | 1 tasks | 1 files |

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
- [Phase 13]: Phase 13 plan 02: English fallback values used verbatim in all 19 non-English locales for new smartModes.* keys — real translations deferred (Phase 11 precedent)
- [Phase 13]: TranslationEngineChoice uses snake_case serde (not_chosen/translate_gemma/generic_model) matching SmartModeKind pattern
- [Phase 13]: [Phase 13-01]: TranslateGemma SHA256 computed from real file download (7f7357c14abd9da4eb200b38b05da502cd6e10d7e1d403fbc9f78c19f3209b72); run_translation dispatches per TranslationEngineChoice — NotChosen=warn+None, TranslateGemma=native template format, GenericModel=explicit prompt with target label
- [Phase 13]: SmartModeShortcutChip routes through setSmartModeBinding exclusively (not updateBinding); suspend/resume is best-effort; dir=ltr on chip for RTL safety
- [Phase 13]: SmartModesSection renders custom h2 elements instead of SettingsGroup to support dynamic amber color on Translation header
- [Phase 13-05]: clear_smart_mode_binding deletes binding entry entirely (not empty-set) so init_shortcuts never re-registers it
- [Phase 13-05]: transcribe_with_post_process retired only as a persisted global shortcut; action string kept in coordinator/actions/signal/CLI for --toggle-post-process CLI flag
- [Phase 13-05]: smart_mode_templates() is the 10-mode catalogue source (pub); default_smart_modes() seeds only Clean Up for first-run and migration
- [Phase 13]: Commit shortcut on first non-modifier keydown (not keyup drain) — fixes macOS Cmd+key swallow where OS never fires keyup for non-modifier while Cmd held
- [Phase 13]: [Phase 13-06]: English fallback values used verbatim in all 19 non-English locales for new shortcut.* keys (clear/clearAriaLabel/recording) — real translations deferred
- [Phase 13-07]: SEEDED_MODE_ID_TO_I18N_KEY exported from SmartModeCard so SmartModeTemplatePicker can localize template names without duplicating the map
- [Phase 13-07]: SmartModeTemplatePicker uses addSmartMode (new id) not seeded id — prevents id collision when user deletes a default and recreates it via picker
- [Phase 13-07]: smartModeTemplates() added manually to bindings.ts returning SmartMode[] (no Result) — infallible command, follows getAvailableTypingTools() pattern
- [Phase 13 post-UAT]: **TranslateGemma DROPPED.** Controlled benchmark (`src-tauri/examples/translation_ab.rs`, 14 texts x 4 models) — a same-size generic (Gemma-3-4B) matches/beats the dedicated translate model on quality AND is far more reliable (TranslateGemma had runaway generation, hallucinated continuations, self-emitted `<<<note>>>` markers, ~6/14 polluted outputs; Gemma-3-4B 0/14). Removed from catalogue; `TranslationEngineChoice::TranslateGemma` enum variant kept for settings back-compat, treated as GenericModel. Translation routes through the active LLM model.
- [Phase 13 post-UAT]: Output-language fix — small LLMs answer in the language of the (English) instructions and ignore "preserve the language". Fix: detect input language (whatlang) and inject an explicit "write your entire response in {lang}" directive, placed FIRST (appending after the base lands it next to the trailing `Text:` label → model echoes it into the output). Directive defers to explicit-translation prompts.
- [Phase 13 post-UAT]: Settings store must be refreshed after shortcut bind/clear/edit — SmartModeCard reads bindings from the Zustand settings store, but the chip mutates via direct commands; refetchModes now also calls refreshSettings.
- [Phase 13 post-UAT]: Modifier-only shortcuts (e.g. Option alone) commit on modifier RELEASE when no main key was pressed; combos still commit on main keydown. Guarded against double-commit.
- [Phase 13 post-UAT]: Translation modal infinite loop fixed — `useLlmModelStore()` whole-store subscription + store-dependent effect caused refresh->re-render->refresh (froze UI / webview black-screen). Use narrow selectors + effect depending only on `open`. Root ErrorBoundary added as a backstop.
- [Phase 13]: 13-10: Apple Intelligence free-text path — CleanedTranscript @Generable struct removed from apple_intelligence.swift; plain session.respond(to:) now used for all Smart Mode prompts so translate/bullets/etc. work correctly via the system-prompt instructions field rather than being biased by the transcript-cleanup schema. token_limit hardcoded to 0 in actions.rs Apple branch (model label is not a word cap).
- [Phase 13-09]: [13-09] Reversal of [13-07]: add_smart_mode reuses stable seed id for template re-adds. Overwrite+reuse branches cover both collision cases safely.
- [Phase 13-09]: [13-09] reconcile_dangling_smart_mode_bindings runs at load (after migration, before init_shortcuts) to drop orphaned smart_mode_* bindings including those from 13-04 seed reduction.
- [Phase 13-09]: [13-09] resume_all_shortcuts skips cancel binding (dynamically managed during recording) to avoid permanently registering a dynamically-managed shortcut.
- [Phase 13]: [13-11] Picker fetches listSmartModes() on open for name-based dedup; currentNames Set memoized with useMemo; outputHint gated to kind=rewrite; 5 new i18n keys in en + 19 non-English locales with English fallback
- [Phase 13]: [13-12] TranslationEngineChoiceModal active badge driven by engineChoice + activeIsRecommended props (not GGUF activeModelId) — covers Apple Intelligence and non-GGUF providers
- [Phase 13]: [13-12] SmartModeShortcutChip: module-level resumeAll() helper + suspendAllShortcuts on record start; resumeAll on commitCombo (success+conflict), handleClickOutside, and effect cleanup — every exit path covered
- [Phase 13]: 13-10 VERIFIED LIVE (macOS Apple Silicon): Apple Intelligence Smart Modes now correct — Spanish translation produced (not unchanged English), Bullet Points full multi-line (no 26-char truncation), Clean Up still works. Closes shared root cause behind UAT tests 10 + 11.
- [Phase 13-smart-modes-ui-translation-presets-i18n]: [13-14] activeModelName gated on isEmbeddedActive (providerId === 'embedded') — prevents lingering GGUF activeModelId from short-circuiting to Gemma when Apple Foundation is active; activeIsRecommended = isEmbeddedActive && activeModelId === 'gemma-3-4b'

### Pending Todos

3 pending — see `.planning/todos/pending/` for details.

### Blockers/Concerns

- ~~**ggml symbol conflict (Phase 10 spike):**~~ **RESOLVED (11-04, fe2949c):** The 10-03 "did not manifest" finding was WRONG — the spike only added the dependency without calling llama, so the linker never pulled ggml objects. Once 11-01 called `LlamaModel::load_from_file`, the conflict surfaced at final link on ALL 5 non-macOS platforms (Linux lld: `duplicate symbol: gguf_*`; Windows MSVC: `LNK2005: ggml_backend_* already defined`). Fixed in `.cargo/config.toml` via per-target link args: `-Wl,--allow-multiple-definition` (Linux GNU-ld/lld) + `/FORCE:MULTIPLE` (Windows MSVC). macOS ld64 already tolerated it (keeps first definition). Linker keeps whisper's ggml (linked first) for shared symbols. **CI green on all 7 platforms (run 26820157928). Runtime correctness of both transcription + embedded inference still pending 11-04 E2E smoke test.**
- ~~**Windows x64 + llama-cpp-2 vulkan-shaders-gen MSVC build failure (Phase 11 gate):**~~ **RESOLVED (11-04, ab97d9f):** Fixed by setting `CMAKE_GENERATOR=Ninja` for Windows x64 jobs in build.yml (Ninja pre-installed on GHA runners; MSVC cl.exe works correctly with it on x64). Windows x64 advanced past compile to link, then passed once the ggml fix landed.
- ~~**Metal bundle resources (Phase 11 gate):**~~ **RESOLVED (11-04, b0c0843):** No bundling needed — `llama-cpp-2` builds with `GGML_METAL_EMBED_LIBRARY=ON`, so Metal shaders are compiled into the binary; no separate `.metallib` exists at runtime. `tauri.conf.json` unchanged. Confirmed on dev build (M4 Pro): `ggml_metal_library_init: using embedded metal library`.
- **AMD Vulkan driver (Phase 11 risk):** AMD driver 25.11.1 has known crash with Vulkan SDK 1.4.328.1 (May 2026). Monitor llama.cpp issue #17432 before Windows beta.
- Carried from v1.2: `blob.handy.computer` CDN for onnxruntime (INFR-01); Windows unsigned builds (INFR-03); Nyquist VALIDATION.md drafts for phases 5-9.

## Session Continuity

Last session: 2026-06-05T20:27:28.184Z
Stopped at: Completed 13-14-PLAN.md (engine-name gate for non-embedded providers)
Resume file: None
