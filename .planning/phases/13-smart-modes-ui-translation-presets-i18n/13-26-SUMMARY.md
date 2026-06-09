---
phase: 13-smart-modes-ui-translation-presets-i18n
plan: 26
subsystem: ui
tags: [react, tailwind, i18n, layout, conflict-error, shortcut-chip]

# Dependency graph
requires:
  - phase: 13-smart-modes-ui-translation-presets-i18n
    plan: 25
    provides: "G16 fix — localizeSmartModeName shared helper for localized conflict message names"
provides:
  - "[G17] Shortcut conflict error hoisted out of shrink-0 chip column onto full-width card row with whitespace-normal break-words"
  - "onConflictChange optional callback prop on SmartModeShortcutChip for lifting conflict state to parent"
  - "SmartModeCard owns shortcutConflict state and renders role=alert p below the top row"
affects: [SmartModeCard, SmartModeShortcutChip, shortcut-conflict-layout, i18n-overflow]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Conflict error owned by card (not chip): chip fires onConflictChange callback, card renders full-width error row beneath top flex row"
    - "Full-width wrapping error: w-full whitespace-normal break-words inside flex-col card prevents overflow regardless of locale text length"

key-files:
  created: []
  modified:
    - src/components/settings/post-processing/SmartModeShortcutChip.tsx
    - src/components/settings/post-processing/SmartModeCard.tsx

key-decisions:
  - "[G17] Conflict error hoisted OUT of shrink-0 chip column onto own full-width card row via onConflictChange callback — guarantees no overlap with card title or 'Ajouter un raccourci' placeholder at any locale text length"
  - "onConflictChange is optional on SmartModeShortcutChipProps — backward compatible with any future consumers that do not need the hoisted error"
  - "Chip return changed from <div> wrapper with inline <p> to bare <>{renderChip()}</> — chip no longer owns the error UI"

patterns-established:
  - "Lift-and-own error UI: child component fires callback with error string; parent component renders the error in a layout-appropriate position"

requirements-completed: [MODE-05, L10N-01]

# Metrics
duration: 3min
completed: 2026-06-09
---

# Phase 13 Plan 26: [G17] Shortcut Conflict Layout Fix Summary

**Conflict error lifted from shrink-0 chip column to full-width card row via onConflictChange callback, eliminating locale text overflow that overwrote the card title and 'Ajouter un raccourci' placeholder**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-06-09T10:36:30Z
- **Completed:** 2026-06-09T10:39:13Z
- **Tasks:** 1
- **Files modified:** 2

## Accomplishments
- Removed inline `<p role="alert">` from `SmartModeShortcutChip` return — chip now renders only the chip itself (`<>{renderChip()}</>`)
- Added `onConflictChange?: (message: string | null) => void` prop + reporting `useEffect` to chip so conflict state bubbles up to the card
- Added `shortcutConflict` state to `SmartModeCard`, passed `onConflictChange={setShortcutConflict}` to the chip
- Added full-width sibling row `<p className="w-full text-xs text-red-400 whitespace-normal break-words" role="alert" dir="auto">` below the top row inside the card's `flex flex-col gap-2` container
- `bun run lint` exits 0; chip is only consumed by `SmartModeCard.tsx` (no other consumers)

## Task Commits

Each task was committed atomically:

1. **Task 1: Lift the conflict error to a full-width card row (chip + card)** - `7c70282` (feat)

**Plan metadata:** (docs commit — see below)

## Files Created/Modified
- `src/components/settings/post-processing/SmartModeShortcutChip.tsx` - Added `onConflictChange` optional prop, reporting effect, changed return to `<>{renderChip()}</>`
- `src/components/settings/post-processing/SmartModeCard.tsx` - Added `shortcutConflict` state, passes `onConflictChange` to chip, renders full-width conflict row beneath top row

## Decisions Made
- Hoisted error to card rather than fixing with `max-w-*`/`overflow-hidden` on the chip — the plan's preferred approach, which guarantees wrapping at any locale text length without width-constrained truncation
- `onConflictChange` optional (not required) so the contract is backward compatible

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- [G17] layout defect closed
- Phase 13 UAT round 5 needed to confirm both [G16] (13-25) and [G17] (this plan) live under FR locale
- After UAT passes, Phase 13 can be marked complete and milestone v1.3 final build/release validation can proceed

## Self-Check: PASSED
- `7c70282` commit exists: confirmed
- `src/components/settings/post-processing/SmartModeShortcutChip.tsx` modified: confirmed
- `src/components/settings/post-processing/SmartModeCard.tsx` modified: confirmed

---
*Phase: 13-smart-modes-ui-translation-presets-i18n*
*Completed: 2026-06-09*
