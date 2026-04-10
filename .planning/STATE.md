---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: Auto-Update & Upstream Sync
status: defining_requirements
last_updated: "2026-04-10T16:00:00.000Z"
last_activity: 2026-04-10 — Milestone v1.1 started
progress:
  total_phases: 0
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-04-10)

**Core value:** L'application doit être identifiable et utilisable comme Dictus Desktop — pas comme Handy.
**Current focus:** Defining requirements for v1.1

## Current Position

Phase: Not started (defining requirements)
Plan: —
Status: Defining requirements
Last activity: 2026-04-10 — Milestone v1.1 started

## Accumulated Context

### From v1.0
- Overlay redesign: 84px pill, symmetric center→edges waveform, cancel pill
- Tray icon: black-on-transparent template PNG
- Handy acknowledgment added to About panel (all 20 locales)
- Upstream remote added, sync strategy documented (issue #1)
- Auto-updater disabled (no Dictus endpoint yet)
- UpdateChecker.tsx still points to cjpais/Handy releases
- release.yml asset-prefix still uses "handy"

## Deferred to V3+

- TECH-03: Rename binary from `handy` to `dictus`
- TECH-01: Rename handy_keys module
- INFR-01: CDN migration (blob.handy.computer → Dictus-owned)
- INFR-03: Code signing (macOS + Windows)
- SETT-01: Settings sections renamed
