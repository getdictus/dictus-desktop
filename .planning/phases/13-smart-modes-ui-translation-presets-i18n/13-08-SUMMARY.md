---
phase: 13-smart-modes-ui-translation-presets-i18n
plan: "08"
type: execute
completed: 2026-06-08
status: partial
tasks_completed: "2/2 (Task 1 PASS; Task 2 human-verify: round-3 fixes PASS, 3 new gaps surfaced)"
---

# 13-08 SUMMARY — Re-verify Phase 13 gaps in a live build (round 3)

## Outcome

**Round-3 gap fixes [G11] and [G12] both verified live PASS.** The human-verify
checkpoint also surfaced **3 new gaps** (all UX/i18n), so the phase is NOT marked
complete — routes to a round-4 gap-closure pass.

## Task 1 — Automated gates (PASS)

Run inline by the orchestrator (code-inspectable):

| Gate | Result |
|------|--------|
| `bun run lint` | ✓ clean |
| `bun run check:translations` | ✓ all 19 languages, keys present |
| `clearSmartModeBinding` / `smartModeTemplates` in bindings.ts | ✓ present |
| `cargo fmt --check` | ✓ exit 0 |
| `cargo clippy --all-targets -- -D warnings` | ✓ clean (only pre-existing tauri-patch warning) |
| `cargo test -p dictus` | ✓ 119 passed, 0 failed (incl. new prefix-collision tests) |

## Task 2 — Live human-verify (macOS Apple Silicon, FR locale)

**PASS (round-3 fixes):**
- **[G11]** combo prefix/base-key collision — `Cmd Left + 3` now rejected inline
  against an existing `Cmd Left` binding; two combos sharing only a modifier prefix
  still bind. (13-18)
- **[G12]** translation engine recommendation-only — modal is now an informational
  "Gemma 3 4B recommended" panel, no second model-switcher; enabling it does not
  re-point other Smart Modes. (13-19)

**NEW GAPS (→ /gsd:plan-phase 13 --gaps, round 4):**
- **[G13]** (minor, test 7) — the [G11] conflict error message is misleading for the
  base-key case (same prose as exact-duplicate) AND is hardcoded English in the Rust
  backend (not translated). Needs: backend returns conflict KIND + structured/codified
  error; frontend maps code → localized `t()` string with a distinct base-key message.
- **[G14]** (major, test 3) — **deferred i18n debt surfaced as English leakage.** Under
  FR, the Post-processing screen shows English ("Rewrite", "Translation", "Add shortcut",
  "New Rewrite", "Name", "Prompt", picker + modal copy). Root cause: `smartModes.*` (and
  Phase 11 `library.*`/`embedded.*`) values are English fallback in all 19 non-English
  locales (documented "translations deferred" decision). `check:translations` only checks
  key presence. User requested a COMPLETE app-wide translation audit — this reverses the
  earlier deferral decision.
- **[G15]** (minor, test 11) — Gemma 3 4B description should lead on translation strength
  ("excellent pour la traduction") rather than "excellent en français", since it is the
  recommended translation engine. Across all 20 locales.

## Decision recorded this round

- **Translate all backend-originated user-facing error messages.** Pattern: backend
  returns an error code + params (not finished English prose); frontend renders the
  localized string via `t()` with interpolation. ([G13] establishes this pattern.)
- **Reverse the "translations deferred" decision** (13-02/13-15 names-only): all
  user-facing strings must be translated in every supported language ([G14]).

## Next

`/gsd:plan-phase 13 --gaps` → round-4 gap-closure plans for [G13][G14][G15].
Phase 13 stays open until [G14] (the app-wide i18n audit) lands.
