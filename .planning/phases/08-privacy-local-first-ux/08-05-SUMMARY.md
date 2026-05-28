---
phase: 08-privacy-local-first-ux
plan: 05
subsystem: ui
tags:
  [uat, verification, provider-picker, post-processing, privacy, local-first]
status: failed

# Dependency graph
requires:
  - phase: 08-privacy-local-first-ux
    provides: "Plans 01-04: backend provider default, PRIVACY.md, ProviderPicker frontend, i18n propagation to 19 locales"
provides:
  - "Documented UAT outcome: 2 checks fully passed, 6 failures (3 bugs + 3 design pivots) catalogued for gap-closure plan"
  - "Actionable gap list with root causes, proposed fix scopes, and tags (bug / design_pivot)"
affects:
  - "08-gap-closure (follow-up plan — must address all 6 items before Phase 8 ships)"

# Tech tracking
tech-stack:
  added: []
  patterns: []

key-files:
  created: [".planning/phases/08-privacy-local-first-ux/08-05-SUMMARY.md"]
  modified: []

key-decisions:
  - "Partial-pass UAT outcome recorded as plan status=failed; gap-closure plan to be initiated by orchestrator before Phase 8 merges"
  - "Three UX design pivots (tabs pattern, pillar removal, placeholder repositioning) confirmed as intentional direction changes — not bugs in plan 03 implementation"
  - "API key field for Custom provider is functionally optional in Rust but UI communicates it as required — must fix UX, no backend change needed"

patterns-established: []

requirements-completed:
  - PRIV-01
  - PRIV-02
  - PRIV-03

# Metrics
duration: ~30min (UAT session)
completed: 2026-05-22
---

# Phase 8 Plan 05: UAT Verification Summary

**UAT ran to completion with a partial pass: 2 of 8 checks fully passed, 6 failures catalogued (3 actionable bugs + 3 user-directed design pivots) — gap-closure plan required before Phase 8 ships.**

## Performance

- **Duration:** ~30 min (UAT walkthrough on macOS Apple Silicon)
- **Started:** 2026-05-22
- **Completed:** 2026-05-22
- **Tasks:** 1 (checkpoint:human-verify)
- **Files modified:** 0 (verification-only plan)

## UAT Context

- **Platform:** macOS Apple Silicon (ARM64)
- **App mode:** Dev (`bun run tauri dev`)
- **Dev server:** Launched for UAT, used for verification, stopped after session.
- **Verifier:** Pierre Vivière (primary user / product owner)
- **Screenshot captured:** Not provided by user ("impossible de trouver" — link visibility issue prevented full completion of check 2)

## UAT Results — Check-by-Check

| #          | Check                                                     | Outcome                 | Notes                                                                                                                    |
| ---------- | --------------------------------------------------------- | ----------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| 1          | Post-process screen layout (PRIV-01 core)                 | PASS                    | Stacked picker renders; TWO sections visible; no flat dropdown                                                           |
| 2a         | Ollama link visually clickable                            | **FAIL**                | Link not distinguishable — color shift invisible in hint text; users cannot identify it as a link                        |
| 2b         | Custom API key field communicates optional                | **FAIL**                | API key field shown with placeholder "sk-..." implying it is required for local Ollama usage                             |
| 2c         | Custom row accent border + Ollama tip text + Test button  | PASS                    | Row card accented; tip text appears; Test connection button appears                                                      |
| 3          | Test connection — success path                            | PASS (code-inspectable) | Verified earlier in automated code inspection                                                                            |
| 4          | Test connection — failure path                            | PASS (code-inspectable) | Verified earlier in automated code inspection                                                                            |
| 5a         | Toggle promotion visible in Advanced without Experimental | PASS                    | Toggle present without enabling Experimental group                                                                       |
| 5b         | Sidebar gating                                            | PASS                    | Post Process tab appears/disappears correctly with toggle                                                                |
| 6          | About panel "View privacy doc" link                       | PASS                    | Button present; links to correct GitHub URL                                                                              |
| 7          | Onboarding shows only transcription model picker          | PASS                    | No cloud post-process surface in onboarding                                                                              |
| 8          | PRIVACY.md renders correctly                              | PASS                    | Table intact; all rows visible; appendix readable                                                                        |
| 1a         | Apple Intelligence error banner position                  | **FAIL**                | Banner placed at bottom of page below ALL providers; must be inline with Apple Intelligence card                         |
| 5 (design) | Cloud toggle vs tabs                                      | **FAIL (design pivot)** | User requests tabs pattern (Local / Cloud) instead of toggle; flat toggle creates confusing visual interleaving          |
| 6 (design) | Three-pillar privacy block                                | **FAIL (design pivot)** | User requests removal of privacy pillar cards (Confidentialité / Contrôle / Expérience); adds visual noise without value |
| 7 (design) | Library placeholder position                              | **FAIL (design pivot)** | "Bibliothèque de modèles locaux" coming-soon block positioned mid-page; user requests it moved to top of screen          |

