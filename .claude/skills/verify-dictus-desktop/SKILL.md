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

| Tier                   | Who and where                                                     | Proves                                                                                                                                                                                                  | Does not prove                                                                                                                                                                 |
| ---------------------- | ----------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1. Linux, Xvfb         | An agent with this skill on this machine                          | The app builds, packages (.deb) and starts from an empty data dir; UI, navigation, onboarding, model download/verify/extract/load, settings persistence, the log; every screen of `features/screens.md` | Audio (no microphone or speaker in the isolated session), a real desktop (no window manager, no tray host, no Wayland), macOS and Windows code paths, the signed release build |
| 2. Windows VM          | An agent, `dictus-win` on this host                               | The CI installer installs per user and starts on a clean Windows 11, the first window renders, missing-runtime crashes (#72)                                                                            | Audio, a real GPU, Windows on ARM, long sessions (the evaluation ISO is expired: keep runs short)                                                                              |
| 3. macOS, Pierre's Mac | An agent in a T3 thread on the Mac, macOS-specific behaviour only | Permissions (microphone, accessibility), Metal, the overlay, the menu bar, macOS paste                                                                                                                  | Anything tier 1 can prove: do it there first                                                                                                                                   |
| 4. Pierre              | Pierre, in a batch test                                           | Real voice, transcription quality, the feel of shortcuts, paste and sounds in his own apps                                                                                                              | Nothing is merged before his OK                                                                                                                                                |

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
$H/launch.sh --deb <file.deb>      # a given .deb (e.g. a CI artifact)
$H/run.sh onboarding [--no-build]  # Launch + Doctor + journey-onboarding.sh + Cleanup, in one process
```

`launch.sh`, in order:

1. Creates the run dir `/home/pierre/OpenBot/Shared/pilotage/dictus-desktop/preuves/verify-dictus-desktop/<YYYYMMDD-HHMMSS>-tier1-<label>-<sha>/`, its ledger `run.env`, and `fingerprint-before.txt`. Refuses if this session has a run not yet cleaned up.
2. Unless `--no-build`/`--deb`: `build.sh` runs `bun install --frozen-lockfile` (if `node_modules` is missing) and `bun run tauri build --debug --bundles deb` under `flock /tmp/dictus-gradle.lock`, log in `build.log`. Measured on 2026-11-17: 294 s cold in a fresh worktree. The command then exits 1 on the updater signing step (`A public key has been found, but no private key`): expected without `TAURI_SIGNING_PRIVATE_KEY`, never add a key. A .deb newer than the build start is the success criterion.
3. Takes the lock for the app phase (waits up to 30 min; `DICTUS_LOCK_WAIT_S`), arms the watchdog.
4. `dpkg-deb -x` into `/tmp/verify-dictus-desktop.XXXXXX/pkg` and runs `pkg/usr/bin/dictus`. The raw `target/debug/dictus` panics on missing resources (#71).
5. Starts `Xvfb :N` (first free in `:120`–`:199`, never `:99`/`:100`, 1280x900), a private `dbus-daemon --session`, then Dictus with `env -i`: `HOME` and every `XDG_*` dir under the scratch dir, `LANG=en_US.UTF-8` (English labels), `GDK_BACKEND=x11`, `WEBKIT_DISABLE_DMABUF_RENDERER=1`. The private D-Bus also isolates the single-instance check (D-Bus on Linux), so a running Dictus of Pierre's is never handed the launch.

**Ready means** the `Dictus` window is mapped and the process is still alive 3 s later. The window appears in about 4 s (the ~25 s seen in #71 did not happen with the private D-Bus). Last line, the only one to rely on:

```
READY run=<run-dir> display=:120 pid=<pid> window=0x200003 geometry=680x570+0+0
```

Every helper after Launch finds the run on its own (session pointer under `…/verify-dictus-desktop/sessions/`). Two agents sharing one checkout each export a distinct `DICTUS_VERIFY_SESSION`.

**After PR #75 merges** (dev identity `com.dictus.desktop.dev`, product name "Dictus Dev"): debug builds stop sharing the release identity. Prefer the dev identity then: run with `DICTUS_IDENTIFIER=com.dictus.desktop.dev` (the isolated data dir, `evidence.sh` and `log.sh` follow it); `x.py` already matches the title `Dictus Dev`. Keep the isolated HOME anyway: it is what makes onboarding reachable from an empty data dir.

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

**The recipe this skill is proven on**: feature `onboarding`, 2026-11-17, run `20261117-215926-tier1-onboarding-92ded2a` (PASS in 43 s with the build), see the run report `/home/pierre/OpenBot/Shared/pilotage/dictus-desktop/preuves/2026-11-17-verify-dictus-desktop-tier1-onboarding.md`.

```bash
H=.claude/skills/verify-dictus-desktop/helpers
$H/run.sh onboarding        # or, step by step:
$H/launch.sh onboarding && $H/doctor.sh && $H/journey-onboarding.sh; $H/cleanup.sh
```

Every other feature's recipe is in `features/<id>.md`, under "Driving it with x.py".

Rules for every feature:

- One journey per run, then Cleanup. Do not leave the app up between steps of your own reasoning: the watchdog will clean it up after 20 min anyway.
- Download only the model the journey needs. Moonshine V2 Tiny (31 MB) is the default; it lands in the scratch dir and Cleanup deletes it.
- There is no audio device in the isolated session: anything that records or plays sound is reported as not driven on tier 1, with that reason.

## Evidence

```bash
$H/shot.sh 01-before          # the Dictus window only (ffmpeg x11grab cropped to the window)
$H/evidence.sh 02-after       # screenshot + <name>.state.txt: models on disk, key settings, last log lines
$H/screen.sh app.settings.models   # a screen-map capture: screens/<id>.png in this skill (tier 1 only)
```

Everything for a run lands in `/home/pierre/OpenBot/Shared/pilotage/dictus-desktop/preuves/verify-dictus-desktop/<run-id>/`, **never in the repository** (only `screen.sh` writes in the repo, under `screens/`):

| File                                              | Written by                     | Content                                                                                                    |
| ------------------------------------------------- | ------------------------------ | ---------------------------------------------------------------------------------------------------------- |
| `report.txt`                                      | all helpers                    | timeline: BEGIN, LOCK, BUILD, LAUNCHED, WINDOW, READY, each capture, JOURNEY verdict, FINGERPRINT, CLEANUP |
| `run.env`                                         | launch.sh, vm.sh               | the ledger: every resource the run started (pid:starttime), DISPLAY, SCRATCH, DEB, COMMIT, CLEANED         |
| `build.log`, `app.log`, `xvfb.log`, `app.env`     | launch.sh                      | build output, app stdout/stderr, the app's environment (D-Bus address excluded)                            |
| `<name>.png`, `<name>.state.txt`, `<name>.log`    | shot.sh, evidence.sh, journeys | captures, isolated state, log lines since a mark                                                           |
| `fingerprint-before.txt`, `fingerprint-after.txt` | launch.sh, cleanup.sh          | sha256 of Pierre's real `history.db` and `settings_store.json`                                             |
| `watchdog.log`                                    | the watchdog                   | its cleanup output, when it fired                                                                          |

Proof standards: drive the real user path (clicks and keys on the window), not a Tauri command or a settings file edit; capture the screen before and after the action and the log line that proves it; check the side effect too (the model files on disk, the setting written). A screenshot without its log line is a picture, not a proof. After Cleanup, `ls <run-dir>`: the evidence must still be there.

## Cleanup

```bash
.claude/skills/verify-dictus-desktop/helpers/cleanup.sh             # this session's run
.claude/skills/verify-dictus-desktop/helpers/cleanup.sh <run-dir>
```

Undoes exactly what the ledger lists, in order: the app's process group, the private D-Bus, Xvfb, the HTTP server, the VM (destroy, then revert to `clean-no-vcredist`), the scratch dir (isolated HOME/XDG, extracted .deb, downloaded models and installers), the watchdog, and last the lock holder. A process is killed only if its pid still has the start time recorded when the run started it: never by name, never another holder of the lock. It then compares the fingerprint and prints `CLEANUP OK run=<dir> evidence=<n>` (exit 0), `CLEANUP INCOMPLETE: <what is left>` (exit 1), or `CLEANUP OK` then `ALARM: real data changed` (exit 2: stop and report to Pierre). Safe to run twice. `launch.sh` runs it on any failure (trap), `run.sh` on every exit, the watchdog after the TTL.

Then check: `flock -n /tmp/dictus-gradle.lock true && echo free` (unless someone else took it since), and `ls /tmp/.X*-lock` shows no display of yours.

## Tier 2: Windows VM

Domain `dictus-win` (4 GB) on `qemu:///system`, snapshots `clean-no-vcredist` (clean Windows 11, no Visual C++ runtime) and `dictus-0.3.0-ready` (with Dictus 0.3.0). **Never touch the domain `windows`.** Only after a green tier 1, only with the lock free, under 20 min. The guest agent runs as SYSTEM: a silent NSIS install as SYSTEM lands in the system profile and a SYSTEM process has no desktop, so install and launch go through a scheduled task running as user `dictus`, interactive.

```bash
H=.claude/skills/verify-dictus-desktop/helpers
$H/vm.sh start                  # new tier-2 run, lock + watchdog, revert clean-no-vcredist, start, guest-ping
$H/vm.sh fetch 74               # installer of PR #74 from pr-test-build.yml (gh run download)
$H/vm.sh serve                  # http.server bound to 192.168.122.1 only
$H/vm.sh install                # download in the guest, NSIS /S as dictus, wait for dictus.exe
$H/vm.sh launch                 # start as dictus, wait 30 s, Get-Process dictus
$H/shot.sh 01-vm-dictus         # virsh screenshot (the whole VM screen: check it shows nothing private)
$H/cleanup.sh                   # destroy + revert to clean-no-vcredist, stop HTTP, delete installers, lock
```

`pr-test-build.yml` is `workflow_dispatch` with a PR number; its Windows x64 artifact is `handy-pr-<pr>-x86_64-pc-windows-msvc` (NSIS `*-setup.exe` and MSI), kept 30 days. `vm.sh fetch` refuses when no successful run has that artifact: dispatching the workflow is a CI action, ask first. `vm.sh ps '<powershell>'` runs any check in the guest (`vm-exec.py`, from Pierre's `gexec.py`).

Not yet proven end to end in this skill: `vm.sh` was written from Pierre's notes and has not run (2026-11-17). Its first run is the proof; fix what fails.

## Helpers

All executable, run from the repository root, all read `helpers/env.sh` (overrides: `DICTUS_EVIDENCE_ROOT`, `DICTUS_IDENTIFIER`, `DICTUS_TTL_MIN`, `DICTUS_LOCK_WAIT_S`, `DICTUS_VERIFY_SESSION`, `DICTUS_VERIFY_RUN`, `CARGO_TARGET_DIR`, `DICTUS_HTTP_PORT`).

| Script                  | Invocation                                                | Ends with                                                           |
| ----------------------- | --------------------------------------------------------- | ------------------------------------------------------------------- |
| `doctor.sh`             | `doctor.sh [run-dir]`                                     | `DOCTOR OK` / `DOCTOR FAIL <n>`                                     |
| `build.sh`              | `build.sh [log]`                                          | `DEB <path>` / `BUILD FAILED: …`                                    |
| `launch.sh`             | `launch.sh [--no-build] [--deb f] [label]`                | `READY run=… display=… pid=… window=… geometry=…`                   |
| `run.sh`                | `run.sh <journey> [launch options]`                       | journey verdict, then `CLEANUP OK …`                                |
| `journey-onboarding.sh` | `journey-onboarding.sh [run-dir]`                         | `JOURNEY onboarding PASS in Ns` / `… FAIL: <step>`                  |
| `x.py`                  | `x.py window\|click\|move\|scroll\|type\|key …`           | `click X,Y button=1`, …                                             |
| `nav.sh`                | `nav.sh <section> [run-dir]`                              | `nav <section>`                                                     |
| `log.sh`                | `log.sh mark` · `wait '<re>' [s] [mark]` · `since <mark>` | `found: <line>` / `error: no log line …`                            |
| `shot.sh`               | `shot.sh <name> [run-dir]`                                | `shot <name>: <path>`                                               |
| `evidence.sh`           | `evidence.sh <name> [run-dir]`                            | `shot …`, `state <name>: <path>`                                    |
| `screen.sh`             | `screen.sh <screen-id> [run-dir]`                         | `screen <id>: screens/<id>.png (N KB)`                              |
| `vm.sh`                 | `vm.sh start\|fetch\|serve\|install\|launch\|ps`          | `VM READY`, `INSTALLER`, `SERVING`, `INSTALLED`, `RUNNING`          |
| `vm-exec.py`            | `vm-exec.py dictus-win '<powershell>' [s]`                | guest output, `[exit N]`                                            |
| `cleanup.sh`            | `cleanup.sh [run-dir]`                                    | `CLEANUP OK run=… evidence=N` / `CLEANUP INCOMPLETE: …` / `ALARM …` |

## Keeping the map current

**A PR that adds or changes a user-facing feature or screen MUST update `features/` and `features/screens.md` in the same PR**, with new captures from `screen.sh` when a screen changes. Reuse the iOS and Android IDs (`getdictus/dictus-ios` and `getdictus/dictus-android`, `.claude/skills/verify-dictus-*/features/`) for the same feature; a feature only the desktop has gets a new ID marked _desktop-only_. Run `/pstack:maintain-verification-skill` for a full pass over the map; a user has to invoke it.
