---
status: diagnosed
trigger: "le raccourci 'Option' (paramétré avant la phase 13 pour le post-process) lance encore une transcription quand on l'active, mais visuellement je ne le vois pas dans l'interface."
created: 2026-06-05T14:30:00Z
updated: 2026-06-05T15:05:00Z
---

## Current Focus

hypothesis: CONFIRMED — identity/duplication desync, NOT a key-format mismatch. The firing binding smart_mode_mode_email belongs to a seeded mode_email entry that is not the Email card the user now sees (a picker-added duplicate with id mode_{timestamp}, or an orphaned seed). Same key format both sides.
test: Done — traced card lookup, backend registration, migration, seed-id git history, picker id minting.
expecting: n/a
next_action: Return diagnosis (find_root_cause_only).

## Symptoms

expected: Every persisted smart-mode shortcut binding is displayed on its card; a backend-active binding is never invisible.
actual: Option is bound to smart_mode_mode_email and fires correctly as a smart mode (log: binding=smart_mode_mode_email hotkey=option), but the email card shows no shortcut chip — invisible/uneditable/unclearable.
errors: None
reproduction: Test 9 in Phase 13 UAT. Upgrader who had Option set for post-process before Phase 12/13.
started: After v12->v13 migration (upgrade path).

## Eliminated

- hypothesis: Key-format mismatch — frontend looks up the binding under a different key than the backend registers (e.g. double mode_ prefix or seeded-vs-stored id).
  evidence: Frontend SmartModeCard.tsx:87 computes bindingKey = "smart_mode_" + mode.id. Backend smart_mode_binding_id() (mod.rs:1066) = format!("smart_mode_{}", mode_id). For mode.id = "mode_email" BOTH produce exactly "smart_mode_mode_email" — the same key the log shows. No format mismatch.
  timestamp: 2026-06-05T14:45:00Z

- hypothesis: v12->v13 migration mapped the legacy transcribe_with_post_process Option binding onto smart_mode_mode_email.
  evidence: Migration combo-transfer (settings.rs:895-917) writes to smart_mode_{active_id}, where active_id derives from post_process_selected_prompt_id and falls back to mode_clean_up. A real v12 user's selected id is a custom/improve id or null → key becomes smart_mode_mode_clean_up or smart_mode_{custom}, never smart_mode_mode_email. mode_email is a Phase-12/13 SEED id a v12 user never had. So the live binding did NOT come from migration.
  timestamp: 2026-06-05T14:50:00Z

## Evidence

- timestamp: 2026-06-05T14:30:00Z
  checked: STATE.md decisions + UAT gaps
  found: Line 117 — "SmartModeCard reads bindings from the Zustand settings store, but the chip mutates via direct commands; refetchModes now also calls refreshSettings." Line 96 — migration transfers transcribe_with_post_process combo to smart_mode_{active_id}. Log confirms binding=smart_mode_mode_email fires.
  implication: Card source-of-truth is the settings store bindings map. Need to confirm the key it indexes by.

- timestamp: 2026-06-05T14:40:00Z
  checked: handy_keys.rs init_shortcuts (424-471) + tauri_impl.rs:40 — how smart-mode shortcuts get registered
  found: Backend registers a smart-mode shortcut ONLY by iterating user_settings.smart_modes (line 454), building binding_key = "smart_mode_{mode.id}" and looking it up in user_settings.bindings, registering if current_binding is non-empty. So for "smart_mode_mode_email" to FIRE there MUST exist (a) a SmartMode with id == "mode_email" in smart_modes AND (b) bindings["smart_mode_mode_email"] with a non-empty combo — both read from the same load_or_create_app_settings.
  implication: Backend registration and frontend card use the IDENTICAL pair of inputs. If the backend can fire it, the card has the same data to draw it — UNLESS the card the user sees is not the mode whose id is mode_email.

