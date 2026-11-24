#!/usr/bin/env bash
# Feature `dictation.paste` on tier 1: one corpus dictation pasted into a terminal (xterm) and one into
# a GUI text field (zenity --entry, GTK), with the app's default paste method. During each insertion a
# second recording is started, to measure how long the microphone stays unavailable meanwhile.
# Needs a READY run launched with the launch-args below (run.sh adds them).
#   journey-paste.sh [<clip-id>]                    default en-long-01-workday-retry-fix (248 words)
# One line per target:
#   JOURNEY paste <terminal|gui> <clip> method=… chars_expected=N chars_received=N insert_ms=N
#     arrive_ms=N next_rec_ms=N clipboard=restored|cleared|overwritten primary=…
# insert_ms: log "Using paste method" → "Text pasted successfully" (polled every 20 ms).
# arrive_ms: first → last character received by the terminal. next_rec_ms: the second start command
# issued right after "Using paste method" → "Recording started". clipboard/primary: a sentinel put
# there before the dictation, read back 1 s after the paste.
# Videos paste-<target>.mp4 are pixelated so no text is legible; no screenshot is taken.
# Corpus rules as journey-dictation.sh: counts and timings only, leak check before the verdict.
# launch-args: --model parakeet-tdt-0.6b-v3 --mic
set -uo pipefail
source "$(dirname "$0")/env.sh"
clip_sel="${1:-en-long-01-workday-retry-fix}"
run="$(resolve_run "")" || die "no run of this session"
export DICTUS_VERIFY_RUN="$run" DISPLAY="$(ledger_get "$run" DISPLAY)"
H="$DICTUS_HELPERS"
scratch="$(ledger_get "$run" SCRATCH)"
data="$(run_data "$run")"
db="$data/history.db"
model="$(ledger_get "$run" MODEL)"
# The app's file log (Debug) holds the timings; since #83 it holds no transcript. Checked for leaks.
keep_log() {
  "$H/log.sh" since 0 > "$run/app-file.log"
  [ -n "${corpus:-}" ] && [ -d "$corpus/repo" ] || return 0
  leak="$(py dictation.py leak-check "$corpus/repo" "$db" "$run")" || { report "$run" "$leak"; echo "JOURNEY paste FAIL: $leak"; exit 1; }
  report "$run" "$leak"
}
fail() { keep_log; report "$run" "JOURNEY paste FAIL: $1"; echo "JOURNEY paste FAIL: $1"; exit 1; }
setting() { python3 -I -c 'import json,sys;print(json.load(open(sys.argv[1]))["settings"].get(sys.argv[2],""))' "$data/settings_store.json" "$1" 2>/dev/null; }
py() { python3 -I "$H/$1" "${@:2}"; }
ms() { echo $((($2 - $1) / 1000000)); }
xenv=(env -i PATH=/usr/bin:/bin HOME="$scratch/home" LANG=en_US.UTF-8 DISPLAY="$DISPLAY")
SENTINEL_CLIPBOARD="dictus-verify-clipboard-sentinel"
SENTINEL_PRIMARY="dictus-verify-primary-sentinel"

[ -n "$model" ] || fail "precondition: launch.sh --model <id> (no model seeded)"
[ "$(ledger_get "$run" MIC)" = 1 ] || fail "precondition: launch.sh --mic (no virtual microphone)"

corpus="$(mktemp -d /tmp/verify-dictus-corpus.XXXXXX)" || fail "mktemp"
chmod 700 "$corpus"
ledger_set "$run" CORPUS "$corpus"
gh repo clone "$DICTUS_CORPUS_REPO" "$corpus/repo" -- --depth 1 -q > /dev/null 2>&1 || fail "gh repo clone $DICTUS_CORPUS_REPO"
IFS=$'\t' read -r file id _ _ _ < <(py dictation.py clips "$corpus/repo" "$clip_sel")
[ -n "${id:-}" ] || fail "no clip $clip_sel"

"$H/log.sh" wait 'Signal handlers registered' 30 > /dev/null || fail "app log not ready"
for _ in $(seq 30); do [ "$(setting selected_model)" = "$model" ] && break; sleep 1; done
[ "$(setting selected_model)" = "$model" ] || fail "the app did not auto-select the seeded model $model"
method="$(setting paste_method)"
report "$run" "PASTE method=$method paste_delay_ms=$(setting paste_delay_ms) typing_tool=$(setting typing_tool)"

# X clients die with the run's Xvfb, which cleanup.sh stops.
setsid "${xenv[@]}" xterm -T dictus-paste-terminal -geometry 90x24+690+0 \
  -e /usr/bin/python3 -I "$H/paste.py" counter "$scratch/terminal.count" < /dev/null > /dev/null 2>&1 &

selection() { "${xenv[@]}" timeout 5 xclip -o -selection "$1" 2> /dev/null; }
verdict() { case "$1" in "$2") echo restored ;; "") echo cleared ;; *) echo overwritten ;; esac; }

