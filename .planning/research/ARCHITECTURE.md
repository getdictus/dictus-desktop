# Architecture Research — v1.2 Polish & Automation Integration Points

**Domain:** Tauri desktop app + GitHub Actions CI/CD + Claude Code agents
**Milestone:** v1.2 Polish & Automation
**Researched:** 2026-04-15
**Confidence:** HIGH (all conclusions drawn directly from source files)

---

## 1. CI Pipeline Integration

### Current state

Three workflows exist:

| Workflow | Trigger | What it does |
|---|---|---|
| `upstream-sync.yml` | Cron Mon 08:00 UTC + manual | Fetches upstream, compares SHA to `.github/upstream-sha.txt`, opens GitHub Issue labeled `upstream-sync` if drift detected |
| `build.yml` | `workflow_call` only | Reusable 7-platform matrix build (macOS×2, Linux×3, Windows×2) |
| `release.yml` | `workflow_dispatch` | Creates draft release, calls `build.yml` with `sign-binaries: true` |

A flat file `.github/upstream-sha.txt` is the only persisted sync state — stores the 40-char SHA of the last merged upstream commit.

`verify-sync.sh` currently lives at `.planning/phases/05-upstream-sync/scripts/verify-sync.sh`. It is referenced in `UPSTREAM.md §6` with a hardcoded relative path. It is run manually before pushing a sync PR — not enforced by CI.

### Target state after v1.2 refactor

```
.github/
├── workflows/
│   ├── upstream-sync.yml      REPLACE body with community action (~30 lines)
│   ├── verify-sync.yml        NEW: CI gate on PRs labeled 'upstream-sync'
│   ├── build.yml              UNCHANGED
│   └── release.yml            UNCHANGED
├── pull_request_template/
│   └── upstream-sync.md       NEW: PR checklist (cap-at-SHA? risk-rating? verify-sync green?)
└── scripts/
    └── verify-sync.sh         MOVED from .planning/phases/05-upstream-sync/scripts/
```

`UPSTREAM.md §6` must be updated to reference `.github/scripts/verify-sync.sh` instead of the old path.

### CI data flow across all three workflows

```
WEEKLY CRON
    |
    v
upstream-sync.yml (community action, e.g. aormsby/Fork-Sync-With-Upstream-action)
    -> git fetch cjpais/Handy
    -> opens draft PR: base=main, head=upstream/sync-YYYY-MM-DD
    -> applies label 'upstream-sync' to the PR

    |  (PR creation triggers label-based workflows)
    v

[PARALLEL]
claude-agents.yml (on: pull_request, label='upstream-sync')
    -> job: adapter  — resolves conflicts, commits to branch
    -> job: auditor  — reads diff, posts manual test checklist as PR comment

verify-sync.yml (on: pull_request, label='upstream-sync')
    -> checks out PR branch
    -> runs .github/scripts/verify-sync.sh
    -> required status check (merge blocked if FAIL)

    |  (all checks green, developer reviews adapter output + auditor checklist)
    v

Developer merges PR (Create merge commit — NEVER squash)
    -> .github/upstream-sha.txt updated as part of merge commit (if kept)
```

`build.yml` and `release.yml` are entirely unaffected by the sync refactor.

### verify-sync.yml trigger design

```yaml
on:
  pull_request:
    types: [opened, synchronize, labeled]

jobs:
  verify:
    if: contains(github.event.pull_request.labels.*.name, 'upstream-sync')
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: bash .github/scripts/verify-sync.sh
```

The `if:` condition gates the job — unrelated PRs never trigger the check. The job must be configured as a required status check in branch protection for the gate to be enforced.

### What disappears

- Custom SHA-comparison shell script in `upstream-sync.yml` — replaced by community action
- Custom `github-script` issue-creation block — replaced by the draft PR the community action opens
- `.github/upstream-sha.txt` — if the community action uses `git merge-base` instead of a flat file, this file can be removed; otherwise keep it but stop updating it from the detection workflow
- `.planning/phases/05-upstream-sync/scripts/` path — `verify-sync.sh` moves to `.github/scripts/verify-sync.sh`

---

## 2. Claude Code Agent (Double-Agent) Architecture

### Problem

Two independent Claude Code agent runs are needed on the same upstream-sync PR:
- **Adapter agent**: modifies code (resolves merge conflicts, preserves Dictus identity, applies verify-sync.sh fixes). Has write access to the branch.
- **Auditor agent**: reads the resolved PR diff and generates a manual test checklist as a PR comment. Must be read-only and must not pollute the adapter's context.

