---
status: diagnosed
trigger: "Deleting a smart mode that had a shortcut ('Option') did not free the binding — combo stays assigned, can't be reassigned, still fires a transcription. Delete does not clean up its binding."
created: 2026-06-05T13:00:00Z
updated: 2026-06-05T13:10:00Z
---

## Current Focus

hypothesis: CONFIRMED — neither the frontend delete handler nor the backend delete_smart_mode command unregisters the OS shortcut or removes the smart_mode_{id} entry from settings.bindings. The dedicated clear_smart_mode_binding fn exists but is never invoked on delete.
test: traced full delete path frontend -> command -> backend.
expecting: (confirmed) mode removed from smart_modes but binding entry survives + OS shortcut stays registered.
next_action: return diagnosis (diagnose-only mode).

## Symptoms

expected: Deleting a smart mode clears its shortcut binding — unregister OS shortcut + remove binding entry — freeing the combo so it can be reassigned and stops firing.
actual: After deleting a mode bound to 'Option', the combo stays assigned to the deleted mode, cannot be reassigned to a new mode, and pressing Option still fires a transcription.
errors: None (log shows the binding still fires)
reproduction: UAT Test 6 — create/have a mode with a shortcut, delete it, try to reassign the combo.
started: Discovered during Phase 13 UAT re-test.

## Eliminated

- hypothesis: Orphaned binding gets re-registered on next app start (init_shortcuts re-binds it).
  evidence: handy_keys::init_shortcuts (handy_keys.rs:454-466) iterates `user_settings.smart_modes`, not the bindings map, for smart-mode registration. A deleted mode is gone from smart_modes, so it is NOT re-registered after restart. The orphaned OS shortcut persists only for the LIVE session; the orphaned settings.bindings entry persists across restarts but is inert at registration time. So the primary live symptom is in-session, not re-registration.
  timestamp: 2026-06-05T13:09:00Z

## Evidence

- timestamp: 2026-06-05T13:05:00Z
  checked: src/components/settings/post-processing/SmartModeCard.tsx handleDelete (lines 141-152)
  found: handleDelete only calls `commands.deleteSmartMode(mode.id)` then `onChanged()`. It never calls clearSmartModeBinding (which exists in bindings.ts:323 and is used by the chip's clear button at SmartModeShortcutChip.tsx:199).
  implication: Frontend delete relies entirely on the backend command to clean up the binding.

- timestamp: 2026-06-05T13:06:00Z
  checked: src-tauri/src/shortcut/mod.rs delete_smart_mode (1116-1125) + delete_mode_in_place (1045-1062)
  found: delete_smart_mode only removes the mode from settings.smart_modes (via delete_mode_in_place), reassigns smart_mode_active_id, and writes settings. It NEVER touches settings.bindings and NEVER calls unregister_shortcut. delete_mode_in_place is pure mode-list logic and is binding-agnostic.
  implication: The smart_mode_{id} binding entry survives in settings.bindings AND the OS shortcut stays registered in the running keyboard implementation.

- timestamp: 2026-06-05T13:07:00Z
  checked: clear_smart_mode_binding (mod.rs:1188-1206)
  found: This is the function that does the correct cleanup — unregister_shortcut if current_binding non-empty, then settings.bindings.remove(binding_id). It is the right primitive but is ONLY called by the chip's X (clear) button, never by delete.
  implication: The fix is to invoke this same cleanup logic from the delete path.

- timestamp: 2026-06-05T13:08:00Z
  checked: change_binding (mod.rs:108-202) — the reassignment path used when binding the combo to a NEW mode
  found: change_binding calls register_shortcut for the new combo. The orphaned shortcut from the deleted mode is STILL registered in the live handy_keys/tauri state, so registering the same combo fails (success:false / Err returned to chip), which the chip surfaces as a conflict / no attribution.
  implication: Explains "réassigner 'Option' ne marche pas" — the OS-level register collides with the still-registered orphan.

- timestamp: 2026-06-05T13:08:30Z
  checked: handy_keys.rs:454-466 + log evidence in gap-9 (Option bound to smart_mode_mode_email, fires as a smart mode)
  found: The still-registered orphan keeps firing in-session ("cliquer sur Option lance encore une transcription"). It is a smart-mode binding firing TranscribeAction, NOT the legacy transcribe_with_post_process. The "legacy post-process" wording in the user report is a mislabel — it is an orphaned smart-mode binding.
  implication: Same desync family as gap-9 but the trigger here is the DELETE path leaving the binding behind.

## Resolution

root_cause: |
  delete_smart_mode (src-tauri/src/shortcut/mod.rs:1116-1125) removes the mode from
  settings.smart_modes but does NOT clean up its shortcut binding. It never (a) calls
  unregister_shortcut to release the OS-level shortcut, nor (b) removes the
  smart_mode_{id} entry from settings.bindings. The dedicated clear_smart_mode_binding
  helper (mod.rs:1188-1206) does exactly this cleanup but is only wired to the chip's X
  button — never to delete. The frontend handleDelete (SmartModeCard.tsx:141-152) also
  does not call clearSmartModeBinding before/after deleteSmartMode. Result: the OS
  shortcut stays registered (keeps firing this session) and the orphaned bindings entry
  blocks reassigning the combo (change_binding's register_shortcut collides with the
  still-registered orphan and returns success:false, shown as a conflict).
fix: |
  (diagnose-only — not applied) On delete, perform the same cleanup as
  clear_smart_mode_binding for the deleted mode's binding id. Cleanest fix: inside
  delete_smart_mode, after a successful delete_mode_in_place, unregister + remove
  settings.bindings[smart_mode_{id}] in the same settings mutation (one read/modify/
  write). This is preferable to a frontend two-call sequence because it is atomic and
  cannot leave a half-cleaned state if the second call fails.
verification: ""
files_changed: []
