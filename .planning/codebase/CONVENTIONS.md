# Coding Conventions

**Analysis Date:** 2026-04-05

## Naming Patterns

**Files:**
- TypeScript/React components: PascalCase with `.tsx` extension (e.g., `MicrophoneSelector.tsx`, `Button.tsx`)
- TypeScript utilities/hooks: camelCase with `.ts` extension (e.g., `useSettings.ts`, `keyboard.ts`)
- Rust source files: snake_case (e.g., `settings.rs`, `audio_feedback.rs`, `transcription_coordinator.rs`)
- Rust modules: snake_case directories matching module names (e.g., `audio_toolkit/`, `managers/`)
- Index files: Used as barrel exports in `src/` structure (e.g., `src/components/settings/index.ts`)

**Functions:**
- TypeScript/React: camelCase (e.g., `handleMicrophoneSelect`, `refreshAudioDevices`, `getSetting`)
- Rust: snake_case (e.g., `get_settings`, `write_settings`, `set_mute`, `list_input_devices`)
- React hooks: camelCase with `use` prefix (e.g., `useSettings`, `useTranslation`, `useOsType`)
- Rust manager methods: snake_case, public functions use pub visibility (e.g., `pub fn initialize()`)

**Variables:**
- TypeScript/React: camelCase for all variables (e.g., `selectedMicrophone`, `audioDevices`, `isLoading`)
- Rust: snake_case (e.g., `app_settings`, `audio_devices`, `is_downloading`)
- React component props: Typed interfaces with camelCase properties (e.g., `MicrophoneSelectorProps`, `SettingContainerProps`)
- Zustand store selectors: camelCase functions returning state slices (e.g., `(state) => state.isLoading`)

**Types:**
- TypeScript interfaces: PascalCase (e.g., `SettingsStore`, `UseSettingsReturn`, `AudioDevice`)
- Rust structs: PascalCase (e.g., `ModelInfo`, `DownloadProgress`, `AppSettings`)
- Rust enums: PascalCase variants with camelCase or PascalCase depending on type (e.g., `enum EngineType { Whisper, Parakeet }`)
- TypeScript type aliases: PascalCase (e.g., `SupportedLanguageCode`)

## Code Style

**Formatting:**
- Tool: Prettier with minimal config (LF line endings only)
- Config: `.prettierrc` - no extra rules beyond default Prettier behavior
- Command: `bun run format` applies Prettier to all TypeScript, JSX, JSON, and YAML files
- Run: `bun run format:check` before committing to verify no formatting violations

**Linting:**
- Tool: ESLint 9.x with @typescript-eslint parser
- Config: `eslint.config.js` (new flat config format)
- Rules: Primary enforcement is `i18next/no-literal-string` to prevent hardcoded user-facing text
- Scope: Lints `src/**/*.{ts,tsx}` only
- Run: `bun run lint` to check, `bun run lint:fix` to auto-fix violations
- Ignores: Attributes like `className`, `style`, `type`, `id`, `name`, `key`, `data-*`, `aria-*` (non-translatable)

**TypeScript:**
- Target: ES2020 with DOM and DOM.Iterable libraries
- Strict mode: `true` - enforces strict type checking throughout
- Module resolution: bundler mode with ESNext output
- JSX: React JSX transform (automatic)
- Path aliases: `@/*` → `./src/*` for absolute imports
- Unused variables: Not enforced (`noUnusedLocals: false`, `noUnusedParameters: false`)

**Rust:**
- Tool: `cargo fmt` (built-in formatter)
- Configuration: Edition 2021, default Rust formatting rules
- Linting: `cargo clippy` recommended before committing (not enforced in CI)
- Error handling: Explicit `Result<T, E>` types with `.map_err()` for conversion
- Visibility: Explicit `pub` for public APIs, private by default
- Attributes: Uses `#[tauri::command]` and `#[specta::specta]` for IPC command generation

## Import Organization

