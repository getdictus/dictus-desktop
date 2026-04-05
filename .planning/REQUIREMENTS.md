# Requirements: Dictus Desktop V1

**Defined:** 2026-04-05
**Core Value:** L'application doit être identifiable et utilisable comme Dictus Desktop — pas comme Handy.

## v1 Requirements

Requirements for Milestone V1 "Dictus Desktop Foundation". Each maps to roadmap phases.

### Bundle Identity

- [ ] **BNDL-01**: productName changé en "Dictus" dans tauri.conf.json
- [ ] **BNDL-02**: identifier changé en "com.dictus.desktop"
- [ ] **BNDL-03**: Cargo.toml metadata mis à jour (name, description, default-run, lib.name)
- [ ] **BNDL-04**: Auto-updater endpoint désactivé ou mis à jour (plus de référence upstream Handy)
- [ ] **BNDL-05**: Références de code signing upstream supprimées

### Visual Rebrand

- [ ] **VISU-01**: Icône app Dictus générée pour toutes les plateformes (via tauri icon)
- [ ] **VISU-02**: Composants logo remplacés (HandyTextLogo → Dictus, HandyHand → Dictus icon)
- [ ] **VISU-03**: Branding sidebar mis à jour
- [ ] **VISU-04**: Strings i18n mises à jour dans tous les fichiers de locale (20+)
- [ ] **VISU-05**: Design tokens Dictus injectés via Tailwind v4 @theme (palette bleu Dictus, typographie)
- [ ] **VISU-06**: Palette couleurs basculée de rose Handy vers bleu Dictus dans toute l'UI
- [ ] **VISU-07**: Waveform de recording redesignée style Dictus
- [ ] **VISU-08**: Animations et micro-interactions alignées avec l'identité Dictus

### Onboarding

- [ ] **ONBR-01**: Onboarding rebrandé avec textes, visuels et ton Dictus

### Language

- [ ] **LANG-01**: UX force language clarifiée : choix Auto / Français / English présenté clairement

### Documentation

- [ ] **DOCS-01**: README Dictus Desktop (fork assumé, vision, licence MIT, positionnement open-source)
- [ ] **DOCS-02**: CLAUDE.md et BUILD.md mis à jour (références Handy supprimées)
- [ ] **DOCS-03**: Section About in-app rebrandée Dictus

## v2 Requirements

Deferred to future milestones. Tracked but not in current roadmap.

### Internal Technical Rename

- **TECH-01**: Renommage module handy_keys et symboles associés (avec compatibilité serde)
- **TECH-02**: Portable mode marker string mis à jour (backward-compat)
- **TECH-03**: Cargo binary name changé (co-commit avec main.rs)
- **TECH-04**: Audit du patch Tauri (cjpais/tauri.git handy-2.10.2)

### Settings UX

- **SETT-01**: Sections settings renommées (General → Dictation, postprocessing → Smart Modes)
- **SETT-02**: Labels et wording adaptés au ton Dictus dans tous les settings

### Smart Features

- **FEAT-01**: Smart model routing (modèle court / modèle long)
- **FEAT-02**: Modèles recommandés (fast / balanced / accurate)

### Infrastructure

- **INFR-01**: CDN modèles Dictus (remplacer blob.handy.computer)
- **INFR-02**: Endpoint updater Dictus Desktop
- **INFR-03**: Code signing Dictus (macOS + Windows)

## Out of Scope

| Feature | Reason |
|---------|--------|
| Synchronisation mobile ↔ desktop | Architecture V4+, pas de cloud pour V1 |
| Compte utilisateur / clés cross-device | Pas pertinent sans sync |
| Architecture Nostr / pair-à-pair | Recherche ultérieure |
| Redesign pixel-perfect complet | V1 = alignement visuel, pas refonte totale des composants |
| App mobile (Android/iOS native) | Projets séparés existants |
| Real-time streaming transcription | Complexité excessive pour V1 |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| BNDL-01 | — | Pending |
| BNDL-02 | — | Pending |
| BNDL-03 | — | Pending |
| BNDL-04 | — | Pending |
| BNDL-05 | — | Pending |
| VISU-01 | — | Pending |
| VISU-02 | — | Pending |
| VISU-03 | — | Pending |
| VISU-04 | — | Pending |
| VISU-05 | — | Pending |
| VISU-06 | — | Pending |
| VISU-07 | — | Pending |
| VISU-08 | — | Pending |
| ONBR-01 | — | Pending |
| LANG-01 | — | Pending |
| DOCS-01 | — | Pending |
| DOCS-02 | — | Pending |
| DOCS-03 | — | Pending |

**Coverage:**
- v1 requirements: 18 total
- Mapped to phases: 0
- Unmapped: 18 ⚠️

---
*Requirements defined: 2026-04-05*
*Last updated: 2026-04-05 after initial definition*