loaded="" passed=0
for target in terminal gui; do
  if [ "$target" = gui ]; then
    rm -f "$scratch/gui.count"
    ( "${xenv[@]}" zenity --entry --title dictus-paste-gui --width=560 2> /dev/null | wc -m > "$scratch/gui.count" ) < /dev/null &
  fi
  geo=""
  for _ in $(seq 50); do geo="$(py paste.py window "dictus-paste-$target" 2> /dev/null)" && break; sleep 0.2; done
  [ -n "$geo" ] || fail "$target: no target window"
  read -r _ wh <<< "$geo"
  IFS='x+' read -r w h x y <<< "$wh"
  "$H/x.py" move $((x + w / 2)) $((y + h / 2)) --abs > /dev/null
  printf %s "$SENTINEL_CLIPBOARD" | "${xenv[@]}" xclip -selection clipboard -i
  printf %s "$SENTINEL_PRIMARY" | "${xenv[@]}" xclip -selection primary -i

  m="$("$H/log.sh" mark)"
  before="$(py dictation.py last-id "$db")"
  "$H/cli.sh" --toggle-transcription > /dev/null || fail "$target: cli start"
  "$H/log.sh" wait 'TranscribeAction::start called' 10 "$m" > /dev/null || fail "$target: the app did not start recording"
  py mic.py "$scratch" link 15 > /dev/null || fail "$target: virtual mic link"
  if [ -z "$loaded" ]; then
    loaded="$("$H/log.sh" wait "Successfully loaded transcription model: $model" 180 0)" || fail "model $model did not load"
  fi
  py mic.py "$scratch" play "$corpus/repo/$file" > /dev/null || fail "$target: play"

  ffmpeg -loglevel error -y -f x11grab -video_size "${w}x${h}" -i "$DISPLAY+$x,$y" -t "${DICTUS_PASTE_VIDEO_S:-40}" \
    -vf "scale=iw/10:ih/10,scale=iw*10:ih*10:flags=neighbor" -pix_fmt yuv420p "$run/paste-$target.mp4" < /dev/null &
  video=$!
  "$H/cli.sh" --toggle-transcription > /dev/null &
  stop_cli=$!
  t_paste="$(py paste.py wait-log "$data/logs" "$m" 'Using paste method' 240)" || fail "$target: no paste after stop"
  m_next="$("$H/log.sh" mark)"
  t_next="$(date +%s%N)"
  "$H/cli.sh" --toggle-transcription > /dev/null &
  next_cli=$!
  py mic.py "$scratch" link 120 > /dev/null &
  next_link=$!
  t_rec="$(py paste.py wait-log "$data/logs" "$m_next" 'Recording started in' 120)" || fail "$target: the next recording never started"
  t_done="$(py paste.py wait-log "$data/logs" "$m" 'Text pasted successfully' 120)" || fail "$target: the paste did not finish"
  wait "$stop_cli" "$next_cli" "$next_link"
  "$H/cli.sh" --cancel > /dev/null || fail "$target: cancel the next recording"

  row="$(py dictation.py wait-row "$db" "$before" "$t_next" 5)" || fail "$target: no history entry"
  expected="$(py paste.py chars "$db" "$(sed -n 's/^row=\([0-9]*\) .*/\1/p' <<< "$row")")"
  arrive=na
  if [ "$target" = terminal ]; then
    count="$(py paste.py settle "$scratch/terminal.count" 1 60)" || fail "terminal: nothing received"
    received="$(sed -n 's/^chars=\([0-9]*\) .*/\1/p' <<< "$count")"
    first="$(sed -n 's/.*first_ns=\([0-9]*\).*/\1/p' <<< "$count")"
    last="$(sed -n 's/.*last_ns=\([0-9]*\).*/\1/p' <<< "$count")"
    arrive="$(ms "$first" "$last")"
  else
    sleep 1
    "$H/x.py" key Return > /dev/null
    for _ in $(seq 50); do [ -s "$scratch/gui.count" ] && break; sleep 0.2; done
    received=$(($(cat "$scratch/gui.count" 2> /dev/null || echo 1) - 1))
  fi
  sleep 1
  clipboard="$(verdict "$(selection clipboard)" "$SENTINEL_CLIPBOARD")"
  primary="$(verdict "$(selection primary)" "$SENTINEL_PRIMARY")"
  wait "$video"

  [ "$received" = "$expected" ] && passed=$((passed + 1))
  line="JOURNEY paste $target $id method=$method chars_expected=$expected chars_received=$received insert_ms=$(ms "$t_paste" "$t_done") arrive_ms=$arrive next_rec_ms=$(ms "$t_next" "$t_rec") clipboard=$clipboard primary=$primary"
  report "$run" "$line"
  echo "$line"
  # A terminal counter that saw the gui paste would count it twice: point it elsewhere first.
  [ "$target" = terminal ] && "$H/x.py" move 10 10 --abs > /dev/null
  sleep 1
done

keep_log
summary="JOURNEY paste $id $([ "$passed" = 2 ] && echo PASS || echo FAIL) $passed/2 method=$method model=$model"
report "$run" "$summary"
echo "$summary"
[ "$passed" = 2 ]
