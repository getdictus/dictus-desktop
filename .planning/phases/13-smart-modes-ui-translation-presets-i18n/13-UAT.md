---
status: diagnosed
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
  status: diagnosed
  reason: "User reported: a duplicate mode can be added with no visual distinction; re-importing an edited default (e.g. Clean Up) should warn it already exists and will overwrite — needs a uniqueness check by name or id"
  severity: major
  test: 4
  root_cause: "The picker's only dedup guard (existingModeIds.includes(template.id), SmartModeTemplatePicker.tsx:83) is dead code: add_smart_mode (shortcut/mod.rs:1080) discards the template's seeded id and mints a fresh format!(\"mode_{}\", timestamp) id, so a mode created from 'Clean Up' becomes mode_1733… and never equals template.id (mode_clean_up). The 'added' badge never shows and the button stays clickable. This is a direct consequence of decision [13-07] (use addSmartMode/new id to avoid delete-then-recreate collisions), which removed the only signal the id check relied on. There is no name-based fallback and no overwrite path — handlePickTemplate only ever calls addSmartMode, never updateSmartMode."
  artifacts:
    - path: "src/components/settings/post-processing/SmartModeTemplatePicker.tsx"
      issue: "Lines 39-51,83: add-only (no overwrite); id-based dedup is dead code; unused name map SEEDED_MODE_DEFAULT_NAME available"
    - path: "src-tauri/src/shortcut/mod.rs"
      issue: "Lines 1072-1091: add_smart_mode forces fresh timestamp id, defeating id-based detection"
    - path: "src/components/settings/post-processing/SmartModeCard.tsx"
      issue: "Lines 29-40: SEEDED_MODE_DEFAULT_NAME map exists but picker never uses it for name-based detection"
  missing:
    - "Switch picker dedup to name-based matching against current modes' display names (reuse SEEDED_MODE_DEFAULT_NAME + localized names)"
    - "On match: mark the template visually and on click warn 'already exists / overwrite?' → updateSmartMode(existingId, ...) instead of addSmartMode"
    - "Handle the duplicate cases: pristine default re-add vs edited default re-add vs identical custom"
  debug_session: ".planning/debug/picker-duplicates-output-hint.md"

- truth: "Custom mode form guides the user to reference the transcript via ${output}"
  status: diagnosed
  reason: "User reported: Custom form has no placeholders; crucially the prompt field gives no indication that ${output} must be included for the transcription to be used — users can't discover this"
  severity: major
  test: 4
  root_cause: "Confirmed: the authoritative transcript token is the literal string ${output} (consumed in actions.rs:74 chat path / actions.rs:333 legacy path). The 6 built-in rewrite templates (settings.rs:672-757) already end with 'Text:\\n${output}', but in SmartModeCard.tsx the name Input (line 201) and prompt Textarea (line 215) have NO placeholder and NO helper/example text and never reference ${output}. A user building a custom rewrite mode cannot discover the token, so their prompt silently fails to inject the transcript on the substitution path."
  artifacts:
    - path: "src/components/settings/post-processing/SmartModeCard.tsx"
      issue: "Lines 201,215: name Input + prompt Textarea lack placeholders and any ${output} example/helper"
    - path: "src-tauri/src/actions.rs"
      issue: "Lines 74,333: authoritative ${output} substitution — the hint shown to users must match this exact token"
  missing:
    - "Add a name placeholder and a prompt placeholder/example demonstrating ${output} (e.g. 'Rewrite the following text…\\n\\nText:\\n${output}') plus a short helper line"
    - "Optionally validate/warn (or auto-append) when a custom rewrite prompt omits ${output}"
  debug_session: ".planning/debug/picker-duplicates-output-hint.md"

