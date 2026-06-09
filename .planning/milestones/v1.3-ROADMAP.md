# Roadmap: Dictus Desktop

## Milestones

- ✅ **v1.0 Handy→Dictus Rebrand** — Phases 1-3 (shipped 2026-04-10) — [archive](milestones/v1.0-ROADMAP.md)
- ✅ **v1.1 Auto-Update & Upstream Sync** — Phases 4-5 (shipped 2026-04-14) — [archive](milestones/v1.1-ROADMAP.md)
- ✅ **v1.2 Polish & Local-First UX** — Phases 6-9 (shipped 2026-05-29) — [archive](milestones/v1.2-ROADMAP.md)
- 🚧 **v1.3 Smart Modes & Local LLM** — Phases 10-13 (in progress)

## Phases

<details>
<summary>✅ v1.0 Handy→Dictus Rebrand (Phases 1-3) — SHIPPED 2026-04-10</summary>

- [x] Phase 1: Bundle Identity (1/1 plans) — completed 2026-04-05
- [x] Phase 2: Visual Rebrand (5/5 plans) — completed 2026-04-09
- [x] Phase 3: Documentation and Cleanup (2/2 plans) — completed 2026-04-09

</details>

<details>
<summary>✅ v1.1 Auto-Update & Upstream Sync (Phases 4-5) — SHIPPED 2026-04-14</summary>

- [x] Phase 4: Updater Infrastructure (4/4 plans) — completed 2026-04-13
- [x] Phase 5: Upstream Sync (3/3 plans) — completed 2026-04-14

</details>

<details>
<summary>✅ v1.2 Polish & Local-First UX (Phases 6-9) — SHIPPED 2026-05-29</summary>

- [x] Phase 6: Brand & Icon Polish (4/4 plans) — completed 2026-04-16
- [x] Phase 7: macOS Clean Shutdown (1/1 plan) — completed 2026-04-23
- [x] Phase 8: Privacy / Local-First UX (10/10 plans) — completed 2026-05-22
- [x] Phase 9: v1.2 Audit Gap Closure (1/1 plan) — completed 2026-05-28

</details>

### 🚧 v1.3 Smart Modes & Local LLM (In Progress)

**Milestone Goal:** Make local post-transcription processing a real and powerful default — an embedded LLM runtime (all platforms, no external Ollama) plus Smart Modes (curated prompts, editable, creatable, each bindable to a shortcut), with multi-target translation as a first-class mode.

- [x] **Phase 10: Prerequisite Gate** — Upstream Sync #2, TECH-04 refactor, and ggml feasibility spike clear all build blockers before feature work begins
- [x] **Phase 11: LLM Runtime Foundation** (4/4 plans) — completed 2026-06-03 — In-process GGUF engine, GPU backends, model downloader, curated catalogue, custom drag/drop, and functional model library
- [x] **Phase 12: Smart Modes Data Layer** — Settings schema migration, SmartMode type + CRUD backend, per-mode shortcut routing, embedded provider, and 10 default modes (completed 2026-06-03)
- [x] **Phase 13: Smart Modes UI + Translation + i18n** — Card list UI, create/edit/delete UI, shortcut bind with conflict detection, translation presets, and 20-locale propagation (completed 2026-06-09)

## Phase Details

### Phase 10: Prerequisite Gate
**Goal**: All build blockers are cleared and the codebase is in a known-clean state before any v1.3 feature code is written
**Depends on**: Phase 9
**Requirements**: PREP-01, PREP-02, PREP-03
**Success Criteria** (what must be TRUE):
  1. Upstream Sync #2 is merged to main with per-commit triage complete; `verify-sync.sh` exits 0; fork policy documents selective cherry-pick going forward and records the AWS Bedrock `aee682f` exclusion decision
  2. `llm_client.rs send_chat_completion_with_schema` accepts a `ChatCompletionRequest` struct instead of 8 positional args; `#[allow(clippy::too_many_arguments)]` is removed; `cargo clippy --all-targets -- -D warnings` exits 0 on all platforms
  3. `llama-cpp-2` compiles alongside `transcribe-rs` on all 7 CI platforms with no linker symbol conflicts; if ggml duplication is detected, `[patch.crates-io]` or equivalent resolution is applied and CI is green

