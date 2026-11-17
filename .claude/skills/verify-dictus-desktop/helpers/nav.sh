#!/usr/bin/env bash
# Open a sidebar section of the main window (tier 1). Coordinates are window-relative, measured on
# the 680x570 window of v0.3.0 with English labels; the only place they are written.
#   nav.sh general|models|advanced|history|transcribe-file|post-process|debug|about [run-dir]
# Post Process appears between Transcribe File and About only when post-processing is enabled,
# and Debug only in debug mode (Ctrl+Shift+D): each takes the row after the one before it and shifts
# About down by 43 px (set DICTUS_ABOUT_Y=362 with one of them, 405 with both; Debug is 362 with both).
set -euo pipefail
source "$(dirname "$0")/env.sh"
run="$(resolve_run "${2:-}")" || die "no run of this session"
export DISPLAY="$(ledger_get "$run" DISPLAY)"
case "${1:?section}" in
  general) y=105 ;; models) y=148 ;; advanced) y=191 ;; history) y=233 ;;
  transcribe-file) y=276 ;; post-process|debug) y=319 ;; about) y=${DICTUS_ABOUT_Y:-319} ;;
  *) die "unknown section $1" ;;
esac
"$DICTUS_HELPERS/x.py" click 70 "$y" > /dev/null
sleep 1.5
report "$run" "nav $1"
echo "nav $1"
