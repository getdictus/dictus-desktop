---
phase: 11-llm-runtime-foundation
verified: 2026-06-03T00:00:00Z
status: human_needed
score: 6/6 truths code-verified (2 of 3 platforms need human runtime confirmation)
re_verification:
  previous_status: null
  previous_score: null
human_verification:
  - test: "Windows x64 release-build runtime GPU inference smoke"
    expected: "Download → load → embedded inference runs with Vulkan GPU offload (or CPU fallback) on a packaged tauri build"
    why_human: "Requires real Windows GPU hardware + multi-GB download; CI confirms build/link green but runtime inference was not human-smoked this session (11-04-SUMMARY deviation note)"
  - test: "Linux release-build runtime GPU inference smoke"
    expected: "Download → load → embedded inference runs with Vulkan GPU offload (or CPU fallback) on a packaged tauri build"
    why_human: "Requires real Linux GPU hardware + multi-GB download; CI confirms build/link green but runtime inference was not human-smoked this session"
  - test: "Post-processing engine-list ordering + Custom Test-button placement (commit c360a62)"
    expected: "Engine list orders Apple → downloaded → downloadable → perso → Custom; selecting a model does not reorder; Custom Test-connection button renders full-width under Base URL with no clipping"
    why_human: "Visual rendering / layout; logic is code-verified but not eyeballed in-app. Tracked in .planning/todos/pending/2026-06-03-verify-postproc-engine-list-ux.md (non-blocking)"
---

# Phase 11: LLM Runtime Foundation Verification Report

**Phase Goal:** In-process GGUF LLM engine with GPU backends, model downloader, curated catalogue, custom GGUF drag/drop, and a functional model library replacing the placeholder — with a cross-platform build gate (macOS metallib, Windows x64 GPU link, ggml coexistence) cleared before Phase 12 opens.
**Verified:** 2026-06-03
**Status:** human_needed (all code-verifiable must-haves PASS; only cross-platform GPU runtime smoke on Windows/Linux + a visual UI tweak remain for human)
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (ROADMAP Success Criteria)

| #   | Truth | Status | Evidence |
| --- | ----- | ------ | -------- |
| 1   | Browse curated catalogue with on-disk size shown before download (MDL-01) | ✓ VERIFIED | `LlmManager::catalogue()` (llm.rs:116) returns 4 models with `size_mb` (1120/2490/2492/2019) and `is_downloaded:false` pre-download. Catalogue count differs from ROADMAP (see note). |
| 2   | Download with live progress, cancel, resume; SHA256-verified; HuggingFace CDN URL | ✓ VERIFIED | Range-request resume `request.header("Range", format!("bytes={}-", resume_from))` (llm.rs:496); `verify_sha256()` (llm.rs:380); all 4 URLs are `huggingface.co/.../resolve/main/...`; progress via `llm-download-progress` event wired in store (llmModelStore.ts:115). |
| 3   | Delete a downloaded model and reclaim disk space (MDL-03) | ✓ VERIFIED | `cancel_download` + delete path (`deleteLlmModel` command, llm.rs file-path deletion); `commands.deleteLlmModel` wired in store (llmModelStore.ts:94). |
| 4   | Add custom GGUF via drag/drop or file picker (MDL-04) | ✓ VERIFIED | `import_custom_gguf()` (llm.rs:740) with 4-byte GGUF magic validation; `CustomGgufDropZone.tsx` wires `onDragDropEvent` (drag) + `open({filters:[gguf]})` (picker) + inline `invalidGguf` error Alert. |
| 5   | Placeholder replaced by real library at top of local-processing page; e2e gate passes macOS/Win/Linux (MDL-05) | ⚠ PARTIAL | `<LlmLibrarySection>` rendered at top of PostProcessingSettings.tsx:116 with `<CustomGgufDropZone>` footer; `comingSoon` key removed from en/translation.json. macOS e2e confirmed; Windows/Linux runtime GPU smoke = human-pending (CI build/link green). |
| 6   | "Embedded (local)" selectable provider; non-blocking inference; Metal/Vulkan + CPU fallback; idle unload (LLM-01..04) | ✓ VERIFIED | Embedded branch `if settings.post_process_provider_id == "embedded"` (actions.rs:73) → `run_inference` (actions.rs:123). `run_inference` on `spawn_blocking` (LLM-01). GPU via `with_n_gpu_layers(u32::MAX)` + per-target Cargo features metal/vulkan. Idle watcher thread + `should_unload()` (llm.rs:1012). |