> **Research spike flag (Phase 10):** The ggml conflict resolution path (PREP-01) cannot be determined until `cargo tree | grep ggml` is run against the live Cargo.lock with both crates present. The exact fix — shared ggml via `[patch.crates-io]`, separate feature flags, or an alternative — depends on what the spike reveals. Have the `[patch.crates-io]` approach ready as fallback.
>
> **Ordering within Phase 10:** Execute PREP-03 (Sync #2) first — the sync may touch `llm_client.rs` and `managers/`; refactor on the up-to-date base. Then PREP-02 (TECH-04 struct refactor). Then PREP-01 (ggml feasibility spike).

**Plans**: 3 plans
- [x] 10-01-PLAN.md — PREP-03: Upstream Sync #2 (merge 17-commit delta, revert Bedrock, Fork Policy + Exclusion Log)
- [x] 10-02-PLAN.md — PREP-02: TECH-04 refactor (8-arg fn → ChatCompletionParams struct, drop #[allow], clippy clean)
- [x] 10-03-PLAN.md — PREP-01: ggml feasibility spike (llama-cpp-2 + transcribe-rs coexistence, 6/7 CI platforms; Windows-x64 vulkan deferred to Phase 11)

### Phase 11: LLM Runtime Foundation
**Goal**: Users can download, manage, and run a local GGUF model in-process, with GPU acceleration where available, and the model library card is fully functional
**Depends on**: Phase 10
**Requirements**: LLM-01, LLM-02, LLM-03, LLM-04, MDL-01, MDL-02, MDL-03, MDL-04, MDL-05
**Success Criteria** (what must be TRUE):
  1. User can browse a curated catalogue of 4 local instruct models (Qwen2.5 1.5B, Gemma 3 4B, Phi-4 Mini, Llama 3.2 3B) with the on-disk file size shown before any download begins (post-UAT refresh — TranslateGemma deferred to the Phase 13 translation mode)
  2. User can download a catalogue model in-app, watch live progress, cancel mid-download, and resume; the completed file is SHA256-verified; the URL is a HuggingFace CDN URL (never `blob.handy.computer`)
  3. User can delete a downloaded model from the library and see disk space reclaimed
  4. User can add a custom GGUF model by drag/drop or file picker; it appears in the library alongside catalogue entries
  5. The "Bibliothèque de modèles locaux" placeholder card is replaced by the real, functional model library anchored at the top of the local-processing page; end-to-end verification gate passes on macOS, Windows, and Linux before Phase 12 opens (download → load → inference → streaming tokens confirmed in all three)
  6. "Embedded (local)" is selectable as a post-processing provider alongside existing options; cloud remains opt-in; inference runs on a background thread and never blocks the overlay or shortcuts; GPU acceleration uses Metal on macOS and Vulkan on Windows/Linux with graceful CPU fallback; the loaded model unloads after its idle timeout without affecting the transcription model

> **Research spike flag (Phase 11):** The exact `.metallib` files that `llama-cpp-2` places in `OUT_DIR` and which ones must be added to `tauri.conf.json bundle.resources` are not confirmed in documentation. The first `tauri build` release smoke test on macOS is the verification gate — test with `tauri build`, not `tauri dev`, which does not replicate the production bundle layout.

**Plans**: 4 plans
- [x] 11-01-PLAN.md — LlmManager backend: catalogue, download/verify/delete, GGUF validation, load/infer/unload, idle watcher (TDD)
- [x] 11-02-PLAN.md — LLM Tauri commands + manager registration + embedded provider branch in actions.rs
- [x] 11-03-PLAN.md — Frontend LlmLibrarySection + custom GGUF import + embedded provider UI + English i18n
- [x] 11-04-PLAN.md — Platform gate: macOS .metallib (embedded), Windows x64 Vulkan fix (Ninja), ggml link-flag coexistence, catalogue refresh + unified engine list (post-UAT)

### Phase 12: Smart Modes Data Layer
**Goal**: The Smart Modes data model exists in settings with a working migration from v1.2 prompts, the embedded provider routes through the runtime, and per-mode shortcut infrastructure is functional in the backend
**Depends on**: Phase 11
**Requirements**: MODE-01, MODE-02, MODE-03, MODE-04
**Success Criteria** (what must be TRUE):
  1. Upgrading from a v1.2 settings file migrates existing post-processing prompts to Smart Modes with no data loss; prompt text and shortcut bindings are preserved; `settings_schema_version` is written; the migration is verified against an actual v1.2 settings JSON fixture
  2. Dictus ships with ~10 default Smart Modes ("Clean Up" as safe first mode, plus Make Formal, Make Casual, Write as Email, Write as SMS, Bullet Points, Summarize, Translate to English, Translate to Spanish, Translate to French); each is available as a selectable post-processing option
  3. Smart Modes can be created, edited, and deleted via backend commands; name, prompt text, and optional target language are stored per mode; tauri-specta bindings are regenerated and importable by the frontend
  4. Each Smart Mode can have a distinct global shortcut registered at init and updated dynamically; triggering a mode's shortcut routes the transcription through that mode's prompt; the `smart_mode_{id}` binding prefix is handled throughout the shortcut and actions pipeline

**Plans**: 3 plans
- [ ] 12-01-PLAN.md — Data model + 10 default modes + fixture-tested v1.2→v1.3 migration (settings.rs) [Wave 1]
- [ ] 12-02-PLAN.md — Smart Mode CRUD commands + per-mode binding command + specta registration (shortcut/mod.rs, lib.rs) [Wave 2]
- [ ] 12-03-PLAN.md — smart_mode_ routing: is_transcribe_binding + SmartModeAction + process_transcription_output override + init registration (coordinator, actions, init loops) [Wave 2]

### Phase 13: Smart Modes UI + Translation Presets + i18n
**Goal**: Users interact with Smart Modes through a visual card list UI, can create/edit/delete modes and bind shortcuts from the settings panel, translation is a first-class preset group, and all strings are localized across 20 locales
**Depends on**: Phase 12
**Requirements**: MODE-03, MODE-04, MODE-05, MODE-06, TRANS-01, TRANS-02, L10N-01
**Success Criteria** (what must be TRUE):
  1. Smart Modes are displayed as a visual card list in settings, replacing the single-prompt dropdown; each card shows the mode name, prompt preview, and its bound shortcut (if any)
  2. User can create a new Smart Mode, edit an existing mode's name and prompt, and delete a mode — all from the settings UI without leaving the page
  3. User can bind a distinct global shortcut to each Smart Mode from an inline shortcut input; if the chosen shortcut conflicts with an existing binding, an inline warning appears at bind time with no silent registration failure
  4. Translation presets (EN/ES/FR/ZH as minimum; finalized in-phase) are available as built-in Smart Modes grouped under a "Translation" label; each preset runs fully offline through the embedded LLM; each is bindable to its own shortcut
  5. All new user-facing strings (model library labels, Smart Modes UI, default mode names, translation preset names) are propagated across all 20 locales; `bun run check:translations` passes with 0 errors; the full end-to-end flow (record → transcribe → Smart Mode fires → embedded LLM responds → output pasted) is verified

> **NOTE on MODE-03:** This requirement spans two phases. Phase 12 delivers the backend CRUD commands and type bindings. Phase 13 delivers the UI surface. The requirement is assigned to Phase 13 (the phase that completes the user-observable deliverable), and Phase 12 success criterion 3 documents the backend prerequisite.

**Plans**: 26 plans (4 original + 4 gap-closure from 13-04 UAT + 4 gap-closure from 13-08 re-test + 3 gap-closure from 13-12 re-test + 2 gap-closure from 13-08 round-2 re-test + 2 gap-closure from 13-16/13-17 round-3 re-test + 5 gap-closure from 13-08 round-4 re-test [G13]/[G14]/[G15] + 2 gap-closure from 13-24 round-4 live UAT [G16]/[G17])
- [x] 13-01-PLAN.md — Backend: TranslateGemma catalogue entry + translation_engine_choice setting/command + Translation execution wiring in actions.rs [Wave 1]
- [x] 13-02-PLAN.md — i18n: smartModes.* keys in en + propagation to all 19 non-English locales (L10N-01) [Wave 1]
- [x] 13-03-PLAN.md — React components: SmartModeShortcutChip + SmartModeCard + TranslationEngineChoiceModal + SmartModesSection [Wave 2]
- [x] 13-04-PLAN.md — Mount SmartModesSection in PostProcessingSettings, remove legacy prompt UI + transcribe_with_post_process row (UAT surfaced 8 gaps) [Wave 3]
- [x] 13-05-PLAN.md — Gap closure (backend): clear_smart_mode_binding command, kill legacy transcribe_with_post_process shortcut, seed only Clean Up, expose smart_mode_templates catalogue [Wave 1]
- [x] 13-06-PLAN.md — Gap closure (frontend): keydown-commit shortcut capture (fix Cmd+1), clear affordance, compact right-aligned chip [Wave 2]
- [x] 13-07-PLAN.md — Gap closure (frontend): edit-aware card title, Prompt label, template picker + single '+', changeable translation engine [Wave 2]
- [x] 13-08-PLAN.md — Gap closure: automated gates + human-verify checkpoint re-running the 8 failed UAT tests [Wave 3]
- [x] 13-09-PLAN.md — Gap closure (backend): dedup-aware add_smart_mode (seeded id reuse/overwrite), atomic delete binding cleanup, dangling-binding reconciliation, suspend_all/resume_all commands (tests 4/6/7/9) [Wave 1]
- [x] 13-10-PLAN.md — Gap closure (Apple Swift + actions.rs): task-agnostic Apple Intelligence path (drop fixed CleanedTranscript schema, no word-count truncation) — fixes translation-as-rewrite + bullet truncation (tests 10/11) [Wave 1]
- [x] 13-11-PLAN.md — Gap closure (frontend): name-based picker dedup + overwrite-warn, Custom-form placeholders + ${output} hint (test 4) [Wave 2]
- [x] 13-12-PLAN.md — Gap closure (frontend): chip suspend-all/resume-all capture, engine modal active badge from persisted choice (tests 7/11) [Wave 3]
- [x] 13-13-PLAN.md — Gap closure (backend): cross-mode collision detection in set_smart_mode_binding/find_conflicting_binding ([B5] test 7 block-half) [Wave 1]
- [x] 13-14-PLAN.md — Gap closure (frontend): engine badge gated on post_process_provider_id==='embedded' ([C7] test 11 display-half) [Wave 1]
- [x] 13-15-PLAN.md — Gap closure (i18n): seeded Smart Mode NAME keys translated across all 20 locales ([D8] test 3) [Wave 1]
- [x] 13-16-PLAN.md — Gap closure ([E9] test 7): chip captures via backend handy-keys-event stream on handy_keys to preserve left/right-distinct modifiers (CmdRight) + idempotent resume_all_shortcuts (MODE-04, MODE-05) [Wave 1] — VERIFIED LIVE PASS; new gap [G11] surfaced
- [x] 13-17-PLAN.md — Gap closure ([F10] test 11): provider-switch built + works mechanically, but UX SUPERSEDED → recommendation-only [G12] (TRANS-02) [Wave 1]
- [x] 13-18-PLAN.md — Gap closure ([G11] test 7): backend prefix/base-key overlap rule in find_conflicting_binding — block a combo whose leading key is already bound, surfaced via the existing chip conflict UI (MODE-04, MODE-05) [Wave 1]
- [x] 13-19-PLAN.md — Gap closure ([G12] test 11): revert 13-17 global provider-switch + convert translation engine modal/section to recommendation-only (Gemma recommended; single active model via main selector) + 20-locale i18n (TRANS-01, TRANS-02) [Wave 1]
- [x] 13-20-PLAN.md — Gap closure ([G13] test 7): backend structured conflict error — ConflictKind enum + find_conflicting_binding returns the kind + set_smart_mode_binding emits codified SHORTCUT_CONFLICT payload (no English prose) (MODE-04, MODE-05, L10N-01) [Wave 1]
- [x] 13-21-PLAN.md — Gap closure ([G13] test 7): chip maps SHORTCUT_CONFLICT code → localized t() with distinct base-key message; 2 new conflict keys across 20 locales (English fallback pending 13-23) (MODE-04, MODE-05, L10N-01) [Wave 2]
- [x] 13-22-PLAN.md — Gap closure ([G14]/[G15]): i18n English-fallback audit + check:translations --check-untranslated guardrail + Gemma 3 4B EN-source reword (lead on translation) (L10N-01, TRANS-01, TRANS-02) [Wave 1]
- [x] 13-23-PLAN.md — Gap closure ([G14]/[G15]): translate all smartModes.* + debt keys + about/privacy/ecosystem/acknowledgments into all 19 non-English locales; 0 EN fallbacks; IT common.on/off fixed (L10N-01, TRANS-01, TRANS-02) [Wave 3]
- [ ] 13-24-PLAN.md — Gap closure ([G13]/[G14]/[G15]): automated gates + human-verify live FR-build checkpoint (no English leakage, distinct localized conflict messages, Gemma description) (L10N-01, MODE-05, TRANS-01) [Wave 4]
- [ ] 13-25-PLAN.md — Gap closure ([G16] test 7): SHORTCUT_CONFLICT payload carries conflicting binding id + shared localizeSmartModeName helper so the chip renders the conflicting mode's LOCALIZED name (e.g. Nettoyage, not Clean Up) for exact-duplicate + base-overlap (MODE-04, MODE-05, L10N-01) [Wave 1]
- [ ] 13-26-PLAN.md — Gap closure ([G17] test 7): hoist the conflict error out of the chip's shrink-0 column onto a full-width wrapping card row so the longer localized base-overlap message never overlaps the card title / 'Ajouter un raccourci' placeholder (MODE-05, L10N-01) [Wave 2]

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Bundle Identity | v1.0 | 1/1 | Complete | 2026-04-05 |
| 2. Visual Rebrand | v1.0 | 5/5 | Complete | 2026-04-09 |
| 3. Documentation and Cleanup | v1.0 | 2/2 | Complete | 2026-04-09 |
| 4. Updater Infrastructure | v1.1 | 4/4 | Complete | 2026-04-13 |
| 5. Upstream Sync | v1.1 | 3/3 | Complete | 2026-04-14 |
| 6. Brand & Icon Polish | v1.2 | 4/4 | Complete | 2026-04-16 |
| 7. macOS Clean Shutdown | v1.2 | 1/1 | Complete | 2026-04-23 |
| 8. Privacy / Local-First UX | v1.2 | 10/10 | Complete | 2026-05-22 |
| 9. v1.2 Audit Gap Closure | v1.2 | 1/1 | Complete | 2026-05-28 |
| 10. Prerequisite Gate | 3/3 | Complete    | 2026-06-01 | 2026-06-01 |
| 11. LLM Runtime Foundation | 3/4 | In Progress|  | - |
| 12. Smart Modes Data Layer | 3/3 | Complete    | 2026-06-03 | - |
| 13. Smart Modes UI + Translation + i18n | 26/26 | Complete    | 2026-06-09 | - |
