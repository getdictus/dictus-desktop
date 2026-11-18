#!/usr/bin/env bash
# Build the .deb under the shared lock, held for the build only (flock releases it on exit).
#   build.sh [--release] [log-file]       debug by default; --release for timings (bench.sh)
# Last line: DEB <path> (exit 0), or BUILD FAILED (exit 1). The updater signing step fails without
# TAURI_SIGNING_PRIVATE_KEY after the .deb is written: that exit code is ignored when a fresh .deb exists.
set -uo pipefail
source "$(dirname "$0")/env.sh"
profile=debug
[ "${1:-}" = --release ] && { profile=release; shift; }
log="${1:-/dev/stdout}"
start="$(date +%s)"
echo "waiting for $DICTUS_LOCK: $(lock_describe)" >&2
flock -w "${DICTUS_LOCK_WAIT_S:-1800}" "$DICTUS_LOCK" bash -c '
  cd "$1" || exit 1
  [ -d node_modules ] || bun install --frozen-lockfile || exit 1
  if [ "$2" = release ]; then bun run tauri build --bundles deb; else bun run tauri build --debug --bundles deb; fi
' _ "$REPO_ROOT" "$profile" > "$log" 2>&1
rc=$?
deb="$(find "$DICTUS_TARGET_DIR/$profile/bundle/deb" -maxdepth 1 -name '*.deb' -newermt "@$start" -printf '%T@ %p\n' 2>/dev/null | sort -n | tail -1 | cut -d' ' -f2-)"
echo "build exit $rc in $(($(date +%s) - start))s, lock released: $(lock_describe)" >&2
[ -n "$deb" ] || { echo "BUILD FAILED: no fresh .deb in $DICTUS_TARGET_DIR/$profile/bundle/deb"; exit 1; }
echo "DEB $deb"
