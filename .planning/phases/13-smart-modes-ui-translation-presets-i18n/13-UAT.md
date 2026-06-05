---
status: complete
phase: 13-smart-modes-ui-translation-presets-i18n
source:
  - 13-01-SUMMARY.md
  - 13-02-SUMMARY.md
  - 13-03-SUMMARY.md
  - 13-04-SUMMARY.md
  - 13-05-SUMMARY.md
  - 13-06-SUMMARY.md
  - 13-07-SUMMARY.md
started: 2026-06-05T12:53:42Z
updated: 2026-06-05T12:53:42Z
note: |
  Fresh full re-test (user choice). The first UAT was run at the 13-04 checkpoint
  and found 4 issues / 8 gaps; plans 13-05, 13-06, 13-07 were written to close them.
  Code-inspectable items (i18n parity, field labels, double-plus, clear affordance
  presence, legacy global-shortcut retirement) are auto-verified with evidence and
  marked pass. App-interaction tests are presented to the user one at a time.
---

## Current Test

[testing complete]

## Tests

### 1. i18n parity — all 20 locales (L10N-01)
expected: All smartModes.* keys (sections, card labels, picker.*, shortcut.*, translation.*) present in en + 19 locales; check:translations passes at full key count
result: pass
note: Auto-verified — `bun run check:translations` green ("All 19 languages have complete translations"); en has smartModes.shortcut.{clear,clearAriaLabel,recording}, picker.{rewriteTitle,translationTitle,custom,customSubtext,added}, translation.changeEngine

### 2. Field labels + single create "+" (code-level)
expected: Prompt field labeled "Prompt" (not "Libellé du prompt"); name field "Name"; create buttons show a single Plus icon, no double "++"
result: pass
note: Auto-verified — card.promptLabel="Prompt", card.nameLabel="Name", createRewrite="New Rewrite"/createTranslation="New Translation" (no leading "+"); SmartModesSection renders one <Plus> icon + plain text. Visual confirmation folded into Test 4.

### 3. Card list layout + localized seed names (MODE-06)
expected: Smart Modes render as a vertical card list with "Rewrite" (Clean Up first) + "Translation" sections, not the old dropdown; seeded names re-localize on app language switch
result: pass

### 4. Create mode via template picker (MODE-03 / MODE-06)
expected: Clicking "New Rewrite"/"New Translation" opens a PICKER of ~10 predefined templates + a "Custom" option (not a blank inline card directly); picking a template adds the card; "Custom" opens the inline name+prompt form; create persists and the card appears
result: issue
reported: "dans l'idée ça fonctionne mais plusieurs pb: (a) on peut ajouter un mode déjà présent sans aucune distinction visuelle — il faut gérer les doublons; (b) si l'utilisateur a modifié Clean Up puis ré-importe Clean Up depuis le picker, il faut un message avertissant que le mode existe déjà et que ça va l'écraser (check d'unicité par nom ou id); (c) en Custom, il faut des placeholders dans les champs (nom du mode) et SURTOUT un exemple dans le prompt montrant comment référencer la transcription avec ${output} — actuellement aucune indication, l'utilisateur ne peut pas deviner qu'il doit mettre ${output} pour que la transcription soit prise en compte"
severity: major

### 5. Edit a mode — title updates after Save (gap-1 fix)
expected: Edit a card, change its name, Save → the card immediately shows the NEW title (previously the save persisted but the card kept the old title)
result: pass

### 6. Delete a mode + recreate from picker (MODE-03)
expected: Delete → confirm → card disappears; a deleted default (e.g. Clean Up) can be re-created from the template picker afterward
result: issue
reported: "j'ai supprimé un mode qui avait un raccourci 'Option' — le raccourci n'a PAS été libéré: il est encore assigné au mode supprimé. Quand je veux réassigner 'Option' à un nouveau mode ça ne marche pas, et cliquer sur Option lance encore une transcription (legacy post-process). Donc supprimer un mode ne nettoie pas son binding."
severity: major

### 7. Bind a multi-key shortcut reliably + conflict (MODE-04 / MODE-05, blocker gap-3 fix)
expected: Click a card's shortcut chip, press Cmd+1 (or similar modifier+key) → it binds and PERSISTS (no longer appears-then-disappears); binding the same combo on another card shows an inline conflict and is blocked
result: issue
reported: "la persistance des combos est OK (Command+4 / Command+0 enregistrés). MAIS quand on essaie d'attribuer à une 2e carte un raccourci déjà utilisé, pendant la capture le raccourci EXISTANT se déclenche (lance la transcription) au lieu d'être capturé → aucun conflit affiché, aucune attribution. Vrai pour une touche simple (Command seul) comme pour les combos. Les raccourcis globaux ne sont pas suspendus pendant la capture."
severity: major

