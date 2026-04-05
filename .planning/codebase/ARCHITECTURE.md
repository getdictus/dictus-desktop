# Architecture

**Analysis Date:** 2026-04-05

## Pattern Overview

**Overall:** Manager-based layered architecture with command-event separation between Tauri backend and React frontend.

**Key Characteristics:**
- Clear separation between core business logic (Rust managers), I/O operations (audio toolkit), and UI (React components)
- Event-driven communication across process boundary using Tauri's command-event pattern
- Exclusive single-threaded coordinator for transcription pipeline to eliminate race conditions
- RAII guards and atomic state for safe concurrent operations in audio and model management
- Hierarchical settings management with in-app overrides and persistent storage

## Layers

**Presentation (Frontend):**
- Purpose: User interface and user input handling
- Location: `src/`
- Contains: React components, hooks, Zustand stores
- Depends on: Tauri API bindings, settings store state
- Used by: User interactions (clicks, keyboard, settings changes)

**Application (Commands):**
- Purpose: RPC handlers bridging frontend requests to backend managers
- Location: `src-tauri/src/commands/`
- Contains: Tauri command handlers (decorated with `#[tauri::command]`)
- Depends on: Managers, settings module
- Used by: Frontend via Tauri IPC

**Business Logic (Managers):**
- Purpose: Core functionality orchestration and state management
- Location: `src-tauri/src/managers/`
- Contains: `TranscriptionManager`, `AudioRecordingManager`, `ModelManager`, `HistoryManager`
- Depends on: Audio toolkit, settings, transcription coordinator
- Used by: Commands, shortcut handlers, tray menu

**Event Coordination:**
- Purpose: Serializes transcription lifecycle events to prevent race conditions
- Location: `src-tauri/src/transcription_coordinator.rs`
- Contains: Command dispatch loop managing recording→processing→idle state machine
- Depends on: Managers (AudioRecordingManager)
- Used by: Shortcut handlers, signal handlers

**Input Handling (Shortcuts):**
- Purpose: Global keyboard shortcuts and input events
- Location: `src-tauri/src/shortcut/`
- Contains: Two keyboard implementations (Tauri native, HandyKeys), binding change handlers
- Depends on: Settings, managers
- Used by: OS-level keyboard events, transcription coordinator

**Audio Processing (Audio Toolkit):**
- Purpose: Low-level audio capture, processing, and device management
- Location: `src-tauri/src/audio_toolkit/`
- Contains: `AudioRecorder`, device enumeration, Voice Activity Detection (VAD), resampling
- Depends on: CPAL (cross-platform audio), Silero VAD model
- Used by: AudioRecordingManager

**Persistence (Settings & History):**
- Purpose: Application state and user data storage
- Location: `src-tauri/src/settings.rs`, `src-tauri/src/managers/history.rs`
- Contains: Settings struct, JSON serialization, history database
- Depends on: Tauri store plugin
- Used by: All managers, commands

## Data Flow

**Recording Pipeline:**

1. User presses hotkey → Shortcut handler captures event
2. Handler sends to TranscriptionCoordinator (serialized command queue)
3. Coordinator transitions from Idle → Recording stage
4. AudioRecordingManager starts capturing audio (device → buffer)
5. VAD processes frames to detect voice activity
6. When recording ends, audio sent to TranscriptionManager
7. TranscriptionManager loads model and transcribes
8. Coordinator transitions to Processing stage
9. Post-processing (optional LLM call) via llm_client
10. Result copied to clipboard or pasted via input system
11. Coordinator transitions back to Idle, notifies frontend
12. History entry created, frontend updates

**Model Loading Pipeline:**

1. Frontend calls `commands::models::download_model` or `set_active_model`
2. ModelManager validates model, downloads if needed
3. TranscriptionManager loads model asynchronously in background thread
4. LoadingGuard ensures `is_loading` flag is reset even on panic
5. Frontend polls via `get_transcription_model_status` or listens to `model-state-changed` event
6. Idle timeout watcher unloads model after configured time (default 60min)

**Settings Flow:**

Frontend:
```
useSettings() hook
  ↓ reads from
settingsStore (Zustand)
  ↓ initialized from
commands.getAppSettings()
  ↓ on change calls
commands.updateSetting(key, value)
  ↓
Backend command handler writes to settings.json
  ↓
SettingsStore updates local state
```

Backend:
```
get_settings(&app) reads from disk cache
  ↓
On setting change, write_settings() persists
  ↓
Managers may react to setting changes (e.g., accelerator settings)
```

**State Management:**

- **Tauri Managed State**: Managers and coordinators stored via `app.manage()`
- **Frontend Zustand**: Settings, models, audio devices
- **Persistent Storage**: Settings (JSON), transcription history (database), models (filesystem)

## Key Abstractions

**Manager Pattern:**

