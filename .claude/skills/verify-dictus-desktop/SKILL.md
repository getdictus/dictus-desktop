---
name: verify-dictus-desktop
description: Verify Dictus Desktop (Tauri 2, com.dictus.desktop) the way a user does. Tier 1 builds the debug .deb on this shared Linux machine under /tmp/dictus-gradle.lock and runs it on a private Xvfb with a private D-Bus and an empty HOME, driven by an XTEST helper. Tier 2 is the Windows VM dictus-win, tier 3 Pierre's Mac (no focus stealing), tier 4 Pierre. Use when a change to src/ or src-tauri/ has to be seen working in the real app rather than in unit tests, when a feature of features/README.md must be exercised, or when the screen map needs new captures.
---

# Verify Dictus Desktop

Five steps, always in this order: **Launch → Doctor → Drive → Evidence → Cleanup**. Helpers live in `helpers/` and run from the repository root. Read first: `features/README.md` maps every user feature to its recipe, `features/screens.md` every screen, with captures in `screens/`.

This Linux machine is shared: Andy's Android builds, OpenBot (Xvfb `:99` and `:100`) and Pierre's own Dictus data live here. Three rules follow from that, and every helper enforces them:

- **The shared lock `/tmp/dictus-gradle.lock` is held only while work runs.** `build.sh` holds it for the build alone (`flock` releases it when the build ends). `launch.sh` takes it again only while the app runs, and `cleanup.sh` releases it. A watchdog armed by Launch runs Cleanup after `DICTUS_TTL_MIN` minutes (default 20), so a run never holds the lock idle. Never wait on anything (a question, an approval, a long think) while a run is up: run `cleanup.sh` first, then wait.
- **Pierre's real data is never touched.** Each run records a sha256 fingerprint of `~/.local/share/com.dictus.desktop/{history.db,settings_store.json}` (and the `.dev` identity's) before it starts and checks it again in Cleanup and Doctor.
- **Nothing destructive goes inside `bash -c` / `sh -c` strings, loops or one-off pipelines.** `rm`, `kill`, `virsh destroy` and `snapshot-revert` live only in the committed helpers (mostly `cleanup.sh`) or run as plain top-level commands. A hand-written loop that deletes or kills something can stall on a permission prompt while the run holds the lock. If any command needs an approval, run `cleanup.sh`, stop and report. Never answer a T3 permission request yourself.

## Test tiers

Pick the lowest tier that can prove the change. The PR body names the tier used and what was left to the next tier.

