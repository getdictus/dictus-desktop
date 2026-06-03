# Phase 13: Smart Modes UI + Translation Presets + i18n — Research

**Researched:** 2026-06-03
**Domain:** React/TypeScript frontend, Tauri commands, i18n propagation, Rust backend extension
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **Card list layout:** Vertical list of full-width cards, reusing model-library card visual language verbatim (`ModelsSettings.tsx` / `LlmLibrarySection.tsx`).
- **Each card (collapsed):** mode name, truncated prompt preview (Translation cards show target language instead), bound-shortcut chip, kind badge (Rewrite/Translation).
- **Two labeled sections:** "Rewrite" then "Translation", visually distinguished by color (amber header when Translation is pre-enabled, not just a header label).
- **No "active/default mode" concept in the UI. No primary post-process button.** Every Smart Mode is triggered solely by its own bound shortcut.
- **Inline editing:** clicking edit expands the card in place (no modal, no side panel). Creating opens a fresh expanded card at the bottom of the relevant section.
- **Two distinct create buttons**, one per section: "+ New Rewrite" / "+ New Translation". Kind is implicit; form adapts accordingly.
- **Delete:** requires confirmation dialog (Tauri `ask()`); deleting cleanly unregisters the shortcut; seeded modes ARE deletable; no "reset to default" affordance.
- **Shortcut input lives on the collapsed card** — reuse existing `GlobalShortcutInput` adapted to route through `setSmartModeBinding` / `BindingResponse`.
- **Conflict warning = inline red text directly under the field.** On conflict, BLOCK the bind. Conflict scope = ALL global app shortcuts.
- **Reintroduce TranslateGemma** as a dedicated translation engine path via its native chat template.
- **Translation modes ship DISABLED by default.** The 4 seeded cards render visible but greyed-out; an "Enable translation" CTA explains the choice before anything downloads.
- **At first activation, a single GLOBAL engine choice:** Download TranslateGemma OR use active post-processing model with hardcoded prompt. Choice is changeable later.
- **Presets: EN/ES/FR/ZH only** (the Phase 12 seeds). Users add more.
- **Custom translation modes: yes** — language list gated by chosen engine.
- **Seeded default mode NAMES translated via i18n at display time.** Stored name stays stable (English constant). User-created/edited modes use their literal stored name.
- **All new user-facing strings propagated to 20 locales.** `bun run check:translations` must pass with 0 errors.

### Claude's Discretion

- Exact card color scheme (resolved in UI-SPEC: Rewrite = neutral border; Translation disabled = `opacity-60 bg-mid-gray/5`; Translation section header = `text-amber-500` when pre-enabled).
- Exact TranslateGemma GGUF (repo/filename/size/SHA256) — research confirmed: `bullerwins/translategemma-4b-it-GGUF`, filename `translategemma-4b-it-Q4_K_M.gguf`, ~2490 MB; SHA256 must be computed at download time.
- Curated language list for custom translation modes and compatibility surfacing per engine.
- Hardcoded translation prompt for generic-model path (resolved in UI-SPEC: `"Translate the following text to {target_language}. Output only the translation, no explanation or commentary."`).
- RTL handling for `ar`/`he` (resolved in UI-SPEC: standard Tailwind flex reverses naturally; shortcut chip wraps with `dir="ltr"`).
- Empty-state / no-LLM-model behavior: reuse Phase 11 inline empty-state pattern.
- Inline-expand animation, prompt-preview truncation length (80 chars), shortcut-chip styling.
- Global translation-engine choice stored in settings and surfaced for later change.

### Deferred Ideas (OUT OF SCOPE)

