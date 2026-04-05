# Codebase Structure

**Analysis Date:** 2026-04-05

## Directory Layout

```
dictus-desktop/
├── src/                              # React frontend (TypeScript)
│   ├── App.tsx                       # Entry point, onboarding orchestrator
│   ├── components/
│   │   ├── ui/                       # Reusable UI components (Button, Input, etc.)
│   │   ├── settings/                 # 35+ settings pages (Model, Audio, Shortcuts, etc.)
│   │   ├── model-selector/           # Model download/selection interface
│   │   ├── onboarding/               # First-run flow (accessibility, model)
│   │   ├── footer/                   # Recording footer with status
│   │   ├── icons/                    # SVG icon components
│   │   ├── shared/                   # Cross-feature shared components
│   │   └── update-checker/           # Update notification
│   ├── hooks/
│   │   ├── useSettings.ts            # Settings state management hook
│   │   └── useOsType.ts              # Platform detection hook
│   ├── stores/
│   │   ├── settingsStore.ts          # Zustand store for app settings
│   │   └── modelStore.ts             # Zustand store for model management
│   ├── lib/
│   │   ├── types/                    # TypeScript interfaces (from Rust bindings)
│   │   ├── constants/                # App constants, shortcuts, models list
│   │   └── utils/                    # Utility functions (RTL, clipboard, etc.)
│   ├── i18n/
│   │   ├── index.ts                  # i18next configuration
│   │   ├── locales/
│   │   │   ├── en/translation.json   # English (source)
│   │   │   ├── es/translation.json   # Spanish
│   │   │   ├── fr/translation.json   # French
│   │   │   └── vi/translation.json   # Vietnamese
│   │   └── languages.ts              # Language metadata
│   ├── overlay/                      # Recording overlay window code
│   └── bindings.ts                   # Auto-generated Tauri type bindings (commit to git)
│
├── src-tauri/                        # Rust backend
│   ├── src/
│   │   ├── lib.rs                    # Main entry point, Tauri setup, manager init
│   │   ├── main.rs                   # CLI args parsing, calls run()
│   │   ├── managers/
│   │   │   ├── mod.rs                # Module exports
│   │   │   ├── transcription.rs       # Speech→text pipeline, model loading/unloading
│   │   │   ├── audio.rs              # Audio device/recording state, mute control
│   │   │   ├── model.rs              # Model download/deletion, validation
│   │   │   ├── history.rs            # Transcription history database
│   │   │   └── transcription_mock.rs # Mock engine for testing
│   │   ├── commands/
│   │   │   ├── mod.rs                # Command registration, settings/app commands
│   │   │   ├── transcription.rs       # Model load status, unload
│   │   │   ├── models.rs             # Download, delete, switch models
│   │   │   ├── audio.rs              # Mic/speaker device selection, permissions
│   │   │   └── history.rs            # Fetch, toggle, delete history entries
│   │   ├── audio_toolkit/
│   │   │   ├── mod.rs                # Exports public API
│   │   │   ├── audio/
│   │   │   │   ├── mod.rs            # Re-exports AudioRecorder, device APIs
│   │   │   │   ├── recorder.rs        # CPAL stream, frame capture, microphone checks
│   │   │   │   ├── device.rs         # Device enumeration (input/output)
│   │   │   │   ├── resampler.rs      # Frame rate conversion
│   │   │   │   ├── visualizer.rs     # Real-time waveform data
│   │   │   │   └── utils.rs          # WAV I/O, CPAL host
│   │   │   ├── vad/
│   │   │   │   ├── mod.rs            # Trait definition, exports
│   │   │   │   ├── silero.rs         # Silero VAD v4 ONNX implementation
│   │   │   │   └── smoothed.rs       # Smoothing filter for VAD output
│   │   │   ├── text.rs               # Custom words substitution, output filtering
│   │   │   ├── constants.rs          # Audio config (sample rates, frame sizes)
│   │   │   ├── utils.rs              # CPAL host singleton
│   │   │   └── bin/
│   │   │       └── cli.rs            # Standalone audio CLI tool
│   │   ├── shortcut/
│   │   │   ├── mod.rs                # Initialization, trait dispatch
│   │   │   ├── handler.rs            # Input event handling
│   │   │   ├── tauri_impl.rs         # Tauri global-shortcut implementation
│   │   │   ├── handy_keys.rs         # HandyKeys library implementation
│   │   │   └── [command fns]         # Setting change handlers
│   │   ├── transcription_coordinator.rs  # Event serializer state machine
│   │   ├── actions.rs                # ShortcutAction trait, TranscribeAction impl
│   │   ├── settings.rs               # Settings struct, defaults, persistence
│   │   ├── tray.rs                   # System tray menu, icon management
│   │   ├── overlay.rs                # Recording overlay window
│   │   ├── clipboard.rs              # Clipboard operations
│   │   ├── input.rs                  # Keyboard/mouse simulation (Enigo)
│   │   ├── llm_client.rs             # HTTP client for post-processing LLM
│   │   ├── audio_feedback.rs         # Audio notifications (start/stop beeps)
│   │   ├── apple_intelligence.rs     # macOS native post-processing
│   │   ├── signal_handle.rs          # Unix signal handling (Ctrl+C, SIGUSR1)
│   │   ├── portable.rs               # Portable mode detection
│   │   ├── cli.rs                    # CLI argument definitions (clap)
│   │   ├── utils.rs                  # Utility functions (cancel, overlay, tray)
│   │   ├── helpers/
│   │   │   ├── mod.rs                # Module exports
│   │   │   ├── clamshell.rs          # Laptop lid/clamshell detection
│   │   │   └── [others]              # Platform-specific helpers
│   │   └── Cargo.toml                # Rust dependencies
│   ├── Cargo.toml                    # Workspace config
│   ├── build.rs                      # Build script (embed resources)
│   └── src-tauri.conf.json           # Tauri config (window, plugins, updater)
│
├── tests/                            # E2E tests
│   └── [test files]                  # Playwright tests (optional)
│
├── package.json                      # Frontend dependencies
├── tsconfig.json                     # TypeScript config
├── eslintrc.js                       # ESLint config (no hardcoded strings rule)
├── prettier.config.js                # Code formatter config
├── vite.config.ts                    # Frontend bundler config
├── .env.example                      # Example environment vars (DO NOT commit .env)
└── CLAUDE.md                         # Development guidance
```

