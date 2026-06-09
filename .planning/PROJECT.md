# Dictus Desktop

## What This Is

Dictus Desktop est l'application desktop officielle de l'écosystème Dictus — une app de speech-to-text **locale, cross-platform (macOS/Windows/Linux), privacy-first**, construite sur Tauri 2.x (Rust + React/TypeScript). Fork de [Handy](https://github.com/cjpais/Handy), entièrement rebrandé avec l'identité visuelle Dictus, auto-updater signé Ed25519, sync upstream cadencé, et UI post-processing qui rend la promesse locale lisible (Local/Cloud tabs, défaut Apple Intelligence sur macOS arm64, `docs/PRIVACY.md`).

## Core Value

L'application doit être identifiable et utilisable comme **Dictus Desktop** — pas comme Handy — rester vivante (updates automatiques) sans décrocher du upstream Handy, **et présenter le local-first comme défaut visible** dans l'UX (pas seulement dans la doc).

## Current State

**GSD milestones shipped:** v1.0 (2026-04-10), v1.1 (2026-04-14), v1.2 (2026-05-29), v1.3 (2026-06-09)
**App version:** 0.1.0 (first public release, `getdictus/dictus-desktop`)
**Latest app release tag:** `v0.1.0`

> GSD milestones (`v1.0`…`v1.3`) are internal planning units — no git tags.
> App versions (`0.x.y`) are the only tagged entities. See `docs/VERSIONING.md`.
> v1.3 code is on `feat/v1.3-smart-modes`; an app-version release (e.g. `0.2.0`) is a separate follow-up step, not produced by milestone archival.

**v1.3 highlights (shipped 2026-06-09):**
- Embedded LLM runtime on all platforms — in-process GGUF via `llama-cpp-2` (no external Ollama), GPU auto-select (Metal embedded / Vulkan / CPU fallback), background-thread inference, idle unload coexisting with the transcription model; ggml duplicate-symbol conflict resolved via linker keep-first-definition; CI green on all 7 platforms
- Functional local model library — curated 4-model catalogue (Qwen2.5-1.5B, Gemma-3-4B, Phi-4-Mini, Llama-3.2-3B) with size-before-download, in-app download/cancel/resume + SHA256 from HuggingFace CDN, delete-to-reclaim, custom GGUF drag/drop; placeholder card replaced by the real library
- Smart Modes — lossless v1.2→v1.3 prompt migration, visual card list replacing the single-prompt dropdown, create/edit/delete in-panel, per-mode global shortcut with structured fully-localized conflict detection
- First-class offline translation — runs through the active embedded LLM; TranslateGemma dropped after A/B benchmark (Gemma-3-4B leads); recommendation-only engine; whatlang output-language directive
- Full 20-locale localization — reversed the "names-only" deferral; all new strings truly translated; `--check-untranslated` guardrail added

**v1.3 tech debt carried forward:**
- Phases 10 & 11 VALIDATION.md `status: draft` — `/gsd:validate-phase 10`/`11` to close (adds to the v1.1/v1.2 Nyquist backlog)
- Phase 11 Windows/Linux runtime GPU smoke not human-tested (CI build/link green, user-accepted)
- Doc wording drift recorded in `milestones/v1.3-REQUIREMENTS.md`: MDL-01 catalogue (3 named → 4 shipped), MODE-02 seed count (10 curated, 1 seeded)
- `drop_non_drop` clippy warning at `managers/llm.rs:1221` (pre-existing); AMD Vulkan driver crash to monitor (llama.cpp #17432) before Windows beta

<details>
<summary>Earlier milestone highlights & carried debt (v1.0–v1.2)</summary>

**v1.2 highlights:** brand cleanup (`dictus-*.wav`, Portable Mode marker, verify-sync.sh 15 assertions); platform icons regenerated (opaque navy tile, 6-layer Windows ICO); macOS clean shutdown (`flush_and_exit` releases CGEventTap before exit); privacy/local-first UX (Local/Cloud tabs, platform-aware default, `docs/PRIVACY.md`, 25 i18n keys); audit gap closure (clippy gate green, retroactive 07-VERIFICATION).

**v1.2 tech debt:** 4 phase VALIDATION.md `draft`; TECH-04 (resolved in v1.3 PREP-02); `DictusLogo.tsx` i18next lint.

**Carried from v1.1:** `blob.handy.computer` CDN for onnxruntime (INFR-01); v0.1.0 Windows builds unsigned (INFR-03, Azure Trusted Signing pending); Phase 5 VALIDATION.md draft. macOS quit-unexpectedly crash non-reproducible under clean env post-fix (defensive).

</details>

## Next Milestone: TBD

v1.3 is shipped and archived. The next milestone is not yet defined — run `/gsd:new-milestone` to gather context, research, and define fresh requirements (it will create a new `.planning/REQUIREMENTS.md`).

**Candidate threads for next milestone** (not committed — surface during `/gsd:new-milestone`):
- Default Smart Mode prompt quality tuning (user noted output not fully matching intent; tracked in pending todos + memory `smart_mode_prompt_tuning`)
- Close the Nyquist VALIDATION.md backlog (phases 5–11) and Phase 11 runtime GPU smoke on Windows/Linux
- An app-version release (`0.2.0`) shipping the v1.3 feature set
- Model-library polish deferred to Future Requirements (MDL-F1..F6) and Smart Mode extensions (MODE-F1..F5)

## Requirements

### Validated

- ✓ Transcription speech-to-text locale via Whisper/Parakeet — existing
- ✓ Voice Activity Detection (Silero VAD) — existing
- ✓ Gestion et téléchargement de modèles — existing
- ✓ Support multi-plateforme macOS/Windows/Linux — existing
- ✓ Internationalisation (20 locales) — existing + v1.0
- ✓ System tray avec contrôles — existing
- ✓ CLI flags — existing
- ✓ Bundle identity com.dictus.desktop — v1.0
- ✓ Rebrand complet visible (logos, icônes, palette bleu) — v1.0
- ✓ i18n Handy→Dictus dans toutes les locales — v1.0
- ✓ Overlay waveform redesigné (84px pill, centre→bords) — v1.0
- ✓ Tray icon template macOS — v1.0
- ✓ Onboarding rebrandé Dictus — v1.0
- ✓ README Dictus Desktop avec fork attribution — v1.0
- ✓ About panel rebrandé avec Handy acknowledgment — v1.0
- ✓ Auto-updater Ed25519 keypair + GitHub Secrets (UPDT-01, UPDT-02) — v1.1
- ✓ tauri.conf.json updater config: pubkey + endpoint + createUpdaterArtifacts (UPDT-03, UPDT-04, UPDT-05) — v1.1
- ✓ CI asset-prefix `dictus` + includeUpdaterJson (UPDT-06, UPDT-07, UPDT-08) — v1.1
- ✓ UpdateChecker.tsx fallback URL → getdictus/dictus-desktop (UPDT-09) — v1.1
- ✓ v0.1.0 dry-run release validates latest.json accessible (UPDT-10) — v1.1
- ✓ Weekly `upstream-sync.yml` detection + idempotent issue creation (SYNC-01, SYNC-02) — v1.1
- ✓ UPSTREAM.md fork-point-aware merge runbook (SYNC-03) — v1.1
- ✓ First upstream merge 4 commits onto main (SYNC-04) — v1.1
- ✓ Post-merge identity checklist via verify-sync.sh (SYNC-05) — v1.1
- ✓ Recording filenames `dictus-*.wav` (BRAND-01) — v1.2
- ✓ Portable mode marker "Dictus Portable Mode" (BRAND-02) — v1.2
- ✓ DebugPaths runtime path via `commands.getAppDirPath()` (BRAND-03) — v1.2
- ✓ verify-sync.sh extended with BRAND-01a/02a/03a assertions (BRAND-04) — v1.2
- ✓ Linux opaque PNG icon — no black-corners artifact possible (ICON-01) — v1.2
- ✓ Windows ICO with 6 required layers (16/24/32/48/64/256) (ICON-02) — v1.2
- ✓ tauri.conf.json bundle.icon array with 7 entries (ICON-03) — v1.2
- ✓ 1024×1024 RGBA source PNG in `dictus-brand` (ICON-04) — v1.2
- ✓ verify-sync.sh relocated to `.github/scripts/` (SYNC-06) — v1.2
- ✓ macOS crash diagnosis committed before fix (SHUT-01) — v1.2
- ✓ `flush_and_exit` helper at both exit sites (SHUT-02) — v1.2
- ✓ No "Dictus quit unexpectedly" dialog on tray quit / post-update relaunch (SHUT-03) — v1.2
- ✓ Local/Cloud tabs with platform-aware default (PRIV-01) — v1.2
- ✓ `docs/PRIVACY.md` documents outbound endpoints (PRIV-02) — v1.2
- ✓ Onboarding presents local transcription as primary path (PRIV-03) — v1.2
- ✓ `cargo clippy --all-targets -- -D warnings` exits 0 (AUDIT-01) — v1.2
- ✓ Retroactive 07-VERIFICATION.md with `status: passed` (AUDIT-02) — v1.2
- ✓ 08-UAT.md frontmatter promoted to `status: passed` (AUDIT-03) — v1.2
- ✓ `llama-cpp-2` coexists with `transcribe-rs` on all 7 CI platforms — ggml conflict resolved via linker keep-first (PREP-01) — v1.3
- ✓ TECH-04 resolved — `send_chat_completion_with_schema` 8-arg → request struct, `#[allow]` dropped (PREP-02) — v1.3
- ✓ Upstream Sync #2 merged, selective cherry-pick fork policy, Bedrock `aee682f` exclusion logged (PREP-03) — v1.3
- ✓ In-process GGUF runtime on background thread, no external Ollama (LLM-01) — v1.3
- ✓ Per-platform GPU auto-select (Metal/Vulkan) + CPU fallback (LLM-02) — v1.3 (Win/Linux runtime smoke = carried debt)
- ✓ LLM idle-timeout unload coexisting with transcription model (LLM-03) — v1.3
- ✓ "Embedded (local)" selectable post-process provider, cloud stays opt-in (LLM-04) — v1.3
- ✓ Curated catalogue with size-before-download (MDL-01) — v1.3 (shipped 4 generic-instruct models, not the 3 originally named)
- ✓ In-app download w/ progress/cancel/resume, SHA256, HuggingFace CDN (MDL-02) — v1.3
- ✓ Delete downloaded model, reclaim disk (MDL-03) — v1.3
- ✓ Custom GGUF by drag/drop or file-pick (MDL-04) — v1.3
- ✓ Placeholder card → real functional model library, top-of-page (MDL-05) — v1.3
- ✓ Lossless v1.2→v1.3 prompt migration, `settings_schema_version`, fixture-tested (MODE-01) — v1.3
- ✓ Curated Smart Modes shipped + safe Clean Up default (MODE-02) — v1.3 (10 curated, 1 seeded + 9 via picker)
- ✓ Create / edit / delete Smart Modes (name + prompt + optional target language) (MODE-03) — v1.3
- ✓ Distinct global shortcut per Smart Mode, applies that mode's prompt (MODE-04) — v1.3
- ✓ Shortcut conflicts detected + inline localized warning at bind time (MODE-05) — v1.3
- ✓ Smart Modes as a visual card list replacing the dropdown (MODE-06) — v1.3
- ✓ Translation as a first-class bindable Smart Mode, multi-target (TRANS-01) — v1.3
- ✓ Translation runs fully offline through the embedded LLM (TRANS-02) — v1.3
- ✓ All new strings localized across 20 locales, `check:translations` 0 errors (L10N-01) — v1.3

### Active

_None — next milestone not yet defined. Run `/gsd:new-milestone` to populate. See "Next Milestone: TBD" above for candidate threads._

### Deferred

- [ ] **TECH-01** — Renommage module `handy_keys` (handy-keys external crate must NOT be renamed)
- [ ] **TECH-03** — Cargo binary rename `handy`→`dictus` (defers macOS permission/scripts risk)
- ✓ **TECH-04** — `send_chat_completion_with_schema` 8-arg → request struct — **resolved v1.3 (PREP-02)**
- [ ] **INFR-01** — CDN modèles Dictus (replace `blob.handy.computer` for onnxruntime; LLM weights already on HuggingFace CDN as of v1.3)
- [ ] **INFR-03** — Windows Azure Trusted Signing setup
- [ ] **DATA-01** — On-disk data directory migration (`handy` → `dictus`); requires backup logic
- [ ] **SETT-01** — Sections settings renommées
- ✓ **SYNC-A1** — AI-assisted cherry-pick triage — **exercised v1.3 (PREP-03 Upstream Sync #2)**; policy now selective cherry-pick going forward
- [ ] v1.1 post-sync gate hardening (add validate.sh to UPSTREAM.md §6)
- [ ] Nyquist VALIDATION.md closure for phases 5–11 (draft state: 5, 6, 7, 8, 9, 10, 11)
- [ ] Phase 11 Windows/Linux runtime GPU smoke (download → load → embedded inference)
- [ ] Default Smart Mode prompt quality tuning (output not fully matching intent — user feedback v1.3)
- [ ] App-version release (`0.2.0`) shipping the v1.3 feature set

### Out of Scope

- Synchronisation mobile ↔ desktop — architecture V4+
- Compte utilisateur / clés cross-device — pas de cloud
- Architecture Nostr / pair-à-pair — recherche ultérieure
- Smart model routing — V2-V3
- Real-time streaming transcription — complexité excessive
- Auto-merge upstream — Dictus rebrand affects same files, human review mandatory
- Delta updates — complexity not justified at current scale
- Per-item modal warning on cloud provider selection — section-level labeling is sufficient (decided v1.2)
- Hiding cloud providers entirely — opt-in, not invisible (decided v1.2)
- Replacing `handy-keys` external crate — build-breaking; out of scope

## Context

**Origine :** Fork de [cjpais/Handy](https://github.com/cjpais/Handy) (fork point: commit `85a8ed77`, 13 mars 2026, entre v0.7.11 et v0.8.0). Remote `upstream` configuré.

**Écosystème Dictus :** iOS (`dictus-ios`/`dictus-premium`), Android (`dictus-android`), Desktop, Website (`dictus-website`), Brand kit (`dictus-brand`).

**Stack :** Tauri 2.x, Rust backend (managers pattern), React 18 + TypeScript, Tailwind CSS, Zustand, Vite.

**Upstream state:** 4 commits synced through `fdc8cb7` (Sync #1, 2026-04-14). Remaining upstream delta includes AWS Bedrock commit (`aee682f`) excluded per local-first philosophy — flagged for Sync #2 discussion.

**Codebase (after v1.2):** ~178 files changed during v1.2, +16,160 / -701 LOC. Total v1.0+v1.1+v1.2 change volume around 378 files, ~28k LOC delta.

**User feedback themes from v1.2:**
- Cloud opt-in toggle felt heavyweight relative to user mental model — replaced with Local/Cloud tabs (UAT pivot 2026-05-22)
- Three-pillar marketing block (Confidentialité / Contrôle / Expérience) was perceived as marketing copy, not utility — removed
- Library "coming soon" placeholder should be top-of-page anchor, not footer — hoisted
- macOS quit-unexpectedly dialog non-reproducible under clean env after fix — possibly multi-install pollution, see memory `project_macos_quit_crash_investigation.md`

## Constraints

- **Tech stack** : Tauri 2.x + React/TypeScript + Rust — pas de migration
- **Licence** : MIT, fork assumé avec attribution
- **Plateformes** : macOS, Windows, Linux
- **Référence design** : Dictus iOS comme guide visuel
- **Bundle ID** : `com.dictus.desktop`
- **Identity integrity** : productName, identifier, i18n, `X-Title` header must remain `Dictus` through upstream merges — enforced by `verify-sync.sh` (15 assertions)
- **Local-first** : cloud providers opt-in only, never prominent/default — enforced UI-side by platform-aware default + Local/Cloud tabs; documented user-side by `docs/PRIVACY.md`

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Fork de Handy comme base | Stack moderne, briques utiles présentes | ✓ Good |
| Rebrand en deux passes (visible puis interne) | V1 rapide, renommages internes déférés V2 | ✓ Good |
| com.dictus.desktop comme bundle ID | Format simple, aligné futur domaine | ✓ Good |
| Onboarding : rebrand seulement, pas de refonte | Garder le flow existant, changer textes/visuels | ✓ Good |
| Overlay 84px avec waveform symétrique | Plus visible, identité Dictus distincte de Handy | ✓ Good |
| Tray icon template noir/transparent | macOS auto-tinte, un seul fichier pour les deux thèmes | ✓ Good |
| Binary rename déféré V2+ (TECH-03) | Risque casser permissions macOS et scripts | ⚠️ Revisit |
| Wave 0 pattern (validate.sh FAIL-first, plans make green) | Feedback loop from first commit of downstream plans | ✓ Good |
| Triple-backup Ed25519 key (Bitwarden×2 + iCloud age -p) | Defense in depth, no single point of loss | ✓ Good |
| Raw base64 pubkey in tauri.conf.json (no PEM armor) | Matches `tauri signer generate` output | ✓ Good |
| `includeUpdaterJson: true` in tauri-action | Auto-generates latest.json per release | ✓ Good |
| upstream-sha.txt updated only on merge-to-main | Avoids false idempotency before work done | ✓ Good |
| SYNC-05d awk skip of acknowledgments block | Legitimate Handy attribution must not fail identity scan | ✓ Good |
| UPSTREAM.md at repo root (not docs/) | Max visibility alongside README | ✓ Good |
| Sync #1 capped at `fdc8cb7` (not upstream/main HEAD) | Skip AWS Bedrock commit per local-first | ✓ Good |
| Detection workflow read-only (no auto-commit/PR) | Human review mandatory on same files upstream changes | ✓ Good |
| v0.1.0 Windows builds unsigned (OS level) | Azure Trusted Signing deferred | ⚠️ Revisit |
| BRAND-02 single-direction marker (no Handy legacy dual-read) | Pierre is sole user, no installed base | ✓ Good |
| ICON-01 opaque navy tile (not transparent) | Black-corners artifact physically impossible | ✓ Good |
| Linux/Windows visual rendering deferred | Automated backstops accepted; "on corrigera ca plus tard" | ⚠️ Revisit |
| Path (a) graceful CGEventTap cleanup over `std::process::exit` | Diagnosis pointed at `tauri-plugin-global-shortcut` Drop on main thread; fix releases CGEventTap while runloop is alive | ✓ Good |
| `simulate_updater_restart` UI-gated by `settings.debug_mode` (not `#[cfg(debug_assertions)]`) | Production installs can validate updater-relaunch path | ✓ Good |
| Phase 7 closed without second-reproduction checkpoint | Crash non-reproducible under clean env post-fix; multi-day observation sufficed | ⚠️ Revisit |
| `docs/PRIVACY.md` as single source of truth (no in-app Privacy page) | Markdown is editable, indexed, version-controlled | ✓ Good |
| Platform-aware default (Apple Intelligence on macOS arm64 only) | Local-first by default on capable platform | ✓ Good |
| Custom provider id stays `custom` (label changes to "Custom (local)") | Persisted settings stability | ✓ Good |
| Local/Cloud tabs replace cloud opt-in toggle | Tabs more discoverable + structurally separate (UAT pivot 2026-05-22) | ✓ Good |
| Three-pillar marketing block removed | User feedback: felt like marketing not utility | ✓ Good |
| Library "coming soon" placeholder hoisted to top | Anchors local-first narrative immediately | ✓ Good |
| `enable_cloud_providers` Rust field kept vestigial | Avoid schema migration; UI no longer reads/writes | ⚠️ Revisit |
| TECH-04 (`llm_client.rs:137` 8-arg refactor) deferred | Signature change risk exceeds v1.2 window; suppressed via `#[allow]` | ✓ Resolved v1.3 (PREP-02) |
| `07-VERIFICATION.md` retroactively authored from SUMMARY + Pierre observation | Phase 7 closed without VERIFICATION.md; Phase 9 backfilled to clear audit trail | ✓ Good |
| Phase 9 scoped to AUDIT-01/02/03 only (not Nyquist closure) | Surgical 60-min phase; Nyquist VALIDATION.md drafts deferred to `/gsd:validate-phase` | ✓ Good |
| `llama-cpp-2` as the embedded engine (over candle / mistral.rs) | In-process GGUF, mature Metal/Vulkan; fallback not needed | ✓ Good |
| ggml duplicate-symbol resolved via linker keep-first-definition (`--allow-multiple-definition` / `/FORCE:MULTIPLE`) | macOS ld64 already did this implicitly; whisper's ggml (linked first) wins shared symbols | ⚠️ Revisit (runtime correctness not CI-validated) |
| Metal shaders embedded (`GGML_METAL_EMBED_LIBRARY=ON`), no `.metallib` bundling | No separate file at runtime; `tauri.conf.json` unchanged | ✓ Good |
| Windows x64 needs `CMAKE_GENERATOR=Ninja` for `vulkan-shaders-gen` | MSVC MSBuild ExternalProject fails; Ninja pre-installed on GHA | ✓ Good |
| Catalogue = 4 generic-instruct models (dropped Qwen3-4B reasoning + TranslateGemma) | Generic instruct models suit short post-process; reasoning/specialized do not (runtime-verified) | ✓ Good |
| TranslateGemma dropped after A/B benchmark | Same-size generic (Gemma-3-4B) matched/beat it, far fewer polluted outputs | ✓ Good |
| Translation engine = recommendation-only (one active model, no per-mode switcher) | Simpler mental model; per-mode model deferred to MODE-F3 | ✓ Good |
| Seed only Clean Up on first run; 9 other curated modes via template picker | Safe default, avoids overwhelming the modes list (13-05 UAT) | ✓ Good |
| Smart Mode shortcut = `smart_mode_{id}` binding prefix wired through coordinator/actions/init | Reuses existing prompt↔shortcut machinery; no new opaque concept | ✓ Good |
| Structured `SHORTCUT_CONFLICT` payload (code+params) rendered via frontend `t()` | No English prose crosses the boundary; fully localizable conflict errors | ✓ Good |
| Reversed "names-only" i18n deferral — full 20-locale translation | English fallback values were leaking in non-EN builds (live FR UAT) | ✓ Good |
| `--toggle-post-process` / SIGUSR1 re-routed to active Smart Mode | Legacy binding fired a frozen pre-v1.3 prompt snapshot post-migration | ✓ Good |
| whatlang output-language directive placed first in the prompt | Small LLMs answer in the instruction language; trailing placement echoed into output | ✓ Good |

---

*Last updated: 2026-06-09 after completing milestone v1.3 Smart Modes & Local LLM*
