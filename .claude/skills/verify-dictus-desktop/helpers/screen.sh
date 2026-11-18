#!/usr/bin/env bash
# Capture the current window as a screen-map image: <skill>/screens/<screen-id>.png (in the repo,
# overwritten in place) and a copy in the run dir. Only from a tier-1 run: its data is the isolated HOME.
#   screen.sh <screen-id> [run-dir]
set -euo pipefail
source "$(dirname "$0")/env.sh"
id="${1:?usage: screen.sh <screen-id> [run-dir]}"
run="$(resolve_run "${2:-}")" || die "no run of this session"
[ "$(ledger_get "$run" TIER)" = tier1 ] || die "screen-map captures come from tier 1 only"
"$DICTUS_HELPERS/shot.sh" "screen-$id" "$run" > /dev/null
mkdir -p "$DICTUS_HELPERS/../screens"
cp "$run/screen-$id.png" "$DICTUS_HELPERS/../screens/$id.png"
echo "screen $id: screens/$id.png ($(($(stat -c %s "$run/screen-$id.png") / 1024)) KB)"
