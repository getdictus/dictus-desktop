# Phase 13: Smart Modes UI + Translation Presets + i18n - Context

**Gathered:** 2026-06-03
**Status:** Ready for planning

<domain>
## Phase Boundary

The user-facing surface for Smart Modes. Phase 12 delivered the data model + backend CRUD + per-mode shortcut routing; Phase 13 makes it visible and editable:

- A **visual card-list UI** replacing the v1.2 single-prompt dropdown (MODE-06).
- **Create / edit / delete** Smart Modes from the settings panel without leaving the page (MODE-03 UI half).
- **Inline per-mode shortcut binding** with a **conflict warning** surfaced at bind time (MODE-04 UI + MODE-05).
- **Translation as a first-class preset group** that runs fully offline (TRANS-01, TRANS-02) — including reintroducing a dedicated translation engine path.
- **20-locale propagation** of all new strings; `bun run check:translations` passes 0 errors (L10N-01).

**Backend already done (Phase 12) — do NOT rebuild:** `SmartMode {id, name, kind, prompt, target_language}`, the 10 seeded defaults, CRUD commands (`addSmartMode/updateSmartMode/deleteSmartMode/setActiveSmartMode/listSmartModes/setSmartModeBinding`), `smart_mode_{id}` shortcut registration, and the `BindingResponse {success, binding, error}` conflict result. All exist in `src/bindings.ts`.

**Out of this phase:** new mode capabilities (enable/disable-without-delete, export/import, per-mode provider, app-based auto-activation, gallery — all MODE-Fx, deferred).

</domain>

<decisions>
## Implementation Decisions

### Card list layout & grouping (MODE-06)
- **Vertical list of full-width cards**, reusing the model-library card visual language verbatim (same page, same style as `ModelsSettings.tsx` / `LlmLibrarySection.tsx`).
- **Each card shows (collapsed):** mode name, a **truncated prompt preview** (for Translation modes, show the **target language** instead of a prompt), a **bound-shortcut chip** (or an "unbound" affordance), and a **kind badge** (Rewrite / Translation).
- **Two labeled sections — "Rewrite" then "Translation"** — matching the Phase 12 seed order (Clean Up → rewrites → translations). The two groups must be **visually distinguished by color**, not just a header — a real, immediate visual difference between Rewrite and Translation cards.
- **No "active/default mode" concept in the UI. No primary post-process button.** Every Smart Mode is triggered solely by its **own bound shortcut**; a mode with no shortcut simply sits unused. There is no click-to-activate and no "Set active" control on cards.
  - This is consistent with the Phase 12 migration: the user's v1.2 `transcribe_with_post_process` combo was transferred onto the migrated mode's **own** per-mode shortcut, so it keeps firing and now triggers that specific mode.
  - **Planner reconciliation flag:** Phase 12's `smart_mode_active_id` / `setActiveSmartMode` likely become **vestigial in the UI**. Verify nothing breaks when no mode is "active" (e.g. the post-process pipeline must not assume a default mode). The raw `transcribe` binding (no post-process) stays untouched.

### Create / edit / delete flow (MODE-03 UI)
- **Inline editing** — clicking "edit" **expands the card in place** (name + prompt + target language + shortcut editable inline). Creating opens a fresh expanded card at the top of the relevant section. No modal, no side panel — stay on the page.
- **Two distinct create buttons**, one per section: "+ New Rewrite" in the Rewrite section, "+ New Translation" in the Translation section. **Kind is implicit from which button was used**; the inline form adapts — Rewrite shows a free **prompt** field; Translation shows a **target-language selector** (no prompt field — Phase 12 decision: translation cards carry a language, not a prompt).
- **Delete:** requires a **confirmation dialog**; deleting a mode **cleanly unregisters its global shortcut** (no orphan binding); **default/seeded modes ARE deletable** (it's the user's list).
- **Seeded default modes are fully editable** — name, prompt, and shortcut, exactly like custom modes, with no special lock and no "reset to default" affordance. Honors "Smart Modes = prompts, pas un concept opaque."

