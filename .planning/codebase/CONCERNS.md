# Codebase Concerns

**Analysis Date:** 2026-04-05

## Tech Debt

**Model Configuration Hardcoded:**
- Issue: Model configurations (URLs, SHA256, language lists) are hardcoded in `src-tauri/src/managers/model.rs` (lines 109-500+) instead of being read from a JSON file
- Files: `src-tauri/src/managers/model.rs:125`
- Impact: Difficult to update models without code changes; violates maintainability; adding new models requires code modifications
- Fix approach: Extract model registry to external JSON file loaded at startup; update on-the-fly without recompilation

**Mutex Unwrap Patterns:**
- Issue: 62 instances of `.lock().unwrap()` throughout codebase
- Files: `src-tauri/src/managers/model.rs`, `src-tauri/src/managers/transcription.rs`, `src-tauri/src/settings.rs`, `src-tauri/src/shortcut/mod.rs`
- Impact: Panic if a thread holding a mutex panics, poisoning the lock and potentially hanging the app; violates defensive programming
- Fix approach: Use `.lock().unwrap_or_else(|poisoned| poisoned.into_inner())` pattern (already used in `src-tauri/src/managers/transcription.rs:168`); apply consistently across all mutex accesses

**Settings Store Error Handling:**
- Issue: `store.set()` calls use `.unwrap()` without error handling
- Files: `src-tauri/src/settings.rs:858, 867, 873, 878, 892, 897` and multiple locations in tray.rs
- Impact: Silently fails to persist settings if store write fails; users lose configuration changes
- Fix approach: Log errors and provide user feedback when settings persistence fails; use Result return type

## Known Bugs

**Windows Unsafe COM Code:**
- Symptoms: Audio muting may fail on some Windows systems; potential memory safety issues
- Files: `src-tauri/src/managers/audio.rs:23-54` (unsafe block)
- Trigger: Calling `set_mute()` on Windows with non-standard audio drivers
- Cause: Unsafe COM interop code; CLSCTX_ALL may access unexpected device interfaces
- Workaround: Audio muting falls back to enigo keyboard input if COM fails
- Mitigation: Function already has silent failure fallback; logs on debug builds would help

**Mutex Poison Recovery in Drop:**
- Symptoms: Drop handler for DownloadCleanup may panic if mutex is poisoned
- Files: `src-tauri/src/managers/model.rs:73-85` (Drop impl calls `.lock().unwrap()`)
- Trigger: Model download fails and another thread panics while holding cleanup lock
- Cause: `Drop` impl doesn't use poison recovery pattern
- Workaround: None — can cause secondary panic during error recovery
- Fix approach: Apply same pattern as `lock_engine()`: use `.unwrap_or_else(|poisoned| poisoned.into_inner())`

**LoadingGuard Drop Panic Risk:**
- Symptoms: Same as above — drop handler calls `.lock().unwrap()`
- Files: `src-tauri/src/managers/transcription.rs:57-62`
- Trigger: Model loading panics while LoadingGuard is being dropped
- Cause: Missing poison recovery in Drop impl
- Fix approach: Apply mutex poison recovery pattern

## Security Considerations

**Windows COM Initialization:**
- Risk: CoInitializeEx is called per mute operation without validation of return value
- Files: `src-tauri/src/managers/audio.rs:43`
- Current mitigation: Ignores return value; worst case is COM already initialized (expected)
- Recommendations: Document that CoInitializeEx can be called multiple times safely; consider early initialization at app startup

**Unsafe Block Scope:**
- Risk: Large unsafe block (23 lines) handles multiple COM operations
- Files: `src-tauri/src/managers/audio.rs:23-54`
- Current mitigation: All operations wrapped in `unwrap_or_return!()` macro to bail early
- Recommendations: Smaller unsafe scopes for each COM operation; more detailed comments explaining invariants

**Custom Model Verification Bypass:**
- Risk: Custom user models skip SHA256 verification intentionally
- Files: `src-tauri/src/managers/model.rs:940-944`
- Current mitigation: Verification only for official models; custom models marked with `is_custom: true`
- Recommendations: Consider warning users about unverified custom models; implement optional verification

## Performance Bottlenecks

**Linux Clipboard Operations:**
- Problem: Multiple subprocess calls to check for available tools (wtype, dotool, ydotool, xdotool, wl-copy) on every clipboard operation
- Files: `src-tauri/src/clipboard.rs:224-281` (multiple `is_*_available()` functions)
- Cause: No caching of tool availability; `which` subprocess called repeatedly
- Impact: Slow clipboard operations on Linux; multiple process spawns per transcription
- Improvement path: Cache availability checks in a static once_cell on first use; refresh periodically if needed

