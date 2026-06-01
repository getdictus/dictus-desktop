# Phase 11: LLM Runtime Foundation - Context

**Gathered:** 2026-06-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can download, manage, and run a local GGUF model **in-process** (via `llama-cpp-2`, confirmed viable in Phase 10), with GPU acceleration where available, and the "Bibliothèque de modèles locaux" placeholder card becomes a **real, functional model library** anchored at the top of the post-processing (local-processing) page.

Delivers: curated 3-model catalogue, in-app download (progress/cancel/resume, SHA256-verified, HuggingFace CDN), delete + disk reclaim, custom GGUF import (drag-drop + file picker), and "Embedded (local)" as a selectable post-processing provider running on a background thread with per-platform GPU acceleration and independent idle-unload.

**Out of this phase** (belongs to 12-13): Smart Modes data model, default modes, per-mode shortcuts, the card-list Smart Modes UI, and translation presets. Phase 11 only makes the runtime + model library real.

</domain>

<decisions>
## Implementation Decisions

### Catalogue & quantization
- **One curated GGUF per catalogue model** — no per-model quant variants in the library. Quant-tier labels (Small/Balanced/Quality) remain deferred to MDL-F3.
- **The three models span the speed↔quality range** so users get the choice via *which model they pick*, not via quant variants:
  - **Qwen2.5-1.5B → Q4_K_M** (~1GB) — the small/fast pick for low-end machines
  - **Qwen3-4B → Q4_K_M** (~2.5GB) — the quality pick
  - **TranslateGemma-4B → Q4_K_M** (~2.5GB) — translation-oriented (groundwork for Phase 13)
- All shipped at **Q4_K_M (balanced)** tier. Exact HF repo, filename, byte size, and SHA256 per model are confirmed during research/planning.
- **Recommended default model = Qwen2.5-1.5B** (`is_recommended`-style flag) — lowest-friction first download, runs on any machine; Qwen3-4B is presented prominently as the higher-quality option.
- Rationale to preserve: the small/fast vs quality tradeoff was deliberately pushed into model selection (1.5B vs 4B) rather than adding quant rows.

### Embedded provider behavior
- **"Embedded (local)" is always selectable** as a post-processing provider (add to the local-provider set alongside `apple_intelligence` and `custom`). First-time users must be able to discover it even before downloading a model.
- **No-model state:** when embedded is selected with no GGUF present, show an **inline empty state** in the provider area ("download a model below to enable embedded processing") that **scrolls/links to the library card on the same page with the recommended model (Qwen2.5-1.5B) highlighted for one-click download.** Never a blocking modal, never a silent auto-download.
- **Active model selection:** the user marks one downloaded LLM as **active in the library**, reusing the existing transcription active-model indicator pattern. The embedded provider always runs the marked-active GGUF. (No separate provider-level dropdown.)
- **No silent default switch:** keep v1.2's platform-aware default provider logic unchanged (Apple Intelligence on macOS arm64, Custom elsewhere). Embedded is **not** auto-promoted to default on upgrade. Rationale: (1) cannot default to a provider that requires a multi-GB download — no zero-state exists; (2) avoids surprising existing users on upgrade. LLM-04's "embedded is the primary *local* option" is satisfied by prominent presentation, not by forcing the active provider.

### Library UI treatment
- **Reuse the existing transcription model-card style** (`ModelsSettings.tsx` visual language) verbatim — download button, progress bar, cancel, delete, active indicator. No second "pick a local model" visual pattern.
- **Each entry shows:** on-disk size (before download per MDL-01, and after), a one-line role/description ("Fast, lightweight" / "Best quality" / "Translation-focused"), active + downloaded/downloadable state, and live download progress with cancel (resume on reopen).
- **One list, custom models tagged** with a small "Custom" badge alongside catalogue entries (mirrors the existing `is_custom` handling for transcription models).

### Custom GGUF import
- **Both import methods:** drag-and-drop onto the library/drop-zone *and* an "Add custom model" file-picker button (`tauri-plugin-dialog`, already a dependency). File picker is the reliable baseline (Linux/Wayland drag-drop can be flaky — acceptable since picker is the fallback).
- **Copy the file into `$APP_DATA/models/`** (same directory as downloaded models). This is the OS application-data directory, **never replaced by app updates** (only the app bundle is) — so custom models persist across every Dictus update, which is a hard requirement from the user. Reference-in-place is rejected (breaks if the user moves/deletes the original; ambiguous delete semantics).
- **Validate the GGUF header at import time** (4-byte `GGUF` magic marker; `llama-cpp-2` also rejects malformed/unsupported files on parse). If invalid, **reject inline with a clear "not a valid GGUF model" message** and do not add a broken entry to the library.