### Shortcut binding & conflict (MODE-04 / MODE-05)
- **Shortcut input lives directly on the collapsed card** (a clickable key-combo field always visible per card, for quick rebind). Name/prompt edits still require expanding the card; the shortcut is the one field editable without expanding. Reuse the existing `GlobalShortcutInput` component (adapted to route through `setSmartModeBinding` instead of `updateBinding`).
- **Conflict warning = inline red text directly under the field** (e.g. "This shortcut is already used by: Make Formal"). Surfaced at bind time, contextual, persistent — matches MODE-05 ("inline warning at bind time"), not an ephemeral toast.
- **On conflict, BLOCK the bind** — the shortcut is not registered; the user must pick another or first free the conflicting mode. No silent registration failure, no override path.
- **Conflict scope = ALL global app shortcuts** — checked against other Smart Modes **and** system bindings (raw `transcribe`, `cancel`, etc.). This is what the existing binding system already enforces; prevents a mode from stealing the transcription or cancel hotkey.

### Translation presets & engine (TRANS-01 / TRANS-02)
- **Reintroduce a dedicated translation engine (TranslateGemma) as the proper translation path** via its native chat template (resolves the Phase 11/12 inconsistency — TranslateGemma was dropped from the catalogue in Phase 11 post-UAT). Exact GGUF repo/filename/size/SHA256 confirmed during research (Phase 11 candidate was TranslateGemma-4B Q4_K_M ≈ 2.5GB).
- **Translation modes ship DISABLED by default.** The 4 seeded translation cards (EN/ES/FR/ZH) render **visible but greyed-out** in the Translation section, with an **"Enable translation" CTA** that **explains the choice** before anything downloads — discoverable, never hidden.
- **At first activation, a single GLOBAL engine choice** (not per-mode): the user picks **either**
  1. **Download the dedicated TranslateGemma model** (best translation quality, explains why), **or**
  2. **Use the active post-processing model** (the already-downloaded general-instruct model) via a **hardcoded, non-editable translation prompt** (consistent with "translation cards have no editable prompt"). No extra download.
  - The choice applies to **all** translation modes and is **changeable later in settings**. Both paths run **fully offline through the embedded LLM** (TRANS-02 satisfied either way).
- **Presets kept at EN/ES/FR/ZH** (the Phase 12 seed) — tight starter set; users add more themselves.
- **Custom translation modes: yes** — "+ New Translation" opens a target-language selector. **Language list is gated by the chosen engine:** if TranslateGemma is active, restrict to its officially-supported languages (validatable); if the generic-model path is active, offer a curated list **best-effort** (cannot strictly validate what a general model handles). Exact list/validation = Claude's discretion.

### Localization (L10N-01)
- **Seeded default mode NAMES are translated via i18n at display time** ("Clean Up" → "Nettoyer" in FR, "Translate → French" → localized). The stored name stays stable; only the **displayed** label is localized.
  - **User-created or user-edited modes keep their literal stored name** (not localized) — only the system-seeded set maps to i18n keys.
- **All new user-facing strings** (model-library labels carried from Phase 11, Smart Modes UI, default mode names, translation preset names, the enable-translation CTA/explanation, conflict messages) propagate to **all 20 locales**; `bun run check:translations` must pass with **0 errors**.

### Claude's Discretion
- Exact card color scheme distinguishing Rewrite vs Translation groups.
- Exact TranslateGemma GGUF (repo/filename/size/SHA256) — research-confirmed; mirror the Phase 11 catalogue entry shape.
- The curated language list for custom translation modes and how compatibility is validated/surfaced per engine.
- Hardcoded translation prompt wording for the generic-model fallback path.
- RTL handling for `ar` / `he` locales in the card layout (verify, but standard).
- Empty-state / no-LLM-model behavior reuse from Phase 11's inline empty-state pattern.
- Exact inline-expand animation, prompt-preview truncation length, and shortcut-chip styling.
- How the global translation-engine choice is stored in settings and surfaced for later change.

</decisions>

<specifics>
## Specific Ideas

