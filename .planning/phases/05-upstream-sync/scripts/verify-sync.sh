#!/usr/bin/env bash
# Phase 5 validator — SYNC-05 post-merge identity + build assertions.
# Run after merging upstream/main to verify Dictus identity has not regressed.
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

echo "Phase 5 — Upstream Sync identity + build assertions"
echo

check "SYNC-05a productName is Dictus" \
  "grep -q '\"productName\": \"Dictus\"' src-tauri/tauri.conf.json"

check "SYNC-05b identifier is com.dictus.desktop" \
  "grep -q '\"identifier\": \"com.dictus.desktop\"' src-tauri/tauri.conf.json"

check "SYNC-05c updater endpoint is getdictus/dictus-desktop" \
  "jq -e '.plugins.updater.endpoints[0] | test(\"getdictus/dictus-desktop\")' src-tauri/tauri.conf.json"

check "SYNC-05d no Handy in en i18n (except attribution)" \
  "! awk '/\"acknowledgments\"/{skip=1} /^\s*},$/{if(skip)skip=0} !skip' src/i18n/locales/en/translation.json | grep -q '\"Handy\"'"

check "SYNC-05e llm_client User-Agent is Dictus/1.0" \
  "grep -q 'Dictus/1.0' src-tauri/src/llm_client.rs"

check "SYNC-05f llm_client X-Title is Dictus" \
  "grep -q '\"X-Title\".*\"Dictus\"\|\"Dictus\"' src-tauri/src/llm_client.rs"

check "SYNC-05g no handy.computer reference in actions.rs" \
  "! grep 'handy\.computer' src-tauri/src/actions.rs"

check "SYNC-05h no handy.computer reference in llm_client.rs" \
  "! grep 'handy\.computer' src-tauri/src/llm_client.rs"

check "SYNC-05i no handy.log reference in source code (Pitfall 6)" \
  "! grep -rn 'handy\.log' src/ src-tauri/src/ 2>/dev/null"

check "SYNC-05j cargo build succeeds" \
  "cargo build --manifest-path src-tauri/Cargo.toml 2>&1"

echo
if [ $FAIL -eq 0 ]; then
  echo "All Phase 5 checks passed."
  exit 0
else
  echo "One or more Phase 5 checks FAILED."
  exit 1
fi
