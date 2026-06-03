# Phase 12: Smart Modes Data Layer — Research

**Researched:** 2026-06-03
**Domain:** Rust settings schema migration, tauri-specta CRUD commands, per-mode shortcut routing
**Confidence:** HIGH — all findings verified against actual source code in this repository

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Data model shape:**
- Self-contained `SmartMode` — a Smart Mode IS the evolved prompt, not a wrapper. One type with a `kind` field.
- Single source of truth: migrate `post_process_prompts` → `smart_modes` list; drop/deprecate old fields after migration.
- One type, kind-gated fields (not a tagged Rust enum — keeps specta/CRUD/migration uniform):
  ```
  SmartMode { id, name, kind: SmartModeKind (Rewrite|Translation), prompt, target_language: Option<{code, label}> }
  ```
- Rewrite uses `prompt` + active generic instruct model.
- Translation uses `target_language` + TranslateGemma native chat template (NOT a wrapped prompt).
- `target_language` = `{ code: String, label: String }` (ISO code + display name).

**Default Smart Modes (10 total):**
- Order: Clean Up first → rewrites → translations.
- Rewrite (6): `Clean Up` (active default on fresh install), `Make Formal`, `Make Casual`, `Email`, `Bullet Points`, `Summarize`.
- Translation (4): `Translate → English`, `→ Spanish`, `→ French`, `→ Chinese`.
- Exact prompt wording NOT locked — refined during implementation against real local models.

**Migration strategy (v1.2 → v1.3):**
- Migrate existing + add defaults; no data loss.
- "Improve Transcriptions" reconciliation: if unmodified → replace with `Clean Up`; if edited → keep as custom Rewrite AND add `Clean Up` separately.
- Active selection preserved: `post_process_selected_prompt_id` maps to migrated mode.
- Introduce `settings_schema_version`; detect missing/old → migrate → stamp; idempotent; on failure leave v1.2 intact.
- Verified against actual v1.2 settings JSON fixture.

**Per-mode shortcuts:**
- Retire `transcribe_with_post_process` binding ID; transfer its key combo to the migrated active mode's per-mode shortcut.
- Plain `transcribe` binding unaffected.
- Defaults ship unbound (only the one that inherited the transferred combo is pre-bound).
- `smart_mode_{id}` binding scheme throughout shortcut + actions pipeline.
- Init registers shortcuts only for modes whose binding is set; add/update/unregister dynamically on mode create/edit/delete.
- Conflict handling: bind command returns explicit error/conflict result; never silently fails.

### Claude's Discretion
- `settings_schema_version` numbering, idempotency strategy, migration code structure.
- Init-registration and dynamic add/remove mechanics for `smart_mode_{id}` shortcuts.
- Exact prompt wording for all 6 rewrite defaults.
- Internal struct/field naming, ID generation scheme (mirror `prompt_{timestamp}`), active-mode pointer field name after rename.
- How `actions.rs` `post_process_transcription()` is refactored to accept per-mode override and branch on `kind`.

### Deferred Ideas (OUT OF SCOPE)
- Translation execution engine (TranslateGemma + native chat-template path) — Phase 13.
- Smart Modes UI (card list, create/edit/delete UI, inline shortcut bind, conflict-warning UI) — Phase 13.
- Translation preset finalization beyond EN/ES/FR/ZH — Phase 13.
- 20-locale string propagation — Phase 13 L10N-01.
- Write as SMS — cut from shipped set.
- Per-mode provider selection, enable/disable, export/import, auto-activation, gallery — Future Requirements.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| MODE-01 | On upgrade, existing post-processing prompts migrate automatically to Smart Modes with no data loss (`settings_schema_version` + explicit migration; verified against a v1.2 settings file) | Migration hook placement identified in `load_or_create_app_settings` + `ensure_post_process_defaults`; serde default story confirmed; v1.2 JSON shape known |
| MODE-02 | Dictus ships ~10 default Smart Modes with "Clean Up" as safe first mode | Default-seeding pattern confirmed (mirrors `default_post_process_prompts()`); exactly 10 modes specified |
| MODE-03 | (Backend half) Smart Modes can be created, edited, and deleted via backend commands; tauri-specta bindings regenerated | CRUD pattern fully documented from `shortcut/mod.rs:924-996`; specta registration pattern confirmed in `lib.rs:415-532` |
| MODE-04 | Each Smart Mode can have a distinct global shortcut; `smart_mode_{id}` routing through shortcut + actions pipeline | `change_binding` flow, `ACTION_MAP` extension, `is_transcribe_binding` update, `process_transcription_output` refactor all documented with exact line references |
</phase_requirements>

---

## Summary

Phase 12 is a pure backend/data-layer phase with four integration surfaces: `settings.rs` (type + migration + defaults), `shortcut/mod.rs` (CRUD commands + binding commands), `actions.rs` (`ACTION_MAP` + pipeline routing), and `lib.rs` (specta command registration). All code evidence was read from the live codebase.

The core challenge is the **three-way coupling** between the migration (which must be versioned, idempotent, and loss-safe), the `ACTION_MAP` (which today is a `Lazy<HashMap>` of static entries and must be extended to handle dynamic `smart_mode_{id}` keys at runtime), and the `TranscriptionCoordinator` (which routes `binding_id` → `ACTION_MAP` and uses `is_transcribe_binding` to intercept transcribe-class bindings — that function must be extended to match `smart_mode_*` prefix).