## Directory Purposes

**src/:**
- **Purpose:** React + TypeScript frontend for user interface
- **Contains:** Components, hooks, stores, translations, auto-generated bindings
- **Key files:** App.tsx (root), bindings.ts (types from Rust)

**src/components/:**
- **Purpose:** React component library
- **Contains:** UI primitives, page-level components, feature-specific UIs
- **Key files:** Settings folder (35+ pages), model-selector, onboarding

**src/stores/:**
- **Purpose:** Zustand state management
- **Contains:** Shared mutable state across components
- **Key files:** settingsStore (app configuration), modelStore (downloads)

**src/hooks/:**
- **Purpose:** React custom hooks for state/side-effects
- **Contains:** useSettings (initialize + update settings), useOsType (platform detection)
- **Pattern:** Each hook wraps a Zustand store and manages initialization

**src/i18n/:**
- **Purpose:** Internationalization (translations)
- **Contains:** i18next config, JSON translation files
- **Key pattern:** All user text must be in JSON, imported via `useTranslation()`

**src-tauri/src/managers/:**
- **Purpose:** Core business logic (Rust)
- **Contains:** Domain-specific state machines and operations
- **Key pattern:** Each manager is Arc<Mutex<T>>, cloned across threads
- **Examples:**
  - TranscriptionManager: Model loading/transcribing, idle timeout watcher
  - AudioRecordingManager: CPAL stream lifecycle, device selection
  - ModelManager: Download, delete, validate model files
  - HistoryManager: SQL database of past transcriptions

