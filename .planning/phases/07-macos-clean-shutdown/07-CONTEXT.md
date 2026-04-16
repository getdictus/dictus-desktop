# Phase 7: macOS Clean Shutdown - Context

**Gathered:** 2026-04-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Diagnose and fix the "Dictus quit unexpectedly — Reopen / Report / Ignore" OS crash dialog that appears on macOS Sequoia 15.x when the app quits cleanly — via tray → "Quit Dictus" and after an auto-updater relaunch. The dialog is cosmetic (the app does exit), but it undermines the "Dictus Desktop" identity established in v1.0/v1.1. Per SHUT-01, a diagnosis (crashing thread identified + fix strategy chosen) must be committed to the phase plan **before** any code changes land.

**Requirements covered:** SHUT-01, SHUT-02, SHUT-03.

**Out of scope:** Touching the Handy upstream repo (no issue, no PR — Dictus-only fix); chasing third-party plugin internals (patch-in-tree or vendor forks rejected); the general mutex-poison `Drop` risks flagged in `.planning/codebase/CONCERNS.md` (related but separate tech-debt, only addressed if the diagnosis points to them as the direct root cause); any fix on platforms other than macOS Sequoia 15.x on Apple Silicon.

</domain>

<decisions>
## Implementation Decisions

### Diagnosis workflow

- **Evidence source:** reproduce first on a fresh build from current `main`. Generate a matching `.ips` crash report on Pierre's M4 Pro / Sequoia 15.x, rather than relying on an older report that may not map 1:1 to the current binary.
- **Repro recipe:** `bun run tauri build` → install resulting `.app` → launch → tray → "Quit Dictus". Repeat until `Console.app → User Reports → Dictus` shows a fresh entry. Capture the `.ips` path and any trailing lines from the Dictus log file at `~/Library/Logs/com.dictus.desktop/dictus.log` (or portable `Data/logs/dictus.log` if in portable mode).
- **Depth:** Console.app crash report + `RUST_BACKTRACE=1` logs only. No `lldb` attach, no plugin bisection upfront. Minimum viable diagnosis per SHUT-01.
- **Ambiguity handling:** stay pragmatic — if logs don't cleanly pinpoint a thread, adapt based on what's there. No commitment to escalate to `lldb` or plugin bisection. Bug is non-blocking (cosmetic), so simplicity wins over certainty.
- **Write-up location:** inline in `07-01-PLAN.md` preamble — a `## Diagnosis` section at the top of the first plan, above the task list. Contains: suspect thread, relevant stack frames, chosen fix strategy, and the evidence link (`.ips` excerpt + log excerpt). Matches SHUT-01 wording ("documented in phase plan") and keeps diagnosis + tasks in one file.
- **Gate:** no code changes are committed in Phase 7 until the `## Diagnosis` section is written and committed. First commit of the phase is `docs(07): capture diagnosis of quit-unexpectedly crash (SHUT-01)` or similar.

### Fix strategy selection

- **Rule:** the diagnosis dictates which path we take. Two acceptance criteria per SHUT-02:
  - **(a) Graceful cleanup** — preferred when the root cause is in Dictus Rust code. Examples: explicit manager drop order before `app.exit(0)`; `app_handle.run_on_main_thread()` wrapper for `tauri_plugin_global_shortcut` unregister; poison-safe `Drop` impl that the shutdown sequence is triggering. Preserves destructors (log flush, file close, GPU context release).
  - **(b) `std::process::exit(0)` workaround** — accepted when the suspect is a third-party plugin (`tauri_plugin_global_shortcut`, `tauri_plugin_single_instance`, `tauri_plugin_updater`, `tauri_plugin_store`). Hard-exits after flushing logs. Forfeits normal teardown by design.
- **No (a)→(b) retry loop inside one attempt:** each plan tries one path end-to-end. If graceful cleanup doesn't clear the dialog, the next plan iteration swaps to `std::process::exit(0)`. See iteration cap below.
- **Always flush before exit:** regardless of (a) or (b), insert `log::logger().flush()` (or the equivalent `tauri_plugin_log` flush API if that's what's available) immediately before `app.exit(0)` at both call sites (`src-tauri/src/lib.rs:255` tray quit, `src-tauri/src/lib.rs:623` CloseRequested no-tray). Cheap insurance; any future shutdown regression gets its trailing logs on disk. Add as a small helper if it's used from two places.
- **Iteration cap:** 2 plan iterations maximum. One graceful-cleanup attempt, one `std::process::exit(0)` fallback. If neither kills the dialog, the phase ships the best-achieved state (fix committed, dialog documented as upstream-pending), PHASE-STATUS records the residual, and we move on. Bug is cosmetic — not worth infinite iteration.
- **Plugin vendoring is rejected:** if the suspect is third-party, we do NOT fork/vendor the plugin to patch it. `std::process::exit(0)` is cleaner for our scope.