The settings deserialization story is clean: every new field added to `AppSettings` requires a `#[serde(default)]` attribute (or a default function), ensuring old JSON loads without error. The `ensure_post_process_defaults` function (called on every `get_settings` and `load_or_create_app_settings`) is the correct place to hook versioned migration logic. The `load_or_create_app_settings` function already handles the serde parse path and calls this hook.

**Primary recommendation:** Add `settings_schema_version: u32` with `#[serde(default)]` to `AppSettings`, implement versioned migration inside a new `migrate_settings_if_needed()` function called from both `load_or_create_app_settings` and `get_settings`, seed defaults with an idempotency check, and extend `ACTION_MAP` + `is_transcribe_binding` to handle the `smart_mode_` prefix — passing the mode-id through `TranscribeAction` via `binding_id` at runtime rather than at static init time.

---

## Standard Stack

### Core (already in Cargo.toml — no new dependencies needed for Phase 12)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `serde` | 1 (features = ["derive"]) | Serialize/Deserialize for `AppSettings`, `SmartMode`, `SmartModeKind` | Already used; all `AppSettings` fields derive it |
| `serde_json` | 1 | JSON round-trip in unit tests (migration fixture tests) | Already in `[dev-dependencies]` and `[dependencies]` |
| `specta` | =2.0.0-rc.22 | `Type` derive for `SmartMode`, `SmartModeKind`, `TargetLanguage` — required for tauri-specta export | Already pinned; must stay at this exact version |
| `tauri-specta` | =2.0.0-rc.21 features=["derive","typescript"] | Auto-generates `src/bindings.ts` from `#[tauri::command] #[specta::specta]` | Already in use; new commands auto-register |
| `chrono` | (in deps) | `Utc::now().timestamp_millis()` for new mode ID generation | Already used in `add_post_process_prompt` |
| `tempfile` | 3 | `TempDir` for migration fixture tests | Already in `[dev-dependencies]` at line 124 |

**Installation:** No new dependencies required for Phase 12.

### Specta Version Constraint

**CRITICAL:** specta is pinned to `=2.0.0-rc.22` and tauri-specta to `=2.0.0-rc.21`. Do NOT change these versions. Any new types flowing through commands must derive `specta::Type` with these exact versions.

---

## Architecture Patterns

### Confirmed Project Structure

```
src-tauri/src/
├── settings.rs              # AppSettings, SmartMode, SmartModeKind, TargetLanguage types
│                            # + migration fn, default_smart_modes(), ensure_smart_mode_defaults()
├── shortcut/
│   ├── mod.rs               # Smart Mode CRUD commands (new) + existing binding commands
│   └── handler.rs           # is_transcribe_binding check — must be updated
├── transcription_coordinator.rs  # is_transcribe_binding() — must match smart_mode_ prefix
├── actions.rs               # ACTION_MAP + post_process_transcription() routing
├── lib.rs                   # specta_builder.commands([...]) — new commands register here
└── commands/
    └── mod.rs               # (Smart Mode CRUD may live in shortcut/mod.rs per existing pattern)
```

**Command placement decision:** Existing prompt CRUD commands (`add/update/delete_post_process_prompt`, `set_post_process_selected_prompt`) live in `shortcut/mod.rs` (lines 922-1056), not in `commands/`. Smart Mode CRUD should follow the same pattern — place in `shortcut/mod.rs` for consistency. The `commands/` subdirectory is organized by concern (audio, models, llm, transcription, history); Smart Modes are settings-adjacent, same as prompts.

### Pattern 1: Adding a New Versioned Field to AppSettings

All new fields must carry `#[serde(default)]` or a `#[serde(default = "fn_name")]` attribute. This is the only mechanism that allows old v1.2 JSON (which lacks `smart_modes` and `settings_schema_version`) to deserialize successfully.

```rust
// Source: settings.rs:307-409 (AppSettings derive + existing fields pattern)
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct AppSettings {
    // ... existing fields ...
    #[serde(default)]
    pub settings_schema_version: u32,       // 0 = unversioned (v1.2 or earlier)
    #[serde(default = "default_smart_modes")]
    pub smart_modes: Vec<SmartMode>,
    #[serde(default)]
    pub smart_mode_active_id: Option<String>,
}
```

The `settings_schema_version` field defaults to `0` when absent from JSON (v1.2 files). After migration it is written as `1`. This is the version guard.

### Pattern 2: New Type Deriving for tauri-specta

All types that flow through tauri commands must derive `Serialize + Deserialize + Clone + specta::Type`. Plain unit enums use `#[serde(rename_all = "snake_case")]` matching the codebase convention.

```rust
// Source: settings.rs:109-129 (ModelUnloadTimeout enum — canonical enum derive pattern)
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type, Default)]
#[serde(rename_all = "snake_case")]
pub enum SmartModeKind {
    #[default]
    Rewrite,
    Translation,
}

// For the TargetLanguage struct (nested in SmartMode):
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct TargetLanguage {
    pub code: String,    // ISO 639-1, e.g. "fr"
    pub label: String,   // Display name, e.g. "French"
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct SmartMode {
    pub id: String,
    pub name: String,
    pub kind: SmartModeKind,
    pub prompt: String,
    #[serde(default)]
    pub target_language: Option<TargetLanguage>,
}
```

`SmartMode` and `SmartModeKind` declared in `settings.rs` (same file as `AppSettings`). `TargetLanguage` also in `settings.rs` (co-located with the type that uses it). All three are exported by specta automatically when they appear in command signatures or as fields of a command parameter.

### Pattern 3: Migration Hook Placement

The confirmed call path for migration:

```
App startup
  → init_shortcuts() [shortcut/mod.rs:34]
    → load_or_create_app_settings() [settings.rs:822]
      → serde_json::from_value::<AppSettings>(settings_value)   // old JSON loads fine due to #[serde(default)]
      → ensure_post_process_defaults(&mut settings)             // existing implicit-defaults sync
      ← NEW: migrate_settings_if_needed(&mut settings)          // versioned migration — runs AFTER serde
```

`migrate_settings_if_needed` is also called from `get_settings` (settings.rs:875) since that is called on every hot path. It must be **idempotent** — guarded by `settings_schema_version == 0` check so it only runs once per installation.

```rust
// Source: settings.rs:634-687 (ensure_post_process_defaults pattern) + settings.rs:822-873 (load path)
fn migrate_settings_if_needed(settings: &mut AppSettings) -> bool {
    const CURRENT_VERSION: u32 = 1;
    if settings.settings_schema_version >= CURRENT_VERSION {
        return false; // already migrated
    }
    // Run v1.2 → v1.3 migration:
    //   1. Convert each LLMPrompt in post_process_prompts → Rewrite SmartMode
    //   2. Reconcile "Improve Transcriptions": pristine → replace with Clean Up; edited → keep both
    //   3. Seed the 10 curated defaults (idempotency: check by name or stable ID, not re-seed if present)
    //   4. Map post_process_selected_prompt_id → smart_mode_active_id
    //   5. Transfer transcribe_with_post_process binding combo to active mode's smart_mode_{id} binding
    //   6. Stamp settings_schema_version = 1
    // On any panic/error: leave settings unchanged (don't partially migrate)
    settings.settings_schema_version = CURRENT_VERSION;
    true
}
```

Loss-safe strategy: wrap migration body in `std::panic::catch_unwind` or validate all preconditions before mutating. If migration fails, log an error and return `false` (no version stamp written, will retry next launch — which is safe because the migration is idempotent once it succeeds).

### Pattern 4: CRUD Commands (mirror existing prompt CRUD exactly)

Exact existing CRUD signatures from `shortcut/mod.rs:922-1056`:

```rust
// Source: shortcut/mod.rs:922-996
#[tauri::command] #[specta::specta]
pub fn add_post_process_prompt(app: AppHandle, name: String, prompt: String) -> Result<LLMPrompt, String>
// ID = format!("prompt_{}", chrono::Utc::now().timestamp_millis())
// push to settings.post_process_prompts, write_settings, return new item

#[tauri::command] #[specta::specta]
pub fn update_post_process_prompt(app: AppHandle, id: String, name: String, prompt: String) -> Result<(), String>
// find by id, update fields, write_settings

#[tauri::command] #[specta::specta]
pub fn delete_post_process_prompt(app: AppHandle, id: String) -> Result<(), String>
// guard: len <= 1 → Err("Cannot delete the last prompt")
// retain(|p| p.id != id), reassign selection to first if deleted was selected

#[tauri::command] #[specta::specta]
pub fn set_post_process_selected_prompt(app: AppHandle, id: String) -> Result<(), String>
// verify prompt exists, set post_process_selected_prompt_id, write_settings
```

Smart Mode CRUD mirrors these with additional fields (`kind`, `target_language`) and adapted guards:

```rust
// New commands to write (same file: shortcut/mod.rs)
add_smart_mode(app, name, kind, prompt, target_language) -> Result<SmartMode, String>
update_smart_mode(app, id, name, prompt, target_language) -> Result<(), String>
delete_smart_mode(app, id) -> Result<(), String>   // guard: cannot delete last mode
set_active_smart_mode(app, id) -> Result<(), String>
list_smart_modes(app) -> Result<Vec<SmartMode>, String>  // convenience for frontend
```

`add_smart_mode` generates: `format!("mode_{}", chrono::Utc::now().timestamp_millis())`.

### Pattern 5: Per-Mode Shortcut Registration

The `change_binding` command (shortcut/mod.rs:108-202) is the template for dynamic bind/unbind. Its return type is `BindingResponse { success: bool, binding: Option<ShortcutBinding>, error: Option<String> }`. For smart mode binding updates:

```rust
// New command (shortcut/mod.rs) — wraps change_binding internally
#[tauri::command] #[specta::specta]
pub fn set_smart_mode_binding(app: AppHandle, mode_id: String, binding: String) -> Result<BindingResponse, String>
// 1. Construct binding_id = format!("smart_mode_{}", mode_id)
// 2. Ensure a ShortcutBinding entry exists in settings.bindings for this id
// 3. Delegate to change_binding(app, binding_id, binding)
// Conflict detection: change_binding already returns BindingResponse { success: false, error: Some("...") }
```

Bindings for smart modes are stored in the same `settings.bindings: HashMap<String, ShortcutBinding>` as all other bindings. The `smart_mode_{id}` key is the binding ID.

Init registration at startup:

```rust
// tauri_impl::init_shortcuts (tauri_impl.rs:17-40) — extend the loop:
// After registering default bindings, iterate settings.smart_modes:
for mode in &user_settings.smart_modes {
    let binding_key = format!("smart_mode_{}", mode.id);
    if let Some(binding) = user_settings.bindings.get(&binding_key) {
        if !binding.current_binding.is_empty() {
            if let Err(e) = register_shortcut(app, binding.clone()) {
                error!("Failed to register smart mode shortcut {}: {}", binding_key, e);
            }
        }
    }
}
```

### Pattern 6: ACTION_MAP + is_transcribe_binding Extension