### Recommendation: two jobs in one workflow, NOT two workflows

File: `.github/workflows/claude-agents.yml` (new)

```yaml
on:
  pull_request:
    types: [labeled, synchronize]

jobs:
  adapter:
    if: contains(github.event.pull_request.labels.*.name, 'upstream-sync')
    runs-on: ubuntu-latest
    permissions:
      contents: write
      pull-requests: write
    steps:
      - uses: anthropics/claude-code-action@...
        with:
          # adapter-specific config: write access, code modification mode

  auditor:
    if: contains(github.event.pull_request.labels.*.name, 'upstream-sync')
    needs: [adapter]   # wait for adapter to commit its changes first
    runs-on: ubuntu-latest
    permissions:
      contents: read
      pull-requests: write   # write only to post a comment
    steps:
      - uses: anthropics/claude-code-action@...
        with:
          # auditor-specific config: read-only, generates test checklist comment
```

Two jobs in one workflow provides:
- Single workflow run visible in GitHub Actions UI
- Guaranteed context isolation — each job gets a fresh runner, fresh process, fresh Claude session
- `needs: [adapter]` ensures the auditor reads the final resolved diff, not the raw conflict state

### Why not two separate workflows

Two separate workflows triggered independently on the same PR event create a race condition: both could start simultaneously, and if the adapter pushes commits, the auditor might read an intermediate state. The `needs:` dependency in a single workflow eliminates this.

### Context isolation guarantee

GitHub provisions each job on a fresh runner with a fresh `actions/checkout`. No shared disk, no shared process memory. The Claude Code action's context window starts fresh per job. No additional isolation mechanism is needed.

### Access model

| Job | Push to branch | Post PR comment | Read files |
|---|---|---|---|
| adapter | YES | YES | YES |
| auditor | NO | YES | YES |

Use `permissions:` blocks (shown above) rather than org-level secrets to enforce the boundary. The auditor's `contents: read` prevents it from committing even if it tries.

---

## 3. Icon Build Pipeline

### Current icon inventory

`tauri.conf.json` bundle.icon array (lines 32-37) lists five files:
```
icons/32x32.png
icons/128x128.png
icons/128x128@2x.png
icons/icon.icns
icons/icon.ico
```

Additional files present on disk but NOT in the bundle.icon array:
- `icons/64x64.png` — present, unused by bundler
- `icons/icon.png` — 512×512 source PNG
- `icons/logo.png` — Dictus logo variant
- `icons/Square*.png` (9 files) — Windows Store tiles, not used for standard bundles
- `icons/StoreLogo.png` — Windows Store, not used

### Platform-specific gaps

**Linux**: The AppImage/.deb uses the PNG icons from the array. Linux launchers (GNOME, KDE) expect square icons with no padding. If `icon.png`, `32x32.png`, `128x128.png` were generated from a macOS-optimized source (rounded corners, padding inset), they will appear visually inconsistent in Linux app grids. A dedicated `linux-square-256x256.png` with full-bleed square edges should be added.

**Windows**: `icon.ico` must embed multiple resolutions (at minimum 16×16, 32×32, 48×48, 256×256). A single-resolution ICO will be scaled by Windows Explorer and the taskbar, producing blurry icons. The current `icon.ico` may have been generated with limited resolutions — needs verification.

### Recommendation: commit pre-generated variants, no build-time transform

Tauri's bundler reads the `bundle.icon` array before building and expects files to exist at the listed paths. There is no native hook for image transformation before the bundler reads the array. Build-time ImageMagick in CI would require patching `tauri.conf.json` mid-build (the `build.yml` already does this for ONNX runtime dylib paths, so the pattern exists — but it is complex).

The simpler and more reliable approach:
1. Use `bun run tauri icon <source.png>` (1024×1024 minimum source) — this regenerates the complete icon set including multi-resolution ICO and ICNS
2. Add `icons/linux-square-256x256.png` manually (no padding variant)
3. Commit all generated files to the repo
4. Add `icons/linux-square-256x256.png` to the `bundle.icon` array in `tauri.conf.json`

### Source-of-truth chain

```
dictus-brand repo (SVG or 1024x1024 PNG)
    |
    v (run locally: bun run tauri icon <source.png>)
src-tauri/icons/  (committed output)
    |
    v (tauri.conf.json bundle.icon array)
Tauri bundler -> platform packages
```

### tauri.conf.json change for Linux square icon