- **"S'il y a un raccourci alors le mode est actif tout simplement"** — the user explicitly rejected any separate "active mode" abstraction and any primary post-process button. The mental model is flat: bind a shortcut to a mode, press it, that mode runs. This drove the no-default-mode decision.
- **Translation is genuinely special and worth a dedicated model** — the user wants to *impose* TranslateGemma as the quality path, gated behind an explicit, explained download, rather than silently translating with a generic model. The generic-model-via-hardcoded-prompt path is the convenience fallback, offered as a clear choice at activation.
- **Tight default set** — keep EN/ES/FR/ZH; don't pre-load more presets (consistent with the Phase 12 "10 felt like a bit much" curation).
- Working branch is `feat/v1.3-smart-modes`.

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase definition & requirements
- `.planning/ROADMAP.md` §"Phase 13: Smart Modes UI + Translation Presets + i18n" — 5 success criteria (card list, create/edit/delete, shortcut bind + conflict, translation presets offline, 20-locale propagation + E2E flow). Note the §"NOTE on MODE-03" two-phase split (backend in Phase 12, UI here).
- `.planning/REQUIREMENTS.md` — MODE-03/04 (Phase 12 + 13), MODE-05 (conflict warning), MODE-06 (card list), TRANS-01 (multi-target presets, finalized in-phase), TRANS-02 (offline via embedded LLM), L10N-01 (20 locales + `check:translations`). Out of Scope table + Future Requirements MODE-F1..F5 (do NOT build).
- `.planning/PROJECT.md` — "Smart Modes = prompts, pas un nouveau concept opaque"; local-first constraint; cloud opt-in; v1.3 scope.

### Phase 12 carry-forward (data layer is DONE)
- `.planning/phases/12-smart-modes-data-layer/12-CONTEXT.md` — full data-model + migration + CRUD + shortcut-routing decisions. Especially: `kind`-gated `SmartMode`, translation = `target_language` not a prompt, `smart_mode_{id}` scheme, conflict returned (not silently failed), seeded set/order, migrated-combo transfer.
- `src/bindings.ts` — `SmartMode` (~1008), `SmartModeKind` (~1009), `TargetLanguage {code, label}` (~1011), `BindingResponse {success, binding, error}` (~970), and commands `addSmartMode`/`updateSmartMode`/`deleteSmartMode`/`setActiveSmartMode`/`listSmartModes`/`setSmartModeBinding` (~275-315). Auto-generated — do not hand-edit.
- `.planning/STATE.md` §lines 78-79 — TranslateGemma deferred here; needs native chat-template (direct text + target language), generic prompts pass through unchanged on specialized models; catalogue = 4 general-instruct models.

### Phase 11 carry-forward (runtime + library UI)
- `.planning/phases/11-llm-runtime-foundation/11-CONTEXT.md` — embedded provider behavior, no-model inline empty-state pattern, active-model selection, library card conventions, `LOCAL_PROVIDER_IDS_SET` includes `"embedded"`, custom-GGUF import.
- `src-tauri/src/managers/llm.rs` §`catalogue()` (~116-170) — current 4-model catalogue (Qwen2.5 1.5B / Gemma 3 4B / Phi-4 Mini / Llama 3.2 3B); the **TranslateGemma entry is added here** in Phase 13. `LlmModelInfo` shape, download/verify/delete, GGUF validation, load/infer/unload, chat-template application (~884-892 `chat_template` / `apply_chat_template`).

### Frontend (the surfaces this phase builds/replaces)
- `src/components/settings/post-processing/PostProcessingSettings.tsx` — current single-prompt selection surface to replace with the card list; `LOCAL_PROVIDER_IDS_SET`; the page where Smart Modes UI anchors below the model library.
- `src/components/settings/post-processing/LlmLibrarySection.tsx` + `ModelSelect.tsx` + `usePostProcessProviderState.ts` + `types.ts` — model-library card style + provider state to mirror/extend.
- `src/components/settings/GlobalShortcutInput.tsx` — reusable shortcut-capture component (props: `shortcutId`, `descriptionMode`, `grouped`, `disabled`); adapt to route Smart Mode binds through `setSmartModeBinding` and surface `BindingResponse` conflicts inline. `ShortcutInput.tsx` / `HandyKeysShortcutInput.tsx` are related.
- `src/lib/utils/keyboard.ts` — `getKeyName` / `formatKeyCombination` / `normalizeKey` used by shortcut inputs.

