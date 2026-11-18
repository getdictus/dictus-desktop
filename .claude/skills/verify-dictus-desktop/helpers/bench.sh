#!/usr/bin/env bash
# Dictation benchmark on tier 1: for each model, a fresh launch with only that model on disk (the app
# auto-selects it), then journey-dictation.sh --reps N, then cleanup.sh, which releases the lock before
# the next model. Collects WER, stop → text, transcription time, the cold model load and the first
# dictation after launch into a bench dir next to the runs.
#   bench.sh [--models a,b] [--clips all-fr|all-en|<clip-id>] [--reps N] [--deb f | --debug]
# Defaults: parakeet-tdt-0.6b-v3 and turbo on all-fr, moonshine-base on all-en, 3 reps, the release .deb
# (build.sh --release first). --clips applies to every model given.
# Before its verdict it searches the bench dir for corpus text (dictation.py leak-check, a fresh clone in
# a 700 temp dir, deleted). The per-run leak checks of the journeys cover the transcripts.
# Last line: BENCH <dir> models=<n> failed=<n> (exit 0 when every model's journey passed).
set -uo pipefail
source "$(dirname "$0")/env.sh"
H="$DICTUS_HELPERS"
declare -A DEFAULT_CLIPS=([parakeet-tdt-0.6b-v3]=all-fr [turbo]=all-fr [moonshine-base]=all-en)
models="parakeet-tdt-0.6b-v3,turbo,moonshine-base" clips="" reps=3 deb_args=(--release)
while [ $# -gt 0 ]; do
  case "$1" in
    --models) models="$2"; shift ;;
    --clips) clips="$2"; shift ;;
    --reps) reps="$2"; shift ;;
    --deb) deb_args=(--deb "$2"); shift ;;
    --debug) deb_args=() ;;
    *) die "unknown option $1" ;;
  esac
  shift
done
IFS=, read -ra model_list <<< "$models"
for m in "${model_list[@]}"; do
  [ -n "$clips" ] || [ -n "${DEFAULT_CLIPS[$m]:-}" ] || die "no default clips for $m: pass --clips"
done
export DICTUS_TTL_MIN="${DICTUS_TTL_MIN:-30}"

bench="$DICTUS_EVIDENCE_ROOT/$(date +%Y%m%d-%H%M%S)-tier1-bench-$(git -C "$REPO_ROOT" rev-parse --short HEAD)"
mkdir -p "$bench"
{
  echo "date: $(date -Iseconds)"
  echo "commit: $(git -C "$REPO_ROOT" rev-parse HEAD) ($(git -C "$REPO_ROOT" rev-parse --abbrev-ref HEAD))"
  echo "app code: $(git -C "$REPO_ROOT" log -1 --format=%H -- src src-tauri)"
  echo "cpu: $(sed -n 's/^model name\s*: //p' /proc/cpuinfo | head -1), $(lscpu -p=CORE | grep -v '^#' | sort -u | wc -l) cores, $(nproc) threads"
  echo "ram: $(awk '/MemTotal/ {printf "%.1f GiB", $2/1048576}' /proc/meminfo)"
  echo "gpu: $(lspci 2>/dev/null | grep -iE 'vga|3d|display' | cut -d: -f3- | sed 's/^ //' | paste -sd ';' || echo none)"
  echo "gpu access: $(stat -c '%n %A %G' /dev/dri/renderD* 2>/dev/null | paste -sd ';'); user groups: $(id -Gn)"
  echo "kernel: $(uname -r)"
  echo "load at start: $(cut -d' ' -f1-3 /proc/loadavg)"
  echo "plan: models=$models clips=${clips:-default} reps=$reps deb=${deb_args[*]:-debug}"
} > "$bench/machine.txt"
echo "bench dir: $bench"

runs=() failed=0 cur="" corpus=""
trap '[ -n "$cur" ] && bash "$H/cleanup.sh" "$cur" > /dev/null; [ -n "$corpus" ] && rm -rf "$corpus"' EXIT
for m in "${model_list[@]}"; do
  sel="${clips:-${DEFAULT_CLIPS[$m]}}"
  echo "== $m on $sel, $reps reps"
  out="$("$H/launch.sh" --no-build "${deb_args[@]}" --model "$m" --mic "bench-$m" | tail -1)"
  run="$(sed -n 's/^READY run=\([^ ]*\) .*/\1/p' <<< "$out")"
  if [ -z "$run" ]; then
    echo "launch failed for $m: $out"; failed=$((failed + 1)); continue
  fi
  runs+=("$run") cur="$run"
  "$H/doctor.sh" "$run" > "$run/doctor.txt" || { tail -3 "$run/doctor.txt"; }
  "$H/journey-dictation.sh" --reps "$reps" "$sel" || failed=$((failed + 1))
  {
    echo "[$m] run: $(basename "$run") deb_sha=$(ledger_get "$run" DEB_SHA) identifier=$(ledger_get "$run" IDENTIFIER)"
    echo "[$m] settings: selected_language=$(python3 -I -c 'import json,sys;print(json.load(open(sys.argv[1]))["settings"].get("selected_language",""))' "$(run_data "$run")/settings_store.json" 2>/dev/null)"
    grep -ahoE 'Whisper accelerator set to: .*|ORT accelerator set to: .*|ggml_vulkan: [^|]*|whisper_backend_init_gpu: [^"]*|using [A-Za-z0-9_ ]+ backend' "$run/app.log" "$(run_data "$run")"/logs/*.log 2>/dev/null \
      | sed 's/^/['"$m"'] accel: /' | sort -u
  } >> "$bench/machine.txt"
  "$H/cleanup.sh" "$run" | tail -1
  cur=""
done

[ "${#runs[@]}" -gt 0 ] && python3 -I "$H/bench.py" collect "$bench" "${runs[@]}"
printf '%s\n' "${runs[@]}" > "$bench/runs.txt"
corpus="$(mktemp -d /tmp/verify-dictus-corpus.XXXXXX)" && chmod 700 "$corpus"
if gh repo clone "$DICTUS_CORPUS_REPO" "$corpus/repo" -- --depth 1 -q > /dev/null 2>&1; then
  leak="$(python3 -I "$H/dictation.py" leak-check "$corpus/repo" /nonexistent "$bench")" || failed=$((failed + 1))
  echo "$leak" | tee -a "$bench/machine.txt"
else
  echo "LEAK CHECK SKIPPED: clone failed" | tee -a "$bench/machine.txt"; failed=$((failed + 1))
fi
rm -rf "$corpus"
corpus=""
echo "BENCH $bench models=${#model_list[@]} failed=$failed"
[ "$failed" = 0 ]
