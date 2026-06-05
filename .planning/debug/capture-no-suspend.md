---
status: diagnosed
trigger: "Test 7 UAT: assigning an already-used shortcut to a 2nd card — during capture the EXISTING shortcut fires (launches transcription) instead of being captured; no conflict shown, no assignment. True for single key (Command alone) and combos. Global shortcuts are not suspended during capture."
created: 2026-06-05T13:10:00Z
updated: 2026-06-05T13:30:00Z
mode: find_root_cause_only
---

## Current Focus

hypothesis: SmartModeShortcutChip captures keys via webview keydown and only suspends ITS OWN binding (not all globals), so other modes' OS-level shortcuts keep firing during capture.
test: Compare chip capture path vs the app's working ShortcutInput (GlobalShortcutInput / HandyKeysShortcutInput) and the backend suspend mechanism.
expecting: Confirmed — chip never suspends the conflicting binding, and on handy-keys it doesn't even use the backend recording path.
next_action: Return diagnosis (find_root_cause_only).

## Symptoms

expected: While a chip is recording, pressing an already-bound combo is captured by the recording UI (then conflict-detected), NOT fired as the existing global shortcut.
actual: The existing global shortcut fires (starts a transcription). No combo captured, no conflict shown, no assignment.
errors: None (existing shortcut fires instead of capture).
reproduction: UAT Test 7 — bind Command+4 to card A; on card B click chip and press Command+4 (or Command alone); the card-A shortcut fires.
started: Discovered during Phase 13 UAT re-test. The 13-03 chip used a "best-effort suspend/resume (catch(()=>{}))".

## Eliminated

- hypothesis: The handy-keys recording listener blocks/suppresses OS events, so the chip just needs to call the same listener.
  evidence: handy-keys 0.2.4 — start_recording uses `KeyboardListener::new()` which is the NON-blocking constructor ("Events are observed but not blocked"). The blocking variant is `new_with_blocking`. So even the working path does not rely on the recording listener to suppress events — it relies on suspending/unregistering the global hotkey first (and on capturing via a backend stream that fires regardless of window focus).
  timestamp: 2026-06-05T13:25:00Z

## Evidence

- timestamp: 2026-06-05T13:15:00Z
  checked: SmartModeShortcutChip.tsx handleClick (line 183-195) and commitCombo (70-99)
  found: On record start it calls `commands.suspendBinding("smart_mode_" + modeId)` ONLY, and ONLY when `currentBinding` is truthy (the chip being edited is already bound). It NEVER suspends any other mode's binding nor the legacy transcribe binding. Capture is done purely via `window.addEventListener("keydown"/"keyup")` in the webview.
  implication: All OTHER global shortcuts remain registered and live at the OS level during capture, so pressing an already-used combo triggers that other shortcut's action (transcription) before/instead of the webview capture.

- timestamp: 2026-06-05T13:16:00Z
  checked: backend suspend_binding (shortcut/mod.rs:215-223)
  found: suspend_binding unregisters exactly ONE binding by id. There is NO "suspend all" / "disable all global shortcuts" command. (One exists internally — `unregister_all_shortcuts` at mod.rs:357 — but it is private and only used by keyboard-implementation switching, not exposed as a Tauri command.)
  implication: The chip has no backend primitive to silence the conflicting binding even if it wanted to; the only command-level tool is per-id suspend.

- timestamp: 2026-06-05T13:18:00Z
  checked: GlobalShortcutInput.tsx startRecording (line 180-191) and HandyKeysShortcutInput.tsx startRecording (173-191) + ShortcutInput.tsx dispatch
  found: The app's existing single-shortcut editor edits ONE binding at a time and suspends THAT binding before recording. It does not need to suspend others because the app's legacy UX edits a fixed, known set of distinct bindings (transcribe / cancel / etc.) that don't collide during a single edit. Crucially, on macOS the active impl is typically handy_keys, and HandyKeysShortcutInput captures keys via the BACKEND event stream `handy-keys-event` (commands.startHandyKeysRecording), which fires regardless of webview focus and reports the raw combo — not via webview keydown.
  implication: Two architectural differences vs the chip: (1) the working editor suspends the binding it is editing; (2) on handy-keys it captures via the backend recording stream, not webview keydown. The chip uses webview keydown on ALL implementations and suspends almost nothing.