### i18n
- `src/i18n/locales/en/translation.json` — source strings; new keys for Smart Modes UI, default mode names, translation presets, enable-translation CTA, conflict messages added here.
- `src/i18n/locales/{ar,bg,cs,de,en,es,fr,he,it,ja,ko,pl,pt,ru,sv,tr,uk,vi,zh,zh-TW}/translation.json` — **20 locales** total (note `ar`/`he` are RTL). All new keys propagated to every locale.
- `scripts/check-translations.ts` — run via `bun run check:translations`; must exit 0 (L10N-01 gate).

### Backend routing (read-only context — done in Phase 12, may need verification)
- `src-tauri/src/actions.rs` — `post_process_transcription()` pipeline + `kind` branch (Rewrite prompt path vs Translation routed); `ACTION_MAP`; `smart_mode_{id}` routing. The **Translation execution path is wired here in Phase 13** (TranslateGemma native chat template OR generic-model hardcoded prompt, per the global engine choice).
- `src-tauri/src/shortcut/mod.rs` — `setSmartModeBinding` / per-mode registration; `change_binding` template; conflict detection source.
- `src-tauri/src/settings.rs` — `SmartMode` type, seeded defaults, `smart_modes` / `smart_mode_active_id`; the global translation-engine choice is stored here.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`GlobalShortcutInput.tsx`** — drop-in shortcut-capture UI; the per-card bind reuses it, swapping `updateBinding` for `setSmartModeBinding` and rendering the `BindingResponse` conflict inline.
- **Model-library card style (`LlmLibrarySection.tsx` / `ModelsSettings.tsx`)** — the verbatim visual language for Smart Mode cards (full-width, badges, action icons, active/state indicators).
- **Phase 11 inline empty-state pattern** — reused for the "Enable translation" CTA and any no-model state.
- **`llm.rs catalogue()` + download/verify/load machinery** — TranslateGemma plugs in as one more catalogue entry; the chat-template path (`apply_chat_template`) already exists for the native-template translation call.
- **All Phase 12 CRUD/binding commands are in `bindings.ts`** — the frontend imports them directly; no new backend commands needed for CRUD/bind (translation engine wiring is the backend addition).

### Established Patterns
- **i18next, no hardcoded JSX strings** (ESLint-enforced) — every new label is a key in `en/translation.json` propagated to 20 locales.
- **tauri-plugin-store settings + auto-merged defaults** — the global translation-engine choice persists here.
- **Flat `smart_mode_{id}` binding scheme** alongside `transcribe`/`cancel`; conflict-checked against the full binding set.
- **`$APP_DATA/models/` is update-safe** — the downloaded TranslateGemma GGUF persists across app updates (same as other models).

### Integration Points
- `PostProcessingSettings.tsx` (card-list mounts below the model library) ↔ Phase 12 CRUD commands in `bindings.ts` ↔ `GlobalShortcutInput` (adapted) ↔ `setSmartModeBinding` / `BindingResponse`.
- `actions.rs` `post_process_transcription()` Translation branch ↔ `llm.rs` (TranslateGemma native chat template OR active-model hardcoded prompt) — the global engine choice in `settings.rs` selects the path.
- `llm.rs catalogue()` ↔ download pipeline ↔ Translation section CTA (download-on-activate).
- New i18n keys ↔ 20 locale files ↔ `check-translations.ts` gate.

</code_context>

<deferred>
## Deferred Ideas

- **Extended translation presets** (DE/IT/PT/JA…) beyond EN/ES/FR/ZH — users create them; not pre-seeded.
- **Per-mode translation engine choice** — rejected for v1.3 (global choice only); doubles UI/state.
- **Per-mode provider selection** (MODE-F3), **enable/disable without delete** (MODE-F1), **export/import modes** (MODE-F2), **app-based auto-activation** (MODE-F4), **mode gallery** (MODE-F5) — Future Requirements, not v1.3.
- **"Reset seeded mode to default" affordance** — considered, dropped (would require storing original prompts; seeded modes are just freely-editable modes).
- **Write as SMS** default mode — cut in Phase 12 (overlaps Make Casual).

</deferred>

---

*Phase: 13-smart-modes-ui-translation-presets-i18n*
*Context gathered: 2026-06-03*