**Order (TypeScript/React):**
1. React and React ecosystem (`react`, `react-dom`, hooks)
2. External libraries (`zustand`, `i18next`, `sonner`, `zod`)
3. Tauri API (`@tauri-apps/api`, `@tauri-apps/plugin-*`)
4. Internal absolute imports (`@/components`, `@/hooks`, `@/stores`, `@/lib`)
5. Internal relative imports (`./sibling`, `../parent`)
6. CSS/style imports

Example from `src/App.tsx`:
```typescript
import { useEffect, useState, useRef } from "react";
import { toast, Toaster } from "sonner";
import { useTranslation } from "react-i18next";
import { listen } from "@tauri-apps/api/event";
import { ModelStateEvent } from "./lib/types/events";
import "./App.css";
import AccessibilityPermissions from "./components/AccessibilityPermissions";
import { useSettings } from "./hooks/useSettings";
import { commands } from "@/bindings";
```

**Order (Rust):**
1. Standard library imports (`std::*`)
2. External crates (log, serde, anyhow, etc.)
3. Internal crate modules (`crate::*`)
4. Relative module imports

**Path Aliases:**
- TypeScript: `@/` resolves to `./src/` directory
- Used consistently across all imports for clean, maintainable code
- Enables IDE navigation and refactoring

## Error Handling

**TypeScript/React:**
- Async/await with try-catch blocks for promise handling
- User-visible errors: Toast notifications via `sonner` library
  - Success: `toast.success(t("key"))`
  - Error: `toast.error(t("key"), { description: errorMessage })`
- Store actions return `Promise<void>` and handle errors internally
- Errors logged to console via `console.error()` or `console.warn()`
- Example from `src/stores/settingsStore.ts`:
```typescript
try {
  const result = await commands.getAppSettings();
  if (result.status === "ok") {
    set({ settings: result.data, isLoading: false });
  } else {
    console.error("Failed to load settings:", result.error);
    set({ isLoading: false });
  }
} catch (error) {
  console.error("Failed to load settings:", error);
  set({ isLoading: false });
}
```

**Rust:**
- Functions return `Result<T, String>` where T is the success type
- Error messages formatted as user-readable strings: `format!("Failed to {}: {}", action, error)`
- Platform-specific error handling with `#[cfg()]` attributes
- Logging via `log` crate (debug, info, warn, error levels)
- Example from `src-tauri/src/commands/mod.rs`:
```rust
#[tauri::command]
#[specta::specta]
pub fn get_app_dir_path(app: AppHandle) -> Result<String, String> {
    let app_data_dir = crate::portable::app_data_dir(&app)
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    Ok(app_data_dir.to_string_lossy().to_string())
}
```

**Backend → Frontend Error Communication:**
- Tauri events emit errors for significant operations
- Event types: `recording-error` (with `error_type` and `detail` fields), `model-state-changed` (with `event_type` and error info)
- Frontend listens via `listen<EventType>("event-name", callback)` from `@tauri-apps/api/event`

## Logging

**Framework (Rust):** `log` crate with `tauri-plugin-log` for file output
- Levels: Off, Error, Warn, Info, Debug, Trace
- File logging: Rotated logs in platform-specific app log directory
- Log level controlled by `RUST_LOG` env var or set via `setLogLevel` command
- Global atomic `FILE_LOG_LEVEL` tracks file logging level dynamically

**Framework (TypeScript):** `console` object (no dedicated logger)
- `console.log()` - general information (not used in production code)
- `console.warn()` - recoverable issues
- `console.error()` - unrecoverable errors
- No structured logging; messages are strings

**Patterns:**
- Rust logs operations at appropriate level (info for startup, debug for normal flow, error for failures)
- TypeScript logs to console primarily for debugging and error reporting
- User-visible errors go via toast notifications, not console

## Comments

**When to Comment (TypeScript/React):**
- Explain non-obvious business logic (e.g., why a setting needs special handling)
- Document complex conditional logic (see `App.tsx` permission checking flow)
- Inline comments for subtle workarounds or platform-specific behavior
- Avoid obvious comments restating the code

Example from `src/App.tsx`:
```typescript
// Track if this is a returning user who just needs to grant permissions
// (vs a new user who needs full onboarding including model selection)
const [isReturningUser, setIsReturningUser] = useState(false);
```

