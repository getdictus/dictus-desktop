---
phase: 4
slug: updater-infrastructure
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-11
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | config-grep assertions (no new test framework) |
| **Config file** | none — assertions run via shell/Python one-liners against repo files |
| **Quick run command** | `bash .planning/phases/04-updater-infrastructure/scripts/validate.sh` |
| **Full suite command** | `bun run lint && bash .planning/phases/04-updater-infrastructure/scripts/validate.sh` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `bash .planning/phases/04-updater-infrastructure/scripts/validate.sh`
- **After every plan wave:** Run `bun run lint && bash .planning/phases/04-updater-infrastructure/scripts/validate.sh`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 4-01-01 | 01 | 1 | UPDT-01 | manual runbook | `test -f docs/RUNBOOK-updater-signing.md` | ❌ W0 | ⬜ pending |
| 4-01-02 | 01 | 1 | UPDT-02 | manual runbook | `grep -q 'Ed25519' docs/RUNBOOK-updater-signing.md` | ❌ W0 | ⬜ pending |
| 4-02-01 | 02 | 2 | UPDT-03 | config grep | `jq -e '.bundle.createUpdaterArtifacts == true' src-tauri/tauri.conf.json` | ✅ | ⬜ pending |
| 4-02-02 | 02 | 2 | UPDT-04 | config grep | `jq -e '.plugins.updater.endpoints[0] \| test("getdictus/dictus-desktop")' src-tauri/tauri.conf.json` | ✅ | ⬜ pending |
| 4-02-03 | 02 | 2 | UPDT-05 | config grep | `jq -e '.plugins.updater.pubkey \| length > 0' src-tauri/tauri.conf.json` | ✅ | ⬜ pending |
| 4-03-01 | 03 | 2 | UPDT-06 | workflow grep | `grep -q 'asset-prefix: "dictus"' .github/workflows/release.yml` | ✅ | ⬜ pending |
| 4-03-02 | 03 | 2 | UPDT-07 | workflow grep | `grep -q 'asset-prefix: "dictus"' .github/workflows/build.yml` | ✅ | ⬜ pending |
| 4-03-03 | 03 | 2 | UPDT-08 | workflow grep | `grep -q 'includeUpdaterJson: true' .github/workflows/build.yml` | ✅ | ⬜ pending |
| 4-04-01 | 04 | 2 | UPDT-09 | code grep | `grep -q 'getdictus/dictus-desktop' src/components/update-checker/UpdateChecker.tsx` | ✅ | ⬜ pending |
| 4-05-01 | 05 | 3 | UPDT-10 | manual release | Pierre triggers release.yml, verifies `latest.json` accessible | ❌ manual | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `.planning/phases/04-updater-infrastructure/scripts/validate.sh` — shell-based config assertions
- [ ] `jq` available on CI runners (usually pre-installed; verify in validate.sh)

*Assertion-based validation — no test framework install needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Keypair generated and backed up | UPDT-01, UPDT-02 | Involves local keygen + secret backup, not reproducible in CI | Follow `docs/RUNBOOK-updater-signing.md` — run `bunx tauri signer generate`, back up private key to 1Password/secure location, add public key to `tauri.conf.json`, add private key + password to GitHub Secrets |
| Dry-run release produces `latest.json` | UPDT-10 | Requires Pierre to trigger `release.yml` and inspect GitHub Releases | Pierre triggers release workflow, waits for 7 platform builds, verifies `latest.json` artifact appears on GitHub Releases page and is fetchable via `curl` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