### SHUT-03 verification

- **Pass bar:** 3 consecutive clean quits from a fresh `bun run tauri build` install, no "Dictus quit unexpectedly" dialog on any of them.
- **Tray-Quit path:** launch the installed `.app`, tray → "Quit Dictus", observe. Repeat 3x. Manual observation — no AppleScript / accessibility automation.
- **Post-auto-update relaunch path:** simulate the updater restart flow via `tauri_plugin_process` (the same API the updater plugin calls after applying an update). The plan identifies the exact symbol/call path and wraps it in a manual trigger — e.g., a debug-only command or a direct invocation from the JS console when `debug_mode` is enabled. Real v0.1.x test release is NOT cut for this validation; Phase 7 stays pre-release.
- **Environment:** Pierre's dev machine only — M4 Pro, macOS Sequoia 15.x, Apple Silicon. No Intel Mac, no clean VM. Solo-project discipline matches Phase 6.
- **Automation:** none. Quit, watch, no dialog = pass. Cosmetic bug, low cadence — scripting ReportCrash introspection isn't worth the effort.

### Upstream coordination

- **Dictus-only fix.** No issue filed on `cjpais/Handy`, no PR upstream. Pierre confirmed the bug is pre-existing on unmodified Handy, but engaging upstream adds effort with uncertain benefit. Future upstream syncs won't conflict because Handy never fixed it.
- **Protection against future sync revert:** add one line to `UPSTREAM.md` conflict-rules section — `lib.rs quit-exit handlers / log-flush helper — Dictus version wins (SHUT-02)`. Mirrors the existing pattern for `tauri.conf.json`, i18n files, and `Cargo.toml`. No `verify-sync.sh` assertion in Phase 7 — rule is too shape-sensitive to grep cleanly; human review + the conflict-rules note is sufficient.
- **No plan for reconciling a future Handy fix.** If upstream eventually ships their own fix, deal with it at that sync. Phase 7 doesn't pre-write that playbook.
- **Commit message should reference SHUT-02.** So a future auto-adapter (Phase 10) reading commit history can see that `lib.rs:255` / `lib.rs:623` edits are deliberate fork divergence, not accidental drift.

### Claude's Discretion

- Exact structure of the `## Diagnosis` section inside `07-01-PLAN.md` — as long as it names the suspect thread, the evidence, and the chosen fix path before the first task.
- Whether `log::logger().flush()` lives in a tiny helper (e.g., `fn flush_logs_then_exit(app, code)`) or is inlined at both sites — pick whichever reads cleaner given the two-site usage.
- Exact naming / location of the auto-update-relaunch simulation trigger (debug-only Tauri command vs JS-console-callable path). Match existing debug-panel patterns at `src/components/settings/debug/`.
- Commit granularity — one commit per task vs one commit per surface — follow existing Phase 6 style.
- Whether the `## Diagnosis` section is folded into `07-01-PLAN.md` or (if diagnosis grows longer than expected) extracted to a sibling `07-DIAGNOSIS.md` mid-phase. Default to inline; split only if length forces it.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements and scope
- `.planning/REQUIREMENTS.md` §macOS Clean Shutdown — SHUT-01, SHUT-02, SHUT-03 acceptance criteria (diagnosis before fix; explicit cleanup OR `std::process::exit(0)`; clean quit + post-update relaunch)
- `.planning/ROADMAP.md` §"Phase 7: macOS Clean Shutdown" — goal + 3 success criteria
- `.planning/PROJECT.md` §"Current Milestone: v1.2 Polish & Automation" — milestone framing, including "macOS clean shutdown fix (Dictus quit unexpectedly dialog)" as a v1.2 target

### Open tickets
- `.planning/todos/pending/2026-04-15-fix-macos-quit-unexpectedly-dialog-on-clean-shutdown.md` — original problem statement, observed environment (Dictus v0.1.1 on macOS Sequoia 15.x, M4 Pro), confirmed as pre-existing upstream Handy bug, and initial list of suspect areas (background thread panic, Metal/GPU teardown, SIGKILL timeout, global-shortcut unhook, single-instance socket cleanup). Plan MUST close or supersede this todo.

