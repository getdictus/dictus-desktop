# Technology Stack

**Analysis Date:** 2026-04-05

## Languages

**Primary:**

- Rust 2021 edition - Backend (speech-to-text pipeline, audio recording, model management)
- TypeScript ~5.6.3 - Frontend (React UI, configuration, settings)
- React 18.3.1 - UI framework for frontend

**Secondary:**

- JSON - Configuration and i18n localization
- Shell/Bash - Build scripts and CLI utilities
- NSIS - Windows installer scripting

## Runtime

**Environment:**

- Tauri 2.10.2 - Cross-platform desktop application framework
- Node.js/Bun - Frontend build and development
- Rust (stable) - Backend compilation and execution

**Package Manager:**

- Bun (primary) - Frontend dependencies and scripts
- Cargo - Rust backend dependencies and compilation
- npm/Bun lockfile management (bun.lock)

## Frameworks

**Core:**

- Tauri 2.10.2 - Desktop framework (Rust + WebView)
  - Features enabled: protocol-asset, macos-private-api, tray-icon, image-png
- React 18.3.1 - Frontend UI framework
- Vite 6.4.1 - Frontend build tool and dev server

**UI & Styling:**

- Tailwind CSS 4.1.16 - Utility CSS framework
- Lucide React 0.542.0 - Icon library
- React Select 5.8.0 - Dropdown/select component
- Sonner 2.0.7 - Toast notification library

**Testing:**

- Playwright 1.58.0 - E2E/browser testing
- Configuration: `playwright.config.ts` (Chromium only)

**Build/Dev:**

- @vitejs/plugin-react 4.7.0 - React support for Vite
- @tailwindcss/vite 4.1.16 - Tailwind CSS Vite integration
- TypeScript - Type checking and compilation
- ESLint 9.39.1 - JavaScript linting
- Prettier 3.6.2 - Code formatting

## Key Dependencies

**Backend - Audio Processing:**

- cpal 0.16.0 - Cross-platform audio device handling
- rubato 0.16.2 - Audio resampling
- hound 3.5.1 - WAV file I/O
- rodio - Audio playback (forked from rustdesk-org)

**Backend - Speech Recognition:**

- transcribe-rs 0.3.8 - Unified transcription API
  - Features: whisper-cpp, onnx
  - Platform-specific acceleration:
    - Windows: whisper-vulkan, ort-directml
    - macOS: whisper-metal
    - Linux: whisper-vulkan
- vad-rs - Voice Activity Detection (VAD) engine (forked from cjpais)

**Backend - Model Management:**

- rusqlite 0.37 - SQLite database client
- rusqlite_migration 2.3 - Database migration framework
- tar 0.4.44 - Tar archive extraction
- flate2 1.0 - gzip decompression
- sha2 0.10 - SHA256 hashing for model verification

**Backend - HTTP & Networking:**

- reqwest 0.12 - HTTP client (with json, stream features)
- futures-util 0.3 - Async utilities

**Backend - System Integration:**

- rdev - Global keyboard event handling (forked from rustdesk-org)
- enigo 0.6.1 - Input simulation (keyboard/mouse)
- tauri-nspanel (macOS only) - Native panel support for overlay
- gtk-layer-shell 0.8 (Linux only) - Layer shell protocol support
- gtk 0.18 (Linux only) - GTK bindings for Linux
- windows 0.61.3 (Windows only) - Windows API bindings

**Backend - Core Utilities:**

- serde/serde_json 1 - Serialization/deserialization
- tokio 1.43.0 - Async runtime
- anyhow 1.0.95 - Error handling
- log 0.4.25 - Logging
- env_filter 0.1.0 - Log filtering
- chrono 0.4 - Date/time utilities
- clap 4 - CLI argument parsing (with derive feature)

**Backend - LLM & Text Processing:**

- ferrous-opencc 0.2.3 - Traditional/Simplified Chinese conversion
- strsim 0.11.0 - String similarity (for custom word matching)
- natural 0.5.0 - Natural language processing
- regex 1 - Regular expressions
- rustfft 6.4.0 - FFT for audio analysis

