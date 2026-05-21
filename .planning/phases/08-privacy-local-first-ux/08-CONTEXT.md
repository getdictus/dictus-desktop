# Phase 8: Privacy / Local-First UX - Context

**Gathered:** 2026-05-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Make Dictus's local-first promise legible in three existing surfaces:

1. The **post-processing settings UI** — reorder providers so local sits clearly above external, with an explicit visual separation.
2. A **`docs/PRIVACY.md`** file documenting every outbound network endpoint, what data leaves the device, and how to disable each.
3. The **onboarding flow** — confirm that the local-first message is already implicit (cloud is not surfaced in onboarding today).

Plus one framing change scoped into this phase: **promote post-processing out of the Experimental group** so the local providers (Apple Intelligence, Custom→Ollama) become discoverable in one click instead of three.

**Requirements covered:** PRIV-01, PRIV-02, PRIV-03.

**Out of scope (explicitly):**
- Adding/removing providers (no new Ollama-as-distinct-entry, no new Gemini, no dropping Z.AI/OpenRouter/Cerebras).
- Embedded local LLM runtime (deferred — own milestone after v1.3).
- Smart Modes / shortcut↔prompt binding / translation prompts (deferred — v1.3).
- Post-process waveform overlay animation (deferred — own todo).
- Full Dictus CDN migration (INFR-01 deferred — Phase 8 only *documents* `blob.handy.computer`).
- In-app Privacy settings page (decided against — `docs/PRIVACY.md` only).

</domain>

<decisions>
## Implementation Decisions

### Promote post-processing out of Experimental (framing)

- Move `PostProcessingToggle` out of the `experimental` group in `src/components/settings/advanced/AdvancedSettings.tsx:60-70` into a regular Advanced group (planner picks: either standalone group "Post-processing" / "Smart text", or fold into an existing group — likely its own `SettingsGroup` since v1.3 Smart Modes will expand it).
- `post_process_enabled` stays **OFF by default** in `default_post_process_enabled()` (`settings.rs`). This phase makes the *toggle* discoverable, not the *feature* default-on.
- The other entries currently in the Experimental group (`KeyboardImplementationSelector`, `AccelerationSelector`, `LazyStreamClose`) stay where they are. Only `PostProcessingToggle` moves.
- Reason: Apple Intelligence is a *local* provider currently buried three clicks deep behind `experimental_enabled`. Promoting the toggle directly serves the local-first legibility goal.

### Provider list contents and ordering (PRIV-01)

- **Keep all 8 current providers as-is.** Do not add Ollama as a distinct provider. Do not add Gemini. Do not drop Z.AI / OpenRouter / Cerebras. The literal text of PRIV-01 ("Ollama, Apple Intelligence, Custom local | OpenAI, Anthropic, Groq, Gemini") was written without checking actual code state and is **explicitly overridden** here.
- **Local group order (top):** Apple Intelligence (macOS ARM64 only), `Custom (local)`.
- **External group order (below):** keep current code order — OpenAI, Z.AI, OpenRouter, Anthropic, Groq, Cerebras. Preserves upstream Handy ordering and avoids gratuitous churn in upstream-sync diffs.
- **Relabel `Custom`** → `Custom (local)`. Single label change across i18n keys. The provider id (`"custom"`) stays unchanged so persisted settings survive.
- **Default provider when post-processing is first enabled:** Apple Intelligence on `cfg(all(target_os = "macos", target_arch = "aarch64"))`, else `Custom (local)`. Today it's hardcoded to `"openai"` in `default_post_process_provider_id()` (`settings.rs:520`) — change that function to be platform-aware. Users on macOS Intel / Windows / Linux without Ollama running will see a connection error when they actually try to use it — that's acceptable because it surfaces the local path instead of the cloud path.

### Custom (local) polish — discoverability for the Ollama path

- **Inline Ollama hint** under the base URL field when `selectedProvider.id === "custom"`: one-line caption with link, e.g. `"Tip: install Ollama (→ ollama.com) and run `ollama serve` to use local models."` Implementation: pure i18n key + small JSX in `PostProcessingSettings.tsx`. Open the link via Tauri opener (existing pattern, see `UpdateChecker.tsx:206`).
- **Test connection button** next to the base URL field for the `Custom (local)` provider. Behavior: pings `base_url/models` (reusing `fetch_post_process_models` infra), reports success ("Connected — N models available") or failure ("Could not reach <base_url>"). Non-blocking, only shown for Custom. Wire as a new small backend command if `fetch_post_process_models` isn't directly reusable, otherwise reuse.
- Both polishes are scoped to the Custom (local) provider only — do **not** add test-connection to external providers in Phase 8 (potential follow-up).

