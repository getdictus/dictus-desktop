# Phase 11: LLM Runtime Foundation - Research

**Researched:** 2026-06-01
**Domain:** Rust llama-cpp-2 crate, GGUF model management, Tauri manager pattern, background-thread inference
**Confidence:** HIGH (most areas verified against live code + official sources)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Catalogue & quantization:**
- One GGUF per catalogue model (no quant variants). Three models: Qwen2.5-1.5B Q4_K_M, Qwen3-4B Q4_K_M, TranslateGemma-4B Q4_K_M. All balanced tier.
- Recommended default model = Qwen2.5-1.5B (`is_recommended` flag). Qwen3-4B is the quality pick.

**Embedded provider behavior:**
- "Embedded (local)" is always selectable (even before a model is downloaded).
- No-model state: inline empty state in provider area → scroll-link to library card with recommended model highlighted. Never a blocking modal, never silent auto-download.
- Active model selection: user marks one downloaded LLM as active in the library; embedded provider always runs that GGUF.
- No silent default switch: v1.2 platform-aware default logic unchanged. Embedded NOT auto-promoted on upgrade.

**Library UI treatment:**
- Reuse `ModelsSettings.tsx` card style verbatim (download/cancel/delete/progress/active-indicator).
- Each entry shows: on-disk size (before download), one-line description, active/downloaded/downloadable state, live progress with cancel.
- One list; custom models tagged with "Custom" badge (mirrors `is_custom` in transcription models).

**Custom GGUF import:**
- Both methods: drag-and-drop + "Add custom model" file-picker button (`tauri-plugin-dialog` already a dep).
- Copy file into `$APP_DATA/models/` (never reference in place).
- Validate GGUF 4-byte magic header at import time; reject inline with clear message if invalid.

### Claude's Discretion

- Exact HF repo URLs, filenames, byte sizes, and SHA256 hashes for the 3 catalogue GGUFs (research-confirmed below).
- GPU backend detection mechanism and whether to adopt upstream `966ff99` async GPU query.
- Idle-unload timeout value/surfacing for the LLM (independent of transcription unload; reuse watcher pattern).
- Inference parameters (context length, max tokens, temperature) for post-processing.
- Background-thread/concurrency design for inference.
- Download error/retry behavior, resume mechanics, drop-zone exact placement.
- `.metallib` bundle-resource wiring for macOS (verify via `tauri build` release smoke test).
- Windows-x64 `vulkan-shaders-gen` MSVC build fix (Phase 11 gate).

### Deferred Ideas (OUT OF SCOPE)

- Quant-tier labels / multiple quants per model (MDL-F3)
- Hardware fit badge (MDL-F2), GPU/CPU status badge (MDL-F4), RAM-based auto-quant recommendation (MDL-F1)
- Larger catalogue / HF browse-search (MDL-F5/F6)
- Embedded becomes auto-default provider
- Per-mode provider selection (MODE-F3)
- CUDA backend (LLM-F1) / n_gpu_layers slider (LLM-F2)
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| LLM-01 | GGUF model runs in-process via embedded engine (no external Ollama), on a background thread so inference never blocks the overlay, tray, or shortcuts | llama-cpp-2 0.1.146 confirmed in Cargo.toml; `tokio::task::spawn_blocking` pattern (already used in SHA256 verification in model.rs:1200); idle-watcher thread pattern in transcription.rs:92-158 |
| LLM-02 | GPU acceleration auto-selected per platform (Metal on macOS, Vulkan on Windows/Linux) with graceful CPU fallback | Per-target Cargo feature flags already in place in Cargo.toml (lines 97-116); `LlamaModelParams::with_n_gpu_layers(i32::MAX)` enables GPU; CPU fallback via `n_gpu_layers(0)` |
| LLM-03 | Loaded LLM unloads after own idle timeout; coexists safely with transcription model (sequential pipeline, independent unload) | Idle-watcher pattern in transcription.rs:92-158 is the direct template; transcription manager uses `Arc<AtomicU64>` for last_activity; independent `shutdown_signal` per manager |
| LLM-04 | "Embedded (local)" selectable as post-processing provider alongside Apple Intelligence, Custom, cloud | `LOCAL_PROVIDER_IDS_SET` at PostProcessingSettings.tsx:28 must add `"embedded"`; `PostProcessProvider` struct supports this without schema changes; actions.rs post-processing path needs embedded branch |
| MDL-01 | Curated catalogue with on-disk size shown BEFORE download | `ModelInfo.size_mb` field already used in transcription model cards; LLM catalogue entries need this populated (sizes confirmed in research) |
| MDL-02 | Download with progress/cancel/resume; SHA256-verified; HuggingFace CDN (never blob.handy.computer) | `ModelManager.download_model()` is 100% reusable — range-request resume, `DownloadCleanup` RAII guard, SHA256 via spawn_blocking, 10/sec event throttle. Only the URL and expected hash differ. |
| MDL-03 | Delete downloaded model and reclaim disk space | `ModelManager.delete_model()` already handles file-based models (non-directory path); GGUF files are single-file — same code path applies |
| MDL-04 | Custom GGUF import via drag/drop or file picker; GGUF header validated | `tauri-plugin-dialog` already a dependency; GGUF magic: `b"GGUF"` (4 bytes, `0x47 0x47 0x55 0x46`); copy to `$APP_DATA/models/`; discovery pattern in `discover_custom_whisper_models` is the template |
| MDL-05 | Placeholder card replaced by real, functional model library at top of local-processing page | PostProcessingSettings.tsx:638-663 is the placeholder render site; `SettingsGroup` + `ModelCard` composition confirmed in UI-SPEC |
</phase_requirements>

