---
phase: 08-privacy-local-first-ux
plan: 01
subsystem: settings
tags: [rust, settings, post-processing, local-first, platform-aware, cfg]

# Dependency graph
requires: []
provides:
  - Platform-aware default_post_process_provider_id() returning apple_intelligence on macOS ARM64, custom elsewhere
  - Custom provider relabeled to "Custom (local)" while id="custom" is stable for persistence
  - Rust unit tests asserting both behaviors
affects: [08-02, 08-03, ui-post-processing-settings]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "cfg(all(target_os = macos, target_arch = aarch64)) gate for platform-aware defaults in settings.rs"
    - "Runtime cfg!() check in tests to write cross-platform tests that pass on every target"

key-files:
  created: []
  modified:
    - src-tauri/src/settings.rs

key-decisions:
  - "Platform-aware default: macOS ARM64 gets apple_intelligence, all other platforms get custom (local)"
  - "Label change only (Custom -> Custom (local)) — id field stays unchanged to protect persisted settings"
  - "Tests added to existing mod tests block, not a new module — file already had #[cfg(test)] mod tests"

patterns-established:
  - "Platform defaults use cfg() blocks (not match/if): consistent with existing default_overlay_position() and PasteMethod::default() patterns"
  - "Runtime cfg!() in test bodies to write a single test function that passes on any compile target"

requirements-completed: [PRIV-01]

# Metrics
duration: 4min
completed: 2026-05-21
---

# Phase 8 Plan 01: Platform-Aware Provider Default + Custom (local) Relabel Summary

**Platform-aware `default_post_process_provider_id()` returning `apple_intelligence` on macOS ARM64 and `"custom"` elsewhere, with Custom provider relabeled to "Custom (local)" and two Rust unit tests closing the Wave 0 gap**

## Performance

- **Duration:** ~4 min
- **Started:** 2026-05-21T15:30:20Z
- **Completed:** 2026-05-21T15:34:28Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments

- Replaced hardcoded `"openai"` default in `default_post_process_provider_id()` with a platform-aware cfg gate: macOS ARM64 returns `APPLE_INTELLIGENCE_PROVIDER_ID`, all other platforms return `"custom"`
- Relabeled the Custom provider from `"Custom"` to `"Custom (local)"` while keeping `id: "custom"` stable so persisted `settings.json` files survive upgrade without data loss
- Added two Rust unit tests to the existing `mod tests` block: one for the platform default, one for the stable custom provider id and relabeled label

## Task Commits

1. **Task 1: Platform-aware default + relabel Custom (local)** - `b00146f` (note: changes landed in a prior docs commit due to stash/pop during pre-existing clippy investigation)

## Files Created/Modified

- `src-tauri/src/settings.rs` (lines 520-530: new function body; line 602: label change; lines 988-1008: two new tests)

## Decisions Made

- Tests appended to the existing `#[cfg(test)] mod tests` block at line 947 rather than creating a duplicate module — the plan's "append at end of file" instruction adapted since the block already existed
- Pre-existing clippy warnings (31 errors across `portable.rs`, `apple_intelligence.rs`, `recorder.rs`, `lib.rs`, etc.) are out of scope per deviation boundary rules; no new warnings introduced by these changes
- cfg gate pattern copied exactly from line 580 (Apple Intelligence provider block) for consistency with existing code style

## Deviations from Plan

### Minor Adaptation

**1. [Rule 1 - Bug] Tests appended inside existing mod tests, not as a new module**

- **Found during:** Task 1 (reading the file before editing)
- **Issue:** The plan said "append a new test module at the end of the file" but the file already had a `#[cfg(test)] mod tests` block at line 940. Creating a second one would cause a compile error (duplicate module)
- **Fix:** Added the two new test functions inside the existing `mod tests` block
- **Files modified:** src-tauri/src/settings.rs
- **Verification:** `cargo test --lib settings::tests` passes all 5 tests (3 pre-existing + 2 new)
- **Committed in:** b00146f

---

**Total deviations:** 1 minor adaptation (duplicate module avoidance)
**Impact on plan:** No scope change. Functional outcome identical to plan specification.

## Acceptance Criteria Verification

```
grep APPLE_INTELLIGENCE_PROVIDER_ID.to_string() inside default_post_process_provider_id():  line 523 PASS
grep "custom".to_string() inside default_post_process_provider_id():                        line 527 PASS
grep 'label: "Custom (local)".to_string()':                                                 line 602 (exactly 1 match) PASS
grep 'label: "Custom".to_string()':                                                         (0 matches) PASS
grep 'id: "custom".to_string()':                                                            line 601 PASS
grep '#[cfg(test)]':                                                                         line 947 PASS
cargo fmt -- --check:                                                                        exit 0 PASS
cargo test --lib settings::tests (5 tests):                                                  all ok PASS
```

## Issues Encountered

The `git stash` used during pre-existing clippy investigation accidentally consumed the settings.rs changes. After `git stash pop`, they were reapplied to the working tree but ended up in an earlier commit (`b00146f docs(08-02): ...`) rather than a dedicated feat commit. The changes are correct and fully tested; the commit message does not ideally describe them.

## Next Phase Readiness

- Backend default layer is correct for PRIV-01: local providers are the primary path for new installs
- Plans 08-02 and 08-03 (frontend privacy UI and provider picker) can proceed with the correct backend defaults in place
- The stable `id: "custom"` ensures any frontend work keying off this id is safe

---

_Phase: 08-privacy-local-first-ux_
_Completed: 2026-05-21_