The `ACTION_MAP` is a `Lazy<HashMap<String, Arc<dyn ShortcutAction>>>` (actions.rs:778). It is a **static** initialization — it cannot contain per-mode entries that are only known at runtime. The clean solution is the same pattern used for the `TranscriptionCoordinator`:

`handler.rs:38` already has a special-case check `if is_transcribe_binding(binding_id)` that routes to the `TranscriptionCoordinator` instead of `ACTION_MAP`. Extend `is_transcribe_binding` to include the smart mode prefix:

```rust
// Source: transcription_coordinator.rs:40-42
// CURRENT:
pub fn is_transcribe_binding(id: &str) -> bool {
    id == "transcribe" || id == "transcribe_with_post_process"
}
// UPDATED:
pub fn is_transcribe_binding(id: &str) -> bool {
    id == "transcribe" || id == "transcribe_with_post_process" || id.starts_with("smart_mode_")
}
```

This routes all `smart_mode_{id}` shortcuts through the `TranscriptionCoordinator`, which calls `ACTION_MAP.get(binding_id)`. Since `ACTION_MAP` won't have a static entry for a dynamic `smart_mode_{id}`, we need a **fallback handler** in `ACTION_MAP` — or better, handle the prefix inside the coordinator's `start()`/`stop()` path:

The coordinator's `start()` function (transcription_coordinator.rs:161-175) calls `ACTION_MAP.get(binding_id)`. For `smart_mode_*` keys that won't exist in the static map, this returns `None` and logs a warning. **The correct fix** is to add a single catch-all in the coordinator's start/stop path that handles the `smart_mode_` prefix and dispatches to a new `SmartModeTranscribeAction`:

```rust
// Alternative (cleaner): In transcription_coordinator.rs start() fn:
fn start(app: &AppHandle, stage: &mut Stage, binding_id: &str, hotkey_string: &str) {
    let action: Arc<dyn ShortcutAction> = if binding_id.starts_with("smart_mode_") {
        let mode_id = binding_id.strip_prefix("smart_mode_").unwrap();
        Arc::new(SmartModeAction { mode_id: mode_id.to_string() })
    } else {
        let Some(a) = ACTION_MAP.get(binding_id) else {
            warn!("No action in ACTION_MAP for '{binding_id}'");
            return;
        };
        Arc::clone(a)
    };
    action.start(app, binding_id, hotkey_string);
    // ...
}
```

OR alternatively, insert `smart_mode_*` into `ACTION_MAP` dynamically at app startup after modes are loaded — but `Lazy` doesn't support post-init mutation without an `RwLock`. The coordinator-local branch is simpler and doesn't require locking.

**Recommended approach:** Add the `smart_mode_` prefix branch directly in `transcription_coordinator.rs` `start()` and `stop()` functions. Introduce `SmartModeAction { mode_id: String }` implementing `ShortcutAction`, which reads the mode from settings and calls `process_transcription_output` with a per-mode override.

### Pattern 7: process_transcription_output Refactoring

Current signature (actions.rs:427):

```rust
pub(crate) async fn process_transcription_output(
    app: &AppHandle,
    transcription: &str,
    post_process: bool,
) -> ProcessedTranscription
```

The `post_process: bool` reads the active mode from settings. For per-mode shortcuts, the triggered mode ID is known at call time. Refactor to accept an override:

```rust
pub(crate) async fn process_transcription_output(
    app: &AppHandle,
    transcription: &str,
    post_process: bool,
    mode_id_override: Option<&str>,   // Some("mode_123...") for smart_mode_ triggers
) -> ProcessedTranscription
```

Inside `post_process_transcription()`, when `mode_id_override` is `Some(id)`, look up the `SmartMode` by `id` instead of using `smart_mode_active_id`. Branch on `mode.kind`:
- `SmartModeKind::Rewrite` → existing prompt path through the active generic model
- `SmartModeKind::Translation` → stub path for Phase 13: log "translation engine not yet wired", return `None`

All existing call sites (`transcription_coordinator.rs`, CLI signal handler) pass `None` for `mode_id_override` (preserving existing behavior).

### Anti-Patterns to Avoid

- **Adding smart_mode_* entries to the static `Lazy<ACTION_MAP>`**: This map cannot be mutated after init and doesn't know mode IDs at compile time.
- **Adding `settings_schema_version` without `#[serde(default)]`**: Will cause a hard parse failure on all existing v1.2 settings files.
- **Running migration logic inside `write_settings()`**: That function is called constantly and must remain a pure write. Migration belongs in the load path.
- **Seeding defaults inside `get_settings()` without idempotency guard**: `get_settings()` is hot-path; the version check prevents re-seeding on every call.
- **Calling `ensure_post_process_defaults` after migration stamps version**: The existing function adds providers/api_keys/models; migration adds smart modes. They serve different purposes and can coexist if called in order.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Shortcut conflict detection | Custom "is this combo already registered?" logic | `register_shortcut()` return value + OS-level rejection | The OS-level global shortcut API rejects duplicates; `register_shortcut()` returns `Err(String)` on failure |
| JSON settings versioning | Custom base64/hash comparison | `settings_schema_version: u32` with serde default | Simple integer guards are idempotent, testable, and forward-compatible |
| Dynamic binding store | Separate data structure for smart mode bindings | `settings.bindings: HashMap<String, ShortcutBinding>` | Already serialized, already merged on load, already the source of truth for all bindings |
| New ID generation | UUID crate | `format!("mode_{}", chrono::Utc::now().timestamp_millis())` | Exact same pattern used in `add_post_process_prompt`; no new dependency |
| Specta type export | Manual TypeScript type definitions | `#[specta::specta]` derive | Auto-generated into `src/bindings.ts` on every debug build; never hand-edit that file |

