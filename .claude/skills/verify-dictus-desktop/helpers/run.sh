#!/usr/bin/env bash
# Whole tier-1 run in one process: launch, doctor, one journey, cleanup. Cleanup runs on every exit
# (trap EXIT), failure included, and the evidence stays in the run dir.
#   run.sh <journey> [launch.sh options]      e.g. run.sh onboarding --no-build
# Last lines: the journey verdict, then CLEANUP OK run=<dir> ...
set -uo pipefail
source "$(dirname "$0")/env.sh"
journey="${1:?usage: run.sh <journey> [launch options]}"; shift
[ -x "$DICTUS_HELPERS/journey-$journey.sh" ] || die "no helpers/journey-$journey.sh"
trap 'bash "$DICTUS_HELPERS/cleanup.sh"' EXIT
"$DICTUS_HELPERS/launch.sh" "$@" "$journey" || exit 1
"$DICTUS_HELPERS/doctor.sh" || exit 1
"$DICTUS_HELPERS/journey-$journey.sh"
