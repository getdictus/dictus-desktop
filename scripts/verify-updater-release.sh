#!/usr/bin/env bash
# verify-updater-release.sh — UPDT-10 assertion
# Fetches latest.json from the published GitHub Release and verifies structure.
# Exit 0 = release is healthy. Non-zero = broken (404, empty JSON, redirect, missing platforms).
# Called manually by Pierre AFTER publishing a draft release per docs/RUNBOOK-updater-signing.md §6.6.
set -euo pipefail

ENDPOINT="https://github.com/getdictus/dictus-desktop/releases/latest/download/latest.json"

command -v jq >/dev/null 2>&1 || { echo "jq required — brew install jq"; exit 2; }
command -v curl >/dev/null 2>&1 || { echo "curl required"; exit 2; }

echo "Fetching $ENDPOINT"
if ! JSON=$(curl -sfL "$ENDPOINT"); then
  echo "FAIL — curl returned non-2xx (possible 404, redirect, or network error)"
  exit 1
fi

echo "$JSON" | jq -e '.version and .platforms and (.platforms | keys | length > 0)' >/dev/null || {
  echo "FAIL — latest.json missing .version, .platforms, or has empty platforms object"
  echo "$JSON"
  exit 1
}

VERSION=$(echo "$JSON" | jq -r .version)
PLATFORMS=$(echo "$JSON" | jq -r '.platforms | keys | join(", ")')
echo "ok  version=$VERSION"
echo "ok  platforms=$PLATFORMS"
echo
echo "UPDT-10 PASS — latest.json is published and structurally valid"