Add one line to the `bundle.icon` array:
```json
"icon": [
  "icons/32x32.png",
  "icons/128x128.png",
  "icons/128x128@2x.png",
  "icons/icon.icns",
  "icons/icon.ico",
  "icons/linux-square-256x256.png"
]
```

Adding a PNG does not affect macOS (uses ICNS) or Windows (uses ICO). Tauri selects the appropriate format per platform.

---

## 4. Rust Shutdown Order

### Current shutdown paths

`app.exit(0)` is called from two locations:

- `src-tauri/src/lib.rs:254` — inside `on_menu_event` in `initialize_core_logic()`, on tray "quit" menu item
- `src-tauri/src/lib.rs:622` — inside `on_window_event` CloseRequested handler, when no tray is visible

Both call `tauri::AppHandle::exit(0)`, which triggers Tauri's shutdown sequence: signals the Tokio runtime, then drops managed state and plugins.

### Manager initialization order (lib.rs:162-173)

```
AudioRecordingManager::new()    line 163  -- CPAL CoreAudio stream on macOS
ModelManager::new()             line 166  -- owns model file handles
TranscriptionManager::new()     line 169  -- holds Arc<ModelManager>, Metal GPU contexts
HistoryManager::new()           line 172  -- SQLite connection
```

All four registered via `app_handle.manage()` (lines 179-183). Tauri drops managed state in undefined order during shutdown — `Arc<T>` drop is not guaranteed to run on the main thread.

### Plugin registration order (lib.rs:481-543)

Rust drops in reverse registration order. The last-registered plugin drops first:

```
Last registered (drops first):
  tauri_plugin_autostart
  tauri_plugin_global_shortcut  <-- HIGHEST CRASH SUSPECT
  tauri_plugin_store            <-- writes to disk on drop
  tauri_plugin_opener
  tauri_plugin_macos_permissions
  tauri_plugin_clipboard_manager
  tauri_plugin_os
  tauri_plugin_updater
  tauri_plugin_process
  tauri_plugin_fs
  tauri_plugin_single_instance  <-- Unix socket at /tmp/com_dictus_desktop_si.sock
  tauri_nspanel (macOS only)
  tauri_plugin_log              <-- background flush thread
  tauri_plugin_dialog
First registered (drops last):
```

### Most likely crash suspects (macOS)

1. **`tauri_plugin_global_shortcut`** — Unhooking global keyboard listeners on macOS requires main-thread access (CGEventTap runs on the main thread). If the plugin's `Drop` impl executes on a Tokio worker thread, macOS can SIGABRT. This is the highest-probability suspect because `tauri_plugin_global_shortcut` is registered late (drops early) and global shortcut teardown is a known macOS threading pitfall.

2. **`AudioRecordingManager` / CPAL stream** — CPAL audio streams on macOS wrap CoreAudio streams which have thread-affinity requirements. If an active CPAL stream is dropped off the main thread during shutdown, CoreAudio can SIGABRT.

3. **`TranscriptionManager` / Metal GPU context** — whisper-rs / ort Metal contexts must be released on or before the Metal device teardown. Timing depends on when the GPU context is created vs when the Tokio runtime shuts down.

### Recommended fix strategy

**Phase 1: Explicit cleanup before exit (low risk, try first)**

In both exit call sites, add cleanup before `app.exit(0)`:

```rust
// In tray "quit" handler and in on_window_event CloseRequested:
fn clean_exit(app: &AppHandle) {
    // Stop any active audio recording to release CoreAudio stream
    if let Some(rm) = app.try_state::<Arc<AudioRecordingManager>>() {
        // call a stop_all / shutdown method
    }
    // Give tauri_plugin_global_shortcut a chance to unhook on main thread
    // by unregistering shortcuts explicitly before exit
    app.exit(0);
}
```

**Phase 2: Hard exit fallback (if Phase 1 is insufficient)**

Replace `app.exit(0)` with:
```rust
log::logger().flush(); // ensure log file is written
std::process::exit(0); // bypass all destructors
```

Trade-off: bypasses `tauri_plugin_store` flush (settings changes in the last few ms may be lost) and skips Unix socket cleanup (already handled by the stale-socket cleanup at `lib.rs:341-358` on next launch).

**Diagnosis prerequisite**: Read the crash report in Console.app → User Reports → Dictus to identify the crashing thread's stack trace before implementing fixes.

---

## 5. Settings UI — Provider List Reorganization

### Current provider order (settings.rs:524-603)

`default_post_process_providers()` builds the list in this order:

1. OpenAI — `openai` (default selected: `default_post_process_provider_id()` returns `"openai"`)
2. Z.AI — `zai`
3. OpenRouter — `openrouter`
4. Anthropic — `anthropic`
5. Groq — `groq`
6. Cerebras — `cerebras`
7. Apple Intelligence — `apple_intelligence` (macOS ARM64 only, appended at line 582)
8. Custom — `custom` (always last, appended at line 593)

There is no Ollama-specific entry. "Custom" with default `base_url: "http://localhost:11434/v1"` is the intended Ollama path.

### Rendering path (full chain)

```
settings.rs:default_post_process_providers()
    | serialized via get_app_settings Tauri command
    v
useSettings hook -> settingsStore (Zustand)
    v
usePostProcessProviderState hook
    | builds `providerOptions: DropdownOption[]` from settings.post_process_providers
    v
PostProcessingSettings.tsx:39 (ProviderSelect)
    v
Dropdown component -- renders options in received array order, no client-side reorder
```

The Dropdown renders in the order it receives. Provider order is **entirely controlled by the `Vec` order in `default_post_process_providers()` in settings.rs**. No frontend reordering logic exists.

### Files to edit

| File | Change | Risk |
|---|---|---|
| `src-tauri/src/settings.rs:524-603` | Reorder the `providers` vec — local providers first (Ollama/Custom, Apple Intelligence), cloud providers after | LOW |
| `src-tauri/src/settings.rs:520-521` | Change `default_post_process_provider_id()` to return a local provider ID if desired | MEDIUM — affects new installs only |
| `src/components/settings/PostProcessingSettingsApi/ProviderSelect.tsx` | Add visual section dividers if Dropdown supports optgroup-style rendering | LOW–MEDIUM |

### Migration concern: existing saved provider preference

The persisted value is a plain string ID (`post_process_provider_id: "openai"` in the store JSON). Reordering the list in `default_post_process_providers()` does not change any ID strings. **No data migration is needed.** Existing users keep their current selection.

The only risk is if `default_post_process_provider_id()` is changed — this only affects fresh installs and is not a migration concern for existing users.

### Adding Ollama as a first-class entry

If a dedicated `ollama` entry is added (separate from `custom`):
- Add a new `PostProcessProvider` with `id: "ollama"`, `base_url: "http://localhost:11434/v1"`, `allow_base_url_edit: false`
- This does not affect existing `custom` users — they remain on `custom`
- Do NOT auto-migrate `custom` users to `ollama` — their base URL may have been changed

### "External (advanced)" visual grouping

The `Dropdown` component needs to support option groups (divider rows or headers) before the UI can visually separate local vs cloud. Assess the current Dropdown API in `src/components/ui/` before committing to this. If Dropdown does not support groups, a simpler approach is adding a divider `DropdownOption` with `disabled: true` and a label like `"— External providers —"`.

---

## 6. Brand Cleanup Migration Strategy

### Recording filename change

**Primary location**: `src-tauri/src/actions.rs:538`
```rust
let file_name = format!("handy-{}.wav", chrono::Utc::now().timestamp());
```

**Other occurrences (test fixtures only)**:
- `src-tauri/src/managers/history.rs:689` — `format!("handy-{}.wav", timestamp)` in the `insert_entry` test helper
- `src-tauri/src/tray.rs:273` — `file_name: "handy-1.wav".to_string()` in the `build_entry` test helper

**History DB relationship**: The DB stores `file_name` as TEXT in `transcription_history` (schema at `history.rs:22`). The `get_audio_file_path` command constructs the path as `recordings_dir.join(entry.file_name)`. If `actions.rs:538` is changed to write `dictus-{ts}.wav` going forward, existing rows with `handy-{ts}.wav` will still point to files on disk named `handy-{ts}.wav` — those files are untouched and playback continues to work. The DB filename and on-disk filename are always kept in sync at write time; no secondary index maps one to the other.

**Existing history entries**: NOT broken by a forward-only rename. Old rows reference old files (`handy-{ts}.wav` on disk = `handy-{ts}.wav` in DB). New rows will reference `dictus-{ts}.wav` files. Both coexist without conflict.

**Options**:

| Option | What it does | Risk |
|---|---|---|
| A: Forward-only | Change `actions.rs:538` to write `dictus-{ts}.wav`; fix test fixtures | Zero — old history untouched |
| B: Full migration | Rename existing WAV files on disk + UPDATE all DB rows in `HistoryManager::new()` | Medium — file system + DB transaction |
| C: Defer runtime | Fix only test fixtures; extend `verify-sync.sh`; rename in v1.3 | Zero for now |

