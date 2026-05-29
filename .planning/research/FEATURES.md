# Feature Research

**Domain:** Desktop speech-to-text with embedded local LLM post-processing and multi-mode shortcut system
**Researched:** 2026-05-29
**Confidence:** MEDIUM-HIGH (competitive landscape verified via live sources; Rust runtime specifics from official repos + crates.io; UX patterns from live product docs)

---

## Context: What Already Exists in Dictus

Understanding what is ALREADY shipped is critical to avoid re-researching or re-scoping:

- **Single prompt, single shortcut**: `post_process_prompts` is a list of `LLMPrompt {id, name, prompt}`. The user creates/edits/deletes prompts from a dropdown + textarea UI. ONE prompt is "selected" at a time and bound globally to `transcribe_with_post_process` shortcut.
- **No per-prompt shortcut binding**: the shortcut fires the currently-selected prompt. There is no way to bind a specific prompt to its own key.
- **Providers**: Apple Intelligence (macOS arm64), Custom/Ollama (external process), and cloud providers. No embedded runtime.
- **Model library placeholder**: A dashed "coming soon" card titled "Bibliothèque de modèles locaux" sits at the top of post-processing settings. It exists in the UI but links to nothing.
- **`ModelUnloadTimeout`**: Already modeled in `settings.rs` (Never / Immediately / Min2 / Min5 / Min10 / Min15 / Hour1 / Sec15-debug). The Whisper model-unload system is already working — the same mental model can be reused.

v1.3 changes the **shape** of the existing system: prompts become "Smart Modes", the shortcut model becomes per-mode, the placeholder becomes a real model downloader, and the embedded runtime is added as a new local provider type.

---

## Feature Landscape

### 1. Embedded Local LLM Runtime — Table Stakes vs Differentiators

#### Table Stakes (Users Expect These)

| Feature | Why Expected | Complexity | Existing Dictus Hook |
|---------|--------------|------------|----------------------|
| In-app model downloader with progress bar | Whisper picker set this expectation; users want one download flow for everything | MEDIUM | `managers/model.rs` + model picker UI — mirror exact same pattern |
| Show disk size before download | Jan/LM Studio both show size upfront; without it users get nasty surprises on small SSDs | LOW | Add to model metadata struct |
| Recommended model card with "Start here" badge | GPT4All, Jan, LM Studio all surface one default recommendation; new users need a guided path | LOW | `RECOMMENDED_PROVIDER_ID` pattern already exists in provider picker |
| Download cancellation | Jan allows cancel mid-download; omitting this makes large model downloads frustrating | MEDIUM | HTTP streaming + channel abort signal |
| Show installed vs available distinction | Users need to know what is already downloaded | LOW | Local filesystem scan, same as Whisper models |
| Model delete / free disk space | Jan has "Delete All" button with total space freed; individual delete required | LOW | `fs::remove_file` + recalculate freed space |
| Model appears in provider picker after download | The downloaded model must become selectable as the local provider immediately | MEDIUM | Integrate with existing `PostProcessProvider` list |

