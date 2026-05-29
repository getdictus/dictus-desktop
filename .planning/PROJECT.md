# Dictus Desktop

## What This Is

Dictus Desktop est l'application desktop officielle de l'écosystème Dictus — une app de speech-to-text **locale, cross-platform (macOS/Windows/Linux), privacy-first**, construite sur Tauri 2.x (Rust + React/TypeScript). Fork de [Handy](https://github.com/cjpais/Handy), entièrement rebrandé avec l'identité visuelle Dictus, auto-updater signé Ed25519, sync upstream cadencé, et UI post-processing qui rend la promesse locale lisible (Local/Cloud tabs, défaut Apple Intelligence sur macOS arm64, `docs/PRIVACY.md`).

## Core Value

L'application doit être identifiable et utilisable comme **Dictus Desktop** — pas comme Handy — rester vivante (updates automatiques) sans décrocher du upstream Handy, **et présenter le local-first comme défaut visible** dans l'UX (pas seulement dans la doc).

## Current State

**GSD milestones shipped:** v1.0 (2026-04-10), v1.1 (2026-04-14), v1.2 (2026-05-29)
**App version:** 0.1.0 (first public release, `getdictus/dictus-desktop`)
**Latest app release tag:** `v0.1.0`

> GSD milestones (`v1.0`, `v1.1`, `v1.2`) are internal planning units — no git tags.
> App versions (`0.x.y`) are the only tagged entities. See `docs/VERSIONING.md`.

**v1.2 highlights:**
- Brand cleanup complete — `dictus-*.wav` filenames, "Dictus Portable Mode" marker, runtime app-data path in DebugPaths, verify-sync.sh extended (15 assertions) at `.github/scripts/`
- Platform icons regenerated from 1024×1024 opaque navy-tile source — Linux black-corners artifact impossible, Windows ICO has all 6 required layers, tauri.conf.json bundle.icon enumerates 7 entries
- macOS clean shutdown — `flush_and_exit` helper releases CGEventTap on main runloop before `app.exit(0)`; multi-day no-crash validation
- Privacy / local-first UX — platform-aware default (Apple Intelligence on macOS arm64, Custom (local) elsewhere), Local/Cloud tabs replace cloud opt-in toggle, `docs/PRIVACY.md` documents network surface, 25 i18n keys propagated across 19 sibling locales
- v1.2 audit gap closure — clippy gate green (33 errors resolved across 14 files), retroactive 07-VERIFICATION.md authored, 08-UAT promoted to passed

**v1.2 tech debt carried forward:**
- 4 phase VALIDATION.md files still `status: draft, nyquist_compliant: false` — Nyquist sampling map never finalised; `/gsd:validate-phase 6` (and 7/8/9) to close retroactively
- **TECH-04** — `llm_client.rs:137 send_chat_completion_with_schema` 8-arg refactor deferred (suppressed via `#[allow(clippy::too_many_arguments)]`)
- Pre-existing lint failure on `src/components/icons/DictusLogo.tsx` (i18next/no-literal-string on hardcoded `Dictus` SVG `<text>`)
- macOS quit-unexpectedly crash non-reproducible under clean env post-fix (memory `project_macos_quit_crash_investigation.md`) — fix is defensive

**Carried from v1.1:**
- `blob.handy.computer` CDN for onnxruntime still in use (INFR-01)
- v0.1.0 Windows builds unsigned at OS level (INFR-03, Azure Trusted Signing pending)
- Phase 5 VALIDATION.md left draft — needs `/gsd:validate-phase 5`

## Current Milestone: v1.3 Smart Modes & Local LLM

**Goal:** Faire du traitement local post-transcription un défaut réel et puissant — un runtime LLM embarqué (toutes plateformes, sans Ollama externe) plus des « Smart Modes » (prompts soignés, éditables, créables, chacun associable à un shortcut), dont la traduction multi-cibles devient un mode first-class.

**Target features:**
- **Runtime LLM local embarqué** — moteur LLM en Rust intégré (llama.cpp/candle/mistral.rs), téléchargeur de modèles intégré (UX calquée sur le picker Whisper/Parakeet), détection GPU par plateforme (Metal + Vulkan, CUDA optionnel), gestion mémoire/cycle de vie (unload timeout, presets de quantization). Devient l'option locale principale ; la carte placeholder « Bibliothèque de modèles locaux » devient réelle.
- **Smart Modes** — série de prompts post-transcription soignés livrés par défaut, éditables, l'utilisateur peut créer ses propres modes, chacun associable à son propre shortcut (extension du modèle prompt↔shortcut actuel + amélioration UX/UI).
- **Traduction first-class** — presets multi-cibles (EN/ES/ZH/FR…), chacun un Smart Mode bindable à un shortcut.

**Scope decisions (2026-05-29):**
- Runtime embarqué livré sur **toutes les plateformes** dès v1.3 (Metal/Vulkan, pas macOS-first).
- Apple Foundation + Custom→Ollama + providers cloud **restent tous** en option ; l'embarqué s'ajoute comme option locale principale (ne remplace rien).
- Smart Modes = prompts (pas un nouveau concept opaque) — défauts soignés + édition + création + binding shortcut.

**Phases continue from Phase 10** (v1.2 ended at Phase 9).

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

### Active

<!-- v1.3 Smart Modes & Local LLM — detailed REQ-IDs in .planning/REQUIREMENTS.md -->

- [ ] Runtime LLM local embarqué (moteur Rust + téléchargeur intégré + GPU detection + gestion mémoire) — toutes plateformes
- [ ] Smart Modes : prompts post-transcription soignés, éditables, créables, chacun associable à un shortcut
- [ ] Traduction first-class : presets multi-cibles bindables à des shortcuts

### Deferred

- [ ] **TECH-01** — Renommage module `handy_keys` (handy-keys external crate must NOT be renamed)
- [ ] **TECH-03** — Cargo binary rename `handy`→`dictus` (defers macOS permission/scripts risk)
- [ ] **TECH-04** — `llm_client.rs:137 send_chat_completion_with_schema` 8-arg → struct refactor (added v1.2)
- [ ] **INFR-01** — CDN modèles Dictus (replace `blob.handy.computer`)
- [ ] **INFR-03** — Windows Azure Trusted Signing setup
- [ ] **DATA-01** — On-disk data directory migration (`handy` → `dictus`); requires backup logic
- [ ] **SETT-01** — Sections settings renommées
- [ ] **SYNC-A1** — AI-assisted cherry-pick triage on larger upstream deltas
- [ ] v1.1 post-sync gate hardening (add validate.sh to UPSTREAM.md §6)
- [ ] Nyquist VALIDATION.md closure for phases 5, 6, 7, 8, 9 (5 files in draft state)

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
| TECH-04 (`llm_client.rs:137` 8-arg refactor) deferred | Signature change risk exceeds v1.2 window; suppressed via `#[allow]` | ⚠️ Revisit |
| `07-VERIFICATION.md` retroactively authored from SUMMARY + Pierre observation | Phase 7 closed without VERIFICATION.md; Phase 9 backfilled to clear audit trail | ✓ Good |
| Phase 9 scoped to AUDIT-01/02/03 only (not Nyquist closure) | Surgical 60-min phase; Nyquist VALIDATION.md drafts deferred to `/gsd:validate-phase` | ✓ Good |

---

*Last updated: 2026-05-29 after starting milestone v1.3 Smart Modes & Local LLM*
