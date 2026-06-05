---
phase: 13-smart-modes-ui-translation-presets-i18n
plan: "09"
subsystem: shortcut
tags: [rust, smart-modes, shortcuts, bindings, tauri-commands, specta]

requires:
  - phase: 13-smart-modes-ui-translation-presets-i18n
    provides: "clear_smart_mode_binding (entry-removal pattern), smart_mode_binding_id, delete_mode_in_place, smart_mode_templates catalogue"

provides:
  - "resolve_seeded_id(name, kind): returns stable seed id for picker dedup"
  - "add_smart_mode: now dedup-aware — seeded templates reuse/overwrite, custom modes mint timestamps"
  - "delete_smart_mode: atomically unregisters OS shortcut + removes binding entry"
  - "reconcile_dangling_smart_mode_bindings(): drops orphaned smart_mode_* bindings on load"
  - "suspend_all_shortcuts / resume_all_shortcuts: Tauri commands for chip capture infra"
  - "suspendAllShortcuts / resumeAllShortcuts in bindings.ts for frontend use"

affects:
  - "13-11 (picker UI — depends on dedup-aware add_smart_mode)"
  - "13-12 (shortcut chip capture — depends on suspend_all/resume_all)"

tech-stack:
  added: []
  patterns:
    - "Seeded-id dedup: resolve_seeded_id anchors picker re-add to stable id, preventing card↔binding identity split"
    - "Atomic delete: fold OS unregister + binding entry removal into single settings read/modify/write"
    - "Load-time reconciliation: reconcile_dangling_smart_mode_bindings runs after migration, before init_shortcuts"

key-files:
  created: []
  modified:
    - src-tauri/src/shortcut/mod.rs
    - src-tauri/src/settings.rs
    - src-tauri/src/lib.rs
    - src/bindings.ts

key-decisions:
  - "[13-09] Reversal of [13-07]: add_smart_mode now reuses stable seed id for template re-adds (not a fresh timestamp). Overwrite branch covers 'mode still exists' case; reuse branch covers 'mode was deleted' case — both safe. Decision [13-07] concern (collision on delete+recreate) is fully resolved by these two branches."
  - "[13-09] reconcile runs at load (before init_shortcuts), not at delete-time, ensuring orphans from intermediate builds (e.g. 13-04 seed reduction) are cleaned up even without a matching delete event"
  - "[13-09] resume_all_shortcuts skips 'cancel' binding (dynamically managed during recording) and silently warns on per-binding failures so one bad binding cannot block all others"

patterns-established:
  - "dedup-aware add: always check resolve_seeded_id before minting a timestamp id in add operations"
  - "atomic binding cleanup: any operation that removes a mode must also unregister+delete its binding in the same write"
  - "load-time reconciliation: post-migration cleanup that runs before shortcut registration is the right place for orphan removal"

requirements-completed: [MODE-03, MODE-04, MODE-05]

duration: 5min
completed: 2026-06-05
---

# Phase 13 Plan 09: Backend Gap Closure (Identity/Binding Desync + Capture Infra) Summary

**Dedup-aware add_smart_mode, atomic delete binding cleanup, load-time orphan reconciliation, and suspend_all/resume_all Tauri commands for shortcut-capture chip**

## Performance

- **Duration:** 5 min
- **Started:** 2026-06-05T14:58:32Z
- **Completed:** 2026-06-05T15:03:30Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments

- `add_smart_mode` is now dedup-aware: seeded templates (matched by name+kind) reuse their stable seed id rather than minting a fresh `mode_{timestamp}`, preventing duplicate cards and card↔binding identity splits (UAT tests 4 + 9)
- `delete_smart_mode` atomically unregisters the OS shortcut and removes the `smart_mode_{id}` binding entry in a single read/modify/write, closing UAT test 6
- `reconcile_dangling_smart_mode_bindings` runs in `load_or_create_app_settings` after migration, removing orphaned `smart_mode_*` bindings before `init_shortcuts` iterates — the structural fix for UAT test 9 blocker (invisible phantom bindings from 13-04 seed reduction)
- `suspend_all_shortcuts` / `resume_all_shortcuts` Tauri commands added and registered (specta + lib.rs), providing the capture infrastructure required by the shortcut-chip in Plan 13-12

## Task Commits

1. **Task 1: Dedup-aware add_smart_mode** - `44d7b80` (feat)
2. **Task 2: Atomic delete cleanup + dangling-binding reconciliation** - `d38c7c4` (feat)
3. **Task 3 RED: Reconcile + resolve_seeded_id unit tests** - `f9ecc4a` (test)
4. **Task 3 GREEN: suspend_all_shortcuts / resume_all_shortcuts + bindings.ts** - `faf1946` (feat)

## Files Created/Modified

- `src-tauri/src/shortcut/mod.rs` — `resolve_seeded_id` helper, dedup-aware `add_smart_mode`, atomic `delete_smart_mode`, `suspend_all_shortcuts`, `resume_all_shortcuts`, unit tests for resolve_seeded_id
- `src-tauri/src/settings.rs` — `reconcile_dangling_smart_mode_bindings`, wired into `load_or_create_app_settings`, unit tests for reconcile
- `src-tauri/src/lib.rs` — `suspend_all_shortcuts` + `resume_all_shortcuts` added to `collect_commands!`
- `src/bindings.ts` — `suspendAllShortcuts` + `resumeAllShortcuts` manually added (auto-regenerated on next debug launch)

## Decisions Made

- **Reversal of [13-07]:** Plan 13-07 used fresh timestamps to avoid id collision when a user deletes a default mode and recreates it. The overwrite branch (mode still in list → update in place) and reuse branch (mode deleted → reinsert with seed id) together fully cover that collision scenario safely, making a stable seed id correct. See `key-decisions` above.
- **reconcile at load, not at delete-time:** Covers orphans from intermediate builds (e.g. 13-04 all-10-mode seed later reduced). A delete-only fix would miss orphans already persisted before this build.
- **resume_all_shortcuts skips "cancel":** The cancel binding is dynamically registered at recording start — including it in resume would register it permanently and break the dynamic lifecycle.

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

- Pre-existing failing test `managers::llm::tests::test_catalogue_has_five_models` (catalogue has 4 models, test expects 5 — from a prior catalogue change). Not introduced by this plan, logged for deferred fix.

## Next Phase Readiness

- `resolve_seeded_id` and dedup-aware `add_smart_mode` are ready for Plan 13-11 (SmartModeTemplatePicker UI).
- `suspend_all_shortcuts` / `resume_all_shortcuts` are registered and in `bindings.ts` for Plan 13-12 (shortcut chip capture UI).
- `reconcile_dangling_smart_mode_bindings` is wired and will clean up the orphaned `smart_mode_mode_email` binding on next app launch, unblocking UAT test 9.

---
*Phase: 13-smart-modes-ui-translation-presets-i18n*
*Completed: 2026-06-05*
