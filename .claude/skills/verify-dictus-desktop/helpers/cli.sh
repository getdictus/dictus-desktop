#!/usr/bin/env bash
# Run the run's dictus binary a second time, in the app's exact isolated environment, with CLI flags.
# The second instance hands its arguments to the running one (single instance over the private D-Bus)
# and exits: this is the `dictation.cli` entry point (`--toggle-transcription`, `--cancel`, ...).
#   cli.sh <dictus flags...>
# Prints CLI <flags> exit=<n> in <ms>ms.
set -uo pipefail
source "$(dirname "$0")/env.sh"
run="$(resolve_run "${DICTUS_VERIFY_RUN:-}")" || die "no run of this session"
scratch="$(ledger_get "$run" SCRATCH)"
[ -s "$scratch/app.env" ] || die "no $scratch/app.env: launch.sh did not finish"
ref_alive "$(ledger_get "$run" APP)" || die "the run's dictus is not running"
mapfile -t app_env < "$scratch/app.env"
t0="$(date +%s%N)"
timeout 30 env -i "${app_env[@]}" "$scratch/pkg/usr/bin/dictus" "$@" < /dev/null >> "$run/cli.log" 2>&1
rc=$?
echo "CLI $* exit=$rc in $((($(date +%s%N) - t0) / 1000000))ms"
exit "$rc"