### Section UI — stacked two-block layout (PRIV-01)

- **Drop the single flat Dropdown** for provider selection on the post-processing screen. Replace with a **stacked two-section radio-style picker**, both sections visible at once:

  ```
   ── On your device ──────────────────────────────
     (●) Apple Intelligence
         Free • on-device • macOS Apple Silicon
     ( ) Custom (local)
         Ollama / LM Studio on localhost
         [Tip: install Ollama (→ ollama.com)…]
         [Test connection]   ← only on Custom (local)

   ── External — data leaves this device ──────────
     ( ) OpenAI
     ( ) Z.AI
     ( ) OpenRouter
     ( ) Anthropic
     ( ) Groq
     ( ) Cerebras
  ```

- **Section labels are neutral**, no warning banner, no `Learn more →` link in this UI. The visual separation does the work. ("Trust users, avoid paternalistic tone.")
- **Per-row description text** is in i18n. Apple Intelligence: "Free • on-device • macOS Apple Silicon". Custom (local): "Ollama / LM Studio on localhost". External providers can have empty descriptions or terse one-liners — planner's discretion, must be in i18n.
- **Implementation choice (planner picks):** either (a) extend the existing `Dropdown` component with optional grouped sections + radio-mode rendering, or (b) build a new dedicated `ProviderPicker` component that consumes the same data shape (`providerOptions`) and replaces `<ProviderSelect>` only on this screen. Option (b) is more contained; option (a) is more reusable for future Smart Modes UI. Planner verifies which is idiomatic with the existing settings components.
- **Platform behavior:** on platforms without Apple Intelligence, the "On your device" section shows only `Custom (local)` — no greyed-out Apple Intelligence row, no "coming soon" copy. Honest and uncluttered.
- **i18n:** new keys required for section headers, per-row descriptions, the Ollama tip, and the Test connection button. All 20 locales need entries (or fall back to English per existing i18n pattern). The two section header strings are critical and must be translated.

### PRIVACY.md (PRIV-02)

- **Location:** `docs/PRIVACY.md` — standalone markdown file, sibling to `RUNBOOK-updater-signing.md` and `VERSIONING.md`. **No** in-app Privacy settings section. **No** in-app summary or link from the post-processing footer or onboarding.
- **Shape:**
  - Short prose preamble framing the local-first philosophy (transcription audio never leaves the device; post-processing is opt-in and clearly separated; everything else listed below is the *complete* outbound surface).
  - Per-endpoint table with columns: **Endpoint | When triggered | Data sent | How to disable | Default**.
  - One row per endpoint. Rows to include (exhaustive based on scout):
    1. `https://github.com/getdictus/dictus-desktop/releases/latest/download/latest.json` — updater check via `tauri-plugin-updater`. Triggered: on app start + manual "Check for updates". Data sent: none beyond standard HTTP request. Disable: no in-app toggle today; user can block GitHub at firewall. *(Note: lack of disable toggle is acceptable per current design; flag for future polish.)*
    2. `https://blob.handy.computer/*` — transcription model CDN (Whisper / Parakeet / Moonshine / Breeze / SenseVoice download URLs in `managers/model.rs:130-490`). Triggered: when user downloads a model. Data sent: HTTP GET only. Disable: don't download additional models. **Migration to Dictus-owned CDN is planned (INFR-01, deferred).**
    3. `https://github.com/getdictus/dictus-desktop/releases/latest` — opened in browser when user clicks "View releases" in `UpdateChecker.tsx:206`. Manual action only.
    4. `https://api.openai.com/v1` — OpenAI post-processing. Triggered: only when `post_process_enabled` AND provider=openai AND user invokes post-process shortcut. Data sent: the transcribed text + selected prompt. Disable: turn off post-processing or switch provider.
    5. `https://api.z.ai/api/paas/v4` — Z.AI post-processing. Same trigger/data/disable pattern.
    6. `https://openrouter.ai/api/v1` — OpenRouter post-processing. Same pattern. **Note:** `Referer` header currently set to `https://github.com/cjpais/Handy` (upstream legacy, `llm_client.rs:70`) — document honestly, flag as "will be scrubbed in next upstream sync polish".
    7. `https://api.anthropic.com/v1` — Anthropic post-processing. Same pattern.
    8. `https://api.groq.com/openai/v1` — Groq post-processing. Same pattern.
    9. `https://api.cerebras.ai/v1` — Cerebras post-processing. Same pattern.
    10. `http://localhost:11434/v1` (default Custom base_url) — local Ollama / LM Studio. Local-only, traffic does not leave the device. Listed for completeness; planner may move to a separate "Local-only network calls" mini-section if it clutters the main table.
    11. `apple-intelligence://local` — Apple Intelligence on-device. No network. Same treatment as Ollama row.