**src-tauri/src/commands/:**
- **Purpose:** RPC handlers (command interface)
- **Contains:** `#[tauri::command]` functions that call managers
- **Pattern:** One file per domain (models.rs, audio.rs, transcription.rs)
- **Output:** Auto-generates TypeScript bindings (bindings.ts)

**src-tauri/src/audio_toolkit/:**
- **Purpose:** Low-level audio capture and processing
- **Contains:** CPAL wrapper, VAD, resampling, device enumeration
- **Pattern:** Modular sub-modules, public API re-exported in mod.rs
- **Not:** Transcription models (handled by transcribe-rs crate)

**src-tauri/src/shortcut/:**
- **Purpose:** Global keyboard shortcut handling
- **Contains:** Tauri native and HandyKeys implementations, setting change handlers
- **Key pattern:** Two implementations, runtime switchable
- **Files:** mod.rs (dispatch), tauri_impl.rs (Tauri), handy_keys.rs (HandyKeys)

**src-tauri/src/helpers/:**
- **Purpose:** Platform-specific utilities
- **Contains:** Clamshell detection, OS checks, system integration
- **Not:** General utilities (those in utils.rs)

## Key File Locations

**Entry Points:**

- `src/App.tsx`: React root component, onboarding orchestrator
- `src-tauri/src/main.rs`: Binary entry point, parses CLI args
- `src-tauri/src/lib.rs`: `run()` function, Tauri setup, manager initialization
- `vite.config.ts`: Frontend build configuration
- `src-tauri/src-tauri.conf.json`: Tauri window, plugins, update config

**Configuration:**

- `src-tauri/Cargo.toml`: Rust dependencies (transcribe-rs, tauri, etc.)
- `package.json`: Frontend dependencies (React, Tauri plugins, etc.)
- `tsconfig.json`: TypeScript strict mode, path aliases
- `.prettierrc.js`: Code formatting rules
- `.eslintrc.js`: Linting rules (enforces i18n for strings)
- `src-tauri/src/settings.rs`: AppSettings struct, defaults, persistence

**Core Logic:**

- `src-tauri/src/managers/transcription.rs`: Speech→text pipeline
- `src-tauri/src/transcription_coordinator.rs`: Event serialization state machine
- `src-tauri/src/actions.rs`: ShortcutAction trait, implementations
- `src-tauri/src/audio_toolkit/audio/recorder.rs`: CPAL stream and frame capture
- `src/stores/settingsStore.ts`: Zustand store, API layer to backend

**Testing:**

- `tests/`: E2E tests with Playwright (optional)
- `src-tauri/src/managers/transcription_mock.rs`: Mock transcription engine

**Build Artifacts (not committed):**

- `dist/`: Frontend bundle (Vite)
- `src-tauri/target/`: Rust build output
- `node_modules/`: Frontend dependencies

## Naming Conventions

**Files:**

- **Rust:** snake_case (e.g., `transcription_manager.rs`, `audio_recorder.rs`)
- **TypeScript:** PascalCase for components (e.g., `ModelSelector.tsx`), camelCase for utilities
- **Settings pages:** PascalCase (e.g., `AudioSettings.tsx`, `ShortcutsSettings.tsx`)
- **Tests:** `*.test.ts` or `*.spec.ts` suffix

**Directories:**

- **Rust modules:** snake_case, plural for collections (e.g., `managers/`, `commands/`)
- **React components:** PascalCase (e.g., `src/components/settings/`)
- **Features:** kebab-case (e.g., `model-selector/`, `onboarding/`)

**Functions & Variables:**

- **Rust:** snake_case (e.g., `download_model()`, `get_settings()`)
- **TypeScript:** camelCase (e.g., `useSettings()`, `refreshAudioDevices()`)
- **React components:** PascalCase (e.g., `ModelSelector`, `SettingsPanel`)

**Types & Enums:**

