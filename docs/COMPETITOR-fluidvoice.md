# FluidVoice Competitive Analysis

**Date:** 2026-06-29  
**Competitor:** [`altic-dev/FluidVoice`](https://github.com/altic-dev/FluidVoice)  
**Analysed commit:** `ddae598` (`main`, 2026-06-28)  
**Local clone:** `/home/clawdbot/clawd/projects/FluidVoice`  
**Dictus Desktop baseline:** `getdictus/dictus-desktop` `main` at `d49d270`

## Executive summary

FluidVoice is a strong direct macOS competitor. It is narrower than Dictus Desktop on platform support, but currently stronger on macOS-specific product polish and on speech model breadth.

The most important takeaways:

1. **FluidVoice's strongest moat is macOS-native execution**: Swift/AppKit, menu bar, Dynamic Island/notch-aware overlay, Accessibility-based direct typing, Homebrew distribution, and Apple Silicon-first CoreML models.
2. **Speech model coverage is broader than Dictus Desktop's README suggests**, especially on Apple Silicon: Nemotron Speech 3.5, Parakeet Flash, Parakeet TDT v3/v2, Cohere Transcribe, Apple Speech, and Whisper.
3. **Its local post-processing story is partially closed**: "Fluid Intelligence" is marketed as local/on-device, but the runtime is private and separately maintained. The open-source app supports cloud providers and local OpenAI-compatible endpoints, but the flagship local AI layer is not fully auditable.
4. **Dictus Desktop's strategic edge is cross-platform + fully open local LLM runtime**: the current main branch already includes `llama-cpp-2`, a GGUF catalogue, custom GGUF import, and smart-mode UI primitives. That should become a headline feature, not an implementation detail.
5. **The biggest improvement opportunities for Dictus are not only models**: live preview/streaming UX, per-app prompt routing, command/edit modes, audio history, onboarding, and distribution polish are where FluidVoice feels more mature.

## Product positioning

| Area                              | FluidVoice                                               | Dictus Desktop                                  | Takeaway                                                                                                  |
| --------------------------------- | -------------------------------------------------------- | ----------------------------------------------- | --------------------------------------------------------------------------------------------------------- |
| Platforms                         | macOS only                                               | macOS, Windows, Linux                           | Dictus has the broader positioning. Keep this as a core differentiator.                                   |
| License                           | GPLv3 from 2026-02-23 onward                             | MIT                                             | Dictus is easier to embed, fork, reuse, and contribute to commercially.                                   |
| Primary UX                        | macOS menu-bar/native dictation app                      | Cross-platform Tauri desktop app                | FluidVoice can move faster on macOS-specific UX; Dictus needs restraint and consistency across platforms. |
| Distribution                      | Homebrew cask + manual release                           | GitHub Releases/packages                        | Homebrew should be considered for Dictus macOS.                                                           |
| Core promise                      | Local-first macOS dictation with optional AI enhancement | 100% offline dictation across desktop platforms | Dictus should sharpen: “open, cross-platform, no private runtime.”                                        |
| Community signal at analysis time | ~4,146 stars, 257 forks, latest `v1.6.1`                 | Smaller public footprint                        | FluidVoice has much stronger GitHub traction today.                                                       |

## Architecture comparison

### FluidVoice

FluidVoice is a native macOS Swift app:

- Swift Package Manager project (`Package.swift`)
- macOS 15+ target
- Core dependencies:
  - `altic-dev/FluidAudio` on branch `B/cohere-coreml-asr`
  - `SwiftWhisper`
  - `DynamicNotchKit`
  - `PostHog`
  - `AppUpdater`
- Apple Silicon-first code paths for most advanced ASR models
- Intel fallback primarily through Whisper

Notable technical choices:

- CoreML ASR model downloads from Hugging Face for Nemotron/Cohere/custom FluidAudio model bundles.
- Separate streaming and final ASR managers for Parakeet, including vocabulary boosting on final pass.
- Native Accessibility-based typing and global hotkeys.
- Optional local API server and richer macOS integration surface.

### Dictus Desktop

Dictus Desktop is a cross-platform Tauri app:

- Rust backend + React/TypeScript frontend
- `transcribe-rs` for Whisper/ONNX ASR engines
- `llama-cpp-2` for embedded GGUF local LLM post-processing on current `main`
- Cross-platform shortcuts and text insertion (`rdev`, `handy-keys`, Tauri plugins, platform-specific tools)
- Runs on macOS, Windows, and Linux

Key trade-off: Dictus has more platform complexity, but its local LLM path is more open and portable than FluidVoice's private `Fluid Intelligence` runtime.

## Speech transcription models

### FluidVoice supported ASR models

Based on the README and source inspection (`SettingsStore.SpeechModel`, `ASRService`, provider implementations):

| Model family                               | Implementation                          | Language support           | Size / constraints               | Notes                                                                |
| ------------------------------------------ | --------------------------------------- | -------------------------- | -------------------------------- | -------------------------------------------------------------------- |
| Nemotron Speech 3.5 Ultra Fast Low Latency | CoreML via FluidAudio/Nemotron provider | Around 40 languages        | ~668 MB, Apple Silicon           | Streaming-capable, heavily marketed in 1.6.x.                        |
| Nemotron 3.5 Multilingual                  | CoreML via FluidAudio/Nemotron provider | Around 40 languages        | ~531 MB, Apple Silicon           | Slower/higher-accuracy path.                                         |
| Parakeet Flash / realtime EOU              | FluidAudio `StreamingEouAsrManager`     | English only               | ~428 MB, Apple Silicon           | Low-latency partial transcript + end-of-utterance flow.              |
| Parakeet TDT v3                            | FluidAudio `AsrManager`                 | 25 European languages      | ~461 MB, Apple Silicon           | Default-style fast multilingual model; supports vocabulary boosting. |
| Parakeet TDT v2                            | FluidAudio `AsrManager`                 | English                    | ~443 MB, Apple Silicon           | Optimized English accuracy/speed.                                    |
| Cohere Transcribe                          | external CoreML artifacts               | 14+ languages              | ~1.5 GB, Apple Silicon/macOS 15+ | High-accuracy multilingual option.                                   |
| Apple Speech legacy                        | macOS native                            | System languages           | Built in                         | Zero-download fallback.                                              |
| Apple Speech Analyzer                      | macOS 26+ native                        | EN/ES/FR/DE/IT/JA/KO/PT/ZH | Built in                         | Modern Apple Speech path.                                            |
| Whisper Tiny/Base/Small/Medium/Large       | SwiftWhisper                            | 99 languages               | ~74 MB to ~2.9 GB                | Universal/Intel-compatible fallback.                                 |
| Qwen3 ASR                                  | FluidAudio, currently disabled in UI    | ~30 languages              | ~2 GB                            | Code exists, `qwenPreviewEnabled = false`.                           |

### Dictus Desktop supported ASR models

Dictus Desktop's model manager currently exposes more models than the README summary implies:

| Model family                                  | Implementation                | Language support                                              | Size / constraints | Notes                                                 |
| --------------------------------------------- | ----------------------------- | ------------------------------------------------------------- | ------------------ | ----------------------------------------------------- |
| Whisper Small/Medium/Turbo/Large + Breeze ASR | `transcribe-rs` / whisper.cpp | Whisper 99 languages; Breeze optimized for Taiwanese Mandarin | ~465 MB to ~1.5 GB | GPU acceleration via Metal/Vulkan where available.    |
| Parakeet V2/V3                                | `transcribe-rs` ONNX          | V2 English, V3 25 European languages                          | ~451-456 MB        | Recommended Parakeet V3 path in README.               |
| Moonshine Base / V2 streaming                 | `transcribe-rs` ONNX          | English                                                       | ~31-192 MB         | Good opportunity to surface as “ultra-light English”. |
| SenseVoice                                    | `transcribe-rs` ONNX          | zh/en/yue/ja/ko                                               | ~152 MB            | Strong Asian-language coverage.                       |
| GigaAM v3                                     | `transcribe-rs` ONNX          | Russian                                                       | ~151 MB            | Niche but useful.                                     |
| Canary 180M / 1B                              | `transcribe-rs` ONNX          | 4 languages / 25 European languages                           | ~146 MB / ~691 MB  | Translation support on these models.                  |
| Cohere                                        | `transcribe-rs` ONNX          | 16 listed languages                                           | ~1.7 GB            | High-accuracy option.                                 |
| Custom Whisper `.bin`                         | local discovery               | Depends on model                                              | User-provided      | Nice power-user feature.                              |

### Model takeaway

FluidVoice currently wins the **macOS Apple Silicon model story** because it presents newer CoreML models clearly: Nemotron, Parakeet Flash, Cohere, Apple Speech. Dictus may already have broader ASR model variety, but it is less visible and less productized in the public README.

Action for Dictus: update product/docs/UI messaging so users see the real model breadth without making the app look confusing.

## Post-processing and AI enhancement

### FluidVoice

FluidVoice has three post-processing layers:

1. **Private local runtime: Fluid Intelligence**
   - Marketed as on-device AI enhancement for smart formatting, capitalization, and post-processing.
   - The public app has `PrivateAIIntegrationService`, `PrivateAIProvider`, compile-time feature hooks, and UI/provider plumbing.
   - The actual runtime/provider bridge is private behind `PRIVATE_AI_PROVIDER` / `PrivateAIProviderBridge`.
   - README says it needs ~3.5 GB disk.

2. **Cloud or hosted providers**
   - Built-in provider list includes OpenAI, Anthropic, xAI, Groq, Cerebras, Google, OpenRouter.
   - Default models in `ModelRepository.swift` include:
     - OpenAI: `gpt-4.1`
     - Anthropic: `claude-sonnet-4-20250514`
     - xAI: `grok-3-fast`
     - Groq: `openai/gpt-oss-120b`
     - Cerebras: `gpt-oss-120b`
     - Google: `gemini-2.5-flash`
     - OpenRouter: `openai/gpt-oss-20b`

3. **Local external providers**
   - Ollama (`http://localhost:11434/v1`)
   - LM Studio (`http://localhost:1234/v1`)
   - Custom OpenAI-compatible providers
   - Apple Intelligence path on supported macOS versions.

FluidVoice also supports:

- Prompt profiles for dictate/edit modes
- Per-app prompt bindings
- Reasoning-parameter controls for some models
- Structured request/response handling
- Command Mode and Write/Edit Mode UX

### Dictus Desktop

Dictus Desktop has two relevant layers on current `main`:

1. **External/cloud/local OpenAI-compatible post-processing**
   - Providers include cloud options and local custom endpoints.
   - Post-processing is off by default.
   - Structured JSON output is used where supported.
   - Apple Intelligence has a native local path on macOS Apple Silicon.

2. **Embedded local GGUF runtime**
   - Implemented via `llama-cpp-2`.
   - Catalogue includes:
     - `Qwen2.5 1.5B` (`qwen2.5-1.5b-instruct-q4_k_m.gguf`) — recommended, ~1.1 GB
     - `Gemma 3 4B` (`gemma-3-4b-it-Q4_K_M.gguf`) — multilingual/translation, ~2.5 GB
     - `Phi-4 Mini` (`Phi-4-mini-instruct-Q4_K_M.gguf`) — precise formatting, ~2.5 GB
     - `Llama 3.2 3B` (`Llama-3.2-3B-Instruct-Q4_K_M.gguf`) — balanced generalist, ~2.0 GB
   - Supports custom `.gguf` discovery/import.
   - Metal on macOS, Vulkan on Windows/Linux via target-specific `llama-cpp-2` features.

### Post-processing takeaway

FluidVoice has a stronger **marketed** local enhancement feature, but the key local runtime is private. Dictus has the opportunity to own the phrase:

> Fully open local AI post-processing. No private runtime, no hidden model, no account.

This matters because privacy-first users can audit Dictus more completely.

## Runtime clarification

In this report, “runtime” means the executable inference layer that loads a model, keeps it in memory, prepares prompts/audio tensors, runs inference, and returns output to the app. It is not the model file itself and not the UI.

For ASR, examples of runtimes are SwiftWhisper, FluidAudio/CoreML managers, `transcribe-rs`, whisper.cpp, ONNX Runtime, or Apple Speech APIs. For LLM post-processing, examples are OpenAI-compatible HTTP clients, Apple Intelligence, Ollama/LM Studio, llama.cpp, or FluidVoice's private “Fluid Intelligence” backend.

FluidVoice's public code exposes interfaces for `PrivateAIIntegrationService` / `PrivateAIProvider`, but the actual `PrivateAIProviderBridge` is behind the `PRIVATE_AI_PROVIDER` compile-time flag and is not part of the normal open-source implementation. That is the specific reason this report calls Fluid Intelligence a private runtime.

## Streaming transcription behaviour

FluidVoice is not just “record full audio, then transcribe once.” Source inspection confirms a hybrid streaming design:

- `SettingsStore.enableStreamingPreview` defaults to `true`.
- `ASRService.start()` starts a streaming transcription task when the selected speech model supports streaming.
- Providers expose `transcribeStreaming(_:)` and `transcribeFinal(_:)` separately.
- Parakeet Flash and Nemotron streaming providers append only the new audio delta, call `processBufferedAudio()`, and read `getPartialTranscript()` for live text.
- On stop, FluidVoice still runs a finalization path via `transcribeFinal(_:)`, but for Parakeet TDT with token-timed chunk merge it can reuse a cached live-preview result when coverage is good enough.

Dictus Desktop's current Tauri/Rust path is more batch-oriented: `AudioRecordingManager` records samples, `stop_recording()` returns the full sample buffer, then `TranscriptionManager.transcribe(samples)` runs once on the complete audio before paste/post-processing. Even models named “streaming” in the current `transcribe-rs` path are called through a full-buffer `.transcribe(...)` API rather than a live partial-preview loop.

Implication: FluidVoice gives faster perceived feedback and may avoid some long-buffer failure modes by repeatedly processing shorter increments. This is especially relevant for Parakeet/code-switching issues. It is not guaranteed to fix French + a few English words, but a chunked/streaming path gives us more control: language hints, chunk boundaries, partial confidence, retry/fallback on problematic chunks, and possibly switching to Whisper/Cohere for final repair when Parakeet degrades on long mixed-language audio.

## UX and feature gaps

FluidVoice features worth learning from:

| Feature                    | FluidVoice                                        | Dictus status / opportunity                                                                                 |
| -------------------------- | ------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| Live transcription preview | Strongly marketed; overlay/notch-aware            | Dictus should prioritize visible low-latency feedback, especially for Parakeet/Moonshine paths.             |
| Command Mode               | Voice actions on Mac                              | Dictus should consider a cross-platform “Commands” mode later, but only after core dictation is rock-solid. |
| Write/Edit Mode            | Rewrite selected text / dictate inline            | Dictus Smart Modes are moving this way. Make the UX obvious.                                                |
| Per-app prompt routing     | Implemented                                       | High-value improvement for Dictus Smart Modes: Mail vs IDE vs browser prompts.                              |
| Audio history              | Local optional recordings + ZIP export            | Dictus has history concepts; consider a clear privacy-first local history panel.                            |
| Onboarding                 | Language-first model setup + AI enhancement setup | Dictus should simplify first-run: choose language/use case → recommended ASR → optional Smart Mode.         |
| macOS polish               | DynamicNotchKit, native Swift/AppKit              | Dictus cannot match every native flourish cross-platform, but can improve macOS packaging and overlays.     |
| Homebrew install           | `brew install --cask fluidvoice`                  | Add a Dictus Homebrew cask when releases are stable.                                                        |
| Anonymous analytics        | Enabled by default, opt-out                       | Dictus currently has a privacy advantage: no telemetry. Keep it that way.                                   |

## Privacy and trust comparison

| Area                 | FluidVoice                                                  | Dictus Desktop                                                      |
| -------------------- | ----------------------------------------------------------- | ------------------------------------------------------------------- |
| Audio transcription  | Local by default                                            | Local by default                                                    |
| Cloud AI             | Optional                                                    | Optional/off by default                                             |
| Local AI enhancement | Marketed, but private runtime                               | Embedded GGUF path is open-source                                   |
| Telemetry            | Anonymous analytics default ON, opt-out; PostHog dependency | No telemetry/crash reporter per `docs/PRIVACY.md`                   |
| License              | GPLv3                                                       | MIT                                                                 |
| Model downloads      | Hugging Face + app-managed caches                           | Currently `blob.handy.computer` for ASR; Hugging Face for GGUF LLMs |

Privacy positioning opportunity:

- Avoid copying FluidVoice's private-runtime approach.
- Emphasize that Dictus's local LLM path is inspectable and replaceable.
- Migrate ASR model downloads away from upstream Handy CDN to Dictus-owned or Hugging Face endpoints when feasible.

## Strategic recommendations

### P0 — Messaging and packaging

1. **Update the README model table** to reflect actual Dictus ASR breadth: Whisper, Parakeet, Moonshine, SenseVoice, GigaAM, Canary, Cohere, custom Whisper.
2. **Make embedded GGUF Smart Modes a top-level differentiator**: open local AI, no private runtime.
3. **Add Homebrew cask distribution** for macOS once release signing/notarization is solid.
4. **Keep “no telemetry” visible**. FluidVoice's analytics are privacy-scoped, but default-on telemetry is still a contrast point.

### P1 — Product gaps worth copying conceptually

1. **Per-app prompt routing** for Smart Modes.
2. **Live preview / partial transcription overlay**, especially for streaming-capable models.
3. **Better onboarding**: language/use-case based model recommendation.
4. **Local audio/transcription history** with explicit privacy controls, export, and retention limits.
5. **Rewrite selected text** UX as a first-class mode, not just a shortcut variation.

### P2 — Model roadmap

1. Investigate adding or exposing **Nemotron Speech 3.5** equivalents in Dictus if licensing and runtime portability are acceptable.
2. Benchmark Dictus's existing **Moonshine streaming** and **Canary** models against FluidVoice's Parakeet Flash/Nemotron messaging.
3. Improve model cards: language support, latency, accuracy, download size, hardware fit, and best-use labels.
4. Consider a “fastest English” preset and a “best multilingual” preset to reduce model-choice overload.

### P3 — Competitive caution

1. Do not depend on GPLv3 FluidVoice code unless the licensing implications are explicitly acceptable. Treat this report as product/architecture research, not a code-copy source.
2. FluidVoice's private local AI runtime should not be copied structurally. Dictus's open embedded runtime is a better fit for our values.
3. Avoid macOS-only features becoming blockers for cross-platform quality. Prefer platform-adaptive UI rather than permanent macOS-first architecture.

## Suggested next tickets

1. `docs: refresh Dictus Desktop model table and competitive positioning`
2. `feat: add per-app Smart Mode prompt routing`
3. `feat: add live transcription preview for streaming ASR engines`
4. `feat: add Homebrew cask release path`
5. `chore: migrate ASR model downloads away from blob.handy.computer`
6. `research: benchmark Dictus ASR models against FluidVoice model families`
7. `feat: add local history retention/export UX`

## Bottom line

FluidVoice is ahead in macOS-native polish, public traction, and Apple Silicon ASR model packaging. Dictus Desktop can realistically differentiate by being:

- cross-platform,
- MIT licensed,
- telemetry-free,
- fully open for both local ASR and local LLM post-processing,
- part of the wider Dictus iOS/Android/Desktop ecosystem.

The best response is not to become a FluidVoice clone. The best response is to make Dictus's open cross-platform local-AI story much clearer, then selectively adopt the UX ideas that improve daily dictation speed: live preview, per-app prompts, rewrite/edit modes, and better onboarding.