### Code surfaces to touch (grounded paths)
- `src-tauri/src/lib.rs:254-256` — tray "quit" menu event: `"quit" => { app.exit(0); }`. Both log-flush insertion and any graceful cleanup land here.
- `src-tauri/src/lib.rs:601-625` — `on_window_event(... CloseRequested ...)` closure; `window.app_handle().exit(0)` at line 623 for the no-tray path. Symmetric fix required.
- `src-tauri/src/tray.rs:129-130` — tray menu item for Quit (referenced in todo). Reading only — no change expected unless diagnosis points here.
- `src-tauri/src/signal_handle.rs` — SIGUSR1/SIGUSR2 signal handler and `send_transcription_input()` helper. Reading only — relevant if the diagnosis implicates signal delivery during shutdown.

### Related tech-debt context (informational, not required fix surface)
- `.planning/codebase/CONCERNS.md` §"Mutex Poison Recovery in Drop" — `ModelManager::DownloadCleanup` `Drop` impl at `src-tauri/src/managers/model.rs:73-85` calls `.lock().unwrap()` and can secondary-panic during shutdown teardown
- `.planning/codebase/CONCERNS.md` §"LoadingGuard Drop Panic Risk" — `TranscriptionManager::LoadingGuard` at `src-tauri/src/managers/transcription.rs:57-62` has the same missing-poison-recovery pattern
- Both are candidate root causes for a panicking background thread at shutdown. Planner should treat them as hypotheses to verify against the `.ips` stack, not as must-fix work items.