#### Differentiators (Competitive Advantage)

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Hardware fit badge (Fits / May be slow / Won't fit) | Jan pioneered this: infers fit from model size vs available RAM/VRAM at display time, no download needed. Critical on 8 GB machines. | MEDIUM | Use `sysinfo` crate for total RAM; compare to model metadata's RAM requirement field |
| Quantization tier selector (Small / Balanced / Large) | Jan groups Q4/Q5/Q8 into human labels; avoids exposing GGUF suffixes to non-expert users | LOW | Map Q4_K_M → "Small (faster)", Q5_K_M → "Balanced", Q8_0 → "Large (better quality)" |
| GPU/CPU status badge | Users on Metal/Vulkan GPU should see "GPU accelerated" vs "CPU only" so they understand performance expectations | LOW | Detect Metal (macOS) / Vulkan (Win/Linux) via existing platform detection logic |
| Auto-select quantization based on available RAM | LM Studio recommends a variant based on detected RAM. Removes cognitive load entirely. | MEDIUM | If RAM < 6 GB → recommend Q4_K_M; 6–12 GB → Q5_K_M; >12 GB → Q8_0 |
| Idle unload timeout (reuse existing `ModelUnloadTimeout`) | Already modeled in settings.rs for Whisper. Whisper unloads after N minutes of inactivity; embedded LLM should do the same to free memory between dictation sessions. | LOW | `ModelUnloadTimeout` enum already exists — bind embedded runtime to same setting |

#### Anti-Features (Avoid)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Hugging Face browser integration | LM Studio embeds HF search; users want "more models" | Adds dependency on external registry, breaks offline-first UX, exposes users to non-curated models | Curate a fixed catalog of 4-6 well-tested models with download URLs. Start closed, open later. |
| Per-model GPU layer configuration (n_gpu_layers slider) | Power users want tuning | Configuration surface explosion; most users just want it to work | Auto-detect layers from available VRAM; expose an "Advanced" toggle for power users in v2 |
| Running as separate llama-server process | Mirrors Ollama architecture | Contradicts the "no external process" pitch; adds port conflicts, process lifecycle complexity | Embed llama.cpp via Rust bindings (`llama-cpp-2` crate) in-process |
| Model versioning / update notifications | Nice in a full model manager | Adds version-tracking infrastructure not needed for a fixed curated catalog | Pin model SHA in catalog; users re-download if a new recommended version ships |

#### Minimum Viable Downloader UX (concrete)

Mirrors the existing Whisper model picker:

1. **Header card**: model name + recommended badge + description (1 line)
2. **Fit badge**: "Fits in your memory" (green) / "May be slow" (yellow) / "Insufficient memory" (red) — calculated once from total system RAM vs model RAM requirement
3. **Quantization row**: radio buttons labeled "Small (faster, Q4_K_M, ~2.5 GB)" / "Balanced (Q5_K_M, ~3.2 GB)" / "Quality (Q8_0, ~4.5 GB)" — recommended one pre-selected based on RAM
4. **Download button** → progress bar (percent + MB/s + cancel button)
5. **Delete button** (only shown when model is installed) — shows freed disk space
6. **After download**: model appears as selectable option in the local provider tab, taking precedence as new platform-default

Recommended launch models (MEDIUM confidence — validate sizes at implementation time):
- **gemma-3-4b-it Q4_K_M** (~2.5 GB) — strong multilingual, good instruction following, fits 6 GB RAM
- **Qwen3-4B Q4_K_M** (~2.5 GB) — best multilingual of the small tier, ideal for translation modes
- These are alternatives, not both required at launch. One recommended model per size class is enough.

---

### 2. Smart Modes (per-prompt shortcut binding) — Table Stakes vs Differentiators

#### Table Stakes (Users Expect These)

| Feature | Why Expected | Complexity | Existing Dictus Hook |
|---------|--------------|------------|----------------------|
| Multiple named modes (prompts with names) | Superwhisper's mode concept is the competitive baseline; Dictus already has named `LLMPrompt` objects — the data model exists | LOW | `post_process_prompts: Vec<LLMPrompt>` already in settings |
| Edit mode name and prompt text | Already works in current UI | DONE | `PostProcessingSettingsPrompts` component |
| Create new mode | Already works | DONE | `commands.addPostProcessPrompt()` |
| Delete mode | Already works (disabled if only 1 prompt) | DONE | `commands.deletePostProcessPrompt()` |
| Per-mode shortcut binding | This is the core missing link. Superwhisper added this in v2.12.0 (April 2026). Users expect: record → choose the right transformation → paste. Without per-mode shortcuts the user must open settings to switch modes. | HIGH | `ShortcutBinding {id, name, description, default_binding, current_binding}` struct exists. Need to extend `LLMPrompt` with an optional `shortcut` field and register/deregister global shortcuts dynamically. |
| Visual mode list (not just a dropdown) | Superwhisper uses a sidebar list for mode management; a dropdown doesn't communicate "mode library" well | MEDIUM | Replace current Dropdown with a card/list component showing all modes |
| Mode reordering | Superwhisper added drag-drop ordering (v1.46.1); users put their most-used modes first | LOW | Drag-and-drop list; persist order in settings |

#### Differentiators (Competitive Advantage)

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Shipped default mode set ("well-crafted prompts") | Superwhisper ships with Message / Email / Note / Super / Meeting defaults that "just work". Dictus ships 0 prompts today — blank slate is intimidating. | LOW (authoring) | See "Recommended Default Prompt Set" section below |
| Mode enable/disable quickly (toggle per row) | Disable a mode without deleting it | LOW | Add `enabled: bool` to `LLMPrompt` |
| Translation modes grouped as a sub-category | Elevates translation as a first-class concept rather than "one of many prompts" | LOW (UI label) | Add optional `category` field to `LLMPrompt`; group in list view |
| Shortcut conflict detection with inline warning | As of 2026, most dictation apps (including Superwhisper early releases) had silent failures when shortcuts conflicted. Proactive inline warning before save is a clear differentiator. | MEDIUM | Check proposed binding against existing `shortcut_bindings` list on keystroke; show inline error if collision detected |

#### Anti-Features (Avoid)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Per-mode provider selection (different LLM per mode) | Superwhisper supports model per mode | Doubles the configuration surface; confusing with embedded + Ollama + cloud all in play | Modes use the globally selected provider. Provider selection remains global. Revisit in v2. |
| Auto-activation by active app | Superwhisper's per-app auto-switch reduces manual mode selection | High complexity: requires accessibility permissions on macOS to read active app, additional on Windows | Use explicit shortcuts instead; revisit in v2 |
| Mode templates / community sharing | GPT4All and Superwhisper have template galleries | Requires moderation, hosting, authentication | Ship a curated built-in set; let users export/import JSON in v2 |

#### Minimum Viable Smart Modes Editor UX (concrete)

Replaces the current "Prompts" section in Post-Processing Settings:

1. **Mode list panel** (left/top): all modes displayed as named cards — each shows name + first 40 chars of prompt text + assigned shortcut (or "No shortcut" in muted text). "New Mode" button at top.
2. **Mode detail panel** (right/bottom): when a mode is selected, shows:
   - Name field (text input)
   - Prompt textarea (same as today)
   - Shortcut field (`ShortcutInput` component, same as existing shortcut UI) — shows inline warning if binding conflicts with an existing shortcut
   - Save / Delete buttons
3. **Shortcut conflict logic**: on keystroke in ShortcutInput, check against all existing bindings (global transcription shortcuts + other mode shortcuts). Show inline red warning: "This shortcut is already used by [Mode Name]."
4. **Delete guard**: prevent deletion of last mode (existing behavior, keep).

---

### 3. Translation as a First-Class Mode

#### Table Stakes (Users Expect These)

| Feature | Why Expected | Complexity | Existing Dictus Hook |
|---------|--------------|------------|----------------------|
| Target language selection per translation mode | Without target language, "translate" is ambiguous. Wispr Flow's Command Mode requires the user to say "translate to Polish" verbally — for a preset shortcut approach, the target must be in the prompt | LOW | Encode target language directly in the prompt text: "Translate to Spanish:" |
| Source language auto-detection | Whisper already detects source language during transcription; LLMs handle any source natively. Users should not need to specify source. | NONE (already done by Whisper + LLM) | Whisper transcribes detected language; the LLM prompt just says "translate to [target]" — source is implicit |
| Multiple translation presets (EN / ES / FR / ZH / DE as minimum) | Users who dictate in one language and type in another need per-language shortcuts | LOW (prompt authoring) | Each translation target = one Smart Mode with a binding |
| Translation modes visually grouped | Superwhisper translation is only "to English" via the transcription layer. Multi-target translation as named modes is new; visual grouping aids discoverability | LOW | `category: "translation"` label on mode cards |

#### Differentiators (Competitive Advantage)

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Dedicated language selector field on translation modes | Instead of editing raw prompt text to change the target language, a dropdown or picker specifically for target language selection | LOW | Detect if mode has `category: "translation"` and show a language picker that auto-generates the prompt |
| Quality calibration note for small models | Qwen3-4B and Gemma3-4B handle common language pairs (EN/ES/FR/ZH/PT/DE/IT) well at Q4 but degrade on less-resourced languages. Surfacing this in the UI ("Best for major languages") manages expectations | LOW | Tooltip or inline note on translation modes when embedded runtime is selected |

#### Anti-Features (Avoid)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Auto-route translation to cloud for quality | Users who can afford it might want best quality | Contradicts local-first philosophy; silent cloud routing is exactly what Dictus avoids | Show quality caveat inline. Let users opt in to cloud provider manually. |
| Whisper translate mode (speech → English only) | Whisper has a built-in translate task that outputs English | Output-only-English is a hard constraint of Whisper's translate task; does not support multi-target | Use LLM post-processing for translation; Whisper transcribes in source language |

#### Recommended Default Translation Preset Languages

EN, ES, FR, ZH (Simplified) — these 4 map directly to the Dictus i18n locales already validated and match the most likely tech-adjacent multilingual user base. Ship these 4 at launch. German, Portuguese, Japanese as easy user-created additions.

---

### 4. Recommended Default Prompt Set ("Well-Crafted Prompts")

These should ship pre-loaded when the user first opens Smart Modes, with no prompts in the list. Each is actionable, model-agnostic, and works with both embedded small LLMs and cloud providers.

| Mode Name | Category | Prompt Intent | Complexity |
|-----------|----------|---------------|------------|
| Clean Up | general | Remove filler words ("um", "uh", false starts), fix punctuation and capitalization. Keep original meaning and length. | LOW |
| Make Formal | tone | Rewrite the text with professional, formal language suitable for business communication. | LOW |
| Make Casual | tone | Rewrite the text in a relaxed, conversational tone. | LOW |
| Write as Email | formatting | Format as a professional email: subject line suggestion, greeting, body, closing. | LOW |
| Write as SMS | formatting | Condense to a short text message. Remove salutations. Max 2-3 sentences. | LOW |
| Bullet Points | formatting | Convert the spoken content into a concise bullet-point list. | LOW |
| Summarize | general | Summarize the key points in 2-3 sentences. | LOW |
| Translate to English | translation | Translate the following text to English. Preserve tone and meaning. | LOW |
| Translate to Spanish | translation | Translate the following text to Spanish. Preserve tone and meaning. | LOW |
| Translate to French | translation | Translate the following text to French. Preserve tone and meaning. | LOW |

Notes on the default set:
- 10 modes at first launch is the upper bound — Superwhisper ships 6 built-in modes; more than 10 overwhelms.
- "Clean Up" should be the first mode and the recommended default for `transcribe_with_post_process` — it is the least surprising behavior for a new user.
- All prompts are designed to be model-agnostic: they work identically with Gemma 3 4B local, Ollama, Apple Intelligence, or GPT-4o.
- Translation modes: only show all 3 translation defaults if the user's detected system locale suggests multilingual use; otherwise surface just "Translate to English" and let users create others.

---

## Feature Dependencies

```
[Per-mode shortcut binding]
    └──requires──> [Dynamic shortcut registration in Rust shortcut manager]
                       └──requires──> [LLMPrompt extended with optional shortcut field]
                                          └──requires──> [Settings schema migration with serde default]

[Embedded LLM runtime]
    └──requires──> [llama.cpp or candle Rust crate integrated in Cargo.toml]
    └──requires──> [New manager: EmbeddedLlmManager mirroring model.rs pattern]
    └──requires──> [Model downloader (HTTP stream + progress events to frontend)]
    └──requires──> [New PostProcessProvider variant "embedded" in provider list]

[Hardware fit badge]
    └──requires──> [sysinfo crate for total RAM detection]
    └──requires──> [Model catalog with RAM requirement metadata]

[Translation modes as first-class]
    └──requires──> [Smart Modes (multiple modes + per-mode shortcut binding)]
    └──enhances──> [Embedded LLM runtime (local translation without cloud)]

[Model downloader UX]
    └──mirrors──> [Existing managers/model.rs + model picker UI pattern]
    └──reuses──> [ModelUnloadTimeout enum in settings.rs]
```

### Dependency Notes

- **Per-mode shortcuts require Rust-side dynamic registration**: The existing `shortcut_bindings` are registered at startup. For per-mode shortcuts, bindings must be registered/deregistered when modes are created/deleted/edited. This is the highest-complexity new piece — it touches `shortcut/handler.rs` and `shortcut/tauri_impl.rs`.
- **Embedded runtime is independent of Smart Modes**: The runtime can be built and integrated before the full Smart Modes UI is complete; modes use whichever provider is currently selected.
- **Translation modes depend on Smart Modes, not on the embedded runtime**: Translation prompts work with any provider (Apple Intelligence, Ollama, cloud). The embedded runtime just enables a fully local path.
- **`ModelUnloadTimeout` enum reuse**: Already exists for Whisper models. Embedded LLM should respect the same setting, or have its own with the same variants. No new settings concept needed.

---

## MVP Definition

### Launch With (v1.3)

- [ ] Embedded local LLM runtime (llama.cpp via `llama-cpp-2` crate) with Metal + Vulkan GPU support
- [ ] Model downloader with progress bar, hardware fit badge, quantization tier selector (Small / Balanced / Quality), cancel and delete
- [ ] 1-2 curated recommended models in the catalog (gemma3-4b or qwen3-4b Q4_K_M and Q5_K_M)
- [ ] Per-mode shortcut binding (extend `LLMPrompt` + dynamic shortcut registration)
- [ ] Visual mode list (cards, not dropdown) with shortcut assignment and inline conflict detection
- [ ] 7-10 pre-loaded default Smart Modes including translation presets
- [ ] Translation modes grouped and labeled visually

### Add After Validation (v1.x)

- [ ] Auto-quantization recommendation based on detected RAM (requires `sysinfo` integration)
- [ ] Mode enable/disable toggle without deletion
- [ ] Export/import modes as JSON
- [ ] `category` field for grouping translation vs general modes

### Future Consideration (v2+)

- [ ] Per-mode provider selection (different LLM per mode)
- [ ] Auto-activation by active app (accessibility permissions required)
- [ ] Community mode gallery / templates
- [ ] Open model catalog (HuggingFace integration)
- [ ] Advanced GPU layer configuration

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Per-mode shortcut binding | HIGH | HIGH | P1 |
| Embedded LLM runtime (basic, no GPU) | HIGH | HIGH | P1 |
| Default Smart Mode prompt set (10 modes) | HIGH | LOW | P1 |
| Model downloader UX (progress, cancel, delete) | HIGH | MEDIUM | P1 |
| Hardware fit badge | MEDIUM | MEDIUM | P1 |
| Visual mode list (cards not dropdown) | HIGH | MEDIUM | P1 |
| Metal / Vulkan GPU acceleration | MEDIUM | HIGH | P1 |
| Translation modes grouped in UI | MEDIUM | LOW | P1 |
| Quantization tier labels (Small / Balanced / Quality) | MEDIUM | LOW | P2 |
| Shortcut conflict detection inline | MEDIUM | MEDIUM | P2 |
| Auto-quantization recommendation based on RAM | MEDIUM | MEDIUM | P2 |
| Mode reordering (drag-drop) | LOW | LOW | P2 |
| Mode enable/disable toggle | LOW | LOW | P3 |
| Export/import modes as JSON | LOW | LOW | P3 |

---

## Competitor Feature Analysis

| Feature | Superwhisper | Wispr Flow | Jan (local LLM reference) | Dictus v1.3 Approach |
|---------|-------------|------------|--------------------------|----------------------|
| Multiple modes / prompts | Yes — 6 built-in + unlimited custom, since v1.19 | Command Mode only (ad-hoc, not persistent presets) | N/A (chat app) | Named Smart Modes stored as LLMPrompt list — extends existing model |
| Per-mode keyboard shortcut | Yes, since v2.12.0 (April 2026) | Up to 4 shortcuts (global, not per-mode) | N/A | Per-mode optional shortcut field + dynamic registration |
| Shortcut conflict handling | Fixed in v2.13.1 (was silent failure before) | Silent failure if conflict | N/A | Inline warning on assignment — proactive, not reactive |
| Embedded LLM runtime | No (local = Ollama external, cloud or Apple) | No (cloud-first) | Yes — full embedded llama.cpp | Embedded llama.cpp via `llama-cpp-2` crate, in-process |
| Model downloader | No (Ollama handles separately) | No | Yes: full hub UX, fit badges, cancellation | Full downloader mirroring Whisper picker + fit badge |
| Translation modes | To English only (Whisper-level); multi-target possible via Custom Mode only | Ad-hoc via Command Mode ("translate to X" verbally) | N/A | Named presets, bindable to shortcuts — multi-target first-class |
| Default prompt set | Message, Email, Note, Super, Meeting, Custom | Not applicable | N/A | 10 pre-loaded modes: Clean Up, tones, formatting, summarize + translations |
| Hardware fit detection | No | No | Fits / May be slow / Won't fit color pill | Color badge based on system RAM vs model requirement |

---

## Sources

- Superwhisper changelog and modes documentation: https://superwhisper.com/changelog, https://superwhisper.com/docs/modes/modes, https://superwhisper.com/docs/modes/custom
- Wispr Flow Command Mode: https://docs.wisprflow.ai/articles/4816967992-how-to-use-command-mode
- Jan AI model hub UX: https://www.jan.ai/docs/desktop/manage-models
- LM Studio 2026 guide: https://codersera.com/blog/lm-studio-complete-guide-2026/
- Superwhisper 2026 review: https://spokenly.app/blog/superwhisper-review
- LLM comparison tools 2026: https://codersera.com/blog/ollama-vs-lm-studio-vs-vllm-vs-llama-cpp-vs-mlx-2026/
- Voice-to-text post-processing prompts (200+ library): https://dev.to/danielrosehill/200-custom-system-prompts-for-voice-to-text-post-processing-2od9
- llama.cpp model management: https://huggingface.co/blog/ggml-org/model-management-in-llamacpp
- llama.cpp idle unload feature request: https://github.com/ggml-org/llama.cpp/issues/18189
- Rust LLM ecosystem (candle, llama-cpp-2): https://hackmd.io/@Hamze/Hy5LiRV1gg, https://crates.io/crates/llama-cpp-2
- Whisper translate limitation (English output only): https://openai.com/index/whisper/
- Small LLM translation quality 2026: https://www.noviai.ai/models-prompts/best-llm-for-translation/
- Best small models 2026 (Qwen3, Gemma3): https://localaimaster.com/blog/small-language-models-guide-2026

---
*Feature research for: Dictus Desktop v1.3 Smart Modes & Local LLM*
*Researched: 2026-05-29*
