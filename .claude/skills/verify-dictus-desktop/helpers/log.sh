#!/usr/bin/env bash
# The app's own log in the run's isolated data dir (tier 1). Timestamps are UTC.
#   log.sh mark                      print the current line count (assert only on what follows)
#   log.sh wait '<regex>' [secs] [mark]   print the first matching line after the mark; exit 1 on timeout
#   log.sh since <mark>              every line after the mark
set -uo pipefail
source "$(dirname "$0")/env.sh"
run="$(resolve_run "${DICTUS_VERIFY_RUN:-}")" || die "no run of this session"
logs() { cat "$(run_data "$run")/logs/"*.log 2>/dev/null; }
case "${1:?mark|wait|since}" in
  mark) logs | wc -l ;;
  since) logs | tail -n +"$((${2:-0} + 1))" ;;
  wait)
    re="$2"; secs="${3:-30}"; mark="${4:-0}"
    for ((i = 0; i < secs * 2; i++)); do
      line="$(logs | tail -n +"$((mark + 1))" | grep -E -m1 -- "$re" || true)"
      [ -n "$line" ] && { echo "found: $line"; exit 0; }
      sleep 0.5
    done
    echo "error: no log line matching /$re/ within ${secs}s"; exit 1 ;;
  *) die "usage: log.sh mark|wait|since" ;;
esac
