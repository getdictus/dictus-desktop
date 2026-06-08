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
  - 13-09-SUMMARY.md
  - 13-10-SUMMARY.md
  - 13-11-SUMMARY.md
started: 2026-06-05T12:53:42Z
updated: 2026-06-08T13:10:00Z
retest:
  date: 2026-06-08
  closed_plans: [13-13, 13-14, 13-15]
  resolved_gaps: 3  # test 3 [D8] fully; test 7 [B5] block-half; test 11 [C7] display-half
  new_gaps: 2       # [E9] chip left/right modifier parity (test 7); [F10] Gemma engine switch no-op under Apple provider (test 11)
  remaining_gaps: 2  # [E9], [F10] — see ## Gaps (status: failed)
  next: "/gsd:plan-phase 13 --gaps"
prior_retest:
  date: 2026-06-05
  closed_plans: [13-09, 13-10, 13-11]
  resolved_gaps: 4
  remaining_gaps: 3  # tests 7, 11, 3 — closed in 13-13/14/15
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
retest_2026-06-08: "COLLISION-BLOCK HALF RESOLVED (13-12 suspend-all + 13-13 backend collision check — duplicate combos now blocked with inline error). NEW DEFECT found: the chip does NOT distinguish left/right modifiers. User's main dictation (General tab) is CmdRight; binding 'Command Right' to the English-translation mode stored it as generic Cmd (logs: smart_mode_mode_translate_en → Cmd, HotkeyId 118), so it fires on EITHER command key — Command Left triggered the EN translation and Command Right produced mixed FR/EN output (transcribe on CmdRight + generic Cmd mode both firing). Root cause: chip captures via webview keydown; getKeyName (keyboard.ts:65-66) collapses MetaLeft/MetaRight → generic meta. The General-tab HandyKeysShortcutInput captures via the backend handy-keys-event recording stream which preserves CmdRight/CmdLeft. Chip was never switched to that stream (deferred sub-item of the original gap)."
severity_retest: major

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
retest_2026-06-08: "DISPLAY HALF RESOLVED (13-14): with Apple Intelligence active the modal now correctly shows 'Use your current model → Apple Intelligence — Currently used' (user screenshot confirms). NEW DEFECT (opposite direction): clicking 'Use Gemma 3 4B' does NOT take — after a translation + reopen, the badge reverts to Apple Intelligence; the switch to the dedicated Gemma engine never persists. Root cause: enableWith() (TranslationEngineChoiceModal.tsx:73-96) ALWAYS persists setTranslationEngineChoice('generic_model'); 'Use Gemma' only additionally calls setActiveModel('gemma-3-4b'), but set_active_llm_model (commands/llm.rs:51-66) writes active_llm_model_id ONLY — it never flips post_process_provider_id off apple_intelligence. The 13-14 badge gate (providerId==='embedded') therefore stays false → recActive false, currentActiveSelected true → reverts to Apple. The modal conflates 'active GGUF model' with 'active post-process provider'."
severity_retest: major

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

### Re-test 2026-06-05 (after gap-closure plans 13-09, 13-10, 13-11, 13-12 + 13-08 gates)
resolved: 4  # picker dedup (test 4), ${output} hint (test 9), delete-orphan (test 6), invisible binding (test 9/blocker), translation-via-Apple (test 10)
remaining: 3
  - test 7  — duplicate shortcut not blocked/warned (capture/suspend fixed by 13-12; backend cross-mode collision check still missing) [B5]
  - test 11 — engine modal still shows Gemma when Apple Foundation active (truncation half fixed by 13-10; frontend badge precedence bug remains) [C7]
  - test 3  — seeded mode names not localized (English fallback values in non-en locales; names-only translation needed) [D8]
next: /gsd:plan-phase 13 --gaps  (user chose formal gap-closure cycle, 2026-06-05)

### Re-test 2026-06-08 (after gap-closure plans 13-13 [B5], 13-14 [C7], 13-15 [D8])
resolved: 3  # test 3 [D8] seeded-name i18n (auto-verified — fr/es/de/ja/zh/ar names native, check:translations green); test 7 [B5] collision-BLOCK half (13-13 backend gate rejects duplicate combos w/ inline error); test 11 [C7] display half (13-14 — Apple Intelligence now shown correctly, user screenshot)
new_gaps: 2
  - test 7  [E9] — chip cannot bind left/right-distinct modifiers (CmdRight degrades to generic Cmd → cross-fires with General-tab CmdRight dictation). Major.
  - test 11 [F10] — 'Use Gemma 3 4B' does not persist when active provider is Apple Intelligence (setActiveModel writes active_llm_model_id only; never flips post_process_provider_id → badge reverts to Apple, Gemma engine never engaged). Major.
