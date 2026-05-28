---
phase: 08-privacy-local-first-ux
plan: "10"
subsystem: i18n
tags:
  - i18n
  - locale-propagation
  - key-parity
  - gap-closure
  - local-first
  - tabs

requires:
  - phase: 08-privacy-local-first-ux
    provides:
      - "English i18n source has tabs.local + tabs.cloud (added by 08-09)"
      - "English i18n source has cloudToggle and modelsAndLocalProcessing.pillars removed (08-09)"
      - "English cloudSelectedNotice copy rewritten to reference the Cloud tab (08-09)"
provides:
  - "Key parity restored across all 19 sibling locales (ar, bg, cs, de, es, fr, he, it, ja, ko, pl, pt, ru, sv, tr, uk, vi, zh, zh-TW)"
  - "tabs.local / tabs.cloud translated per locale, inserted between `api` and `modelsAndLocalProcessing` to match EN insertion order"
  - "cloudToggle block deleted from all 19 sibling locales"
  - "modelsAndLocalProcessing.pillars block deleted from all 19 sibling locales"
  - "cloudSelectedNotice value updated per locale to reference the Cloud tab (brand-name `Cloud` matches each locale's `tabs.cloud` translation for internal consistency)"
  - "`bun run check:translations` exits 0 (19/19 languages have complete translations!)"
affects:
  - "08-VERIFICATION.md (gaps 5/6/7 now fully closed in source AND propagated to siblings — ready for re-UAT)"
  - "Phase 8 ship readiness: all 6 UAT gaps (1a/2a/2b from 08-08; 5/6/7 from 08-09+10) now closed in source"

tech-stack:
  added: []
  patterns:
    - "Script-driven locale propagation: a one-shot Node.js script reads each sibling JSON, mutates in memory (insertion-order-preserving reconstruct of the parent object for new keys, deep-delete for removed blocks, replace for updated values), re-validates with JSON.parse, and writes back. Per-locale failure isolation: log + continue, summarize at end."
    - "Insertion-order mirroring: when adding a key to a nested object, reconstruct the parent object key-by-key, inserting the new key after the EN anchor (`api`). JS preserves string-key insertion order so the file diff stays minimal and review-friendly."
    - "Brand-name vs UI-label discrimination for translation: the word `Cloud` used as a tab label is a UI label (translated per locale per CONTRIBUTING_TRANSLATIONS.md); inside `cloudSelectedNotice` the quoted tab name MUST match each locale's `tabs.cloud` translation for internal consistency."

key-files:
  created: []
  modified:
    - "src/i18n/locales/ar/translation.json"
    - "src/i18n/locales/bg/translation.json"
    - "src/i18n/locales/cs/translation.json"
    - "src/i18n/locales/de/translation.json"
    - "src/i18n/locales/es/translation.json"
    - "src/i18n/locales/fr/translation.json"
    - "src/i18n/locales/he/translation.json"
    - "src/i18n/locales/it/translation.json"
    - "src/i18n/locales/ja/translation.json"
    - "src/i18n/locales/ko/translation.json"
    - "src/i18n/locales/pl/translation.json"
    - "src/i18n/locales/pt/translation.json"
    - "src/i18n/locales/ru/translation.json"
    - "src/i18n/locales/sv/translation.json"
    - "src/i18n/locales/tr/translation.json"
    - "src/i18n/locales/uk/translation.json"
    - "src/i18n/locales/vi/translation.json"
    - "src/i18n/locales/zh/translation.json"
    - "src/i18n/locales/zh-TW/translation.json"

