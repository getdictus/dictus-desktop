---
phase: 13-smart-modes-ui-translation-presets-i18n
plan: 16
subsystem: ui
tags: [react, typescript, smart-modes, shortcuts, handy-keys, rust, tauri]

# Dependency graph
requires:
  - phase: 13-smart-modes-ui-translation-presets-i18n
    provides: find_conflicting_binding raw-string collision check in set_smart_mode_binding (13-13)
  - phase: 13-smart-modes-ui-translation-presets-i18n
    provides: suspend/resume-all + chip conflict UI (13-12)
provides:
  - SmartModeShortcutChip backend handy-keys-event capture path (preserves side-distinct modifiers when keyboard_implementation === 'handy_keys')
  - Side-distinct combo string (command_right / command_left / option_right) reaches set_smart_mode_binding unchanged, so the 13-13 collision check fires correctly
  - Idempotent resume_all_shortcuts (clean-slate unregister before re-register — no "Hotkey already registered" log spam)
affects: [smart-mode-shortcut-chip, shortcut-binding, 13-08]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Dual capture path in shortcut inputs: backend handy-keys-event stream when keyboard_implementation==='handy_keys', webview keydown for 'tauri'"
    - "Pass backend hotkey_string verbatim to commit (never through getKeyName/normalizeKey) to preserve left/right modifier side"

key-files:
  created: []
  modified:
    - src/components/settings/post-processing/SmartModeShortcutChip.tsx
    - src-tauri/src/shortcut/mod.rs

key-decisions:
  - "[13-16] Chip mirrors HandyKeysShortcutInput: startHandyKeysRecording + listen('handy-keys-event') + currentKeysRef commit-on-release; webview effect gated behind !useHandyKeys as the 'tauri' fallback"
  - "[13-16] resume_all_shortcuts made idempotent via clean-slate (unregister each binding before register) so repeated resumeAll() on multiple exit paths never warns"

patterns-established:
  - "Side-distinct shortcut capture: when handy_keys, the backend recording stream is the single source of truth for the combo string; the webview path is structurally side-blind (keyboard.ts collapses MetaLeft/Right) and must not run"

requirements-completed: [MODE-04, MODE-05]

# Metrics
duration: ~3min
completed: 2026-06-08
---

# Phase 13 Plan 16: Smart Mode Chip CmdRight Binding Parity Summary

**SmartModeShortcutChip now captures via the backend handy-keys-event stream on handy_keys, preserving CmdLeft/CmdRight side end-to-end so side-distinct bindings persist and collide correctly; resume_all_shortcuts is idempotent**

## Performance

- **Duration:** ~3 min (code tasks); live verification by user
- **Completed:** 2026-06-08
- **Tasks:** 2 auto + 1 human-verify checkpoint
- **Files modified:** 2

## Accomplishments
- Added a second capture path to SmartModeShortcutChip gated on `getSetting("keyboard_implementation") === "handy_keys"`: starts `commands.startHandyKeysRecording(smart_mode_${modeId})`, subscribes to `listen<HandyKeysEvent>("handy-keys-event")`, previews on key-down, commits the raw side-distinct `hotkey_string` on key-up via `currentKeysRef`
- Existing webview keydown/keyup path retained as the `"tauri"` fallback, gated with `!useHandyKeys`
- Side-distinct combo (e.g. `command_right`) reaches `setSmartModeBinding` unchanged → the 13-13 `find_conflicting_binding` raw-string check now matches the General-tab CmdRight dictation and blocks it inline
- `resume_all_shortcuts` made idempotent (clean-slate unregister before re-register) — no more `Hotkey already registered` log spam from resumeAll() firing on multiple exit paths

## Task Commits

1. **Task 1: Add backend handy-keys-event capture path to SmartModeShortcutChip** - `366fe9a` (feat)
2. **Task 2: Make resume_all_shortcuts idempotent (tolerate already-registered hotkeys)** - `8a7c90e` (fix)

## Files Created/Modified
- `src/components/settings/post-processing/SmartModeShortcutChip.tsx` - Dual capture path; handy_keys backend stream preserves side-distinct modifiers; webview effect gated on `!useHandyKeys`
- `src-tauri/src/shortcut/mod.rs` - `resume_all_shortcuts` clean-slate idempotency

## Decisions Made
- Clean-slate (unregister-then-register) chosen over log-downgrade for `resume_all_shortcuts` idempotency — guarantees register never hits the "already registered" state rather than masking it.

## Deviations from Plan
None - plan executed exactly as written.

## Issues Encountered
None during execution.

## Live Verification (Task 3 checkpoint)
**Result: PASSED** (user, macOS handy_keys, 2026-06-08). CmdLeft binds as "Left Command" (side-distinct), CmdRight is blocked as a conflict with General-tab dictation, side is preserved end-to-end, no log spam.

**New gap discovered during live verify (out of 13-16 scope) → [G11]:** A combo whose **prefix key is itself a bound shortcut** is silently unreachable — e.g. with `CmdLeft` bound to mode A, binding `CmdLeft+1` to mode B never fires B because the OS triggers A on the CmdLeft press before the `+1` lands. Needs a prefix-collision rule extending `find_conflicting_binding`. Recorded as a new failed gap for `/gsd:plan-phase 13 --gaps`.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Gap [E9] closed: side-distinct modifier capture + collision parity verified live.
- Follow-up gap [G11] (combo prefix-key collision) queued for the next gap-closure round.

---
*Phase: 13-smart-modes-ui-translation-presets-i18n*
*Completed: 2026-06-08*