**Frontend - State Management:**

- Zustand 5.0.8 - Lightweight state management
- Immer 11.1.3 - Immutable state updates

**Frontend - Internationalization:**

- i18next 25.7.2 - i18n framework
- react-i18next 16.4.1 - React i18n bindings
- Support: 21 languages (ar, bg, cs, de, en, es, fr, he, it, ja, ko, pl, pt, ru, sv, tr, uk, vi, zh, zh-TW)

**Frontend - Validation:**

- Zod 3.25.76 - TypeScript-first schema validation

**Tauri Plugins:**

- @tauri-apps/api 2.10.0 - Core Tauri API bindings
- @tauri-apps/plugin-clipboard-manager 2.3.2 - Clipboard access
- @tauri-apps/plugin-dialog 2.6 - File/folder dialogs
- @tauri-apps/plugin-fs 2.4.4 - Filesystem operations
- @tauri-apps/plugin-global-shortcut 2.3.1 - Global keyboard shortcuts
- @tauri-apps/plugin-opener 2.5.2 - URL/file opening
- @tauri-apps/plugin-os 2.3.2 - OS information
- @tauri-apps/plugin-process 2.3.1 - Process control
- @tauri-apps/plugin-sql 2.3.1 - SQL database (not currently used in favor of rusqlite)
- @tauri-apps/plugin-store 2.4.1 - Key-value settings storage
- @tauri-apps/plugin-autostart 2.5.1 - Application autostart
- @tauri-apps/plugin-updater 2.10.0 - In-app auto-update
- @tauri-apps/plugin-single-instance 2.3.2 - Single instance enforcement (CLI remote control)
- tauri-plugin-log 2.7.1 - Structured logging with file rotation
- tauri-plugin-macos-permissions-api 2.3.0 - macOS permissions

**Development:**

- @types/react 18.3.26 - React type definitions
- @types/react-dom 18.3.7 - React DOM type definitions
- @types/react-select 5.0.1 - React Select types
- @types/node 24.9.1 - Node.js types
- @typescript-eslint/parser 8.49.0 - TypeScript ESLint parser
- @typescript-eslint/eslint-plugin 8.49.0 - TypeScript ESLint rules
- eslint-plugin-i18next 6.1.3 - i18next ESLint rules (enforces i18n usage)
- @tauri-apps/cli 2.10.0 - Tauri CLI for building

## Configuration

**Environment:**

- Settings persisted via tauri-plugin-store in JSON format
- Key settings: transcription models, audio devices, paste methods, keyboard shortcuts, overlay position, post-processing provider, language preferences
- CLI arguments support via clap (--toggle-transcription, --cancel, --start-hidden, --no-tray, --debug)

**Build:**

- `vite.config.ts` - Vite build configuration with multiple entry points (main app + overlay window)
- `tsconfig.json` - TypeScript compiler settings (ES2020 target, strict mode)
- `src-tauri/tauri.conf.json` - Tauri app configuration (bundle settings, updater, macOS permissions)
- `.prettierrc` - Code formatting rules
- ESLint configuration - Enforces i18next usage, TypeScript strict rules
- `playwright.config.ts` - E2E testing configuration

## Platform Requirements

**Development:**

- Rust (latest stable) - Backend compilation
- Bun or Node.js - Frontend tooling
- CMake 3.5+ (macOS) - Whisper C++ compilation
- Platform-specific dependencies:
  - macOS: Xcode Command Line Tools, entitlements configuration
  - Windows: Visual Studio Build Tools, code signing certificates
  - Linux: GTK development headers, libgtk-layer-shell0

**Production:**

- macOS 10.15+ (Catalina minimum)
- Windows 7+ with Visual C++ Runtime
- Linux with GTK 3.18+ and X11/Wayland support
- Audio hardware with supported input devices

**Signing & Distribution:**

- macOS: Code signing identity, hardened runtime, entitlements
- Windows: Azure Code Signing certificates (trusted-signing-cli)
- Updates via GitHub releases with signature verification

---

_Stack analysis: 2026-04-05_