### 8. Clear a bound shortcut from the UI (gap-4 fix)
expected: A bound shortcut shows an X (clear) button; clicking it removes the binding and the shortcut stops firing
result: pass

### 9. Legacy "Option" / post-process shortcut retired (gap-5 fix)
expected: The pre-update post-process shortcut (e.g. "Option") no longer silently triggers transcription; it is gone as a global shortcut (not invisibly active)
result: issue
reported: "le raccourci 'Option' (utilisé avant la MAJ pour le post-process) lance ENCORE une transcription quand on l'active. Il intercepte le combo et empêche de le réassigner à un nouveau mode. La migration n'a pas retiré/migré le binding legacy persisté pour un utilisateur qui était déjà sur l'ancienne version."
severity: blocker

### 10. Translation — enable + engine choice + runs offline (TRANS-01 / TRANS-02)
expected: Translation cards start greyed with an "Enable offline translation" CTA; choosing an engine (active LLM or dedicated TranslateGemma download) enables them; a translation mode then runs offline and outputs translated text
result: issue
reported: "la traduction ne se fait pas. Capture montre un mode 'Traduction Espagnole' (prompt 'Translate the following text to Spanish. Return ONLY...') qui ne produit pas de traduction. Hypothèse à vérifier en diagnostic: prompt sans ${output} → le modèle ne reçoit jamais le transcript."
severity: major

### 11. Change translation engine anytime (gap-7 fix)
expected: After the first engine choice, the Translation section header shows the current engine + a "Change engine" button; you can switch between the active LLM and the dedicated Gemma model in both directions
result: issue
reported: "le changement de moteur fonctionne avec le LLM embarqué (Qwen 2.5): bascule OK dans les deux sens. MAIS avec Apple Foundation/Apple Intelligence comme modèle actif: j'ouvre le modal, je choisis 'utiliser le moteur actuel', je ferme, la traduction espagnole se fait — mais quand je rouvre le modal, c'est encore Gemma qui est marqué activé. Le choix 'moteur actuel/generic' ne persiste pas quand le modèle actif est Apple Foundation."
severity: major

### 12. Compact chip layout + no active-mode UI (visual)
expected: The shortcut control is a compact chip in the card header (not a full-width bar); there is no "set active" / primary post-process button anywhere — modes fire only via their own shortcuts
result: pass
note: Auto-verified from user screenshot — compact right-aligned chips (Option, Command + 4, Command + 0) with X (clear) + edit/delete icons; no "set active"/primary button present

### 13. Refined default prompts quality (post-summary fixes)
expected: Recently-refined default modes produce good output — Clean Up removes filler without changing meaning; Email mode formats a dictated email with the signature on its own block; Bullets mode produces a clean bullet list
result: pass
note: Quality good with the embedded LLM. (Apple Intelligence truncates bullet output to 26 chars — tracked under Test 11/10 gaps, not a prompt-quality defect.)

## Summary

total: 13
passed: 7
issues: 6
pending: 0
skipped: 0

## Gaps

- truth: "The create picker prevents/handles adding a mode that already exists"
  status: failed
  reason: "User reported: a duplicate mode can be added with no visual distinction; re-importing an edited default (e.g. Clean Up) should warn it already exists and will overwrite — needs a uniqueness check by name or id"
  severity: major
  test: 4
  root_cause: ""
  artifacts:
    - path: "src/components/settings/post-processing/SmartModeTemplatePicker.tsx"
      issue: "Picker always addSmartMode (new id) with no duplicate detection; no 'already added' guard or overwrite warning"
  missing:
    - "Detect when a template is already present (by seeded id / name) and either disable it, mark it visually, or warn-before-overwrite"
    - "Define and handle the duplicate cases (pristine default re-add vs edited default re-add vs identical custom)"
  debug_session: ""

- truth: "Custom mode form guides the user to reference the transcript via ${output}"
  status: failed
  reason: "User reported: Custom form has no placeholders; crucially the prompt field gives no indication that ${output} must be included for the transcription to be used — users can't discover this"
  severity: major
  test: 4
  root_cause: ""
  artifacts:
    - path: "src/components/settings/post-processing/SmartModeCard.tsx"
      issue: "Name/prompt inputs lack placeholders; prompt has no ${output} example/hint"
  missing:
    - "Add a name placeholder and a prompt placeholder/example demonstrating ${output} (the transcript token)"
    - "Optionally validate/auto-append ${output} or warn if a custom prompt omits it"
  debug_session: ""