**Recommended for v1.2**: Option A for the runtime (clean, no migration complexity), Option A for test fixtures (zero risk). Add `verify-sync.sh` assertion: `! grep -n '"handy-' src-tauri/src/actions.rs`.

### Portable mode magic string

**Location**: `src-tauri/src/portable.rs`

Two sites:
- `line 30`: `std::fs::write(&marker_path, "Handy Portable Mode")` — written during v0.8.0 legacy upgrade
- `line 98`: `s.trim().starts_with("Handy Portable Mode")` — detection predicate in `is_valid_portable_marker()`

**Backward compatibility requirement**: Existing portable installs have a `portable` marker file containing `"Handy Portable Mode"`. Changing line 98 alone would break those installs — the detection would return `false`, and the app would start in non-portable mode, writing data to `%APPDATA%\com.dictus.desktop` instead of the `Data/` dir.

**Required implementation**:
```rust
fn is_valid_portable_marker(path: &std::path::Path) -> bool {
    std::fs::read_to_string(path)
        .map(|s| {
            let trimmed = s.trim();
            trimmed.starts_with("Dictus Portable Mode")
                || trimmed.starts_with("Handy Portable Mode") // legacy compat
        })
        .unwrap_or(false)
}
```

On detection of the legacy string (lines 23-33), silently rewrite the marker to `"Dictus Portable Mode"` using the same migration-upgrade pattern already present at line 29. Update the unit tests in `portable.rs:102-165` to test both old and new magic strings.

### DebugPaths.tsx Windows path display

**Location**: `src/components/settings/debug/DebugPaths.tsx:29-46`

Hardcoded strings (with ESLint suppression comments):
- Line 29: `%APPDATA%/handy`
- Line 37: `%APPDATA%/handy/models`
- Line 45: `%APPDATA%/handy/settings_store.json`

**Actual on-disk path**: Tauri derives the app data directory from the `identifier` in `tauri.conf.json` (`com.dictus.desktop`). On Windows, `app_data_dir()` resolves to `%APPDATA%\com.dictus.desktop\`. The displayed path is incorrect for both Handy (wrong identifier) and Dictus.

**Correct fix**: Use the `get_app_dir_path` Tauri command (already exists in `commands/`) to retrieve the actual runtime path. Display the real path instead of a hardcoded string. This eliminates the ESLint suppressions and makes the panel accurate regardless of platform or portable mode.

No migration or data change — display-only fix.

---

## 7. Build Order / Phase Sequencing

### Dependency map

```
INDEPENDENT (no inter-dependencies, can run in parallel):
  Phase A: Brand & Icon Polish
    - actions.rs:538 forward-only filename rename
    - history.rs:689, tray.rs:273 test fixtures
    - portable.rs:30,98 magic string + legacy fallback
    - DebugPaths.tsx:29-46 dynamic path display
    - Icon regeneration (tauri icon + linux-square-256x256.png)
    - verify-sync.sh extended assertions

  Phase B: Privacy / Local-First UX
    - settings.rs:524-603 provider reorder
    - ProviderSelect.tsx visual grouping (if Dropdown supports it)
    - Network surface documentation

  Phase C: macOS Shutdown Fix
    - Diagnose via Console.app crash report
    - Explicit cleanup before app.exit(0) in lib.rs + tray.rs
    - std::process::exit(0) fallback if needed

SEQUENTIAL DEPENDENCY CHAIN:
  Phase D: Sync Infra Refactor
    - Move verify-sync.sh -> .github/scripts/verify-sync.sh
    - Update UPSTREAM.md §6 reference
    - Add verify-sync.yml CI gate
    - Replace upstream-sync.yml body with community action
    - Add upstream-sync.md PR template
    MUST COMPLETE BEFORE ->

  Phase E: Claude Code Agent Layer
    - Add claude-agents.yml (adapter + auditor jobs)
    - Wire to upstream-sync-labeled PR trigger
    - Test against synthetic upstream-sync PR
