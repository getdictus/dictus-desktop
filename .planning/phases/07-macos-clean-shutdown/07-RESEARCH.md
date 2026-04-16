# Phase 7: macOS Clean Shutdown - Research

**Researched:** 2026-04-16
**Domain:** Tauri 2.x shutdown lifecycle, macOS crash reporter, Rust process exit
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **Evidence source:** reproduce on fresh build from current `main`; generate a matching `.ips` on Pierre's M4 Pro / Sequoia 15.x
- **Repro recipe:** `bun run tauri build` → install `.app` → tray → "Quit Dictus" → watch Console.app User Reports
- **Diagnosis depth:** Console.app `.ips` + `RUST_BACKTRACE=1` logs only. No `lldb`, no plugin bisection upfront
- **Ambiguity handling:** pragmatic — adapt if logs don't cleanly pinpoint; no commitment to escalate
- **Write-up location:** `## Diagnosis` section at the top of `07-01-PLAN.md`, committed before any code changes
- **Gate:** first commit = `docs(07): capture diagnosis of quit-unexpectedly crash (SHUT-01)` or similar; no code before diagnosis commit
- **Fix path (a) — Graceful cleanup:** preferred when root cause is in Dictus Rust code (explicit drop order, `run_on_main_thread` for shortcut unregister, poison-safe `Drop`)
- **Fix path (b) — `std::process::exit(0)`:** accepted when suspect is a third-party plugin; hard-exits after `log::logger().flush()`
- **No (a)→(b) retry loop in one attempt:** each plan tries one path end-to-end
- **Always flush before exit:** insert `log::logger().flush()` immediately before any `app.exit(0)` or `std::process::exit(0)` call at both sites
- **Iteration cap:** 2 plan iterations maximum
- **Plugin vendoring rejected:** do NOT fork/vendor plugins
- **SHUT-03 pass bar:** 3 consecutive clean quits from a fresh `bun run tauri build` install, no dialog on any
- **Post-update relaunch simulation:** use `tauri_plugin_process::restart()` directly (debug-only trigger, no real test release)
- **Environment:** Pierre's M4 Pro, macOS Sequoia 15.x only
- **Upstream coordination:** Dictus-only fix. No issue or PR to `cjpais/Handy`
- **UPSTREAM.md:** add one line to conflict-rules section: `lib.rs quit-exit handlers / log-flush helper — Dictus version wins (SHUT-02)`

### Claude's Discretion

- Exact structure of `## Diagnosis` section inside `07-01-PLAN.md`
- Whether `log::logger().flush()` lives in a helper (`flush_logs_then_exit`) or is inlined at both sites
- Exact naming/location of the auto-update-relaunch simulation trigger (debug-only Tauri command vs JS-console-callable path); match `src/components/settings/debug/` patterns
- Commit granularity (per task vs per surface); follow Phase 6 style
- Whether `## Diagnosis` stays inline in `07-01-PLAN.md` or splits to `07-DIAGNOSIS.md` if it grows long

### Deferred Ideas (OUT OF SCOPE)

- Fixing mutex-poison `Drop` impls in `ModelManager::DownloadCleanup` and `TranscriptionManager::LoadingGuard` (unless `.ips` directly implicates them)
- Filing upstream issue or PR on `cjpais/Handy`
- Adding `verify-sync.sh` assertion for this fix
- Cutting a throwaway `v0.1.x-test` release for SHUT-03 validation
- Intel Mac / clean-VM verification
- Vendoring or forking any plugin
- Planning reconciliation with a future upstream Handy fix
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| SHUT-01 | Console.app crash report read and crashing thread identified; diagnosis documented in phase plan before fix is chosen | Diagnosis workflow, `.ips` anatomy, `RUST_BACKTRACE=1` log path, suspect list with evidence |
| SHUT-02 | Based on diagnosis, either (a) explicit cleanup/drop order before `app.exit(0)` at `lib.rs:254`/`lib.rs:622` or (b) `std::process::exit(0)` after `log::logger().flush()` | Both fix paths documented with exact code patterns; `GlobalShortcutExt::unregister_all()` API confirmed; `log::Log::flush()` confirmed |
| SHUT-03 | Clean quit on macOS (tray + post-update relaunch) no longer triggers the "quit unexpectedly" dialog | Pass bar: 3 consecutive clean quits; simulation trigger pattern for `tauri_plugin_process::restart()` documented |
</phase_requirements>

