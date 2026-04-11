#!/usr/bin/env bash
# Phase 4 validator — runs every UPDT-03..UPDT-09 config assertion.
# UPDT-01, UPDT-02 are manual (keypair + GitHub Secrets) — not checked here.
# UPDT-10 is manual (release trigger) — wrapped separately in scripts/verify-updater-release.sh.
set -euo pipefail

FAIL=0
check() {
  local label="$1"; shift
  if eval "$@" >/dev/null 2>&1; then
    echo "  ok  $label"
  else
    echo "  FAIL $label"
    echo "       cmd: $*"
    FAIL=1
  fi
}

command -v jq >/dev/null 2>&1 || { echo "jq required — brew install jq"; exit 2; }

echo "Phase 4 — Updater Infrastructure config assertions"
echo

# UPDT-03 — pubkey populated and valid base64
check "UPDT-03 tauri.conf.json plugins.updater.pubkey non-empty base64" \
  "jq -e '.plugins.updater.pubkey | length > 0' src-tauri/tauri.conf.json && jq -r '.plugins.updater.pubkey' src-tauri/tauri.conf.json | base64 -d >/dev/null"

# UPDT-04 — endpoint points to getdictus/dictus-desktop latest.json
check "UPDT-04 tauri.conf.json plugins.updater.endpoints[0] points to getdictus/dictus-desktop" \
  "jq -e '.plugins.updater.endpoints[0] | test(\"https://github.com/getdictus/dictus-desktop/releases/latest/download/latest.json\")' src-tauri/tauri.conf.json"

# UPDT-05 — bundle.createUpdaterArtifacts == true
check "UPDT-05 tauri.conf.json bundle.createUpdaterArtifacts == true" \
  "jq -e '.bundle.createUpdaterArtifacts == true' src-tauri/tauri.conf.json"

# UPDT-06 — release.yml asset-prefix
check "UPDT-06 release.yml asset-prefix is \"dictus\"" \
  "grep -q 'asset-prefix: \"dictus\"' .github/workflows/release.yml"

# UPDT-07 — build.yml default asset-prefix
check "UPDT-07 build.yml default asset-prefix is \"dictus\"" \
  "grep -q 'default: \"dictus\"' .github/workflows/build.yml"

# UPDT-08 — build.yml has includeUpdaterJson: true in tauri-action with: block
check "UPDT-08 build.yml has includeUpdaterJson: true" \
  "grep -q 'includeUpdaterJson: true' .github/workflows/build.yml"

# UPDT-09 — UpdateChecker.tsx fallback URL
check "UPDT-09 UpdateChecker.tsx fallback URL points to getdictus/dictus-desktop" \
  "grep -q 'https://github.com/getdictus/dictus-desktop/releases/latest' src/components/update-checker/UpdateChecker.tsx && ! grep -q 'cjpais/Handy' src/components/update-checker/UpdateChecker.tsx"

# Anti-regression: deferred TECH-03 comment MUST exist within 5 lines above the otool line in build.yml
# Simple grep heuristic — no awk. If the TECH-03 deferral comment is removed, this fails fast.
check "Anti-regression build.yml has TECH-03 deferral comment near otool line" \
  "grep -B5 'otool -L' .github/workflows/build.yml | grep -q 'TECH-03'"

# Version consistency (applies once Cargo.toml and tauri.conf.json both set)
check "version consistency src-tauri/Cargo.toml vs src-tauri/tauri.conf.json" \
  'test "$(grep -o "^version = \"[^\"]*\"" src-tauri/Cargo.toml | head -1 | cut -d\" -f2)" = "$(jq -r .version src-tauri/tauri.conf.json)"'

echo
if [ $FAIL -ne 0 ]; then
  echo "FAILED — fix issues above and re-run"
  exit 1
fi
echo "ALL CHECKS PASSED"