**Key insight:** The codebase's binding system, settings persistence, and CRUD pattern are all sufficiently general that Smart Modes slot in without new abstractions. Resist the urge to introduce a separate storage layer or a new manager for Smart Modes — they belong in `AppSettings` exactly as prompts do today.

---

## Common Pitfalls

### Pitfall 1: `ACTION_MAP` Lazy Static Cannot Be Extended at Runtime

**What goes wrong:** Attempting to insert `smart_mode_{id}` entries into `ACTION_MAP` after app startup fails because `Lazy<HashMap<...>>` provides no post-init `insert` method.

**Why it happens:** The map is initialized once via `Lazy::new(|| {...})`. It's a static allocation.

**How to avoid:** Route `smart_mode_*` bindings through the `is_transcribe_binding()` / `TranscriptionCoordinator` path (which already handles all transcription variants), then branch on the prefix inside `transcription_coordinator.rs` `start()`/`stop()` to create a `SmartModeAction` instance dynamically.

**Warning signs:** If you see `"No action defined in ACTION_MAP for shortcut ID 'smart_mode_*'"` in logs, `is_transcribe_binding` was not extended.

### Pitfall 2: specta Version Mismatch Breaks `bindings.ts` Export

**What goes wrong:** Adding a new type that uses `#[derive(Type)]` with a mismatched specta version (or without the exact `=2.0.0-rc.22` pin) causes a compile error or silent binding omission.

**Why it happens:** specta 2.0.0-rc.22 is pinned with `=` (exact match). Any crate that re-exports specta at a different version creates a type conflict.

**How to avoid:** All new types in `settings.rs` derive `specta::Type` exactly as existing types do (e.g., `LLMPrompt`, `PostProcessProvider`). Run `cargo build` in debug mode after adding types to confirm `src/bindings.ts` is regenerated correctly.

**Warning signs:** `error[E0277]: the trait specta::Type is not implemented` for a new struct.

### Pitfall 3: `get_settings()` Called Before Migration Completes

**What goes wrong:** `get_settings()` (settings.rs:875) is called on every hot path (every shortcut dispatch, every settings read). If migration is only hooked into `load_or_create_app_settings` but not `get_settings`, the migration never runs in most contexts.

**Why it happens:** `load_or_create_app_settings` is called only at `init_shortcuts()` startup; `get_settings()` is called everywhere else. Both already call `ensure_post_process_defaults`.

**How to avoid:** Hook `migrate_settings_if_needed` inside both `load_or_create_app_settings` AND `get_settings`, guarded by the version check (so it only does work on the first call after a v1.2 upgrade).

**Warning signs:** App starts, old settings persist on first settings panel open, migration runs only after restart.

### Pitfall 4: tauri_impl vs handy_keys Init Divergence

**What goes wrong:** `init_shortcuts` has two implementations — `tauri_impl::init_shortcuts` and `handy_keys::init_shortcuts`. Adding smart_mode shortcut registration to only one leaves HandyKeys users without per-mode shortcuts.

**Why it happens:** `shortcut/mod.rs:34-57` dispatches to one of two implementations based on `keyboard_implementation` setting.

**How to avoid:** Add the smart_mode init loop to BOTH `tauri_impl::init_shortcuts` AND `handy_keys::init_shortcuts`.

**Warning signs:** Smart mode shortcuts work on macOS (tauri) but not on Linux (handy_keys).

### Pitfall 5: Migration Partially Executes on Error

**What goes wrong:** Migration starts converting `post_process_prompts` to `smart_modes`, crashes on entry 3/5, writes partial state, stamps version — now the v1.2 prompts are gone and only 3 modes exist.

**Why it happens:** No rollback mechanism in sequential mutation.

**How to avoid:** Build the new `smart_modes` list separately, then atomically replace both `smart_modes` and stamp `settings_schema_version` only after the entire list is built successfully. Do NOT stamp the version until all mutation is complete.

### Pitfall 6: `is_transcribe_binding` Not Updated

**What goes wrong:** `smart_mode_{id}` shortcuts register successfully, user triggers them, but `handler.rs:38` does NOT route them to `TranscriptionCoordinator` because `is_transcribe_binding` returns `false`. Instead, it falls to `ACTION_MAP.get(binding_id)` which returns `None` (warning + no-op).

**Why it happens:** `is_transcribe_binding` (transcription_coordinator.rs:40) only checks for literal `"transcribe"` and `"transcribe_with_post_process"`.

**How to avoid:** Update `is_transcribe_binding` to also return `true` for `id.starts_with("smart_mode_")`. This is the single point of control for routing shortcut events to the transcription pipeline.

---

## Code Examples

### SmartMode Type Declaration

```rust
// Placement: settings.rs, immediately after LLMPrompt struct definition (line ~94)
// Source: verified pattern from settings.rs ModelUnloadTimeout (lines 117-129) + LLMPrompt (lines 89-94)

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type, Default)]
#[serde(rename_all = "snake_case")]
pub enum SmartModeKind {
    #[default]
    Rewrite,
    Translation,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct TargetLanguage {
    pub code: String,
    pub label: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct SmartMode {
    pub id: String,
    pub name: String,
    pub kind: SmartModeKind,
    pub prompt: String,
    #[serde(default)]
    pub target_language: Option<TargetLanguage>,
}
```

### AppSettings New Fields