---

## Summary

Phase 11 has three distinct work streams: (1) a new `LlmManager` (Rust backend), (2) a `LlmLibrarySection` (React frontend), and (3) wiring "embedded" as a new post-processing provider into the existing pipeline.

The key insight from codebase inspection is that roughly 80% of the required machinery already exists in `managers/model.rs` and `managers/transcription.rs`. The download pipeline (`download_model`, `DownloadCleanup`, SHA256 via `spawn_blocking`, resume via Range header) can be cloned near-verbatim for LLM GGUFs — the only difference is that LLMs are single-file downloads (no tar.gz extraction). The idle-watcher thread pattern in `transcription.rs:92-158` is the exact template for the LLM's independent unload. The `ModelCard.tsx` component is reused verbatim.

The two highest-risk items are: the **Windows-x64 `vulkan-shaders-gen` MSVC build failure** (a known llama.cpp CMake issue, deferred from Phase 10) and the **macOS `.metallib` bundle-resource paths** (unconfirmed; must be discovered via `tauri build` release smoke test, not `tauri dev`). The ggml symbol conflict resolved cleanly in Phase 10 and is not expected to resurface.

**Primary recommendation:** Structure Phase 11 into four waves: (Wave 0) test scaffolding + catalogue manifest; (Wave 1) `LlmManager` Rust backend with download/verify/delete/load/infer/unload; (Wave 2) frontend `LlmLibrarySection` + provider wiring; (Wave 3) platform smoke tests + `.metallib` resolution + Windows MSVC fix.

---

## Confirmed Catalogue Metadata

These values are research-confirmed from official HuggingFace sources. The planner MUST use these exact values.

| Model | Role / Description | HF Repo | Filename | HF CDN URL | SHA256 | Size |
|---|---|---|---|---|---|---|
| Qwen2.5-1.5B | Fast, lightweight — recommended default | `Qwen/Qwen2.5-1.5B-Instruct-GGUF` | `qwen2.5-1.5b-instruct-q4_k_m.gguf` | `https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct-GGUF/resolve/main/qwen2.5-1.5b-instruct-q4_k_m.gguf` | `6a1a2eb6d15622bf3c96857206351ba97e1af16c30d7a74ee38970e434e9407e` | ~1120 MB |
| Qwen3-4B | Best quality for post-processing | `Qwen/Qwen3-4B-GGUF` | `Qwen3-4B-Q4_K_M.gguf` | `https://huggingface.co/Qwen/Qwen3-4B-GGUF/resolve/main/Qwen3-4B-Q4_K_M.gguf` | `7485fe6f11af29433bc51cab58009521f205840f5b4ae3a32fa7f92e8534fdf5` | ~2500 MB |
| TranslateGemma-4B | Translation-focused | `bullerwins/translategemma-4b-it-GGUF` | `translategemma-4b-it-Q4_K_M.gguf` | `https://huggingface.co/bullerwins/translategemma-4b-it-GGUF/resolve/main/translategemma-4b-it-Q4_K_M.gguf` | `7f7357c14abd9da4eb200b38b05da502cd6e10d7e1d403fbc9f78c19f3209b72` | ~2490 MB |

**Confidence on SHA256:** MEDIUM — fetched from live HuggingFace file pages on 2026-06-01. The official `Qwen/Qwen2.5-1.5B-Instruct-GGUF` repo SHA256 is from the official Qwen org. The Qwen3-4B SHA256 is from the official Qwen/Qwen3-4B-GGUF repo. The TranslateGemma SHA256 is from community repo `bullerwins`; should be cross-checked against `mradermacher/translategemma-4b-it-GGUF` or the official `google/translategemma-4b-it` during implementation.

> **Note on TranslateGemma repo choice:** `bullerwins/translategemma-4b-it-GGUF` was chosen because it has Q4_K_M available. The original Google repo (`google/translategemma-4b-it`) does not provide GGUF quantizations directly. Alternative: `mradermacher/translategemma-4b-it-GGUF` (also has Q4_K_M, file: `translategemma-4b-it-it-Q4_K_M.gguf`, size ~2.49 GB) — verify SHA256 during implementation and pick the more stable/trusted source.

---

## Standard Stack

### Core (already in Cargo.toml)

