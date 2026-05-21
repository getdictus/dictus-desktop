# Dictus Desktop — Privacy & Network Surface

Dictus Desktop is designed around local-first processing. Transcription audio never leaves your device — all speech recognition (Whisper, Parakeet, Moonshine, Breeze, SenseVoice) runs entirely on your machine. Post-processing (LLM reformulation) is OFF by default and requires explicit opt-in; when you choose a local provider (Apple Intelligence or Custom→Ollama/LM Studio), that traffic also stays on-device and never touches the network. The table below is the complete list of outbound HTTP endpoints the app may contact under any configuration. No endpoint is omitted.

---

## Outbound endpoints

| #   | Endpoint                                                                           | When triggered                                                                                                         | Data sent                                                                                                  | How to disable                                                                                  | Default                                         |
| --- | ---------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- | ----------------------------------------------- |
| 1   | `https://github.com/getdictus/dictus-desktop/releases/latest/download/latest.json` | App start (when update checks enabled) + manual "Check for updates" in Settings → Debug                                | HTTP GET; standard User-Agent / Accept headers. No transcribed text, no PII.                               | Settings → Debug → "Check for Updates" toggle (`update_checks_enabled`)                         | **ENABLED**                                     |
| 2   | `https://blob.handy.computer/*` [1]                                                | User clicks "Download" on a model card in Models settings or Onboarding                                                | HTTP GET only; standard headers. No audio, no text.                                                        | Do not download additional models — no download happens without explicit user action            | **OFF** (user-initiated only)                   |
| 3   | `https://github.com/getdictus/dictus-desktop/releases/latest`                      | User clicks "View releases" in the portable-update dialog                                                              | Browser navigation only; no HTTP request originates from the app itself                                    | N/A — manual user action                                                                        | N/A                                             |
| 4   | `https://api.openai.com/v1/{chat/completions,models}`                              | (a) Auto-fetch model list when API key + OpenAI provider selected; (b) User triggers post-process shortcut             | `Authorization: Bearer <api_key>` + transcribed text + selected prompt                                     | Settings → Advanced → disable Post Processing, OR switch to a local provider, OR delete API key | **OFF** (`post_process_enabled=false`)          |
| 5   | `https://api.z.ai/api/paas/v4/{chat/completions,models}`                           | Same as #4, when Z.AI provider selected                                                                                | Same as #4                                                                                                 | Same as #4                                                                                      | **OFF**                                         |
| 6   | `https://openrouter.ai/api/v1/{chat/completions,models}`                           | Same as #4, when OpenRouter provider selected                                                                          | Same as #4. Additionally sends legacy `Referer: https://github.com/cjpais/Handy` header (see Notes below). | Same as #4                                                                                      | **OFF**                                         |
| 7   | `https://api.anthropic.com/v1/{messages,models}`                                   | Same as #4, when Anthropic provider selected                                                                           | `x-api-key: <api_key>` + `anthropic-version: 2023-06-01` + transcribed text + selected prompt              | Same as #4                                                                                      | **OFF**                                         |
| 8   | `https://api.groq.com/openai/v1/{chat/completions,models}`                         | Same as #4, when Groq provider selected                                                                                | Same as #4                                                                                                 | Same as #4                                                                                      | **OFF**                                         |
| 9   | `https://api.cerebras.ai/v1/{chat/completions,models}`                             | Same as #4, when Cerebras provider selected                                                                            | Same as #4                                                                                                 | Same as #4                                                                                      | **OFF**                                         |
| 10  | `http://localhost:11434/v1/{chat/completions,models}`                              | (a) Auto-fetch model list when Custom (local) provider selected; (b) Post-process shortcut; (c) Test connection button | Local loopback only — HTTP request never leaves the device                                                 | N/A — local-only                                                                                | **OFF** (until user selects Custom (local))     |
| 11  | `apple-intelligence://local`                                                       | User selects Apple Intelligence + triggers post-process shortcut                                                       | On-device only — no network traffic                                                                        | N/A — on-device                                                                                 | **OFF** (until user selects Apple Intelligence) |

---

## Notes on legacy upstream artifacts

- **`blob.handy.computer` CDN**: This is upstream Handy's CDN. Dictus does not control this domain today. Migration to a Dictus-owned CDN is tracked as INFR-01 (deferred). All model downloads currently resolve to this domain.
- **OpenRouter `Referer` header**: The value `https://github.com/cjpais/Handy` is the upstream Handy repository URL, sent only on OpenRouter requests via `src-tauri/src/llm_client.rs:70`. This is a legacy artifact from the Handy fork. It will be scrubbed in a follow-up upstream-polish PR; it is not changed in Phase 8.

---

## Headers attached to LLM provider calls

The following headers are attached to all LLM provider requests (`src-tauri/src/llm_client.rs:63-95`):

- `Content-Type: application/json`
- `Referer: https://github.com/cjpais/Handy` — upstream Handy legacy header sent on all provider calls (see Notes above)
- `User-Agent: Dictus/1.0 (+https://github.com/getdictus/dictus-desktop)`
- `X-Title: Dictus`
- `Authorization: Bearer <api_key>` — for all providers except Anthropic
- `x-api-key: <api_key>` + `anthropic-version: 2023-06-01` — Anthropic only (instead of Authorization Bearer)

---

## Endpoints we do NOT contact

- No telemetry or analytics endpoint
- No license validation endpoint
- No crash reporter (no Sentry, no Bugsnag, no equivalent)
- No `getdictus.com` API calls — the About panel opens marketing pages in the browser only; the app itself does not call `getdictus.com`

---

## Appendix — full list of model CDN URLs[1]

The 16 model files currently served from `blob.handy.computer`. These URLs are only contacted when a user explicitly clicks "Download" on a model card.

```
https://blob.handy.computer/ggml-small.bin                        (Whisper Small)
https://blob.handy.computer/whisper-medium-q4_1.bin               (Whisper Medium)
https://blob.handy.computer/ggml-large-v3-turbo.bin               (Whisper Turbo)
https://blob.handy.computer/ggml-large-v3-q5_0.bin                (Whisper Large)
https://blob.handy.computer/breeze-asr-q5_k.bin                   (Breeze ASR)
https://blob.handy.computer/parakeet-v2-int8.tar.gz               (Parakeet V2)
https://blob.handy.computer/parakeet-v3-int8.tar.gz               (Parakeet V3)
https://blob.handy.computer/moonshine-base.tar.gz                 (Moonshine Base)
https://blob.handy.computer/moonshine-tiny-streaming-en.tar.gz    (Moonshine V2 Tiny)
https://blob.handy.computer/moonshine-small-streaming-en.tar.gz   (Moonshine V2 Small)
https://blob.handy.computer/moonshine-medium-streaming-en.tar.gz  (Moonshine V2 Medium)
https://blob.handy.computer/sense-voice-int8.tar.gz               (SenseVoice)
https://blob.handy.computer/giga-am-v3-int8.tar.gz                (GigaAM v3)
https://blob.handy.computer/canary-180m-flash.tar.gz              (Canary 180M Flash)
https://blob.handy.computer/canary-1b-v2.tar.gz                   (Canary 1B v2)
https://blob.handy.computer/cohere-int8.tar.gz                    (Cohere)
```
