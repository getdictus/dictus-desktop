# Roadmap: Dictus Desktop

## Milestones

- ✅ **v1.0 Handy→Dictus Rebrand** — Phases 1-3 (shipped 2026-04-10) — [archive](milestones/v1.0-ROADMAP.md)
- ✅ **v1.1 Auto-Update & Upstream Sync** — Phases 4-5 (shipped 2026-04-14) — [archive](milestones/v1.1-ROADMAP.md)
- 🚧 **v1.2 Polish & Automation** — Phases 6-10 (in progress)

## Phases

<details>
<summary>✅ v1.0 Handy→Dictus Rebrand (Phases 1-3) — SHIPPED 2026-04-10</summary>

- [x] Phase 1: Bundle Identity (1/1 plans) — completed 2026-04-05
- [x] Phase 2: Visual Rebrand (5/5 plans) — completed 2026-04-09
- [x] Phase 3: Documentation and Cleanup (2/2 plans) — completed 2026-04-09

</details>

<details>
<summary>✅ v1.1 Auto-Update & Upstream Sync (Phases 4-5) — SHIPPED 2026-04-14</summary>

- [x] Phase 4: Updater Infrastructure (4/4 plans) — completed 2026-04-13
- [x] Phase 5: Upstream Sync (3/3 plans) — completed 2026-04-14

</details>

### 🚧 v1.2 Polish & Automation (In Progress)

**Milestone Goal:** Polish Dictus identity across all user-visible surfaces, fix platform icon artifacts, resolve the macOS clean-shutdown crash, harden the upstream sync CI gate, and layer Claude Code agents onto the sync workflow so future merges get automated identity remediation and an independent audit.

- [x] **Phase 6: Brand & Icon Polish** - Fix all remaining Handy brand leaks and platform icon artifacts; extend verify-sync.sh to guard them (absorbs SYNC-06 from Phase 9) — completed 2026-04-16
- [ ] **Phase 7: macOS Clean Shutdown** - Diagnose and fix the "Dictus quit unexpectedly" crash dialog on macOS Sequoia
- [ ] **Phase 8: Privacy / Local-First UX** - Reorder post-process providers (local first) and document the app's network surface
- [ ] **Phase 9: Sync Infrastructure Refactor** - Replace issue-based detection with community-action draft PRs and promote verify-sync.sh to a required CI gate
- [ ] **Phase 10: Claude Code Agent Layer** - Add adapter + auditor Claude Code agents that fire on labeled upstream-sync PRs

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

**Note:** After Phase 6 ships, run `/gsd:roadmap-update` to reflect SYNC-06 completion — Phase 9 scope will reduce to SYNC-07/08/09/10/11 only.

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
**Plans**: TBD

### Phase 9: Sync Infrastructure Refactor
**Goal**: The upstream sync workflow produces a labeled draft PR automatically each week instead of a tracking issue; verify-sync.sh is promoted to `.github/scripts/`, extended with UPDT-03/UPDT-05 assertions, and enforced as a required CI status check on all upstream-sync PRs.
**Depends on**: Phase 6 (verify-sync.sh extended assertions must be in place before the CI gate goes live)
**Requirements**: SYNC-06, SYNC-07, SYNC-08, SYNC-09, SYNC-10, SYNC-11
**Success Criteria** (what must be TRUE):
  1. `verify-sync.sh` lives at `.github/scripts/verify-sync.sh` and exits non-zero if `plugins.updater.pubkey` is absent or if `plugins.updater.endpoints[0]` does not contain `getdictus/dictus-desktop`
  2. A PR labeled `upstream-sync` triggers the `verify-sync.yml` workflow and its result appears as a required status check — the PR cannot merge to `main` while that check is red
  3. On the weekly cron, `upstream-sync.yml` opens a draft PR (not an issue) with upstream commits on a dedicated branch; re-running while that branch exists does not open a duplicate PR
  4. Every draft upstream-sync PR opens with the `.github/PULL_REQUEST_TEMPLATE/upstream-sync.md` checklist pre-filled in its body
**Plans**: TBD

### Phase 10: Claude Code Agent Layer
**Goal**: Every upstream-sync draft PR automatically receives an adapter pass (agent commits identity fixes within a scoped allow-list) followed by an independent auditor pass (read-only agent posts a review comment) — and neither agent can bypass the `verify-sync.yml` CI gate.
**Depends on**: Phase 9 (labeled draft PRs must exist before agents have meaningful input)
**Requirements**: AGENT-01, AGENT-02, AGENT-03, AGENT-04, AGENT-05, AGENT-06
**Success Criteria** (what must be TRUE):
  1. `CLAUDE_CODE_OAUTH_TOKEN` is configured in repo secrets and the agent workflow runs without API-key billing errors
  2. An upstream-sync draft PR triggers the adapter agent job; the adapter commits only to brand/identity surfaces within its explicit file allow-list and does not modify files outside that scope
  3. After the adapter job completes, the auditor job runs with read-only permissions and posts a review comment on the PR summarizing residual Handy strings, brand compliance findings, and code-quality observations — it does not push any commits
  4. The `verify-sync.yml` CI gate runs as an independent workflow step regardless of what the adapter or auditor agents do — a compromised or misbehaving agent cannot cause it to be skipped

## Progress

**Execution Order:**
v1.2 phases execute in order: 6 → 7 → 8 → 9 → 10
(Phases 6, 7, 8 are independent and can proceed on separate branches; Phase 9 requires Phase 6 complete; Phase 10 requires Phase 9 complete.)

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Bundle Identity | v1.0 | 1/1 | Complete | 2026-04-05 |
| 2. Visual Rebrand | v1.0 | 5/5 | Complete | 2026-04-09 |
| 3. Documentation and Cleanup | v1.0 | 2/2 | Complete | 2026-04-09 |
| 4. Updater Infrastructure | v1.1 | 4/4 | Complete | 2026-04-13 |
| 5. Upstream Sync | v1.1 | 3/3 | Complete | 2026-04-14 |
| 6. Brand & Icon Polish | v1.2 | 4/4 | Complete | 2026-04-16 |
| 7. macOS Clean Shutdown | v1.2 | 0/TBD | Not started | - |
| 8. Privacy / Local-First UX | v1.2 | 0/TBD | Not started | - |
| 9. Sync Infrastructure Refactor | v1.2 | 0/TBD | Not started | - |
| 10. Claude Code Agent Layer | v1.2 | 0/TBD | Not started | - |