- timestamp: 2026-06-05T14:55:00Z
  checked: git history of default_smart_modes() — git log -S, commit a421a61 vs 5243d60
  found: In Phase 12-01 (a421a61) default_smart_modes() seeded ALL 10 templates (mode_clean_up..mode_email..mode_translate_zh). It was reduced to seed ONLY Clean Up in Phase 13 (5243d60), with the full list renamed to smart_mode_templates(). The user UAT'd at the 13-04 checkpoint (STATE.md:15-16), i.e. ran an intermediate build that seeded mode_email.
  implication: This user's persisted smart_modes once contained the seeded mode_email; they bound "Option" to that Email card → bindings["smart_mode_mode_email"] + a smart_modes entry id mode_email were written and PERSIST.

- timestamp: 2026-06-05T15:00:00Z
  checked: SmartModeTemplatePicker.tsx:39-51 + add_smart_mode (settings.rs:1080)
  found: The picker ALWAYS calls addSmartMode, which mints a fresh id format!("mode_{timestamp}"). There is NO dedup against existing modes by name or seeded id (UAT gap "duplicate mode can be added with no visual distinction"). So re-adding "Write as Email" from the picker creates a SECOND email card with id mode_{timestamp}, distinct from the orphaned seeded mode_email.
  implication: The Email card the user now interacts with (added via picker, or re-localized seed) carries id mode_{timestamp}; its bindingKey is smart_mode_mode_{timestamp}, which is empty → chip shows "add shortcut". Meanwhile the OLD seeded mode_email + its Option binding still live in smart_modes/bindings, still register & fire, but the user's attention is on the duplicate card. The Option binding is "invisible" because it belongs to a different (orphaned/duplicate) mode_email entry than the card the user is looking at — OR mode_email was dropped from a rebuilt list while its binding survived.

## Resolution

root_cause: |
  Not a key-format mismatch (frontend "smart_mode_"+mode.id and backend smart_mode_binding_id() produce the identical key). The desync is an IDENTITY/duplication problem caused by the seed-id history plus the picker minting fresh ids:

  1. An intermediate Phase-13 (13-04) build seeded the FULL 10-mode catalogue, so this upgrader's persisted smart_modes contained a mode with id "mode_email". The user bound "Option" to it → bindings["smart_mode_mode_email"] (non-empty) + smart_modes[id=mode_email] were persisted.
  2. handy_keys init_shortcuts registers a smart-mode shortcut by iterating smart_modes and looking up "smart_mode_{id}" — so as long as mode_email + its binding persist, Option keeps registering and firing as the email smart mode (exactly the log).
  3. The Email card the user now SEES is a DIFFERENT entity: either (a) a picker-added duplicate "Write as Email" with id mode_{timestamp} (the picker always addSmartMode → new id, no dedup), whose binding key smart_mode_mode_{timestamp} is empty → chip shows "add shortcut"; or (b) the seeded mode_email entry is no longer rendered as the recognizable Email card after the seed reduction, while its binding row survives.
  Either way the persisted-and-firing binding (smart_mode_mode_email) is attached to a smart_modes entry that is not the card the user identifies as "Write as Email", so the Option chip is invisible/uneditable/unclearable from the UI even though the backend fires it.

  The reason it "still launches a transcription": SmartModeAction::start delegates to TranscribeAction for recording (STATE.md:100), so an orphaned smart-mode binding visibly behaves like a transcription trigger.
fix: |
  Diagnosis-only (find_root_cause_only). Suggested direction:
  - Make the card the single source of truth keyed by the SAME mode id the backend registers: ensure there is exactly ONE smart_modes entry per seeded mode, and that the binding key is always "smart_mode_{that id}". Add picker dedup (the documented UAT gap) so re-adding a seeded template reuses/overwrites the existing seeded id instead of minting mode_{timestamp} — this prevents the duplicate-card identity split.
  - On migration/seed-reduction, reconcile dangling bindings: drop or re-home any bindings["smart_mode_*"] whose mode id no longer exists in smart_modes (so an orphaned mode_email binding can't keep firing invisibly). Symmetric with the delete-orphan-binding fix (clear_smart_mode_binding removes the entry entirely).
  - As a backstop, the Smart Modes UI should surface ANY smart_mode_* binding present in settings even if no matching card exists, so "active but invisible" is structurally impossible.
verification: ""
files_changed: []
