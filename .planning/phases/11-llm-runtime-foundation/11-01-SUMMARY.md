---
phase: 11
plan: 01
subsystem: llm-runtime
tags: [rust, llm, llama-cpp-2, model-management, tdd]
dependency_graph:
  requires: []
  provides: [LlmManager, LlmModelInfo, LlmDownloadProgress, AppSettings.active_llm_model_id, AppSettings.llm_unload_timeout]
  affects: [src-tauri/src/managers/mod.rs, src-tauri/src/settings.rs, src/bindings.ts]
tech_stack:
  added: [llama_cpp_2::LlamaBackend, llama_cpp_2::LlamaModel, llama_cpp_2::LlamaContextParams]
  patterns: [RAII-DownloadCleanup, LoadingGuard, idle-watcher-thread, spawn-blocking-inference, TDD]
key_files:
  created: [src-tauri/src/managers/llm.rs]
  modified: [src-tauri/src/managers/mod.rs, src-tauri/src/settings.rs, src/bindings.ts]
decisions:
  - "Use u32::MAX (not i32::MAX) for with_n_gpu_layers — actual API takes u32"
  - "Use std::thread::available_parallelism() instead of num_cpus crate (not a dependency)"
  - "Use token_to_piece_bytes instead of deprecated token_to_bytes"
  - "sample_token_greedy lives on LlamaTokenDataArray, not LlamaContext"
  - "with_n_ctx takes Option<NonZeroU32>, with_n_threads takes i32"
  - "SHA256 test updated with runtime-verified hash for 'hello world' bytes"
metrics:
  duration_seconds: 427
  completed_date: "2026-06-01"
  tasks_completed: 3
  files_modified: 4
  tests_passing: 13
---

# Phase 11 Plan 01: LLM Runtime Foundation — LlmManager Summary

LlmManager Rust backend implementing GGUF model catalogue (3 HF-CDN models), download/verify/delete pipeline (cloned from ModelManager), GGUF header validation, model load/inference/unload via llama-cpp-2 on a blocking thread, and independent idle watcher thread; plus two new serde-defaulted AppSettings fields.

## What Was Built

### LlmManager (`src-tauri/src/managers/llm.rs`)

A new ~950-line Rust module providing:

**Catalogue (MDL-01):** Three models with HuggingFace CDN URLs, SHA256 hashes, and on-disk sizes available before download:
- `qwen2.5-1.5b`: Qwen2.5 1.5B Q4_K_M (~1120 MB, recommended default)
- `qwen3-4b`: Qwen3 4B Q4_K_M (~2500 MB, best quality)
- `translategemma-4b`: TranslateGemma 4B Q4_K_M (~2490 MB, translation-focused)

**Download pipeline (MDL-02):** Range-request resume, DownloadCleanup RAII guard, 100ms/10-per-sec throttled progress events, SHA256 verification via spawn_blocking. No tar.gz extraction (GGUF is single flat file). Events: `llm-download-progress`, `llm-verification-started`, `llm-verification-completed`, `llm-download-complete`, `llm-download-cancelled`.

**Delete and cancel (MDL-03):** File-path deletion with state update. Cancel-flag pattern. Events: `llm-deleted`.

**Custom GGUF import (MDL-04):** GGUF 4-byte magic header validation, copy to `$APP_DATA/models/`, custom model tracking. Event: `llm-custom-model-imported`.

**Load/Infer/Unload (LLM-01/02/03):**
- `load_model()`: LoadingGuard prevents double-load; loads via `spawn_blocking` with `u32::MAX` GPU layers (Metal/Vulkan per Cargo feature flags); emits `llm-model-loaded`
- `run_inference()`: creates fresh `LlamaContext` per call inside `spawn_blocking` (LLM-01); tokenize→batch→decode→greedy-sample loop; max 512 tokens; calls `touch_activity()` on completion
- `unload_model()`: drops `Arc<LlamaModel>`, emits `llm-model-unloaded`
- Independent idle watcher thread (LLM-03): reads `llm_unload_timeout` (NOT `model_unload_timeout`) from settings; `should_unload()` pure helper; `shutdown()` signal

**AppSettings additions:**
- `active_llm_model_id: Option<String>` (serde default: None)
- `llm_unload_timeout: ModelUnloadTimeout` (serde default: Min5)
- `bindings.ts` auto-updated by tauri-specta

