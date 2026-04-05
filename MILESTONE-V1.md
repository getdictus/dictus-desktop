# Dictus Desktop — Audit initial + proposition de Milestone V1

> Date: 2026-04-05  
> Base analysée: `cjpais/Handy` → clonée localement dans `projects/dictus-desktop`  
> Références comparées: `projects/dictus-ios`, `projects/dictus-android`, `projects/dictus-website`

---

## Executive summary

La base **Handy** est une très bonne fondation pour **Dictus Desktop**.

Pourquoi :

- le positionnement produit est déjà proche de Dictus (speech-to-text local, open source, privacy-first)
- la stack desktop est moderne et adaptée (Tauri + React/TypeScript + Rust)
- le repo semble suffisamment structuré pour permettre un fork produit sérieux, pas juste un rebrand superficiel
- plusieurs briques utiles pour Dictus existent déjà : modèles, langue sélectionnée, post-processing, historique, raccourcis globaux, overlay, transcription locale

Conclusion :

- **oui, le fork est pertinent**
- **oui, on peut en faire un vrai produit Dictus Desktop cohérent**
- la bonne stratégie est de faire une **Milestone V1 centrée sur rebrand + redesign + alignement produit + quelques fonctionnalités prioritaires Dictus**, sans attaquer tout de suite la synchro multi-device

---

# 1. Branding / éléments à renommer

## Ce qu’on a vu

Le repo source est encore très marqué **Handy** à plusieurs niveaux :

### Identité app / bundle
- `package.json`
  - `name: "handy-app"`
- `src-tauri/Cargo.toml`
  - `name = "handy"`
  - `description = "Handy"`
  - `default-run = "handy"`
  - `lib.name = "handy_app_lib"`
- `src-tauri/tauri.conf.json`
  - `productName = "Handy"`
  - `identifier = "com.pais.handy"`

### Frontend / UI
- `src/components/icons/HandyTextLogo.tsx`
- `src/components/icons/HandyHand.tsx`
- `src/components/Sidebar.tsx`
  - logo Handy injecté dans la sidebar
- nombreuses chaînes i18n mentionnent "Handy"

### Backend / commandes / nomenclature
- `startHandyKeysRecording`
- `stopHandyKeysRecording`
- `KeyboardImplementation = "tauri" | "handy_keys"`
- références internes à Handy dans plusieurs settings / shortcuts / docs

### Docs / metadata
- `README.md`
- `BUILD.md`
- `CLAUDE.md`
- `AGENTS.md`
- mentions du site `handy.computer`
- branding / sponsor assets / release endpoints

## Implication

Le rebrand ne sera **pas juste cosmétique**. Il faudra traiter au moins 4 couches :

1. **branding visible utilisateur**
2. **noms techniques internes**
3. **identité bundle/app**
4. **documentation / communication open source**

## Recommandation

Découper le rebrand en deux sous-phases :

### Phase A — rebrand visible / packaging
Objectif : rendre l’application utilisable comme Dictus Desktop sans casser le code inutilement.

À faire tôt :
- nom app: `Dictus`
- repo / docs: `Dictus Desktop`
- icônes / logos / sidebar
- `productName`
- `identifier` provisoire (ex: `com.pivi.dictus.desktop` ou similaire à valider)
- README initial du fork

### Phase B — rebrand technique interne
À faire plus progressivement :
- renommer les symboles fortement Handy-centric quand c’est utile
- ne pas faire une chasse totale aux renommages internes dans la toute première passe si ça ralentit trop

## Mon avis

Pour une V1 rapide et propre :
- **renommer immédiatement le visible et le packaging**
- **renommer plus tard une partie des internals** si ce n’est pas nécessaire à court terme

---

# 2. Architecture UI / settings

## Ce qu’on a vu

L’app frontend tourne autour d’une interface settings assez classique.

### Entrée principale
- `src/App.tsx`
  - onboarding
  - permissions
  - navigation par sidebar
  - rendu des sections via `SECTIONS_CONFIG`

### Sidebar
Dans `src/components/Sidebar.tsx`, sections actuelles :
- `general`
- `models`
- `advanced`
- `history`
- `postprocessing`
- `debug`
- `about`

### Composants settings intéressants déjà présents
- `LanguageSelector`
- `ModelSelector`
- `CustomWords`
- `PostProcessingToggle`
- `PostProcessingSettings`
- `AudioFeedback`
- `GlobalShortcutInput`
- `HistorySettings`
- `TranslateToEnglish`
- `TypingTool`
- `PasteMethod`
- etc.