- Extended translation presets beyond EN/ES/FR/ZH.
- Per-mode translation engine choice.
- Per-mode provider selection (MODE-F3), enable/disable without delete (MODE-F1), export/import modes (MODE-F2), app-based auto-activation (MODE-F4), mode gallery (MODE-F5).
- "Reset seeded mode to default" affordance.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| MODE-03 (UI half) | User can create, edit, and delete Smart Modes (name + prompt + optional target language) from the settings UI | `addSmartMode` / `updateSmartMode` / `deleteSmartMode` commands confirmed in `bindings.ts`; inline-card pattern research complete |
| MODE-04 (UI half) | User can assign a distinct global shortcut to each Smart Mode from the inline shortcut chip | `setSmartModeBinding(modeId, binding)` → `BindingResponse` confirmed; `GlobalShortcutInput` adaptation path mapped |
| MODE-05 | Shortcut conflicts detected and surfaced inline at bind time (no silent registration failure) | `BindingResponse { success, binding, error }` already returns conflict info; inline red text pattern confirmed |
| MODE-06 | Smart Modes presented as a visual card list, replacing single-prompt dropdown | `PostProcessingSettingsPromptsComponent` + `transcribe_with_post_process` ShortcutInput confirmed for removal; card visual language from `LlmLibrarySection.tsx` / `ModelCard.tsx` confirmed |
| TRANS-01 | Translation as first-class Smart Mode with EN/ES/FR/ZH presets, each bindable | Phase 12 seeded modes (`mode_translate_en/es/fr/zh`) confirmed in `settings.rs`; TranslateGemma catalogue entry is the backend addition |
| TRANS-02 | Translation runs fully offline through embedded LLM | `run_inference()` + `apply_chat_template()` confirmed in `llm.rs`; Translation branch stub in `actions.rs` confirmed for wiring |
| L10N-01 | All new strings propagated to 20 locales; `bun run check:translations` passes | Script at `scripts/check-translations.ts` confirmed; 20 locales confirmed (`ar bg cs de en es fr he it ja ko pl pt ru sv tr uk vi zh zh-TW`); RTL infrastructure at `src/lib/utils/rtl.ts` confirmed |
</phase_requirements>

---

## Summary

Phase 13 is primarily a React/TypeScript frontend phase with one Rust backend extension (TranslateGemma catalogue entry + Translation execution wiring). The backend CRUD infrastructure from Phase 12 is complete and correct — all Tauri commands, the `SmartMode`/`SmartModeKind`/`TargetLanguage`/`BindingResponse` types, and the per-mode shortcut routing exist and are accessible via `bindings.ts`. The phase adds the UI surface that makes this infrastructure user-visible.

The core frontend work is: (1) replace `PostProcessingSettingsPromptsComponent` and the `transcribe_with_post_process` `ShortcutInput` with a new `SmartModesSection` containing `SmartModeCard` components; (2) adapt `GlobalShortcutInput` into a per-mode shortcut chip that routes through `setSmartModeBinding` and renders `BindingResponse.error` inline; (3) build the `TranslationEngineChoiceModal` that stores the global engine preference; (4) wire the Translation execution path in `actions.rs` (replace the stub warning with actual inference); (5) add TranslateGemma to `llm.rs` catalogue; and (6) propagate ~30 new i18n keys to all 20 locales.

**Primary recommendation:** Implement in three sequential waves — (Wave 1) backend extensions (TranslateGemma catalogue entry, `translation_engine_choice` settings field, Translation execution wiring in `actions.rs`); (Wave 2) new React components (`SmartModeCard`, `SmartModesSection`, `SmartModeShortcutChip`, `TranslationEngineChoiceModal`); (Wave 3) i18n propagation and `PostProcessingSettings.tsx` surgery. Each wave is independently committable and verifiable.

---

## Standard Stack

### Core (no new dependencies)

| Library | Version | Purpose | Notes |
|---------|---------|---------|-------|
| React | 18.x (existing) | UI components | Hooks-based functional components |
| i18next | existing | Translation strings | `useTranslation()`, no hardcoded JSX |
| lucide-react | existing | Icons (`Pencil`, `Trash2`, `Plus`) | Already used throughout |
| Zustand | existing | Client state (`useLlmModelStore`, `useSettingsStore`) | Existing stores extended |
| @tauri-apps/plugin-dialog | existing | `ask()` for delete confirmation | Already used in `LlmLibrarySection.tsx` |
| tauri-plugin-store | existing | Persist translation engine choice | Same as all other settings |

**No new npm or cargo dependencies.** This phase adds zero new packages.

---

## Architecture Patterns

### Recommended Project Structure (new files only)

```
src/components/settings/post-processing/
├── SmartModesSection.tsx          # Top-level section: Rewrite + Translation groups
├── SmartModeCard.tsx              # Single card: collapsed + inline expanded form
├── SmartModeShortcutChip.tsx      # Per-card shortcut capture + conflict display
└── TranslationEngineChoiceModal.tsx # First-activation global engine choice

src-tauri/src/
├── managers/llm.rs                # +1 catalogue entry: TranslateGemma 4B
├── settings.rs                    # +1 field: translation_engine_choice
└── actions.rs                     # Translation execution path (replace stub)
```