```rust
// Placement: settings.rs AppSettings struct (lines 307-409), after existing post_process_* fields
// Source: verified pattern from AppSettings (lines 370-373 for post_process_prompts pattern)

#[serde(default)]
pub settings_schema_version: u32,

#[serde(default = "default_smart_modes")]
pub smart_modes: Vec<SmartMode>,

#[serde(default)]
pub smart_mode_active_id: Option<String>,
```

### Migration Function Structure

```rust
// Placement: settings.rs, after ensure_post_process_defaults()
// Source: pattern from ensure_post_process_defaults (lines 634-687)

const DEFAULT_IMPROVE_TRANSCRIPTIONS_PROMPT: &str = "Clean this transcript:\n1. Fix spelling...";
// (full text matches the literal at settings.rs:622)

fn migrate_settings_if_needed(settings: &mut AppSettings) -> bool {
    const CURRENT_VERSION: u32 = 1;
    if settings.settings_schema_version >= CURRENT_VERSION {
        return false;
    }
    // Build migrated modes separately for atomic replacement
    let mut new_modes: Vec<SmartMode> = Vec::new();
    let mut new_active_id: Option<String> = None;

    // 1. Convert existing LLMPrompts → Rewrite SmartModes
    // 2. Reconcile "Improve Transcriptions"
    // 3. Seed 10 curated defaults (skip if already present by stable ID)
    // 4. Map post_process_selected_prompt_id → new_active_id
    // 5. Transfer transcribe_with_post_process binding to active mode

    // Atomic commit (only after full success):
    settings.smart_modes = new_modes;
    settings.smart_mode_active_id = new_active_id;
    settings.settings_schema_version = CURRENT_VERSION;
    true
}
```

### Extend is_transcribe_binding

```rust
// Placement: transcription_coordinator.rs:40-42
// Source: current code verified at those exact lines

// BEFORE:
pub fn is_transcribe_binding(id: &str) -> bool {
    id == "transcribe" || id == "transcribe_with_post_process"
}

// AFTER:
pub fn is_transcribe_binding(id: &str) -> bool {
    id == "transcribe" || id == "transcribe_with_post_process" || id.starts_with("smart_mode_")
}
```

### Specta Registration in lib.rs

```rust
// Placement: lib.rs specta_builder.commands([...]) block (lines 415-528)
// Source: verified from lib.rs:447-450 (existing prompt CRUD registration pattern)

// Add after existing shortcut::set_post_process_selected_prompt:
shortcut::add_smart_mode,
shortcut::update_smart_mode,
shortcut::delete_smart_mode,
shortcut::set_active_smart_mode,
shortcut::list_smart_modes,
shortcut::set_smart_mode_binding,
```

### Migration Unit Test Pattern

```rust
// Placement: settings.rs #[cfg(test)] mod tests block (lines 931-1003)
// Source: verified from managers/llm.rs test pattern with tempfile (lines 1018-1025)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_v12_to_v13_preserves_custom_prompt() {
        // Build a v1.2 settings shape (schema_version absent = 0)
        let v12_json = serde_json::json!({
            "bindings": {},
            "push_to_talk": true,
            "audio_feedback": false,
            // ... minimal required fields ...
            "post_process_prompts": [{
                "id": "prompt_custom_123",
                "name": "My Custom Prompt",
                "prompt": "Do something custom with: ${output}"
            }],
            "post_process_selected_prompt_id": "prompt_custom_123"
            // NOTE: no settings_schema_version key → deserializes as 0
        });

        let mut settings: AppSettings = serde_json::from_value(v12_json).unwrap();
        assert_eq!(settings.settings_schema_version, 0);

        let migrated = migrate_settings_if_needed(&mut settings);

        assert!(migrated);
        assert_eq!(settings.settings_schema_version, 1);
        // Custom prompt preserved as Rewrite SmartMode
        assert!(settings.smart_modes.iter().any(|m| m.name == "My Custom Prompt"));
        // Clean Up default seeded
        assert!(settings.smart_modes.iter().any(|m| m.name == "Clean Up"));
        // Active selection mapped
        assert!(settings.smart_mode_active_id.is_some());
        // Idempotency: second call does nothing
        let migrated2 = migrate_settings_if_needed(&mut settings);
        assert!(!migrated2);
    }

    #[test]
    fn migration_v12_pristine_improve_transcriptions_replaced_by_clean_up() {
        // v1.2 default unmodified prompt → gets replaced, no near-duplicate
        let mut settings = get_default_settings(); // has "Improve Transcriptions"
        settings.settings_schema_version = 0;
        migrate_settings_if_needed(&mut settings);
        // "Improve Transcriptions" must NOT appear; "Clean Up" must appear
        assert!(!settings.smart_modes.iter().any(|m| m.name == "Improve Transcriptions"));
        assert!(settings.smart_modes.iter().any(|m| m.name == "Clean Up"));
    }

    #[test]
    fn default_smart_modes_count_is_ten() {
        let settings = get_default_settings();
        // Fresh install: settings_schema_version is 0 OR defaults seeded directly
        // After first ensure_smart_mode_defaults, smart_modes has 10 entries
        assert_eq!(settings.smart_modes.len(), 10);
    }
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `post_process_prompts: Vec<LLMPrompt>` with `{id, name, prompt}` | `smart_modes: Vec<SmartMode>` with `{id, name, kind, prompt, target_language}` | Phase 12 | Single source; kinds can diverge (Rewrite vs Translation) |
| `transcribe_with_post_process` generic binding | `smart_mode_{id}` per-mode bindings + combo transfer | Phase 12 | Each mode bindable independently; no 10-global-shortcut storm |
| Prompt selection via `post_process_selected_prompt_id` | `smart_mode_active_id` | Phase 12 | Cleaner naming; same semantics |
| Static `ACTION_MAP` + `post_process: bool` flag | Dynamic `smart_mode_` prefix branch in coordinator + `mode_id_override` param | Phase 12 | Allows per-mode routing without static map entries |

**Still present after Phase 12 (to be deprecated later):**
- `post_process_prompts` and `post_process_selected_prompt_id` fields on `AppSettings`: these remain in the struct with `#[serde(default)]` to avoid breaking deserialization during migration, but are no longer the operational data source after migration. They can be removed in a future cleanup once all clients use `smart_modes`.