### Claude's Discretion
- Exact HF repo URLs, filenames, byte sizes, and SHA256 hashes for the 3 catalogue GGUFs (research-confirmed).
- GPU backend detection mechanism and whether to adopt upstream `966ff99` async GPU query (flagged forward from Phase 10).
- Idle-unload timeout value/surfacing for the LLM (independent of the transcription unload; reuse the watcher pattern).
- Inference parameters (context length, max tokens, temperature) for post-processing.
- Background-thread/concurrency design for inference.
- Download error/retry behavior, resume mechanics, drop-zone exact placement.
- `.metallib` bundle-resource wiring for macOS (verify via `tauri build` release smoke test — see ROADMAP spike flag).
- Windows-x64 `vulkan-shaders-gen` MSVC build fix (carried from Phase 10 as a Phase 11 gate).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase definition & requirements
- `.planning/ROADMAP.md` §"Phase 11: LLM Runtime Foundation" — 6 success criteria + research spike flag (`.metallib` bundle resources verified via `tauri build`, not `tauri dev`).
- `.planning/REQUIREMENTS.md` — LLM-01..04, MDL-01..05 acceptance criteria; Future Requirements MDL-F1/F2/F3/F4 (explicitly deferred — do NOT build); Out of Scope table (no sidecar/`llama-server`, no silent cloud fallback, no `blob.handy.computer` hosting).
- `.planning/PROJECT.md` — Local-first constraint; v1.3 scope decisions (all platforms ship together, embedded adds-not-replaces, cloud opt-in).