### Prior phase context (decisions carried forward)
- `.planning/phases/06-brand-icon-polish/06-CONTEXT.md` — "faire au plus simple" philosophy, solo-user → no migration ceremony, UPSTREAM.md conflict-rules pattern, commit-convention examples (`fix(…)`, `chore(…)`)
- `.planning/phases/05-upstream-sync/05-CONTEXT.md` — `verify-sync.sh` assertion philosophy (why we're NOT adding one here), UPSTREAM.md conflict-resolution pattern for fork-divergent files
- `.planning/phases/04-updater-infrastructure/04-CONTEXT.md` — `tauri-plugin-updater` setup; relevant because the updater's restart API is the one we simulate for SHUT-03's relaunch path

### External docs / tools
- Apple "Diagnosing an issue with a crash report" — https://developer.apple.com/documentation/xcode/diagnosing-issues-using-crash-reports-and-device-logs — reading `.ips` files, symbolication basics
- `tauri-plugin-process` — https://v2.tauri.app/plugin/process/ — `restart()` API used by the updater plugin after applying an update (the call path SHUT-03 relaunch verification simulates)
- `tauri-plugin-log` — https://v2.tauri.app/plugin/logging/ — logger flush semantics; confirms `log::logger().flush()` is the correct cross-plugin API
- `tauri-plugin-global-shortcut` — https://v2.tauri.app/plugin/global-shortcut/ — primary plugin suspect per upstream symptoms; cleanup-on-drop behavior

### No external ADRs
This project has no `docs/decisions/` ADR directory. Decisions live in `.planning/` and inline in CONTEXT.md.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `tauri_plugin_log` is already wired (`lib.rs:482-511`) with `TargetKind::Folder` writing to `portable::data_dir().join("logs")` or system log dir. Flush API is available via `log::logger().flush()` at exit time without adding dependencies.
- `portable::data_dir()` and `portable::app_data_dir()` are portable-aware helpers already used by the logger setup. The diagnosis write-up can point planner/humans at the exact log path based on install mode.
- `tauri_plugin_process` is already in the dep tree (`lib.rs:531`). The updater-relaunch simulation for SHUT-03 can call its `restart()` API directly without new deps.
- `signal_handle::send_transcription_input()` shows the pattern for a helper called from multiple exit/trigger sites — if we split `flush_then_exit()` into a helper, mirror this style.
- `catch_unwind` wrapper pattern exists in `src-tauri/src/managers/transcription.rs` (line 526) — a proven defensive pattern if the diagnosis points to an unhandled panic during shutdown.

### Established Patterns
- **Exit handling:** always via `app.exit(code)` (Tauri-native). `std::process::exit(0)` is not currently used anywhere in `src-tauri/src/`; introducing it (fix path b) is a visible fork-divergence worth a comment.
- **Macros:** `#[cfg(target_os = "macos")]` guards already used extensively in `lib.rs` (`set_activation_policy`, `tauri_nspanel::init`); any macOS-specific cleanup should follow this guard pattern rather than running on all platforms.
- **Commit conventions:** `feat:`, `fix:`, `chore:`, `docs:`. Phase 7 commits likely fit `docs(07): capture diagnosis ... (SHUT-01)` for the diagnosis-first commit and `fix(shutdown): ... (SHUT-02)` for code changes.
- **Plan numbering:** Phase 6 used `06-01-PLAN.md` through `06-04-PLAN.md`. Phase 7 likely ships `07-01-PLAN.md` (possibly `07-02-PLAN.md` if the fallback iteration triggers).

### Integration Points
- `app.exit(0)` ↔ `tauri::RunEvent::ExitRequested` (if we ever register one) ↔ plugin `Drop` impls ↔ OS termination. The fix lives in the narrow window between tray-menu click and OS `kill`.
- `tauri_plugin_updater::restart()` → `tauri_plugin_process::restart()` → spawn new process / exit current. The simulation for SHUT-03 relaunch calls into this chain without a real `latest.json` delta.
- `log::logger().flush()` ↔ `tauri_plugin_log` targets (stdout filter + file filter). Must be called before the hard exit, or file target may lose trailing buffered bytes on fix path (b).
- `UPSTREAM.md` conflict-rules section ↔ weekly upstream sync (Phase 5 SYNC-03) ↔ future Phase 10 auto-adapter. One-line note there is the contract that protects this fix.

### Deferred touchpoints (DO NOT modify this phase)
- The mutex-poison `Drop` risks in `DownloadCleanup` and `LoadingGuard` — touch ONLY if the `.ips` stack directly implicates one of them as the crashing thread. Otherwise they remain CONCERNS.md tech debt for a later phase.
- `app.exit()` → `std::process::exit()` migration across the whole app — out of scope. Even if fix path (b) wins, `std::process::exit(0)` replaces `app.exit(0)` only at the two named lines (`lib.rs:255`, `lib.rs:623`), not everywhere.
- Third-party plugin source (`tauri-plugin-global-shortcut` etc.) — not vendored, not patched.
- macOS code-signing / notarization (INFR-03 deferred) — the crash reproduces on unsigned dev builds; signing state is orthogonal.

</code_context>

<specifics>
## Specific Ideas

- "Faire au plus simple" carries over from Phase 6. Cosmetic bug, solo user, pre-existing upstream — don't over-engineer diagnosis or fix.
- Pierre reproduced the dialog on unmodified `cjpais/Handy` before Phase 7 was scoped. That's strong evidence the suspect is upstream code or a shared-dep plugin, not a v1.0/v1.1 regression introduced by the Dictus rebrand.
- A clean shutdown log line as the last action before exit is a nice "fingerprint" — if it's present and no `.ips` was generated, we have a confident pass signal.
- This is a two-plan-max phase by design. Don't let it balloon.

</specifics>

<deferred>
## Deferred Ideas

- **Fixing mutex-poison `Drop` impls in `ModelManager::DownloadCleanup` and `TranscriptionManager::LoadingGuard`** — CONCERNS.md tech debt. Only touched in Phase 7 if the diagnosis directly implicates them; otherwise deferred to a future hardening phase.
- **Filing an issue or PR on `cjpais/Handy` about this bug** — explicitly rejected for Phase 7. Could be revisited once Dictus has a validated fix and bandwidth to contribute back, but not a v1.2 commitment.
- **Adding a `verify-sync.sh` assertion to prevent future sync from reverting the fix** — rejected as too shape-sensitive. UPSTREAM.md conflict-rules note carries that protection load instead.
- **Cutting a throwaway `v0.1.x-test` release to validate post-update relaunch** — rejected in favor of the `tauri_plugin_process::restart()` simulation.
- **Intel Mac / clean-VM verification** — out of scope; Pierre's M4 Pro is the sole target environment for Phase 7.
- **Vendoring or forking `tauri-plugin-global-shortcut` (or any other suspect plugin) to patch it in-tree** — rejected; `std::process::exit(0)` fallback is the accepted third-party-plugin workaround.
- **Planning the reconciliation with a future upstream Handy fix** — deliberately deferred; revisit if/when upstream ships their own fix.

</deferred>

---

*Phase: 07-macos-clean-shutdown*
*Context gathered: 2026-04-16*