**Score:** 6/6 truths code-verified. Truth 5 carries a human runtime-smoke dependency on Windows/Linux.

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `src-tauri/src/managers/llm.rs` | LlmManager: download/verify/load/infer/unload + idle watcher | ✓ VERIFIED | 1242 lines; `LlmManager` struct, `load_model`, `run_inference`, `unload_model`, `cancel_download`, `import_custom_gguf`, `shutdown`, `should_unload`, `touch_activity`, idle-watcher thread, `LlamaBackend` init all present |
| `src-tauri/src/commands/llm.rs` | 7 Tauri commands | ✓ VERIFIED | 7 `#[tauri::command]` fns: get/download/cancel/delete/set-active/get-active/import-custom |
| `src/bindings.ts` | LLM commands + types exported | ✓ VERIFIED | `getLlmModels`..`importCustomLlmModel` (lines 654-702); `LlmModelInfo`, `LlmDownloadProgress` types; `llmDownloadProgress` event |
| `src-tauri/src/lib.rs` | LlmManager registered + commands + event + shutdown | ✓ VERIFIED | `LlmManager::new` managed (lib.rs:222); 7 commands in collect_commands (495-501); `LlmDownloadProgress` event (531); shutdown via `try_state` (85) |
| `src-tauri/src/actions.rs` | Embedded provider branch | ✓ VERIFIED | Sentinel check (actions.rs:73) before HTTP lookup, routes to `run_inference` (123) |
| `src/stores/llmModelStore.ts` | Zustand store wiring all commands + events | ✓ VERIFIED | 272 lines; all 6 commands + 6+ event listeners with unlisten cleanup |
| `src/components/.../LlmLibrarySection.tsx` | Real library, unified engine list | ✓ VERIFIED | 368 lines; `engineMode`, `ProviderEntry` peers, `ModelCard` reuse, stable-sort zones |
| `src/components/.../CustomGgufDropZone.tsx` | Drag-drop + picker + error | ✓ VERIFIED | 111 lines; onDragDropEvent + open() + invalidGguf Alert |
| `src/components/.../EmbeddedNoModelEmptyState.tsx` | (claimed in 11-03) | ℹ SUPERSEDED | File absent; cleanly removed in 11-04 unified-engine-list rework. Zero dangling references in src/. Not a gap — superseded by the unified picker design. |
| `.cargo/config.toml` | ggml coexistence link flags | ✓ VERIFIED | Per-target: Linux `--allow-multiple-definition`, Windows MSVC `/FORCE:MULTIPLE`, macOS ld64 native tolerance |
| `.github/workflows/build.yml` | Windows x64 Ninja workaround | ✓ VERIFIED | `CMAKE_GENERATOR=Ninja` set for Windows job (lines 294, 310) |
| `src-tauri/tauri.conf.json` | metallib bundling decision | ✓ VERIFIED | Valid JSON; `bundle.resources = ["resources/**/*"]`, no explicit metallib entry (embedded — confirmed working on macOS per handoff) |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | -- | --- | ------ | ------- |
| actions.rs embedded branch | LlmManager.run_inference | `app.state::<Arc<LlmManager>>()` | ✓ WIRED | actions.rs:75 → :123 |
| llmModelStore | LLM commands | `commands.*` (bindings.ts) | ✓ WIRED | All 6 invoked + progress/verify/complete/cancel/fail listeners |
| PostProcessingSettings | LlmLibrarySection | JSX `<LlmLibrarySection ... footer={<CustomGgufDropZone/>}>` | ✓ WIRED | Rendered (not just imported) at line 116 |
| CustomGgufDropZone | import_custom_llm_model | `store.importCustom(filePath)` | ✓ WIRED | Both drag + picker paths reach the command |
| lib.rs | idle-watcher shutdown | `flush_and_exit()` / `try_state` | ✓ WIRED | lib.rs:85 |
| Cargo per-target features | GPU backend | metal (macOS) / vulkan (Win/Linux) | ✓ WIRED | Cargo.toml:97/110/116 |

