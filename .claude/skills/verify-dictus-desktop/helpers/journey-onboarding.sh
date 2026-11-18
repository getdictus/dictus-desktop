#!/usr/bin/env bash
# Feature `onboarding` on tier 1, from an empty data dir: pick Moonshine V2 Tiny (31 MB), wait for
# download + SHA256 + extraction, land on General with that model selected. Needs a READY run.
#   journey-onboarding.sh [run-dir]
# Last line: JOURNEY onboarding PASS (exit 0) or JOURNEY onboarding FAIL: <step> (exit 1).
set -uo pipefail
source "$(dirname "$0")/env.sh"
run="$(resolve_run "${1:-}")" || die "no run of this session"
export DICTUS_VERIFY_RUN="$run" DISPLAY="$(ledger_get "$run" DISPLAY)"
H="$DICTUS_HELPERS"
data="$(run_data "$run")"
fail() { report "$run" "JOURNEY onboarding FAIL: $1"; echo "JOURNEY onboarding FAIL: $1"; exit 1; }
setting() { python3 -I -c 'import json,sys;print(json.load(open(sys.argv[1]))["settings"].get(sys.argv[2],""))' "$data/settings_store.json" "$1" 2>/dev/null; }

[ -z "$(ls "$data/models" 2>/dev/null)" ] || fail "precondition: models dir is not empty"
"$H/evidence.sh" 10-onboarding-picker > /dev/null
"$H/log.sh" wait 'Recording overlay window created|Signal handlers registered' 20 > /dev/null || fail "app log not ready"
m="$("$H/log.sh" mark)"
t0="$(date +%s)"
"$H/x.py" click 340 320 > /dev/null
"$H/log.sh" wait 'Starting fresh download of model moonshine-tiny-streaming-en' 10 "$m" || fail "click on Moonshine V2 Tiny started no download"
sleep 1
"$H/evidence.sh" 11-onboarding-downloading > /dev/null
"$H/log.sh" wait 'SHA256 verified for model moonshine-tiny-streaming-en' 180 "$m" || fail "download or SHA256 check"
"$H/log.sh" wait 'Successfully downloaded model moonshine-tiny-streaming-en' 60 "$m" || fail "extraction"
for _ in $(seq 30); do [ "$(setting selected_model)" = moonshine-tiny-streaming-en ] && break; sleep 1; done
[ "$(setting selected_model)" = moonshine-tiny-streaming-en ] || fail "selected_model not set"
sleep 2
"$H/evidence.sh" 12-onboarding-done > /dev/null
[ -s "$data/models/moonshine-tiny-streaming-en/encoder.ort" ] || fail "model files missing"
"$H/log.sh" since "$m" > "$run/12-onboarding-done.log"
report "$run" "JOURNEY onboarding PASS in $(($(date +%s) - t0))s"
echo "JOURNEY onboarding PASS in $(($(date +%s) - t0))s: compare 10/11/12-onboarding-*.png (picker, Extracting..., General page)"