### Pattern 1: SmartMode Card Visual Language

Reuse `ModelCard.tsx` / `LlmLibrarySection.tsx` provider row directly as the visual template. The card frame is:
```tsx
// Source: src/components/settings/post-processing/LlmLibrarySection.tsx renderProviderRow()
className={`flex flex-col gap-1 px-4 py-3 rounded-xl border-2 transition-all ${
  isExpanded
    ? "border-logo-primary/40 bg-logo-primary/5"
    : "border-mid-gray/20 hover:border-logo-primary/50 hover:bg-logo-primary/5"
}`}
```
Translation disabled state adds `opacity-60` to card content and `bg-mid-gray/5` background (no hover, `aria-disabled="true"`).

### Pattern 2: Tauri Command → BindingResponse

The shortcut chip calls `setSmartModeBinding` and reads `BindingResponse.success`/`error` to drive inline conflict display:
```typescript
// Source: src/bindings.ts line 315
async setSmartModeBinding(modeId: string, binding: string): Promise<Result<BindingResponse, string>>
// BindingResponse shape (line 970):
// { success: boolean; binding: ShortcutBinding | null; error: string | null }
```
On `success === false`, display `BindingResponse.error` as `text-xs text-red-400 mt-1` below the chip with `role="alert"`. Do NOT call `updateBinding` from `useSettings` — that path is for system bindings, not Smart Mode bindings.

### Pattern 3: Inline Edit State Management

Use local `useState` per card (expanded/collapsed, draft name, draft prompt/language). No global state for edit drafts. On "Save mode": call `updateSmartMode(id, name, prompt, targetLanguage)`, await, then call `listSmartModes()` to refresh the parent list. On "Create mode": call `addSmartMode(name, kind, prompt, targetLanguage)`, await success, then refresh. This matches the existing `PostProcessingSettingsPromptsComponent` pattern.

### Pattern 4: Delete via Tauri Dialog

```typescript
// Source: src/components/settings/post-processing/LlmLibrarySection.tsx line 147
import { ask } from "@tauri-apps/plugin-dialog";
const confirmed = await ask(t('smartModes.delete.confirm', { name }), {
  title: t('smartModes.delete.title'),
  kind: "warning",
});
if (confirmed) {
  await commands.deleteSmartMode(id);
  // refresh list
}
```
Shortcut is auto-unregistered server-side by `deleteSmartMode` — no frontend cleanup needed.

### Pattern 5: Seeded Mode Name i18n Lookup

Map seeded mode IDs to i18n key suffixes for display-time localization:
```typescript
const SEEDED_MODE_ID_TO_I18N_KEY: Record<string, string> = {
  "mode_clean_up":       "smartModes.defaultModes.cleanUp",
  "mode_make_formal":    "smartModes.defaultModes.makeFormal",
  "mode_make_casual":    "smartModes.defaultModes.makeCasual",
  "mode_email":          "smartModes.defaultModes.writeAsEmail",
  "mode_bullet_points":  "smartModes.defaultModes.bulletPoints",
  "mode_summarize":      "smartModes.defaultModes.summarize",
  "mode_translate_en":   "smartModes.defaultModes.translateToEnglish",
  "mode_translate_es":   "smartModes.defaultModes.translateToSpanish",
  "mode_translate_fr":   "smartModes.defaultModes.translateToFrench",
  "mode_translate_zh":   "smartModes.defaultModes.translateToChinese",
};
// Display: t(SEEDED_MODE_ID_TO_I18N_KEY[mode.id] ?? mode.name, mode.name)
// User-created modes: display mode.name directly (no key lookup)
```

### Pattern 6: Translation Execution Wiring (Rust)