key-decisions:
  - "Script-driven over manual Edit-tool: 19 files × 4 mutations = 76 surgical edits is too brittle for an Edit-tool sweep; a Node script with JSON.parse/stringify roundtrips guarantees JSON validity and atomic per-file writes. Re-validation via JSON.parse after stringify catches any object-shape regressions."
  - "Insertion position for `tabs` mirrors EN (between `api` and `modelsAndLocalProcessing`): keeps file diff minimal and review-friendly — reviewers see the same insertion shape in every locale, identical to the EN diff from 08-09."
  - "`tabs.local` / `tabs.cloud` translated per locale per the executor-authoritative table in 08-10-PLAN.md: word choice respects each locale's conventional UX vocabulary (e.g., `Cục bộ` not `Local` for Vietnamese; `本地`/`本地` for zh/zh-TW; `Lokalne`/`Chmura` for Polish; etc.)."
  - '`cloudSelectedNotice` value mirrors the EN/FR source clause structure (you have cloud selected → switch to tab OR choose local), with locale-appropriate quotation conventions: « » for fr/es/it/pt; „" for de/cs/pl/bg; "" for en/uk/sv; "" for ar/he/ja/ko/zh/vi/tr; 「」 for ja/zh-TW. Brand-name `Cloud` (as quoted tab name) MUST match each locale''s `tabs.cloud` translation — script-enforced via post-write verification.'
  - "Per-locale failure isolation: script logs + continues per file rather than failing the batch. All 19 succeeded on first run; failure isolation was a safety net, not exercised."
  - "Trailing-newline preserved per-file: script reads the raw file, detects trailing `\\n`, and re-applies it after stringify so the diff stays focused on key changes (not whitespace drift)."
  - "Workspace hygiene: propagation script written to `/tmp/propagate-08-10.mjs` (not checked into repo) — pure execution tooling, no need to retain. Deleted on session end per rules."

patterns-established:
  - "Locale-propagation-via-script-not-Edit-tool: for any cross-locale change touching ≥10 files or ≥2 key paths, write a Node script that does JSON-roundtrip mutations. Avoid the Edit tool for bulk locale work — too many surgical edits, too many silent JSON-syntax risks."
  - "Insertion-order-preserving key add: to add a nested key in a specific position, reconstruct the parent object key-by-key using `for (const k of Object.keys(parent))` and insert the new key after the anchor. JS preserves string-key order so this is reliable."
  - "Brand-vs-label discrimination for translation: words that appear as quoted UI labels inside other strings MUST match the source label's translation. Enforced by post-write per-locale verification (`notice.includes(tabs.cloud)`)."

requirements-completed: [PRIV-01, PRIV-02, PRIV-03]

duration: 1 min 56 s
completed: 2026-05-22
---

# Phase 8 Plan 10: Sibling-Locale i18n Propagation Summary

**Propagated the English i18n source changes from Plan 08-09 to all 19 sibling locales (ar, bg, cs, de, es, fr, he, it, ja, ko, pl, pt, ru, sv, tr, uk, vi, zh, zh-TW): added locale-translated `tabs.local`/`tabs.cloud` between `api` and `modelsAndLocalProcessing`, removed `cloudToggle` and `modelsAndLocalProcessing.pillars` blocks, updated `cloudSelectedNotice` copy to reference the Cloud tab with each locale's exact tab-label translation — restoring key-parity (`bun run check:translations` exits 0, 19/19 languages pass).**

## Performance

- **Duration:** 1 min 56 s
- **Started:** 2026-05-22T20:19:55Z
- **Completed:** 2026-05-22T20:21:51Z
- **Tasks:** 1 (atomic batch via script)
- **Files modified:** 19 (+76 insertions / −342 deletions; net −266 lines because the deleted `pillars` block had 6 strings per locale × 19 locales = 114 strings worth of content removed, while the added `tabs` block + updated notice add far fewer chars)

## Accomplishments

