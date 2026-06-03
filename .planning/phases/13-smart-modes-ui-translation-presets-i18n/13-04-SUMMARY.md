---
phase: 13-smart-modes-ui-translation-presets-i18n
plan: "04"
subsystem: frontend/settings/post-processing
tags: [smart-modes, ui, integration, cleanup]
dependency_graph:
  requires: ["13-03"]
  provides: ["MODE-03", "MODE-04", "MODE-05", "MODE-06", "TRANS-01", "TRANS-02"]
  affects: [PostProcessingSettings, SmartModesSection]
tech_stack:
  added: []
  patterns: [React.memo, functional-component]
key_files:
  created: []
  modified:
    - src/components/settings/post-processing/PostProcessingSettings.tsx
  deleted:
    - src/components/settings/PostProcessingSettingsPrompts.tsx
decisions:
  - "Removed entire hotkey SettingsGroup (confirmed it contained only transcribe_with_post_process ShortcutInput)"
  - "Deleted PostProcessingSettingsPrompts.tsx re-export file and index.ts entry (no external consumers)"
  - "SmartModesSection renders directly without wrapping SettingsGroup (it manages its own headers per 13-03 design)"
metrics:
  duration_minutes: 8
  completed_date: "2026-06-03"
  tasks_completed: 1
  tasks_total: 2
  files_changed: 3
---

# Phase 13 Plan 04: Smart Modes Integration Summary

**One-liner:** Replace legacy single-prompt PostProcessing UI with SmartModesSection card list, removing the hotkey group and all dead Prompts component code.

## Status

Partial — Task 1 complete and committed; Task 2 (E2E human-verify checkpoint) awaits user verification.

## Completed Tasks

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Remove legacy prompt UI and mount SmartModesSection | 3c8e969 | PostProcessingSettings.tsx (modified), PostProcessingSettingsPrompts.tsx (deleted), index.ts (modified) |

## Pending Tasks

| Task | Name | Type | Status |
|------|------|------|--------|
| 2 | E2E Smart Mode flow verification | checkpoint:human-verify | Awaiting user verification |

## What Was Done

**Task 1 — Remove legacy prompt UI and mount SmartModesSection:**

1. Removed the `transcribe_with_post_process` ShortcutInput and its parent "Hotkey" SettingsGroup (verified it was the sole row).
2. Removed the entire `PostProcessingSettingsPromptsComponent` (270 lines) and its `PostProcessingSettingsPrompts` React.memo export.
3. Added `import { SmartModesSection } from "./SmartModesSection"` and rendered `<SmartModesSection />` directly below the Engine SettingsGroup.
4. Cleaned up all dead imports: `commands`, `Dropdown`, `Textarea`, `Input`, `ShortcutInput`.
5. Deleted `src/components/settings/PostProcessingSettingsPrompts.tsx` (re-export shim with no consumers).
6. Removed the corresponding line from `src/components/settings/index.ts`.

Lint: clean (0 errors). Translation check: all 19 languages complete.

## Deviations from Plan

None — plan executed as written. Dead import cleanup (step 3 in plan action) matched exactly what the grep showed.

## Self-Check

- [x] Task 1 committed: `3c8e969`
- [x] `grep -q "SmartModesSection" PostProcessingSettings.tsx` exits 0
- [x] `grep -q 'shortcutId="transcribe_with_post_process"'` exits 1
- [x] `grep -q "<PostProcessingSettingsPrompts"` exits 1
- [x] `bun run lint` exits 0
- [x] `bun run check:translations` exits 0

## Self-Check: PASSED
