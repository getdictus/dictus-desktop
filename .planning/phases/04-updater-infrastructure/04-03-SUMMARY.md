---
phase: 04-updater-infrastructure
plan: 03
subsystem: infra
tags: [tauri-updater, ed25519, minisign, github-secrets, signing]

requires:
  - phase: 04-updater-infrastructure-01
    provides: RUNBOOK-updater-signing.md (step-by-step Pierre followed for keypair gen)
  - phase: 04-updater-infrastructure-02
    provides: tauri.conf.json scaffold with empty plugins.updater.{pubkey,endpoints} and bundle.createUpdaterArtifacts=false — Plan 03 populated them

provides:
  - Ed25519 signing keypair owned by Pierre, backed up to three independent locations (local private key deleted)
  - GitHub Secrets TAURI_SIGNING_PRIVATE_KEY and TAURI_SIGNING_PRIVATE_KEY_PASSWORD set on getdictus/dictus-desktop
  - tauri.conf.json with live updater config (pubkey + endpoint + createUpdaterArtifacts=true)
  - First green run of validate.sh for UPDT-03..UPDT-09

affects: [04-updater-infrastructure-04, 05-upstream-sync, release-pipeline]

tech-stack:
  added: []  # No new libs — all config-only
  patterns:
    - "Offline keypair generation + triple-backup pattern for root-of-trust keys (Bitwarden x2 + offline age-encrypted)"
    - "Raw base64 pubkey in tauri.conf.json (no PEM armor) — matches tauri signer generate output verbatim"
    - "Direct asset download URL (releases/latest/download/latest.json) — not HTML release page"

key-files:
  created:
    - .planning/phases/04-updater-infrastructure/deferred-items.md
  modified:
    - src-tauri/tauri.conf.json

key-decisions:
  - "Triple backup: Bitwarden key note + Bitwarden passphrase note (separate) + iCloud Drive age -p encrypted file. Key material and passphrase intentionally split across two Bitwarden items for defense in depth."
  - "iCloud age -p offline backup accepted in lieu of yubikey-age wrap (plan's original spirit was 'independent offline encrypted backup' — age -p with a strong independent passphrase satisfies that)."
  - "Task 3 executed before Task 2 completion on disk because Task 3 has zero runtime dependency on GitHub Secrets — secrets only matter at CI build time. User set secrets in parallel."
  - "Repo visibility left as PRIVATE for now; public-flip tracked as a Plan 04-04 pre-flight gate (not a 04-03 blocker since file edits work regardless of repo visibility)."

patterns-established:
  - "Key generation as checkpoint:human-action: private key material never passes through Claude's shell, is never base64-echoed, is never copied to scratch files."
  - "GitHub Secrets entry as checkpoint:human-action: values flow from Bitwarden → browser UI directly, bypassing the agent."
  - "Phase validator runs at the end of each contributing plan to confirm assertion flip (from red to green) — 04-03 is the point where UPDT-03..09 all turn green simultaneously."

requirements-completed: [UPDT-01, UPDT-02, UPDT-03, UPDT-04, UPDT-05]

duration: ~15min  # Task 3 execution only; Tasks 1+2 were human-action out-of-band
completed: 2026-04-11
---

# Phase 04 Plan 03: Updater Signing Keypair + Config Wiring Summary

**Ed25519 keypair generated offline and triple-backed-up, GitHub Secrets wired, tauri.conf.json populated with live pubkey + latest.json endpoint — updater pipeline is now config-complete.**

## Performance

- **Duration:** ~15 min (Task 3 automated portion; Tasks 1–2 were out-of-band human actions)
- **Started (Task 3):** 2026-04-11T16:12Z
- **Completed (Task 3):** 2026-04-11T16:28Z
- **Tasks:** 3 (1 human-action, 1 human-action, 1 auto)
- **Files modified:** 1 code file + 1 deferred-items log

## Accomplishments

- **Root-of-trust keypair established.** Pierre generated an Ed25519 keypair via `bunx tauri signer generate -w ~/.tauri/dictus.key`, backed it up to three independent locations (Bitwarden "Dictus Updater Signing Key" note, Bitwarden "Dictus Updater Signing Passphrase" note, and an iCloud Drive `~/dictus.key.age` encrypted with `age -p`), then deleted the local private key. Only `~/.tauri/dictus.key.pub` remains on disk.
- **CI signing credentials wired.** `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` set as repository secrets on getdictus/dictus-desktop, consumed by the existing `${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}` references at `.github/workflows/build.yml:354-355` — no workflow edit needed.
- **Runtime updater config populated.** `src-tauri/tauri.conf.json` now carries the raw base64 Ed25519 pubkey (no PEM armor), the direct asset-download endpoint URL (`.../releases/latest/download/latest.json`), and `bundle.createUpdaterArtifacts: true` to enable .sig file generation at build.
- **Phase validator flipped green for UPDT-03..09.** First end-to-end pass of `.planning/phases/04-updater-infrastructure/scripts/validate.sh` shows all seven updater assertions passing. The only remaining failure is a pre-existing anti-regression check mismatch (logged to deferred-items.md, owned by Plan 04-01).

