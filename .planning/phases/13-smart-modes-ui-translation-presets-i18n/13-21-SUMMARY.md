---
phase: 13-smart-modes-ui-translation-presets-i18n
plan: 21
subsystem: shortcut-conflict-ui
tags: [gap-closure, G13, frontend, react, i18n]
dependency_graph:
  requires: [13-20]
  provides: [localizeBindingError, shortcutConflictBase-key, shortcutBaseHint-key]
  affects: [SmartModeShortcutChip.tsx, all-20-locales]
tech_stack:
  added: []
  patterns: [structured-error-code-parsing, t()-interpolation, pipe-delimited-payload-split]
key_files:
  created: []
  modified:
    - src/components/settings/post-processing/SmartModeShortcutChip.tsx
    - src/i18n/locales/en/translation.json
    - src/i18n/locales/ar/translation.json
    - src/i18n/locales/bg/translation.json
    - src/i18n/locales/cs/translation.json
    - src/i18n/locales/de/translation.json
    - src/i18n/locales/es/translation.json
    - src/i18n/locales/fr/translation.json
    - src/i18n/locales/he/translation.json
    - src/i18n/locales/it/translation.json
    - src/i18n/locales/ja/translation.json
    - src/i18n/locales/ko/translation.json
    - src/i18n/locales/pl/translation.json
    - src/i18n/locales/pt/translation.json
    - src/i18n/locales/ru/translation.json
    - src/i18n/locales/sv/translation.json
    - src/i18n/locales/tr/translation.json
    - src/i18n/locales/uk/translation.json
    - src/i18n/locales/vi/translation.json
    - src/i18n/locales/zh/translation.json
    - src/i18n/locales/zh-TW/translation.json
decisions:
  - localizeBindingError() is a module-level pure function (takes TFunction arg) so it is testable without rendering the component
  - Missing base segment in split() (exact_duplicate case has 3 parts) handled by destructuring default "" — no guard needed
  - shortcutBaseHint added to en now (not rendered yet) so 13-23's translation set is complete without requiring a second locale sweep
  - English fallback used verbatim in all 19 non-English locales; quality translations deferred to 13-23
metrics:
  duration: "~3 minutes"
  completed: "2026-06-08T19:09:36Z"
  tasks_completed: 2
  files_modified: 21
---

# Phase 13 Plan 21: G13 Frontend Conflict Message Localization Summary

**One-liner:** `SmartModeShortcutChip` now parses the structured `SHORTCUT_CONFLICT|<code>|<name>[|<base>]` backend payload into a localized, situation-specific `t()` message — exact-duplicate and base-key overlap render two distinct strings across all 20 locales.

## What Was Built

Gap [G13] frontend closure: the chip no longer renders the raw `SHORTCUT_CONFLICT|...` backend payload verbatim. A `localizeBindingError()` helper maps the structured error code to the appropriate localized string with `{{name}}` and `{{base}}` interpolation.

### localizeBindingError helper (Task 1)

Added `import type { TFunction } from "i18next"` and a module-level pure function above the component in `SmartModeShortcutChip.tsx`:

```tsx
function localizeBindingError(
  raw: string | null | undefined,
  t: TFunction,
): string {
  if (raw && raw.startsWith("SHORTCUT_CONFLICT|")) {
    const [, code, name = "", base = ""] = raw.split("|");
    if (code === "base_overlap") {
      return t("smartModes.card.shortcutConflictBase", { name, base });
    }
    return t("smartModes.card.shortcutConflict", { name });
  }
  return raw ?? t("smartModes.card.shortcutConflict", { name: "" });
}
```

The `commitCombo` else branch now calls `setConflict(localizeBindingError(response.error, t))` instead of the previous verbatim assignment.

### New i18n keys (Task 2)

Added to `smartModes.card` in `en/translation.json` adjacent to the existing `shortcutConflict` key:

- `shortcutConflictBase` — "The start of this shortcut ({{base}}) is already used by {{name}} — it would fire before the next key"
- `shortcutBaseHint` — "A shortcut whose start matches another shortcut can't work — the first key fires before you finish." (not yet rendered; pre-populated for 13-23 quality translations)

Both keys propagated to all 19 non-English locales with English fallback values. `check:translations` exits 0.

## Tasks

| # | Name | Status | Commit |
|---|------|--------|--------|
| 1 | Map the SHORTCUT_CONFLICT payload to a localized t() string in the chip | DONE | f1be5b5 |
| 2 | Add shortcutConflictBase (+ optional advisory note) key to en and all 19 locales | DONE | af69878 |

## Test Coverage

- `bun run lint` exits 0
- `bunx tsc --noEmit` exits 0
- `bun run check:translations` exits 0 (All 19 languages have complete translations)

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

- src/components/settings/post-processing/SmartModeShortcutChip.tsx: FOUND
- src/i18n/locales/en/translation.json: FOUND
- commit f1be5b5: FOUND
- commit af69878: FOUND
- `grep -n "SHORTCUT_CONFLICT" SmartModeShortcutChip.tsx` → lines 54, 55, 63: PASS
- `grep -n "shortcutConflictBase" SmartModeShortcutChip.tsx` → line 66: PASS
- `grep -rl "shortcutConflictBase" src/i18n/locales/ | wc -l` → 20: PASS
- `grep -rl "shortcutBaseHint" src/i18n/locales/ | wc -l` → 20: PASS
- `bun run check:translations` → "All 19 languages have complete translations": PASS
- `bun run lint` → exit 0: PASS
- `bunx tsc --noEmit` → exit 0: PASS