## Lecture produit

La base UI actuelle est pensée comme un **outil desktop utilitaire** centré sur :
- config système
- choix de modèles
- raccourcis
- historique
- post-processing

Dictus, lui, a une logique un peu plus **produit / expérience utilisateur**, surtout si on veut aligner avec l’iOS :
- onboarding plus soigné
- branding plus fort
- paramètres orientés usage plutôt qu’orientés tech
- cohérence visuelle avec les apps Dictus existantes

## Écart entre Handy et Dictus

### Handy aujourd’hui
- UX utilitaire / desktop-tool
- beaucoup de réglages exposés
- focus sur le moteur et l’intégration OS

### Dictus cible
- UX plus produit, plus claire, plus “grand public power-user”
- branding et ton plus cohérents avec Dictus iOS
- hiérarchie plus simple des options clés

## Recommandation UI V1

Garder la structure générale Handy au début, mais réorganiser l’expérience :

### Ce qu’on garde
- sidebar
- système de sections
- onboarding existant comme base
- modèles / historique / advanced / about

### Ce qu’on adapte
- section `general` à repenser comme **Dictation** / **Experience**
- `postprocessing` à recadrer comme **Smart Modes** ou **AI Modes** plus tard
- `about` à rebrand Dictus
- onboarding à réaligner sur le ton Dictus
- logos / couleurs / wording / copy

### Ce qu’on pourrait viser ensuite
Une structure plus proche de :
- General / Dictation
- Models
- Shortcuts
- Smart Modes
- History
- Advanced
- About

---

# 3. Gestion des langues et modèles

## Ce qu’on a vu

La bonne surprise, c’est que **Handy a déjà pas mal de briques utiles** pour Dictus.

### Langues
On a déjà :
- `selected_language`
- `change_selected_language_setting(...)`
- logique backend de validation langue/modèle
- `LanguageSelector.tsx`

### Modèles
On a déjà :
- `selected_model`
- gestion de modèles côté frontend et backend
- `ModelsSettings`
- `commands/models.rs`
- `managers/model.rs`
- `managers/transcription.rs`

### Translation / post-processing
On a déjà :
- `translate_to_english`
- `post_process_enabled`
- prompts LLM configurables
- providers / models post-process

## Ce que ça veut dire pour Dictus

Le besoin que tu as cité — **forcer la langue pour Whisper** — est très probablement **beaucoup plus proche que prévu**.

Ce n’est pas une feature totalement nouvelle à inventer. C’est plutôt :
- reprendre l’existant
- vérifier le comportement réel côté transcription manager
- l’adapter au produit Dictus
- clarifier l’UX autour du choix de langue

## Opportunités directes

### Feature 1 — force language proprement
Très bon candidat Milestone V1.

Objectif :
- exposer clairement `Auto / Français / English / ...`
- garantir que le paramètre est bien respecté en transcription
- vérifier compatibilité par modèle
- aligner wording avec Dictus

### Feature 2 — modèles recommandés
On pourrait plus tard introduire une logique Dictus de sélection recommandée, par ex :
- fast
- balanced
- accurate

### Feature 3 — smart model routing
Ton idée “modèle court / modèle long + langue forcée” ressemble à un très bon chantier V1+ ou V2.
Ce n’est pas nécessaire pour le premier fork produit, mais c’est un axe fort.

## Conclusion sur ce point

- la base Handy est **déjà très compatible** avec les ambitions Dictus sur la langue et les modèles
- la feature “forcer la langue” me semble **faisable tôt**
- ce sujet doit clairement faire partie de la Milestone V1

---

# 4. Rapprochement possible avec `dictus-ios`

## Ce qu’on a vu côté iOS

Le repo `dictus-ios` donne déjà plusieurs points de référence forts :

### Produit
- ton privacy-first, open source, on-device
- distinction STT pur vs modes intelligents
- branding Dictus déjà formulé
- README propre et cohérent

### UX / structure produit
Le PRD iOS montre une logique forte :
- onboarding
- model manager
- choix de langue
- approche “modes”
- design assumé
- philosophie simple + utile

### Design / wording
Même si la tech diffère, l’app iOS fournit une excellente base pour :
- naming des features
- ton du produit
- hiérarchie des settings
- storytelling du README

## Ce qu’il ne faut pas faire

Ne pas essayer de copier aveuglément l’iOS.