- truth: "Deleting a mode clears its shortcut binding (no orphaned binding keeps firing)"
  status: diagnosed
  reason: "User reported: deleting a mode that had 'Option' did not free the binding; the combo stays assigned to the deleted mode, can't be reassigned, and still fires a transcription"
  severity: major
  test: 6
  root_cause: "delete_smart_mode (shortcut/mod.rs:1116-1125) removes the mode from settings.smart_modes (via binding-agnostic delete_mode_in_place) but never unregisters the OS shortcut and never removes the smart_mode_{id} entry from settings.bindings. The correct cleanup primitive clear_smart_mode_binding (mod.rs:1188-1206) exists but is wired ONLY to the chip's X button (SmartModeShortcutChip.tsx:199). Frontend handleDelete (SmartModeCard.tsx:141-152) calls deleteSmartMode without clearSmartModeBinding. Result: the OS shortcut stays registered in-session and keeps firing; the orphaned bindings entry makes change_binding/register_shortcut for a new mode collide (success:false → surfaced as conflict). Note: after restart the deleted mode is NOT re-registered (init_shortcuts iterates smart_modes, handy_keys.rs:454-466), so the live in-session orphan is the active defect; the persisted bindings entry is inert at registration but should still be cleaned."
  artifacts:
    - path: "src-tauri/src/shortcut/mod.rs"
      issue: "delete_smart_mode (1116) missing binding cleanup; clear_smart_mode_binding (1188) is the reusable primitive to fold in"
    - path: "src/components/settings/post-processing/SmartModeCard.tsx"
      issue: "handleDelete (141-152) does not invoke clearSmartModeBinding"
  missing:
    - "Make delete_smart_mode perform the same cleanup as clear_smart_mode_binding atomically (unregister OS shortcut if bound + remove settings.bindings[smart_mode_{id}]) within the same read/modify/write"
    - "Optionally call clearSmartModeBinding in handleDelete as belt-and-suspenders"
  debug_session: ".planning/debug/delete-orphan-binding.md"

- truth: "Every persisted smart-mode binding is shown on its card (no active-but-invisible binding)"
  status: diagnosed
  reason: "User reported: 'Option' (set pre-phase-13 for post-process) still triggers something when pressed, but the UI shows no shortcut for it."
  severity: blocker
  test: 9
  root_cause: "NOT a key-format mismatch (frontend SmartModeCard.tsx:87 'smart_mode_'+mode.id and backend smart_mode_binding_id() mod.rs:1066 produce the identical smart_mode_mode_email) and NOT a migration mapping. It is an IDENTITY/DUPLICATION desync: the user UAT'd at the 13-04 checkpoint when default_smart_modes() seeded the FULL 10-mode catalogue (commit a421a61), so their persisted smart_modes contained an entry id=mode_email. They bound Option to it → bindings[smart_mode_mode_email] + smart_modes[mode_email] persist (smart_modes is never rebuilt from defaults; commit 5243d60 only changed first-run seeding). handy_keys::init_shortcuts (handy_keys.rs:454-456) keeps registering Option from that persisted entry → it fires (SmartModeAction delegates to TranscribeAction). Meanwhile the 'Write as Email' card the user now sees is a DIFFERENT entity: the picker mints a fresh mode_{timestamp} id (no dedup), whose binding key is empty → its chip shows 'add shortcut'. The Option binding belongs to the orphaned seeded mode_email, invisible/unclearable from the UI while still firing. Same root family as the delete-orphan gap (test 6) and rooted in the same no-dedup picker as test 4."
  artifacts:
    - path: "src-tauri/src/settings.rs"
      issue: "default_smart_modes() seed-set history (a421a61 all 10 → 5243d60 Clean Up only); add_smart_mode mints mode_{timestamp} with no dedup; migration writes smart_mode_{active_id}"
    - path: "src-tauri/src/shortcut/handy_keys.rs"
      issue: "Lines 454-466: registers smart-mode shortcuts from smart_modes × bindings — registers an orphaned mode_email binding indefinitely"
    - path: "src/components/settings/post-processing/SmartModeTemplatePicker.tsx"
      issue: "No dedup → creates a duplicate Email card with a new id, splitting the identity"
    - path: "src/components/settings/post-processing/SmartModeCard.tsx"
      issue: "Lines 85-90: lookup is correct, but only sees the binding if the card's mode.id matches the persisted binding's id"
  missing:
    - "Add picker name-based dedup (shared with test 4) so re-adding a seeded template reuses/overwrites the existing id instead of minting mode_{timestamp} — prevents the duplicate-card identity split"
    - "On migration/seed-reduction, reconcile dangling bindings: drop or re-home any bindings[smart_mode_*] whose mode id no longer exists in smart_modes (symmetric with the delete-orphan fix)"
    - "Backstop: surface any smart_mode_* binding present in settings even when no matching card exists, so 'active but invisible' is structurally impossible"
  debug_session: ".planning/debug/option-binding-invisible.md"