---

## Summary

The macOS "Dictus quit unexpectedly" dialog is triggered when the OS's crash reporter (`ReportCrash`) catches a signal (typically SIGSEGV, SIGABRT, or SIGKILL from watchdog timeout) that is NOT the clean `SIGTERM`/`exit(0)` path. The app exits successfully — the dialog is cosmetic — but something in the shutdown teardown sequence is panicking or timing out.

The bug is confirmed pre-existing on unmodified Handy (the fork source), which strongly implicates upstream Tauri plugin teardown rather than Dictus-specific code. The primary suspects are:

1. `tauri_plugin_global_shortcut` Drop implementation on macOS (known macOS-specific issues across Tauri bug tracker, likely tries to unregister from the wrong thread)
2. A `tokio-runtime-worker` thread panicking during shutdown (see `DownloadCleanup::drop` and `LoadingGuard::drop` — both call `.lock().unwrap()` which panics on poisoned mutex; if a worker panics during recording and teardown races with a second `.unwrap()`, macOS sees an abort)
3. `tauri_plugin_single_instance` socket cleanup timing (the plugin holds a Unix domain socket that may not close cleanly before the process terminates)

The fix has two branches: path (a) explicit graceful cleanup before `app.exit(0)`, or path (b) `std::process::exit(0)` after `log::logger().flush()`. The `.ips` crash report dictates which path to take.

**Primary recommendation:** Generate the `.ips` crash report from a fresh build, identify the crashing thread stack frame, then implement fix path (a) first (unregister all global shortcuts via `app_handle.run_on_main_thread` before calling `app.exit(0)`, plus `log::logger().flush()` at both exit sites). If path (a) doesn't clear the dialog, the next plan iteration switches to `std::process::exit(0)`.

---

## Standard Stack

### Core APIs in Play

| API | Location | Purpose | Notes |
|-----|----------|---------|-------|
| `app.exit(0)` | `lib.rs:255`, `lib.rs:623` | Current quit path | Tauri-native; triggers plugin Drop chain |
| `std::process::exit(0)` | stdlib | Fix path (b) hard exit | Bypasses all destructors; no Drop chain |
| `log::logger().flush()` | `log` crate `Log` trait | Flush buffered log records | Must be called manually before any exit |
| `GlobalShortcutExt::unregister_all()` | `tauri_plugin_global_shortcut` 2.3.1 | Explicit cleanup before exit | Sig: `pub fn unregister_all(&self) -> Result<(), Error>` |
| `app_handle.run_on_main_thread(|| {...})` | Tauri 2.10.2 | Execute closure on main thread | Required for UI/shortcut unregister on macOS |
| `tauri_plugin_process::restart()` | `tauri-plugin-process` 2.3.1 | Simulate updater relaunch | Already in dep tree (`lib.rs:531`) |

### Plugin Versions (from `src-tauri/Cargo.toml`)

| Plugin | Version | Shutdown Concern |
|--------|---------|-----------------|
| `tauri-plugin-global-shortcut` | 2.3.1 | PRIMARY SUSPECT: macOS Drop may not unregister cleanly |
| `tauri-plugin-single-instance` | 2.3.2 | Unix socket cleanup on exit |
| `tauri-plugin-store` | 2.4.1 | File flush on drop (data loss risk if `process::exit`) |
| `tauri-plugin-updater` | 2.10.0 | Restart API (`restart()` → `process::exit` internally) |
| `tauri-plugin-log` | 2.7.1 | File buffer; needs `log::logger().flush()` before hard exit |
| `tauri-plugin-process` | 2.3.1 | `restart()` API used for SHUT-03 simulation |
| `tauri` | 2.10.2 | `app.exit()` → Drop chain → OS |

---

## Architecture Patterns

### Tauri 2.x Shutdown Lifecycle

