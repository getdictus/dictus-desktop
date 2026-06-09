---
phase: 13-smart-modes-ui-translation-presets-i18n
verified: 2026-06-09T11:00:00Z
status: passed
score: 14/14 must-haves verified
human_verified: 2026-06-09T11:30:00Z  # [G16] localized name + [G17] no-overflow confirmed live under FR by user
re_verification: false
human_verification:
  - test: "Under FR locale, bind a shortcut that exactly duplicates an existing seeded Smart Mode (e.g. 'Nettoyage'). Observe the conflict message."
    expected: "Message reads 'Déjà utilisé par : Nettoyage' (French localized name), NOT 'Déjà utilisé par : Clean Up'."
    why_human: "localizeSmartModeName logic resolves seeded-and-pristine names client-side; correctness depends on the FR translation key being applied at runtime, which requires a live app with active locale."
  - test: "Under FR locale, bind a shortcut whose base key overlaps a seeded Smart Mode. Observe the full base-overlap conflict message."
    expected: "Full localized base-overlap sentence 'Le début de ce raccourci (…) est déjà utilisé par Nettoyage — …' renders on its own full-width row BELOW the card's top row. Card title ('Nettoyage') and 'Ajouter un raccourci' placeholder remain fully visible and are not overpainted. The error wraps within the card width."
    why_human: "The layout fix (G17) is a CSS-class + component-tree structural change. No overflow at any locale text length can only be confirmed visually against the live rendered card under FR."
---

# Phase 13: Smart Modes UI / Translation Presets / i18n Verification Report

**Phase Goal:** Users interact with Smart Modes through a visual card list UI, can create/edit/delete modes and bind shortcuts from the settings panel, translation is a first-class preset group, and all strings are localized across 20 locales.

**Verified:** 2026-06-09T11:00:00Z
**Status:** passed (automated checks passed; 2 live-visual items confirmed by user under FR locale 2026-06-09)
**Re-verification:** No — initial verification

This round targets the two gap-closure plans (13-25 [G16] and 13-26 [G17]) as the final outstanding items for Phase 13. All broader phase requirements (MODE-03 through TRANS-02 and L10N-01) are addressed by earlier plans and confirmed by artifacts and the translation check.

---

## Goal Achievement

### Observable Truths

| #  | Truth | Status | Evidence |
|----|-------|--------|----------|
| 1  | Smart Modes are presented as a visual card list (MODE-06) | VERIFIED | `SmartModesSection.tsx` maps over modes and renders `<SmartModeCard>` (lines 128, 138, 177, 213); card component is substantive (400+ lines) |
| 2  | User can create, edit, and delete Smart Modes (MODE-03) | VERIFIED | `SmartModeCard.tsx` has full edit form with draft state, save/discard handlers; delete button wired to `handleDelete` |
| 3  | User can assign a distinct global shortcut per Smart Mode (MODE-04) | VERIFIED | `SmartModeShortcutChip` in collapsed card wires `set_smart_mode_binding` command; `currentBinding` derived from settings |
| 4  | Translation is a first-class Smart Mode preset (TRANS-01) | VERIFIED | `SEEDED_MODE_ID_TO_I18N_KEY` and `SEEDED_MODE_DEFAULT_NAME` in `SmartModeCard.tsx` include `mode_translate_en/es/fr/zh`; translate presets rendered via `SmartModesSection` |
| 5  | Translation runs via the embedded LLM (TRANS-02) | VERIFIED | Architecture confirmed in prior phases (Phases 11–12); translate Smart Modes use the generic LLM prompt path; no Whisper translate task dependency |
| 6  | Shortcut conflicts are detected and surfaced with inline warning (MODE-05) | VERIFIED | `find_conflicting_binding` returns `ConflictKind`; `set_smart_mode_binding` builds structured payload; chip maps payload to `localizeBindingError`; card renders the alert row |
| 7  | All new strings are localized across 20 locales (L10N-01) | VERIFIED | `bun run check:translations` exits 0 — all 19 non-EN languages have complete key coverage; 20 locale directories confirmed |
| 8  | [G16] Backend SHORTCUT_CONFLICT payload carries binding id before name | VERIFIED | `mod.rs` line 1432: `SHORTCUT_CONFLICT|exact_duplicate|{}|{}` with `other_id_for_payload, other_name`; line 1439: `SHORTCUT_CONFLICT|base_overlap|{}|{}|{}` with id, name, base; field order locked by unit test |
| 9  | [G16] Unit test `conflict_payload_carries_binding_id_before_name` passes | VERIFIED | `cargo test --lib conflict_payload_carries_binding_id_before_name` → `test result: ok. 1 passed` |
| 10 | [G16] Shared `localizeSmartModeName` helper is exported and card delegates to it | VERIFIED | `SmartModeCard.tsx` line 49: `export function localizeSmartModeName(...)`; line 87: `const displayName = mode ? localizeSmartModeName(mode.id, mode.name, t) : ""` |
| 11 | [G16] Chip imports helper and resolves localized name before interpolation | VERIFIED | `SmartModeShortcutChip.tsx` line 7: import; line 76–78: `id.startsWith("smart_mode_") ? localizeSmartModeName(id.slice("smart_mode_".length), name, t) : name` |
| 12 | [G17] Conflict error removed from chip return; chip renders only `{renderChip()}` | VERIFIED | `SmartModeShortcutChip.tsx` line 389: `return <>{renderChip()}</>`; `role="alert"` grep in chip returns nothing |
| 13 | [G17] Full-width wrapping conflict row added to card below top row | VERIFIED | `SmartModeCard.tsx` lines 370–378: `<p role="alert" dir="auto" className="w-full text-xs text-red-400 whitespace-normal break-words">`; card outer is `flex flex-col gap-2` |
| 14 | [G16/G17] Live FR: localized name rendered and no overflow of base-overlap message | HUMAN NEEDED | Code structure is correct; runtime rendering under FR locale requires live visual test |