- truth: "A translation mode runs offline and outputs translated text"
  status: diagnosed
  reason: "User reported: translation does not happen — a 'Traduction Espagnole' mode ('Translate the following text to Spanish. Return ONLY...') produces no translation"
  severity: major
  test: 10
  root_cause: "It is the Apple-Intelligence-structured-outputs path, NOT a missing ${output} token. The failing 'Traduction Espagnole' is a REWRITE-kind mode → post_process_with_prompt → the user's active post-process provider (apple_intelligence). Because apple_intelligence has supports_structured_output:true (settings.rs:617), the rewrite enters the structured branch (actions.rs:213-261) and calls Swift processTextWithSystemPrompt, which ALWAYS uses a hardcoded @Generable struct CleanedTranscript { let cleanedText: String } (apple_intelligence.swift:5-9). That schema frames every task as 'clean a transcript', so 'Translate to Spanish' yields near-unchanged text — translation does not happen. (The transcript IS sent as user_content, so ${output} is irrelevant here.) The dedicated Translation-kind mode → run_translation → embedded LLM works (log 14:08:06). Same Apple schema root cause as the test-11 truncation."
  artifacts:
    - path: "src-tauri/src/actions.rs"
      issue: "Lines 973-985 rewrite vs run_translation split; lines 213-261 Apple structured-output branch"
    - path: "src-tauri/src/apple_intelligence.swift"
      issue: "Lines 5-9: hardcoded CleanedTranscript @Generable schema applied to ALL tasks (semantic mismatch)"
    - path: "src-tauri/src/settings.rs"
      issue: "Line 617: apple_intelligence.supports_structured_output=true is the entry point into the broken branch"
  missing:
    - "Make the Apple Intelligence smart-mode path task-agnostic: use plain free-text session.respond(to:) carrying the user prompt as instructions instead of the hardcoded CleanedTranscript schema (fixes translation-as-rewrite AND non-cleanup rewrites)"
    - "Optionally route translate-style rewrite modes to the working embedded translation path, or discourage hand-rolled translation-as-rewrite while a structured-output provider is active"
  debug_session: ".planning/debug/translation-rewrite-apple-path.md"