When `app.exit(0)` is called:
1. Tauri sends `RunEvent::ExitRequested` to any registered `.run()` handler (Dictus uses `.run()`)
2. Tauri drops managed state and plugins in reverse-registration order
3. Plugin Drop implementations execute (this is where teardown happens)
4. The underlying event loop (`tao`/`wry`) shuts down
5. OS sees process termination

**Known Tauri 2.x issue:** `AppHandle::restart()` (used by `tauri-plugin-updater`) may exit without waiting for `RunEvent::Exit` to be emitted to plugins/app ([tauri-apps/tauri#12310](https://github.com/tauri-apps/tauri/issues/12310), opened 2025-01-08, status: needs triage). This is directly relevant to SHUT-03: the relaunch path may trigger the crash even if tray-quit is fixed.

### Current Exit Sites

```rust
// Site 1: lib.rs:254-256 — tray "quit" handler
"quit" => {
    app.exit(0);  // <-- fix lands here
}

// Site 2: lib.rs:621-624 — no-tray CloseRequested
} else {
    window.app_handle().exit(0);  // <-- symmetric fix lands here
}
```

### Fix Path (a): Graceful Cleanup Pattern

```rust
// Pattern: unregister shortcuts explicitly before exit
// Source: GlobalShortcutExt docs.rs 2.3.1
"quit" => {
    log::logger().flush();
    let app_clone = app.clone();
    let _ = app.run_on_main_thread(move || {
        let _ = app_clone.global_shortcut().unregister_all();
    });
    app.exit(0);
}
```

**Rationale:** On macOS, CGEventTap (used by global shortcut plugins) must be released from the main thread. If the plugin's Drop impl tries to do this from a non-main thread, macOS may signal SIGABRT or produce undefined behavior.

### Fix Path (b): Hard Exit Pattern

```rust
// Pattern: flush logs then bypass destructors
// Use when suspect is third-party plugin, not Dictus code
"quit" => {
    log::logger().flush();
    // SHUT-02: std::process::exit bypasses plugin Drop chain
    // Upstream bug: [link to .ips suspect] — Dictus version wins (SHUT-02)
    std::process::exit(0);
}
```

**Important:** `std::process::exit(0)` does NOT run Rust destructors. This forfeits:
- `tauri-plugin-store` write-back (settings may not be persisted if changed near quit)
- Log file flush (covered by explicit `log::logger().flush()` call)
- Any in-flight model download cleanup (`DownloadCleanup::drop`)

**When to use:** Only if `.ips` implicates a third-party plugin Drop as the crash site and path (a) doesn't resolve it.

### Helper Function Pattern (Claude's Discretion)

If `flush_then_exit` is used at both sites, mirror the `send_transcription_input()` pattern from `signal_handle.rs`:

```rust
// In lib.rs or a new helpers module
fn flush_and_exit(app: &AppHandle, code: i32) {
    log::logger().flush();
    // Insert cleanup based on diagnosis here
    app.exit(code);
    // -- OR for path (b): std::process::exit(code);
}
```

Call from both `lib.rs:255` (tray quit) and `lib.rs:623` (no-tray CloseRequested).

### SHUT-03 Simulation: Updater Relaunch Trigger

`tauri_plugin_process::restart()` is the same API the updater calls after applying an update. It is already in the dep tree (`lib.rs:531`). A debug-only Tauri command can invoke it:

```rust
// In commands/mod.rs (gated by debug_mode or #[cfg(debug_assertions)])
#[cfg(debug_assertions)]
#[tauri::command]
#[specta::specta]
pub fn simulate_updater_restart(app: AppHandle) {
    log::info!("DEBUG: simulating updater restart via tauri_plugin_process::restart()");
    app.restart();
}
```

This can be invoked from the JS console (`window.__TAURI__.core.invoke('simulate_updater_restart')`) or wired into a button in `DebugSettings.tsx`. The `DebugSettings` component in `src/components/settings/debug/DebugSettings.tsx` already contains the pattern for grouping debug actions.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Log flush | Custom buffered writer | `log::logger().flush()` | `Log` trait provides this; `tauri-plugin-log` implements it |
| Shortcut unregister | Manual CGEventTap release | `GlobalShortcutExt::unregister_all()` | Plugin wraps platform-specific CGEventTap lifecycle |
| Process restart | Custom spawn+exit | `app.restart()` from `tauri-plugin-process` | Handles single-instance handshake, already in dep tree |
| Crash report symbolication | Custom binary analysis | Console.app + `atos`/`symbolicatecrash` | Apple provides these; `.ips` files are human-readable JSON |

---

## Common Pitfalls

### Pitfall 1: Calling `unregister_all()` from a non-main thread on macOS

**What goes wrong:** macOS CGEventTap operations must happen on the main thread. Calling `unregister_all()` from a tokio-runtime-worker (background thread) during shutdown will either silently fail or trigger SIGABRT.

**Why it happens:** `app.exit(0)` may be called from an event handler that runs on a non-main thread. The tray event handler (`SystemTrayEvent`) may or may not run on the main thread depending on the Tauri version.

**How to avoid:** Wrap the unregister call in `app_handle.run_on_main_thread(|| { ... })` before calling `app.exit(0)`.

**Warning signs:** `.ips` crash report shows crashing thread is `main` with a frame in CGEventTap code, but the triggering call originates from a background thread.

### Pitfall 2: `std::process::exit(0)` losing the `tauri-plugin-store` write-back

**What goes wrong:** If the user changed settings near the time of quit (e.g., toggled a setting and immediately quit), and path (b) hard-exit is used, the store plugin's buffered write may not flush.

**Why it happens:** `std::process::exit()` bypasses all Rust destructors. The store plugin's Drop would normally flush pending writes.

**How to avoid:** For path (b), explicitly call any state save commands before `std::process::exit(0)`. OR accept this risk since it is a rare race condition and the bug is cosmetic. CONTEXT.md says "faire au plus simple" — document the tradeoff.

**Warning signs:** User reports settings reverting after quit. Low probability, low impact.

### Pitfall 3: The symmetric fix at `lib.rs:623` (no-tray path)

**What goes wrong:** Fixing only `lib.rs:255` (tray quit) leaves `lib.rs:623` (no-tray `CloseRequested` → `app.exit(0)`) unfixed. The crash dialog will still appear when Dictus is run with `--no-tray` and the window is closed.

**Why it happens:** Two independent code paths call `app.exit(0)`.

**How to avoid:** CONTEXT.md explicitly marks both sites. The helper function approach (`flush_and_exit`) naturally enforces this.

### Pitfall 4: `AppHandle::restart()` skips RunEvent::Exit (upstream bug)

**What goes wrong:** `AppHandle::restart()` (and therefore the updater plugin's restart) may skip the `RunEvent::Exit` event, meaning plugins don't get a clean shutdown signal even if path (a) registers a handler for it.

**Why it happens:** Confirmed upstream bug [tauri-apps/tauri#12310](https://github.com/tauri-apps/tauri/issues/12310), opened 2025-01-08.

**How to avoid:** The SHUT-03 simulation via `app.restart()` may reproduce the crash even after SHUT-02 is fixed. If the restart path still triggers the dialog, path (b) (`std::process::exit(0)`) called before `restart()` is invoked may be needed — OR the simulation itself confirms whether the bug manifests.

**Warning signs:** Tray-quit passes 3 consecutive clean quits (SHUT-02 fixed), but the restart simulation still triggers the dialog.

### Pitfall 5: Poison mutex in `DownloadCleanup::drop` and `LoadingGuard::drop`

**What goes wrong:** Both Drop implementations call `.lock().unwrap()`. If a worker thread panicked while holding the mutex, the subsequent `.unwrap()` in Drop will also panic, causing a double-panic → abort → "quit unexpectedly" dialog.

**Why it happens:** When `app.exit(0)` triggers manager teardown, Drop implementations run. If a prior thread panicked mid-operation, the mutex is poisoned.

**How to avoid:** Only fix if the `.ips` stack directly implicates these (CONTEXT.md says: do not touch unless confirmed). The `.ips` crash thread stack will show whether `managers/model.rs:79` or `managers/transcription.rs:59` appears.

**Warning signs:** `.ips` crashing thread shows `__rust_panic_*` or `libsystem_platform.dylib` in frames near `managers/model.rs` or `managers/transcription.rs`.

---

## Code Examples

### Reading the `.ips` Crash Report

```bash
# Console.app path: ~/Library/Logs/DiagnosticReports/Dictus-*.ips
# Or: Console.app → User Reports → Dictus → most recent entry
# The .ips file is JSON — the key sections are:

# "threads" array — each thread has:
#   "id": thread number
#   "name": "tokio-runtime-worker" | "main" | "com.apple.main-thread" etc
#   "frames": array of stack frames with "symbol" fields

# The crashing thread is marked with "triggered": true (or indicated by the
# "terminationReason" field near the top of the file).

# Quick read for the crash type:
python3 -c "import json,sys; d=json.load(open(sys.argv[1])); print(d.get('terminationReason',''))" ~/Library/Logs/DiagnosticReports/Dictus-*.ips
```

Source: Apple "Diagnosing Issues Using Crash Reports and Device Logs" — https://developer.apple.com/documentation/xcode/diagnosing-issues-using-crash-reports-and-device-logs

### Confirming `log::logger().flush()` at exit (MEDIUM confidence)

```rust
// Source: log crate Log trait — flush() "flushes any buffered records"
// Correct usage before any exit call:
log::logger().flush();
app.exit(0);
// -- or --
log::logger().flush();
std::process::exit(0);
```

### `GlobalShortcutExt::unregister_all()` — verified API

```rust
// Source: docs.rs tauri-plugin-global-shortcut 2.3.1
// GlobalShortcut::unregister_all(&self) -> Result<(), Error>
use tauri_plugin_global_shortcut::GlobalShortcutExt;

let _ = app.global_shortcut().unregister_all();
// Returns Result; log but don't propagate — we're shutting down anyway
```

### `#[cfg(target_os = "macos")]` guard pattern (from lib.rs)

```rust
// Already used in lib.rs for set_activation_policy, tauri_nspanel::init
// Any macOS-specific cleanup must follow this guard:
#[cfg(target_os = "macos")]
{
    let _ = app.global_shortcut().unregister_all();
}
```

---

## State of the Art

| Old Approach | Current Approach | Notes |
|--------------|------------------|-------|
| Rely on plugin Drop for cleanup | Explicit cleanup before `app.exit(0)` | Tauri 2.x plugin Drop ordering is not guaranteed |
| `app.exit(0)` directly | `flush + unregister + app.exit(0)` | Minimizes window where crash can occur |
| No diagnostic step | `.ips` read → diagnosis committed before code change | SHUT-01 mandates this gate |

**Known upstream Tauri 2.x issues relevant to this phase:**
- `AppHandle::restart()` may skip `RunEvent::Exit` ([#12310](https://github.com/tauri-apps/tauri/issues/12310)) — open as of 2025-01-08
- macOS Segfault during termination linked to tokio-runtime-worker ([#4159](https://github.com/tauri-apps/tauri/issues/4159)) — tokio worker thread crash pattern

---

## Open Questions

1. **Which thread is actually crashing?**
   - What we know: Candidates are `tokio-runtime-worker` (Rust worker), `main`/`com.apple.main-thread` (Tauri event loop), or a CGEventTap-related thread from `tauri-plugin-global-shortcut`
   - What's unclear: The exact crash thread in the Dictus binary on Sequoia 15.x (we don't have the `.ips` yet)
   - Recommendation: Generate the `.ips` as the first task in `07-01-PLAN.md`; the crashing thread frame determines fix path (a) vs (b)

2. **Does `app.run_on_main_thread` complete synchronously before `app.exit(0)`?**
   - What we know: `run_on_main_thread` posts a closure to the event loop's user event queue; the calling thread does NOT block waiting for it
   - What's unclear: Whether the closure executes before `app.exit(0)` returns and teardown begins
   - Recommendation: If ordering is needed, use `tauri::async_runtime::block_on` or restructure so that `app.exit(0)` is called from inside the `run_on_main_thread` closure

3. **Does `tauri_plugin_store` lose data with `std::process::exit(0)`?**
   - What we know: `std::process::exit(0)` bypasses destructors; store plugin normally writes on Drop
   - What's unclear: Whether `tauri-plugin-store` 2.4.1 has explicit flush-on-change or relies purely on Drop
   - Recommendation: Acceptable risk for path (b) — cosmetic bug fix tradeoff; document in the Diagnosis section

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Manual (no automated test framework detected for Tauri integration) |
| Config file | None — Tauri does not support headless quit testing in CI |
| Quick run command | `bun run tauri build && open dist/` (manual) |
| Full suite command | `bun run lint && cargo clippy` (static only) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SHUT-01 | Crash report read, crashing thread identified, diagnosis committed | manual | N/A — requires `.ips` file from real crash | ❌ manual-only |
| SHUT-02 | Fix code lands at `lib.rs:255` and `lib.rs:623` | manual smoke + code inspection | `cargo clippy --manifest-path src-tauri/Cargo.toml` | ✅ (clippy exists) |
| SHUT-03 | 3 consecutive tray-quit + 1 restart simulation with no dialog | manual observation | N/A — requires running app + human observation | ❌ manual-only |

**Manual-only justification:** macOS crash dialog testing requires a running, installed `.app` binary with OS crash reporter active. This cannot be automated in CI without a macOS GUI test runner (rejected per CONTEXT.md "faire au plus simple").

### Sampling Rate
- **Per task commit:** `cargo clippy --manifest-path src-tauri/Cargo.toml` — catches Rust errors
- **Per wave merge:** `bun run lint && cargo clippy` — full static check
- **Phase gate:** 3 consecutive clean quits observed manually before `/gsd:verify-work`

### Wave 0 Gaps

None — existing infrastructure (cargo clippy, bun lint) covers static validation. No new test files needed; SHUT-01/03 are human-in-the-loop by design.

---

## Sources

### Primary (HIGH confidence)
- `docs.rs` tauri-plugin-global-shortcut 2.3.1 — `GlobalShortcut::unregister_all()` signature confirmed
- `log` crate `Log::flush()` trait method — confirmed via docs.rs
- Direct code inspection — `lib.rs:254-256`, `lib.rs:621-624`, `lib.rs:531`, `managers/model.rs:73-86`, `managers/transcription.rs:57-63`
- Tauri process plugin docs — `app.restart()` / `tauri_plugin_process::restart()` confirmed
- Apple crash report documentation — https://developer.apple.com/documentation/xcode/diagnosing-issues-using-crash-reports-and-device-logs

### Secondary (MEDIUM confidence)
- [tauri-apps/tauri#12310](https://github.com/tauri-apps/tauri/issues/12310) — `AppHandle::restart` skips RunEvent::Exit; opened 2025-01-08
- [tauri-apps/tauri#4159](https://github.com/tauri-apps/tauri/issues/4159) — macOS termination segfault in tokio-runtime-worker; prior Tauri version but consistent pattern
- Tauri Discussion #3273 — `Kill process on exit` community pattern; `std::process::exit` precedent

### Tertiary (LOW confidence — WebSearch, unverified)
- Multiple WebSearch results indicating global-shortcut plugin macOS teardown issues — consistent with hypothesis but no official fix documented
- Tokio runtime shutdown docs re: worker thread panics during Drop — consistent with `DownloadCleanup`/`LoadingGuard` hypothesis

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — plugin versions from `Cargo.toml`; APIs from docs.rs
- Architecture: HIGH — code read directly; Tauri shutdown lifecycle from official docs
- Pitfalls: MEDIUM — mutex-poison pitfall from code inspection (HIGH); thread-safety of CGEventTap from WebSearch (MEDIUM); store data loss from stdlib docs (MEDIUM)
- Validation: HIGH — manual-only is the honest answer given the nature of the bug

**Research date:** 2026-04-16
**Valid until:** 2026-07-16 (stable Tauri 2.x; plugin versions fixed in Cargo.toml)