Desktop ≠ keyboard extension iOS.
Il faut garder les différences structurelles :
- raccourcis globaux desktop
- paste into any app
- tray / overlay / permissions OS
- historique desktop
- input tooling Linux/Wayland/macOS/Windows

## Ce qu’il faut réutiliser

### À reprendre presque tel quel
- vision produit
- ton open source / privacy
- vocabulaire Dictus
- positionnement “free, open-source, on-device”
- réflexion sur les modes intelligents
- logique de langue forcée

### À adapter
- design system et palette
- structure des réglages
- onboarding
- modèles recommandés
- copy marketing / README / about

### À ne pas forcer tout de suite
- synchronisation multi-device
- architecture de compte / clé / chiffrement
- logique mobile-specific

## Conclusion sur ce point

`dictus-ios` est une **référence produit et design**, pas une base technique de portage.  
Pour Dictus Desktop, il faut :
- **s’inspirer fortement de l’expérience et du ton**
- **garder la structure desktop native de Handy**

---

# 5. Proposition de Milestone V1

## Nom proposé

**Milestone V1 — Dictus Desktop foundation**

## Objectif

Transformer le fork Handy en une première base produit **Dictus Desktop** cohérente, installable et prête pour les itérations suivantes.

## Scope de la Milestone V1

### A. Fork cleanup + identité produit
- définir le repo comme **Dictus Desktop**
- rebrand visible de l’application
- renommer `productName`
- préparer l’identifiant bundle final/provisoire
- remplacer les assets/logo les plus visibles
- écrire un README de fork propre :
  - Dictus Desktop
  - basé sur Handy
  - licence MIT conservée
  - vision et périmètre

### B. Audit et alignement UI
- cartographier les sections actuelles
- décider de la première structure UX Dictus Desktop
- adapter les labels et le ton
- identifier les composants à rethématiser en priorité

### C. Langue et transcription
- vérifier le comportement actuel de `selected_language`
- formaliser le mode `Auto / Français / English`
- valider la feature “force language” comme comportement produit attendu
- définir les limitations éventuelles par modèle

### D. Base de design inspirée de Dictus iOS
- extraire les principes visuels et wording de Dictus iOS
- définir un premier niveau d’alignement
  - nommage
  - palette
  - ton
  - structure de l’about / onboarding / settings

### E. Documentation projet
- créer les documents de démarrage du projet desktop
- préparer le terrain pour GSD

## Hors scope explicite

### Pas en V1
- synchronisation mobile ↔ desktop
- architecture Nostr / pair-à-pair / local sync
- compte utilisateur ou gestion de clés cross-device
- gros refactoring backend non nécessaire au fork
- redesign complet pixel-perfect dès la première milestone

## Livrables attendus de la Milestone V1

1. repo `dictus-desktop` proprement initialisé comme fork produit
2. branding visible Dictus au lieu de Handy
3. doc de positionnement et README cohérent
4. décision UX claire pour les sections principales
5. spécification claire de la gestion de langue
6. base suffisamment stable pour lancer une milestone 2 orientée design + features

---

# Recommandation de roadmap courte

## Milestone V1
**Foundation / fork / rebrand / language groundwork**

## Milestone V2
**Design alignment + Dictus-specific desktop UX**
- redesign des écrans clés
- meilleure hiérarchie settings
- alignement plus fort avec Dictus iOS
- polish onboarding / history / about / models

## Milestone V3
**Desktop-specific Dictus features**
- smart model routing
- meilleurs modes intelligents
- améliorations paste / shortcuts / dictation flow

## Milestone V4+
**Cross-device sync research and implementation**
- LAN / local-first / relay-based options
- chiffrement
- sync d’historique / préférences / snippets / etc.

---

# Mon avis final

La stratégie est bonne.

Le plus intelligent maintenant est de considérer `dictus-desktop` comme :
- **un fork produit assumé** de Handy
- **une future branche officielle desktop de Dictus**
- **un chantier séparé de la sync cross-device**

Et la première milestone doit être volontairement pragmatique :
- partir du réel
- clarifier l’identité
- sécuriser la base technique
- rendre la langue / modèles cohérents avec Dictus
- préparer ensuite le vrai travail de design et d’itération

---

# Next step recommandé

Après lecture de ce document, prochaine action recommandée :

1. valider cette Milestone V1
2. préparer le kickoff GSD dans `dictus-desktop`
3. démarrer par une phase de type :
   - map codebase
   - new project / milestone
   - discuss phase V1

Et utiliser `dictus-ios` comme référence produit/design pendant le travail.