next: /gsd:plan-phase 13 --gaps  (close [E9] modifier-side parity + [F10] engine-switch provider flip)

## Gaps

- truth: "The create picker prevents/handles adding a mode that already exists"
  status: resolved
  resolved_by: "13-11 (name-based dedup + overwrite-warn in picker) + 13-09 (seeded-id reuse/overwrite in add_smart_mode). Re-test 2026-06-05: PASS."
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
  status: resolved
  resolved_by: "13-11 (custom-form name/prompt placeholders + ${output} outputHint helper). Re-test 2026-06-05: PASS (test 9)."
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
  status: resolved
  resolved_by: "13-09 (atomic delete_smart_mode: unregister OS shortcut + remove bindings entry in one write). Re-test 2026-06-05: PASS."
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
  status: resolved
  resolved_by: "13-09 (seeded-id reuse in add_smart_mode kills the duplicate-card identity split + reconcile_dangling_smart_mode_bindings drops orphaned smart_mode_* bindings on load). Re-test 2026-06-05: PASS."
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
  status: resolved
  resolved_by: "13-10 (Apple Intelligence path made task-agnostic: dropped hardcoded CleanedTranscript @Generable schema, plain free-text session.respond). Re-test 2026-06-05: PASS — Apple Intelligence now produces actual Spanish output, user-verified live."
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
  status: failed
  reason: "RE-TEST 2026-06-05 (after 13-12): the CAPTURE half is fixed — suspend-all during recording means an already-bound combo no longer fires its action; it is captured. But the BLOCK/WARN half still fails: the same combo can be bound to two different cards with NO conflict message; both cards keep showing it; at runtime the last-bound one wins. Logs confirm the duplicate is persisted on both modes and only collides at OS re-registration: `resume_all_shortcuts: failed to re-register 'smart_mode_mode_...': Hotkey already registered: Cmd+2`."
  severity: major
  test: 7
  remaining_root_cause: "Cross-mode duplicate detection is missing at commit time. SmartModeShortcutChip.commitCombo (chip.tsx:83) already renders response.error when setSmartModeBinding returns success:false — but the backend set_smart_mode_binding/change_binding accepts a combo already bound to a DIFFERENT mode without returning a conflict. So no error surfaces, both bindings persist, and resume_all_shortcuts hits 'Hotkey already registered'. The 13-12 suspend-all work is correct and should be kept; the new fix is purely the backend collision check."
  fixed_so_far: "13-12 task 1 — chip suspends ALL global shortcuts during capture and resumes on every exit path (commit 262576b). Verified: combo no longer fires during capture."
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
  status: failed
  reason: "RE-TEST 2026-06-05 (after 13-10 + 13-12): TRUNCATION half RESOLVED — Apple bullet output is now full-length (13-10, user-verified). MODAL-DISPLAY half STILL FAILS: with Apple Foundation active, the engine modal still shows Gemma as the current model instead of the active Apple provider."
  severity: major
  test: 11
  remaining_root_cause: "13-12 moved the badge off useLlmModelStore onto an activeEngineName prop, but SmartModesSection.tsx:111-114 resolves it as `activeModelName ?? providerLabel ?? providerId`. activeModelName comes from the LLM store's lingering activeModelId (gemma-3-4b), which is non-null even when the active POST-PROCESS provider is Apple Foundation — so the `??` short-circuits to Gemma. Fix: gate on post_process_provider_id — use the GGUF model name ONLY when the embedded/local-LLM provider is active; otherwise use the provider's label. activeIsRecommended must likewise be false when the active provider is not the embedded GGUF one."
  fixed_so_far: "13-10 — Apple truncation gone (task-agnostic path, user-verified full bullets). 13-12 task 2 — badge plumbed via engineChoice/activeEngineName/activeIsRecommended props (commit 89e17d6) but the precedence bug above means it still mis-resolves to Gemma for non-GGUF providers."
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

