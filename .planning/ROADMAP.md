# Roadmap: Dictus Desktop

## Milestones

- ✅ **v1.0 Handy→Dictus Rebrand** — Phases 1-3 (shipped 2009-04-10) — [archive](milestones/v1.0-ROADMAP.md)
- ✅ **v1.1 Auto-Update & Upstream Sync** — Phases 4-5 (shipped 2009-04-14) — [archive](milestones/v1.1-ROADMAP.md)
- 🚧 **v1.2 Polish & Local-First UX** — Phases 6-8 (in progress)

## Phases

<details>
<summary>✅ v1.0 Handy→Dictus Rebrand (Phases 1-3) — SHIPPED 2009-04-10</summary>

- [x] Phase 1: Bundle Identity (1/1 plans) — completed 2009-04-05
- [x] Phase 2: Visual Rebrand (5/5 plans) — completed 2009-04-09
- [x] Phase 3: Documentation and Cleanup (2/2 plans) — completed 2009-04-09

</details>

<details>
<summary>✅ v1.1 Auto-Update & Upstream Sync (Phases 4-5) — SHIPPED 2009-04-14</summary>

- [x] Phase 4: Updater Infrastructure (4/4 plans) — completed 2009-04-13
- [x] Phase 5: Upstream Sync (3/3 plans) — completed 2009-04-14

</details>

### 🚧 v1.2 Polish & Local-First UX (In Progress)

**Milestone Goal:** Polish Dictus identity across all user-visible surfaces, fix platform icon artifacts, resolve the macOS clean-shutdown crash, and make the local-first promise legible in the post-processing UX — providers reordered with local options first, network surface documented, onboarding copy emphasizing local transcription as the primary path.

> **Scope change (2026-05-21):** The "Automation" half of the original milestone (Phases 9 & 10 — automated sync infrastructure + Claude Code agent layer) was removed as over-engineered relative to actual upstream cadence. The simpler manual-sync workflow plan is captured in `.planning/todos/pending/2026-05-21-upstream-sync-strategy-review.md`. The Translation Mode feature originally floated as a v1.2 candidate is deferred to milestone **v1.3 "Smart Mode & Translation"**.

- [x] **Phase 6: Brand & Icon Polish** - Fix all remaining Handy brand leaks and platform icon artifacts; extend verify-sync.sh to guard them (absorbs SYNC-06 from Phase 9) — completed 2009-04-16
- [x] **Phase 7: macOS Clean Shutdown** - Diagnose and fix the "Dictus quit unexpectedly" crash dialog on macOS Sequoia — completed 2009-04-23
- [x] **Phase 8: Privacy / Local-First UX** - Reorder post-process providers (local first) and document the app's network surface (completed 2026-05-22)

## Phase Details

### Phase 6: Brand & Icon Polish

**Goal**: Dictus identity is complete and verifiable — no Handy strings appear in recording filenames, portable mode detection, or the debug path display; platform icons render correctly on Linux and Windows; verify-sync.sh guards all three surfaces and lives at its permanent home `.github/scripts/`.
**Depends on**: Nothing (independent of all v1.2 phases)
**Requirements**: BRAND-01, BRAND-02, BRAND-03, BRAND-04, ICON-01, ICON-02, ICON-03, ICON-04 (+ SYNC-06 pulled forward from Phase 9 per CONTEXT.md roadmap ripple)
**Success Criteria** (what must be TRUE):

1. New recordings created by the app are named `dictus-{timestamp}.wav` — no `handy-` prefix appears in the filesystem or history panel
2. DebugPaths settings panel shows the real data directory path (e.g., `/Users/name/Library/Application Support/com.dictus.desktop`) instead of the hardcoded `%APPDATA%/handy` string
3. Linux app launcher and taskbar show the Dictus icon with no black corner artifact; Windows executable embeds a multi-resolution `.ico` with Dictus logo at every required layer
4. Running `verify-sync.sh` on a branch that reintroduces `handy-*.wav`, `"Handy Portable Mode"`, or the hardcoded debug path causes the script to exit non-zero
   **Plans**: 4 plans

- [x] 06-01-PLAN.md — Relocate verify-sync.sh to .github/scripts/ and extend with BRAND-01a/02a/03a/ICON-02a assertions (SYNC-06, BRAND-04, ICON-02 verification surface)
- [x] 06-02-PLAN.md — Replace handy- filename prefix and "Handy Portable Mode" marker in Rust backend (BRAND-01, BRAND-02)
- [x] 06-03-PLAN.md — Rewrite DebugPaths.tsx to render backend-provided portable-aware path (BRAND-03)
- [x] 06-04-PLAN.md — Regenerate platform icons from square transparent source and extend bundle.icon config (ICON-01, ICON-02, ICON-03, ICON-04) [has checkpoints]