| Tier                   | Who and where                                                     | Proves                                                                                                                                                                                                                                                                                                                                                                                                                                                            | Does not prove                                                                                                                                                                                                                                 |
| ---------------------- | ----------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1. Linux, Xvfb         | An agent with this skill on this machine                          | The app builds, packages (.deb) and starts from an empty data dir; UI, navigation, onboarding, model download/verify/extract/load, settings persistence, the log; every screen of `features/screens.md`; a real dictation minus the shortcut: CLI start/stop, recording from a virtual microphone, VAD, Parakeet V3 transcription, the History entry, scored by WER (`journey-dictation.sh`); its paste into an xterm and a GTK entry on X11 (`journey-paste.sh`) | The global shortcut keys, paste into the user's own apps and on Wayland, a real microphone or sound card, sound playback, a real desktop (no window manager, no tray host, no Wayland), macOS and Windows code paths, the signed release build |
| 2. Windows VM          | An agent, `dictus-win` on this host                               | The CI or draft installer (NSIS per user, MSI per machine) installs and starts on a clean Windows 11, the first window renders, missing-runtime crashes (#72)                                                                                                                                                                                                                                                                                                     | Audio, a real GPU, Windows on ARM, long sessions (the evaluation ISO is expired: keep runs short)                                                                                                                                              |
| 3. macOS, Pierre's Mac | An agent in a T3 thread on the Mac, macOS-specific behaviour only | Permissions (microphone, accessibility), Metal, the overlay, the menu bar, macOS paste                                                                                                                                                                                                                                                                                                                                                                            | Anything tier 1 can prove: do it there first                                                                                                                                                                                                   |
| 4. Pierre              | Pierre, in a batch test                                           | Real voice, transcription quality, the feel of shortcuts, paste and sounds in his own apps                                                                                                                                                                                                                                                                                                                                                                        | Nothing is merged before his OK                                                                                                                                                                                                                |

**1. Linux on Xvfb.** The default for every PR. Launch, Doctor, Drive, Evidence and Cleanup below are this tier.

**2. Windows VM.** Only after a green tier 1, and only when the lock is free. See "Tier 2: Windows VM" below. Target under 20 minutes per run.

**3. macOS on Pierre's Mac.** Driven from a T3 thread on the Mac, for what only macOS has. These rules are Pierre's, word for word:

> « Tests sur le Mac : jamais de vol de focus
>
> - Tests d'interface et captures : d'abord sur le Linux (Xvfb). Le Mac seulement pour ce qui est propre à macOS.
> - Sur le Mac : app lancée en arrière-plan (`open -g`, ou build de dev lancé sans focus ni activation), capture de la fenêtre par son id (`screencapture -x -o -l <windowid>`). Jamais d'activation de l'app, de `set_focus` ni de passage au premier plan.
> - Ne jamais quitter le Dictus installé de Pierre : les builds de dev ont leur propre identifiant (#70).
> - Un test qui exige le premier plan : demander d'abord à Pierre, via Titus. »

**4. Pierre.** Real voice in French and other languages, text quality, how shortcuts, paste and sounds feel. Batch test: Pierre replies per PR with `#<num> OK` or `#<num> KO: <what failed>`. Nothing is merged before his OK.

## Launch (tier 1)

```bash
H=.claude/skills/verify-dictus-desktop/helpers
$H/doctor.sh                       # read-only: lock, displays, RAM, tools, real-data fingerprint, VM
$H/launch.sh                       # build.sh (lock for the build only), then lock + app on Xvfb
$H/launch.sh --no-build            # reuse the newest src-tauri/target/debug/bundle/deb/*.deb
$H/launch.sh --no-build --release  # the same from target/release (build.sh --release)
$H/launch.sh --deb <file.deb>      # a given .deb (e.g. a CI artifact)
$H/launch.sh --model <id>          # seed a catalogue model from the shared cache before the app starts
$H/launch.sh --mic                 # private PipeWire + virtual microphone in the scratch dir
$H/run.sh onboarding [--no-build]  # Launch + Doctor + journey-onboarding.sh + Cleanup, in one process
$H/run.sh dictation --no-build -- all-fr   # the same for journey-dictation.sh (adds its --model and --mic)
```

`launch.sh`, in order:

1. Creates the run dir `/home/pierre/OpenBot/Shared/pilotage/dictus-desktop/preuves/verify-dictus-desktop/<YYYYMMDD-HHMMSS>-tier1-<label>-<sha>/`, its ledger `run.env`, and `fingerprint-before.txt`. Refuses if this session has a run not yet cleaned up.
2. Unless `--no-build`/`--deb`: `build.sh` runs `bun install --frozen-lockfile` (if `node_modules` is missing) and `bun run tauri build --debug --bundles deb` under `flock /tmp/dictus-gradle.lock`, log in `build.log`. Measured on 2026-11-17: 294 s cold in a fresh worktree. The command then exits 1 on the updater signing step (`A public key has been found, but no private key`): expected without `TAURI_SIGNING_PRIVATE_KEY`, never add a key. A .deb newer than the build start is the success criterion.
3. Takes the lock for the app phase (waits up to 30 min; `DICTUS_LOCK_WAIT_S`), arms the watchdog.
4. `dpkg-deb -x` into `/tmp/verify-dictus-desktop.XXXXXX/pkg` and runs `pkg/usr/bin/dictus`. The raw `target/debug/dictus` panics on missing resources (#71).
5. Starts `Xvfb :N` (first free in `:120`–`:199`, never `:99`/`:100`, 1280x900), a private `dbus-daemon --session`, then Dictus with `env -i`: `HOME` and every `XDG_*` dir under the scratch dir, `LANG=en_US.UTF-8` (English labels), `GDK_BACKEND=x11`, `WEBKIT_DISABLE_DMABUF_RENDERER=1`. The private D-Bus also isolates the single-instance check (D-Bus on Linux), so a running Dictus of Pierre's is never handed the launch. The app's full environment, D-Bus address included, is kept in `<scratch>/app.env` for `cli.sh`.
6. With `--model <id>`, before the app starts: `model-cache.sh` (before the lock) returns the archive, then it is extracted into the isolated `models/` the way the app does it. With `--mic`: the private PipeWire (see "Dictation with the test-voices corpus").

**Ready means** the `Dictus` window is mapped and the process is still alive 3 s later. The window appears in about 4 s (the ~25 s seen in #71 did not happen with the private D-Bus). Last line, the only one to rely on:

```
READY run=<run-dir> display=:120 pid=<pid> window=0x200003 geometry=680x570+0+0
```

Every helper after Launch finds the run on its own (session pointer under `…/verify-dictus-desktop/sessions/`). Two agents sharing one checkout each export a distinct `DICTUS_VERIFY_SESSION`.

**Identity.** Since PR #75 a debug .deb is "Dictus Dev" (package `dictus-dev`, identifier `com.dictus.desktop.dev`, update checking disabled); a release .deb is package `dictus`, `com.dictus.desktop`. `launch.sh` reads the package name and records `IDENTIFIER` in the ledger; every helper finds the isolated data dir from it (`run_data` in `env.sh`). `DICTUS_IDENTIFIER` is only the fallback for a run without that line. `x.py` matches both titles. Keep the isolated HOME anyway: it is what makes onboarding reachable from an empty data dir.

## Doctor

```bash
.claude/skills/verify-dictus-desktop/helpers/doctor.sh             # this session's run, if any
.claude/skills/verify-dictus-desktop/helpers/doctor.sh <run-dir>
```

Read-only. One `OK` / `WARN` / `FAIL` line per check, then `DOCTOR OK` or `DOCTOR FAIL <n>` (exit 1):

- Tools: bun, cargo, Xvfb, dbus-daemon, ffmpeg, xwininfo, dpkg-deb, lslocks, flock, python3, libXtst; virsh and gh (tier 2).
- The lock: free, held by this run, or held by another process (pid, command, working directory, from `lslocks`).
- Displays in use and the next free one; RAM available and heavy processes (java, cargo, rustc, qemu) running: Andy's Android builds share this machine.
- The real-data fingerprint, and whether a Dictus not started by this run is running (it would change the fingerprint legitimately).
- `dictus-win` state and snapshots (WARN when running and not started by this run).
- With a run: Xvfb and dictus are this run's (pid + start time), the window is mapped, the app's `HOME` is the scratch dir, the real data is unchanged since the run began.

Run it before Launch, after Launch, and whenever something looks off. A `FAIL` stops the run: Cleanup, then fix or report.

## Drive (tier 1)

No xdotool on this machine. `helpers/x.py` sends XTEST events through ctypes on the system libX11/libXtst, on the run's display. Coordinates are relative to the Dictus window (680x570 at `+0+0`; no window manager), read from a fresh `shot.sh` capture, never guessed. The webview exposes no accessibility tree to X, so the stable handles are the English labels in the captures plus the log lines and files below.

```bash
H=.claude/skills/verify-dictus-desktop/helpers
export DISPLAY=:120                      # from READY, or: sed -n 's/^DISPLAY=//p' <run-dir>/run.env
$H/x.py window                           # "<id> 680x570+0+0"
$H/x.py click 340 320                    # window-relative; --abs for screen coordinates
$H/x.py scroll down 5 --at 400 300
$H/x.py type "hello"                     # ASCII, Xvfb's US keymap
$H/x.py key ctrl+shift+d                 # chords: ctrl, shift, alt, super + any keysym
$H/nav.sh models                         # sidebar: general models advanced history transcribe-file post-process debug about
M=$($H/log.sh mark)                      # assert only on log lines after the mark
$H/log.sh wait 'SHA256 verified for model' 120 "$M"
$H/log.sh since "$M"
```

**Logs that prove an action** (the app's own log, `logs/*.log` in the isolated data dir, UTC timestamps):

| Action         | Line (regex)                                                                                                                                                      |
| -------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| App ready      | `Signal handlers registered`, `Recording overlay window created`                                                                                                  |
| Model download | `Starting fresh download of model <id>`, `SHA256 verified for model <id>`, `Successfully extracted archive for model: <id>`, `Successfully downloaded model <id>` |
| Model load     | `Successfully loaded transcription model: <id> \(took \d+ms\)`                                                                                                    |
| Shortcuts      | `Shortcuts initialized successfully`                                                                                                                              |

**The recipe this skill is proven on**: feature `onboarding`, 2026-11-17, run `20261117-215926-tier1-onboarding-92ded2a` (PASS in 43 s with the build), see the run report `/home/pierre/OpenBot/Shared/pilotage/dictus-desktop/preuves/2026-11-17-verify-dictus-desktop-tier1-onboarding.md`. Journey `dictation.corpus`: 2026-11-18, run `20261118-161131-tier1-dictation-2132175` (app code of `main` at `2132175`, PASS 5/5 in 131 s without the build), report `/home/pierre/OpenBot/Shared/pilotage/dictus-desktop/preuves/2026-11-18-verify-dictation-corpus.md`.

```bash
H=.claude/skills/verify-dictus-desktop/helpers
$H/run.sh onboarding        # or, step by step:
$H/launch.sh onboarding && $H/doctor.sh && $H/journey-onboarding.sh; $H/cleanup.sh
```

Every other feature's recipe is in `features/<id>.md`, under "Driving it with x.py".

Rules for every feature:

- One journey per run, then Cleanup. Do not leave the app up between steps of your own reasoning: the watchdog will clean it up after 20 min anyway.
- Download only the model the journey needs. Moonshine V2 Tiny (31 MB) is the default; it lands in the scratch dir and Cleanup deletes it. A bigger model goes through `--model` and the shared cache instead of a download per run.
- There is no sound card in the isolated session. Recording works only with `--mic` (the virtual microphone below); sound playback is never drivable on tier 1: report it as not driven, with that reason.

## Dictation with the test-voices corpus (tier 1)

Journey `dictation.corpus`, recipe in `features/dictation.global-shortcut.md`. It proves that a dictation goes through the real app: CLI start, recording from a microphone, VAD, transcription with Parakeet V3, the History entry. It leaves out the shortcut keys and paste.

```bash
H=.claude/skills/verify-dictus-desktop/helpers
$H/run.sh dictation --no-build -- all-fr            # every French clip; or -- fr-03-shopping for one
```

Output, one line per clip, then the verdict, then Cleanup:

```
JOURNEY dictation fr-02-appointment-numbers PASS wer=0.071 wer_sp=0.071 ref_words=14 hyp_words=14 recorded_ms=5100 gate=capture+wer<=0.25 stop_to_text_ms=508 transcribe_ms=290 cli_stop_ms=57 dur_s=4.8 category=short lang=fr model=parakeet-tdt-0.6b-v3
JOURNEY dictation all-fr PASS 5/5 wer_max=0.25 model=parakeet-tdt-0.6b-v3
```

**The corpus is private** (`getdictus/dictus-test-voices`, its NOTICE): internal testing only, never published or redistributed, never used to train or evaluate a model for training. The journey enforces that:

- It clones the repository at run time (`gh repo clone … -- --depth 1`) into a fresh `mktemp -d /tmp/verify-dictus-corpus.XXXXXX`, mode 700, listed in the ledger as `CORPUS`. `cleanup.sh` deletes it on every exit, and also deletes any clone older than the TTL that a dead run left behind. Doctor warns while one exists.
- Nothing from the corpus is ever committed to this repository: no clip, no expected text, no transcript.
- Texts never leave memory. `dictation.py` reads the expected text from `manifest.json` and the transcript from the isolated `history.db`. It prints only ids, word counts, scores and timings. No capture is taken after the first dictation: the History page, the clipboard and the window could show the transcript. `app.log` in the run dir is the app's stdout. Since #83 the app logs only the transcript's length (`Transcription result: N chars`) and keeps text out of every log unless Debug → « Log transcripts » is on (off by default; then at Debug level, in the file log only), so `launch.sh` sets no `RUST_LOG` filter. Before #83 the transcript went to stdout at Info and to the file log at Debug; the first run of this journey leaked it into `app.log`, and the leak check caught it.
- Before its verdict the journey runs `dictation.py leak-check`: every text file of the run dir is searched for any 5-word run of any manifest text or transcript, after normalisation. A hit fails the run and names the file only.

**Virtual microphone.** Dictus records with cpal's ALSA backend, from ALSA `default`. On this machine `default` is the PipeWire ALSA plugin (`/usr/share/alsa/alsa.conf.d/99-pipewire-default.conf`), which connects to `$XDG_RUNTIME_DIR/pipewire-0`. `launch.sh --mic` starts a private `pipewire` daemon with the app's isolated `HOME` and `XDG_RUNTIME_DIR` (the scratch dir), so the app's `default` reaches it and nothing else. Its config (`mic.py conf`, written to `<scratch>/pipewire.conf`) loads:

- no ALSA, Bluetooth or V4L2 plugin, so it cannot open a sound card;
- no session manager;
- a dummy driver for the clock;
- a `pipe-tunnel` source `dictus-virtual-mic` reading `<scratch>/mic.fifo` as 16 kHz mono s16.

`mic.py check` asserts the private graph has that source and zero devices, and Doctor runs it. With no session manager nothing configures or links ports. `mic.py link` does WirePlumber's job for the one stream: `PortConfig` (dsp) on both nodes, then `pw-link`. `mic.py play` writes the WAV into the FIFO at real time; the source outputs silence when the FIFO is empty. Every `pw-*` call runs with the run's `HOME` and `XDG_RUNTIME_DIR` only, so it cannot reach the machine's own PipeWire, which another agent uses. No package was installed: `pipewire`, `pw-cli`, `pw-link` and `pw-dump` were already there.

Measured on 2026-11-18 while choosing this design:

- An ALSA-only device (`~/.asoundrc` with the `file` plugin's `infile` on a FIFO, `null` slave) is not paced. `arecord` got 3 s of "audio" in 0.56 s, 84 % of it zeros.
- WirePlumber was not used: its ALSA monitor would probe the real sound cards. The private D-Bus would hide the real session's device reservation from it, so it could open a card the real session is using.
- With `module-access` missing, the daemon accepts clients but never answers them.

Limits:

- About 0.5 s of silence comes before the clip: the stream must exist before it can be linked.
- The clip is a clean TTS signal, sent digitally: no room, no noise, no level or gain issue. Tier 4 covers real voice.
- Paste is proven by its own journey, `journey-paste.sh` (`dictation.paste` in `features/dictation.global-shortcut.md`): into an xterm and a GTK entry on the run's X11 display, never into Wayland or a real desktop's apps.

**Model.** Parakeet V3 (`parakeet-tdt-0.6b-v3`, 456 MB, French among 25 languages; Moonshine V2 Tiny is English only). `model-cache.sh` keeps the archive in `~/.cache/verify-dictus-desktop/models/` (`DICTUS_MODEL_CACHE`), outside any isolated HOME and never deleted by Cleanup. It reads the url and SHA256 from `src-tauri/src/managers/model.rs`, so a catalogue change invalidates the cache, and it verifies the SHA256 on every run, as the app does after a download. `launch.sh --model` extracts it into the isolated `models/` before the app starts. **Selection is the app's own**: with one model on disk and none selected, `ModelManager::auto_select_model_if_needed` selects it at startup. The journey asserts `selected_model` in `settings_store.json` and edits no setting. The first clip waits for `Successfully loaded transcription model` (logged in `report.txt` as `MODEL LOADED (took Nms)`), so no `stop_to_text_ms` includes the load.

**Start and stop.** `cli.sh --toggle-transcription`: a second `dictus` with the app's exact environment, handed to the running instance over the private D-Bus (`tauri_plugin_single_instance`). The shortcut keys are not used.

**Measures.** All in `report.txt` and on stdout:

- `stop_to_text_ms`: from the stop command being issued to the History row being readable, polled every 10 ms. It includes `cli_stop_ms`, the second instance's start and hand-off, which a shortcut does not pay.
- `transcribe_ms`: the app's own "Transcription completed in" duration, the number only.
- `recorded_ms`: the length of the saved recording.

**WER.** Word error rate against the manifest text. Normalisation, both sides:

- NFKC, lowercase;
- apostrophes and hyphens become spaces, then every non-word character goes;
- tokens are `\w+` runs.

`wer` is that. `wer_sp` also removes spoken punctuation from both sides (« virgule », « point », « point d'interrogation », « deux points », « à la ligne », …; list in `dictation.py`), so a clip that dictates « virgule » scores the same whether the app writes the word or the mark. Before scoring, a time (« 14:30 », « 14h30 », « 14 h 30 ») is written « 14 h 30 », and digits are split from letters. The model writes times in either form from one run to the next: `fr-02` scored 0.429 and then 0.214 before this rule. Digits against words (« 14 » / « quatorze ») are not normalised and count as errors.

**Verdict, per clip** (computed in `dictation.py score`):

- Every clip must pass the capture gate: the saved recording is at least as long as the clip, and the transcript is not empty.
- A `short` clip must also have `min(wer, wer_sp) <= 0.25` (`DICTUS_WER_MAX`). The threshold catches a broken path: silence, a wrong model or truncated audio give a WER near 1. It does not judge fine quality, which is tier 4's job.
- A `long-codeswitch` clip reports its WER without gating on it.

The run passes when every clip passes and the leak check is clean.

The long clip is ungated because of a model finding, measured on 2026-11-18. Parakeet V3 transcribes the 72 s French/English clip `fr-long-01` to 54–96 of 244 words (WER 0.83–0.93 over three runs). Every matched word falls in the first 60 % of the text. The recording is complete: steady level over the full 75 s, `recorded_ms` ≥ clip length. The short clips score 0.000–0.071 through the same path. A tier-1 PR cannot fix that, so the journey shows the number and does not turn red on it. Gate it once the model or the app handles long code-switched dictation.

## Dictation benchmark (tier 1)

`bench.sh` turns the dictation journey into a baseline to compare before and after a performance change: per model and clip, WER, stop → text, transcription time, the cold model load, and the first dictation after launch against the following ones.

```bash
H=.claude/skills/verify-dictus-desktop/helpers
$H/build.sh --release <log>                 # once per app commit; a timing on the debug .deb means nothing
$H/bench.sh                                 # parakeet-tdt-0.6b-v3 + turbo on all-fr, moonshine-base on all-en, 3 reps
$H/bench.sh --models turbo --clips fr-03-shopping --reps 5
$H/bench.py compare <before>/bench.csv <after>/bench.csv
```

**Release build only.** `bench.sh` uses the newest `src-tauri/target/release/bundle/deb/*.deb` (`--deb <f>` for another, `--debug` to opt out). The debug .deb compiles whisper.cpp without optimisation (the `cmake` crate follows the Rust profile), so its timings do not describe what users run. The release .deb is the production identity `com.dictus.desktop`; it runs in the same isolated HOME, and the fingerprint check guards Pierre's real data as for any run.

**One launch per model.** For each model `bench.sh` runs `launch.sh --no-build --release --model <id> --mic`, Doctor, `journey-dictation.sh --reps N <clips>`, then `cleanup.sh`, which releases the lock before the next model. Nothing holds the lock between models, and an interrupted bench cleans up its current run (trap).

**Model switching.** Seed exactly one model per launch and let the app select it. With one model on disk and none selected, `auto_select_model_if_needed` (`managers/model.rs`) selects it at startup, and the journey asserts `selected_model` before the first clip. This is the reliable way, for three reasons:

- With several models on disk, auto-selection takes the first downloaded model of a `HashMap`, so which one wins is not defined.
- Writing `selected_model` into `settings_store.json` before launch means writing the app's whole settings schema by hand: `get_settings` replaces a store that does not deserialize with the defaults.
- Switching in the UI (Models page or footer) was not used: each switch needs a capture to prove it, and the bench wants a cold launch per model anyway to measure the load.

**Models.** Catalogue ids and URLs come from `managers/model.rs` through `model-cache.sh`, which keeps them in `~/.cache/verify-dictus-desktop/models/` and checks the SHA256 on every run. A directory model (Parakeet, Moonshine) is a `.tar.gz` extracted into `models/<filename>`. A Whisper model is a single file (`turbo` is `ggml-large-v3-turbo.bin`, 1.5 GB) copied there. Moonshine Base is English only: the bench gives it the English clips.

**Order and repetitions.** `--reps N` plays the short clips in N rounds (every clip, then every clip again), so warm-up and drift do not favour one clip. The long clip plays once, in round 1. The first dictation after launch is `seq=1`.

**Measures**, per dictation in `bench.csv` (one row each, ids and numbers only):

- `wer`, `wer_sp`: the journey's two normalisations (see "WER" above).
- `stop_to_text_ms`: stop command issued → History row readable. It includes `cli_stop_ms`, the second instance's hand-off, which a shortcut does not pay.
- `transcribe_ms`: the app's "Transcription completed in" duration.
- `load_ms`: the app's `Successfully loaded transcription model … (took Nms)`, cold, once per launch. The load starts with the first recording; the journey waits for it before playing the first clip, so no dictation timing includes it.
- `seq`, `rep`: the dictation's rank since launch, and its round.

`bench.md` has two tables. Per model and clip: n, WER median (min–max), stop → text and transcription median (min–max) over the warm dictations (`seq` > 1). Per model: the cold load, the first dictation's stop → text and transcription, next to the median of the same clip's later dictations. `machine.txt` records CPU, threads, RAM, GPU, kernel, load at start, the commit and the last commit touching app code, the .deb hash, the language setting, and the accelerator lines from the app log (Whisper Vulkan or CPU, ORT). `runs.txt` lists the per-model run dirs, each with its own report and leak check.

**Comparing two runs.** Run `bench.sh` on the base commit and on the change, with the same `--models`, `--clips` and `--reps`, on a quiet machine (check `load at start` in both `machine.txt`). `bench.py compare before.csv after.csv` prints, per model, clip and metric, both medians with their ranges, the median delta, and `within noise` when the ranges overlap. A delta inside overlapping ranges is no measured difference: add reps, or interleave the two builds (`--deb`), before claiming one.

**Verdict.** `failed=N` counts the models whose journey failed a gate, or whose launch or leak check failed. The numbers are recorded either way: a gate failure is a finding to read in the table, not a broken bench.

**Baseline**, 2026-11-18, release .deb of app code `main` `2132175`, 3 reps, 18 min of app runtime for the three models: bench dir `20261118-163127-tier1-bench-b69c0a8`, report `/home/pierre/OpenBot/Shared/pilotage/dictus-desktop/preuves/2026-11-18-bench-baseline.md`. What limits the numbers on this machine:

- Whisper runs on the CPU. The user cannot open `/dev/dri/renderD128` (group `render`), so `ggml_vulkan: No devices found.` and `whisper_backend_init_gpu: no GPU found`. The process holds a flat 400 % CPU while transcribing: 4 threads of 16. Whisper pads every clip to its 30 s window, so Turbo takes ~40 s for a 4 s clip and ~87 s for the 72 s one. A Whisper timing from tier 1 is a 4-thread CPU timing, not what a user with a GPU sees.
- Moonshine Base rejects audio over 64 s (`Audio duration must be between 0.1s and 64s`, in the app's debug log). The app then saves an empty History entry in ~200 ms, so the long English clip scores WER 1.000 and fails the capture gate. It is a product limit, not a bench fault.
- Parakeet V3 gives two different outputs for `fr-02-appointment-numbers` across reps (14 or 16 words, WER 0.071 to 0.357), with the same recording length. Its other short clips are stable at 0.000.

**Corpus rules.** The journey's rules apply unchanged: each run clones the corpus into its own 700 temp dir, deleted by Cleanup, and runs its leak check. `bench.py` parses every field against a fixed schema (a number or an `[A-Za-z0-9._-]` id), so no text can reach the bench files. `bench.sh` ends with a leak check of the bench dir against a fresh clone, deleted at once.

## Evidence

```bash
$H/shot.sh 01-before          # the Dictus window only (ffmpeg x11grab cropped to the window)
$H/evidence.sh 02-after       # screenshot + <name>.state.txt: models on disk, key settings, last log lines
$H/screen.sh app.settings.models   # a screen-map capture: screens/<id>.png in this skill (tier 1 only)
```

Everything for a run lands in `/home/pierre/OpenBot/Shared/pilotage/dictus-desktop/preuves/verify-dictus-desktop/<run-id>/`, **never in the repository** (only `screen.sh` writes in the repo, under `screens/`):

| File                                              | Written by                     | Content                                                                                                                |
| ------------------------------------------------- | ------------------------------ | ---------------------------------------------------------------------------------------------------------------------- |
| `report.txt`                                      | all helpers                    | timeline: BEGIN, LOCK, BUILD, LAUNCHED, WINDOW, READY, each capture, JOURNEY verdict, FINGERPRINT, CLEANUP             |
| `run.env`                                         | launch.sh, vm.sh               | the ledger: every resource the run started (pid:starttime), DISPLAY, SCRATCH, DEB, INSTALLER, APP_EXE, COMMIT, CLEANED |
| `build.log`, `app.log`, `xvfb.log`, `app.env`     | launch.sh                      | build output, app stdout/stderr, the app's environment (D-Bus address excluded)                                        |
| `<name>.png`, `<name>.state.txt`, `<name>.log`    | shot.sh, evidence.sh, journeys | captures, isolated state, log lines since a mark                                                                       |
| `fingerprint-before.txt`, `fingerprint-after.txt` | launch.sh, cleanup.sh          | sha256 of Pierre's real `history.db` and `settings_store.json`                                                         |
| `watchdog.log`                                    | the watchdog                   | its cleanup output, when it fired                                                                                      |
| `pipewire.log`, `cli.log`                         | launch.sh --mic, cli.sh        | the private PipeWire's output, the second instances' stdout (Info level)                                               |

Proof standards: drive the real user path (clicks and keys on the window), not a Tauri command or a settings file edit; capture the screen before and after the action and the log line that proves it; check the side effect too (the model files on disk, the setting written). A screenshot without its log line is a picture, not a proof. After Cleanup, `ls <run-dir>`: the evidence must still be there.

## Cleanup

```bash
.claude/skills/verify-dictus-desktop/helpers/cleanup.sh             # this session's run
.claude/skills/verify-dictus-desktop/helpers/cleanup.sh <run-dir>
```

Undoes exactly what the ledger lists, in order: the app's process group, the private PipeWire, the private D-Bus, Xvfb, the HTTP server, the VM (destroy, then revert to `clean-no-vcredist`), the test-voices corpus clone (plus any clone older than the TTL left by a dead run), the scratch dir (isolated HOME/XDG, extracted .deb, models, `history.db`, recordings, installers), the watchdog, and last the lock holder. A process is killed only if its pid still has the start time recorded when the run started it: never by name, never another holder of the lock. It then compares the fingerprint and prints `CLEANUP OK run=<dir> evidence=<n>` (exit 0), `CLEANUP INCOMPLETE: <what is left>` (exit 1), or `CLEANUP OK` then `ALARM: real data changed` (exit 2: stop and report to Pierre). Safe to run twice. `launch.sh` runs it on any failure (trap), `run.sh` on every exit, the watchdog after the TTL.

Then check: `flock -n /tmp/dictus-gradle.lock true && echo free` (unless someone else took it since), and `ls /tmp/.X*-lock` shows no display of yours.

## Tier 2: Windows VM

Domain `dictus-win` (4 GB) on `qemu:///system`, snapshots `clean-no-vcredist` (clean Windows 11, no Visual C++ runtime) and `dictus-0.3.0-ready` (with Dictus 0.3.0). **Never touch the domain `windows`.** Only after a green tier 1, only with the lock free, under 20 min. The guest agent runs as SYSTEM: a silent NSIS install as SYSTEM lands in the system profile and a SYSTEM process has no desktop, so the NSIS install and every launch go through a scheduled task running as user `dictus`, interactive. The MSI is the exception: it installs per machine into `C:\Program Files\Dictus`, which needs admin rights that a task as `dictus` does not have (limited token), so `msiexec /i … /qn /norestart` runs as SYSTEM through the guest agent and Dictus is then launched as `dictus`.

```bash
H=.claude/skills/verify-dictus-desktop/helpers
$H/vm.sh start                  # new tier-2 run, lock + watchdog, revert clean-no-vcredist, start, guest-ping
$H/vm.sh fetch 74               # NSIS installer of PR #74 from pr-test-build.yml (gh run download)
#   or: $H/vm.sh fetch-release <release-id> [nsis|msi]   # a draft's x64 installer, by asset id
$H/vm.sh serve                  # http.server bound to 192.168.122.1 only
$H/vm.sh install                # download in the guest; NSIS /S as dictus or msiexec /qn as SYSTEM; wait for the end
$H/vm.sh launch                 # start as dictus, wait 30 s, Get-Process dictus
$H/shot.sh 01-vm-dictus         # virsh screenshot (the whole VM screen: check it shows nothing private)
$H/cleanup.sh                   # destroy + revert to clean-no-vcredist, stop HTTP, delete installers, lock
```

`pr-test-build.yml` is `workflow_dispatch` with a PR number; its Windows x64 artifact is `handy-pr-<pr>-x86_64-pc-windows-msvc` (NSIS `*-setup.exe` and MSI), kept 30 days. `vm.sh fetch` refuses when no successful run has that artifact: dispatching the workflow is a CI action, ask first. `vm.sh ps '<powershell>'` runs any check in the guest (`vm-exec.py`, from Pierre's `gexec.py`).

`vm.sh fetch-release <release-id> [nsis|msi]` takes a release, drafts included, and downloads its one Windows x64 asset (`*_x64-setup.exe`, the default, or `*_x64_*.msi`) by asset id through the API: `gh release download` looks up a tag, and a draft has none yet. The release id comes from `gh api repos/getdictus/dictus-desktop/releases --jq '.[] | "\(.id) \(.name) draft=\(.draft)"'`. It checks the size against the API and logs the sha256 in `report.txt`. One installer per run: for both types, run `start`…`cleanup` twice so each starts from the clean snapshot.

`vm.sh install` reports `INSTALLED` only once the installer has finished. For NSIS it waits for the `DictusVerifyInstall` task to reach `State` Ready with `LastTaskResult` 0 (5 min max), not for `dictus.exe` to exist: on 2026-11-19 the file was there 12 s after the start and the task ended at 24 s, and a launch in between fails with `0x80070020` (sharing violation). Any other result stops it with the task's state and result. For MSI it fails on any `msiexec` exit code other than 0 or 3010; the verbose log is `C:\Users\Public\dictus-msi.log` in the guest. `run.env` keeps `APP_EXE`, the installed `dictus.exe`, for `launch`; `vm-install.txt` in the run dir has the timings.

`vm.sh launch` fails when no `dictus` process runs 30 s after the start and prints the `DictusVerifyLaunch` task's `State` and `LastTaskResult` (for example `Ready 0x80070002`, file not found). Guest output ends lines with CR: strip it before matching (`tr -d '\r'`).

Proven on 2026-11-19 with the 0.3.1 draft (release 451105484) from `clean-no-vcredist`: NSIS and MSI each fetch → serve → install → launch → RUNNING in about 1 min 30 s of VM time, version 0.3.1, onboarding window on screen.

## Helpers

All executable, run from the repository root, all read `helpers/env.sh` (overrides: `DICTUS_EVIDENCE_ROOT`, `DICTUS_IDENTIFIER`, `DICTUS_TTL_MIN`, `DICTUS_LOCK_WAIT_S`, `DICTUS_VERIFY_SESSION`, `DICTUS_VERIFY_RUN`, `CARGO_TARGET_DIR`, `DICTUS_HTTP_PORT`, `DICTUS_MODEL_CACHE`, `DICTUS_CORPUS_REPO`, `DICTUS_WER_MAX`).

| Script                  | Invocation                                                                  | Ends with                                                                                 |
| ----------------------- | --------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| `doctor.sh`             | `doctor.sh [run-dir]`                                                       | `DOCTOR OK` / `DOCTOR FAIL <n>`                                                           |
| `build.sh`              | `build.sh [--release] [log]`                                                | `DEB <path>` / `BUILD FAILED: …`                                                          |
| `launch.sh`             | `launch.sh [--no-build] [--release] [--deb f] [--model id] [--mic] [label]` | `READY run=… display=… pid=… window=… geometry=…`                                         |
| `run.sh`                | `run.sh <journey> [launch options] [-- journey args]`                       | journey verdict, then `CLEANUP OK …`                                                      |
| `journey-onboarding.sh` | `journey-onboarding.sh [run-dir]`                                           | `JOURNEY onboarding PASS in Ns` / `… FAIL: <step>`                                        |
| `journey-dictation.sh`  | `journey-dictation.sh [--reps N] [<clip-id>\|all-<lang>]`                   | per clip `JOURNEY dictation <id> PASS\|FAIL wer=…`, then `… all-fr PASS n/n`              |
| `journey-paste.sh`      | `journey-paste.sh [<clip-id>]`                                              | per target `JOURNEY paste <terminal\|gui> … insert_ms=… next_rec_ms=…`, then `… PASS 2/2` |
| `paste.py`              | `paste.py counter\|count\|settle\|wait-log\|chars\|window …`                | counts and times; never a text                                                            |
| `model-cache.sh`        | `model-cache.sh <model-id>`                                                 | `MODEL <id> <file> <name>`                                                                |
| `bench.sh`              | `bench.sh [--models a,b] [--clips sel] [--reps N] [--deb f\|--debug]`       | `BENCH <dir> models=N failed=N`                                                           |
| `bench.py`              | `bench.py collect <bench-dir> <run>…` · `compare <a.csv> <b.csv>`           | `BENCH TABLES …` / a Markdown table                                                       |
| `mic.py`                | `mic.py <scratch> conf\|check\|link [s]\|play <wav>`                        | `MIC OK …`, `MIC LINKED …`, `MIC PLAYED …`                                                |
| `cli.sh`                | `cli.sh <dictus flags>`                                                     | `CLI <flags> exit=N in Nms`                                                               |
| `dictation.py`          | `dictation.py clips\|last-id\|wait-row\|score\|transcribe-ms\|leak-check …` | ids, scores, timings; never a text                                                        |
| `x.py`                  | `x.py window\|click\|move\|scroll\|type\|key …`                             | `click X,Y button=1`, …                                                                   |
| `nav.sh`                | `nav.sh <section> [run-dir]`                                                | `nav <section>`                                                                           |
| `log.sh`                | `log.sh mark` · `wait '<re>' [s] [mark]` · `since <mark>`                   | `found: <line>` / `error: no log line …`                                                  |
| `shot.sh`               | `shot.sh <name> [run-dir]`                                                  | `shot <name>: <path>`                                                                     |
| `evidence.sh`           | `evidence.sh <name> [run-dir]`                                              | `shot …`, `state <name>: <path>`                                                          |
| `screen.sh`             | `screen.sh <screen-id> [run-dir]`                                           | `screen <id>: screens/<id>.png (N KB)`                                                    |
| `vm.sh`                 | `vm.sh start\|fetch\|fetch-release\|serve\|install\|launch\|ps`             | `VM READY`, `INSTALLER`, `SERVING`, `INSTALLED`, `RUNNING`                                |
| `vm-exec.py`            | `vm-exec.py dictus-win '<powershell>' [s]`                                  | guest output, `[exit N]`                                                                  |
| `cleanup.sh`            | `cleanup.sh [run-dir]`                                                      | `CLEANUP OK run=… evidence=N` / `CLEANUP INCOMPLETE: …` / `ALARM …`                       |

## Keeping the map current

**A PR that adds or changes a user-facing feature or screen MUST update `features/` and `features/screens.md` in the same PR**, with new captures from `screen.sh` when a screen changes. Reuse the iOS and Android IDs (`getdictus/dictus-ios` and `getdictus/dictus-android`, `.claude/skills/verify-dictus-*/features/`) for the same feature; a feature only the desktop has gets a new ID marked _desktop-only_. Run `/pstack:maintain-verification-skill` for a full pass over the map; a user has to invoke it.