Purpose: Encapsulate domain logic with internal state and thread-safe access
Examples: `TranscriptionManager`, `AudioRecordingManager`, `ModelManager`, `HistoryManager`
Pattern: 
- Held in `Arc<Mutex<T>>` for cloning across threads
- Methods accept `&self`, state mutations via interior mutability
- RAII guards (e.g., `LoadingGuard`) manage cleanup automatically
- Direct to Rust for type-safe state, avoid string keys

**RAII Guards:**

Pattern: Cleanup executed on drop, even during panic or early return
Examples:
- `LoadingGuard` in TranscriptionManager — resets `is_loading` flag
- `DownloadCleanup` in ModelManager — clears download state

**Action Trait:**

Purpose: Dispatch different behaviors based on shortcut type
Location: `src-tauri/src/actions.rs`
Pattern: 
- Trait `ShortcutAction` with `start()` and `stop()` methods
- Multiple implementations (TranscribeAction, CancelAction, etc.)
- Stored in `ACTION_MAP` HashMap keyed by binding_id

**Coordinator State Machine:**

Purpose: Serialize all transcription lifecycle events
Location: `src-tauri/src/transcription_coordinator.rs`
Pattern:
- Enum `Stage` with variants: Idle, Recording(binding_id), Processing
- Channel `(tx, rx)` for command dispatch
- Single background thread owns the stage, preventing race conditions
- Debouncing of rapid key presses (30ms)

**Event Emission:**

Pattern: Backend sends events to all listeners via `app.emit("event-name", data)`
Examples:
- `model-state-changed` — frontend updates available models
- `recording-error` — frontend shows error toast
- `history-updated` — frontend refreshes history list

## Entry Points

**Backend Launch:**

Location: `src-tauri/src/main.rs`
Triggers: Application startup
Responsibilities:
1. Parse CLI arguments via clap
2. Call `handy_app_lib::run(cli_args)` → `src-tauri/src/lib.rs::run()`
3. Initialize Tauri builder with plugins
4. Set up logging (console + file with rotation)
5. Register all commands via specta code generation
6. Initialize managers and put in Tauri state
7. Register shortcut handlers and tray icon

**Frontend Launch:**

Location: `src/App.tsx`
Triggers: Browser/webview load
Responsibilities:
1. Initialize i18next for translations
2. Check onboarding status (accessibility, model selection)
3. Set up event listeners for backend events
4. Initialize Zustand store (fetch settings, audio devices)
5. Render onboarding or main interface
6. Set up keyboard shortcuts (Cmd+Shift+D for debug mode)

**Command Handler Pattern:**

Location: `src-tauri/src/commands/*.rs`
Pattern:
```rust
#[tauri::command]
#[specta::specta]
pub fn command_name(app: AppHandle, state: State<Manager>) -> Result<Type, String> {
    // Access manager via state::<Manager>()
    // Access app via app.state::<Manager>()
    // Return Result<T, String> for error handling
}
```
- `#[tauri::command]` — Registers with Tauri
- `#[specta::specta]` — Generates TypeScript bindings
- Failures returned as `Err(String)` → JSON error response

## Error Handling

**Strategy:** Explicit error propagation with recovery where possible, silent fallback otherwise.

**Patterns:**

1. **Result<T, String>** — Standard for commands, converts to frontend error
   - Example: `commands::models::download_model` returns `Result<(), String>`

2. **Panic-Safe RAII** — Guards ensure state consistency even on panic
   - Example: LoadingGuard via `catch_unwind` in TranscriptionManager
   - `std::panic::catch_unwind(AssertUnwindSafe(...))` wraps critical sections

3. **Silent Fallback** — Non-critical failures log and continue
   - Example: Audio mute on unsupported platform → logs, doesn't error
   - Example: Custom sounds missing → plays default or none

4. **User Notification** — Via toast or error event
   - Example: `RecordingErrorEvent` emitted to show UI error toast
   - Example: Microphone permission denied → prompt to settings

## Cross-Cutting Concerns

**Logging:** 
- Framework: `log` crate with tauri_plugin_log
- Console: Respects RUST_LOG env var (info-level default)
- File: Respects FILE_LOG_LEVEL atomic (debug-level default)
- Rotation: Single file kept, max 500KB per file
- Path: `{app_data}/logs/handy.log` or system app log dir

**Validation:** 
- Frontend: Zod schemas for settings forms
- Backend: Implicit via Rust type system (enums, ranges)
- Model validation: SHA256 checksums, file integrity checks

**Authentication:** 
- Accessibility (macOS): Gated at App.tsx via component
- Microphone: OS-level permission, checked on start
- Not implemented: User accounts, API tokens

**Concurrency:**
- Audio recording: Single CPAL stream per recording
- Transcription: Arc<Mutex> shared across threads, single loading thread
- Settings: TOML-JSON with write-through persistence
- Commands: Async via Tauri's runtime, thread pool for heavy work

**Internationalization (i18n):**
- Framework: i18next (React) + manual strings in Rust
- Files: `src/i18n/locales/{lang}/translation.json`
- ESLint rule: No hardcoded strings in JSX components
- Supported: English, Spanish, French, Vietnamese

---

*Architecture analysis: 2026-04-05*
