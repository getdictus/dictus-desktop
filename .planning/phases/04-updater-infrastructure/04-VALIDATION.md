---
phase: 4
slug: updater-infrastructure
status: accepted
nyquist_compliant: true
wave_0_complete: true
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
| 4-01-01 | 04-01 | 1 | UPDT-01, UPDT-02 (runbook) | runbook content grep | `test -f docs/RUNBOOK-updater-signing.md && grep -q 'Ed25519' docs/RUNBOOK-updater-signing.md` | ✅ | ⬜ pending |
| 4-01-02 | 04-01 | 1 | UPDT-03..UPDT-09 (validator) | bash syntax + content | `bash -n .planning/phases/04-updater-infrastructure/scripts/validate.sh` | ✅ | ⬜ pending |
| 4-01-03 | 04-01 | 1 | UPDT-10 (curl wrapper) | bash syntax + content | `bash -n scripts/verify-updater-release.sh` | ✅ | ⬜ pending |
| 4-02-01 | 04-02 | 1 | UPDT-06, UPDT-07, UPDT-08 | workflow grep | `grep -q 'asset-prefix: "dictus"' .github/workflows/release.yml && grep -q 'default: "dictus"' .github/workflows/build.yml && grep -q 'includeUpdaterJson: true' .github/workflows/build.yml` | ✅ | ⬜ pending |
| 4-02-02 | 04-02 | 1 | UPDT-09 | code grep | `grep -q 'getdictus/dictus-desktop' src/components/update-checker/UpdateChecker.tsx && ! grep -q 'cjpais/Handy' src/components/update-checker/UpdateChecker.tsx` | ✅ | ⬜ pending |
| 4-03-01 | 04-03 | 2 | UPDT-01 | manual keygen | Pierre generates Ed25519 keypair locally; verifies via resume signal | N/A | ⬜ pending |
| 4-03-02 | 04-03 | 2 | UPDT-02 | manual GitHub Secrets | `gh secret list --repo getdictus/dictus-desktop \| grep TAURI_SIGNING` | N/A | ⬜ pending |
| 4-03-03 | 04-03 | 2 | UPDT-03, UPDT-04, UPDT-05 | jq config grep | `jq -e '.bundle.createUpdaterArtifacts == true' src-tauri/tauri.conf.json && jq -e '.plugins.updater.pubkey \| length > 0' src-tauri/tauri.conf.json && jq -e '.plugins.updater.endpoints[0] \| test("getdictus/dictus-desktop")' src-tauri/tauri.conf.json` | ✅ | ⬜ pending |
| 4-04-01 | 04-04 | 3 | UPDT-10 | manual release | Pierre triggers release.yml, verifies draft, publishes; resume signal = release URL | N/A | ⬜ pending |
| 4-04-02 | 04-04 | 3 | UPDT-10 | curl + jq assertion | `bash scripts/verify-updater-release.sh && bash .planning/phases/04-updater-infrastructure/scripts/validate.sh` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `.planning/phases/04-updater-infrastructure/scripts/validate.sh` — shell-based config assertions (created in Plan 04-01 Task 2)
- [x] `scripts/verify-updater-release.sh` — UPDT-10 curl+jq wrapper (created in Plan 04-01 Task 3)
- [x] `jq` available on CI runners (usually pre-installed; validate.sh fails fast with `jq required` if missing)

*Assertion-based validation — no test framework install needed. Wave 0 is satisfied entirely by Plan 04-01.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Keypair generated and backed up | UPDT-01 | Local keygen requires interactive passphrase entry; private key must never pass through an automated environment | Pierre follows `docs/RUNBOOK-updater-signing.md` §3 — runs `bunx tauri signer generate`, backs up to Bitwarden + age-encrypted offline file, then deletes local copy. Resume signal = posting the (non-secret) raw base64 pubkey back to chat. |
| GitHub Secrets set on the repo | UPDT-02 | Secret values come from a private key Claude must never see; entry must be done in browser UI | Pierre adds `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` at https://github.com/getdictus/dictus-desktop/settings/secrets/actions. Secondary check (safe): `gh secret list --repo getdictus/dictus-desktop` shows both secret names (values not exposed). |
| Dry-run release produces `latest.json` | UPDT-10 | Requires Pierre to trigger `release.yml` and inspect GitHub Releases | Pierre triggers release workflow, waits for 7-platform builds, verifies draft has `latest.json` + `.sig` files + `dictus_*` prefix, publishes. Then `bash scripts/verify-updater-release.sh` asserts the published endpoint is structurally valid. |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or are explicitly `N/A — manual` with documented reason
- [x] Sampling continuity: no 3 consecutive tasks without automated verify (manual tasks are bracketed by automated checks before/after)
- [x] Wave 0 covers all MISSING references (validate.sh + verify-updater-release.sh both created in Plan 04-01)
- [x] No watch-mode flags
- [x] Feedback latency < 10s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** accepted