**When to Comment (Rust):**
- Explain non-obvious unsafe code blocks
- Document expected behavior of complex algorithms
- Clarify platform-specific implementations (Windows/macOS/Linux differences)
- Add doc comments for public APIs (exported functions/types)

Example from `src-tauri/src/managers/audio.rs`:
```rust
// Expected behavior:
// - Windows: works on most systems using standard audio drivers.
// - Linux: works on many systems (PipeWire, PulseAudio, ALSA),
//   but some distros may lack the tools used.
// - macOS: works on most standard setups via AppleScript.
```

**JSDoc/TSDoc:**
- Not enforced; inline comments preferred for clarity
- Function types are self-documenting via TypeScript interfaces
- No required doc comments for internal functions

## Function Design

**Size:**
- Keep functions under 50 lines (aim for readability)
- React components can exceed 50 lines if complex (see `App.tsx` at 275 lines)
- Rust command handlers typically short (5-20 lines)
- Utility functions prefer single responsibility (<30 lines)

**Parameters:**
- TypeScript: Use typed objects instead of multiple parameters
  - Example: `updateSetting<K>(key: K, value: V)` instead of 5+ params
- React: Props via interface (PascalCase with `Props` suffix)
- Rust: AppHandle first, then specific parameters
  - Example: `pub fn get_app_settings(app: AppHandle) -> Result<AppSettings, String>`

**Return Values:**
- TypeScript async functions return `Promise<T>`
- TypeScript store actions return `Promise<void>` (errors handled internally)
- Rust functions return `Result<T, String>` for fallible operations
- React components return JSX.Element (or null for conditional rendering)
- Functions follow early-return pattern for error cases

## Module Design

**Exports:**
- TypeScript: Named exports preferred, default exports for React components
  - Utilities: `export const functionName = () => {}`
  - Components: `export default ComponentName` (allows `import ComponentName from "path"`)
  - Hooks: `export const useHook = () => {}`
  
- Rust: Explicit `pub` visibility for public APIs
  - Modules: `pub mod module_name;`
  - Functions: `pub fn function_name() {}`
  - Structs/Types: `pub struct StructName {}` and `#[derive(...)]`

**Barrel Files:**
- Used in `src/components/settings/index.ts`, `src/components/ui/index.ts`, `src/components/onboarding/index.ts`
- Pattern: Re-export all components from a directory for easier imports
  - `export * from "./MicrophoneSelector";`
  - Imported as: `import { MicrophoneSelector } from "@/components/settings";`
- Not used in Rust (modules imported via `mod module_name;` in `lib.rs`)

## Internationalization (i18n)

**Framework:** i18next with React i18n integration
- Config: `src/i18n/index.ts` initializes i18next with auto-discovered locale files
- Locales: `src/i18n/locales/{code}/translation.json` (18+ languages supported)
- Metadata: `src/i18n/languages.ts` defines language name, native name, priority, and RTL direction

**Usage in Components:**
- Hook: `const { t, i18n } = useTranslation();`
- Translate strings: `t("key.path")` returns translated string
- With interpolation: `t("key", { variable: value })`
- With fallback: `t("key", { defaultValue: "fallback text" })`

**Enforcement:**
- ESLint rule `i18next/no-literal-string` prevents hardcoded strings in JSX
- Ignore list: `className`, `style`, `type`, `id`, `name`, `key`, `data-*`, `aria-*` attributes
- All user-facing text MUST use translation keys

**Example from `src/components/settings/MicrophoneSelector.tsx`:**
```typescript
<Dropdown
  options={microphoneOptions}
  placeholder={t("settings.sound.microphone.placeholder")}
  disabled={isUpdating("selected_microphone")}
/>
```

**RTL Support:**
- Auto-detected based on language (Arabic, Hebrew marked with `direction: "rtl"`)
- Document `dir` attribute updated via `updateDocumentDirection()`
- Components support RTL via Tailwind's `dir:` selector or parent `dir` attribute
- Example from `src/App.tsx`: `<div dir={direction} className="h-screen flex...">`

---

*Convention analysis: 2026-04-05*
