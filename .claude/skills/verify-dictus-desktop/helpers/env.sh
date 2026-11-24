#!/usr/bin/env bash
# Sourced by every helper: paths, the run ledger, the two locks, the real-data fingerprint.
# Override any DICTUS_* variable from the environment before calling a helper.
export PATH="$HOME/.bun/bin:$HOME/.cargo/bin:$PATH"

REPO_ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
DICTUS_HELPERS="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# The machine's shared build lock (Andy's Gradle and npm builds take it too): build.sh holds it around
# each compile command only. The run lock is this skill's own; Gradle never takes it.
DICTUS_LOCK="${DICTUS_LOCK:-/tmp/dictus-gradle.lock}"
DICTUS_RUN_LOCK="${DICTUS_RUN_LOCK:-/tmp/dictus-desktop-run.lock}"
DICTUS_EVIDENCE_ROOT="${DICTUS_EVIDENCE_ROOT:-/home/pierre/OpenBot/Shared/pilotage/dictus-desktop/preuves/verify-dictus-desktop}"
DICTUS_IDENTIFIER="${DICTUS_IDENTIFIER:-com.dictus.desktop}"
DICTUS_REAL_DATA="${DICTUS_REAL_DATA:-$HOME/.local/share/$DICTUS_IDENTIFIER}"
DICTUS_TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/src-tauri/target}"
DICTUS_TTL_MIN="${DICTUS_TTL_MIN:-20}"
DICTUS_VM="${DICTUS_VM:-dictus-win}"
DICTUS_VM_CLEAN_SNAPSHOT="${DICTUS_VM_CLEAN_SNAPSHOT:-clean-no-vcredist}"
DICTUS_HTTP_HOST="192.168.122.1"
DICTUS_MODEL_CACHE="${DICTUS_MODEL_CACHE:-$HOME/.cache/verify-dictus-desktop/models}"
# The private test-voices corpus is cloned per run into /tmp/verify-dictus-corpus.XXXXXX (mode 700).
DICTUS_CORPUS_REPO="${DICTUS_CORPUS_REPO:-getdictus/dictus-test-voices}"
VIRSH=(virsh -c qemu:///system)

# A session is this checkout, or DICTUS_VERIFY_SESSION when two agents share one checkout.
DICTUS_SESSION="${DICTUS_VERIFY_SESSION:-$(basename "$REPO_ROOT")-$(printf '%s' "$REPO_ROOT" | sha256sum | cut -c1-12)}"
DICTUS_SESSION_PTR="$DICTUS_EVIDENCE_ROOT/sessions/$DICTUS_SESSION.run"

ts() { date '+%Y-%m-%d %H:%M:%S'; }
# The identifier of the run's build (launch.sh reads it from the .deb), else DICTUS_IDENTIFIER.
run_identifier() { local i; i="$(ledger_get "$1" IDENTIFIER)"; echo "${i:-$DICTUS_IDENTIFIER}"; }
run_data() { echo "$(ledger_get "$1" SCRATCH)/home/.local/share/$(run_identifier "$1")"; }
die() { echo "error: $*" >&2; exit 1; }

# The run ledger is <run-dir>/run.env: one KEY=VALUE per line, appended as resources start.
# Cleanup undoes exactly what it lists, nothing else.
ledger_set() { echo "$2=$3" >> "$1/run.env"; }
ledger_get() { sed -n "s/^$2=//p" "$1/run.env" 2>/dev/null | tail -1; }
report() { echo "$(ts) $2" >> "$1/report.txt"; }

# The run of this session, or the one given. Refuses a run that is not this session's.
resolve_run() {
  local run="${1:-${DICTUS_VERIFY_RUN:-}}"
  if [ -z "$run" ]; then
    [ -s "$DICTUS_SESSION_PTR" ] || return 1
    run="$(cat "$DICTUS_SESSION_PTR")"
  fi
  [ -f "$run/run.env" ] || return 1
  [ "$(ledger_get "$run" SESSION)" = "$DICTUS_SESSION" ] || { echo "error: $run belongs to another session" >&2; return 1; }
  echo "$run"
}

# A pid is "ours" only if it still has the start time recorded when we started it (no pid reuse).
proc_start() { awk '{print $22}' "/proc/$1/stat" 2>/dev/null; }
pid_ref() { echo "$1:$(proc_start "$1")"; }
ref_alive() { local pid="${1%%:*}" st="${1##*:}"; [ -n "$pid" ] && [ "$(proc_start "$pid")" = "$st" ]; }

# Run lock holder: `flock -o DICTUS_RUN_LOCK sh -c 'touch ready; exec sleep infinity'` in its own process
# group. Holding the run lock = that group is alive; releasing = killing that group. Never touches other
# holders. The build lock is never held this way: build.sh takes it with a plain `flock <lock> <command>`.
lock_holders() { lslocks -n -o PID,COMMAND,PATH 2>/dev/null | awk -v p="$1" '$3 == p {print $1, $2}'; }
lock_describe() {
  local h pid
  h="$(lock_holders "$1")"
  if [ -z "$h" ]; then echo "free"; return; fi
  while read -r pid comm; do
    echo "held by pid=$pid comm=$comm age=$(ps -o etimes= -p "$pid" 2>/dev/null | tr -d ' ')s cwd=$(readlink "/proc/$pid/cwd" 2>/dev/null || echo '?')"
  done <<< "$h"
}
lock_acquire() {
  local run="$1" wait_s="${2:-1800}" ready="$1/.lock-ready" pid i
  rm -f "$ready"
  setsid flock -o "$DICTUS_RUN_LOCK" sh -c 'touch "$1"; exec sleep infinity' _ "$ready" < /dev/null > /dev/null 2>&1 &
  pid=$!
  ledger_set "$run" LOCK_HOLDER "$(pid_ref "$pid")"
  for ((i = 0; i < wait_s; i++)); do
    [ -e "$ready" ] && { report "$run" "RUN LOCK ACQUIRED $DICTUS_RUN_LOCK holder=$pid"; echo "RUN LOCK ACQUIRED holder=$pid"; return 0; }
    [ $((i % 60)) -eq 5 ] && echo "waiting for $DICTUS_RUN_LOCK: $(lock_describe "$DICTUS_RUN_LOCK")"
    sleep 1
  done
  kill -- "-$pid" 2>/dev/null
  echo "error: run lock not acquired after ${wait_s}s: $(lock_describe "$DICTUS_RUN_LOCK")" >&2
  return 1
}

# sha256 of the real data files a stray run could change. Missing files are listed as missing.
fingerprint() {
  local f dir
  for dir in "$DICTUS_REAL_DATA" "$HOME/.local/share/com.dictus.desktop.dev"; do
    for f in history.db settings_store.json; do
      if [ -f "$dir/$f" ]; then
        echo "$(sha256sum "$dir/$f" | cut -c1-64) $dir/$f"
      else
        echo "missing $dir/$f"
      fi
    done
  done
}

# Display numbers this skill may use. :99 and :100 belong to OpenBot.
free_display() {
  local n
  for n in $(seq 120 199); do
    [ -e "/tmp/.X$n-lock" ] || [ -e "/tmp/.X11-unix/X$n" ] || { echo "$n"; return 0; }
  done
  return 1
}

# Start a run: run dir + ledger, session pointer, real-data fingerprint. Takes no lock.
# Prints the run dir on stdout's last line.
run_begin() {
  local tier="$1" label="$2" prev run sha
  if prev="$(resolve_run 2>/dev/null)" && [ -z "$(ledger_get "$prev" CLEANED)" ]; then
    die "run $prev of this session is not cleaned up: run cleanup.sh first"
  fi
  sha="$(git -C "$REPO_ROOT" rev-parse --short HEAD)"
  run="$DICTUS_EVIDENCE_ROOT/$(date +%Y%m%d-%H%M%S)-$tier-$label-$sha"
  mkdir -p "$run" "$(dirname "$DICTUS_SESSION_PTR")"
  : > "$run/run.env"
  ledger_set "$run" SESSION "$DICTUS_SESSION"
  ledger_set "$run" TIER "$tier"
  ledger_set "$run" COMMIT "$(git -C "$REPO_ROOT" rev-parse HEAD)"
  ledger_set "$run" BRANCH "$(git -C "$REPO_ROOT" rev-parse --abbrev-ref HEAD)"
  echo "$run" > "$DICTUS_SESSION_PTR"
  report "$run" "BEGIN tier=$tier label=$label commit=$sha session=$DICTUS_SESSION"
  fingerprint > "$run/fingerprint-before.txt"
  echo "$run"
}

# Take the run lock for the part of the run where the app (or the VM) runs, and arm the watchdog:
# after DICTUS_TTL_MIN minutes it runs cleanup.sh, so an idle or dead agent never keeps the run lock.
# Tier 1 and tier 2 share it: the VM's 4 GB next to an app run does not fit in this machine's RAM.
run_hold() {
  local run="$1" wd
  lock_acquire "$run" "${DICTUS_LOCK_WAIT_S:-1800}" >&2 || return 1
  setsid sh -c 'sleep "$1"; DICTUS_FROM_WATCHDOG=1 exec bash "$2" "$3"' _ "$((DICTUS_TTL_MIN * 60))" \
    "$DICTUS_HELPERS/cleanup.sh" "$run" < /dev/null >> "$run/watchdog.log" 2>&1 &
  wd=$!
  ledger_set "$run" WATCHDOG "$(pid_ref "$wd")"
  sleep 0.3
  ref_alive "$(ledger_get "$run" WATCHDOG)" || { echo "error: watchdog did not start" >&2; return 1; }
  report "$run" "WATCHDOG armed: cleanup at $(date -d "+$DICTUS_TTL_MIN min" '+%H:%M:%S')"
}