- **Tone:** factual, scannable. Audit-friendly. No marketing language.
- **Linking:** add link from About panel ("Privacy & network surface →" near the existing Handy acknowledgment) and from `README.md`. Tauri opener for the About panel link (open in browser to GitHub blob, not in-app webview).
- **Maintenance hook:** add a one-liner to `UPSTREAM.md` noting that any upstream commit adding a new HTTP endpoint requires a PRIVACY.md row.

### Onboarding (PRIV-03)

- **No code change.** The onboarding flow (`src/components/onboarding/Onboarding.tsx`) currently shows only the transcription-model picker — cloud post-processing is not surfaced at all, so local transcription is already the implicit primary path.
- **PRIV-03 is satisfied by existing state.** This is documented in CONTEXT.md so downstream agents (and future audits) understand the requirement was reviewed and met without modification.
- If the planner finds a copy nit during implementation (e.g., the subtitle benefits from one extra word), that's Claude's discretion — but the default position is **no change**.

### Claude's Discretion

- Exact i18n key paths and naming (e.g., `settings.postProcessing.providers.sectionLocal` vs `settings.postProcessing.local.title`) — planner picks consistent with existing keys.
- Whether to extend `Dropdown.tsx` or build a new `ProviderPicker` — based on existing component patterns.
- Whether the Test connection button reuses `fetch_post_process_models` directly or gets a new dedicated command — planner picks lowest-friction path.
- Exact phrasing of section headers and row descriptions in English (translations follow) — must be in i18n, must be factual.
- Exact column structure of the PRIVACY.md table within the agreed columns (Endpoint / When / Data / Disable / Default).
- Where the "Privacy & network surface →" link sits inside the About panel (above/below Handy acknowledgment, with/without icon).
- Where in `AdvancedSettings.tsx` the promoted `PostProcessingToggle` lands (own group vs folded into Output group).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements and scope
- `.planning/REQUIREMENTS.md` §"Privacy / Local-First UX (PRIV)" — PRIV-01, PRIV-02, PRIV-03 acceptance criteria (with the note that the literal provider list in PRIV-01 is overridden by this CONTEXT.md)
- `.planning/REQUIREMENTS.md` §"Out of Scope" — "Per-item modal warning on cloud provider selection" and "Hiding cloud providers entirely" both explicitly out of scope, informs the neutral-label decision
- `.planning/ROADMAP.md` §"Phase 8: Privacy / Local-First UX" — goal + 3 success criteria
- `.planning/PROJECT.md` §"Current Milestone: v1.2 Polish & Local-First UX" — milestone framing, local-first constraint
- `.planning/PROJECT.md` §"Constraints" — "Local-first : cloud providers opt-in only, never prominent (AWS Bedrock excluded from Sync #1)" — anchors the section ordering decision

