#!/usr/bin/env bash
# Read-only: is this machine, and this session's run if any, worth driving? Starts, kills, writes nothing
# (except printing). One OK / WARN / FAIL line per check, then DOCTOR OK or DOCTOR FAIL <n> (exit 1).
#   doctor.sh [run-dir]
set -u
source "$(dirname "$0")/env.sh"
fails=0
ok() { echo "OK   $*"; }
warn() { echo "WARN $*"; }
fail() { echo "FAIL $*"; fails=$((fails + 1)); }

for t in bun cargo Xvfb dbus-daemon ffmpeg xwininfo dpkg-deb lslocks flock python3 virsh gh; do
  command -v "$t" > /dev/null && ok "tool $t" || { case "$t" in virsh|gh) warn "tool $t missing (tier 2 only)" ;; *) fail "tool $t missing" ;; esac; }
done
for t in pipewire pw-cli pw-link pw-dump; do
  command -v "$t" > /dev/null && ok "tool $t (virtual mic)" || warn "tool $t missing (journey-dictation.sh only)"
done
corpora="$(find /tmp -maxdepth 1 -name 'verify-dictus-corpus.*' -user "$(id -u)" 2>/dev/null | paste -sd' ')"
[ -z "$corpora" ] && ok "no test-voices corpus clone in /tmp" || warn "corpus clone(s) present: $corpora (cleanup.sh removes them once older than the TTL)"
[ -e /usr/lib/x86_64-linux-gnu/libXtst.so.6 ] && ok "libXtst.so.6 (x.py)" || fail "libXtst.so.6 missing (x.py cannot click)"

run="$(resolve_run "${1:-}" 2>/dev/null)" || run=""
[ -n "$run" ] && [ -n "$(ledger_get "$run" CLEANED)" ] && run=""
holder="$( [ -n "$run" ] && ledger_get "$run" LOCK_HOLDER)"
lock="$(lock_describe "$DICTUS_RUN_LOCK")"
if [ -n "$holder" ] && ref_alive "$holder"; then
  ok "run lock $DICTUS_RUN_LOCK held by this run: $lock"
elif [ -n "$run" ]; then
  fail "run lock $DICTUS_RUN_LOCK not held by this run: $lock"
elif [ "$lock" = free ]; then
  ok "run lock $DICTUS_RUN_LOCK free"
else
  warn "run lock $DICTUS_RUN_LOCK $lock (launch.sh and vm.sh start will wait for it)"
fi
build_lock="$(lock_describe "$DICTUS_LOCK")"
if [ "$build_lock" = free ]; then
  ok "build lock $DICTUS_LOCK free"
else
  warn "build lock $DICTUS_LOCK $build_lock (a build waits for it; the app phase does not)"
fi

used="$(ls /tmp/.X*-lock 2>/dev/null | sed 's|/tmp/.X\(.*\)-lock|:\1|' | paste -sd' ')"
next="$(free_display)" && ok "displays in use: ${used:-none}; next free for this skill :$next (never :99/:100)" || fail "no free display in :120-:199"

avail_gb="$(awk '/MemAvailable/ {printf "%.1f", $2/1048576}' /proc/meminfo)"
heavy="$(pgrep -c -x 'java|cargo|rustc|qemu-system-x86|gradle' 2>/dev/null; true)"
awk -v a="$avail_gb" 'BEGIN{exit !(a >= 3)}' && ok "RAM available ${avail_gb} GB; heavy processes (java/cargo/rustc/qemu) running: $heavy" \
  || warn "RAM available ${avail_gb} GB < 3 GB; heavy processes running: $heavy (Andy's Android builds share this machine)"

fingerprint | while read -r a b; do
  [ "$a" = missing ] && echo "OK   real data: $b absent" || echo "OK   real data: ${a:0:16} $b"
done
others=""
app_ref="$( [ -n "$run" ] && ledger_get "$run" APP)"
for p in $(pgrep -x dictus); do [ "$p" = "${app_ref%%:*}" ] || others="$others $p"; done
[ -z "$others" ] && ok "no Dictus running outside this run" \
  || warn "a Dictus not started by this run is running (pid$others): it may legitimately change the real-data fingerprint"

if command -v virsh > /dev/null; then
  st="$("${VIRSH[@]}" domstate "$DICTUS_VM" 2>/dev/null || echo absent)"
  snaps="$("${VIRSH[@]}" snapshot-list "$DICTUS_VM" --name 2>/dev/null | paste -sd' ')"
  vm_mine="$( [ -n "$run" ] && ledger_get "$run" VM_STARTED)"
  if [ "$st" = "shut off" ] || [ "$vm_mine" = 1 ]; then ok "VM $DICTUS_VM: $st; snapshots: $snaps"
  else warn "VM $DICTUS_VM: $st, not started by this run: do not drive it"; fi
  echo "$snaps" | grep -qw "$DICTUS_VM_CLEAN_SNAPSHOT" || warn "VM snapshot $DICTUS_VM_CLEAN_SNAPSHOT missing"
fi

if [ -n "$run" ]; then
  ok "run $run (tier $(ledger_get "$run" TIER), commit $(ledger_get "$run" COMMIT | cut -c1-9))"
  if [ "$(ledger_get "$run" TIER)" = tier1 ]; then
    ref_alive "$(ledger_get "$run" XVFB)" && ok "Xvfb $(ledger_get "$run" DISPLAY) is this run's" || fail "Xvfb of this run is gone"
    ref_alive "$app_ref" && ok "dictus pid ${app_ref%%:*} is this run's (deb sha $(ledger_get "$run" DEB_SHA))" || fail "dictus of this run is not running: see $run/app.log"
    finder=x.py; [ "$(ledger_get "$run" WM_KIND)" = gnome-wayland ] && finder=wm.py
    w="$(DISPLAY="$(ledger_get "$run" DISPLAY)" DICTUS_VERIFY_RUN="$run" "$DICTUS_HELPERS/$finder" window 2>/dev/null)" && ok "window $w" || fail "no mapped Dictus window"
    home="$(sed -n 's/^HOME=//p' "$run/app.env")"
    case "$home" in /tmp/verify-dictus-desktop.*/home) ok "app HOME is isolated: $home" ;; *) fail "app HOME is not isolated: $home" ;; esac
    if [ -n "$(ledger_get "$run" PIPEWIRE)" ]; then
      ref_alive "$(ledger_get "$run" PIPEWIRE)" && ok "private pipewire is this run's" || fail "private pipewire of this run is gone: see $run/pipewire.log"
      m="$(python3 -I "$DICTUS_HELPERS/mic.py" "$(ledger_get "$run" SCRATCH)" check 2>&1)" && ok "virtual mic: $m" || fail "virtual mic: $m"
    fi
  fi
  diff -q "$run/fingerprint-before.txt" <(fingerprint) > /dev/null && ok "real data unchanged since the run began" || fail "real data changed since the run began"
else
  ok "no active run of this session"
fi

[ "$fails" -eq 0 ] && echo "DOCTOR OK" || { echo "DOCTOR FAIL $fails"; exit 1; }
