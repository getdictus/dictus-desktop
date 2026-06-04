---
status: complete
phase: 13-smart-modes-ui-translation-presets-i18n
source:
  - 13-01-SUMMARY.md
  - 13-02-SUMMARY.md
  - 13-03-SUMMARY.md
  - 13-04-SUMMARY.md
started: 2026-06-04T08:47:43Z
updated: 2026-06-04T08:47:43Z
note: |
  UAT performed live at the 13-04 human-verify checkpoint (running dev build,
  Whisper Turbo + embedded LLM active). Tests 1 and 5 passed. Tests 2-4 surfaced
  bugs and design gaps. Two design decisions were taken with the user during this
  session and are recorded inline in the relevant gaps:
    - Mode defaults: stop seeding all modes. "+" opens a picker of predefined
      templates (the ~10 current modes) + a "custom" option; seed only "Clean Up"
      on first run so defaults stay recreatable after deletion.
    - Shortcuts: allow single-key bindings (user's choice), but set/edit/delete
      must follow an industry-standard, top-tier UX. Research best practices for
      shortcut capture/edit/delete during gap planning.
---

## Current Test

[testing complete]

## Tests

### 1. Card list — two sections, localized seed names (MODE-06)
expected: Smart Modes render as a vertical card list in "Rewrite" (Clean Up first) and "Translation" sections, not the old single-prompt dropdown; seeded names localize on app language switch
result: pass

### 2. Create / Edit / Delete a mode (MODE-03)
expected: "+ New Rewrite" opens an inline card; fill name + prompt → "Create mode" → card appears; Edit → change name → "Save mode" persists and the card shows the new title; Delete → confirm → card disappears
result: issue
reported: "si je modifie le titre, le titre de la carte une fois sauvegardé n'est pas affiché mais la sauvegarde a bien été prise en compte. Le champ pour le prompt s'appelle 'Libellé du prompt' mais il devrait être tout simplement 'Prompt'"
severity: major

### 3. Shortcut bind + conflict + delete (MODE-04 / MODE-05)
expected: Click a card's shortcut chip, press a combo → chip shows it bound; binding the same combo on another card shows inline conflict and is blocked; an assigned shortcut can be edited and removed; multi-key combos (Cmd+1) bind reliably; pre-update post-process binding ("Option") is migrated/visible, not silently active
reported: |
  Impossible d'assigner un raccourci comme "Option" (utilisé avant la MAJ pour le post-process) : il lance encore la transcription mais rien n'apparaît dans l'UI sur ce raccourci.
  Impossible de mettre un multi-raccourci comme Command+1 : il apparaît puis disparaît.
  Parfois ça marche (j'ai mis 2 et 3 sur trad espagnol/chinois alors que je voulais Cmd+2 / Cmd+3) mais visuellement on dirait qu'il n'y a pas de raccourci.
  Impossible de supprimer un raccourci depuis l'interface. L'UX de gestion des raccourcis est à revoir : un petit rectangle compact à droite de la carte (comme ailleurs dans l'app), pas une grande barre pleine largeur, avec suppression facile.
result: issue
severity: blocker

### 4. Translation modes — engine choice (TRANS-01 / TRANS-02)
expected: 4 Translation cards greyed with "Enable offline translation" CTA; "Choose translation engine" opens the modal with both options; after choosing, translation runs offline; the engine choice can be changed later (active LLM ↔ dedicated Gemma)
reported: "j'ai bien les modes translation [...] mais une fois qu'on a choisi le modèle existant on ne peut plus mettre le modèle Gemma Translate. Il faudrait que l'utilisateur puisse changer le modèle / re-choisir entre le LLM existant choisi plus haut ou télécharger un modèle dédié à la traduction"
severity: major

### 5. No active-mode / primary post-process UI
expected: No "set active" or primary post-process button anywhere; modes fire only via their own shortcuts
result: pass

### 6. Default modes strategy + create-button affordance (design review)
expected: A sensible, discoverable set of modes that the user controls; default Dictus modes remain recreatable after deletion; clear single create affordance
reported: "trop de modes par défaut, pas une bonne idée de les ajouter tous par défaut. Quand on clique sur '+', on devrait choisir parmi les modes prédéfinis (les ~10 actuels) + un mode custom. Si on supprime tout à la main, on ne peut plus les retrouver. Aussi: les boutons d'ajout affichent un double '+' ('+ + New Rewrite' / '+ + New Translation')"
result: issue
severity: major

## Summary

total: 6
passed: 2
issues: 4
pending: 0
skipped: 0

## Gaps

- truth: "Editing a mode's title updates the card's displayed title after Save"
  status: failed
  reason: "User reported: title saved (persisted) but the card still shows the old title — UI does not re-read/re-render after update"
  severity: major
  test: 2
  root_cause: ""
  artifacts:
    - path: "src/components/settings/post-processing/SmartModeCard.tsx"
      issue: "Card display likely not reacting to updated mode state after updateSmartMode"
  missing:
    - "Card must reflect the new name immediately after Save (re-read from store / correct state key)"
  debug_session: ""

- truth: "The prompt input is labeled 'Prompt'"
  status: failed
  reason: "User reported: the prompt field is labeled 'Libellé du prompt' but should simply be 'Prompt'"
  severity: cosmetic
  test: 2
  root_cause: "i18n key used for the prompt field label is the wrong/misleading one"
  artifacts:
    - path: "src/components/settings/post-processing/SmartModeCard.tsx"
      issue: "Prompt field uses a label key meaning 'prompt label' instead of 'Prompt'"
    - path: "src/i18n/locales/en/translation.json"
      issue: "smartModes.* prompt-field label key wording"
  missing:
    - "Relabel the prompt field to 'Prompt' (update the i18n key/value across all 20 locales)"
  debug_session: ""

- truth: "Multi-key shortcut combos (e.g. Cmd+1) bind reliably and persist"
  status: failed
  reason: "User reported: Cmd+1 appears then disappears; user wanted Cmd+2/Cmd+3 but only bare 2/3 stuck"
  severity: blocker
  test: 3
  root_cause: ""
  artifacts:
    - path: "src/components/settings/post-processing/SmartModeShortcutChip.tsx"
      issue: "keydown/keyup capture drops modifier+key combos; combo not committed/persisted"
  missing:
    - "Capture must reliably record modifier+key combos and persist via setSmartModeBinding"
  debug_session: ""

- truth: "An assigned shortcut can be removed from the UI"
  status: failed
  reason: "User reported: cannot delete a shortcut once set; whole shortcut management UX needs rework"
  severity: major
  test: 3
  root_cause: ""
  artifacts:
    - path: "src/components/settings/post-processing/SmartModeShortcutChip.tsx"
      issue: "No clear/remove affordance for a bound shortcut"
  missing:
    - "Add an easy, discoverable way to clear/remove a shortcut"
    - "DECISION: single-key bindings are allowed (user's choice); set/edit/delete must follow industry-standard UX — research best practices during planning"
  debug_session: ""

- truth: "Pre-update post-process binding (Option) is migrated and visible, never silently active"
  status: failed
  reason: "User reported: old 'Option' binding still triggers transcription but is invisible in the new Smart Modes UI"
  severity: major
  test: 3
  root_cause: ""
  artifacts:
    - path: "src-tauri/src/settings.rs"
      issue: "Legacy transcribe_with_post_process / Option binding not migrated or surfaced in new UI"
  missing:
    - "Migrate or surface the legacy post-process shortcut so it is visible and editable (or cleanly removed)"
  debug_session: ""

- truth: "Shortcut control is a compact chip to the right of the card, consistent with the rest of the app"
  status: failed
  reason: "User reported: the full-width shortcut bar is not liked; prefer a small compact rectangle to the right of the card (or detached), matching the app's existing shortcut UI"
  severity: minor
  test: 3
  root_cause: "Current SmartModeShortcutChip renders as a full-width row"
  artifacts:
    - path: "src/components/settings/post-processing/SmartModeShortcutChip.tsx"
      issue: "Full-width layout; restyle to compact chip aligned right"
    - path: "src/components/settings/post-processing/SmartModeCard.tsx"
      issue: "Card layout must place the chip compactly (right-aligned), not as a full-width bar"
  missing:
    - "Redesign shortcut chip as a compact, right-aligned control using the frontend-design skill; align with existing app shortcut UI patterns"
  debug_session: ""

- truth: "Translation engine choice can be changed after the initial selection"
  status: failed
  reason: "User reported: after picking the active model, cannot switch to the dedicated Gemma translation model; user wants to re-choose between active LLM and a downloaded dedicated model anytime"
  severity: major
  test: 4
  root_cause: "TranslationEngineChoiceModal only triggers pre-enable; no path to re-open and change the persisted translation_engine_choice"
  artifacts:
    - path: "src/components/settings/post-processing/TranslationEngineChoiceModal.tsx"
      issue: "No re-entry point to change engine after first choice"
    - path: "src/components/settings/post-processing/SmartModesSection.tsx"
      issue: "Translation group needs an always-available 'change translation engine' affordance"
  missing:
    - "Allow re-opening the engine choice (active LLM ↔ dedicated Gemma download) and updating translation_engine_choice anytime"
  debug_session: ""

- truth: "Default modes are user-chosen from a template picker; defaults stay recreatable; single clear create affordance"
  status: failed
  reason: "User reported: too many modes seeded by default; deleted defaults are unrecoverable; '+' buttons render a double plus ('+ + New Rewrite')"
  severity: major
  test: 6
  root_cause: "All modes seeded by default (Phase 12 data layer); create button text includes a '+' on top of the icon"
  artifacts:
    - path: "src/components/settings/post-processing/SmartModesSection.tsx"
      issue: "Create buttons show double '+'; seeding strategy exposes too many defaults"
    - path: "src-tauri/src/settings.rs"
      issue: "Default seeding currently creates all modes; change to seed only Clean Up"
  missing:
    - "DECISION: seed only 'Clean Up' on first run; '+' opens a picker of predefined templates (the ~10 current modes) + a 'custom' option (custom lets user set name + prompt)"
    - "Predefined templates must remain available so deleted defaults can be recreated"
    - "Fix the double '+' on the create buttons (icon + text duplication)"
  debug_session: ""