### Engine & coexistence (from Phase 10)
- `.planning/phases/10-prerequisite-gate/10-CONTEXT.md` — Engine = `llama-cpp-2` 0.1.146 decision, candle/mistral.rs fallback (not needed), `[patch.crates-io]` pattern.
- `.planning/STATE.md` §Blockers/Concerns — Phase 11 gates carried from Phase 10: Windows-x64 `vulkan-shaders-gen` MSVC build failure (CI run 26749959299), `.metallib` bundle-resource verification, AMD Vulkan driver crash risk (llama.cpp #17432), upstream `966ff99` async GPU query flagged for adoption decision.
- `src-tauri/Cargo.toml` — `transcribe-rs` per-platform feature gating + `[patch.crates-io]` block; `llama-cpp-2` GPU features follow the same per-target pattern.

### Model library & download (reuse targets)
- `src-tauri/src/managers/model.rs` — `ModelInfo` struct (incl. `url`, `sha256`, `is_downloaded`, `is_downloading`, `partial_size`, `is_custom`), `ModelManager`, `DownloadProgress`, `DownloadCleanup` RAII guard, SHA256 verification, tar.gz extraction. The LLM library mirrors this download/verify/cancel/delete machinery.
- `src-tauri/src/commands/models.rs` — existing model Tauri commands (download/cancel/delete) to mirror for LLM models.
- `src/components/settings/models/ModelsSettings.tsx` — the transcription model-picker UI whose card style is reused (download/cancel/delete/progress/active-indicator UX, `is_custom` tagging, active-model sort).
- `src/components/model-selector/` — `ModelSelector.tsx`, `DownloadProgressDisplay.tsx`, `ModelDropdown.tsx`, `ModelStatusButton.tsx` (supporting components).

### Provider integration
- `src/components/settings/post-processing/PostProcessingSettings.tsx` — `LOCAL_PROVIDER_IDS_SET = ["apple_intelligence", "custom"]` (line 28; add `"embedded"`); placeholder library card render site (~line 638, `…modelsAndLocalProcessing.library.*` i18n keys); platform-aware default logic (~line 580+).
- `src/components/settings/PostProcessingSettingsApi/` — `usePostProcessProviderState.ts`, `ProviderSelect.tsx`, `ProviderPicker.tsx`, `types.ts` (provider selection state).
- `src-tauri/src/settings.rs` — `PostProcessProvider` struct (now derives `Default` after Phase 10-02); `enable_cloud_providers` vestigial field.
- `src-tauri/src/llm_client.rs` — post-process client; `ChatCompletionParams` struct (Phase 10-02 refactor); identity headers (`X-Title: Dictus`) must stay.
- `src-tauri/src/actions.rs` §207/§265 — post-processing call sites.

### Lifecycle / unload
- `src-tauri/src/managers/transcription.rs` §92-158 — idle-watcher thread + `unload_model()` (`ModelUnloadTimeout`); template for the LLM's independent idle-unload (LLM-03).

### Storage & i18n
- `.planning/codebase/INTEGRATIONS.md` — model files at `$APP_DATA/models/`, settings via tauri-plugin-store, SHA256 download verification, per-platform GPU backends.
- `src/i18n/locales/en/translation.json` §`settings.postProcessing.modelsAndLocalProcessing.library.*` — placeholder strings to replace; new strings propagate to 20 locales in Phase 13 (L10N-01), not here.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`ModelInfo` / `ModelManager` (`managers/model.rs`)** — already has `url`, `sha256`, `is_downloaded`, `is_downloading`, `partial_size`, `is_custom`, plus a `DownloadCleanup` RAII guard and SHA256 verification. The LLM library reuses this download/verify/cancel/delete machinery (LLM GGUFs are single files, not tar.gz — extraction step skipped).
- **`ModelsSettings.tsx`** — the transcription picker; its card style (download/cancel/delete/progress/active-indicator, `is_custom` tag, active-first sort) is reused verbatim for the LLM library.
- **`tauri-plugin-dialog`** — already a dependency; powers the custom-model file picker.
- **Idle-watcher pattern (`transcription.rs:92-158`)** — template for the LLM's own independent idle-unload.
- **`[patch.crates-io]` block (`Cargo.toml`)** — fallback mechanism if any ggml duplication resurfaces (didn't manifest in Phase 10).

### Established Patterns
- **`$APP_DATA/models/` is update-safe** — downloaded models persist across app updates because the app-data dir is never replaced (only the bundle is). Custom GGUFs copied here inherit that persistence.
- **Per-platform Cargo features** — `transcribe-rs` feature-gates the whisper backend by OS; `llama-cpp-2` GPU features (Metal/Vulkan) follow the same per-target pattern.
- **Local-provider set** — `LOCAL_PROVIDER_IDS_SET` in `PostProcessingSettings.tsx` gates which providers count as "local"; `"embedded"` is added here.
- **Identity preservation** — `X-Title: Dictus` and related headers in `llm_client.rs` must remain through any provider changes.

### Integration Points
- `managers/model.rs` ↔ `commands/models.rs` ↔ `ModelsSettings.tsx` — the existing download pipeline the LLM library plugs into.
- New embedded LLM runtime ↔ `llm_client.rs` / `actions.rs` post-processing path — "Embedded (local)" provider routes here instead of an HTTP endpoint.
- `PostProcessingSettings.tsx` library-card render site (~line 638) — the placeholder card is replaced by the real library, still anchored at the top.
- macOS `.metallib` from `llama-cpp-2` `OUT_DIR` → `tauri.conf.json bundle.resources` (unconfirmed paths; verify via `tauri build` release smoke test before Phase 12 — ROADMAP spike flag).

</code_context>

<specifics>
## Specific Ideas

- **Speed↔quality choice lives in model selection, not quant variants** — the 1.5B (fast) and 4B (quality) models *are* the user-facing tradeoff. Preserve this framing; don't reintroduce per-model quant rows (that's deferred MDL-F3).
- **Best-possible no-model UX** — the user explicitly asked for the best UX when embedded is selected without a model: discoverable + guided (inline empty state → highlighted recommended download on the same page), never hidden, never blocking, never auto-downloading.
- **Custom-model persistence across updates is a hard requirement** (user-stated) — `$APP_DATA/models/` copy-in satisfies it; do not reference files in place.
- **Invalid custom files rejected at import** with a clear user-facing message — validate before adding to the library, no dead entries.
- Working branch is `feat/v1.3-smart-modes`.

</specifics>

<deferred>
## Deferred Ideas

- **Quant-tier labels / multiple quants per model** (MDL-F3) — explicitly out of v1.3; the catalogue ships one Q4_K_M per model.
- **Hardware fit badge** (MDL-F2), **GPU/CPU status badge** (MDL-F4), **RAM-based auto-quant recommendation** (MDL-F1) — Future Requirements, not Phase 11.
- **Larger catalogue / HF browse-search** (MDL-F5/F6) — future.
- **Embedded becomes the auto-default provider** — considered and rejected for v1.3 (upgrade-stability + no zero-state for a download-required provider). Could revisit once a model is guaranteed present.
- **Per-mode provider selection** (MODE-F3, a different LLM per Smart Mode) — Smart Modes territory, future.
- **CUDA backend** (LLM-F1) / **n_gpu_layers slider** (LLM-F2) — v1.4+.

</deferred>

---

*Phase: 11-llm-runtime-foundation*
*Context gathered: 2026-06-01*