- truth: "Assigning an already-used shortcut shows an inline conflict and is blocked"
  status: diagnosed
  reason: "User reported: during capture, pressing an already-bound combo TRIGGERS that existing shortcut (fires transcription) instead of being captured — so no conflict is shown and no reassignment happens. Applies to single keys and combos alike."
  severity: major
  test: 7
  root_cause: "SmartModeShortcutChip never suspends the OTHER (conflicting) global shortcuts while recording. Two defects: (1) Wrong scope — on record start (chip.tsx:191-193) it calls suspendBinding('smart_mode_'+modeId) and only if the chip is already bound, so it suspends at most its own binding; every other OS hotkey stays live and fires before any conflict can be detected. There is no command-layer 'suspend all global shortcuts' primitive (only private unregister_all_shortcuts, mod.rs:357, used for impl switching). (2) Wrong capture channel — the chip captures via webview window keydown (chip.tsx:171-173) only, which doesn't stop OS-level hotkeys. The app's working ShortcutInput/HandyKeysShortcutInput/GlobalShortcutInput edit ONE known binding and suspend it (GlobalShortcutInput.tsx:184), and on handy_keys also capture via the backend recording stream (handy-keys-event) regardless of focus. handy_keys recording is non-blocking (KeyboardListener::new, not new_with_blocking) so registered hotkeys still fire during recording — confirming suspension (not listener swallowing) is the mechanism that must change."
  artifacts:
    - path: "src/components/settings/post-processing/SmartModeShortcutChip.tsx"
      issue: "Suspends only its own binding (and only if bound); captures via webview keydown; no global suspend"
    - path: "src-tauri/src/shortcut/mod.rs"
      issue: "suspend_binding (215) is per-id only; all-bindings logic (unregister_all_shortcuts, 357) exists but is not exposed as a command"
    - path: "src/components/settings/HandyKeysShortcutInput.tsx"
      issue: "Working reference: per-binding suspend + backend recording-stream capture on handy_keys"
  missing:
    - "Expose a command-level suspend-all / resume-all global-shortcuts pair (reuse unregister_all_shortcuts + a re-register-all counterpart)"
    - "Chip calls suspend-all on record start and resume-all on EVERY exit path (commit, conflict, click-outside chip.tsx:158-169, effect cleanup 175-181)"
    - "For handy_keys correctness, capture via the backend recording stream when keyboard_implementation==='handy_keys' (mirror HandyKeysShortcutInput), since after suspend-all the OS hotkeys no longer reach the webview"
  debug_session: ".planning/debug/capture-no-suspend.md"

- truth: "Translation engine choice persists and the modal reflects it on reopen (incl. Apple Foundation); post-process output is not truncated"
  status: diagnosed
  reason: "User reported: with Apple Foundation active, choosing 'use current model' doesn't persist — reopening the modal still shows Gemma. Works with embedded Qwen. Separately, Apple bullet output truncated to 26 chars."
  severity: major
  test: 11
  root_cause: "TWO independent Apple-path defects. (1) The choice DOES persist (setTranslationEngineChoice writes generic_model, commands/llm.rs:77-85; SmartModesSection reads it correctly). The modal MIS-DISPLAYS it: TranslationEngineChoiceModal infers the active engine from useLlmModelStore.activeModelId (the embedded GGUF id), not from translation_engine_choice — recActive = activeModelId==='gemma-3-4b' (line 56), currentActiveSelected = activeModelId!=null && !=='gemma-3-4b' (61-62). Apple Intelligence is a post-process PROVIDER, not a GGUF model, so active_llm_model_id is null → both flags false → 'use current model' never gets the active badge → looks like Gemma is still chosen. Works with Qwen because Qwen sets a real activeModelId. (2) Apple 26-char truncation = the SAME hardcoded CleanedTranscript @Generable schema (apple_intelligence.swift:5-9) with includeSchemaInPrompt:true biasing the model to a terse cleaned-text value; NOT a word/token cap (token_limit parses 'Apple Intelligence'→0→no-op; maximumResponseTokens unset)."
  artifacts:
    - path: "src/components/settings/post-processing/TranslationEngineChoiceModal.tsx"
      issue: "Lines 27-62,197: derives active-engine UI from activeModelId instead of translation_engine_choice; gates 'use current' on activeDownloaded (false for Apple)"
    - path: "src/stores/llmModelStore.ts"
      issue: "Lines 51-68: activeModelId is GGUF-only (null for Apple Intelligence provider)"
    - path: "src-tauri/src/apple_intelligence.swift"
      issue: "Lines 5-9,96-107: hardcoded CleanedTranscript schema + schema-in-prompt → 26-char truncation (shared with test 10)"
  missing:
    - "Drive the modal's active-engine badge from the persisted translation_engine_choice (treat generic_model + non-Gemma active engine as 'use current model = active'); stop requiring activeModelId!=null / activeDownloaded for non-GGUF providers"
    - "Apple truncation: covered by the test-10 fix (task-agnostic Apple path / drop the fixed CleanedTranscript schema for smart modes)"
  debug_session: ".planning/debug/engine-persist-apple-truncation.md"