**Model File Verification:**
- Problem: SHA256 verification reads entire model file (up to 1GB) into memory via chunking
- Files: `src-tauri/src/managers/model.rs:972-985`
- Cause: Streaming computation is correct but done synchronously on main/UI thread
- Impact: UI freezes during download verification on large models
- Improvement path: Move verification to background thread; emit progress events

**Model Discovery:**
- Problem: Directory scan on startup discovers custom models by iterating filesystem
- Files: `src-tauri/src/managers/model.rs:818-935`
- Cause: Synchronous fs::read_dir() on app data directory at startup
- Impact: Startup delay with many custom models; blocks initialization
- Improvement path: Load custom models lazily or in background; cache discovery results

**Command Launch Overhead:**
- Problem: Clipboard operations spawn multiple subprocesses to determine available tools
- Files: `src-tauri/src/clipboard.rs:202-281`
- Cause: Each `is_*_available()` spawns a `which` subprocess
- Impact: Dozens of process spawns per paste operation on Linux
- Improvement path: Cache availability at app start or lazy-load once per session

## Fragile Areas

**Mutex Poison Recovery:**
- Files: `src-tauri/src/managers/transcription.rs:166-172`, `src-tauri/src/managers/transcription.rs:508-524`
- Why fragile: Engine mutex can be poisoned if transcription panics; recovery done via `unwrap_or_else(|poisoned| poisoned.into_inner())`
- Safe modification: All engine accesses must use `lock_engine()` method; never use direct `.lock().unwrap()`
- Test coverage: Gaps — no unit tests for poison recovery; requires integration test with forced panic
- Mitigation: Code already uses `catch_unwind()` to prevent engine panics from poisoning (line 526); good defensive practice

**Audio Recording Thread:**
- Files: `src-tauri/src/audio_toolkit/audio/recorder.rs:87-200+`
- Why fragile: Worker thread spawned with complex state (stream, channels, resampling, VAD); thread panic would hang the app
- Safe modification: All worker thread operations wrapped in error handling; stop_flag checked in main loop
- Test coverage: No unit tests for thread panic scenarios
- Mitigation: Good error propagation via sync_channel; consider wrapping main loop in catch_unwind

**Shortcut Implementation Fallback:**
- Files: `src-tauri/src/shortcut/mod.rs:34-56`
- Why fragile: HandyKeys initialization can fail and fallback to Tauri; fallback persisted to settings automatically
- Safe modification: Fallback mechanism prevents crashes but silently changes user's setting; test both paths
- Test coverage: Integration test needed to verify fallback behavior
- Mitigation: Explicit logging at warn level; setting persisted to prevent repeated retry

**GTK Layer Shell Initialization on Linux:**
- Files: `src-tauri/src/overlay.rs:68-80`
- Why fragile: GTK layer shell init on KDE Wayland unstable; falls back to regular overlay
- Safe modification: Conditional logic checks XDG_SESSION_TYPE and XDG_CURRENT_DESKTOP; do not remove checks
- Test coverage: Platform-specific tests difficult; manual testing on KDE Wayland required
- Mitigation: Explicit comment explaining protocol instability; graceful degradation to always-on-top

## Scaling Limits

**Model Manager Available Models HashMap:**
- Current capacity: Single Mutex<HashMap> containing 15+ models with hardcoded definitions
- Limit: Adding models requires code changes and recompilation; model registry not externalized
- Scaling path: Move to JSON-based model registry; lazy-load custom models; implement model discovery protocol

**History Database Schema:**
- Current capacity: SQLite with basic schema; 4 migrations defined
- Limit: No index on timestamp; pagination uses offset (N+1 problem); post-processing adds more columns
- Scaling path: Add indexes on timestamp/file_name; implement cursor-based pagination; consider partitioning history

**Transcription Coordinator Message Queue:**
- Current capacity: Single-threaded mpsc channel for all transcription events
- Limit: Debounce window is 30ms; rapid input processed sequentially
- Scaling path: Current approach acceptable; would need buffered channel if latency requirements tighten

## Dependencies at Risk

**Windows Audio COM Interop:**
- Risk: Unsafe COM code in `audio.rs` is brittle and platform-specific
- Files: `src-tauri/src/managers/audio.rs:23-54`
- Impact: Audio muting functionality fragile; potential memory safety issues if COM API changes
- Migration plan: Consider using `windows-audio` crate if available; reduce unsafe surface area; add feature flag for optional audio muting

