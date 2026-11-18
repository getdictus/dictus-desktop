#!/usr/bin/env bash
# Whole tier-1 run in one process: launch, doctor, one journey, cleanup. Cleanup runs on every exit
# (trap EXIT), failure included, and the evidence stays in the run dir.
#   run.sh <journey> [launch.sh options] [-- journey arguments]
#   e.g. run.sh onboarding --no-build         run.sh dictation --no-build -- all-fr
# A journey's `# launch-args:` header line is added to the launch options (e.g. --model, --mic).
# Last lines: the journey verdict, then CLEANUP OK run=<dir> ...
set -uo pipefail
source "$(dirname "$0")/env.sh"
journey="${1:?usage: run.sh <journey> [launch options] [-- journey args]}"; shift
script="$DICTUS_HELPERS/journey-$journey.sh"
[ -x "$script" ] || die "no helpers/journey-$journey.sh"
launch_args=()
while [ $# -gt 0 ] && [ "$1" != "--" ]; do launch_args+=("$1"); shift; done
[ "${1:-}" = "--" ] && shift
read -ra journey_launch <<< "$(sed -n 's/^# launch-args: //p' "$script")"
trap 'bash "$DICTUS_HELPERS/cleanup.sh"' EXIT
"$DICTUS_HELPERS/launch.sh" "${launch_args[@]}" "${journey_launch[@]}" "$journey" || exit 1
"$DICTUS_HELPERS/doctor.sh" || exit 1
"$script" "$@"
