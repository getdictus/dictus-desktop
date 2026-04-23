---
created: 2026-04-15T07:00:17.100Z
title: Fix macOS "Dictus quit unexpectedly" dialog on clean shutdown
area: general
files:
  - src-tauri/src/lib.rs:254-255
  - src-tauri/src/lib.rs:622-623
  - src-tauri/src/tray.rs:129-130
---

## Problem

macOS displays the "Dictus quit unexpectedly — Reopen / Report / Ignore" crash dialog every time the app quits cleanly. Reproduced on:
- Tray icon → "Quit Dictus" menu item
- Post-auto-updater relaunch (update v0.1.0 → v0.1.1 cycle)

The quit handler calls `app.exit(0)` via Tauri (tray.rs → lib.rs:255 for tray quit, lib.rs:623 for no-tray close). Main exit is clean and intentional — the crash must come from a background thread panicking/being SIGKILLed during shutdown.

**Pre-existing upstream bug.** User confirmed the same dialog appears on Handy (the fork source), so it is not a v1.1 regression. Non-blocking — the app does quit, the dialog is cosmetic annoyance.

Observed on Dictus v0.1.1, macOS Sequoia 15.x (Apple Silicon M4 Pro).

## Solution

**Likely suspects to investigate:**
1. Background thread panic during shutdown — audio recording loop, VAD worker, Whisper/Parakeet model unload, HTTP updater client
2. Metal/GPU context teardown — Whisper/Parakeet GPU contexts not released cleanly
3. SIGKILL timeout — a thread not responding to termination signal, macOS force-kills after grace period
4. Global shortcut unhook — `tauri-plugin-global-shortcut` cleanup on drop
5. Single-instance plugin socket cleanup on macOS

**Diagnosis steps:**
1. Open Console.app → User Reports → Dictus → read the most recent crash report
2. Identify the crashing thread's stack trace (main vs worker)
3. Reproduce under `lldb -- /Applications/Dictus.app/Contents/MacOS/Dictus` and trigger Quit; inspect where the process dies after exit(0)
4. Check `RUST_BACKTRACE=1` logs around shutdown (Dictus log path on macOS)
5. Try disabling plugins one by one (updater, global-shortcut, single-instance, store) to isolate the offender

**Possible fixes:**
- Add explicit drop order for managers in `lib.rs` setup closure
- Call `std::process::exit(0)` (hard exit, bypass destructors) if the offender cannot be fixed upstream — tradeoff is losing graceful resource cleanup
- File upstream issue on cjpais/Handy if confirmed as shared bug
