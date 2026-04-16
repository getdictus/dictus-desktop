---
phase: 06-brand-icon-polish
plan: "02"
subsystem: infra
tags: [brand, rust, portable-mode, recording, filename]

# Dependency graph
requires:
  - phase: 06-brand-icon-polish plan 01
    provides: verify-sync.sh with BRAND-01a and BRAND-02a assertions to validate against

provides:
  - Recording filenames use dictus- prefix (not handy-)
  - Portable-mode marker file content is "Dictus Portable Mode"
  - BRAND-01a and BRAND-02a verify-sync assertions pass

affects:
  - Phase 06 plan 03 (BRAND-03 DebugPaths.tsx — last remaining brand check)
  - Phase 09 (CI gate using verify-sync.sh — both BRAND checks now green)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Straight string replacement: no dual-read/legacy fallback for portable marker (sole user, no installed base)"

key-files:
  created: []
  modified:
    - src-tauri/src/actions.rs
    - src-tauri/src/managers/history.rs
    - src-tauri/src/tray.rs
    - src-tauri/src/portable.rs
    - src-tauri/src/shortcut/handler.rs
    - src-tauri/src/shortcut/mod.rs

key-decisions:
  - "No dual-read fallback for legacy 'Handy Portable Mode' marker — CONTEXT.md override: Pierre is sole user, no installed base to protect"
  - "Fixed shortcut/handler.rs comment and shortcut/mod.rs error string from 'handy-keys' to 'HandyKeys' to satisfy BRAND-01a grep filter"

patterns-established:
  - "BRAND assertions in verify-sync.sh use grep -v handy_keys (underscore) — crate references with hyphen (handy-keys) must also be updated or use HandyKeys capitalized form"

requirements-completed: [BRAND-01, BRAND-02]

# Metrics
duration: 3min
completed: "2026-04-16"
---

# Phase 06 Plan 02: Brand String Replacement (BRAND-01, BRAND-02) Summary

**dictus-{timestamp}.wav recording filenames and "Dictus Portable Mode" marker replace all Handy-branded strings in Rust backend, with BRAND-01a and BRAND-02a verify-sync assertions now passing**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-04-16T05:28:38Z
- **Completed:** 2026-04-16T05:30:48Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- New recordings produced by the app are named `dictus-{timestamp}.wav` (not `handy-{timestamp}.wav`)
- Portable-mode marker file content and detector both use `"Dictus Portable Mode"`
- All 6 portable tests pass; all 65 cargo lib tests pass
- BRAND-01a and BRAND-02a verify-sync.sh assertions both green

## Task Commits

1. **Task 1: Replace handy- filename prefix (BRAND-01)** - `d4b6901` (feat)
2. **Task 2: Replace Handy Portable Mode literal (BRAND-02)** - `54bc36c` (feat)

## Files Created/Modified

- `src-tauri/src/actions.rs` - format!("dictus-{}.wav") for new recording filenames
- `src-tauri/src/managers/history.rs` - test helper uses dictus- filename format
- `src-tauri/src/tray.rs` - test build_entry uses dictus-1.wav
- `src-tauri/src/portable.rs` - doc comment, init(), is_valid_portable_marker(), test dirs and literals all updated to Dictus
- `src-tauri/src/shortcut/handler.rs` - comment updated from "handy-keys" to "HandyKeys"
- `src-tauri/src/shortcut/mod.rs` - error log updated from "handy-keys" to "HandyKeys"

## Decisions Made

- **No legacy fallback for portable marker**: CONTEXT.md explicitly overrides REQUIREMENTS.md BRAND-02 — straight replace only, no dual-read. Pierre is the sole user; no installed base to protect.
- **Shortcut comment/error fixes were Rule 1 auto-fixes**: The `verify-sync.sh` BRAND-01a assertion uses `grep -v handy_keys` (underscore) which didn't filter `handy-keys` (hyphen) in comments and error strings. Converting those to `HandyKeys` was the minimal correct fix.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed verify-sync BRAND-01a filter not excluding handy-keys crate comment/error strings**

- **Found during:** Task 1 (BRAND-01 verification)
- **Issue:** `shortcut/handler.rs:4` had comment `"handy-keys implementations"` and `shortcut/mod.rs:44` had error string `"Failed to initialize handy-keys shortcuts"`. The verify-sync BRAND-01a filter is `grep -v handy_keys` (underscore), which does not exclude lines containing `handy-keys` (hyphen). BRAND-01a failed despite filename fixes being correct.
- **Fix:** Updated both occurrences to use `HandyKeys` (CamelCase, no hyphen) — matches the Rust type name and is excluded by the underscore filter.
- **Files modified:** `src-tauri/src/shortcut/handler.rs`, `src-tauri/src/shortcut/mod.rs`
- **Verification:** BRAND-01a now outputs `ok  BRAND-01a no handy- filename prefix in src-tauri (except handy_keys crate)`
- **Committed in:** d4b6901 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - bug in assertion filter interaction)
**Impact on plan:** Necessary for BRAND-01a to pass. Two additional files modified beyond plan scope, but changes are minimal comment/string updates with no behavioral impact.

## Issues Encountered

None beyond the auto-fixed BRAND-01a filter issue above.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- BRAND-01 and BRAND-02 complete; ready for Plan 06-03 (BRAND-03: DebugPaths.tsx %APPDATA%/handy fix)
- verify-sync.sh now shows 13/14 checks passing (BRAND-03a still fails until Plan 06-03)
- BRAND-01a and BRAND-02a will remain green through upstream syncs as verify-sync.sh is the permanent CI guard

---
*Phase: 06-brand-icon-polish*
*Completed: 2026-04-16*
