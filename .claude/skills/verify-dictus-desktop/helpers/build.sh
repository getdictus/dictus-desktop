#!/usr/bin/env bash
# Build the .deb. Each compile command (bun install, bun run tauri build) takes the shared build lock with
# its own `flock`, which releases it when that command ends. Nothing else in this skill takes that lock.
#   build.sh [--release] [log-file]       debug by default; --release for timings (bench.sh)
# Last line: DEB <path> (exit 0), or BUILD FAILED (exit 1). The updater signing step fails without
# TAURI_SIGNING_PRIVATE_KEY after the .deb is written: that exit code is ignored when a fresh .deb exists.
set -uo pipefail
source "$(dirname "$0")/env.sh"
profile=debug
[ "${1:-}" = --release ] && { profile=release; shift; }
log="${1:-/dev/stdout}"
start="$(date +%s)"
exec 3>&2
compile() {
  echo "$*: waiting for $DICTUS_LOCK: $(lock_describe "$DICTUS_LOCK")" >&3
  (cd "$REPO_ROOT" && flock -w "${DICTUS_LOCK_WAIT_S:-1800}" "$DICTUS_LOCK" "$@")
}
{
  rc=0
  [ -d "$REPO_ROOT/node_modules" ] || compile bun install --frozen-lockfile || rc=$?
  if [ "$rc" = 0 ]; then
    if [ "$profile" = release ]; then compile bun run tauri build --bundles deb; else compile bun run tauri build --debug --bundles deb; fi
    rc=$?
  fi
} > "$log" 2>&1
deb="$(find "$DICTUS_TARGET_DIR/$profile/bundle/deb" -maxdepth 1 -name '*.deb' -newermt "@$start" -printf '%T@ %p\n' 2>/dev/null | sort -n | tail -1 | cut -d' ' -f2-)"
echo "build exit $rc in $(($(date +%s) - start))s, build lock released: $(lock_describe "$DICTUS_LOCK")" >&2
[ -n "$deb" ] || { echo "BUILD FAILED: no fresh .deb in $DICTUS_TARGET_DIR/$profile/bundle/deb"; exit 1; }
echo "DEB $deb"
