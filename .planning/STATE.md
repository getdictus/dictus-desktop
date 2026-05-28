---
gsd_state_version: 1.0
milestone: v1.2
milestone_name: Polish & Local-First UX
status: verifying
stopped_at: Completed 08-10-PLAN.md
last_updated: "2026-05-22T20:33:12.565Z"
last_activity: "2026-05-22 — 08-10 complete: i18n key changes propagated to all 19 sibling locales (tabs.local/.cloud translated and inserted between api and modelsAndLocalProcessing, cloudToggle and pillars blocks deleted, cloudSelectedNotice updated with locale-matching Cloud tab label); bun run check:translations exits 0 (19/19 pass)"
progress:
  total_phases: 3
  completed_phases: 3
  total_plans: 15
  completed_plans: 15
  percent: 100
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-04-15 at v1.2 kickoff)

**Core value:** L'application doit être identifiable et utilisable comme Dictus Desktop — pas comme Handy — et rester vivante sans décrocher du upstream Handy.
**Current focus:** v1.2 Phase 8 — Privacy / Local-First UX (next up, ready to discuss/plan)

## Current Position

Phase: 8 of 8 overall (Phase 3 of 3 in v1.2) — all plans executed, ready for re-UAT
Plan: 08-10 complete (locale propagation across 19 sibling locales). Phase 8 source-side gap closure done.
Status: Phase 8 complete in source — all 6 UAT gaps closed: bugs 1a/2a/2b via 08-08, design pivots 5/6/7 via 08-09 (en source) + 08-10 (19 sibling locales). `bun run check:translations` exits 0. Next: re-UAT via `/gsd:execute-phase 8` UAT checkpoint or direct `/gsd:verify-work 8`.
Last activity: 2026-05-22 — 08-10 complete: i18n key changes propagated to all 19 sibling locales (tabs.local/.cloud translated and inserted between api and modelsAndLocalProcessing, cloudToggle and pillars blocks deleted, cloudSelectedNotice updated with locale-matching Cloud tab label); bun run check:translations exits 0 (19/19 pass)

Progress: [██████████] 100% (v1.2 — Phase 8 source-side gap closure done; all 10 Phase 8 plans + UAT-checkpoint complete; re-UAT pending)

## Performance Metrics

**Velocity:**

- Total plans completed: 15 (v1.0 + v1.1 combined)
- Average duration: unknown
- Total execution time: unknown

**By Phase:**

| Phase      | Plans | Total | Avg/Plan |
| ---------- | ----- | ----- | -------- |
| 1-3 (v1.0) | 8     | -     | -        |
| 4-5 (v1.1) | 7     | -     | -        |

_Updated after each plan completion_
| Phase 06-brand-icon-polish P01 | 3 | 3 tasks | 2 files |
| Phase 06-brand-icon-polish P02 | 3 | 2 tasks | 6 files |
| Phase 06-brand-icon-polish P03 | 8 | 1 tasks | 1 files |
| Phase 06-brand-icon-polish P04 | ~60min | 4 tasks | 12 files |
| Phase 07-macos-clean-shutdown P01 | ~1h coding + multi-day validation | 4 tasks | 7 files |
| Phase 08-privacy-local-first-ux P02 | ~5min | 2 tasks | 3 files |
| Phase 08-privacy-local-first-ux P01 | 4min | 1 tasks | 1 files |
| Phase 08-privacy-local-first-ux P03 | ~25min | 3 tasks | 7 files |
| Phase 08-privacy-local-first-ux P04 | ~4min | 1 tasks | 19 files |
| Phase 08-privacy-local-first-ux P06 | ~6min | 3 tasks | 8 files |
| Phase 08-privacy-local-first-ux P07 | ~5min | 1 tasks | 19 files |
| Phase 08-privacy-local-first-ux P05 | ~30min (UAT) | 1 task (checkpoint) | 0 files (verification-only) |
| Phase 08-privacy-local-first-ux P08 | 2 min | 2 tasks | 2 files |
| Phase 08-privacy-local-first-ux P09 | 3m 29s | 3 tasks | 3 files |
| Phase 08-privacy-local-first-ux P10 | 1m 56s | 1 tasks | 19 files |

## Accumulated Context

### Decisions