- truth: "Seeded smart-mode names display localized to the active app language (MODE-06 / L10N-01)"
  status: failed
  reason: "RE-TEST 2026-06-05 (newly surfaced; test 3 was auto-marked pass on code inspection but live test fails): with the app set to French, the seeded mode names render in English — 'Clean Up', 'Bullet Points', 'Write as Email', 'Trad Span' — instead of localized labels (e.g. 'Nettoyage'). A renamed mode correctly keeps the user's literal name; the defect is only the un-renamed seeded names."
  severity: minor
  test: 3
  remaining_root_cause: "NOT an architecture problem — id↔label is already decoupled correctly. SmartModeCard.tsx:73-75 localizes an un-renamed seeded mode via t(SEEDED_MODE_ID_TO_I18N_KEY[mode.id]) (e.g. mode_clean_up → smartModes.defaultModes.cleanUp). The defect is i18n DATA: the smartModes.defaultModes.* values in all 19 non-English locales are still the English fallback strings (translation was explicitly deferred — decision [Phase 13] 'English fallback values used verbatim … real translations deferred'). So t() returns 'Clean Up' even under fr. Fix = supply real translations for the ~10 smartModes.defaultModes.* NAME keys across all 20 shipped locales (names only; prompt/description text stays English per user decision 2026-06-05). check:translations must stay green."
  artifacts:
    - path: "src/components/settings/post-processing/SmartModeCard.tsx"
      issue: "Lines 14-24,73-75: SEEDED_MODE_ID_TO_I18N_KEY map + t(i18nKey) localization is correct; depends entirely on the locale VALUES being real translations"
    - path: "src/i18n/locales/*/translation.json"
      issue: "smartModes.defaultModes.* keys hold English fallback values in all 19 non-English locales (cleanUp, makeFormal, makeCasual, writeAsEmail, bulletPoints, summarize, translateToEnglish/Spanish/French/Chinese)"
  missing:
    - "Translate the ~10 smartModes.defaultModes.* NAME keys into real strings for all 20 shipped locales (names only)"
    - "Keep en as source of truth; do NOT translate the prompt/description bodies (user decision: names only)"
    - "Re-run bun run check:translations (must stay green at full key count)"

- truth: "Smart Mode shortcut chip binds left/right-distinct modifiers (CmdRight/CmdLeft) the same way the General-tab shortcut input does"
  status: failed
  reason: "RE-TEST 2026-06-08 (newly surfaced while confirming test 7 [B5]): the collision-block half is now fixed (13-13), but the chip cannot capture a left/right-distinct modifier. User's main dictation (General tab) is CmdRight; binding 'Command Right' to the EN-translation mode stored generic Cmd (logs: smart_mode_mode_translate_en → Cmd, HotkeyId 118). Generic Cmd fires on EITHER command key → Command Left triggered the EN translation, Command Right gave mixed FR/EN output. The General tab correctly stored CmdRight (transcribe, HotkeyId 117/121). Parity is broken between the two inputs."
  severity: major
  test: 7
  tag: E9
  root_cause: "SmartModeShortcutChip captures via webview window keydown (SmartModeShortcutChip.tsx:107-133, getKeyName(e,osType)). getKeyName (src/lib/utils/keyboard.ts:65-66) maps BOTH MetaLeft and MetaRight → getModifierName('meta'), and likewise collapses ShiftLeft/Right, ControlLeft/Right, AltLeft/Right (keyboard.ts:59-68) — so the webview path can never produce a side-distinct modifier. The General-tab HandyKeysShortcutInput captures via the backend 'handy-keys-event' recording stream (HandyKeysShortcutInput.tsx:82-84, startRecording 173-186) which preserves CmdRight/CmdLeft. The chip was never switched to that backend stream — this is the deferred 'capture via the backend recording stream when keyboard_implementation===handy_keys' sub-item from the original test-7 gap (13-12/13-13 added suspend-all + backend collision check but kept webview capture). Because the combo is degraded to Cmd BEFORE setSmartModeBinding runs, the 13-13 collision check cannot catch it (generic Cmd != CmdRight, so no conflict is reported and the generic binding persists and cross-fires)."
  artifacts:
    - path: "src/components/settings/post-processing/SmartModeShortcutChip.tsx"
      issue: "Lines 107-133,169-171: captures via webview window keydown + getKeyName; no left/right side. Does not use the backend handy-keys-event recording stream the General tab uses."
    - path: "src/lib/utils/keyboard.ts"
      issue: "Lines 59-68: MetaLeft/MetaRight (+ Shift/Control/Alt L/R) both collapse to a single generic modifier name — webview capture is structurally side-blind."
    - path: "src/components/settings/HandyKeysShortcutInput.tsx"
      issue: "Lines 82-84,173-186: working reference — captures via backend recording stream, preserves CmdRight/CmdLeft."
  missing:
    - "When keyboard_implementation==='handy_keys', capture the chip's shortcut via the backend handy-keys-event recording stream (mirror HandyKeysShortcutInput) so left/right-distinct modifiers (CmdRight/CmdLeft, OptRight, etc.) are preserved end-to-end"
    - "Ensure the captured side-distinct combo string matches what set_smart_mode_binding/find_conflicting_binding compare against, so a CmdRight smart-mode binding both persists as CmdRight AND collides correctly with the General-tab CmdRight dictation"
    - "Secondary (observed in logs): resume_all_shortcuts logs many 'Hotkey already registered' warnings — resumeAll is called on multiple exit paths (commitCombo + effect cleanup) and the backend re-register is not idempotent; make resume_all_shortcuts tolerate already-registered hotkeys (or guard double-resume) to keep logs clean and avoid leaving a stale suspend state"
  debug_session: ""

