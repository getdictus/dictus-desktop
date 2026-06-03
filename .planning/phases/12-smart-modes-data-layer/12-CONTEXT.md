# Phase 12: Smart Modes Data Layer - Context

**Gathered:** 2026-06-03
**Status:** Ready for planning

<domain>
## Phase Boundary

The Smart Modes **data model** lives in settings and the backend pipeline routes through it. Phase 12 delivers:

- A self-contained `SmartMode` type (with a `kind` distinction) stored in settings, replacing the v1.2 single-prompt model.
- An explicit, fixture-verified v1.2→v1.3 **migration** with `settings_schema_version`, preserving prompt text and shortcut combos (no data loss).
- **CRUD backend commands** (create/edit/delete/list) with regenerated tauri-specta bindings importable by the frontend.
- **~10 default Smart Modes** seeded (6 Rewrite + 4 Translation).
- **Per-mode shortcut routing** via the `smart_mode_{id}` binding scheme: registered at init, updatable dynamically, routing a transcription through the triggered mode.

**Out of this phase (belongs to Phase 13):**
- The visual card-list UI, create/edit/delete UI surface, and inline shortcut-bind UI (MODE-06, MODE-03 UI half).
- The shortcut-conflict **warning UI** (MODE-05) — Phase 12 only returns the conflict error from the backend; Phase 13 surfaces it.
- **Translation execution** — the TranslateGemma engine + native chat-template path (TRANS-01/02). Phase 12 ships translation modes as *data + shortcut routing only*; firing one routes correctly but the translation engine is wired in Phase 13.
- 20-locale string propagation (L10N-01).

</domain>

<decisions>
## Implementation Decisions

### Data model shape
- **Self-contained `SmartMode`** — a Smart Mode IS the evolved prompt, not a wrapper around one (honors PROJECT.md: "Smart Modes = prompts, pas un nouveau concept opaque"). The v1.2 `LLMPrompt` evolves into `SmartMode`.
- **Single source of truth (replace, not parallel):** migrate `post_process_prompts` → a new `smart_modes` list, point the post-processing pipeline at it, and drop/deprecate the old `post_process_prompts` / `post_process_selected_prompt_id` fields after migration. No two overlapping lists.
- **One type, kind-gated fields** (not a tagged Rust enum — keeps tauri-specta bindings, CRUD, migration, and shortcut code uniform across modes):
  ```
  SmartMode {
    id: String,
    name: String,
    kind: SmartModeKind,              // Rewrite | Translation — EXPLICIT, not derived
    prompt: String,                    // used by Rewrite; empty/ignored for Translation
    target_language: Option<{ code, label }>,  // required for Translation; None for Rewrite
    // per-mode shortcut handled via the existing binding system keyed by `smart_mode_{id}`
  }
  ```
- **Rewrite vs Translation are two genuinely different shapes** (explicit `kind`):
  - **Rewrite** wraps a free `prompt` and runs through the active generic instruct model.
  - **Translation** imposes a `target_language` on a translation-specialized engine (TranslateGemma) via its **native chat template** — NOT a wrapped prompt. Rationale (Phase 11 post-UAT, STATE.md:78-79): specialized translation models "ignore generic post-process prompts, pass text through unchanged"; TranslateGemma "needs its native format (direct text + target language), NOT a wrapped custom prompt."
- **`target_language` = ISO code + display name** (e.g. `{ code: "fr", label: "French" }`). Stable identity for the engine chat template, language-agnostic of UI locale, clean for Phase 13 grouping/labelling/flags.

