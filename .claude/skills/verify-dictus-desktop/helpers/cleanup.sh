#!/usr/bin/env bash
# Undo exactly what the run ledger (<run-dir>/run.env) lists, keep the evidence. Safe to run twice.
#   cleanup.sh [run-dir]
# Order: app, private PipeWire, D-Bus, Xvfb, HTTP server, VM (destroy + revert to the clean snapshot),
# test-voices corpus clone, scratch dir (isolated HOME/XDG, extracted .deb, models, history, recordings,
# installers), watchdog, lock. Also deletes any corpus clone left by a run that died without cleanup.
# Kills only the process groups this run started, checked by pid AND start time. Never by name.
# Prints CLEANUP OK run=<dir> evidence=<n files> (exit 0), CLEANUP INCOMPLETE: <what is left> (exit 1),
# or CLEANUP OK then ALARM when the real-data fingerprint changed (exit 2).
set -u
source "$(dirname "$0")/env.sh"
run="$(resolve_run "${1:-}")" || { echo "no run of this session: nothing to clean (the lock is released only by the run that holds it)"; exit 0; }
left=()
alarm=0

# Kill a process group we started: the leader if it is still ours, then any member started after it.
kill_group() {
  local key="$1" ref pid st m
  ref="$(ledger_get "$run" "$key")"; [ -n "$ref" ] || return 0
  pid="${ref%%:*}"; st="${ref##*:}"
  members() { for m in $(pgrep -g "$pid" 2>/dev/null); do [ "$(proc_start "$m")" -ge "$st" ] 2>/dev/null && echo "$m"; done; }
  local ms; ms="$(members)"
  [ -z "$ms" ] && return 0
  kill -TERM $ms 2>/dev/null
  for _ in $(seq 20); do ms="$(members)"; [ -z "$ms" ] && break; sleep 0.25; done
  [ -n "$ms" ] && { kill -KILL $ms 2>/dev/null; sleep 0.5; ms="$(members)"; }
  if [ -n "$ms" ]; then left+=("$key pids $ms"); else echo "stopped $key (group $pid)"; fi
}

kill_group APP
kill_group AUTOLINK
kill_group PIPEWIRE
dbus="$(ledger_get "$run" DBUS)"
if [ -n "$dbus" ] && ref_alive "$dbus"; then
  kill -TERM "${dbus%%:*}"; sleep 0.5
  ref_alive "$dbus" && kill -KILL "${dbus%%:*}"
  ref_alive "$dbus" && left+=("DBUS ${dbus%%:*}") || echo "stopped DBUS"
fi
kill_group XVFB
kill_group HTTP

if [ "$(ledger_get "$run" VM_STARTED)" = 1 ] && [ -z "$(ledger_get "$run" VM_REVERTED)" ]; then
  [ "$DICTUS_VM" = dictus-win ] || die "refusing to touch VM '$DICTUS_VM': this skill drives dictus-win only"
  if "${VIRSH[@]}" domstate "$DICTUS_VM" 2>/dev/null | grep -qv 'shut off'; then
    "${VIRSH[@]}" destroy "$DICTUS_VM" > /dev/null && echo "destroyed $DICTUS_VM"
  fi
  if "${VIRSH[@]}" snapshot-revert "$DICTUS_VM" "$DICTUS_VM_CLEAN_SNAPSHOT" > /dev/null; then
    ledger_set "$run" VM_REVERTED 1
    echo "reverted $DICTUS_VM to $DICTUS_VM_CLEAN_SNAPSHOT"
  else
    left+=("VM $DICTUS_VM not reverted")
  fi
  "${VIRSH[@]}" domstate "$DICTUS_VM" 2>/dev/null | grep -q 'shut off' || left+=("VM $DICTUS_VM still running")
fi

# The corpus is private (internal testing only): no clone may outlive its run.
remove_corpus() {
  case "$1" in
    /tmp/verify-dictus-corpus.*) rm -rf "$1" && echo "removed corpus $1" ;;
    *) left+=("corpus $1 not under /tmp/verify-dictus-corpus.*, not removed") ;;
  esac
  [ -e "$1" ] && left+=("corpus $1 still present")
}
corpus="$(ledger_get "$run" CORPUS)"
[ -n "$corpus" ] && [ -d "$corpus" ] && remove_corpus "$corpus"
# Another session's live run keeps its clone: only those older than any run can live (the watchdog TTL) go.
for stale in $(find /tmp -maxdepth 1 -name 'verify-dictus-corpus.*' -user "$(id -u)" -mmin +"$((DICTUS_TTL_MIN + 5))" 2>/dev/null); do
  remove_corpus "$stale"
done

scratch="$(ledger_get "$run" SCRATCH)"
if [ -n "$scratch" ] && [ -d "$scratch" ]; then
  case "$scratch" in
    /tmp/verify-dictus-desktop.*) rm -rf "$scratch" && echo "removed $scratch" ;;
    *) left+=("scratch $scratch not under /tmp/verify-dictus-desktop.*, not removed") ;;
  esac
fi
disp="$(ledger_get "$run" DISPLAY)"
[ -n "$disp" ] && [ -e "/tmp/.X11-unix/X${disp#:}" ] && left+=("display $disp socket")

fingerprint > "$run/fingerprint-after.txt"
if diff -q "$run/fingerprint-before.txt" "$run/fingerprint-after.txt" > /dev/null 2>&1; then
  echo "FINGERPRINT same"; report "$run" "FINGERPRINT same"
else
  echo "FINGERPRINT DIFFERS: real data changed during the run (see fingerprint-before/after.txt)"
  report "$run" "FINGERPRINT DIFFERS"; alarm=1
fi

[ "${DICTUS_FROM_WATCHDOG:-}" = 1 ] && report "$run" "cleanup started by the watchdog after ${DICTUS_TTL_MIN} min" || kill_group WATCHDOG
kill_group LOCK_HOLDER
lock_now="$(lock_describe)"
echo "lock: $lock_now"

if [ ${#left[@]} -eq 0 ]; then
  ledger_set "$run" CLEANED "$(date +%s)"
  report "$run" "CLEANUP OK"
  echo "CLEANUP OK run=$run evidence=$(find "$run" -maxdepth 1 -type f | wc -l)"
  [ "$alarm" = 0 ] || { echo "ALARM: real data changed. Stop and report to Pierre before any other run."; exit 2; }
else
  report "$run" "CLEANUP INCOMPLETE: ${left[*]}"
  echo "CLEANUP INCOMPLETE: ${left[*]}"
  exit 1
fi
