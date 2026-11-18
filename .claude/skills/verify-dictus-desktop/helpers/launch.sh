#!/usr/bin/env bash
# Tier 1: build the debug .deb under the shared lock, extract it, start it on a private Xvfb with a
# private D-Bus and an empty HOME/XDG tree. Any failure runs cleanup.sh before exiting.
#   launch.sh [--no-build] [--deb <file.deb>] [label]
# Last line on success: READY run=<dir> display=:<n> pid=<pid> window=<id> geometry=<WxH+X+Y>
set -euo pipefail
source "$(dirname "$0")/env.sh"

build=1 deb="" label="launch"
while [ $# -gt 0 ]; do
  case "$1" in
    --no-build) build=0 ;;
    --deb) deb="$2"; build=0; shift ;;
    *) label="$1" ;;
  esac
  shift
done

run="$(run_begin tier1 "$label" | tail -1)"
trap 'rc=$?; if [ $rc -ne 0 ]; then echo "LAUNCH FAILED (exit $rc): cleaning up" >&2; report "$run" "LAUNCH FAILED exit=$rc"; bash "$DICTUS_HELPERS/cleanup.sh" "$run" >&2 || true; fi' EXIT

if [ "$build" = 1 ]; then
  echo "building under $DICTUS_LOCK, released when the build ends (log: $run/build.log)"
  out="$("$DICTUS_HELPERS/build.sh" "$run/build.log" | tail -1)"
  report "$run" "BUILD $out"
  [ "${out%% *}" = DEB ] || die "$out: see $run/build.log"
  deb="${out#DEB }"
elif [ -z "$deb" ]; then
  deb="$(find "$DICTUS_TARGET_DIR/debug/bundle/deb" -maxdepth 1 -name '*.deb' -printf '%T@ %p\n' 2>/dev/null | sort -n | tail -1 | cut -d' ' -f2-)"
  [ -n "$deb" ] || die "no .deb in $DICTUS_TARGET_DIR/debug/bundle/deb: run build.sh first"
fi
ledger_set "$run" DEB "$deb"
ledger_set "$run" DEB_SHA "$(sha256sum "$deb" | cut -c1-16)"

run_hold "$run"
scratch="$(mktemp -d /tmp/verify-dictus-desktop.XXXXXX)"
ledger_set "$run" SCRATCH "$scratch"
dpkg-deb -x "$deb" "$scratch/pkg"
bin="$scratch/pkg/usr/bin/dictus"
[ -x "$bin" ] || die "no usr/bin/dictus in $deb"

home="$scratch/home"
mkdir -p "$home"/{.config,.cache,.local/share,.local/state} "$scratch/runtime"
chmod 700 "$scratch/runtime"

n="$(free_display)" || die "no free display in :120-:199"
setsid Xvfb ":$n" -screen 0 1280x900x24 -nolisten tcp < /dev/null > "$run/xvfb.log" 2>&1 &
ledger_set "$run" XVFB "$(pid_ref $!)"
ledger_set "$run" DISPLAY ":$n"
for _ in $(seq 50); do [ -S "/tmp/.X11-unix/X$n" ] && break; sleep 0.2; done
[ -S "/tmp/.X11-unix/X$n" ] || die "Xvfb :$n did not start: see $run/xvfb.log"

dbus_out="$(env -i PATH="$PATH" HOME="$home" XDG_RUNTIME_DIR="$scratch/runtime" \
  dbus-daemon --session --fork --nopidfile --print-address=1 --print-pid=1)"
dbus_addr="$(sed -n 1p <<< "$dbus_out")"
ledger_set "$run" DBUS "$(pid_ref "$(sed -n 2p <<< "$dbus_out")")"

app_env=(env -i PATH="/usr/local/bin:/usr/bin:/bin" HOME="$home" USER="$USER" LANG=en_US.UTF-8
  XDG_CONFIG_HOME="$home/.config" XDG_CACHE_HOME="$home/.cache" XDG_DATA_HOME="$home/.local/share"
  XDG_STATE_HOME="$home/.local/state" XDG_RUNTIME_DIR="$scratch/runtime" XDG_SESSION_TYPE=x11
  DISPLAY=":$n" GDK_BACKEND=x11 DBUS_SESSION_BUS_ADDRESS="$dbus_addr"
  WEBKIT_DISABLE_DMABUF_RENDERER=1)
printf '%s\n' "${app_env[@]:1}" | grep -v '^DBUS' > "$run/app.env"
launched="$(date +%s)"
setsid "${app_env[@]}" "$bin" < /dev/null > "$run/app.log" 2>&1 &
app=$!
ledger_set "$run" APP "$(pid_ref "$app")"
report "$run" "LAUNCHED pid=$app display=:$n deb_sha=$(ledger_get "$run" DEB_SHA)"

win=""
for _ in $(seq 90); do
  ref_alive "$(ledger_get "$run" APP)" || die "dictus exited during startup: see $run/app.log"
  win="$(DISPLAY=":$n" "$DICTUS_HELPERS/x.py" window 2>/dev/null | awk '{print $1}')" && [ -n "$win" ] && break
  sleep 1
done
[ -n "$win" ] || die "no Dictus window after 90 s: see $run/app.log"
sleep 3
ref_alive "$(ledger_get "$run" APP)" || die "dictus exited right after its window appeared: see $run/app.log"
report "$run" "WINDOW after $(($(date +%s) - launched))s"
geo="$(DISPLAY=":$n" "$DICTUS_HELPERS/x.py" window | awk '{print $2}')"
DICTUS_VERIFY_RUN="$run" "$DICTUS_HELPERS/shot.sh" 00-launch > /dev/null
echo "READY run=$run display=:$n pid=$app window=$win geometry=$geo"
report "$run" "READY display=:$n pid=$app window=$win geometry=$geo"