- timestamp: 2026-06-05T13:20:00Z
  checked: handy_keys.rs start_recording (263-299), recording_loop (302-335), and lib.rs doc ("Hotkey blocking: Registered hotkeys are blocked from reaching other applications")
  found: Registered handy-keys hotkeys are blocked from other apps but are still DELIVERED to Dictus's own handler (manager_thread → handle_shortcut_event) — i.e. they still FIRE their action. start_recording does NOT unregister/suspend the existing global hotkeys; it just spins up a parallel non-blocking KeyboardListener and emits its events to the frontend. So a registered combo pressed during recording would BOTH fire its action AND be observed by the listener.
  implication: The working HandyKeysShortcutInput avoids the existing shortcut firing during a single-binding edit ONLY because it suspends the one binding it edits and (in the app's legacy flows) there is no second live binding equal to what's being pressed. The handy-keys recording path is NOT a general "all globals suspended" mechanism. For smart-mode conflict capture, the chip would still fire any OTHER mode's combo even if it adopted the backend recording path, unless ALL smart-mode/global bindings are suspended for the duration.

- timestamp: 2026-06-05T13:22:00Z
  checked: tauri_impl.rs register_shortcut (86-143)
  found: Tauri impl registers an OS global shortcut with an on_shortcut handler. Same conclusion: a registered combo fires its handler during capture unless unregistered.
  implication: Bug is implementation-agnostic. On both Tauri and handy-keys, the only reliable way to capture an already-bound combo is to suspend ALL relevant global bindings while recording.

## Resolution

root_cause: |
  SmartModeShortcutChip does not suspend the OTHER (conflicting) global shortcuts while
  recording, so the already-bound combo fires its existing action instead of being captured.

  Two concrete defects:
  1. Wrong scope of suspend. On record start (handleClick, SmartModeShortcutChip.tsx:191-193)
     the chip calls `suspendBinding("smart_mode_" + modeId)` and ONLY when the chip being
     edited is already bound. It never suspends the binding the user is about to collide with
     (or any other global binding / the legacy transcribe binding). Every other OS-level
     shortcut stays live, so pressing an already-used combo (e.g. Command+4, or Command alone)
     triggers THAT mode's transcription action before the conflict can be detected. No backend
     "suspend all global shortcuts" command exists at the command layer to call — only the
     private `unregister_all_shortcuts` used during implementation switching.
  2. Wrong capture channel on handy-keys. The chip always captures via webview
     `window.addEventListener("keydown")`, which (a) only fires while the Dictus window has
     focus and (b) does not stop the OS-level hotkey from firing. The app's working editor
     (HandyKeysShortcutInput) instead drives the backend recording stream
     (startHandyKeysRecording → `handy-keys-event`) when the active implementation is
     handy_keys, and it suspends the single binding it is editing. The chip adopted neither
     behaviour.

  Why GlobalShortcutInput/HandyKeysShortcutInput "work": they edit ONE known binding at a
  time and suspend exactly that binding before recording; the app's legacy flows never put a
  second live binding equal to the keys being pressed, so nothing else fires. That assumption
  breaks for Smart Modes, where the whole point of conflict detection is that the user presses
  a combo that ANOTHER mode currently owns and that other binding is still live.

  Note: suspending all bindings is necessary but NOT sufficient for the handy-keys capture
  case — once all globals are suspended, those keys no longer reach the chip via the OS
  hotkey handler either, so on handy_keys the chip must capture via the backend recording
  stream (or accept webview-keydown capture with the app focused). On the Tauri impl,
  webview keydown capture works once the globals are unregistered.

fix: ""  # find_root_cause_only — not applied

verification: ""

files_changed: []

suggested_fix_direction: |
  - Add a command-level "suspend all global shortcuts" / "resume all" pair (expose the existing
    private `unregister_all_shortcuts` logic in shortcut/mod.rs as Tauri commands, plus a
    re-register-all counterpart). Call suspend-all on chip record start and resume-all on
    commit/cancel/click-outside/unmount. This is the minimal reliable fix for the "existing
    shortcut fires" symptom on BOTH implementations.
  - Keep conflict detection where it already is: set_smart_mode_binding → change_binding can
    report the conflict; with globals suspended the pressed combo is captured and the conflict
    surfaces in the chip's `conflict` state instead of firing.
  - For full handy-keys correctness, route chip capture through the backend recording stream
    (startHandyKeysRecording / handy-keys-event) when keyboard_implementation === "handy_keys",
    mirroring HandyKeysShortcutInput, since after suspend-all the OS hotkeys no longer reach the
    webview as fired actions but the chip's webview keydown also won't see them reliably unless
    the window is focused. On the Tauri impl, webview keydown after suspend-all is sufficient.
  - Ensure resume-all runs on every exit path (commit success, conflict, click-outside at
    chip.tsx:158-169, and effect cleanup at 175-181) so a crash/early-return during recording
    can't leave all globals dead.