**Score:** 13/14 truths verified automated; 1 deferred to human (covers 2 human_verification items above)

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/src/shortcut/mod.rs` | SHORTCUT_CONFLICT payload with `\|<id>\|<name>` field order; unit test | VERIFIED | Lines 1416–1448 contain new payload; test at line 1861 |
| `src/components/settings/post-processing/SmartModeCard.tsx` | `export function localizeSmartModeName`; `shortcutConflict` state; full-width conflict row | VERIFIED | Lines 49–58 (helper), 84 (state), 370–378 (conflict row) |
| `src/components/settings/post-processing/SmartModeShortcutChip.tsx` | `localizeSmartModeName` import; `onConflictChange` prop; `return <>{renderChip()}</>` | VERIFIED | Lines 7 (import), 28 (prop), 135–137 (effect), 389 (return) |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `mod.rs set_smart_mode_binding` | `SmartModeShortcutChip localizeBindingError` | `SHORTCUT_CONFLICT\|<code>\|<id>\|<name>[|\<base>]` payload | VERIFIED | Both exact_duplicate (4 fields) and base_overlap (5 fields) formats confirmed in mod.rs; chip parses `parts[1..4]` |
| `SmartModeShortcutChip localizeBindingError` | `SmartModeCard localizeSmartModeName` | shared import — strips `smart_mode_` prefix, applies seeded-and-pristine rule | VERIFIED | `import { localizeSmartModeName }` at chip line 7; `id.slice("smart_mode_".length)` at line 77 |
| `SmartModeShortcutChip conflict state` | `SmartModeCard full-width error row` | `onConflictChange` callback prop | VERIFIED | Chip fires `onConflictChange?.(conflict)` via `useEffect` (lines 135–137); card passes `onConflictChange={setShortcutConflict}` at line 336; card renders `{shortcutConflict && <p ...>}` at line 370 |

---

### Requirements Coverage

| Requirement | Source Plans | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| MODE-03 | 13-01 through 13-09 | Create/edit/delete Smart Modes | SATISFIED | Edit form + delete handler in SmartModeCard.tsx; commands wired in SmartModesSection.tsx |
| MODE-04 | 13-10 through 13-18; 13-25 | Assign distinct global shortcut per mode | SATISFIED | SmartModeShortcutChip wires `set_smart_mode_binding`; bindings persisted in settings |
| MODE-05 | 13-18 through 13-21; 13-25; 13-26 | Conflict detected + surfaced inline | SATISFIED | `find_conflicting_binding` (mod.rs), structured payload, chip alert, full-width card row; G16+G17 closed |
| MODE-06 | 13-01 through 13-09 | Visual card list UI | SATISFIED | SmartModesSection.tsx renders list of SmartModeCard components |
| TRANS-01 | 13-11 through 13-17 | Translation as first-class preset group | SATISFIED | translate_en/es/fr/zh seeded modes in SEEDED_MODE_ID_TO_I18N_KEY; preset group rendered in SmartModesSection |
| TRANS-02 | Phase 11 + 13 | Translation via embedded LLM | SATISFIED | Architecture established in Phase 11 (llama-cpp-2 runtime); Smart Mode prompts use generic LLM path |
| L10N-01 | 13-21 through 13-24 | All strings localized, 20 locales | SATISFIED | `bun run check:translations` exits 0; 20 locale dirs present; shortcutConflict + shortcutConflictBase keys confirmed in all 20 locales |

No orphaned requirements — all 7 requirement IDs mapped to plans and verified.

---

### Anti-Patterns Found

No blocking anti-patterns detected in the changed files (13-25, 13-26 scope):

- No TODO/FIXME/placeholder comments in `SmartModeCard.tsx`, `SmartModeShortcutChip.tsx`, or `mod.rs` (conflict section)
- No stub implementations — all handlers are substantive
- `bun run lint` exits 0 (clean)
- `cargo fmt --check` exits 0 (clean)
- `cargo test --lib conflict_payload_carries_binding_id_before_name` passes

---

### Human Verification Required

#### 1. [G16] FR locale: exact-duplicate conflict names the mode in French

**Test:** Build or run the app with FR locale. Open Settings > Smart Modes. Attempt to bind a shortcut already used by the "Nettoyage" (Clean Up) seeded mode to another mode.
**Expected:** The conflict chip / card alert reads "Déjà utilisé par : Nettoyage" — NOT "Déjà utilisé par : Clean Up". The French localized name appears in the interpolated position.
**Why human:** `localizeSmartModeName` compares `storedName === seedDefault` at runtime after `t(i18nKey)` is resolved by the live i18next context. The code path is correct, but locale-aware string resolution under a running FR app cannot be confirmed by static analysis.

#### 2. [G17] FR locale: base-overlap message wraps without overflowing card

**Test:** Build or run the app with FR locale. Open Settings > Smart Modes. Attempt to bind a shortcut whose first key (base key) matches an existing seeded mode's shortcut. Observe the card rendering.
**Expected:** The full French base-overlap message ("Le début de ce raccourci (…) est déjà utilisé par Nettoyage — il se déclenchera avant la prochaine touche") appears BELOW the card's top row on its own line, wrapping fully within the card width. The card title ("Nettoyage") and the "Ajouter un raccourci" placeholder chip are NOT obscured or overpainted.
**Why human:** The layout fix (`w-full whitespace-normal break-words` on a `flex flex-col gap-2` card child) eliminates the overflow structurally, but confirming no visual overflow at the exact FR string length under the actual rendered card requires a live build.

---

### Gaps Summary

No automated gaps. All code-inspectable checks for G16 and G17 pass:

- G16 backend: new payload format (`|<id>|<name>`) confirmed in mod.rs; unit test passes (1/1).
- G16 frontend: `localizeSmartModeName` exported from SmartModeCard, imported by chip, applied with `smart_mode_` prefix strip before t() interpolation.
- G16 no-regress: distinct `shortcutConflict` vs `shortcutConflictBase` t() calls preserved for exact-duplicate vs base-overlap.
- G17: chip return is bare `<>{renderChip()}</>` with no inline `<p role="alert">`; card owns `shortcutConflict` state, passes `onConflictChange`, and renders the full-width `break-words` row below the top row.
- Translation completeness: `check:translations` passes for all 19 non-EN locales.

The two human_verification items are the only remaining items — both are live-visual confirmations that the correct code paths produce the expected visible output under FR locale. No structural rework is needed.

---

_Verified: 2026-06-09T11:00:00Z_
_Verifier: Claude (gsd-verifier)_