| Library | Version | Purpose | Status |
|---------|---------|---------|--------|
| `llama-cpp-2` | 0.1.146 | In-process GGUF inference | Already in Cargo.toml, feature-gated per target |
| `tokio` | 1.43.0 | Async runtime; `spawn_blocking` for CPU-bound inference | Already a dependency |
| `reqwest` | 0.12 | HTTP download with streaming and Range header | Already a dependency |
| `sha2` | 0.10 | SHA256 verification post-download | Already a dependency |
| `tauri-plugin-dialog` | 2.6 | File picker for custom GGUF import | Already a dependency |

### llama-cpp-2 Feature Flags (confirmed in Cargo.toml lines 97-116)

```toml
# Windows (x86_64, ARM64)
[target.'cfg(windows)'.dependencies]
llama-cpp-2 = { version = "0.1.146", features = ["vulkan"] }

# macOS
[target.'cfg(target_os = "macos")'.dependencies]
llama-cpp-2 = { version = "0.1.146", features = ["metal"] }

# Linux
[target.'cfg(target_os = "linux")'.dependencies]
llama-cpp-2 = { version = "0.1.146", features = ["vulkan"] }
```

No Cargo.toml changes required for the feature flags — already configured from Phase 10.

### New Module Needed

A new `src-tauri/src/managers/llm.rs` file (mirroring `managers/model.rs` + `managers/transcription.rs` patterns). No new external crates needed.

---

## Architecture Patterns

### Recommended Project Structure (new files only)

```
src-tauri/src/
├── managers/
│   └── llm.rs              # NEW: LlmManager (download + load + infer + unload)
├── commands/
│   └── llm.rs              # NEW: Tauri commands for LLM library and inference
src/
├── components/settings/post-processing/
│   └── LlmLibrarySection.tsx  # NEW: replaces placeholder card
│   └── EmbeddedNoModelEmptyState.tsx  # NEW: inline empty state
│   └── CustomGgufDropZone.tsx         # NEW: drag-drop + file picker button
```

### Pattern 1: LlmManager — mirroring ModelManager + TranscriptionManager

**What:** A single `LlmManager` struct handles: catalogue definition, download state, file operations, model loading/unloading, inference dispatch, and idle-watcher.

**Key types:**

```rust
// Source: mirrors model.rs ModelInfo pattern
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LlmModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub filename: String,
    pub url: Option<String>,        // None for custom models
    pub sha256: Option<String>,     // None for custom models (skip verification)
    pub size_mb: u64,               // shown BEFORE download (MDL-01)
    pub is_downloaded: bool,
    pub is_downloading: bool,
    pub partial_size: u64,          // for resume display
    pub is_custom: bool,
    pub is_recommended: bool,       // Qwen2.5-1.5B = true
}

pub struct LlmManager {
    app_handle: AppHandle,
    models_dir: PathBuf,            // same $APP_DATA/models/ as ModelManager
    available_models: Mutex<HashMap<String, LlmModelInfo>>,
    cancel_flags: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
    // Loaded model state (from transcription.rs pattern)
    loaded_model: Arc<Mutex<Option<LoadedLlmModel>>>,
    active_model_id: Arc<Mutex<Option<String>>>,    // persisted in settings
    last_activity: Arc<AtomicU64>,
    shutdown_signal: Arc<AtomicBool>,
    watcher_handle: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
    is_loading: Arc<Mutex<bool>>,
    loading_condvar: Arc<Condvar>,
}

struct LoadedLlmModel {
    model: Arc<LlamaModel>,
    ctx: LlamaContext<'static>,  // or managed lifetime
}
```

**Critical design note:** `LlamaModel` and `LlamaContext` from `llama-cpp-2` are **not `Send + Sync`** by default at the high level (the backend is C FFI). Inference must run on a `spawn_blocking` Tokio task that owns the context. The `LlamaBackend` must be initialized once at startup and stored in `LlmManager`. The `LlamaModel` is created from the backend — it is `Arc`-sharable. `LlamaContext` is created fresh per session or reused.

### Pattern 2: GPU Backend Initialization

```rust
// Source: docs.rs/llama-cpp-2, confirmed from ramintoosi.ir example
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::model::params::LlamaModelParams;

// Init once at app startup — store in LlmManager
let backend = LlamaBackend::init()?;

// Load with GPU offloading (all layers to GPU)
let model_params = LlamaModelParams::default()
    .with_n_gpu_layers(i32::MAX);  // GPU max; gracefully falls back to CPU if no GPU

let model = LlamaModel::load_from_file(&backend, &model_path, &model_params)?;
```

`with_n_gpu_layers(i32::MAX)` is the recommended pattern for "use GPU if available, CPU otherwise" — llama.cpp silently caps to available GPU layers. No per-platform detection logic needed beyond the Cargo feature flags already in place.

### Pattern 3: Inference on Background Thread

**What:** Inference runs inside `tokio::task::spawn_blocking` to avoid blocking the async executor. This is identical to the SHA256 verification pattern already in `model.rs:1200`.