### Requirements Coverage

| Requirement | Source Plan | Status | Evidence |
| ----------- | ----------- | ------ | -------- |
| LLM-01 (in-process, non-blocking) | 11-01/02 | ✓ SATISFIED | `run_inference` on `spawn_blocking`; embedded branch in actions.rs |
| LLM-02 (GPU auto-select + CPU fallback) | 11-01/04 | ✓ SATISFIED (macOS); ? Win/Linux runtime | `with_n_gpu_layers(u32::MAX)` + per-target features; macOS confirmed, Win/Linux build green, runtime human-pending |
| LLM-03 (idle unload, coexists with transcription) | 11-01 | ✓ SATISFIED | Idle watcher thread, independent `should_unload`, `unload_model` drops Arc |
| LLM-04 (embedded selectable provider) | 11-02/03 | ✓ SATISFIED | Sentinel routing + UI engine-list selection |
| MDL-01 (catalogue with pre-download size) | 11-01 | ✓ SATISFIED | `catalogue()` with `size_mb` |
| MDL-02 (download/progress/cancel/resume/SHA256/HF) | 11-01 | ✓ SATISFIED | Range resume + verify_sha256 + HF URLs |
| MDL-03 (delete + reclaim) | 11-01/02 | ✓ SATISFIED | delete command + file deletion |
| MDL-04 (custom GGUF drag/pick) | 11-01/03 | ✓ SATISFIED | import_custom_gguf + CustomGgufDropZone |
| MDL-05 (placeholder → real library at top) | 11-03/04 | ✓ SATISFIED | LlmLibrarySection rendered top, comingSoon removed |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| actions.rs | 61 | "placeholder" in doc comment | ℹ Info | False positive — legitimate doc on `${output}` template substitution, not a stub |

No TODO/FIXME/unimplemented!/stub patterns in any phase-11 source file.

### Human Verification Required

1. **Windows x64 runtime GPU inference** — packaged release build download → load → embedded inference with Vulkan offload / CPU fallback. CI build+link green; runtime not human-smoked this session (per 11-04-SUMMARY deviation).
2. **Linux runtime GPU inference** — same, on Linux.
3. **Engine-list UX (commit c360a62)** — visual confirmation of zone ordering + Custom Test-button placement. Non-blocking; tracked in pending todo.

### Gaps Summary

No code gaps. Every artifact the summaries claim exists, is substantive, and is wired end-to-end:
- Backend `LlmManager` (1242 lines) with full download/verify/load/infer/unload + idle watcher.
- 7 Tauri commands + bindings.ts exports + lib.rs registration + event + shutdown.
- Embedded provider branch in actions.rs routing to `run_inference`.
- Real `LlmLibrarySection` rendered at the top of PostProcessingSettings, placeholder removed.
- `CustomGgufDropZone` with both drag-drop and file-picker import paths.
- Cross-platform build gate config present: ggml coexistence link flags (all 4 non-macOS targets + macOS native), Windows x64 Ninja workaround, macOS metallib confirmed embedded.

Two honest deltas, neither a phase failure:

1. **Catalogue content drift (non-blocking):** ROADMAP SC-1 names 3 models (Qwen3-4B, Qwen2.5-1.5B, TranslateGemma-4B). The code ships 4 (Qwen2.5-1.5B, Gemma-3-4B, Phi-4-Mini, Llama-3.2-3B). This is a deliberate, documented post-UAT catalogue refresh (11-04-SUMMARY + memory note: TranslateGemma deferred to translation mode). The *capability* (curated catalogue with pre-download sizes, HF URLs, SHA256) fully satisfies MDL-01/02; only the specific model list changed. Recommend the ROADMAP SC-1 wording be updated to match, but this does not block Phase 12.

2. **EmbeddedNoModelEmptyState removed:** Created in 11-03, removed in 11-04's unified-engine-list rework. No dangling references. Documented design evolution, not a regression.

The only items preventing an unconditional PASS are the Windows/Linux **runtime** GPU smoke (build/link is CI-green; runtime relies on handoff) and a visual UI eyeball — both already acknowledged in the summaries and explicitly accepted by the user as non-blocking for opening Phase 12.

---

_Verified: 2026-06-03_
_Verifier: Claude (gsd-verifier)_
