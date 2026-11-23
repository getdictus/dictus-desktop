#!/usr/bin/env bash
# Tier 1: build the debug .deb under the shared lock, extract it, start it on a private Xvfb with a
# private D-Bus and an empty HOME/XDG tree. Any failure runs cleanup.sh before exiting.
#   launch.sh [--no-build] [--release] [--deb <file.deb>] [--model <id>] [--mic] [label]
#   --release     build (or with --no-build, reuse) the release .deb instead of the debug one
#   --model <id>  put that catalogue model (from model-cache.sh) in the isolated data dir before the app starts
#   --mic         private PipeWire with a virtual source in the scratch dir (mic.py), started before the app
#   DICTUS_APP_LANG=fr_FR.UTF-8 starts the app in another locale (default en_US.UTF-8)
# Last line on success: READY run=<dir> display=:<n> pid=<pid> window=<id> geometry=<WxH+X+Y>
set -euo pipefail
source "$(dirname "$0")/env.sh"

build=1 deb="" label="launch" model="" mic=0 profile=debug
while [ $# -gt 0 ]; do
  case "$1" in
    --no-build) build=0 ;;
    --release) profile=release ;;
    --deb) deb="$2"; build=0; shift ;;
    --model) model="$2"; shift ;;
    --mic) mic=1 ;;
    *) label="$1" ;;
  esac
  shift
done

run="$(run_begin tier1 "$label" | tail -1)"
trap 'rc=$?; if [ $rc -ne 0 ]; then echo "LAUNCH FAILED (exit $rc): cleaning up" >&2; report "$run" "LAUNCH FAILED exit=$rc"; bash "$DICTUS_HELPERS/cleanup.sh" "$run" >&2 || true; fi' EXIT

if [ "$build" = 1 ]; then
  echo "building under $DICTUS_LOCK, released when the build ends (log: $run/build.log)"
  build_args=("$run/build.log")
  [ "$profile" = release ] && build_args=(--release "$run/build.log")
  out="$("$DICTUS_HELPERS/build.sh" "${build_args[@]}" | tail -1)"
  report "$run" "BUILD $out"
  [ "${out%% *}" = DEB ] || die "$out: see $run/build.log"
  deb="${out#DEB }"
elif [ -z "$deb" ]; then
  deb="$(find "$DICTUS_TARGET_DIR/$profile/bundle/deb" -maxdepth 1 -name '*.deb' -printf '%T@ %p\n' 2>/dev/null | sort -n | tail -1 | cut -d' ' -f2-)"
  [ -n "$deb" ] || die "no .deb in $DICTUS_TARGET_DIR/$profile/bundle/deb: run build.sh first"
fi
ledger_set "$run" DEB "$deb"
ledger_set "$run" DEB_SHA "$(sha256sum "$deb" | cut -c1-16)"
# Debug builds since #75 are "Dictus Dev" (package dictus-dev, com.dictus.desktop.dev); release is dictus.
case "$(dpkg-deb -f "$deb" Package)" in
  dictus-dev) ledger_set "$run" IDENTIFIER com.dictus.desktop.dev ;;
  dictus) ledger_set "$run" IDENTIFIER com.dictus.desktop ;;
  *) die "unknown package $(dpkg-deb -f "$deb" Package) in $deb" ;;
esac
if [ -n "$model" ]; then
  read -r _ _ archive model_dir < <("$DICTUS_HELPERS/model-cache.sh" "$model" | tail -1)
  [ -n "${model_dir:-}" ] || die "model-cache.sh $model failed"
fi

run_hold "$run"
scratch="$(mktemp -d /tmp/verify-dictus-desktop.XXXXXX)"
ledger_set "$run" SCRATCH "$scratch"
dpkg-deb -x "$deb" "$scratch/pkg"
bin="$scratch/pkg/usr/bin/dictus"
[ -x "$bin" ] || die "no usr/bin/dictus in $deb"

home="$scratch/home"
mkdir -p "$home"/{.config,.cache,.local/share,.local/state} "$scratch/runtime"
chmod 700 "$scratch/runtime"
data="$(run_data "$run")"

if [ -n "$model" ] && [ "${archive%.tar.gz}" = "$archive" ]; then
  mkdir -p "$data/models"
  cp "$archive" "$data/models/$model_dir"
  ledger_set "$run" MODEL "$model"
  report "$run" "MODEL $model seeded from $(basename "$archive") (sha256 verified)"
elif [ -n "$model" ]; then
  # Same layout as the app's own extraction (model.rs): a single top directory becomes models/<filename>.
  mkdir -p "$data/models/$model_dir.seeding"
  tar --warning=no-unknown-keyword -xzf "$archive" -C "$data/models/$model_dir.seeding"
  top="$(find "$data/models/$model_dir.seeding" -mindepth 1 -maxdepth 1 -type d)"
  if [ "$(wc -l <<< "$top")" = 1 ] && [ -d "$top" ]; then
    mv "$top" "$data/models/$model_dir" && rm -rf "$data/models/$model_dir.seeding"
  else
    mv "$data/models/$model_dir.seeding" "$data/models/$model_dir"
  fi
  ledger_set "$run" MODEL "$model"
  report "$run" "MODEL $model seeded from $(basename "$archive") (sha256 verified)"
fi

if [ "$mic" = 1 ]; then
  mkfifo -m 600 "$scratch/mic.fifo"
  python3 -I "$DICTUS_HELPERS/mic.py" "$scratch" conf > "$scratch/pipewire.conf"
  setsid env -i PATH=/usr/bin:/bin HOME="$home" XDG_RUNTIME_DIR="$scratch/runtime" \
    pipewire -c "$scratch/pipewire.conf" < /dev/null > "$run/pipewire.log" 2>&1 &
  ledger_set "$run" PIPEWIRE "$(pid_ref $!)"
  for _ in $(seq 50); do [ -S "$scratch/runtime/pipewire-0" ] && break; sleep 0.1; done
  [ -S "$scratch/runtime/pipewire-0" ] || die "private pipewire did not start: see $run/pipewire.log"
  out="$(python3 -I "$DICTUS_HELPERS/mic.py" "$scratch" check)" || die "virtual mic check failed"
  ledger_set "$run" MIC 1
  report "$run" "$out"
fi

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

app_env=(env -i PATH="/usr/local/bin:/usr/bin:/bin" HOME="$home" USER="$USER" LANG="${DICTUS_APP_LANG:-en_US.UTF-8}"
  XDG_CONFIG_HOME="$home/.config" XDG_CACHE_HOME="$home/.cache" XDG_DATA_HOME="$home/.local/share"
  XDG_STATE_HOME="$home/.local/state" XDG_RUNTIME_DIR="$scratch/runtime" XDG_SESSION_TYPE=x11
  DISPLAY=":$n" GDK_BACKEND=x11 DBUS_SESSION_BUS_ADDRESS="$dbus_addr"
  WEBKIT_DISABLE_DMABUF_RENDERER=1
  RUST_LOG=info,dictus_app_lib::managers::transcription=warn)
printf '%s\n' "${app_env[@]:1}" | grep -v '^DBUS' > "$run/app.env"
printf '%s\n' "${app_env[@]:1}" > "$scratch/app.env"
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