```rust
// Source: model.rs:1200 pattern, adapted for inference
pub async fn run_inference(&self, prompt: String) -> Result<String> {
    let loaded = {
        let guard = self.loaded_model.lock().unwrap();
        guard.as_ref().map(|m| m.model.clone())
            .ok_or_else(|| anyhow::anyhow!("No model loaded"))?
    };
    let backend = self.backend.clone();

    let result = tokio::task::spawn_blocking(move || {
        // Context creation + inference loop runs on blocking thread pool
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(2048))
            .with_n_threads(4);
        let mut ctx = loaded.new_context(&backend, ctx_params)?;
        // ... tokenize, decode, generate tokens loop
        anyhow::Ok(output_text)
    })
    .await
    .map_err(|e| anyhow::anyhow!("Inference task panicked: {}", e))??;

    self.touch_activity();  // reset idle timer
    Ok(result)
}
```

**Streaming tokens:** For Phase 11, streaming to frontend is optional (Phase 11 only needs post-processing result, not live streaming). Use a simple `String` accumulator and emit the final result via the existing post-processing response path. Streaming can be added in Phase 12-13 if needed.

### Pattern 4: Idle Watcher (direct copy from transcription.rs:92-158)

The LLM idle watcher is structurally identical to `TranscriptionManager`'s watcher:
- `last_activity: Arc<AtomicU64>` — reset on each inference call (`touch_activity()`)
- Watcher thread: `thread::spawn`, check every 10s, read `model_unload_timeout` from settings
- Use **a separate setting key** for LLM idle timeout (e.g., `llm_unload_timeout: ModelUnloadTimeout`) to avoid coupling with the transcription timeout. Add to `AppSettings` with same `ModelUnloadTimeout` enum — default `Min5`.
- `shutdown_signal: Arc<AtomicBool>` — set to `true` in `LlmManager::shutdown()` called from `app.on_window_event`

### Pattern 5: Download Pipeline (copy from model.rs)

GGUF files are single flat files — not tar.gz archives. The download pipeline from `ModelManager` is reused with one simplification: skip the extraction step entirely. The relevant functions to mirror:
- `download_model()` — range-request resume, 10/sec event throttle, `DownloadCleanup` RAII
- `verify_sha256()` + `compute_sha256()` — identical, copy verbatim
- `delete_model()` — file-based (non-directory) path only
- `cancel_download()` — cancel flag pattern, identical

**Events to emit** (mirror transcription model events, add `llm-` prefix):
- `llm-download-progress` (payload: `LlmDownloadProgress { model_id, downloaded, total, percentage }`)
- `llm-verification-started` / `llm-verification-completed`
- `llm-download-complete` / `llm-download-cancelled` / `llm-download-failed`
- `llm-model-loaded` / `llm-model-unloaded`
- `llm-deleted`

### Pattern 6: Custom GGUF Import

```rust
// GGUF magic validation — first 4 bytes must be b"GGUF"
pub fn validate_gguf_header(path: &Path) -> Result<()> {
    let mut file = File::open(path)?;
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;
    if &magic != b"GGUF" {
        return Err(anyhow::anyhow!("Not a valid GGUF file"));
    }
    Ok(())
}
```

After validation: copy to `$APP_DATA/models/<filename>`, add to `available_models` with `is_custom: true`, emit `llm-custom-model-imported`. On failure: return error string to frontend for `Alert` display.

### Pattern 7: Provider Integration (actions.rs)

The existing post-processing path in `actions.rs` routes by `provider.id`. The "embedded" branch:

```rust
// In actions.rs post_process() function
if provider.id == "embedded" {
    let llm_manager = app.state::<Arc<LlmManager>>();
    return llm_manager.run_inference(user_content).await.ok();
}
```

The `LlmManager::run_inference()` replaces the HTTP round-trip of `llm_client.rs`. No identity headers needed (local inference). The `system_prompt` and `user_content` construction upstream remains unchanged.

### Pattern 8: Settings — Active LLM Model ID

Add a single field to `AppSettings` to track which LLM model is active:

```rust
// In settings.rs AppSettings struct
#[serde(default)]
pub active_llm_model_id: Option<String>,  // None = no model selected yet

// Separate from selected_model (which is the transcription model)
// Separate timeout for LLM idle (reuse ModelUnloadTimeout enum)
#[serde(default)]
pub llm_unload_timeout: ModelUnloadTimeout,  // default: Min5
```

These two new settings fields need `#[serde(default)]` to be backward-compatible with existing `settings_store.json` files.

### Anti-Patterns to Avoid