### Files to modify (grounded paths from scout)
- `src-tauri/src/settings.rs:520` — `default_post_process_provider_id()` becomes platform-aware (Apple Intelligence on macOS ARM64, else `"custom"`)
- `src-tauri/src/settings.rs:524-603` — `default_post_process_providers()`: relabel `"Custom"` → `"Custom (local)"` (or i18n equivalent), keep all 8 entries, keep current order
- `src-tauri/src/llm_client.rs:70` — `Referer` header `"https://github.com/cjpais/Handy"` flagged in PRIVACY.md (Phase 8 does **not** change the header; that's separate upstream polish)
- `src/components/settings/post-processing/PostProcessingSettings.tsx` — replace `<ProviderSelect>` with new stacked sectioned picker; add Ollama tip + Test connection button when Custom (local) is selected
- `src/components/settings/PostProcessingSettingsApi/ProviderSelect.tsx` — either replace or wrap; planner picks
- `src/components/settings/PostProcessingSettingsApi/usePostProcessProviderState.ts` — providerOptions shape may need grouping metadata (local vs external), or planner adds derived grouped data
- `src/components/ui/Dropdown.tsx` — possibly extended with section header support (only if planner picks option a above)
- `src/components/settings/advanced/AdvancedSettings.tsx:60-70` — move `<PostProcessingToggle>` out of the `experimentalEnabled && (<SettingsGroup>...)` block into a regular SettingsGroup
- `src/components/settings/about/` (panel) — add "Privacy & network surface →" link entry
- `src/i18n/locales/{en,es,fr,vi,...}/translation.json` (20 locales) — new keys: section headers, per-row descriptions, Ollama tip, Test connection button, About panel link label, relabeled `Custom (local)` label
- `docs/PRIVACY.md` — new file
- `README.md` — link to `docs/PRIVACY.md`
- `UPSTREAM.md` — one-liner: new outbound HTTP endpoint in upstream sync requires PRIVACY.md row
- `src-tauri/src/managers/model.rs:130-490` — read-only reference for blob.handy.computer URLs to enumerate in PRIVACY.md

### Prior phase context (decisions carried forward)
- `.planning/phases/06-brand-icon-polish/06-CONTEXT.md` §"Specific Ideas" — "Faire au plus simple" principle: when multiple defensive options exist, simplest wins. Applied here in choosing `docs/PRIVACY.md` only (no in-app section) and "no onboarding change".
- `.planning/phases/02-visual-rebrand/02-CONTEXT.md` — i18n discipline: any new user-facing string must have keys in all 20 locales; English is the source.
- `.planning/phases/04-updater-infrastructure/04-CONTEXT.md` — updater endpoint config: `https://github.com/getdictus/dictus-desktop/releases/latest/download/latest.json` is the canonical URL to document in PRIVACY.md.

### External references
- Tauri opener for external links — pattern in `src/components/update-checker/UpdateChecker.tsx:206` (uses `openUrl()` from `@tauri-apps/plugin-opener` or equivalent — planner verifies the actual import).
- Ollama — `https://ollama.com` (linked from the inline tip).

### No external ADRs
This project has no `docs/decisions/` ADR directory. Decisions live in `.planning/` and inline in CONTEXT.md.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `usePostProcessProviderState` hook (`src/components/settings/PostProcessingSettingsApi/usePostProcessProviderState.ts`) — already exposes `providerOptions`, `selectedProviderId`, `handleProviderSelect`. Extend with grouped-options derivation for the new sectioned UI.
- `fetch_post_process_models` Tauri command (`src-tauri/src/shortcut/mod.rs` registered in `lib.rs:437`) — Test connection button can reuse this; success = at least one model returned, failure = surfaced error.
- `commands.checkAppleIntelligenceAvailable` — already used in `usePostProcessProviderState.ts:84` for Apple availability check. No new logic needed for the platform-aware default.
- `SettingsGroup` and `SettingContainer` components — used throughout settings; the promoted `PostProcessingToggle` and the new sectioned picker should follow this pattern.
- `Trans` from `react-i18next` (used in `PostProcessingSettings.tsx:317`) — supports embedded link components for the Ollama tip (`<a>` for the ollama.com link).
- `Alert` component (`src/components/ui/Alert.tsx`) — already used at `PostProcessingSettings.tsx:49` for Apple Intelligence unavailable; reusable for Test connection success/error feedback.

### Established Patterns
- **Tauri commands:** `#[tauri::command]` in `src-tauri/src/commands/*.rs` or `src-tauri/src/shortcut/mod.rs`, registered via `collect_commands!` in `lib.rs::run()`. Auto-generates TypeScript bindings (`src/bindings.ts`).
- **Settings persistence:** `AppSettings` struct in `settings.rs`, `change_*_setting` handlers in `shortcut/mod.rs`. Changing the default provider doesn't require a migration — `serde_default` covers it.
- **i18n:** ESLint enforces `i18next/no-literal-string` — every new string in JSX must come from `t('key.path')`. The Trans component handles inline links.
- **Provider id stability:** `id: "custom"` is the persisted key; relabeling only changes the `label` field (rendered) and the i18n display string. Existing user settings with `post_process_provider_id: "custom"` continue to work.
- **About panel linking:** see Phase 2 acknowledgments work — existing pattern for adding link entries to About.
- **Commit conventions:** `feat:` for the new sectioned UI / promoted toggle, `docs:` for PRIVACY.md, `chore:` for i18n additions.

### Integration Points
- Promoted `PostProcessingToggle` ↔ `post_process_enabled` setting ↔ Sidebar visibility of "Post Process" tab (`Sidebar.tsx:63: enabled: (settings) => settings?.post_process_enabled ?? false`). Promotion only changes *where* the toggle lives in the settings UI; the downstream chain is unchanged.
- New section UI ↔ `usePostProcessProviderState` ↔ `commands.setPostProcessProvider` ↔ `settings.post_process_provider_id`. The radio-row click maps to the same `handleProviderSelect` call as today.
- `Custom (local)` row ↔ Test connection button ↔ `fetch_post_process_models` ↔ HTTP GET to `base_url + models_endpoint`. Network call only fires on user click.
- PRIVACY.md ↔ About panel link + README link — both via `openUrl()` to a GitHub blob URL (not bundled with the app).
- Platform-aware default ↔ `cfg(all(target_os = "macos", target_arch = "aarch64"))` — same gate already used at `settings.rs:580` for Apple Intelligence provider availability.

### Deferred touchpoints (DO NOT modify this phase)
- `src-tauri/src/llm_client.rs:70` `Referer` header — Phase 8 documents it in PRIVACY.md but does **not** change the value. Scrubbing it is a small follow-up.
- `src-tauri/src/managers/model.rs` `blob.handy.computer` URLs — INFR-01 (CDN migration) deferred; Phase 8 documents the URLs as-is.
- `experimental_enabled` setting and its toggle — stays; only the `PostProcessingToggle` moves out of the conditional group.
- `Sidebar.tsx` "Post Process" tab gating logic — unchanged.

</code_context>

<specifics>
## Specific Ideas

- Pierre's UX intuition: "really emphasize the difference between local and non-local — the visual hierarchy is what carries the local-first promise." This anchors the stacked two-section picker over a single dropdown.
- Honest documentation over polish: `blob.handy.computer` (upstream CDN) and the legacy OpenRouter `Referer: github.com/cjpais/Handy` are both real, both currently in the app, and both get documented in PRIVACY.md with their deferred-cleanup status. Transparency beats hiding the gap.
- Don't trust the requirement text literally: PRIV-01's provider list (Ollama, Gemini) doesn't match the code (no Ollama provider entry, no Gemini; has Z.AI/OpenRouter/Cerebras). Code is the source of truth; CONTEXT.md overrides REQUIREMENTS.md on the literal list.
- The post-processing screen is the future surface for v1.3 Smart Modes — the sectioned picker should be designed so it can host mode-specific provider choices later without restructuring.
- Pierre's mental model for local LLMs (Whisper-style downloadable list) is the right direction — just not feasible without a full embedded runtime. That ambition lives in the deferred "Embedded local LLM runtime" milestone.

</specifics>

<deferred>
## Deferred Ideas

**Captured here so they're not lost. Each is a phase-or-milestone-sized lift.**

- **Smart Modes** — Associate each post-process prompt with its own shortcut so users can "switch modes" easily. Includes translation prompts (target language presets). Already scoped as **milestone v1.3 "Smart Mode & Translation"** in PROJECT.md. The Phase 8 sectioned UI should be friendly to this expansion.
- **Post-process waveform overlay animation** — Today, post-processing shows only a small text indicator. Pierre wants a waveform animation during post-processing similar to the recording overlay. New visual capability, own phase. Track as a todo for the v1.3 milestone (or later).
- **Embedded local LLM runtime** — Whisper-style downloadable LLM picker inside Dictus (no separate Ollama process). Requires picking a runtime (llama.cpp / candle / mistral.rs), building a model registry + downloader UI, GPU detection per platform (Metal / Vulkan / CUDA), memory budget management, quantization presets. **Own milestone after v1.3.** Highest-priority deferred idea — directly serves the local-first vision but is not feasible in v1.2.
- **Dictus-owned model CDN** — Replace `blob.handy.computer` (INFR-01, already deferred in PROJECT.md). Requires CDN choice, model re-hosting, signing strategy. Phase 8 only documents the current state.
- **Scrub OpenRouter `Referer: github.com/cjpais/Handy` header** — Tiny one-line fix in `llm_client.rs:70`. Could land as a small follow-up PR; not folded into Phase 8 to keep the phase scoped to UX + docs.
- **In-app updater opt-out toggle** — Documented in PRIVACY.md as "no in-app toggle today; block GitHub at firewall." Adding the toggle is a UX polish, not a Phase 8 deliverable.
- **Test connection on external providers** — Phase 8 ships it only for `Custom (local)`. External providers are a natural follow-up.
- **In-app Privacy settings page** — Considered and rejected for Phase 8 (i18n cost across 20 locales + duplication risk vs `docs/PRIVACY.md`). Revisit if user feedback says the GitHub link is too far away.
- **Onboarding rework** — PRIV-03 satisfied without changes today. A future onboarding revision (e.g., when Smart Modes ships) may want to surface the privacy/local-first story explicitly. Not in Phase 8.

</deferred>

---

*Phase: 08-privacy-local-first-ux*
*Context gathered: 2026-05-21*
