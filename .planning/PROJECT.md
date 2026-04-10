# Dictus Desktop

## What This Is

Dictus Desktop est l'application desktop officielle de l'écosystème Dictus — une app de speech-to-text locale, cross-platform (macOS/Windows/Linux), privacy-first, construite sur Tauri 2.x (Rust + React/TypeScript). Fork de [Handy](https://github.com/cjpais/Handy), entièrement rebrandé avec l'identité visuelle Dictus (palette bleu, logos, overlay waveform, tray icon).

## Core Value

L'application doit être identifiable et utilisable comme **Dictus Desktop** — pas comme Handy.

## Current State

**Shipped:** v1.0 (2026-04-10)
**Commits:** 52 | **Files changed:** 147 | **LOC:** +8390/-761
**Timeline:** 2026-04-05 → 2026-04-10 (6 jours)

Le rebrand Handy→Dictus est complet : bundle identity, visual rebrand (icons, colors, i18n 20 locales, overlay, tray), documentation (README, CLAUDE.md, About panel, Handy acknowledgment).

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

### Active (V2)

- [ ] TECH-03: Cargo binary rename handy→dictus
- [ ] TECH-01: Renommage module handy_keys
- [ ] INFR-01: CDN modèles Dictus (remplacer blob.handy.computer)
- [ ] INFR-02: Endpoint auto-updater Dictus
- [ ] INFR-03: Code signing Dictus (macOS + Windows)
- [ ] SETT-01: Sections settings renommées
- [ ] Upstream sync strategy (issue #1)

### Out of Scope

- Synchronisation mobile ↔ desktop — architecture V4+
- Compte utilisateur / clés cross-device — pas de cloud
- Architecture Nostr / pair-à-pair — recherche ultérieure
- Smart model routing — V2-V3
- Real-time streaming transcription — complexité excessive

## Context

**Origine :** Fork de [cjpais/Handy](https://github.com/cjpais/Handy) (fork point: commit `85a8ed77`, 13 mars 2026, entre v0.7.11 et v0.8.0). Remote `upstream` configuré.

**Écosystème Dictus :** iOS (`dictus-ios`/`dictus-premium`), Android (`dictus-android`), Desktop, Website (`dictus-website`), Brand kit (`dictus-brand`).

**Stack :** Tauri 2.x, Rust backend (managers pattern), React 18 + TypeScript, Tailwind CSS, Zustand, Vite.

**Upstream delta :** 69 commits et 3 releases (v0.8.0-v0.8.2) depuis le fork. Sync strategy documentée dans issue #1.

## Constraints

- **Tech stack** : Tauri 2.x + React/TypeScript + Rust — pas de migration
- **Licence** : MIT, fork assumé avec attribution
- **Plateformes** : macOS, Windows, Linux
- **Référence design** : Dictus iOS comme guide visuel
- **Bundle ID** : `com.dictus.desktop`

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Fork de Handy comme base | Stack moderne, briques utiles présentes | ✓ Good |
| Rebrand en deux passes (visible puis interne) | V1 rapide, renommages internes déférés V2 | ✓ Good |
| com.dictus.desktop comme bundle ID | Format simple, aligné futur domaine | ✓ Good |
| Onboarding : rebrand seulement, pas de refonte | Garder le flow existant, changer textes/visuels | ✓ Good |
| Overlay 84px avec waveform symétrique | Plus visible, identité Dictus distincte de Handy | ✓ Good |
| Tray icon template noir/transparent | macOS auto-tinte, un seul fichier pour les deux thèmes | ✓ Good |
| Binary rename déféré à V2 (TECH-03) | Risque de casser les permissions macOS et scripts | ⚠️ Revisit |
| Auto-updater désactivé | Pas d'endpoint Dictus encore | ⚠️ Revisit |

---

_Last updated: 2026-04-10 after v1.0 milestone_