- **Do NOT share `LlamaContext` across async tasks** — context is not thread-safe; create per inference call or use a single dedicated blocking thread.
- **Do NOT store `LlamaModel` inside a `tokio::Mutex`** — use `std::sync::Mutex` like `TranscriptionManager` uses for `engine`.
- **Do NOT use `tauri dev` to test bundle resources** — `.metallib` files are only present in `tauri build` output. Always verify macOS GPU acceleration with a release build.
- **Do NOT emit download-progress events on every chunk** — throttle to 100ms intervals (already done in `model.rs:1122`; copy the pattern exactly).
- **Do NOT add "embedded" to `default_post_process_providers()`** — embedded is not an HTTP provider; it has no `base_url` to store. Keep it as a special-case branch in `actions.rs`, not a member of the provider list.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| GPU kernel shaders (macOS) | Custom Metal kernels | `llama-cpp-2` with `"metal"` feature | llama.cpp handles shader compilation at build time; `.metallib` output from OUT_DIR |
| GPU kernel shaders (Win/Linux) | Custom Vulkan shaders | `llama-cpp-2` with `"vulkan"` feature | Same: shader compilation handled by llama-cpp-sys-2 build script |
| Range-request HTTP resume | Custom byte-range downloader | Copy `ModelManager.download_model()` verbatim | Already handles 206/200 fallback, resume_from logic, edge cases |
| SHA256 streaming hash | Custom hash loop | Copy `ModelManager.compute_sha256()` verbatim | 64KB chunk reads, already handles large files |
| Token generation loop | Custom sampler | `llama_cpp_2::context::LlamaContext` + `sampling` module | Temperature, greedy, top-k/p are built into the sampler |
| GGUF header parsing | Custom parser | Read 4 bytes + `llama-cpp-2` rejects malformed files on load | Magic byte check is sufficient for user-facing validation; deep parse done by llama.cpp |
| Idle timer thread | Custom timing mechanism | Copy `TranscriptionManager` watcher thread verbatim | Pattern is tested, handles recording guard, shutdown signal |

---

## Common Pitfalls

### Pitfall 1: macOS `.metallib` Bundle Resources

**What goes wrong:** `tauri dev` does NOT replicate the production bundle layout. Metal shaders compiled by `llama-cpp-2`'s build script land in the Cargo `OUT_DIR` and need to be included in `tauri.conf.json bundle.resources`. If omitted, GPU acceleration silently falls back to CPU (no crash, just slower) or fails on some hardware. The exact `.metallib` filenames are **not documented** in the crate README.

**How to avoid:** After Wave 1 backend implementation, run `tauri build` (not `tauri dev`) on macOS and inspect the build output: `find target/release/build -name "*.metallib"`. Then add those paths to `bundle.resources` in `tauri.conf.json`. This is the ROADMAP spike gate — Phase 12 does not open until this is confirmed.

**Warning signs:** GPU inference benchmark shows identical speed to CPU; `tauri build` produces a larger binary than expected for GPU support.

### Pitfall 2: Windows x64 vulkan-shaders-gen MSVC Build Failure