```

### Recommended sequencing

Phases A, B, C are fully independent of each other and of D/E. They can be executed in any order or simultaneously across branches.

Phase D must complete before Phase E because:
- `verify-sync.sh` must exist at `.github/scripts/verify-sync.sh` before `verify-sync.yml` references it
- The community action must be producing `upstream-sync`-labeled PRs before `claude-agents.yml` has meaningful input
- Phase E is additive — if no labeled PRs exist, the workflow simply never triggers

The natural v1.2 execution order is: A + B + C in parallel, then D, then E.

---

## Component Boundary Summary

| Feature | New Component | Modified Component | File:Line |
|---|---|---|---|
| CI gate | `verify-sync.yml` | — | `.github/workflows/verify-sync.yml` (new) |
| Sync workflow | — | `upstream-sync.yml` body replaced | `.github/workflows/upstream-sync.yml` |
| Agent layer | `claude-agents.yml` | — | `.github/workflows/claude-agents.yml` (new) |
| PR template | `upstream-sync.md` | — | `.github/pull_request_template/upstream-sync.md` (new) |
| verify-sync.sh | — | Path change only | `.planning/phases/.../verify-sync.sh` -> `.github/scripts/verify-sync.sh` |
| UPSTREAM.md §6 reference | — | `UPSTREAM.md` | `UPSTREAM.md` |
| Recording filename | — | `actions.rs:538` | `src-tauri/src/actions.rs` |
| Test fixtures | — | `history.rs:689`, `tray.rs:273` | test helper functions |
| Portable string | — | `portable.rs:30,98` + tests | `src-tauri/src/portable.rs` |
| Debug paths display | — | `DebugPaths.tsx:29-46` | `src/components/settings/debug/DebugPaths.tsx` |
| Provider order | — | `settings.rs:524-603` | `src-tauri/src/settings.rs` |
| Provider UI grouping | — | `ProviderSelect.tsx` | `src/components/settings/PostProcessingSettingsApi/ProviderSelect.tsx` |
| Icon Linux square | `linux-square-256x256.png` | `tauri.conf.json` bundle.icon array | `src-tauri/icons/`, `src-tauri/tauri.conf.json:32-37` |
| Icon Windows ICO | — | `icon.ico` (regenerate with multi-res) | `src-tauri/icons/icon.ico` |
| Shutdown fix | — | `lib.rs:254`, `lib.rs:622` | `src-tauri/src/lib.rs` |
| verify-sync.sh assertions | — | `verify-sync.sh` (add checks) | `.github/scripts/verify-sync.sh` |

---

## Anti-Patterns to Avoid

### Squashing the upstream-sync PR

GitHub squash merge removes upstream commit history from `main`. UPSTREAM.md §7 requires "Create a merge commit". Configure a branch protection rule to disallow squash on `upstream-sync`-labeled PRs, or add a note to the PR template.

### Giving the auditor agent write access to the branch

If both adapter and auditor have `contents: write` and run in parallel, there is a race condition on the branch. Use `permissions: { contents: read }` for the auditor job, and `needs: [adapter]` to sequence it after the adapter commits.

### Changing portable mode detection string without legacy fallback

Replacing `starts_with("Handy Portable Mode")` with `starts_with("Dictus Portable Mode")` alone silently breaks existing portable installs — the app starts in non-portable mode and writes to the system app data directory instead of `Data/`. Always keep the legacy fallback check until all known installs have been upgraded (i.e., indefinitely for a production app).

### Hand-editing `Cargo.lock` during conflict resolution

Already documented in UPSTREAM.md §4.7. After resolving `Cargo.toml`, always run `cargo generate-lockfile`. This is not specific to v1.2 but remains a risk whenever Rust dependencies change.

### Relying on `app.exit(0)` drop order for manager cleanup

Tauri's managed state drop order is not defined. `TranscriptionManager` holds `Arc<ModelManager>` — if Tauri drops `ModelManager` before `TranscriptionManager`, the Arc refcount prevents the model from being unloaded until `TranscriptionManager` drops. This is fine for memory safety but not for GPU resource release ordering. Explicit cleanup before `app.exit(0)` is safer than relying on drop ordering.

---

## Sources

- Direct source analysis: `src-tauri/src/lib.rs`, `src-tauri/src/actions.rs`, `src-tauri/src/portable.rs`, `src-tauri/src/managers/history.rs`, `src-tauri/src/settings.rs`, `src-tauri/src/tray.rs`
- Direct source analysis: `.github/workflows/upstream-sync.yml`, `.github/workflows/build.yml`, `.github/workflows/release.yml`
- Direct source analysis: `.planning/phases/05-upstream-sync/scripts/verify-sync.sh`
- Direct source analysis: `src-tauri/tauri.conf.json`, `src/components/settings/`
- `.planning/todos/pending/` — four todo files documenting known gaps

---

*Architecture research for: Dictus Desktop v1.2 Polish & Automation*
*Researched: 2026-04-15*
