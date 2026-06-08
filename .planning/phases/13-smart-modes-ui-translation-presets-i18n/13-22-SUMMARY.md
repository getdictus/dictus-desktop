---
phase: 13-smart-modes-ui-translation-presets-i18n
plan: 22
subsystem: i18n/tooling/llm-catalogue
tags: [gap-closure, G14, G15, i18n, audit, tooling, rust]
dependency_graph:
  requires: []
  provides: [13-i18n-audit.md, --check-untranslated flag, G15-en-baseline]
  affects: [13-23-PLAN.md, check:translations CI]
tech_stack:
  added: []
  patterns: [leaf-key equality scan, opt-in CLI flag, allowlist guard]
key_files:
  created:
    - .planning/phases/13-smart-modes-ui-translation-presets-i18n/13-i18n-audit.md
  modified:
    - scripts/check-translations.ts
    - package.json
    - src/i18n/locales/en/translation.json
    - src-tauri/src/managers/llm.rs
decisions:
  - "[G14] 91 keys equal to EN in all 19 locales; 69 to translate (62 smartModes.*, 3 simulateUpdaterRestart, 2 errors.boundary, 1 translateGemma4b.description); 22 allowlisted"
  - "[G15] Gemma 3 4B description reworded to lead on translation in EN source + Rust catalogue fallback"
  - "UNTRANSLATED_ALLOWLIST covers: onboarding.models.*.name (brand names), acknowledgment titles, autoSubmit keyboard literals, apiKey/baseUrl placeholders"
  - "--check-untranslated is opt-in (default behavior unchanged) so CI stays green until 13-23 clears the debt"
metrics:
  duration: ~15 min
  completed: 2026-06-08
  tasks_completed: 3
  files_modified: 5
---

# Phase 13 Plan 22: i18n Audit, Untranslated Guard, and G15 EN Baseline Summary

**One-liner:** Committed English-fallback audit (69 keys to translate), added allowlisted --check-untranslated CI gate, and reworded Gemma 3 4B description to lead on translation quality in EN source and Rust catalogue.

## What Was Built

### Task 1 — i18n English-Fallback Audit (13-i18n-audit.md)

A one-off leaf-key equality scan across all 19 non-English locales vs. the EN reference (523 leaf keys) found **91 keys byte-equal to EN in all locales**.

The audit document at `.planning/phases/13-smart-modes-ui-translation-presets-i18n/13-i18n-audit.md` enumerates:

- **To translate (69 keys):** all 62 `smartModes.*` UI keys (sections, card.*, picker.*, shortcut.*, kind.*, translation.* including modal.*), 2 `errors.boundary.*`, 3 `settings.debug.simulateUpdaterRestart.*`, 1 `settings.postProcessing.modelsAndLocalProcessing.library.models.translateGemma4b.description`
- **Allowlist (22 keys):** 16 `onboarding.models.*.name` (Whisper/Parakeet/Moonshine/Canary/SenseVoice/GigaAM/Cohere/Breeze brand names), 2 acknowledgment titles (Handy, Whisper.cpp), 3 keyboard literals (Cmd+Enter, Super+Enter, Ctrl+Enter), `sk-...` and `https://api.openai.com/v1` placeholders
- **Partial (81 keys):** equal in some but not all locales (count only; out of [G14] core scope)

The `smartModes.defaultModes.*` keys (10 names translated in plan 13-15) are correctly absent from the translate set.

### Task 2 — `--check-untranslated` Mode for check-translations.ts

Added opt-in `--check-untranslated` flag to `scripts/check-translations.ts`:

- `UNTRANSLATED_ALLOWLIST_EXACT`: exact-match allowlist for acknowledgment titles, keyboard literals, placeholders
- Prefix+suffix rule: `onboarding.models.*.name` keys are auto-allowlisted
- Currently exits **non-zero** (AR: 105 untranslated keys, FR: 96, etc. — the pre-13-23 debt is properly detected)
- Default `bun run check:translations` (no flag) still exits 0 — existing CI unchanged
- New package.json script: `check:translations:untranslated`

### Task 3 — [G15] Gemma 3 4B EN Description Reworded

Updated both source locations:

| File | Path | New value |
|------|------|-----------|
| `src/i18n/locales/en/translation.json` | `...library.models.gemma3_4b.description` | "Excellent for translation — versatile and multilingual" |
| `src-tauri/src/managers/llm.rs` | `gemma-3-4b` catalogue entry `description` | "Excellent for translation — versatile and multilingual" |

Old wording "Versatile and multilingual — strong in many languages" is fully removed. `cargo build` clean. 13-23 will propagate this new EN string into all 19 locale files.

## Verification

- `grep -c "smartModes\." 13-i18n-audit.md` → 76 (≥60 requirement met)
- `bun run check:translations` → exits 0, "All 19 languages have complete translations!"
- `bun scripts/check-translations.ts --check-untranslated` → exits 1, AR: 62 `smartModes.*` untranslated keys confirmed
- `grep "Excellent for translation" en/translation.json llm.rs` → matches in both files
- `grep "strong in many languages" en/translation.json llm.rs` → no matches (old wording gone)
- `cargo build` → Finished dev profile in 0.94s, 0 errors

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

| Item | Status |
|------|--------|
| 13-i18n-audit.md exists | FOUND |
| scripts/check-translations.ts exists | FOUND |
| Commit 741034e (audit doc) | FOUND |
| Commit a0ad61b (check-untranslated) | FOUND |
| Commit 05d1bae (G15 EN reword) | FOUND |