**GTK Layer Shell Integration:**
- Risk: Protocol instability on KDE Wayland; may break in future Wayland versions
- Files: `src-tauri/src/overlay.rs:18-19, 68-80`
- Impact: Overlay window may not work correctly on KDE Wayland
- Migration plan: Monitor Wayland desktop portal changes; consider alternative overlay implementations; add wayland-client dependency for more direct control

**Tauri Plugin Store Dependency:**
- Risk: Settings persisted via `tauri_plugin_store`; direct `.unwrap()` calls on store operations
- Files: `src-tauri/src/settings.rs:835-837, 885-887`
- Impact: Store initialization failure crashes app startup
- Migration plan: Replace `.expect()` with proper error handling; implement settings migration path; consider fallback to defaults

## Missing Critical Features

**No Settings Validation:**
- Problem: Settings loaded from store without schema validation
- Blocks: Ensures backward compatibility; prevents corrupted settings from breaking app
- Files: `src-tauri/src/settings.rs:889-894`
- Improvement: Implement settings versioning and migration; validate all fields on load

**No Model Download Resume Recovery:**
- Problem: Partial downloads resume automatically but cleanup state can fail
- Blocks: Users stuck with partial downloads if cleanup code panics
- Files: `src-tauri/src/managers/model.rs:1014-1046` (cleanup guard)
- Improvement: Explicit state machine for download phases; verify partial file before resume

**No Transcription Error Recovery UI:**
- Problem: Transcription failures logged but no user-facing recovery options
- Blocks: Users don't know why transcription failed or what to do
- Files: `src-tauri/src/managers/transcription.rs` (no error detail events)
- Improvement: Emit detailed error events to frontend; show error UI with recovery suggestions

## Test Coverage Gaps

**Mutex Poison Recovery:**
- What's not tested: Panic behavior when model load fails mid-transcription
- Files: `src-tauri/src/managers/transcription.rs:166-172`
- Risk: Poisoned mutex can hang app indefinitely
- Priority: High — should test forced panic with transcribe_rs to verify recovery works

**Windows Audio Muting:**
- What's not tested: COM initialization on systems with non-standard audio drivers
- Files: `src-tauri/src/managers/audio.rs:13-100`
- Risk: Silent failures leave audio unmuted during transcription
- Priority: High — platform-specific issue; difficult to test cross-platform

**Linux Clipboard Edge Cases:**
- What's not tested: Behavior when tools unavailable; Wayland vs X11 detection
- Files: `src-tauri/src/clipboard.rs` (multiple platform-specific branches)
- Risk: Text paste fails silently on some Linux configurations
- Priority: Medium — covered by fallback chain but specific tool failures not tested

**GTK Layer Shell Initialization:**
- What's not tested: KDE Wayland layer shell protocol behavior
- Files: `src-tauri/src/overlay.rs:68-80`
- Risk: Overlay positioning incorrect on KDE; falls back silently
- Priority: Medium — manual testing on target platform required

**Custom Model Loading:**
- What's not tested: Loading corrupted custom model files; discovery with many custom models
- Files: `src-tauri/src/managers/model.rs:818-935`
- Risk: Corrupted custom models crash transcription engine
- Priority: Medium — user-supplied input; should validate before loading

## Reliability Concerns

**Panic-Driven Mutex Poison:**
- Issue: If transcription_rs panics during transcription, engine mutex is poisoned unless `lock_engine()` is used consistently
- Files: `src-tauri/src/managers/transcription.rs:166-172, 508-635`
- Impact: Subsequent transcription attempts will hang waiting for poison recovery
- Status: Mitigated by `catch_unwind()` wrapper (line 526) and proper poison recovery pattern, but relies on consistency

**Download State Inconsistency:**
- Issue: Model download state (is_downloading, cancel_flags) cleaned up in Drop impl of DownloadCleanup
- Files: `src-tauri/src/managers/model.rs:66-86`
- Impact: If cleanup Drop panics, download state left inconsistent; subsequent attempts corrupted
- Status: At risk — cleanup Drop itself panics on poisoned mutex

**Settings Deserialization:**
- Issue: Settings loaded from store with `.unwrap_or_else()` that resets to defaults on parse error
- Files: `src-tauri/src/settings.rs:889-894`
- Impact: User settings lost silently if corrupted; no migration path
- Status: Acceptable fallback but should log warnings and offer recovery

---

*Concerns audit: 2026-04-05*