- truth: "Deleting a mode clears its shortcut binding (no orphaned binding keeps firing)"
  status: failed
  reason: "User reported: deleting a mode that had 'Option' did not free the binding; the combo stays assigned to the deleted mode, can't be reassigned, and still fires a transcription"
  severity: major
  test: 6
  root_cause: ""
  artifacts:
    - path: "src-tauri/src/shortcut/mod.rs"
      issue: "deleteSmartMode likely does not call clear_smart_mode_binding / unregister the OS shortcut for the deleted mode"
  missing:
    - "On delete, unregister + remove the mode's shortcut binding so the combo is freed"
  debug_session: ""

- truth: "An 'Option' binding the user no longer sees in the UI must not keep firing (binding/display stay in sync)"
  status: failed
  reason: "User reported: 'Option' (set pre-phase-13 for post-process) still triggers something when pressed, but the UI shows no shortcut for it."
  severity: blocker
  test: 9
  root_cause: "LOG EVIDENCE (dictus.log 14:02:25): Option is bound to smart_mode_mode_email and fires correctly as a smart mode (handy-keys event binding=smart_mode_mode_email hotkey=option Pressed → TranscribeAction::start → SmartModeAction::stop on release). It is NOT legacy transcribe_with_post_process. The bug is a DESYNC: the email mode's option binding is persisted/registered in the backend and fires, but the SmartModeCard does not display it — so it is invisible and uneditable/unclearable from the UI."
  artifacts:
    - path: "src/components/settings/post-processing/SmartModeCard.tsx"
      issue: "Card shortcut chip does not reflect the persisted binding for smart_mode_mode_email (reads binding state from wrong source / not refreshed after migration or edit), so an active binding shows as unbound"
    - path: "src-tauri/src/shortcut/mod.rs"
      issue: "Binding registry can hold a smart_mode binding the UI never surfaces; need a single source of truth the card reads"
    - path: "src-tauri/src/settings.rs"
      issue: "v12->v13 migration may have mapped the legacy option post-process binding onto smart_mode_mode_email without the UI picking it up"
  missing:
    - "Card must display every persisted smart-mode binding (read from the same store the backend registers from) so 'invisible but active' is impossible"
    - "Verify the migration mapping of the old option binding and ensure the resulting binding is visible + clearable"
  debug_session: ""

- truth: "A translation mode runs offline and outputs translated text"
  status: failed
  reason: "User reported: translation does not happen — a 'Traduction Espagnole' mode ('Translate the following text to Spanish. Return ONLY...') produces no translation"
  severity: major
  test: 10
  root_cause: "PARTIALLY RESOLVED BY LOG EVIDENCE (dictus.log 14:08:06-11): a DEDICATED Translation mode (smart_mode_mode_translate_es) via the embedded generic LLM DID translate correctly ('La traducción se hará con el modelo predeterminado.'). So translation works on the embedded-LLM + Translation-mode path. The reported failure is likely the rewrite-style 'Traduction Espagnole' custom mode and/or the Apple Intelligence provider path — to confirm in diagnosis."
  artifacts:
    - path: "src-tauri/src/actions.rs"
      issue: "Rewrite-style translation custom mode may run through apple_intelligence provider (which truncates output) rather than run_translation; verify which path a rewrite-with-translation-prompt takes"
    - path: "src/components/settings/post-processing/SmartModeTemplatePicker.tsx"
      issue: "Predefined translation/rewrite template prompts may omit ${output}"
  missing:
    - "Reproduce the exact failing case (rewrite 'Traduction Espagnole' vs dedicated Translation mode; Apple vs embedded engine) and confirm transcript injection + provider used"
  debug_session: ""

- truth: "Assigning an already-used shortcut shows an inline conflict and is blocked"
  status: failed
  reason: "User reported: during capture, pressing an already-bound combo TRIGGERS that existing shortcut (fires transcription) instead of being captured — so no conflict is shown and no reassignment happens. Applies to single keys and combos alike."
  severity: major
  test: 7
  root_cause: ""
  artifacts:
    - path: "src/components/settings/post-processing/SmartModeShortcutChip.tsx"
      issue: "Global OS shortcuts are not suspended during capture; the 13-03 best-effort suspend/resume (catch(()=>{})) does not actually disable existing bindings while recording"
    - path: "src-tauri/src/shortcut/mod.rs"
      issue: "No reliable suspend-all / disable-global-shortcuts-during-capture path the chip can call"
  missing:
    - "Suspend all global shortcuts while a chip is recording so the pressed combo is captured, conflict can be detected, and the binding is committed instead of firing the existing shortcut"
  debug_session: ""
