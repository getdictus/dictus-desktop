#!/usr/bin/env bash
# Screenshot of the Dictus window only (never the whole display) into the run dir.
#   shot.sh <name> [run-dir]        -> <run-dir>/<name>.png
# Tier 1: ffmpeg x11grab cropped to the window on the run's Xvfb. Tier 2: virsh screenshot of the VM.
set -euo pipefail
source "$(dirname "$0")/env.sh"
name="${1:?usage: shot.sh <name> [run-dir]}"
run="$(resolve_run "${2:-}")" || die "no run of this session"
out="$run/$name.png"
if [ "$(ledger_get "$run" TIER)" = tier2 ]; then
  "${VIRSH[@]}" screenshot "$DICTUS_VM" "$run/$name.ppm" > /dev/null
  ffmpeg -loglevel error -y -i "$run/$name.ppm" "$out" && rm -f "$run/$name.ppm"
else
  export DISPLAY="$(ledger_get "$run" DISPLAY)"
  finder=x.py; [ "$(ledger_get "$run" WM_KIND)" = gnome-wayland ] && finder=wm.py
  read -r _ geo < <(DICTUS_VERIFY_RUN="$run" "$DICTUS_HELPERS/$finder" window)
  size="${geo%%+*}"; off="${geo#*+}"
  ffmpeg -loglevel error -y -f x11grab -draw_mouse 0 -video_size "$size" -i "$DISPLAY+${off/+/,}" -frames:v 1 "$out"
fi
report "$run" "shot $name"
echo "shot $name: $out"