---

## Open Questions

1. **`transcribe_with_post_process` binding removal timing**
   - What we know: CONTEXT.md says retire the binding ID but transfer its combo to the migrated active mode.
   - What's unclear: Should the `transcribe_with_post_process` key be removed from `AppSettings.bindings` during migration? Or left as a tombstone? If removed, `tauri_impl::init_shortcuts` will no longer find it in the default bindings loop, which is fine — but the frontend's binding settings UI may still reference it.
   - Recommendation: Remove from `settings.bindings` during migration; remove from `get_default_settings().bindings` so it's never seeded. The frontend binding UI is rebuilt in Phase 13 and will only show `smart_mode_{id}` bindings.

2. **`handy_keys::init_shortcuts` smart_mode loop**
   - What we know: It needs the same per-mode loop as `tauri_impl::init_shortcuts`.
   - What's unclear: The handy_keys implementation builds a `hotkey_to_binding` map at init from the `settings.bindings` HashMap — it may need no changes beyond the extended `is_transcribe_binding` check in `handler.rs`, since it already iterates all bindings.
   - Recommendation: Verify `handy_keys::init_shortcuts` iterates all non-cancel bindings (not just `default_settings.bindings`) — if it does, smart_mode entries in `settings.bindings` are automatically picked up without a separate loop.

3. **`post_process_prompts` field deprecation scope**
   - What we know: CONTEXT.md says "drop/deprecate after migration" as single-source-of-truth.
   - What's unclear: Phase 12 or Phase 13 removes the field and old CRUD commands?
   - Recommendation: Keep `post_process_prompts` field on `AppSettings` for Phase 12 (needed for migration reads). Do NOT remove the field — removing it before Phase 13's UI is rebuilt would break the frontend's `usePostProcessProviderState.ts` which still references it. Mark it `#[deprecated]` in a comment but leave it in the struct.

---

## Validation Architecture

> `workflow.nyquist_validation` is `true` in `.planning/config.json` — this section is REQUIRED.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` (built-in Rust test harness, no separate framework) |
| Config file | none — `cargo test` discovers `#[cfg(test)] mod tests` inline |
| Quick run command | `cargo test -p dictus-desktop-lib migration --no-default-features 2>&1` |
| Full suite command | `cd src-tauri && cargo test --all-targets 2>&1` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| MODE-01 | v1.2 JSON with custom prompt migrates to SmartMode with no data loss | unit | `cargo test -p dictus-desktop-lib migration_v12` | ❌ Wave 0 |
| MODE-01 | "Improve Transcriptions" pristine → replaced by Clean Up (no near-duplicate) | unit | `cargo test -p dictus-desktop-lib migration_v12_pristine` | ❌ Wave 0 |
| MODE-01 | "Improve Transcriptions" edited → kept as custom + Clean Up added | unit | `cargo test -p dictus-desktop-lib migration_v12_edited` | ❌ Wave 0 |
| MODE-01 | Migration is idempotent (second run is no-op) | unit | `cargo test -p dictus-desktop-lib migration_idempotent` | ❌ Wave 0 |
| MODE-01 | `settings_schema_version` is written after migration | unit | `cargo test -p dictus-desktop-lib migration_stamps_version` | ❌ Wave 0 |
| MODE-01 | Active selection preserved: `post_process_selected_prompt_id` maps to `smart_mode_active_id` | unit | `cargo test -p dictus-desktop-lib migration_active_selection` | ❌ Wave 0 |
| MODE-02 | Fresh install: `smart_modes` contains exactly 10 entries | unit | `cargo test -p dictus-desktop-lib default_smart_modes_count` | ❌ Wave 0 |
| MODE-02 | "Clean Up" is first mode, Rewrites precede Translations | unit | `cargo test -p dictus-desktop-lib default_smart_modes_order` | ❌ Wave 0 |
| MODE-02 | Translation modes have `kind = Translation` and `target_language` set | unit | `cargo test -p dictus-desktop-lib default_translation_modes_have_target_language` | ❌ Wave 0 |
| MODE-03 | `add_smart_mode` → mode persists in settings with generated ID | unit (no Tauri) | `cargo test -p dictus-desktop-lib crud_add` | ❌ Wave 0 |
| MODE-03 | `delete_smart_mode` on last mode returns Err | unit | `cargo test -p dictus-desktop-lib crud_delete_last_guard` | ❌ Wave 0 |
| MODE-03 | specta bindings generated — `SmartMode`, `SmartModeKind` appear in `src/bindings.ts` | smoke (manual) | `grep -q "SmartMode" src/bindings.ts` | ❌ Wave 0 |
| MODE-04 | `is_transcribe_binding("smart_mode_abc")` returns `true` | unit | `cargo test -p dictus-desktop-lib is_transcribe_binding_smart_mode_prefix` | ❌ Wave 0 |
| MODE-04 | `process_transcription_output` with mode_id_override routes to correct mode prompt | unit | `cargo test -p dictus-desktop-lib mode_routing_rewrite` | ❌ Wave 0 |
| MODE-04 | Translation mode routing returns None (stub path, no crash) | unit | `cargo test -p dictus-desktop-lib mode_routing_translation_stub` | ❌ Wave 0 |