### Phase 7: macOS Clean Shutdown

**Goal**: Quitting Dictus on macOS (via tray menu or post-update relaunch) no longer triggers the OS "quit unexpectedly" crash dialog — root cause is diagnosed before a fix is committed.
**Depends on**: Nothing (fully independent)
**Requirements**: SHUT-01, SHUT-02, SHUT-03
**Success Criteria** (what must be TRUE):

1. A Console.app crash report is read and the crashing thread identified; the diagnosis (suspect plugin and fix strategy chosen) is committed to the phase plan before any code changes
2. Clicking "Quit Dictus" in the system tray on macOS Sequoia 15.x dismisses the app cleanly — the "Dictus quit unexpectedly — Reopen / Report / Ignore" OS dialog does not appear
3. Post-auto-update relaunch on macOS completes without triggering the crash dialog
   **Plans**: 1 plan (iteration 1; CONTEXT.md caps at 2 iterations max — Plan 2 only opens if Task 5 reports the dialog still appears)

- [ ] 07-01-PLAN.md — Diagnose `.ips` crash report, apply graceful-cleanup + log-flush at both `lib.rs` exit sites, add debug-only `simulate_updater_restart` trigger, append `UPSTREAM.md` conflict-rules row (SHUT-01, SHUT-02, SHUT-03) [has checkpoint]

### Phase 8: Privacy / Local-First UX

**Goal**: The settings UI and onboarding flow communicate clearly that Dictus is a local-first app — local post-process providers appear before external ones, the network surface is documented, and onboarding copy presents cloud as opt-in.
**Depends on**: Nothing (fully independent)
**Requirements**: PRIV-01, PRIV-02, PRIV-03
**Success Criteria** (what must be TRUE):

1. The post-process provider dropdown renders Ollama, Apple Intelligence, and Custom local providers above a visible "External — data leaves this device" section that groups OpenAI, Anthropic, Groq, and Gemini
2. A `docs/PRIVACY.md` file exists listing every outbound endpoint the app can contact, what data leaves the device, and how to disable each connection
3. Onboarding screens present local transcription as the primary path; any cloud post-processing option is visibly labeled as an opt-in external service
   **Plans**: 10 plans (5 original + 2 gap-closure round 1 [page reframe + i18n] + 3 gap-closure round 2 [tabs pivot + bug fixes + i18n] addressing UAT failures from 2026-05-22)

- [ ] 08-01-PLAN.md — Platform-aware default + relabel `Custom (local)` + Rust unit tests (PRIV-01)
- [ ] 08-02-PLAN.md — Author `docs/PRIVACY.md` + README link + UPSTREAM maintenance hook (PRIV-02)
- [ ] 08-03-PLAN.md — ProviderPicker, TestConnectionButton, Ollama tip, hook extension, toggle promotion, About panel link, English i18n keys (PRIV-01, PRIV-02, PRIV-03)
- [ ] 08-04-PLAN.md — Replicate 14 new i18n keys across 19 sibling locales (PRIV-01, PRIV-02)
- [ ] 08-05-PLAN.md — Human UAT — visual verification of stacked picker, Test connection, About link, toggle relocation, onboarding, PRIVACY.md rendering [has checkpoint]
- [ ] 08-06-PLAN.md — **[Gap closure]** Cloud-providers toggle (OFF by default) gating cloud section + page reframe to "Modèles et traitement local" with status badge, selected model card with privacy tags, library coming-soon placeholder, three-pillar marketing block; backend setting + English i18n source (PRIV-01, PRIV-02, PRIV-03)
- [x] 08-07-PLAN.md — **[Gap closure]** Replicate 25 new Phase 8 gap-closure i18n keys across 19 sibling locales (PRIV-01, PRIV-02, PRIV-03) (completed 2026-05-22)
- [ ] 08-08-PLAN.md — **[Gap closure]** Bug fixes: inline Apple Intelligence Alert via renderRowExtras + Ollama link rest-state underline + hide API key field for Custom (local) — closes gaps 1a/2a/2b (PRIV-01)
- [ ] 08-09-PLAN.md — **[Gap closure]** Design pivots: Local/Cloud tabs replace cloud toggle + remove three-pillar block + hoist library placeholder to top; English i18n source updates — closes gaps 5/6/7 (PRIV-01, PRIV-02, PRIV-03)
- [ ] 08-10-PLAN.md — **[Gap closure]** Propagate tabs._, removed pillars._, removed cloudToggle.\*, updated cloudSelectedNotice across 19 sibling locales (PRIV-01, PRIV-02, PRIV-03)
