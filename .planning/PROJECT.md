# Dictus Desktop

## What This Is

Dictus Desktop est un fork produit de [Handy](https://github.com/cjpais/Handy), transformé en application desktop officielle de l'écosystème Dictus. C'est une app de speech-to-text locale, cross-platform (macOS/Windows/Linux), privacy-first, construite sur Tauri 2.x (Rust + React/TypeScript). L'objectif de cette milestone V1 est de transformer le fork en un produit Dictus cohérent et installable.

## Core Value

L'application doit être identifiable et utilisable comme **Dictus Desktop** — pas comme Handy. Le rebrand complet (visible + packaging + documentation) est la priorité absolue de cette V1.

## Requirements

### Validated

<!-- Fonctionnalités héritées de Handy, déjà opérationnelles -->

- ✓ Transcription speech-to-text locale via Whisper/Parakeet — existing
- ✓ Voice Activity Detection (Silero VAD) — existing
- ✓ Gestion et téléchargement de modèles — existing
- ✓ Sélection de langue pour la transcription — existing
- ✓ Post-processing via LLM (providers configurables) — existing
- ✓ Raccourcis globaux clavier (deux implémentations) — existing
- ✓ Historique des transcriptions — existing
- ✓ Overlay de recording — existing
- ✓ Méthodes de paste multiples (clipboard, input simulation) — existing
- ✓ Audio feedback — existing
- ✓ Custom words (vocabulaire personnalisé) — existing
- ✓ Traduction vers l'anglais — existing
- ✓ Support multi-plateforme macOS/Windows/Linux — existing
- ✓ Internationalisation (en, es, fr, vi) — existing
- ✓ System tray avec contrôles — existing
- ✓ CLI flags (--toggle-transcription, --start-hidden, etc.) — existing

### Active

<!-- Scope V1 — Milestone "Dictus Desktop Foundation" -->

- [ ] Rebrand complet visible : nom, logos, icônes, sidebar → Dictus
- [ ] Identité bundle et packaging : productName, identifier (com.dictus.desktop), Cargo metadata
- [ ] Renommage des assets et composants marqués Handy (HandyTextLogo, HandyHand, etc.)
- [ ] Rebrand de l'onboarding existant (textes, visuels, ton Dictus)
- [ ] Alignement visuel avec Dictus iOS (palette, ton, principes de design)
- [ ] Réorganisation des sections settings (General → Dictation, postprocessing → Smart Modes)
- [ ] Adaptation des labels et wording dans toute l'UI
- [ ] Feature "force language" clarifiée : UX Auto / Français / English, comportement garanti
- [ ] README Dictus Desktop (fork assumé, vision, licence MIT, positionnement)
- [ ] Documentation projet (About rebrandé, positionnement open-source privacy-first)
- [ ] Nettoyage des références Handy dans la documentation (CLAUDE.md, BUILD.md, etc.)
- [ ] Renommage technique interne progressif (handy_keys, startHandyKeysRecording, etc.)

### Out of Scope

- Synchronisation mobile ↔ desktop — chantier V4+, architecture à définir
- Compte utilisateur / gestion de clés cross-device — pas de cloud pour V1
- Architecture Nostr / pair-à-pair / local sync — recherche ultérieure
- Redesign pixel-perfect complet — V1 = alignement visuel, pas refonte totale
- Smart model routing (modèle court/long) — V2-V3
- Modèles recommandés (fast/balanced/accurate) — V2

## Context

**Origine :** Fork de [cjpais/Handy](https://github.com/cjpais/Handy), une app desktop speech-to-text open-source.

**Écosystème Dictus :** Dictus est un produit de transcription privacy-first déjà disponible sur iOS (`dictus-ios`). Le desktop est la deuxième plateforme. L'app iOS sert de **référence produit et design** (palette, ton, naming, UX) mais pas de base technique — le desktop garde sa stack Tauri native.

**Stack existante :** Tauri 2.x, Rust backend (managers pattern), React 18 + TypeScript frontend, Tailwind CSS, Zustand, Vite. Pipeline : Audio → VAD → Whisper/Parakeet → Text → Clipboard/Paste.

**État actuel :** Le code Handy est fonctionnel et structuré. Le branding Handy est présent à 4 niveaux : UI visible, noms techniques internes, identité bundle, documentation. Le rebrand touche ces 4 couches.

## Constraints

- **Tech stack** : Tauri 2.x + React/TypeScript + Rust — on reste sur la stack Handy, pas de migration
- **Licence** : MIT conservée, fork assumé avec attribution
- **Plateformes** : macOS, Windows, Linux — les trois doivent fonctionner en V1
- **Référence design** : Dictus iOS comme guide visuel et produit, pas comme portage technique
- **Bundle ID** : `com.dictus.desktop`
- **Langue forcée** : incluse en V1 uniquement si l'effort reste raisonnable (clarification de l'existant, pas de refactoring lourd)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Fork de Handy comme base | Stack moderne, positionnement proche, briques utiles déjà présentes | — Pending |
| Rebrand en deux passes (visible puis interne) | Ne pas ralentir la V1 avec des renommages internes non critiques | — Pending |
| Dictus iOS comme référence design (pas portage) | Desktop ≠ mobile, garder les spécificités desktop | — Pending |
| com.dictus.desktop comme bundle ID | Format simple, aligné futur domaine | — Pending |
| Onboarding : rebrand seulement, pas de refonte | Garder le flow existant, changer textes/visuels | — Pending |
| Langue forcée conditionnelle en V1 | Si c'est juste clarifier l'existant oui, sinon V2 | — Pending |

---
*Last updated: 2026-04-05 after initialization*