- **Key parity restored across all 19 sibling locales:** `bun run check:translations` now exits 0 with "All 19 languages have complete translations!" — the parity gap deliberately opened by 08-09 is closed.
- **tabs.local + tabs.cloud added in correct insertion position:** Each locale now has `settings.postProcessing.tabs.{local, cloud}` inserted between `api` and `modelsAndLocalProcessing`, matching the EN insertion order from 08-09. Translation values per locale follow the executor-authoritative table in 08-10-PLAN.md (e.g., FR `Local`/`Cloud`, DE `Lokal`/`Cloud`, JA `ローカル`/`クラウド`, AR `محلي`/`سحابي`, ZH `本地`/`云端`).
- **cloudToggle block deleted from all 19 sibling locales:** No locale file contains the `cloudToggle` key anymore (`grep -l cloudToggle src/i18n/locales/*/translation.json` returns empty). The label + description block that gated the cloud opt-in toggle (deprecated in 08-09 in favor of tabs) is gone.
- **modelsAndLocalProcessing.pillars block deleted from all 19 sibling locales:** No locale file contains the `pillars` key anymore (`grep -l pillars src/i18n/locales/*/translation.json` returns empty). The three-pillar marketing grid (privacy/control/simplicity) i18n block — already removed from JSX in 08-09 — is now removed from translation source too.
- **cloudSelectedNotice updated per locale with internal-consistency guarantee:** The notice now reads (per locale) "your selected model is a cloud provider; click the `[Cloud-label]` tab below or choose a local model." The brand-name `Cloud` (as quoted tab name) matches each locale's `tabs.cloud` translation — verified by post-write script (`notice.includes(tabs.cloud)`).
- **Build and prettier pass:** `bun run build` exits 0; `bunx prettier --check src/i18n/locales/` exits 0.

## Task Commits

Task 1 (atomic batch — single commit covers all 19 locale files):

1. **Task 1: Propagate i18n key changes to all 19 sibling locales** — `54b9a85` (chore)

## Files Created/Modified

All 19 sibling locale JSON files received the same 4-edit batch (locale-translated values), executed atomically via `/tmp/propagate-08-10.mjs`:

| Locale | tabs.local | tabs.cloud |
| ------ | ---------- | ---------- |
| ar     | محلي       | سحابي      |
| bg     | Локални    | Облачни    |
| cs     | Lokální    | Cloud      |
| de     | Lokal      | Cloud      |
| es     | Local      | Nube       |
| fr     | Local      | Cloud      |
| he     | מקומי      | ענן        |
| it     | Locale     | Cloud      |
| ja     | ローカル   | クラウド   |
| ko     | 로컬       | 클라우드   |
| pl     | Lokalne    | Chmura     |
| pt     | Local      | Nuvem      |
| ru     | Локальные  | Облако     |
| sv     | Lokalt     | Moln       |
| tr     | Yerel      | Bulut      |
| uk     | Локальні   | Хмара      |
| vi     | Cục bộ     | Đám mây    |
| zh     | 本地       | 云端       |
| zh-TW  | 本地       | 雲端       |

`cloudSelectedNotice` (translation approach):

- Two-clause structure preserved (cloud-selected → switch to tab OR choose local).
- Quotation style follows each locale's convention (« » for fr/es/it/pt; „ for de/cs/pl/bg; "" for ar/he/ja/ko; "" for tr/vi/zh/uk; 「」 for ja/zh-TW; ”” for sv).
- Brand-name `Cloud` (when quoted as tab name) matches each locale's `tabs.cloud` translation exactly — script-verified.
- Verbatim values copied from the executor-authoritative table in 08-10-PLAN.md (interfaces block).

## Decisions Made