**Manual-only checks (no automated command possible without Tauri runtime):**
- Shortcut actually fires when user presses bound combo → requires app running
- Conflict detection returns error when two modes share a combo → requires `register_shortcut` (OS-level, needs Tauri)

### Sampling Rate
- **Per task commit:** `cd src-tauri && cargo test migration -- --nocapture`
- **Per wave merge:** `cd src-tauri && cargo test --all-targets 2>&1 | tail -20`
- **Phase gate:** Full suite green + `grep -q "SmartMode" src/bindings.ts` before `/gsd:verify-work`

### Wave 0 Gaps

All test functions listed above are new (❌). They must be written in `src-tauri/src/settings.rs` (migration tests) and `src-tauri/src/transcription_coordinator.rs` (binding routing tests).

- [ ] `src-tauri/src/settings.rs` — migration tests: `migration_v12_to_v13_preserves_custom_prompt`, `migration_v12_pristine_improve_transcriptions_replaced_by_clean_up`, `migration_idempotent`, `migration_stamps_version`, `migration_active_selection`, `default_smart_modes_count_is_ten`, `default_smart_modes_order`, `default_translation_modes_have_target_language`
- [ ] `src-tauri/src/transcription_coordinator.rs` — routing tests: `is_transcribe_binding_smart_mode_prefix`
- [ ] `src-tauri/src/actions.rs` — routing tests: `mode_routing_rewrite_uses_correct_prompt`, `mode_routing_translation_returns_none`
- [ ] v1.2 settings fixture JSON inline in test body (no file needed — `serde_json::json!({...})` is sufficient, see llm.rs test pattern)

Framework install: `cargo test` already available — no install needed. `tempfile = "3"` already in `[dev-dependencies]` (Cargo.toml:124).

---

## Sources

### Primary (HIGH confidence — code read directly from this repository)

- `src-tauri/src/settings.rs` lines 1-1003 — `AppSettings`, `LLMPrompt`, `PostProcessProvider`, `ModelUnloadTimeout` enum derive pattern, `ensure_post_process_defaults`, `load_or_create_app_settings`, `get_settings`, `write_settings`, `get_default_settings`, migration hook placement, existing test suite
- `src-tauri/src/shortcut/mod.rs` lines 75-202, 690-810, 900-1056 — `register_shortcut`, `unregister_shortcut`, `change_binding`, `BindingResponse`, existing prompt CRUD (`add/update/delete_post_process_prompt`, `set_post_process_selected_prompt`), default bindings
- `src-tauri/src/shortcut/tauri_impl.rs` lines 1-70 — `init_shortcuts` implementation, skip conditions, binding registration loop
- `src-tauri/src/shortcut/handler.rs` — `handle_shortcut_event`, `is_transcribe_binding` routing point
- `src-tauri/src/transcription_coordinator.rs` lines 40-184 — `is_transcribe_binding`, `TranscriptionCoordinator::new`, `start()`, `stop()`, `ACTION_MAP` lookup pattern
- `src-tauri/src/actions.rs` lines 44-200, 420-465, 770-800 — `TranscribeAction { post_process: bool }`, `post_process_transcription`, `process_transcription_output`, `ProcessedTranscription`, `ACTION_MAP`
- `src-tauri/src/lib.rs` lines 415-540 — `specta_builder.commands([...])` full registration list, tauri-specta export path
- `src-tauri/src/commands/mod.rs` — module layout confirming commands organization
- `src-tauri/src/managers/llm.rs` lines 25-50, 1018-1080 — `LlmModelInfo` type (verified `#[derive(Debug, Clone, Serialize, Deserialize, Type)]` pattern), test pattern with `tempfile::TempDir`
- `src-tauri/Cargo.toml` lines 29-30, 53-54, 81-83, 124 — `serde`, `serde_json`, `specta =2.0.0-rc.22`, `tauri-specta =2.0.0-rc.21`, `tempfile = "3"` (dev-dep)
- `.planning/config.json` — `workflow.nyquist_validation: true`

### Secondary (HIGH confidence — project planning documents)

- `.planning/phases/12-smart-modes-data-layer/12-CONTEXT.md` — locked decisions, migration strategy, per-mode shortcut scheme, deferred items
- `.planning/REQUIREMENTS.md` — MODE-01..04 acceptance criteria
- `.planning/STATE.md` lines 78-79 — TranslateGemma deferred, translation needs native chat template
- `.planning/ROADMAP.md` — Phase 12 success criteria, SC-1..4

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all dependencies verified in Cargo.toml
- Migration mechanics: HIGH — `load_or_create_app_settings` and `ensure_post_process_defaults` read in full; serde default story confirmed
- CRUD pattern: HIGH — existing prompt CRUD read line-by-line (shortcut/mod.rs:922-1056)
- Per-mode shortcut routing: HIGH — `ACTION_MAP`, `is_transcribe_binding`, `handler.rs` routing, `transcription_coordinator.rs` `start()`/`stop()` all read
- Specta registration: HIGH — `lib.rs` specta_builder block read in full
- Test infrastructure: HIGH — `#[cfg(test)]` tests exist in `settings.rs` and `managers/llm.rs`; `tempfile` already in dev-deps

**Research date:** 2026-06-03
**Valid until:** 2026-07-03 (stable library versions; specta pin unlikely to change)