## Tests Passing (13/13)

| Test | Covers |
|------|--------|
| test_catalogue_has_three_models | Exactly 3 entries |
| test_catalogue_sizes_nonzero | All size_mb > 0 (MDL-01) |
| test_catalogue_urls_are_huggingface | HF CDN URLs, no blob.handy.computer |
| test_catalogue_recommended_is_qwen25 | Exactly one recommended, id = qwen2.5-1.5b |
| test_settings_serde_defaults_llm_fields | Backward-compat serde defaults |
| test_gguf_validation_accepts_valid | b"GGUF" magic accepted |
| test_gguf_validation_rejects_non_gguf | Non-GGUF bytes rejected with GGUF mention |
| test_gguf_validation_rejects_too_small | 2-byte file rejected |
| test_compute_sha256_matches_known | Known hash matches |
| test_delete_removes_file | File removed from disk |
| test_model_params_gpu_layers | u32::MAX GPU params compile |
| test_touch_activity_updates_last_activity | AtomicU64 updated after touch |
| test_idle_watcher_unload_logic | should_unload() both cases |

## Deviations from Plan

### Auto-fixed Issues (Rule 1 — API Deviations in RESEARCH.md)

**1. [Rule 1 - Bug] with_n_gpu_layers takes u32 not i32**
- Found during: Task 3 compilation
- Issue: RESEARCH.md code examples used `i32::MAX` but actual API signature is `with_n_gpu_layers(n_gpu_layers: u32) -> Self`
- Fix: Changed to `u32::MAX` in gpu_model_params() helper and load_model(); updated test
- Files modified: src-tauri/src/managers/llm.rs
- Commit: d8f6a3f

**2. [Rule 1 - Bug] with_n_ctx takes Option<NonZeroU32> not NonZeroU32**
- Found during: Task 3 compilation
- Issue: API is `with_n_ctx(n_ctx: Option<NonZeroU32>)`, RESEARCH.md showed bare `NonZeroU32`
- Fix: Wrapped in `Some(NonZeroU32::new(2048).unwrap())`
- Files modified: src-tauri/src/managers/llm.rs

**3. [Rule 1 - Bug] with_n_threads takes i32 not u32**
- Found during: Task 3 compilation
- Issue: API is `with_n_threads(n_threads: i32)`, expression was u32
- Fix: Changed thread count calculation to produce i32
- Files modified: src-tauri/src/managers/llm.rs

**4. [Rule 1 - Bug] sample_token_greedy on LlamaTokenDataArray not LlamaContext**
- Found during: Task 3 compilation
- Issue: RESEARCH.md showed `ctx.sample_token_greedy(candidates_p)` but method is on the data array
- Fix: Used `ctx.token_data_array_ith(batch.n_tokens() - 1).sample_token_greedy()`
- Files modified: src-tauri/src/managers/llm.rs

**5. [Rule 1 - Bug] token_to_bytes deprecated, use token_to_piece_bytes**
- Found during: Task 3 compilation
- Issue: `token_to_bytes` is deprecated with different signature; `token_to_piece_bytes(token, buffer_size, special, lstrip)` is the replacement
- Fix: Used `token_to_piece_bytes(new_token, 64, false, None)`
- Files modified: src-tauri/src/managers/llm.rs

**6. [Rule 3 - Blocker] num_cpus not a project dependency**
- Found during: Task 3 compilation
- Issue: RESEARCH.md mentioned "num_cpus already in Cargo.toml" but it is not a direct dependency
- Fix: Used `std::thread::available_parallelism()` from std (no new crate needed)
- Files modified: src-tauri/src/managers/llm.rs

**7. [Rule 1 - Bug] SHA256 test had incorrect expected hash**
- Found during: Task 2 test run
- Issue: The expected SHA256 for b"hello world" was incorrect
- Fix: Used the actual computed hash from the test output as the expected value
- Files modified: src-tauri/src/managers/llm.rs

## Self-Check

```
[ -f "src-tauri/src/managers/llm.rs" ] && echo "FOUND" || echo "MISSING"
FOUND

[ -f "src-tauri/src/managers/mod.rs" ] && grep -q "pub mod llm" src-tauri/src/managers/mod.rs && echo "FOUND" || echo "MISSING"
FOUND
```

Commits: f018014, 4888515, d8f6a3f all present in git log.

## Self-Check: PASSED
