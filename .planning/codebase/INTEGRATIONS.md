# External Integrations

**Analysis Date:** 2026-04-05

## APIs & External Services

**LLM Post-Processing (Optional):**
- OpenAI-compatible API providers (Anthropic, OpenAI, DeepSeek, etc.)
  - SDK/Client: reqwest 0.12 (generic HTTP client)
  - Auth: API key passed via environment/settings, sent as Bearer token or x-api-key header
  - Usage: Optional post-processing of transcription text via `/chat/completions` endpoint
  - Details: `src-tauri/src/llm_client.rs` handles provider abstraction
  - Supported: system prompts, structured JSON schema responses

**Model Download Service:**
- Host: `https://blob.handy.computer`
  - SDK/Client: reqwest 0.12 (with stream feature)
  - Purpose: Downloads pre-trained speech-to-text models (Whisper, Parakeet, Moonshine, etc.)
  - Verification: SHA256 checksum validation after download
  - Details: `src-tauri/src/managers/model.rs` manages downloads, extraction, caching
  - Models are tar.gz compressed and extracted to `$APP_DATA/models/`

## Data Storage

**Databases:**
- SQLite (embedded)
  - Connection: File-based, located at `$APP_DATA/history.db`
  - Client: rusqlite 0.37 (direct binding, not ORM)
  - Migration: rusqlite_migration 2.3 (versioned schema updates)
  - Schema: `src-tauri/src/managers/history.rs`
  - Tables:
    - `transcription_history`: id, file_name, timestamp, saved, title, transcription_text, post_processed_text, post_process_prompt, post_process_requested
  - Usage: Stores all user transcriptions with metadata and post-processing results

**Settings Storage:**
- Plugin: tauri-plugin-store 2.4.1
  - Format: JSON key-value pairs
  - Location: Platform-dependent (Linux: `~/.config/com.pais.handy/`, macOS: `~/Library/Application Support/com.pais.handy/`, Windows: `%APPDATA%\com.pais.handy`)
  - Persisted via `write_settings()` / `get_settings()` functions in `src-tauri/src/settings.rs`
  - Keys stored: Models, audio devices, shortcuts, UI preferences, overlay settings, post-processing config

**File Storage:**
- Local filesystem only
  - Recording audio files: `$APP_DATA/recordings/`
  - Model files: `$APP_DATA/models/`
  - Database: `$APP_DATA/history.db`
  - Logs: `$APP_DATA/logs/` (tauri-plugin-log)
  - No cloud storage integration

**Caching:**
- In-memory model caching (TranscriptionManager)
- Idle-based model unloading with configurable timeout (from 0s to 1 hour)
- No external caching service (all local)

## Authentication & Identity

**Auth Provider:**
- None for app core functionality
- Optional: User-managed LLM API keys (Anthropic, OpenAI, DeepSeek)
  - Stored in local settings
  - Sent directly from client to LLM providers
  - No intermediate authentication service

## Monitoring & Observability

**Error Tracking:**
- None implemented
- Local logging only

**Logs:**
- File-based via tauri-plugin-log 2.7.1
  - Location: `$APP_DATA/logs/`
  - Rotation: Daily (RotationStrategy::Daily)
  - Format: Structured logs (debug, info, warn, error, trace levels)
  - Configured in: `src-tauri/src/lib.rs`
  - Console output available in dev mode with RUST_LOG env var

## CI/CD & Deployment

**Hosting:**
- GitHub releases (for updates)
  - Endpoint: `https://github.com/cjpais/Handy/releases/latest/download/latest.json`
  - Mechanism: Tauri updater plugin with signature verification
  - Artifacts: Platform-specific installers (dmg for macOS, exe for Windows, AppImage/deb for Linux)

**CI Pipeline:**
- No explicit CI service integrated in visible config
- Build managed through Tauri CLI locally
- Windows signing: Azure Code Signing (trusted-signing-cli)

## Environment Configuration

**Required env vars:**
- None mandatory for core functionality
- Optional:
  - `RUST_LOG` - Logging filter (debug, info, warn, error, trace)
  - `TAURI_DEV_HOST` - Dev server host override
  - `CMAKE_POLICY_VERSION_MINIMUM` - For macOS cmake compatibility
  - Model-specific: LLM API keys (anthropic_key, openai_api_key, deepseek_api_key, etc.) passed via settings UI, not env

**Secrets location:**
- Settings store (JSON files in app config directory)
- API keys are user-provided at runtime via Settings UI
- No .env file usage detected

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- Single-instance callback: CLI args routed to running instance via tauri-plugin-single-instance
- Model downloads: HTTP GET requests to `https://blob.handy.computer` for model files

## Platform-Specific Integrations

**macOS:**
- Private API access enabled (via tauri feature flag)
- Metal GPU acceleration for Whisper (transcribe-rs with whisper-metal feature)
- NSPanel support for overlay window (via tauri-nspanel fork)
- Permissions API for microphone/privacy access (tauri-plugin-macos-permissions-api)
- Code signing and entitlements (Entitlements.plist)

