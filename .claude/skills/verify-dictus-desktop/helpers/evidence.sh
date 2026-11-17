#!/usr/bin/env bash
# One proof step: window screenshot + the isolated app state it should have changed.
#   evidence.sh <name> [run-dir]
# Writes <name>.png and <name>.state.txt (models on disk, selected model, new app log lines).
# Reads only the run's isolated HOME, never the real data dir.
set -euo pipefail
source "$(dirname "$0")/env.sh"
name="${1:?usage: evidence.sh <name> [run-dir]}"
run="$(resolve_run "${2:-}")" || die "no run of this session"
"$DICTUS_HELPERS/shot.sh" "$name" "$run"
if [ "$(ledger_get "$run" TIER)" = tier1 ]; then
  data="$(ledger_get "$run" SCRATCH)/home/.local/share/$DICTUS_IDENTIFIER"
  {
    echo "# $name $(ts)"
    echo "## app alive: $(ref_alive "$(ledger_get "$run" APP)" && echo yes || echo no)"
    echo "## models"
    (cd "$data/models" 2>/dev/null && find . -maxdepth 2 \( -type f -o -type d \) -printf '%y %s %p\n' | sort) || echo "(no models dir)"
    echo "## settings"
    python3 -I -c 'import json,sys
d=json.load(open(sys.argv[1])).get("settings",{})
for k in ("selected_model","app_language","post_process_enabled","audio_feedback","debug_mode"):
    print(k, "=", json.dumps(d.get(k)))' "$data/settings_store.json" 2>/dev/null || echo "(no settings_store.json yet)"
    echo "## log (last 15 lines)"
    tail -n 15 "$data/logs/"*.log 2>/dev/null || echo "(no log file)"
  } > "$run/$name.state.txt"
  echo "state $name: $run/$name.state.txt"
fi