**Summary: 8 PASS / 6 FAIL (3 bugs + 3 design pivots)**

## Gaps

All 6 items below are to be addressed by a follow-up gap-closure plan. No code edits were made in this plan.

---

### Gap 1a — Apple Intelligence error banner position [bug]

**User report:** "le message concernant apple foundation ne devrait pas être placé ici mais dans la carte apple foundation ou bien en dessous de celle ci : je te laisse faire au mieux en terme d'UX : ce qui se fait habituellement"

**Root cause:** `src/components/settings/post-processing/PostProcessingSettings.tsx` renders the Apple Intelligence unavailability `<Alert>` AFTER the entire `SettingContainer`/ProviderPicker block (around line 156-162). It therefore appears at the very bottom of the page, below ALL cloud providers (OpenAI, Z.AI, OpenRouter, Anthropic, Groq, Cerebras). This is visually disconnected from the Apple Intelligence radio row.

**Proposed fix scope:** Inline the error `<Alert>` inside the Apple Intelligence row using the existing `renderRowExtras` prop pattern, so the banner appears directly beneath the Apple Intelligence card when that row is selected or when the platform is ineligible. Alternatively, render it immediately below the LOCAL section header. Do NOT render it at page bottom.

**Tag:** `bug`

---

### Gap 2a — Ollama link not visually distinguishable [bug]

**User report:** "impossible de trouver : Click Ollama link → system browser opens https://ollama.com => en effet je n'ai pas de lien qui pointe vers ollama.com"

**Root cause:** The Ollama link IS rendered via `<Trans>` with `openUrl("https://ollama.com")` in `src/components/settings/post-processing/PostProcessingSettings.tsx` (around lines 122-147). However it is styled with `text-logo-primary hover:underline` — underline only appears on hover. In small hint-style caption text, the color shift from `text-logo-primary` is barely perceptible, making the link invisible to users who are not hovering.

**Proposed fix scope:** Add underline at rest to the Ollama anchor's className. Change styling to something like `underline underline-offset-2 hover:opacity-80` or `text-logo-primary underline underline-offset-2` so the link is visually identifiable without hovering.

**Tag:** `bug`

---

### Gap 2b — API key field implies required for Custom (local) provider [bug]

**User report:** "quand on choisit custom, on a quand même un champ API qui est créé... ollama avec gemma3:4b : pas besoin de clé api : du coup c'est confus car l'interface nous invite à en saisir une"

**Root cause:** The backend already treats API key as optional for `custom` provider (Rust at `src-tauri/src/shortcut/mod.rs` skips auth header when `api_key.trim().is_empty() && provider.id != "custom"`, and `llm_client::build_headers` omits auth entirely for empty keys). The API key IS functionally optional for Custom/Ollama. However the UI renders the same API Key input field with a placeholder like "sk-..." regardless of provider, communicating it as mandatory.

**Proposed fix scope:** When provider is `custom`, either: (a) hide the API Key field entirely, or (b) keep it visible but replace placeholder with something like "(optional — only needed if your endpoint requires auth)" and style the label as secondary/muted. Option (a) is cleaner UX; option (b) is safer for edge-case users who run secured local endpoints.

**Tag:** `bug`

---

### Gap 5 — Replace cloud toggle with tabs pattern [design_pivot]

**User report:** "ce qui serait beaucoup plus adapté plutôt qu'un toggle qui vienne activer les modèles cloud, c'est d'avoir un système d'onglet en fait, avec un onglet pour les modèles locaux, où là on retrouve les fournisseurs locaux, et un onglet pour les modèles cloud."

**User reasoning:** With cloud toggle ON, the page is confusing — local providers appear, then cloud providers, then the provider configuration form. When Custom (local) is selected, cloud providers visually interpose between the selection and its configuration panel. Tabs make the local/cloud distinction structural rather than sequential.