- v1.2: Phase 6 must land before Phase 9 — verify-sync.sh extended assertions needed before CI gate goes live
- v1.2: Phases 6, 7, 8 are independent; can run on separate branches simultaneously
- v1.2: Phase 10 hard-depends on Phase 9 (no labeled draft PRs = agent never fires)
- [Phase 05]: UPSTREAM.md §6 post-sync gate missing UPDT-03/UPDT-05 — SYNC-07 in Phase 9 closes this
- [Phase 06-brand-icon-polish]: SYNC-06 completed in Phase 6 Plan 01 ahead of Phase 9 schedule; .github/scripts/verify-sync.sh is now the permanent script location
- [Phase 06-brand-icon-polish]: ICON-02a passes immediately — icon.ico already has all 6 required layer sizes; BRAND-01a/02a/03a expected to fail until Plans 06-02/03 land
- [Phase 06-brand-icon-polish]: No dual-read fallback for Handy Portable Mode marker: CONTEXT.md override, Pierre is sole user
- [Phase 06-brand-icon-polish]: BRAND-01a filter uses underscore grep -v handy_keys: hyphenated handy-keys in comments must use HandyKeys capitalized form
- [Phase 06-brand-icon-polish]: Pre-compute modelsPath/settingsPath as template literals before JSX render to satisfy i18next/no-literal-string ESLint rule when concatenating path suffixes
- [Phase 06-brand-icon-polish]: Reused existing get_app_dir_path Tauri command for DebugPaths — no new backend command created (RESEARCH.md Pitfall 1 honored)
  - [Phase 06-brand-icon-polish P04]: ICON-01 resolved via opaque-navy-tile variant — source PNG has zero transparent pixels, making Linux black-corners artifact physically impossible (approved by Pierre as Option A)
  - [Phase 06-brand-icon-polish P04]: tauri icon CLI does not output 256x256.png/512x512.png; ImageMagick fallback required (convert from 1024 source)
  - [Phase 06-brand-icon-polish P04]: Linux (ICON-01) and Windows (ICON-02) visual verification deferred — macOS approved; automated backstops in place
  - [Phase 06.1 follow-up]: Icon regression fixed 2026-04-23 (commit 34de33e) — Phase 6 rasterization dropped the middle blue bar; re-rendered dictus-brand source and regenerated full platform icon set; legacy Handy `logo.png` removed
  - [Phase 07-macos-clean-shutdown P01]: Path (a) graceful cleanup chosen over path (b) std::process::exit — diagnosis pointed at tauri-plugin-global-shortcut Drop releasing CGEventTap from atexit on main thread; flush_and_exit helper releases CGEventTap while runloop is still alive
  - [Phase 07-macos-clean-shutdown P01]: simulate_updater_restart ships in release builds, UI-gated by settings.debug_mode (not #[cfg(debug_assertions)]) so production installs can validate updater-relaunch path
  - [Phase 07-macos-clean-shutdown P01]: Crash non-reproducible after fix over multi-day validation window; phase closed without a second-reproduction checkpoint
  - [Phase 08-privacy-local-first-ux P02]: docs/PRIVACY.md is single source of truth for network surface — no in-app Privacy page; blob.handy.computer CDN documented as-is (INFR-01 deferred); legacy Referer header flagged but not changed in Phase 8
- [Phase 08-privacy-local-first-ux]: Platform-aware default: macOS ARM64 gets apple_intelligence, all other platforms get custom (local) — cfg gate matches existing pattern at line 580
- [Phase 08-privacy-local-first-ux]: Custom provider id stays 'custom' (stable for persisted settings); only label changed to 'Custom (local)'
  - [Phase 08-privacy-local-first-ux P03]: ProviderSelect.tsx left in place as unused export — deletion deferred to keep diff small; no callers remain
  - [Phase 08-privacy-local-first-ux P03]: PRIV-03 satisfied by existing Onboarding state — zero post_process references in onboarding components; no code change required
  - [Phase 08-privacy-local-first-ux P03]: renderRowExtras prop pattern established for injecting row-level extras inside selected radio card
- [Phase 08-privacy-local-first-ux P04]: simulateUpdaterRestart keys kept in English across all locales (technical debug feature — fallback acceptable per CONTRIBUTING_TRANSLATIONS.md)
- [Phase 08-privacy-local-first-ux]: enable_cloud_providers is UI visibility filter only — does NOT mutate post_process_provider_id; cloud providers hidden by default (false), toggle shows/hides External section in ProviderPicker
- [Phase 08-privacy-local-first-ux]: Library placeholder card is intentional Coming-soon preview of Local-First Models milestone — not scope creep; previews gemma3:4b + GGUF downloader coming in next milestone
- [Phase 08-privacy-local-first-ux]: fr/translation.json carries verbatim French copy of cloudToggle + modelsAndLocalProcessing — source values are canonical French per mockup product intent
- [Phase 08-privacy-local-first-ux]: Brand names (OpenAI, Anthropic, Groq, Cerebras, OpenRouter, Z.AI, Apple Intelligence, Ollama, gemma3:4b, GGUF, Custom (local)) preserved verbatim across all 19 locales in 08-07
- [Phase 08-privacy-local-first-ux P05]: UAT partial-pass (2026-05-22) — 3 bugs + 3 design pivots documented; gap-closure plan required before Phase 8 ships; phase stays at 08, no advancement
- [Phase 08-privacy-local-first-ux P05]: Tabs pattern chosen over cloud toggle (user direction) — enable_cloud_providers setting to be repurposed or removed in gap-closure plan
- [Phase 08-privacy-local-first-ux P05]: Pillars block (Confidentialité / Contrôle / Expérience) confirmed as user-rejected UI pattern — will be fully removed including all 20 locale keys
- [Phase 08-privacy-local-first-ux P08]: Gap-closure for UAT 1a/2a/2b — Apple Intelligence Alert inlined via renderRowExtras, Ollama link underlined at rest, API key field hidden for Custom (local). Option A chosen for 2b (no i18n change).
- [Phase 08-privacy-local-first-ux P08]: ProviderPicker.renderRowExtras now invoked for every row (caller controls null-vs-content) — establishes generic per-row extras pattern that survives 08-09 tabs restructure.
- [Phase 08-privacy-local-first-ux]: P09 — ProviderPicker tabs (Local/Cloud) replace the cloud opt-in toggle; default tab derived from persisted provider id; useEffect keeps tab synced with selection — closes UAT design pivot 5
- [Phase 08-privacy-local-first-ux]: P09 — Library SettingsGroup hoisted to first content block under page header; three-pillar marketing grid removed entirely (JSX + en pillars.\* keys) — closes UAT design pivots 6 & 7
- [Phase 08-privacy-local-first-ux]: P09 — enable_cloud_providers Rust field kept vestigial (no schema migration); UI no longer reads/writes; settings tests still pass; cleanup deferrable to a dedicated migration plan
- [Phase 08-privacy-local-first-ux]: P10 — All 19 sibling locales now mirror EN i18n: tabs.local/.cloud translated and inserted between api and modelsAndLocalProcessing; cloudToggle and pillars blocks deleted; cloudSelectedNotice updated per locale with brand-name Cloud matching each locale's tabs.cloud value. check:translations exits 0 (19/19 pass). Phase 8 source-side gap closure complete; ready for re-UAT.
- [Phase 08-privacy-local-first-ux]: P10 — Locale propagation done via one-shot Node script (JSON.parse/stringify roundtrip) over 19 files; per-locale failure isolation log+continue; all 19 succeeded first try. Insertion-order-preserving key add (reconstruct parent key-by-key) keeps file diffs structurally identical to EN diff from 08-09.

### Pending Todos

3 pending — see `.planning/todos/pending/` for details.
(2 moved to done/ on 2026-04-23: icon regression + macOS quit-unexpectedly)

### Blockers/Concerns

**Phase 8 gap-closure in progress (2026-05-22):** 08-05 UAT partial pass — 3 of 6 gaps now closed by 08-08; 3 design pivots remain for 08-09 + 08-10:

- ~~Bug 1a: Apple Intelligence error banner renders at bottom of page instead of inline with Apple Intelligence card~~ — **closed in 08-08 (bbe82db)** — Alert inlined via ProviderPicker.renderRowExtras
- ~~Bug 2a: Ollama link not visually distinguishable~~ — **closed in 08-08 (bbe82db)** — className changed to `underline underline-offset-2 hover:opacity-80`
- ~~Bug 2b: API key field shown for Custom (local) provider~~ — **closed in 08-08 (bbe82db)** — Option A: ApiKeyField gated on `selectedProvider?.id !== "custom"` (no i18n change)
- Design pivot 5: Replace cloud toggle with tabs pattern (Local / Cloud tabs; user-directed) — 08-09 scope
- Design pivot 6: Remove three-pillar privacy block from post-processing page (user-directed) — 08-09 scope (locale sweep in 08-10)
- Design pivot 7: Move "Bibliothèque de modèles locaux" placeholder to top of page (user-directed) — 08-09 scope
  See `.planning/phases/08-privacy-local-first-ux/08-05-SUMMARY.md § Gaps` and `08-08-SUMMARY.md` for full closure detail.

Carried from v1.1 audit:

- UPSTREAM.md §6 post-sync gate missing UPDT-03/UPDT-05 re-assertion (deferred — captured in `.planning/todos/pending/2026-05-21-upstream-sync-strategy-review.md` for the manual-workflow simplification path)
- Phase 5 VALIDATION.md draft → run `/gsd:validate-phase 5` to close
- `blob.handy.computer` CDN for onnxruntime (INFR-01, deferred)
- Windows builds unsigned at OS level (INFR-03, deferred)

## Session Continuity

Last session: 2026-05-22T20:27:50.463Z
Stopped at: Completed 08-10-PLAN.md
Resume file: None