- truth: "Choosing 'Use Gemma 3 4B' in the translation engine modal actually switches the translation engine to Gemma and persists across reopen (even when the active provider is Apple Intelligence)"
  status: failed
  reason: "RE-TEST 2026-06-08 (newly surfaced while confirming test 11 [C7]): the display half is resolved (Apple Intelligence shown correctly), but switching FROM 'use current model' TO the dedicated Gemma does not take. User clicks 'Use Gemma 3 4B', modal closes, runs a translation, reopens → badge is back on 'Use your current model — Apple Intelligence — Currently used'. The Gemma engine is never engaged; translation keeps using Apple Intelligence."
  severity: major
  test: 11
  tag: F10
  root_cause: "enableWith() (TranslationEngineChoiceModal.tsx:73-96) ALWAYS persists commands.setTranslationEngineChoice('generic_model') for BOTH options; the recommended-Gemma path differs only by an extra useLlmModelStore.setActiveModel('gemma-3-4b') (line 78). setActiveModel → set_active_llm_model (commands/llm.rs:51-66) loads the GGUF and writes settings.active_llm_model_id ONLY — it never changes settings.post_process_provider_id (stays 'apple_intelligence'). The 13-14 badge gate resolves activeIsRecommended = isEmbeddedActive && activeModelId==='gemma-3-4b' where isEmbeddedActive = providerId==='embedded'. Provider is still apple_intelligence → isEmbeddedActive false → activeIsRecommended false → recActive false, currentActiveSelected true → modal reverts to Apple. The modal models translation-engine choice as a single 'generic_model' value + active GGUF id, conflating 'active GGUF model' with 'active post-process provider'; choosing Gemma while a non-embedded provider is active cannot express itself. (Mirror of [C7]: 13-14 fixed the display but exposed that the switch action never flips the provider.)"
  artifacts:
    - path: "src/components/settings/post-processing/TranslationEngineChoiceModal.tsx"
      issue: "Lines 73-96: enableWith() hardcodes setTranslationEngineChoice('generic_model') for both buttons; 'Use Gemma' only calls setActiveModel(gemma) which does not switch the active provider. Lines 64-70: recActive/currentActiveSelected derive entirely from activeIsRecommended (provider-gated), so a Gemma choice under an Apple provider can never show as active."
    - path: "src-tauri/src/commands/llm.rs"
      issue: "Lines 51-66 set_active_llm_model: writes active_llm_model_id only; never touches post_process_provider_id. Lines 75-85 set_translation_engine_choice: persists choice but choice='generic_model' is identical for both options, so it carries no Gemma-vs-current signal."
    - path: "src/components/settings/post-processing/SmartModesSection.tsx"
      issue: "13-14 gate: activeIsRecommended = isEmbeddedActive && activeModelId==='gemma-3-4b'; correct for display but means the engine choice must actually flip providerId to 'embedded' for the Gemma badge to ever show."
  missing:
    - "Make 'Use Gemma 3 4B' actually switch the active post-process provider to the embedded/local-LLM provider (set post_process_provider_id='embedded' + active_llm_model_id='gemma-3-4b') so providerId==='embedded' and the badge + translation engine both reflect Gemma"
    - "Verify the translation path consumes the switched engine end-to-end (run_translation / post_process uses the embedded Gemma, not the lingering Apple provider) — confirm with a live Spanish translation after switching"
    - "Define the reverse: 'Use your current model' restores the user's prior real provider; ensure choosing it after Gemma flips post_process_provider_id back. Persist enough to distinguish the two choices across reopen (don't collapse both to 'generic_model' if that loses the Gemma intent)."
  debug_session: ""