- **Script-driven propagation over Edit-tool sweep:** 19 files × 4 mutations = 76 surgical edits is brittle; a Node script with JSON.parse/stringify roundtrips guarantees JSON validity (re-validated post-write) and atomic per-file writes. Per-locale failure isolation (log + continue) was prudent — though all 19 succeeded first try.
- **Insertion position for `tabs` mirrors EN exactly:** Between `api` and `modelsAndLocalProcessing`. Reconstructing the parent object key-by-key with the new key inserted after `api` preserves the EN diff's structural shape and keeps reviewer cognitive load minimal.
- **`tabs.local` / `tabs.cloud` translated per locale, not transliterated:** Vietnamese uses `Cục bộ` (not English `Local`); Polish uses `Lokalne`/`Chmura`; Chinese uses `本地`/`云端` (or `雲端` for zh-TW Traditional). UI-label words are translated, not brand-locked, per CONTRIBUTING_TRANSLATIONS.md.
- **`cloudSelectedNotice` brand-name `Cloud` MUST match each locale's `tabs.cloud`:** When the notice tells the user to click the "Cloud" tab, the quoted tab name must be the actual translated label they'll see in the UI. Script-verified via `notice.includes(tabs.cloud)` after write.
- **Quotation style per-locale convention:** Different scripts use different quote pairs (« » for Romance languages; „ for Germanic/Slavic; "" for English/CJK; 「」 for Japanese/Traditional Chinese). The executor-authoritative table in the plan dictates the exact quote pair per locale.
- **Trailing-newline preserved per-file:** Script reads raw file, detects trailing `\n`, re-applies after `JSON.stringify`. Keeps the diff focused on real content changes (zero whitespace drift).
- **No backend changes:** Pure i18n locale propagation. Settings store, Tauri commands, Rust types all untouched. Vestigial `enable_cloud_providers` Rust field (documented in 08-09-SUMMARY) remains in place — out of scope for this plan.

## Deviations from Plan

### Auto-fixed Issues

None during execution. The plan was followed exactly as written — script approach as recommended in the plan's `<action>` block.

### Out-of-Scope Discoveries (Logged, Not Fixed)

The plan's verification block calls for repo-wide `bun run lint` to exit 0. Repo-wide lint still fails on `src/components/icons/DictusLogo.tsx` (i18next/no-literal-string on the hardcoded "Dictus" SVG `<text>` content) — pre-existing since 08-08, documented in 08-08-SUMMARY.md and 08-09-SUMMARY.md as a known pre-existing failure unrelated to the post-processing UI work. This plan does not touch any `.tsx` files, so it cannot introduce or fix this lint error.

- **Repo-wide `bun run lint`:** still red on `DictusLogo.tsx` (pre-existing since 08-08). Scoped `bunx prettier --check src/i18n/locales/` exits 0; locale JSON files don't go through ESLint.

These pre-existing failures are tracked separately in deferred work; the rules state out-of-scope issues are logged, not fixed. Three other modified `.tsx` files appear in `git status` (AccessibilityOnboarding.tsx, UpdateChecker.tsx, RecordingOverlay.tsx) but were already in the working tree before this plan started — not modified by this plan.

---

**Total deviations:** 0 auto-fixed inside scope. 1 pre-existing repo-wide lint failure noted as out-of-scope (documented above).
**Impact on plan:** Zero scope creep. Plan landed exactly as written; check:translations went from red (broken by 08-09) to green.

## Issues Encountered

- **None during execution.** Script ran clean across all 19 locales on first attempt. Post-write per-locale verification (JSON validity, tabs presence, cloudToggle/pillars removal, notice contains the cloud-tab label) passed 19/19.

## Authentication Gates

None — pure file mutation, no external services.

## User Setup Required

None — no external service configuration.

## Brand-Name Policy Note

`Cloud` (as the tab label) is treated as a **UI label**, not a **brand name**, per CONTRIBUTING_TRANSLATIONS.md. UI labels are translated per locale (e.g., `Nube` for Spanish, `Облако` for Russian, `云端` for Simplified Chinese). The translation table in 08-10-PLAN.md is the executor-authoritative source — these values are what Pierre would see in each locale's UI.

Inside `cloudSelectedNotice`, the quoted tab name (e.g., `"Nube"` in Spanish, `«Облако»` in Russian) **MUST** match the corresponding `tabs.cloud` value for internal consistency — if the user clicks the tab the notice references, they should see the same word on the tab. The propagation script enforces this via post-write verification (`notice.includes(tabs.cloud)`).

## Phase 8 Ship Readiness

With Plans 08-08, 08-09, and 08-10 all landed:

- **All 6 UAT gaps from 08-05 are closed:**
  - Gap 1a (Apple Intelligence Alert position) — closed by 08-08 (bbe82db)
  - Gap 2a (Ollama link visibility) — closed by 08-08 (bbe82db)
  - Gap 2b (API key field hidden for Custom local) — closed by 08-08 (bbe82db)
  - Gap 5 (cloud toggle → Local/Cloud tabs) — closed by 08-09 (dd45f4f) + 08-10 (54b9a85)
  - Gap 6 (three-pillar marketing grid removed) — closed by 08-09 (3433f9e) + 08-10 (54b9a85)
  - Gap 7 (Library hoisted to top) — closed by 08-09 (3433f9e)
- **`bun run check:translations`** exits 0 (19/19 locales pass)
- **`bun run build`** exits 0
- **Ready for re-UAT:** `/gsd:execute-phase 8` UAT checkpoint, or direct `/gsd:verify-work 8`. The French canonical-voice UAT script from 08-05 should be re-run; spot-check at least 2 additional locales (e.g., de + ja or de + ar) to confirm tab labels render in-locale, no stale `cloudToggle` or `pillars` copy bleed-through.
- **No backend impact:** Settings store, Tauri commands, providers list all untouched. Vestigial `enable_cloud_providers` field carried forward from 08-09; cleanup deferrable to a dedicated migration plan if desired.

---

## Self-Check: PASSED

**Files verified on disk (19 modified locale JSON files):**

- FOUND: src/i18n/locales/ar/translation.json
- FOUND: src/i18n/locales/bg/translation.json
- FOUND: src/i18n/locales/cs/translation.json
- FOUND: src/i18n/locales/de/translation.json
- FOUND: src/i18n/locales/es/translation.json
- FOUND: src/i18n/locales/fr/translation.json
- FOUND: src/i18n/locales/he/translation.json
- FOUND: src/i18n/locales/it/translation.json
- FOUND: src/i18n/locales/ja/translation.json
- FOUND: src/i18n/locales/ko/translation.json
- FOUND: src/i18n/locales/pl/translation.json
- FOUND: src/i18n/locales/pt/translation.json
- FOUND: src/i18n/locales/ru/translation.json
- FOUND: src/i18n/locales/sv/translation.json
- FOUND: src/i18n/locales/tr/translation.json
- FOUND: src/i18n/locales/uk/translation.json
- FOUND: src/i18n/locales/vi/translation.json
- FOUND: src/i18n/locales/zh/translation.json
- FOUND: src/i18n/locales/zh-TW/translation.json

**Commits verified in git log:**

- FOUND: 54b9a85 (Task 1 — `chore(08-10): propagate i18n key changes to 19 sibling locales`)

**Acceptance criteria spot-checks (per the plan's `<verify>` block):**

- JSON validity: 19/19 parse OK (zero "INVALID:" lines)
- `tabs.local` present: 19/19 (zero "MISSING_TABS_LOCAL")
- `tabs.cloud` present: 19/19 (zero "MISSING_TABS_CLOUD")
- `cloudToggle` removed: 19/19 (`grep -l cloudToggle src/i18n/locales/*/translation.json` returns empty)
- `pillars` removed: 19/19 (`grep -l pillars src/i18n/locales/*/translation.json` returns empty)
- `cloudSelectedNotice` contains each locale's `tabs.cloud`: 19/19 (zero "NOTICE_NOT_UPDATED")
- `bun run check:translations` → exits 0 with "All 19 languages have complete translations!" (431 reference keys, all present)
- `bun run build` → exits 0 (1930 modules, built in 2.21s)
- `bunx prettier --check src/i18n/locales/` → exits 0

**Pre-existing failure noted (out of scope per scope-boundary rule):**

- `bun run lint` repo-wide red on `src/components/icons/DictusLogo.tsx` (pre-existing since 08-08; this plan modifies only locale JSON files, which don't go through ESLint).

---

_Phase: 08-privacy-local-first-ux_
_Completed: 2026-05-22_