**Windows:**
- DirectML acceleration for Whisper (transcribe-rs with ort-directml feature)
- Vulkan support via ANGLE (transcribe-rs with whisper-vulkan feature)
- Windows API bindings (win32 audio endpoints, COM, etc.)
- Registry access for system settings (winreg 0.55)
- NSIS installer with custom script template
- Azure Code Signing for release builds

**Linux:**
- Vulkan GPU acceleration for Whisper (transcribe-rs with whisper-vulkan feature)
- GTK 3 integration for UI
- Layer shell protocol support for overlay (gtk-layer-shell 0.8)
- X11/Wayland support (via gtk)
- Overlay disabled by default (feature gated)
- AppImage/deb/rpm package formats

## Speech Recognition Integrations

**Model Providers (All Local):**
- Whisper (OpenAI) - via transcribe-rs whisper-cpp backend
  - Models: small (465MB), medium (469MB), large v3 (1031MB), large v3 turbo (1549MB), breeze-asr (1030MB Taiwanese)
  - Download source: `https://blob.handy.computer`
  - Languages: 99 languages supported

- Parakeet (NVIDIA) - via transcribe-rs ONNX backend
  - Models: v2 (451MB English-only), v3 (456MB, 25 European languages)
  - Download source: `https://blob.handy.computer`

- Moonshine (Fluent AI) - via transcribe-rs ONNX backend
  - Models: base (55MB), tiny streaming (31MB), small streaming (99MB)
  - Download source: `https://blob.handy.computer`

- SenseVoice (Alibaba) - via transcribe-rs ONNX backend
  - Models: Available via ONNX runtime

- GigaAM (via transcribe-rs ONNX)
- Canary (via transcribe-rs ONNX)
- Cohere (via transcribe-rs ONNX)

All models are downloaded, cached locally, and run offline. No API calls to model providers.

## Text Post-Processing

**LLM Provider Configuration:**
- Defined in: `src-tauri/src/settings.rs` as `PostProcessProvider` struct
- Built-in providers:
  - Anthropic Claude (with x-api-key auth header)
  - OpenAI (with Bearer token auth)
  - Custom OpenAI-compatible endpoints (base URL configurable)
  - DeepSeek
  - Others via custom endpoint configuration
- Model discovery: Fetch from provider's `/models` endpoint
- Features: Structured output support (JSON schema mode) for compatible providers

## Audio System Integration

**Audio Device Discovery:**
- cpal 0.16.0 - Cross-platform audio enumeration and recording
- Device list retrieved at runtime via `cpal::available_hosts()` and device enumeration
- User selection persisted in settings

**Voice Activity Detection (VAD):**
- Silero VAD 4.0 (ONNX model)
- Model file: `src-tauri/resources/models/silero_vad_v4.onnx`
- Downloaded separately (not bundled): `https://blob.handy.computer/silero_vad_v4.onnx`
- Used to detect speech segments in audio stream
- Real-time processing with smoothing

**Audio Format:**
- Recording: PCM 16-bit, configurable sample rate (typically 16kHz for transcription)
- Resampling: rubato 0.16.2 for sample rate conversion to model requirements

## Clipboard & Input Simulation

**Clipboard Integration:**
- tauri-plugin-clipboard-manager 2.3.2 - Cross-platform clipboard access
- Usage: Copy transcriptions, paste results to active window

**Input Simulation:**
- enigo 0.6.1 - Keyboard/mouse input simulation
- Usage: Typing transcription text into active application
- Methods: Keyboard injection (Ctrl+V, direct typing), configurable per platform

## Keyboard & Shortcut System

**Global Shortcuts:**
- tauri-plugin-global-shortcut 2.3.1 - System-wide hotkey binding
- Custom shortcut bindings for start/stop recording
- Configurable via settings UI

**Keyboard Implementation:**
- Platform-dependent:
  - macOS/Windows: handy-keys (custom implementation)
  - Linux: Tauri built-in global-shortcut plugin
  - Selection via `KeyboardImplementation` enum in settings

**Physical Keyboard Support:**
- handy-keys 0.2.4 - Fallback keyboard event handling
- rdev (forked) - Cross-platform keyboard event capture

## Misc System Integrations

**Application Lifecycle:**
- Autostart: tauri-plugin-autostart 2.5.1
- Single-instance enforcement: tauri-plugin-single-instance 2.3.2
- CLI arguments passed to running instance for remote control

**System Tray:**
- Built-in Tauri tray support (with icon configuration)
- macOS: NSPanel support for main window behavior

**File Operations:**
- tauri-plugin-fs 2.4.4 - File/directory access
- Dialog boxes: tauri-plugin-dialog 2.6

---

*Integration audit: 2026-04-05*
