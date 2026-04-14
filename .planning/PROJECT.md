# Dictus Desktop

## What This Is

Dictus Desktop est l'application desktop officielle de l'écosystème Dictus — une app de speech-to-text locale, cross-platform (macOS/Windows/Linux), privacy-first, construite sur Tauri 2.x (Rust + React/TypeScript). Fork de [Handy](https://github.com/cjpais/Handy), entièrement rebrandé avec l'identité visuelle Dictus, auto-updater signé Ed25519 et sync upstream automatisé (weekly detection + UPSTREAM.md runbook).

## Core Value

L'application doit être identifiable et utilisable comme **Dictus Desktop** — pas comme Handy — et rester vivante (updates automatiques) sans décrocher du upstream Handy.

## Current State

**Shipped:** v1.0 (2026-04-10), v1.1 (2026-04-14)
**App version:** 0.1.0 (first public release, `getdictus/dictus-desktop`)
**Latest tag:** `v1.1`

**v1.1 highlights:**
- Auto-updater end-to-end: Ed25519 signing, GitHub Releases latest.json endpoint, signed v0.1.0 delivered
- Upstream sync automation: weekly detection workflow + UPSTREAM.md runbook + first sync merged (4 commits, capped at `fdc8cb7`)
- Identity integrity validator `verify-sync.sh` (11 assertions) runs pre-PR and in post-sync gate

**v1.1 tech debt carried forward:**
- Post-sync gate missing UPDT-03/UPDT-05 re-assertion (regression risk if a future merge drops pubkey)
- Phase 5 VALIDATION.md left draft — needs `/gsd:validate-phase 5`
- `blob.handy.computer` CDN still used for onnxruntime (INFR-01)

## Next Milestone: TBD

Upcoming candidates:
- Harden post-sync gate (close regression-risk wiring gap from v1.1 audit)
- Privacy/local-first UX audit (todos captured in `.planning/todos/`)
- INFR-01: Dictus-owned CDN for models (replace `blob.handy.computer`)
- TECH-03: Cargo binary rename `handy`→`dictus`
- INFR-03: Code signing (macOS Developer ID verified, Windows Azure Trusted Signing pending)
- SYNC-A1: AI-assisted cherry-pick triage

Scope to be defined via `/gsd:new-milestone`.

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

### Active

(None yet — next milestone pending `/gsd:new-milestone`.)

### Deferred

- [ ] TECH-03: Cargo binary rename handy→dictus
- [ ] TECH-01: Renommage module handy_keys
- [ ] INFR-01: CDN modèles Dictus (remplacer blob.handy.computer)
- [ ] INFR-03: Code signing Dictus (macOS Developer ID verified in v1.1, Windows Azure Trusted Signing pending)
- [ ] SETT-01: Sections settings renommées
- [ ] SYNC-A1: AI-assisted cherry-pick triage (Open Cloud agent)
- [ ] v1.1 post-sync gate hardening (add validate.sh to UPSTREAM.md §6)
- [ ] Privacy/local-first UX audit

### Out of Scope

- Synchronisation mobile ↔ desktop — architecture V4+
- Compte utilisateur / clés cross-device — pas de cloud
- Architecture Nostr / pair-à-pair — recherche ultérieure
- Smart model routing — V2-V3
- Real-time streaming transcription — complexité excessive
- Auto-merge upstream — Dictus rebrand affects same files, human review mandatory
- Delta updates — complexity not justified at current scale

## Context

**Origine :** Fork de [cjpais/Handy](https://github.com/cjpais/Handy) (fork point: commit `85a8ed77`, 13 mars 2026, entre v0.7.11 et v0.8.0). Remote `upstream` configuré.

**Écosystème Dictus :** iOS (`dictus-ios`/`dictus-premium`), Android (`dictus-android`), Desktop, Website (`dictus-website`), Brand kit (`dictus-brand`).

**Stack :** Tauri 2.x, Rust backend (managers pattern), React 18 + TypeScript, Tailwind CSS, Zustand, Vite.

**Upstream state:** 4 commits synced through `fdc8cb7` (Sync #1, 2026-04-14). Remaining upstream delta includes AWS Bedrock commit (`aee682f`) excluded per local-first philosophy — flagged for Sync #2 discussion.

**Codebase (after v1.1):** ~49 files changed during v1.1, +4277 / -517 LOC. Total v1.0+v1.1 change volume around 200 files, 12k+ LOC.

## Constraints

- **Tech stack** : Tauri 2.x + React/TypeScript + Rust — pas de migration
- **Licence** : MIT, fork assumé avec attribution
- **Plateformes** : macOS, Windows, Linux
- **Référence design** : Dictus iOS comme guide visuel
- **Bundle ID** : `com.dictus.desktop`
- **Identity integrity** : productName, identifier, i18n, `X-Title` header must remain `Dictus` through upstream merges — enforced by `verify-sync.sh`
- **Local-first** : cloud providers opt-in only, never prominent (AWS Bedrock excluded from Sync #1)

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
| Raw base64 pubkey in tauri.conf.json (no PEM armor) | Matches `tauri signer generate` output; PEM caused UnexpectedKeyId | ✓ Good |
| `includeUpdaterJson: true` in tauri-action | Auto-generates latest.json per release, no extra CI steps | ✓ Good |
| upstream-sha.txt updated only on merge-to-main | Avoids false idempotency before work done (Pitfall 2) | ✓ Good |
| SYNC-05d awk skip of acknowledgments block | Legitimate Handy attribution must not fail identity scan | ✓ Good |
| UPSTREAM.md at repo root (not docs/) | Max visibility alongside README | ✓ Good |
| Sync #1 capped at `fdc8cb7` (not upstream/main HEAD) | Skip AWS Bedrock commit per local-first; 6 commits deferred | ✓ Good |
| Detection workflow read-only (no auto-commit/PR) | Human review mandatory on same files upstream changes | ✓ Good |
| v0.1.0 Windows builds unsigned (OS level) | Azure Trusted Signing deferred; SmartScreen warning acceptable | ⚠️ Revisit |

---

_Last updated: 2026-04-14 after v1.1 milestone completion_
