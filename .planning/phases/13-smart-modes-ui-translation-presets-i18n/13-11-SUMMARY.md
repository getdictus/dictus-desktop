---
phase: 13-smart-modes-ui-translation-presets-i18n
plan: 11
subsystem: smart-modes-ui
tags: [gap-closure, uat, picker, dedup, i18n, custom-form]
dependency_graph:
  requires: [13-09]
  provides: [picker-dedup-ux, output-hint-ux]
  affects: [SmartModeTemplatePicker, SmartModeCard, translation-locales]
tech_stack:
  added: []
  patterns: [name-based-dedup, useMemo-set, i18n-english-fallback]
key_files:
  created: []
  modified:
    - src/components/settings/post-processing/SmartModeTemplatePicker.tsx
    - src/components/settings/post-processing/SmartModeCard.tsx
    - src/i18n/locales/en/translation.json
    - "src/i18n/locales/{ar,bg,cs,de,es,fr,he,it,ja,ko,pl,pt,ru,sv,tr,uk,vi,zh,zh-TW}/translation.json"
decisions:
  - "[13-11] Picker fetches listSmartModes() on open (not prop-plumbed) — avoids prop threading and keeps dedup logic self-contained"
  - "[13-11] currentNames Set memoized with useMemo so badge computation and handlePickTemplate use the same derived value"
  - "[13-11] outputHint and promptPlaceholder gated to kind === 'rewrite' (reuses existing rewrite-only Textarea block) — translation form uses target_language not prompt"
  - "[13-11] 5 new i18n keys added with English fallback to all 19 non-English locales per Phase 13 precedent"
metrics:
  duration: "2m"
  completed_date: "2026-06-05"
  tasks_completed: 2
  files_changed: 21
requirements: [MODE-03, MODE-06]
---

# Phase 13 Plan 11: Frontend Dedup UX + ${output} Discoverability Summary

Frontend gap closure for UAT test 4: picker duplicate handling and `${output}` discoverability in the custom rewrite form. Two-task plan targeting `SmartModeTemplatePicker.tsx`, `SmartModeCard.tsx`, and all 20 locale files.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Name-based dedup + overwrite-warn in picker | b162cfb | SmartModeTemplatePicker.tsx, SmartModeCard.tsx |
| 2 | Custom-form placeholders + ${output} helper | d73eaf9 | SmartModeCard.tsx, en/translation.json + 19 locales |

## What Was Built

### Task 1 — Picker dedup fix

The picker's previous dedup guard (`existingModeIds.includes(template.id)`) was dead code: seeded modes added via the picker got timestamp ids (not seed ids), so the check never fired and duplicates were silently added.

Fix:
- Exported `SEEDED_MODE_DEFAULT_NAME` from `SmartModeCard.tsx` (was private `const`)
- Picker now calls `listSmartModes()` alongside `smartModeTemplates()` on open, stored in `currentModes` state
- `currentNames` Set (memoized) localizes seeded-mode names the same way SmartModeCard does, enabling accurate name-based dedup
- "Added" badge now actually appears for present templates
- `handlePickTemplate` shows a native `ask()` confirm dialog before overwriting — uses new `smartModes.picker.overwriteTitle` / `overwriteConfirm` keys
- Backend (13-09) handles the overwrite transparently: `addSmartMode` with same name+kind reuses the seeded id

### Task 2 — Custom form discoverability

Custom/edit form previously had no placeholders or hint about the `${output}` transcript token.

Fix:
- Name `<Input>`: added `placeholder={t("smartModes.card.namePlaceholder")}` → shows "e.g. Make Concise"
- Prompt `<Textarea>`: added `placeholder={t("smartModes.card.promptPlaceholder")}` → shows a concrete example ending with `${output}`
- Added `<p className="text-xs text-mid-gray/70">` helper line under the Textarea showing `outputHint` key (contains the literal `${output}` inside the translation value, not in JSX — eslint rule satisfied)
- Both placeholder and hint gated to the `kind === "rewrite"` block (translation form has no prompt)

### i18n

5 new keys added to `en/translation.json`:
- `smartModes.card.namePlaceholder`
- `smartModes.card.promptPlaceholder`
- `smartModes.card.outputHint`
- `smartModes.picker.overwriteTitle`
- `smartModes.picker.overwriteConfirm`

Mirrored verbatim to all 19 non-English locales per Phase 13 English-fallback pattern.

## Verification

- `bun run lint` — passes (no hardcoded JSX strings)
- `bun run check:translations` — passes ("All 19 languages have complete translations")

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check

Files created/modified:

- `src/components/settings/post-processing/SmartModeTemplatePicker.tsx` — modified (name-based dedup, overwrite warn)
- `src/components/settings/post-processing/SmartModeCard.tsx` — modified (export SEEDED_MODE_DEFAULT_NAME, placeholders, outputHint)
- `src/i18n/locales/en/translation.json` — modified (5 new keys)
- All 19 non-English locale files — modified (5 new keys each)

Commits:
- `b162cfb` — Task 1
- `d73eaf9` — Task 2

## Self-Check: PASSED
