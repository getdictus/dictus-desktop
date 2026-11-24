#!/usr/bin/env bash
# Feature `dictation.corpus` on tier 1: real dictations through the app's own recording and transcription
# pipeline, from the private test-voices corpus played into the run's virtual microphone. Start and stop
# go through `dictus --toggle-transcription` (a second instance, cli.sh); the text is read from the
# isolated history.db and scored by WER against the manifest. Needs a READY run launched with
# the launch-args below (run.sh adds them).
#   journey-dictation.sh [--reps N] [<clip-id>|all-<lang>|short-<lang>]   default all-fr, 1 rep
# all-<lang> takes every clip with a reference text; short-<lang> only the short ones.
# --reps N plays every short clip N times, in rounds (all clips, then all again), the long clip once.
# The model's load (it starts with the first recording) is awaited before the first clip plays:
#   JOURNEY dictation load model=<id> load_ms=N
# One line per dictation: JOURNEY dictation <id> PASS|FAIL wer=… wer_sp=… stop_to_text_ms=… … seq=N rep=N
# Last line: JOURNEY dictation <selector> PASS <n>/<n> … (exit 0) or … FAIL: <step> (exit 1).
# Corpus rules: internal testing only. Texts and transcripts never reach stdout, the run dir or a capture.
# launch-args: --model parakeet-tdt-0.6b-v3 --mic
set -uo pipefail
source "$(dirname "$0")/env.sh"
reps=1
[ "${1:-}" = --reps ] && { reps="$2"; shift 2; }
[[ "$reps" =~ ^[1-9][0-9]*$ ]] || { echo "error: --reps needs a positive integer" >&2; exit 1; }
sel="${1:-all-fr}"
run="$(resolve_run "")" || die "no run of this session"
export DICTUS_VERIFY_RUN="$run" DISPLAY="$(ledger_get "$run" DISPLAY)"
H="$DICTUS_HELPERS"
WER_MAX="${DICTUS_WER_MAX:-0.25}"
scratch="$(ledger_get "$run" SCRATCH)"
data="$(run_data "$run")"
db="$data/history.db"
model="$(ledger_get "$run" MODEL)"
fail() { report "$run" "JOURNEY dictation $sel FAIL: $1"; echo "JOURNEY dictation $sel FAIL: $1"; exit 1; }
setting() { python3 -I -c 'import json,sys;print(json.load(open(sys.argv[1]))["settings"].get(sys.argv[2],""))' "$data/settings_store.json" "$1" 2>/dev/null; }
py() { python3 -I "$H/$1" "${@:2}"; }

[ -n "$model" ] || fail "precondition: launch.sh --model <id> (no model seeded)"
[ "$(ledger_get "$run" MIC)" = 1 ] || fail "precondition: launch.sh --mic (no virtual microphone)"

corpus="$(mktemp -d /tmp/verify-dictus-corpus.XXXXXX)" || fail "mktemp"
chmod 700 "$corpus"
ledger_set "$run" CORPUS "$corpus"
gh repo clone "$DICTUS_CORPUS_REPO" "$corpus/repo" -- --depth 1 -q > /dev/null 2>&1 || fail "gh repo clone $DICTUS_CORPUS_REPO"
report "$run" "CORPUS $DICTUS_CORPUS_REPO@$(git -C "$corpus/repo" rev-parse --short HEAD) in $corpus (mode 700)"
mapfile -t clips < <(py dictation.py clips "$corpus/repo" "$sel")
[ "${#clips[@]}" -gt 0 ] || fail "no clip for $sel"
plan=()
for rep in $(seq "$reps"); do
  for clip in "${clips[@]}"; do
    [ "$rep" = 1 ] || [ "$(cut -f5 <<< "$clip")" = short ] && plan+=("$rep"$'\t'"$clip")
  done
done

"$H/log.sh" wait 'Signal handlers registered' 30 > /dev/null || fail "app log not ready"
for _ in $(seq 30); do [ "$(setting selected_model)" = "$model" ] && break; sleep 1; done
[ "$(setting selected_model)" = "$model" ] || fail "the app did not auto-select the seeded model $model"
"$H/evidence.sh" 20-dictation-ready > /dev/null
# With launch.sh --sink the recording sounds go to the private null sink: link the feedback output
# the app opens at startup.
if [ "$(ledger_get "$run" SINK)" = 1 ]; then
  out="$(py mic.py "$scratch" link-output 30 2>&1)" || fail "feedback output: $out"
  report "$run" "$out"
fi

passed=0 n=0 loaded=""
for entry in "${plan[@]}"; do
  IFS=$'\t' read -r rep file id lang dur category <<< "$entry"
  n=$((n + 1))
  m="$("$H/log.sh" mark)"
  before="$(py dictation.py last-id "$db")"
  "$H/cli.sh" --toggle-transcription > /dev/null || fail "$id: cli start"
  "$H/log.sh" wait 'TranscribeAction::start called' 10 "$m" > /dev/null || fail "$id: the app did not start recording"
  py mic.py "$scratch" link 15 > /dev/null || fail "$id: virtual mic link"
  if [ -z "$loaded" ]; then
    loaded="$("$H/log.sh" wait "Successfully loaded transcription model: $model" 180 0)" || fail "$id: model $model did not load"
    report "$run" "MODEL LOADED ${loaded#*$model }"
    load_ms="$(sed -n 's/.*(took \([0-9]*\)ms).*/\1/p' <<< "$loaded")"
    report "$run" "JOURNEY dictation load model=$model load_ms=$load_ms"
    echo "JOURNEY dictation load model=$model load_ms=$load_ms"
  fi
  py mic.py "$scratch" play "$corpus/repo/$file" > /dev/null || fail "$id: play"
  t_stop="$(date +%s%N)"
  "$H/cli.sh" --toggle-transcription > "$scratch/cli-stop.out" &
  cli_pid=$!
  row="$(py dictation.py wait-row "$db" "$before" "$t_stop" 180)" || fail "$id: no history entry after stop"
  wait "$cli_pid" || fail "$id: cli stop"
  cli_ms="$(sed -n 's/.* in \([0-9]*\)ms$/\1/p' "$scratch/cli-stop.out")"
  row_id="$(sed -n 's/^row=\([0-9]*\) .*/\1/p' <<< "$row")"
  score="$(py dictation.py score "$corpus/repo" "$file" "$db" "$row_id" "$WER_MAX")" || fail "$id: score"
  tr_ms="$(py dictation.py transcribe-ms "$data/logs" "$m")"
  sleep 1
  fb="$(py dictation.py feedback-ms "$data/logs" "$m")"
  [ "${score%% *}" = PASS ] && passed=$((passed + 1))
  line="JOURNEY dictation $id $score ${row#row=* } transcribe_ms=$tr_ms $fb cli_stop_ms=$cli_ms dur_s=$dur category=$category lang=$lang model=$model seq=$n rep=$rep"
  report "$run" "$line"
  echo "$line"
done

leak="$(py dictation.py leak-check "$corpus/repo" "$db" "$run")" || { report "$run" "$leak"; fail "$leak"; }
report "$run" "$leak"
summary="JOURNEY dictation $sel $([ "$passed" = "$n" ] && echo PASS || echo FAIL) $passed/$n wer_max=$WER_MAX model=$model"
report "$run" "$summary"
echo "$summary"
[ "$passed" = "$n" ]