The stub in `actions.rs` at line 479 must be replaced. The wiring reads the `translation_engine_choice` field from settings to determine path:
- If `TranslateGemma` engine chosen: load/use the `translate-gemma-4b` model, build the prompt as `"<source_text>\n{lang_code}"` per the TranslateGemma chat template format (model's embedded Jinja template handles the structure), call `llm_manager.run_inference(prompt).await`.
- If `generic` engine chosen: inject the hardcoded prompt `"Translate the following text to {target_language_label}. Output only the translation, no explanation or commentary.\n\n{transcription}"`, call `run_inference` with the active model.

The existing `run_inference()` signature (`pub async fn run_inference(&self, prompt: String) -> Result<String>`) handles both paths — the chat template is applied internally in `llm.rs` using the model's embedded template.

### Anti-Patterns to Avoid

- **Do not call `setActiveSmartMode`** from the UI — `smart_mode_active_id` is vestigial per CONTEXT.md. The CRUD commands still exist but the UI ignores active-mode state.
- **Do not reuse `updateBinding` from `useSettings`** for Smart Mode shortcuts — it calls `changeBinding`, which is for system-registered binding IDs. Smart Mode bindings use `setSmartModeBinding`.
- **Do not `suspendBinding` for Smart Mode bindings before recording** — the suspend/resume mechanism in `GlobalShortcutInput` uses the binding ID to unregister while recording. Smart Mode bindings use `smart_mode_{id}` as the binding ID; verify `suspendBinding` accepts that format before using it, or implement recording without suspend.
- **Do not leave the "Prompts" SettingsGroup** in `PostProcessingSettings.tsx` — the entire `PostProcessingSettingsPrompts` component and the `transcribe_with_post_process` ShortcutInput row are removed in this phase.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Delete confirmation | Custom modal/alert | `ask()` from `@tauri-apps/plugin-dialog` | Already used in `LlmLibrarySection.tsx`; native OS dialog is consistent |
| Shortcut key capture | Custom key listener | Adapt `GlobalShortcutInput.tsx` | Handles modifier sorting, OS-specific key naming, suspend/resume, click-outside cancel |
| Progress bar during download | Custom progress UI | Existing `ModelCard` download progress pattern via `useLlmModelStore.downloadProgress` | Already wired to `llm-download-progress` event |
| Translation engine choice persistence | Custom storage | `tauri-plugin-store` via `updateSetting` on `AppSettings` field | All settings already persist this way |
| i18n key completeness check | Manual audit | `bun run check:translations` (`scripts/check-translations.ts`) | Automated; exits non-zero on missing/extra keys |
| Chat template application | Custom formatter | `model_arc.apply_chat_template()` in `llm.rs` | Already implemented and working for all 4 catalogue models |

---

## Common Pitfalls

### Pitfall 1: `setSmartModeBinding` vs `updateBinding`/`changeBinding`

**What goes wrong:** Routing the Smart Mode shortcut chip through `useSettings().updateBinding(id, binding)` — this calls `commands.changeBinding(id, binding)` which only works for pre-registered system binding IDs (`transcribe`, `cancel`, etc.). Smart Mode bindings are registered dynamically by `set_smart_mode_binding`, not pre-seeded in the default binding map.

**How to avoid:** Always call `commands.setSmartModeBinding(modeId, binding)` directly and read the returned `BindingResponse`. Never touch `updateBinding` for Smart Modes.

**Warning signs:** The binding appears to save but the shortcut doesn't fire, or `changeBinding` returns an error like "binding not found".

### Pitfall 2: `suspendBinding` ID format for Smart Modes

**What goes wrong:** Calling `commands.suspendBinding("smart_mode_{id}")` before recording expects this ID to exist in the binding map, but the current `suspendBinding` implementation may only work with pre-registered IDs.

**How to avoid:** Verify in `src-tauri/src/shortcut/mod.rs` whether `suspend_binding` looks up by binding key directly or requires a pre-registration. If it only accepts pre-registered IDs, skip the suspend step for Smart Mode chips — recording is harmless without it (the shortcut fires but the recorded-key event also fires, so you get duplicate presses; handle by ignoring shortcut events while `isRecording === true`).

**Warning signs:** `suspendBinding` returns an error; shortcut fires while recording keys.

### Pitfall 3: Check:translations fails on extra keys

**What goes wrong:** Adding keys to `en/translation.json` and propagating them to 19 other locales, but one locale file ends up with an extra key that's NOT in en (perhaps a leftover from Phase 11). The script catches both missing and extra keys.

**How to avoid:** Before adding new keys, run `bun run check:translations` to confirm the baseline is clean. After adding all new keys to en, add the identical key structure (English values as fallback) to all 19 other locales. Re-run the script.

**Warning signs:** `scripts/check-translations.ts` reports "Extra N keys" for a specific locale.

### Pitfall 4: `PostProcessingSettings.tsx` surgery leaves dead UI

**What goes wrong:** Removing `PostProcessingSettingsPromptsComponent` (the old "Prompts" SettingsGroup) without also removing the `transcribe_with_post_process` ShortcutInput row. That shortcut ID still exists in the binding map (it was kept for backwards compatibility in Phase 12) but it should no longer be surfaced in the UI — showing it would confuse users who see both it AND per-mode shortcuts.

**How to avoid:** Remove both the `ShortcutInput shortcutId="transcribe_with_post_process"` row inside the "Shortcuts" SettingsGroup AND the entire "Prompts" SettingsGroup. Replace the Prompts SettingsGroup with `SmartModesSection`.

**Warning signs:** The UI shows a "transcribe_with_post_process" shortcut chip that can't be made meaningful.

### Pitfall 5: Translation cards interactive when no translation engine chosen

**What goes wrong:** Translation cards allow shortcut binding before the user has selected a translation engine, resulting in a bound shortcut that silently does nothing (the stub warning log is the only output).

**How to avoid:** Gate shortcut chip interactivity on `translationEngineChosen !== null`. When engine is not chosen: `aria-disabled="true"`, chip is non-clickable, edit/delete icons are hidden, `opacity-60` applied. The only interactive element in pre-enable state is the "Choose translation engine" CTA.

### Pitfall 6: TranslateGemma SHA256 not pre-known

**What goes wrong:** Hardcoding a placeholder SHA256 or skipping verification for TranslateGemma while all other catalogue models have SHA256 hashes verified at download time.

**How to avoid:** The SHA256 must be computed from the actual downloaded file. Two options: (a) download the file once during implementation, compute SHA256, hardcode it in the catalogue entry (same approach as Qwen/Gemma/Phi/Llama entries); (b) add a `sha256: None` fallback path to `download_llm_model()` that skips verification when no hash is provided (simpler but less secure). Option (a) matches the existing pattern. The file at `bullerwins/translategemma-4b-it-GGUF` does not publish SHA256 in its card.

---

## Code Examples

### Existing Command Signatures (verified from `src/bindings.ts`)

```typescript
// CRUD commands (Phase 12, confirmed line 275-322)
commands.addSmartMode(name: string, kind: SmartModeKind, prompt: string, targetLanguage: TargetLanguage | null): Promise<Result<SmartMode, string>>
commands.updateSmartMode(id: string, name: string, prompt: string, targetLanguage: TargetLanguage | null): Promise<Result<null, string>>
commands.deleteSmartMode(id: string): Promise<Result<null, string>>
commands.listSmartModes(): Promise<Result<SmartMode[], string>>
commands.setSmartModeBinding(modeId: string, binding: string): Promise<Result<BindingResponse, string>>
commands.setActiveSmartMode(id: string): Promise<Result<null, string>>  // vestigial, not used in UI

// Types (confirmed line 1008-1011)
type SmartMode = { id: string; name: string; kind: SmartModeKind; prompt: string; target_language?: TargetLanguage | null }
type SmartModeKind = "rewrite" | "translation"
type TargetLanguage = { code: string; label: string }
type BindingResponse = { success: boolean; binding: ShortcutBinding | null; error: string | null }
```

### Existing ShortcutInput Recording State (verified from `GlobalShortcutInput.tsx`)

```tsx
// Source: src/components/settings/GlobalShortcutInput.tsx
// Recording state chip style (line 279):
className="px-2 py-1 text-sm font-semibold border border-logo-primary bg-logo-primary/30 rounded-md"
// Idle state chip style (line 284):
className="px-2 py-1 text-sm font-semibold bg-mid-gray/10 border border-mid-gray/80 hover:bg-logo-primary/10 rounded-md cursor-pointer hover:border-logo-primary"
// Key sorting logic (lines 82-103): modifiers first, then main key
```

### Existing LlmModelInfo Catalogue Entry Shape (verified from `llm.rs` line 117)

```rust
// Source: src-tauri/src/managers/llm.rs catalogue()
LlmModelInfo {
    id: "translate-gemma-4b".to_string(),
    name: "TranslateGemma 4B".to_string(),
    description: "Dedicated translation model — 55 benchmarked languages".to_string(),
    filename: "translategemma-4b-it-Q4_K_M.gguf".to_string(),
    url: Some("https://huggingface.co/bullerwins/translategemma-4b-it-GGUF/resolve/main/translategemma-4b-it-Q4_K_M.gguf".to_string()),
    sha256: None,  // Must be computed; fill in after first download or run headless SHA256 at build
    size_mb: 2490,
    is_downloaded: false,
    is_downloading: false,
    partial_size: 0,
    is_custom: false,
    is_recommended: false,
}
```

Note: the `MODEL_ID_TO_I18N_KEY` map in `LlmLibrarySection.tsx` must be updated with `"translate-gemma-4b": "translateGemma4b"` and a matching i18n key added.

### Translation Engine Choice Settings Field (new Rust field)

```rust
// src-tauri/src/settings.rs — add to AppSettings struct:
#[serde(default)]
pub translation_engine_choice: Option<TranslationEngineChoice>,

// New enum:
#[derive(Serialize, Deserialize, Debug, Clone, Default, Type, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TranslationEngineChoice {
    #[default]
    NotChosen,       // disabled state — no engine selected yet
    TranslateGemma,  // use TranslateGemma model
    GenericModel,    // use active LLM with hardcoded prompt
}
```

This field must also be exposed in `bindings.ts` via tauri-specta re-generation (happens automatically on `bun run tauri dev`).

### PostProcessingSettings Layout After Surgery

The existing component (verified `PostProcessingSettings.tsx` lines 693-709) currently renders:

```
SettingsGroup "Shortcuts" → ShortcutInput shortcutId="transcribe_with_post_process"
SettingsGroup "Engine" → PostProcessingSettingsApi (unchanged)
SettingsGroup "Prompts" → PostProcessingSettingsPrompts (REMOVE)
```

After Phase 13:
```
SettingsGroup "Shortcuts" → (REMOVE the ShortcutInput for transcribe_with_post_process)
SettingsGroup "Engine" → PostProcessingSettingsApi (unchanged)
SmartModesSection (replaces "Prompts" group entirely)
```

The "Shortcuts" SettingsGroup may be removed entirely if it has no other rows, OR kept if any other shortcut reference lives there — verify before removing.

---

## TranslateGemma Catalogue Details

| Property | Value | Confidence |
|----------|-------|-----------|
| Repository | `bullerwins/translategemma-4b-it-GGUF` (HuggingFace) | HIGH — well-maintained community GGUF, active as of Jan 2026 |
| Filename | `translategemma-4b-it-Q4_K_M.gguf` | HIGH — confirmed via direct HuggingFace fetch |
| URL | `https://huggingface.co/bullerwins/translategemma-4b-it-GGUF/resolve/main/translategemma-4b-it-Q4_K_M.gguf` | HIGH |
| Size | ~2490 MB (2.49 GB) | HIGH — consistent with Q4_K_M 4B quantization |
| SHA256 | Not published by repo maintainer | CONFIRMED GAP — must be computed at implementation time |
| Benchmarked languages | 55 (WMT24++) | HIGH — confirmed from Google model card |
| Total supported language codes | 160+ (model architecture supports more than benchmarked) | MEDIUM — from HuggingFace discussion thread |
| Chat template | Embedded Jinja in GGUF; requires `source_lang_code` + `target_lang_code` | HIGH |
| License | Google Gemma License (commercial-OK, not OSI) | HIGH — same family as Gemma 3 4B already in catalogue |

**SHA256 resolution strategy:** Compute it during Wave 1 implementation by downloading the file and running `sha256sum`. The catalogue entry can ship with `sha256: None` as a temporary measure and be filled in before the phase verification, or the planner can create an explicit task for this.

---

## i18n Propagation Workflow

### How `check:translations` works (verified from `scripts/check-translations.ts`)

1. Loads `src/i18n/locales/en/translation.json` as the reference.
2. For each of the 19 non-English locales, checks every key path from en is present, and no extra keys exist.
3. Exits non-zero on any mismatch.
4. Does NOT validate translation quality — English fallback values are acceptable.

**The 20 locale codes** (verified by directory listing): `ar bg cs de en es fr he it ja ko pl pt ru sv tr uk vi zh zh-TW`

### Propagation Rule

For every new key added to `en/translation.json`, add the IDENTICAL key structure to all 19 other locale files with the English value as the placeholder. Real translations are deferred (consistent with Phase 11 approach where all 19 locales received English fallback strings).

### RTL Handling (confirmed)

The app already has full RTL infrastructure at `src/lib/utils/rtl.ts` and `src/i18n/index.ts`. The `document.dir` attribute is set to `"rtl"` automatically when `ar` or `he` is the active app language. Smart Mode card layout uses `flex`/`justify-between`/`gap` — these reverse correctly under `dir="rtl"`. The only explicit RTL override needed: shortcut chip element should carry `dir="ltr"` to prevent right-to-left rendering of key combos (e.g. `Cmd+K` must not render as `K+dmC`).

### New i18n Keys Required

The full list is documented in `13-UI-SPEC.md` Copywriting Contract section. Summary: ~30 new keys under `smartModes.*` namespace:

- `smartModes.sections.{rewrite, translation}`
- `smartModes.{createRewrite, createTranslation}`
- `smartModes.card.{saveCta, discardCta, createCta, discardNewCta, addShortcut, shortcutConflict, targetLanguage}`
- `smartModes.kind.{rewrite, translation}`
- `smartModes.translation.{enableHeading, enableBody, chooseCta, genericEngineNote, modal.*}`
- `smartModes.delete.{title, confirm}`
- `smartModes.noModel.{heading, body}`
- `smartModes.defaultModes.{cleanUp, makeFormal, makeCasual, writeAsEmail, bulletPoints, summarize, translateToEnglish, translateToSpanish, translateToFrench, translateToChinese}`
- `smartModes.edit.ariaLabel` / `smartModes.delete.ariaLabel`
- `settings.postProcessing.modelsAndLocalProcessing.library.models.translateGemma4b.description` (new catalogue model)

---

## State of the Art

| Old (Phase 12 stub) | New (Phase 13) | Impact |
|---------------------|----------------|--------|
| Translation branch returns `None`, logs warn | Translation executes via TranslateGemma or generic model | TRANS-02 complete |
| `PostProcessingSettingsPrompts`: single dropdown + textarea | `SmartModesSection`: card list with inline editing | MODE-06 + MODE-03 UI complete |
| No per-mode shortcut in UI (binding exists in backend only) | `SmartModeShortcutChip` on each card | MODE-04 + MODE-05 complete |
| 19 locales have English fallbacks for Phase 11 strings | All 20 locales have Phase 13 string coverage | L10N-01 complete |

**`transcribe_with_post_process` binding:** Still registered in the shortcut system (Phase 12 migrated the old combo onto `smart_mode_{active_id}`). The UI reference to it (the `ShortcutInput` row in PostProcessingSettings) is REMOVED in Phase 13. The backend binding entry remains but becomes invisible/unreachable from the UI. This is intentional per CONTEXT.md: the migrated binding fires the mode that received it, not a generic "post process" action.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` (unit) + manual app smoke test |
| Config file | `src-tauri/Cargo.toml` (test modules inline in source files) |
| Quick run command | `cd src-tauri && cargo test -p dictus-desktop 2>&1` |
| Full suite command | `cd src-tauri && cargo test -p dictus-desktop 2>&1 && bun run check:translations` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| MODE-03 | Add/update/delete SmartMode via commands | Integration (manual app) | `cargo test actions_tests` (existing) | ✅ `src-tauri/src/actions.rs` |
| MODE-04 | setSmartModeBinding returns BindingResponse | Unit (existing backend) | `cargo test -p dictus-desktop` | ✅ `src-tauri/src/shortcut/` |
| MODE-05 | Conflict → BindingResponse.success=false | Unit (backend) | `cargo test -p dictus-desktop` | ✅ |
| MODE-06 | Card list renders, replace old prompt UI | Manual smoke | `bun run tauri dev` | ❌ Wave 0: UI smoke test plan |
| TRANS-01 | 4 seeded translation cards present | Manual smoke | `bun run tauri dev` | ❌ |
| TRANS-02 | Translation executes offline via LLM | Manual E2E | `bun run tauri dev` + record + trigger mode | ❌ |
| L10N-01 | check:translations passes | Automated | `bun run check:translations` | ✅ `scripts/check-translations.ts` |

### Sampling Rate

- **Per task commit:** `bun run check:translations && bun run lint`
- **Per wave merge:** `cd src-tauri && cargo test -p dictus-desktop 2>&1 && bun run check:translations && bun run lint`
- **Phase gate:** Full suite + manual E2E smoke test (record → transcribe → bind shortcut → trigger Smart Mode → verify output pasted)

### Wave 0 Gaps

- [ ] No automated test for Translation execution wiring — manual E2E required (`bun run tauri dev`, bind a translation mode shortcut, record audio, trigger)
- [ ] `bun run check:translations` baseline must be verified clean BEFORE adding new keys (run once at Wave 0 start to confirm no pre-existing failures from Phase 11/12)

---

## Open Questions

1. **SHA256 for `translategemma-4b-it-Q4_K_M.gguf`**
   - What we know: file is ~2490 MB at `bullerwins/translategemma-4b-it-GGUF`; maintainer does not publish SHA256
   - What's unclear: exact hash value
   - Recommendation: Wave 1 task explicitly downloads the file and records SHA256 before finalizing the catalogue entry; alternatively ship with `sha256: None` and add a `TODO` in the catalogue comment

2. **`suspendBinding` compatibility with `smart_mode_{id}` binding IDs**
   - What we know: `GlobalShortcutInput` calls `suspendBinding(id)` where id is a pre-registered system binding ID; Smart Mode bindings use the `smart_mode_{id}` scheme registered dynamically
   - What's unclear: whether `suspend_binding` in Rust handles dynamically-registered IDs
   - Recommendation: Check `src-tauri/src/shortcut/mod.rs` `suspend_binding` implementation before implementing `SmartModeShortcutChip`; if it fails for dynamic IDs, skip suspend and handle recording state purely in React

3. **`TranslationEngineChoice` in `bindings.ts` auto-generation**
   - What we know: tauri-specta regenerates `bindings.ts` on `bun run tauri dev`; new types added to `settings.rs` with `#[derive(Type)]` appear automatically
   - What's unclear: whether adding a new enum to `AppSettings` requires any manual step beyond `#[derive(Type)]`
   - Recommendation: Standard — add `#[derive(Type)]` and let specta handle it; verify after first `bun run tauri dev` that the new type appears in `bindings.ts`

---

## Sources

### Primary (HIGH confidence)
- `src/bindings.ts` — All CRUD command signatures and type definitions verified directly (lines 275-322, 970-1011)
- `src-tauri/src/settings.rs` — `SmartMode`, `SmartModeKind`, `TargetLanguage` structs, `AppSettings` fields, seeded defaults (lines 99-120, 402-404, 657-744)
- `src-tauri/src/managers/llm.rs` — `catalogue()`, `run_inference()`, `apply_chat_template()` (lines 116-175, 855-905)
- `src-tauri/src/actions.rs` — Translation stub and `resolve_mode_prompt()` (lines 449-505, 863-879)
- `src/components/settings/GlobalShortcutInput.tsx` — Full shortcut capture implementation
- `src/components/settings/post-processing/PostProcessingSettings.tsx` — Confirmed removal targets
- `src/components/settings/post-processing/LlmLibrarySection.tsx` — Card visual language pattern
- `scripts/check-translations.ts` — Translation check behavior verified
- `src/i18n/locales/` — 20 locales confirmed by directory listing

### Secondary (MEDIUM confidence)
- [bullerwins/translategemma-4b-it-GGUF](https://huggingface.co/bullerwins/translategemma-4b-it-GGUF) — GGUF filename, URL, size confirmed via WebFetch
- [google/translategemma-27b-it HuggingFace discussions](https://huggingface.co/google/translategemma-27b-it/discussions/1) — 55 benchmarked languages, 160+ total, `source_lang_code` required in chat template

### Tertiary (LOW confidence)
- WebSearch for TranslateGemma GGUF details — confirmed by official HuggingFace fetch; SHA256 remains unverified

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all libraries in use
- Architecture: HIGH — code read directly from source; patterns verified against live implementation
- Pitfalls: HIGH — derived from reading actual code paths (the stub in `actions.rs`, the `updateBinding` vs `setSmartModeBinding` divergence)
- TranslateGemma catalogue details: MEDIUM — URL/size confirmed; SHA256 is an open gap requiring implementation-time computation

**Research date:** 2026-06-03
**Valid until:** 2026-07-03 (stable domain; TranslateGemma GGUF URL is stable on HuggingFace CDN)