### Default Smart Modes (10 total)
- **Order:** Clean Up first → rewrites → translations (seeds Phase 13's grouped-by-kind list order).
- **Rewrite (6):** `Clean Up` (active default), `Make Formal`, `Make Casual`, `Email`, `Bullet Points`, `Summarize`.
  - Dropped from the roadmap's listed set: **Write as SMS** (overlaps Make Casual). Bullet Points kept (distinct from Summarize — restructure vs condense; valuable for dictated notes).
- **Translation (4):** `Translate → English`, `→ Spanish`, `→ French`, `→ Chinese` (EN/ES/FR/ZH). ZH included now (user decision); Phase 13's TRANS-01 may finalize/extend presets when the engine lands.
- **`Clean Up` is the safe first/active mode** on **fresh installs** (lowest-risk: fix grammar/filler/punctuation, preserve meaning). Upgraders keep their migrated selection instead (see Migration).
- **Exact prompt wording is NOT locked here** — refined during implementation against real local models (MODE-02). This context locks the *set, kinds, order, and active default*, not the prompt strings.

### Migration & coexistence (v1.2 → v1.3)
- **Migrate existing + add defaults:** every existing `LLMPrompt` becomes a Rewrite `SmartMode` (preserved), AND the 10 curated defaults are seeded so upgraders gain the new set. Fresh installs get just the 10 defaults.
- **"Improve Transcriptions" reconciliation — pristine→replace, edited→keep both:**
  - If the v1.2 default prompt "Improve Transcriptions" is **unmodified**, replace it with the new `Clean Up` default (no near-duplicate).
  - If the user **edited** it, preserve their version as a custom Rewrite mode **and** add `Clean Up` separately (no data loss, no silent overwrite).
- **Active selection preserved:** the upgrader's `post_process_selected_prompt_id` maps to the corresponding migrated mode, which becomes the active mode (fresh installs → Clean Up).
- **Loss-safe + idempotent (mechanics = Claude's discretion):** introduce `settings_schema_version`; detect missing/old version → migrate → stamp version; safe to re-run (no double-seeding, no duplicate defaults); on failure, leave v1.2 data intact rather than half-migrating. **Verified against an actual v1.2 settings JSON fixture** (success criterion 1).

### Per-mode shortcuts
- **Retire the generic `transcribe_with_post_process` binding ID** in favor of per-mode shortcuts — BUT **transfer its key combo** onto the migrated active mode's own per-mode shortcut so the user's working combo keeps firing (now triggers that specific mode). This satisfies both "retire generic" and the migration's "shortcut bindings preserved" criterion (no data loss).
  - The plain `transcribe` (raw transcription, no post-process) binding is **unaffected**.
- **Defaults ship unbound** — the only pre-bound mode on upgrade is the one that inherited the transferred combo. No 10 surprise global shortcuts; avoids conflict storms.
- **`smart_mode_{id}` binding scheme:** per-mode shortcuts use this prefix throughout the shortcut + actions pipeline. Init registers a global shortcut **only for modes whose binding is set** (skip unbound); add/update/unregister dynamically on mode create/edit/delete. **Exact init-registration + dynamic-update mechanics = Claude's discretion**, consistent with the existing `change_binding` flow.
- **Conflict handling at the backend:** if two modes request the same combo, the bind command returns an **explicit error/conflict result** the frontend can surface — it **never silently fails to register** (matches MODE-05 intent). The inline warning **UI** is built in Phase 13.

### Claude's Discretion
- `settings_schema_version` numbering, idempotency strategy, and migration code structure (must be fixture-tested + loss-safe).
- Init-registration and dynamic add/remove mechanics for `smart_mode_{id}` shortcuts (mirror existing `change_binding`).
- Exact prompt wording for all 6 rewrite defaults (refined against real models per MODE-02).
- Internal struct/field naming, ID generation scheme for new modes (mirror `prompt_{timestamp}`), and how the active-mode pointer is stored after the field rename.
- How `actions.rs` `post_process_transcription()` is refactored to accept the per-mode override and branch on `kind` (Rewrite prompt path now; Translation path stubbed/routed for Phase 13).

</decisions>

<specifics>
## Specific Ideas

- **Translation is a distinct first-class shape, not "a prompt with a language."** This came directly from the Phase 11 post-UAT finding: TranslateGemma is a genuine Google translation model (4B ≈ Gemma 3 12B on WMT24++) but only works via its native chat template (direct text + target language), and generic prompts pass through unchanged. The `kind` field exists precisely to carry this difference forward to Phase 13.
- **"10 felt like a bit much"** — the set was deliberately curated down from the roadmap's 7 rewrites by dropping Write as SMS; the user wants a tight, non-redundant starter set, not an exhaustive one.
- **No-data-loss is a hard, tested requirement** — both prompt text and shortcut combos survive upgrade; verified against a real v1.2 settings fixture.
- Working branch is `feat/v1.3-smart-modes`.

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase definition & requirements
- `.planning/ROADMAP.md` §"Phase 12: Smart Modes Data Layer" — 4 success criteria; note SC-1 (migration + schema version + fixture), SC-2 (default modes), SC-3 (CRUD + bindings), SC-4 (`smart_mode_{id}` routing). Also the §"NOTE on MODE-03" two-phase split (backend here, UI in Phase 13).
- `.planning/REQUIREMENTS.md` — MODE-01..04 acceptance criteria; MODE-05/06 + TRANS-01/02 + L10N-01 are **Phase 13** (do NOT build the UI/conflict-warning/translation-engine/locale propagation here); Out of Scope table (no silent migration that drops prompts/shortcuts; embedded adds-not-replaces); Future Requirements MODE-F1..F5 (enable/disable, export/import, per-mode provider, app-based auto-activation, gallery — all deferred).
- `.planning/PROJECT.md` — "Smart Modes = prompts, pas un nouveau concept opaque"; local-first constraint; v1.3 scope decisions.

### Phase 11 carry-forward (translation = special)
- `.planning/STATE.md` §lines 78-79 — **TranslateGemma deferred to the Phase 12/13 translation mode**; needs native chat-template format (direct text + target language), NOT a wrapped prompt; specialized models ignore generic post-process prompts. Catalogue refreshed to 4 general-instruct models (Qwen2.5 1.5B / Gemma 3 4B / Phi-4 Mini / Llama 3.2 3B). License notes per model.
- `.planning/phases/11-llm-runtime-foundation/11-CONTEXT.md` — embedded provider behavior, active-model selection pattern, library UI conventions; `LOCAL_PROVIDER_IDS_SET` now includes `"embedded"`.
- `.planning/phases/11-llm-runtime-foundation/11-VERIFICATION.md` §line 111 — documented catalogue drift (ROADMAP SC names 3 models, code ships 4); non-blocking for Phase 12.

### Settings & migration targets (Rust)
- `src-tauri/src/settings.rs` — `AppSettings` struct (~lines 308-409); `LLMPrompt { id, name, prompt }`; `post_process_prompts: Vec<LLMPrompt>` (~371), `post_process_selected_prompt_id: Option<String>` (~373); default "Improve Transcriptions" prompt (~619-623); `ensure_post_process_defaults()` (~634-687) — the implicit-defaults sync hook where migration logic plugs in; `load_or_create_app_settings()` (~822-873), `get_settings()` (~875-897), `write_settings()` (~899-905); store path `settings_store.json` (~689). **No `settings_schema_version` exists yet — introduce it here.**

### CRUD pattern to mirror (Rust)
- `src-tauri/src/shortcut/mod.rs` §lines 924-996 — `add_post_process_prompt` / `update_post_process_prompt` / `delete_post_process_prompt` / `set_post_process_selected_prompt`. Standard load→mutate Vec→write pattern; mirror for Smart Modes CRUD (note: `delete` guards "cannot delete last prompt" and reassigns selection — adapt for modes).

### Shortcut system (Rust)
- `src-tauri/src/shortcut/mod.rs` — `ShortcutBinding { id, name, description, default_binding, current_binding }` (~81-87); `register_shortcut()`; `change_binding()` command (~108-202) — unregister→validate→register→save (template for dynamic per-mode shortcuts); default bindings `transcribe` / `transcribe_with_post_process` / `cancel` (~702-741).
- `src-tauri/src/actions.rs` — `ACTION_MAP` static binding→action map (~778-799); `post_process_transcription()` pipeline (~66-375) incl. prompt-selection logic (~165-186) that reads `post_process_selected_prompt_id` + finds the prompt — the routing point that must branch on the triggered `smart_mode_{id}` and the mode's `kind`; `process_transcription_output()` (~427-465) returning `ProcessedTranscription`; `TranscribeAction` call site (~664-665); cancel-binding suspend/resume (~540, ~572).

### Bindings & command registration
- `src-tauri/src/lib.rs` — `specta_builder.commands([...])` registration (~415-450); tauri-specta export to `../src/bindings.ts` (~534-540). New Smart Mode commands register here and auto-export.
- `src-tauri/src/commands/` — module layout (`mod.rs`, `audio.rs`, `models.rs`, `llm.rs`, `transcription.rs`, `history.rs`); binding/prompt commands currently live in `shortcut/mod.rs` (decide where Smart Mode commands belong).
- `src/bindings.ts` — auto-generated; do not hand-edit.

### Frontend (read-only context for this phase; UI is Phase 13)
- `src/components/settings/post-processing/PostProcessingSettings.tsx` + `src/components/settings/PostProcessingSettingsApi/` (`types.ts`, `usePostProcessProviderState.ts`) — current single-prompt selection surface that Phase 13 replaces with the card list. Phase 12 only regenerates the bindings these will import.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`LLMPrompt` → `SmartMode` evolution** — the existing `{id, name, prompt}` shape is the migration seed; add `kind` + `target_language` and rename the collection.
- **Prompt CRUD commands (`shortcut/mod.rs:924-996`)** — near drop-in template for Smart Mode CRUD (load→mutate Vec→write; adapt the delete/selection guards).
- **`change_binding()` flow (`shortcut/mod.rs:108-202`)** — template for registering/updating `smart_mode_{id}` shortcuts dynamically and detecting conflicts.
- **`ensure_post_process_defaults()` (`settings.rs:634-687`)** — existing implicit-defaults sync; the explicit versioned migration + default-seeding hooks in here / alongside it.
- **`ACTION_MAP` (`actions.rs:778-799`)** — flat binding→action map; extend so `smart_mode_{id}` resolves to a transcription action carrying that mode's id.
- **tauri-specta auto-export** — new `#[tauri::command] #[specta::specta]` commands auto-flow into `bindings.ts` (no manual TS).

### Established Patterns
- **Settings = JSON via tauri-plugin-store**, defaults merged on load (`load_or_create_app_settings`). Migration must run on this load path, version-guarded.
- **Flat binding-id scheme** (`transcribe`, `transcribe_with_post_process`, `cancel`) — `smart_mode_{id}` joins it; the plain `transcribe` binding stays untouched.
- **`post_process` is a bool on `TranscribeAction`** today → must generalize to carry which mode/prompt to apply (per-mode override).
- **No-data-loss / local-first** are project-level non-negotiables (Out of Scope table).

### Integration Points
- `settings.rs` (type + migration + defaults) ↔ `shortcut/mod.rs` (CRUD + binding commands) ↔ `actions.rs` (`ACTION_MAP` + `post_process_transcription` routing) ↔ `lib.rs` (command + specta registration) ↔ `src/bindings.ts` (regenerated).
- `actions.rs` `post_process_transcription()` is where `kind` branches: Rewrite → existing prompt path through the active generic model; Translation → routed but engine execution stubbed for Phase 13.
- Migration entry point: the settings load path (`load_or_create_app_settings` / `ensure_post_process_defaults`), guarded by `settings_schema_version`.

</code_context>

<deferred>
## Deferred Ideas

- **Translation execution engine** (TranslateGemma reintroduction + native chat-template path) — Phase 13 (TRANS-01/02). Phase 12 ships translation modes as data + routing only.
- **Smart Modes UI** — card list, create/edit/delete UI, inline shortcut bind, conflict-warning UI — Phase 13 (MODE-03 UI half, MODE-05, MODE-06).
- **Translation preset finalization** (additional targets beyond EN/ES/FR/ZH, "Translation" group label) — Phase 13 TRANS-01.
- **20-locale string propagation** for mode names/labels — Phase 13 (L10N-01).
- **Write as SMS** default mode — cut from the shipped set (overlaps Make Casual); could be revisited but intentionally excluded.
- **Per-mode provider selection** (MODE-F3, a different LLM per mode), **enable/disable without delete** (MODE-F1), **export/import modes** (MODE-F2), **app-based auto-activation** (MODE-F4), **mode gallery** (MODE-F5) — Future Requirements, not v1.3.

</deferred>

---

*Phase: 12-smart-modes-data-layer*
*Context gathered: 2026-06-03*