## Task Commits

1. **Task 1: Generate Ed25519 keypair + triple backup** — no commit (human-action executed out-of-band on Pierre's local machine; private key material intentionally never passed through the agent)
2. **Task 2: Set GitHub Secrets on getdictus/dictus-desktop** — no commit (human-action executed via browser UI; values sourced from Bitwarden, not from filesystem)
3. **Task 3: Populate tauri.conf.json updater config fields** — `24fbdc7` (feat)

## Files Created/Modified

- `src-tauri/tauri.conf.json` — Three field changes:
  - `bundle.createUpdaterArtifacts`: `false` → `true`
  - `plugins.updater.pubkey`: `""` → raw base64 Ed25519 pubkey (114 decoded bytes, minisign format)
  - `plugins.updater.endpoints`: `[]` → `["https://github.com/getdictus/dictus-desktop/releases/latest/download/latest.json"]`
- `.planning/phases/04-updater-infrastructure/deferred-items.md` — New phase-local log of out-of-scope discoveries (first entry: validator anti-regression check mismatch)

### Exact git diff on src-tauri/tauri.conf.json

```diff
@@ -25,7 +25,7 @@
   },
   "bundle": {
     "active": true,
-    "createUpdaterArtifacts": false,
+    "createUpdaterArtifacts": true,
     "targets": "all",
     "resources": ["resources/**/*"],
     "license": "MIT",
@@ -65,8 +65,10 @@
   },
   "plugins": {
     "updater": {
-      "pubkey": "",
-      "endpoints": []
+      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDczQ0M5QzAyNDA2QzVBNzYKUldSMldteEFBcHpNYzZjaHRGOGV2QkZkUGwrYSswU3NKbkw0MmF4R1cxOW1yYkxaakNUazNNR0UK",
+      "endpoints": [
+        "https://github.com/getdictus/dictus-desktop/releases/latest/download/latest.json"
+      ]
     }
   }
 }
```

Changes are scoped exclusively to the three target fields. No other field touched.

### UPDT-03/04/05 automated verify (Task 3 scoped block)

```
PASS: valid JSON                       (jq empty)
PASS: createUpdaterArtifacts == true   (jq -e)
PASS: pubkey non-empty                 (length > 0)
PASS: pubkey decodes                   (base64 -d = 114 bytes)
PASS: endpoint URL correct             (exact string match)
PASS: no PEM armor                     (grep BEGIN = no match)
```

### Full validate.sh run (UPDT-03..UPDT-09 + version consistency)

```
ok  UPDT-03 tauri.conf.json plugins.updater.pubkey non-empty base64
ok  UPDT-04 tauri.conf.json plugins.updater.endpoints[0] points to getdictus/dictus-desktop
ok  UPDT-05 tauri.conf.json bundle.createUpdaterArtifacts == true
ok  UPDT-06 release.yml asset-prefix is "dictus"
ok  UPDT-07 build.yml default asset-prefix is "dictus"
ok  UPDT-08 build.yml has includeUpdaterJson: true
ok  UPDT-09 UpdateChecker.tsx fallback URL points to getdictus/dictus-desktop
ok  version consistency src-tauri/Cargo.toml vs src-tauri/tauri.conf.json
```

(One validator-ergonomics failure remains — see Deviations below.)

## Decisions Made

- **Triple-backup split intentionally.** Bitwarden note A holds the key material; Bitwarden note B holds the passphrase; they are separate items so a single-note compromise yields neither a usable key nor a decryptable one.
- **age -p (passphrase-encrypted) accepted instead of age -R (recipient/yubikey-wrapped).** The plan's original spirit — "independent offline encrypted backup on external media" — is satisfied by a passphrase-protected age file on iCloud Drive using a strong passphrase distinct from both Bitwarden master password and the key's own passphrase. Revisit if/when a yubikey-age setup exists.
- **Task 3 ran before Task 2 completion on disk.** Task 3 has no runtime dependency on GitHub Secrets — secrets are only consumed by CI builds. Running Task 3 in parallel with the user's browser action shortened wall-clock time without violating any correctness invariant.
- **Repo visibility deferred.** Repo is currently PRIVATE. This is already tracked in STATE.md as a known Phase 4 blocker and will be gated at the start of Plan 04-04 (before the dry-run release is cut). Not a 04-03 concern because file edits are visibility-agnostic.

## Deviations from Plan

### Auto-fixed Issues

**None from Rules 1–3 applied during Task 3 execution.** The three file edits landed exactly as specified.

### Deferred Issues (out of scope for Plan 04-03)

**1. [Scope Boundary] validate.sh anti-regression check fails**
- **Found during:** Task 3 final validator run
- **Issue:** `grep -B5 'otool -L' .github/workflows/build.yml | grep -q 'TECH-03'` fails because the TECH-03 deferral comment is at lines 375–376 but the first `otool -L` invocation is at line 385 — 9-10 lines of context, outside `-B5`.
- **Why deferred:** Pre-existing at the moment `validate.sh` was committed (Plan 04-01 commit `28f8255` added the validator AFTER Plan 04-02 commit `7105735` placed the comment). Plan 04-03 did not touch either file in the failing assertion. Per GSD scope boundary, auto-fix only applies to issues directly caused by the current task's changes.
- **Logged to:** `.planning/phases/04-updater-infrastructure/deferred-items.md` (with two proposed minute-sized fixes for the owning plan).
- **Impact:** Zero on updater correctness. UPDT-03..09 all pass. This is purely a validator ergonomics issue.

---

**Total deviations:** 0 auto-fixes; 1 deferred out-of-scope item logged.
**Impact on plan:** None. Plan 04-03 scope (UPDT-01..05) is fully complete.

## Issues Encountered

- **Task 1/2 parallelization request.** User asked for Task 3 to run before confirming Task 2 GitHub Secrets entry, since Task 3 is a pure file edit. Evaluated against the plan's `gate="blocking"` annotation on Task 2 and concluded the block is about out-of-band workflow ordering, not a technical dependency. Proceeded in parallel, with user completing Task 2 concurrently.
- **Repo visibility is PRIVATE.** Acknowledged and deferred to Plan 04-04 pre-flight (already a tracked Phase 4 blocker in STATE.md). No action this plan.

## User Setup Required

Task 1 and Task 2 of this plan WERE the user setup. Both are complete:
- Ed25519 keypair generated on Pierre's local machine
- Private key backed up to Bitwarden (key material + passphrase as separate notes) + iCloud Drive (age -p encrypted)
- Local `~/.tauri/dictus.key` deleted; only `~/.tauri/dictus.key.pub` remains
- `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` set on https://github.com/getdictus/dictus-desktop/settings/secrets/actions

**Reminder:** Do NOT commit or paste the private key anywhere. Only the pubkey appears in tauri.conf.json, and the pubkey is NOT secret by design (Minisign Ed25519).

## Next Phase Readiness

**Ready for Plan 04-04 (dry-run release):**
- Signing pipeline is fully wired: build-time (CI secrets + createUpdaterArtifacts) and runtime (pubkey + endpoint).
- First signed release build via tauri-action will produce `.sig` files, and `includeUpdaterJson: true` in build.yml will auto-generate `latest.json` alongside the artifacts.
- Installed clients reading `tauri.conf.json` will correctly verify signatures and hit the GitHub Releases latest.json endpoint.

**Blockers for Plan 04-04:**
- **Repo must be flipped from PRIVATE to PUBLIC** before the first release is cut, or the `releases/latest/download/latest.json` endpoint will 302-redirect to a login page and UPDT-10 will fail. This is a pre-flight gate for Plan 04-04 Task 1.
- Deferred validator anti-regression check mismatch should be fixed early in Plan 04-04 (or folded into a 04-01 follow-up commit) so the validator is trustworthy for the final v1.1 readiness check.

## Self-Check: PASSED

- FOUND: src-tauri/tauri.conf.json (modified with three target fields)
- FOUND: .planning/phases/04-updater-infrastructure/deferred-items.md (new, logs validator mismatch)
- FOUND: .planning/phases/04-updater-infrastructure/04-03-SUMMARY.md (this file)
- FOUND: commit 24fbdc7 in git log (Task 3 atomic commit)

All referenced artifacts exist. UPDT-03..UPDT-09 validator assertions all pass on this HEAD.

---
*Phase: 04-updater-infrastructure*
*Completed: 2026-04-11*
