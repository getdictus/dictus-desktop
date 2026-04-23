---
gsd_state_version: 1.0
milestone: v1.2
milestone_name: Polish & Automation
status: in_progress
stopped_at: Phase 7 complete — ready for Phase 8
last_updated: "2026-04-23T22:40:00.000Z"
last_activity: 2026-04-23 — Plan 07-01 complete (SHUT-01/02/03); Phase 6.1 icon regression fixed a posteriori (commit 34de33e)
progress:
  total_phases: 5
  completed_phases: 2
  total_plans: 5
  completed_plans: 5
  percent: 40
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-04-15 at v1.2 kickoff)

**Core value:** L'application doit être identifiable et utilisable comme Dictus Desktop — pas comme Handy — et rester vivante sans décrocher du upstream Handy.
**Current focus:** v1.2 Phase 8 — Privacy / Local-First UX (next up, ready to discuss/plan)

## Current Position

Phase: 7 of 10 overall (Phase 2 of 5 in v1.2) — complete
Plan: 07-01 complete (SHUT-01/02/03)
Status: Phase 7 complete; validation window closed with no crash reproduction
Last activity: 2026-04-23 — Plan 07-01 SUMMARY written; Phase 6.1 icon regression fixed (commit 34de33e)

Progress: [████░░░░░░] 40% (v1.2 — 2 of 5 phases complete, 5 of 5 planned plans executed)

## Performance Metrics

**Velocity:**
- Total plans completed: 15 (v1.0 + v1.1 combined)
- Average duration: unknown
- Total execution time: unknown

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 1-3 (v1.0) | 8 | - | - |
| 4-5 (v1.1) | 7 | - | - |

*Updated after each plan completion*
| Phase 06-brand-icon-polish P01 | 3 | 3 tasks | 2 files |
| Phase 06-brand-icon-polish P02 | 3 | 2 tasks | 6 files |
| Phase 06-brand-icon-polish P03 | 8 | 1 tasks | 1 files |
| Phase 06-brand-icon-polish P04 | ~60min | 4 tasks | 12 files |
| Phase 07-macos-clean-shutdown P01 | ~1h coding + multi-day validation | 4 tasks | 7 files |

## Accumulated Context

### Decisions

- v1.2: Phase 6 must land before Phase 9 — verify-sync.sh extended assertions needed before CI gate goes live
- v1.2: Phases 6, 7, 8 are independent; can run on separate branches simultaneously
- v1.2: Phase 10 hard-depends on Phase 9 (no labeled draft PRs = agent never fires)
- [Phase 05]: UPSTREAM.md §6 post-sync gate missing UPDT-03/UPDT-05 — SYNC-07 in Phase 9 closes this
- [Phase 06-brand-icon-polish]: SYNC-06 completed in Phase 6 Plan 01 ahead of Phase 9 schedule; .github/scripts/verify-sync.sh is now the permanent script location
- [Phase 06-brand-icon-polish]: ICON-02a passes immediately — icon.ico already has all 6 required layer sizes; BRAND-01a/02a/03a expected to fail until Plans 06-02/03 land
- [Phase 06-brand-icon-polish]: No dual-read fallback for Handy Portable Mode marker: CONTEXT.md override, Pierre is sole user
- [Phase 06-brand-icon-polish]: BRAND-01a filter uses underscore grep -v handy_keys: hyphenated handy-keys in comments must use HandyKeys capitalized form
- [Phase 06-brand-icon-polish]: Pre-compute modelsPath/settingsPath as template literals before JSX render to satisfy i18next/no-literal-string ESLint rule when concatenating path suffixes
- [Phase 06-brand-icon-polish]: Reused existing get_app_dir_path Tauri command for DebugPaths — no new backend command created (RESEARCH.md Pitfall 1 honored)
  - [Phase 06-brand-icon-polish P04]: ICON-01 resolved via opaque-navy-tile variant — source PNG has zero transparent pixels, making Linux black-corners artifact physically impossible (approved by Pierre as Option A)
  - [Phase 06-brand-icon-polish P04]: tauri icon CLI does not output 256x256.png/512x512.png; ImageMagick fallback required (convert from 1024 source)
  - [Phase 06-brand-icon-polish P04]: Linux (ICON-01) and Windows (ICON-02) visual verification deferred — macOS approved; automated backstops in place
  - [Phase 06.1 follow-up]: Icon regression fixed 2026-04-23 (commit 34de33e) — Phase 6 rasterization dropped the middle blue bar; re-rendered dictus-brand source and regenerated full platform icon set; legacy Handy `logo.png` removed
  - [Phase 07-macos-clean-shutdown P01]: Path (a) graceful cleanup chosen over path (b) std::process::exit — diagnosis pointed at tauri-plugin-global-shortcut Drop releasing CGEventTap from atexit on main thread; flush_and_exit helper releases CGEventTap while runloop is still alive
  - [Phase 07-macos-clean-shutdown P01]: simulate_updater_restart ships in release builds, UI-gated by settings.debug_mode (not #[cfg(debug_assertions)]) so production installs can validate updater-relaunch path
  - [Phase 07-macos-clean-shutdown P01]: Crash non-reproducible after fix over multi-day validation window; phase closed without a second-reproduction checkpoint

### Pending Todos

3 pending — see `.planning/todos/pending/` for details.
(2 moved to done/ on 2026-04-23: icon regression + macOS quit-unexpectedly)

### Blockers/Concerns

Carried from v1.1 audit:
- UPSTREAM.md §6 post-sync gate missing UPDT-03/UPDT-05 re-assertion (addressed by SYNC-07 in Phase 9)
- Phase 5 VALIDATION.md draft → run `/gsd:validate-phase 5` to close
- `blob.handy.computer` CDN for onnxruntime (INFR-01, deferred)
- Windows builds unsigned at OS level (INFR-03, deferred)

v1.2-specific research flags:
- Phase 9: Test `peter-evans/create-pull-request@v8` idempotency with `workflow_dispatch` before enabling weekly cron
- Phase 10: Validate `claude setup-token` OAuth path before building agent workflow

## Session Continuity

Last session: 2026-04-16T10:04:33.074Z
Stopped at: Phase 7 context gathered
Resume file: .planning/phases/07-macos-clean-shutdown/07-CONTEXT.md