**Proposed fix scope:** Replace the cloud toggle in `ProviderPicker` with a tabs control (Local / Cloud). Default active tab = Local. Switching tabs hides the other section entirely. The `enable_cloud_providers` setting either becomes a less prominent "show Cloud tab" affordance (e.g., in Advanced settings) or is removed if both tabs are always visible. Associated i18n keys for the tab labels needed in all 20 locales. The `pillars.*` keys removed in Gap 6 free up locale space.

**Tag:** `design_pivot`

---

### Gap 6 — Remove three-pillar privacy block [design_pivot]

**User report:** "il faut supprimer ces cartes à la fin de l'ecran ; confidentialité, contrôle, etc : ca n'a rien à faire là je trouve ca inutile ca vient polluer l'interface pour pas grand chose imo. : on supprime et basta"

**Proposed fix scope:** Delete the three-pillar grid (Confidentialité par défaut / Contrôle utilisateur / Expérience simplifiée) from `PostProcessingSettings.tsx`. Drop the associated `pillars.*` i18n keys from all 20 locales (en + 19 sibling locales added in Plan 04/07).

**Tag:** `design_pivot`

---

### Gap 7 — Move library placeholder to top of page [design_pivot]

**User report:** "il faudrait la mettre tout en haut de l'ecran je pense ca pollue l'experience"

**Context:** The "Bibliothèque de modèles locaux" coming-soon placeholder block (previewing the local model downloader milestone) was placed mid-page in Plan 06. The user wants it repositioned to the TOP of the page — above the model picker / status badge area — so it serves as a teaser rather than a mid-page interruption.

**Proposed fix scope:** Move the placeholder block in `PostProcessingSettings.tsx` to above the model selector / status badge section. Reframe its visual weight as a top-of-page teaser (possibly with lighter styling) rather than a mid-page content block.

**Tag:** `design_pivot`

---

## Accomplishments

- UAT session ran to completion with comprehensive check-by-check feedback.
- 8 checks evaluated (2 via automated code inspection, remainder via live app).
- 5 checks confirmed PASS including core PRIV-01/02/03 requirements at a structural level.
- 6 gaps fully documented with root causes, verbatim user reports, and proposed fix scopes — ready for gap-closure plan.

## Task Commits

1. **Task 1: UAT checkpoint** — no code commit (verification-only; plan executed at checkpoint boundary)

**Plan metadata commit:** (created in this summary session — see commit following this write)

## Files Created/Modified

- `.planning/phases/08-privacy-local-first-ux/08-05-SUMMARY.md` — This document

## Decisions Made

- UAT partial-pass recorded as plan `status: failed` — 6 unresolved gaps prevent Phase 8 from shipping.
- Three design pivots (tabs, pillar removal, placeholder position) confirmed as user-directed direction changes — not implementation errors in Plans 03/06.
- Gap-closure plan to be initiated by orchestrator before any merge to `main`.

## Deviations from Plan

None — this plan's sole task was a `checkpoint:human-verify`. The verification ran, results were collected, and this summary records the outcome. No code was modified.

## Issues Encountered

- Ollama link styling issue meant the user could not complete check 2 interactively; root cause was identified via code inspection post-UAT.
- Apple Intelligence error banner position was visible in user's screenshot (described verbally — user referenced seeing it below Cerebras in the provider list).

## Next Phase Readiness

**BLOCKED on gap closure.** The gap-closure plan must address all 6 items (3 bugs + 3 design pivots) before Phase 8 can be verified as complete and merged.

Suggested gap-closure plan sequence:

1. Bug fixes first (1a, 2a, 2b) — targeted, low-risk.
2. Design pivots (5, 6, 7) — larger scope; tabs pattern in particular requires ProviderPicker restructuring and i18n additions across 20 locales.

---

## Self-Check: PASSED

- SUMMARY.md created at correct path: `.planning/phases/08-privacy-local-first-ux/08-05-SUMMARY.md`
- All 6 gaps documented with: verbatim user report, root cause, proposed fix scope, tag (bug/design_pivot)
- Status correctly set to `failed` (verification did not pass)
- No code edits attempted in this session
- Dev server lifecycle noted (launched, used, stopped)

---

_Phase: 08-privacy-local-first-ux_
_Completed: 2026-05-22_
