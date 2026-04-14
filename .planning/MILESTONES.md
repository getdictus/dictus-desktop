# Milestones

## v1.1 Auto-Update & Upstream Sync (Shipped: 2026-04-14)

**Phases completed:** 2 phases, 7 plans

**Stats:**
- Files changed: 49 | LOC: +4277/-517
- Git range: `84cc346` → `bedf363` (27 commits)
- Timeline: 2026-04-11 → 2026-04-14 (4 days)
- App releases shipped within this milestone: `v0.1.0` (2026-04-11)
- Audit status: `tech_debt` — all 15 requirements satisfied; non-blocking debt documented below

> GSD milestones are internal planning units and **do not** produce git tags.
> See `docs/VERSIONING.md` for the tag/version convention.

**Key accomplishments:**
- **Auto-updater end-to-end** — Ed25519 signing keypair, tauri.conf.json updater config (pubkey + endpoint + createUpdaterArtifacts), CI asset-prefix fixes, and UpdateChecker.tsx fallback corrected (UPDT-01…UPDT-09)
- **v0.1.0 shipped** — first public Dictus Desktop release on `getdictus/dictus-desktop` with latest.json live and tauri-updater assertions green (UPDT-10)
- **Upstream detection automated** — weekly `upstream-sync.yml` workflow compares `cjpais/Handy` HEAD to committed SHA and files a GitHub issue, idempotent on unchanged state (SYNC-01, SYNC-02)
- **UPSTREAM.md runbook** — fork-point-aware merge procedure with 9 hot-zone conflict rules and post-merge verification checklist (SYNC-03)
- **First upstream sync merged** — 4 post-v0.8.2 commits from `cjpais/Handy` merged via PR #3, capped at `fdc8cb7`, identity integrity preserved through `verify-sync.sh` (SYNC-04, SYNC-05)
- **Triple-backup signing custody** — Ed25519 private key backed up to two independent Bitwarden items + iCloud age-encrypted offline, local copy deleted

**Known tech debt (non-blocking):**
- Post-sync gate in UPSTREAM.md §6 does not re-assert UPDT-03/UPDT-05 — regression risk flagged for v1.2
- Phase 5 VALIDATION.md left in draft (`nyquist_compliant: false`) — `/gsd:validate-phase 5` to close retroactively
- `validate.sh` TECH-03 anti-regression grep window too narrow; 8 substantive config assertions still pass
- `build.yml` still references `blob.handy.computer` for onnxruntime (tracked as INFR-01)
- v0.1.0 Windows builds unsigned at OS level (SmartScreen warning expected, Azure Trusted Signing deferred)

See `.planning/v1.1-MILESTONE-AUDIT.md` (archived to `.planning/milestones/v1.1-MILESTONE-AUDIT.md`) for full audit report.

---

## v1.0 Dictus Desktop V1 (Shipped: 2026-04-10)

**Phases completed:** 3 phases, 8 plans, 0 tasks

**Key accomplishments:**
- (none recorded)

---
