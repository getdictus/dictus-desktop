---
gsd_state_version: 1.0
milestone: v1.2
milestone_name: Polish & Automation
status: defining_requirements
stopped_at: "v1.2 kickoff — scope aligned, gathering requirements"
last_updated: "2026-04-15T00:00:00.000Z"
last_activity: "2026-04-15 — v1.2 Polish & Automation milestone started"
progress:
  total_phases: 0
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-04-15 at v1.2 kickoff)

**Core value:** L'application doit être identifiable et utilisable comme Dictus Desktop — pas comme Handy — et rester vivante (updates automatiques) sans décrocher du upstream Handy.
**Current focus:** v1.2 Polish & Automation — defining requirements

## Current Position

Phase: Not started (defining requirements)
Plan: —
Status: Defining requirements
Last activity: 2026-04-15 — Milestone v1.2 Polish & Automation started

Progress: [░░░░░░░░░░] 0% (v1.2 starting)

## Performance Metrics

**Velocity:**
- Total plans completed: 8 (v1.0)
- Average duration: unknown
- Total execution time: unknown

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 1-3 (v1.0) | 8 | - | - |
| Phase 04-updater-infrastructure P03 | ~15min | 3 tasks (2 human-action, 1 auto) | 2 files |
| Phase 04-updater-infrastructure P02 | 1 | 2 tasks | 3 files |
| Phase 04-updater-infrastructure P01 | 15 | 3 tasks | 3 files |
| Phase 05-upstream-sync P02 | 1 | 1 tasks | 1 files |
| Phase 05-upstream-sync P03 | 25min | 2 tasks | 27 files |

## Accumulated Context

### Decisions

- v1.0: Auto-updater disabled — no Dictus endpoint existed
- v1.0: release.yml asset-prefix left as "handy" — deferred to v1.1
- v1.1: Upstream delta is 4 commits (not 69) — merge-base is 39e855d, not the full fork point delta
- [Phase 04-updater-infrastructure]: TECH-03 (binary rename Contents/MacOS/handy) deferred post-v1.1; inline comment protects against accidental rename
- [Phase 04-updater-infrastructure]: includeUpdaterJson: true added to tauri-action — auto-generates latest.json per release without extra CI steps
- [Phase 04-01]: Wave 0 pattern: validate.sh intentionally expected to FAIL on current codebase; Plans 02 and 03 make it green
- [Phase 04-01]: TECH-03 anti-regression check uses simple grep -B5 form (no awk) for debuggability
- [Phase 04-03]: Triple-backup pattern for Ed25519 signing key = Bitwarden key note + Bitwarden passphrase note (separate) + offline age-encrypted file. Key material and passphrase split across two items for defense in depth.
- [Phase 04-03]: Raw base64 pubkey in tauri.conf.json (no PEM armor) — matches `tauri signer generate` output verbatim. PEM-wrapping causes runtime UnexpectedKeyId errors.
- [Phase 04-03]: Updater endpoint uses direct asset download URL (releases/latest/download/latest.json), NOT the HTML releases page URL.
- [Phase 04-03]: Task 3 executed in parallel with Task 2 (GitHub Secrets entry) because Task 3 has zero runtime dependency on the secrets — secrets only matter at CI build time.
- [Phase 04-03]: iCloud Drive `age -p` encrypted offline backup accepted in lieu of yubikey-age recipient wrap. Revisit if/when yubikey-age setup exists.
- [Phase 04-updater-infrastructure]: latest.json has 18 platform entries (not 7) — tauri-action creates one entry per bundle format per architecture; all 7 build matrix targets are represented
- [Phase 04-updater-infrastructure]: Windows builds unsigned at OS level (Authenticode) for v0.1.0 — Azure Trusted Signing not yet set up; Ed25519 .sig files present for tauri-updater, SmartScreen warning expected on first install
- [Phase 05-upstream-sync]: upstream-sha.txt updated only on merge-to-main (not on issue creation) — premature update would make weekly action report false idempotency before work is done (Pitfall 2)
- [Phase 05-upstream-sync]: SYNC-05d i18n check uses awk to skip acknowledgments block — legitimate attribution to Handy upstream project must not fail identity regression check
- [Phase 05-upstream-sync]: UPSTREAM.md placed at repo root alongside README.md for maximum visibility; copy-paste runbook codifies conflict rules for all 9 hot zones and Pitfalls 1/2/6
- [Phase 05-upstream-sync]: Capped merge at fdc8cb7 (not upstream/main HEAD aee682f) — 4 researched commits only, 6 deferred to Sync #2
- [Phase 05-upstream-sync]: AWS Bedrock commit (aee682f) excluded per local-first philosophy — flagged for Sync #2 discussion

### Pending Todos

4 pending — see `.planning/todos/pending/` for details.

### Blockers/Concerns

Carried into next milestone from v1.1 audit (`tech_debt` items):
- UPSTREAM.md §6 post-sync gate missing UPDT-03/UPDT-05 re-assertion — regression risk before Sync #2
- Phase 5 VALIDATION.md draft → run `/gsd:validate-phase 5` to close Nyquist coverage
- `blob.handy.computer` CDN still used for onnxruntime (INFR-01)
- Windows builds unsigned at OS level — Azure Trusted Signing pending (INFR-03)

Human verification still outstanding:
- Confirm v0.1.0 release assets all prefixed `dictus_` (27 assets)
- Trigger `upstream-sync.yml` with upstream==upstream-sha.txt → verify `changed=false` branch
- Post Sync #2, confirm no duplicate issue on subsequent Monday cron

## Session Continuity

Last session: 2026-04-14T21:00:00.000Z
Stopped at: v1.1 archived — next step is /gsd:new-milestone
Resume file: None