- **Rust:** PascalCase (e.g., `TranscriptionManager`, `EngineType`, `ModelInfo`)
- **TypeScript:** PascalCase (e.g., `AppSettings`, `AudioDevice`, `ModelStateEvent`)

## Where to Add New Code

**New Feature (End-to-End):**

Backend:
1. Add new manager if needed: `src-tauri/src/managers/{feature}.rs`
2. Add commands: `src-tauri/src/commands/{feature}.rs`
3. Register commands in `src-tauri/src/lib.rs::run()` (collect_commands! macro)
4. Emit events if needed: `app.emit("event-name", data)`

Frontend:
1. Add component: `src/components/{feature}/` directory
2. Add translations: `src/i18n/locales/en/translation.json` (all languages)
3. Add store if shared state needed: `src/stores/{feature}Store.ts`
4. Add to App.tsx routing or Sidebar config

**New Setting (Configuration):**

Backend:
1. Add field to `AppSettings` struct in `src-tauri/src/settings.rs`
2. Add default value in `get_default_settings()`
3. Add change handler command in `src-tauri/src/shortcut/` (one of the `change_*_setting` functions)
4. Rebuild bindings (debug build auto-exports to src/bindings.ts)

Frontend:
1. Settings page component in `src/components/settings/{Setting}.tsx`
2. Add `settingUpdaters` entry in `src/stores/settingsStore.ts` (maps key to command)
3. Add to sidebar: `src/components/Sidebar.tsx` (SECTIONS_CONFIG)
4. Add translations to all language files

**New Audio Device or Keyboard Shortcut:**

Device:
1. `src-tauri/src/audio_toolkit/audio/device.rs` — list operation
2. `src-tauri/src/managers/audio.rs` — device selection logic
3. Command in `src-tauri/src/commands/audio.rs`

Shortcut:
1. Binding definition in `src-tauri/src/settings.rs::ShortcutBinding`
2. Default in `get_default_settings()`
3. Handler in `src-tauri/src/shortcut/handler.rs` (call matching action)
4. Change function in `src-tauri/src/shortcut/` + register in lib.rs
5. UI in settings: `src/components/settings/KeyboardShortcuts.tsx`

**New Model or Transcription Engine:**

1. Add variant to `EngineType` enum in `src-tauri/src/managers/model.rs`
2. Add `ModelInfo` entry in model list (hardcoded or from config)
3. Add loading logic in `TranscriptionManager` (match EngineType)
4. Add support in both implementations: `tauri_impl.rs` and `handy_keys.rs`
5. Update model selector UI: `src/components/model-selector/`

**Utilities:**

Shared (used across domains):
- Rust: `src-tauri/src/utils.rs`
- TypeScript: `src/lib/utils/`

Module-specific:
- Rust: Within the manager module (private or pub)
- TypeScript: `src/{feature}/utils.ts` or within component folder

## Special Directories

**src/bindings.ts:**
- **Purpose:** Auto-generated TypeScript types from Rust (command inputs/outputs, events)
- **Generated by:** tauri-specta on debug build (from `#[specta::specta]` macros)
- **Committed:** Yes, to git (shared with team)
- **Do not edit:** Edit Rust types instead, rebuild

**src-tauri/target/:**
- **Purpose:** Rust build output
- **Generated:** By `cargo build` / `bun run tauri build`
- **Committed:** No

**node_modules/:**
- **Purpose:** Frontend dependencies
- **Generated:** By `bun install`
- **Committed:** No

**src-tauri/resources/:**
- **Purpose:** Assets bundled with application
- **Contains:** Models (Silero VAD), icons, sounds
- **Key file:** `models/silero_vad_v4.onnx` (downloaded at setup, not in repo)

**dist/:**
- **Purpose:** Frontend bundle output
- **Generated:** By `vite build`
- **Committed:** No

**src-tauri/resources/models/:**
- **Generated:** Yes (models downloaded at runtime)
- **Committed:** No (add to .gitignore)

---

*Structure analysis: 2026-04-05*