**What goes wrong:** CI run 26749959299 confirmed: `vulkan-shaders-gen` ExternalProject cmake_install.cmake is not generated when MSBuild runs the install step on MSVC x64. This is a known upstream llama.cpp bug (issue #11788, closed but root-cause is a CMake/MSBuild path issue with `glslc` linking). **Windows ARM64 builds fine** — only x64 is affected.

**How to avoid:** Options in priority order:
1. Check if `llama-cpp-2 > 0.1.146` includes a fix — the upstream llama.cpp issue was closed, suggesting a patch exists.
2. Apply the workaround from llama.cpp issue #11788: configure `cmake -G "Ninja"` instead of MSBuild on Windows x64 (requires Ninja in CI).
3. As a fallback: disable Vulkan on Windows x64 only (`llama-cpp-2` with no features for that target) and use CPU only on that sub-platform.

**Warning signs:** CI Windows-x64 build fails at `cmake_install.cmake not generated` or `vulkan-shaders-gen` step.

### Pitfall 3: AMD Vulkan Driver Crash (Windows beta gate)

**What goes wrong:** AMD driver 25.11.1 + Vulkan SDK 1.4.328.1 causes a crash when context size > 512 tokens (llama.cpp issue #17432, opened November 2025, status open as of research date).

**How to avoid:** Keep the default context size for embedded inference at 2048 tokens (standard) but monitor issue #17432. If the issue is unresolved at Windows beta release, add AMD-specific fallback: detect GPU vendor before loading (via `list_llama_ggml_backend_devices()` from `llama-cpp-2`) and fall back to CPU for AMD GPUs on Windows pending driver fix.

**Warning signs:** Crash reports from Windows testers with AMD Radeon GPUs on post-processing.

### Pitfall 4: LlamaContext Lifetime and Thread Safety

**What goes wrong:** `LlamaContext` has a lifetime tied to `LlamaModel`. Trying to store `LlamaContext` across async boundaries (in Tauri managed state) leads to lifetime errors. The context also cannot be sent between threads naively.

**How to avoid:** Two valid approaches:
- **Per-call context:** Create a new `LlamaContext` for each inference call inside `spawn_blocking`. Simple, no lifetime issues, small overhead (~milliseconds).
- **Dedicated inference thread:** Use a `std::sync::mpsc` channel: the async command handler sends a `(prompt, oneshot::Sender<Result<String>>)` message; a dedicated OS thread owns `LlamaContext` and responds. This avoids repeated context creation overhead for rapid successive calls.

For Phase 11 (post-processing only, not streaming), per-call context creation is acceptable. The dedicated thread pattern is preferred if context creation overhead proves significant (test with benchmarks).

### Pitfall 5: Double-Loading Protection

**What goes wrong:** If the user clicks "Use this model" while a model is already loading (or inference is running), a second load can corrupt state.

**How to avoid:** Copy `TranscriptionManager.try_start_loading()` pattern — `Arc<Mutex<bool>>` + `Condvar`-notified `LoadingGuard`. Return an error to the frontend if a load is already in progress.

### Pitfall 6: Settings Migration (active_llm_model_id)

**What goes wrong:** Adding a new field to `AppSettings` without `#[serde(default)]` causes deserialization to fail for existing users (their `settings_store.json` doesn't have the field), falling back to default settings and potentially losing other settings.

**How to avoid:** Every new `AppSettings` field MUST have `#[serde(default)]`. `active_llm_model_id: Option<String>` defaults to `None` — safe. `llm_unload_timeout: ModelUnloadTimeout` has `#[default]` on the enum variant (`Min5`) — safe.

---

## Code Examples

### Backend: LlamaBackend + Model Load

```rust
// Source: docs.rs/llama-cpp-2 + ramintoosi.ir example (October 2024, verified against 0.1.146 API)
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::context::params::LlamaContextParams;
use std::num::NonZeroU32;
use std::path::Path;

// Initialize once at startup
let backend = LlamaBackend::init()?;

// Load model (GPU if available, CPU fallback)
let model_params = LlamaModelParams::default()
    .with_n_gpu_layers(i32::MAX);

let model = LlamaModel::load_from_file(&backend, model_path, &model_params)?;
```

### Backend: Inference in spawn_blocking

```rust
// Source: pattern from model.rs:1200 + llama-cpp-2 example
let model_arc = model.clone();
let backend_arc = backend.clone();

let result = tokio::task::spawn_blocking(move || -> anyhow::Result<String> {
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(NonZeroU32::new(2048))
        .with_n_threads(num_cpus::get() as u32 / 2);

    let mut ctx = model_arc.new_context(&backend_arc, ctx_params)?;

    let tokens_list = model_arc.str_to_token(&prompt, AddBos::Always)?;
    let mut batch = llama_cpp_2::llama_batch::LlamaBatch::new(2048, 1);

    for (i, token) in tokens_list.iter().enumerate() {
        batch.add(*token, i as i32, &[0], i == tokens_list.len() - 1)?;
    }
    ctx.decode(&mut batch)?;

    let mut output = String::new();
    for _ in 0..max_tokens {
        let candidates = ctx.candidates_ith(batch.n_tokens() - 1);
        let candidates_p = llama_cpp_2::token::data_array::LlamaTokenDataArray::from_iter(
            candidates, false
        );
        let new_token = ctx.sample_token_greedy(candidates_p);
        if model_arc.is_eog_token(new_token) { break; }
        output.push_str(
            &String::from_utf8_lossy(
                &model_arc.token_to_bytes(new_token, llama_cpp_2::model::Special::Tokenize)?
            )
        );
        // Add to next batch
        batch.clear();
        batch.add(new_token, (tokens_list.len() + output.len()) as i32, &[0], true)?;
        ctx.decode(&mut batch)?;
    }
    Ok(output)
})
.await
.map_err(|e| anyhow::anyhow!("Inference panicked: {}", e))??;
```

### Backend: GGUF Header Validation

```rust
// Source: GGUF spec (https://ggml-org-ggml.mintlify.app/formats/gguf)
// Magic bytes: 0x47 0x47 0x55 0x46 = "GGUF" in ASCII
use std::io::Read;
use std::fs::File;
use std::path::Path;

pub fn validate_gguf_header(path: &Path) -> Result<(), String> {
    let mut file = File::open(path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)
        .map_err(|_| "File too small to be a GGUF model".to_string())?;
    if &magic != b"GGUF" {
        return Err("Not a valid GGUF model file".to_string());
    }
    Ok(())
}
```

### Frontend: Adding "embedded" to LOCAL_PROVIDER_IDS_SET

```typescript
// Source: PostProcessingSettings.tsx line 28
// BEFORE:
const LOCAL_PROVIDER_IDS_SET = new Set(["apple_intelligence", "custom"]);
// AFTER:
const LOCAL_PROVIDER_IDS_SET = new Set(["apple_intelligence", "custom", "embedded"]);
```

### Frontend: LlmLibrarySection layout skeleton

```typescript
// Source: UI-SPEC.md Layout Contract
// Replaces placeholder at PostProcessingSettings.tsx ~line 638
<SettingsGroup title={t("settings.postProcessing.modelsAndLocalProcessing.library.title")}>
  <p className="text-sm text-mid-gray leading-relaxed">
    {t("settings.postProcessing.modelsAndLocalProcessing.library.description")}
  </p>
  {isLoading ? (
    <div className="py-16 flex justify-center">
      <div className="w-8 h-8 border-2 border-logo-primary border-t-transparent rounded-full animate-spin" />
    </div>
  ) : (
    <>
      {downloadedModels.length > 0 && (
        <>
          <p className="text-sm font-medium text-text/60">
            {t("settings.postProcessing.modelsAndLocalProcessing.library.yourModels")}
          </p>
          <div className="space-y-3">
            {downloadedModels.map(model => <ModelCard key={model.id} {...modelCardProps(model)} />)}
          </div>
        </>
      )}
      <p className="text-sm font-medium text-text/60">
        {t("settings.postProcessing.modelsAndLocalProcessing.library.availableModels")}
      </p>
      <div className="space-y-3">
        {catalogueModels.map(model => <ModelCard key={model.id} {...modelCardProps(model)} />)}
      </div>
      <CustomGgufDropZone />
    </>
  )}
</SettingsGroup>
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| External Ollama process | In-process `llama-cpp-2` | Phase 10 decision | No external process dependency; faster startup |
| tar.gz archive download + extract | Direct GGUF single-file download | N/A for LLM (transcription models use tar.gz) | Simpler: skip extraction step in download pipeline |
| blob.handy.computer CDN | HuggingFace CDN `resolve/main/` URLs | Phase 11 requirement | Official model source; HF CDN is globally distributed |
| Placeholder card in PostProcessingSettings | Real `LlmLibrarySection` with download/manage | Phase 11 | Functional model library |

**Deprecated/outdated:**
- `blob.handy.computer` for any new model URLs — explicitly rejected for LLM models; existing transcription models on this CDN are a separate concern (INFR-01, deferred)

---

## Open Questions

1. **`.metallib` exact paths from llama-cpp-2 OUT_DIR**
   - What we know: llama.cpp with Metal compiles `.metallib` shader files during the build; these must be bundled as resources for GPU acceleration on macOS.
   - What's unclear: The exact filenames and relative paths within `target/release/build/llama-cpp-sys-2-*/out/` are not documented anywhere in the crate README or docs.rs.
   - Recommendation: After Wave 1 completes, run `tauri build` on macOS and execute `find target/release/build -name "*.metallib" -o -name "*.metal"` to discover the paths. Then add them to `tauri.conf.json "bundle": { "resources": [...] }`. This is the ROADMAP spike gate.

2. **Windows x64 vulkan-shaders-gen fix**
   - What we know: CI run 26749959299 failed; upstream llama.cpp issue #11788 is closed (suggesting a fix exists upstream).
   - What's unclear: Whether `llama-cpp-2 0.1.146` (released April 30, 2026) incorporates the upstream fix.
   - Recommendation: Wave 1 CI run will confirm. If still failing, try Ninja generator workaround; if that fails, disable Vulkan on Windows x64 (CPU only for that sub-target) as a release fallback.

3. **TranslateGemma SHA256 cross-verification**
   - What we know: `bullerwins/translategemma-4b-it-GGUF` SHA256 = `7f7357c14abd9da4eb200b38b05da502cd6e10d7e1d403fbc9f78c19f3209b72`.
   - What's unclear: Whether `bullerwins` is a trustworthy community quantizer; whether `mradermacher/translategemma-4b-it-GGUF` is a better source.
   - Recommendation: During implementation, compare SHA256 against at least one additional community repo; consider `mradermacher` (high-reputation community quantizer) as the primary source.

4. **LlamaContext reuse vs per-call creation**
   - What we know: Per-call creation avoids lifetime issues; dedicated thread avoids repeated overhead.
   - What's unclear: Context creation latency for 4B models on the target hardware.
   - Recommendation: Implement per-call first (simpler); benchmark against 5 rapid successive post-processing calls; if latency is > 500ms per call, migrate to dedicated thread.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` (Rust unit tests, inline `#[cfg(test)]` modules) |
| Config file | No separate config — standard `cargo test` |
| Quick run command | `cargo test -p dictus --lib -- managers::llm` |
| Full suite command | `cargo test -p dictus --lib` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| LLM-01 | Inference dispatched on background thread, does not block | unit | `cargo test -p dictus --lib -- managers::llm::tests` | ❌ Wave 0 |
| LLM-02 | GPU params set (n_gpu_layers = MAX), CPU fallback path | unit | `cargo test -p dictus --lib -- managers::llm::tests::test_model_params_gpu` | ❌ Wave 0 |
| LLM-03 | Idle watcher fires unload after timeout; activity touch resets timer | unit | `cargo test -p dictus --lib -- managers::llm::tests::test_idle_watcher` | ❌ Wave 0 |
| LLM-04 | `embedded` provider routes to LlmManager.run_inference | integration (manual) | N/A — requires app launch + model download | manual-only |
| MDL-01 | Catalogue models have size_mb > 0 before download | unit | `cargo test -p dictus --lib -- managers::llm::tests::test_catalogue_sizes` | ❌ Wave 0 |
| MDL-02 | SHA256 verification: pass on match, fail+delete on mismatch | unit | `cargo test -p dictus --lib -- managers::llm::tests::test_sha256` | ❌ Wave 0 |
| MDL-03 | delete_llm_model removes file and updates state | unit | `cargo test -p dictus --lib -- managers::llm::tests::test_delete` | ❌ Wave 0 |
| MDL-04 | GGUF header validation: accept valid, reject non-GGUF | unit | `cargo test -p dictus --lib -- managers::llm::tests::test_gguf_validation` | ❌ Wave 0 |
| MDL-05 | LlmLibrarySection renders (no crash, shows catalogue) | smoke (manual) | N/A — requires Tauri dev server | manual-only |

### Sampling Rate

- **Per task commit:** `cargo test -p dictus --lib -- managers::llm`
- **Per wave merge:** `cargo test -p dictus --lib && bun run lint`
- **Phase gate:** All unit tests green + manual end-to-end smoke test (download → load → infer → streaming tokens) on macOS, Windows, Linux before Phase 12 opens

### Wave 0 Gaps

- [ ] `src-tauri/src/managers/llm.rs` — new file, test module at bottom mirrors `model.rs` test pattern
  - `test_catalogue_sizes` — assert all 3 entries have `size_mb > 0` and valid HF URLs
  - `test_sha256_pass` / `test_sha256_fail_deletes_partial` — copy from `model.rs` test pattern, use `tempfile::TempDir`
  - `test_gguf_validation_pass` / `test_gguf_validation_reject_non_gguf` — write 4 magic bytes, assert pass; write garbage, assert fail
  - `test_delete_removes_file` — create temp `.gguf`, call `delete_llm_model`, assert file gone
  - `test_idle_timer_reset` — create manager, call `touch_activity()`, verify `last_activity` updated

---

## Sources

### Primary (HIGH confidence)

- `/Users/pierreviviere/dev/dictus-desktop/src-tauri/Cargo.toml` — llama-cpp-2 0.1.146, per-target feature flags (lines 97-116), existing dependency list
- `/Users/pierreviviere/dev/dictus-desktop/src-tauri/src/managers/model.rs` — full download pipeline, DownloadCleanup, SHA256, custom model discovery
- `/Users/pierreviviere/dev/dictus-desktop/src-tauri/src/managers/transcription.rs:92-158` — idle-watcher thread template
- `/Users/pierreviviere/dev/dictus-desktop/src-tauri/src/settings.rs` — AppSettings struct, PostProcessProvider, ModelUnloadTimeout
- `/Users/pierreviviere/dev/dictus-desktop/src-tauri/src/commands/models.rs` — Tauri command pattern for model operations
- `docs.rs/llama-cpp-2` — LlamaBackend, LlamaModel, LlamaContext, sampling module documentation
- HuggingFace `Qwen/Qwen2.5-1.5B-Instruct-GGUF` file page — SHA256 and filename confirmed 2026-06-01
- HuggingFace `Qwen/Qwen3-4B-GGUF` file page — SHA256 and filename confirmed 2026-06-01
- HuggingFace `bullerwins/translategemma-4b-it-GGUF` file page — SHA256 and filename confirmed 2026-06-01

### Secondary (MEDIUM confidence)

- ramintoosi.ir Llama 3.2 in Rust blog post (October 2024) — verified LlamaBackend/LlamaModel/LlamaContext API patterns against 0.1.x series
- github.com/utilityai/llama-cpp-rs README — confirms feature flag names (metal, vulkan, cuda), version 0.1.146 released April 30, 2026
- GGUF format spec (ggml-org-ggml.mintlify.app) — magic bytes `0x47 0x47 0x55 0x46`
- llama.cpp issue #11788 (MSVC + Vulkan, closed) — root cause and status

### Tertiary (LOW confidence — flagged for validation)

- TranslateGemma SHA256 from `bullerwins` community repo — needs cross-verification against `mradermacher` during implementation
- AMD Vulkan driver crash (issue #17432, November 2025) — status open as of research; monitor during Windows beta gate

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all crates confirmed in Cargo.toml, already compiling per Phase 10
- Architecture: HIGH — direct template code exists in model.rs and transcription.rs; API patterns verified against live docs
- Catalogue metadata (URLs/SHA256): MEDIUM — live HuggingFace pages read 2026-06-01; SHA256 for TranslateGemma needs cross-check
- Pitfalls: HIGH — .metallib and Windows x64 Vulkan are confirmed known risks from STATE.md; AMD driver crash is confirmed open issue
- Validation: HIGH — test patterns copied from existing model.rs test suite

**Research date:** 2026-06-01
**Valid until:** 2026-07-01 (stable llama-cpp-2 API; HF file hashes could change if repo maintainer updates)
