# Phase 6: Brand & Icon Polish - Context

**Gathered:** 2026-04-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Close the remaining Handy brand leaks on user-visible surfaces (recording filenames, portable mode marker, DebugPaths display) and ship correct platform icons (Linux transparent-corners 256×256, Windows multi-layer ICO, committed square source-of-truth PNG). Extend `verify-sync.sh` — and relocate it to `.github/scripts/` — so future upstream syncs cannot silently reintroduce any of these regressions.

**Requirements covered:** BRAND-01, BRAND-02, BRAND-03, BRAND-04, ICON-01, ICON-02, ICON-03, ICON-04 — plus SYNC-06 pulled forward from Phase 9 (see Roadmap Ripple below).

**Out of scope:** Binary rename `handy → dictus` (TECH-03, deferred); renaming `handy_keys` module (TECH-01, deferred); SYNC-07 updater assertions in verify-sync.sh (Phase 9 still owns); any new capability beyond identity integrity.

</domain>

<decisions>
## Implementation Decisions

### Icon source-of-truth (ICON-04)

- **Location:** dictus-brand repo. Add `appicon-desktop-1024.png` (or equivalent square variant) to `dictus-brand/source/` (or a new `dictus-brand/desktop/` subfolder — brand repo maintainer chooses). 1024×1024 RGBA, transparent background, square corners (NO rounded corners baked in — Linux must render clean).
- **dictus-desktop consumption:** Reference the file at `../dictus-brand/...` in plan tasks, same pattern as Phase 2.
- **Do NOT reuse iOS `AppIcon-1024.png`** — it has rounded corners baked in (Phase 2 used it because iOS/macOS OS re-masks, but Linux ships it as-is → black corner artifact = ICON-01 root cause).
- **Visual identity:** Same iOS waveform glyph, rendered on a transparent square canvas. macOS OS applies its own rounded-square mask at runtime; Linux/Windows get clean transparent edges.

### Icon generation pipeline

- **Tool:** `bun run tauri icon <path-to-square-1024>.png` — accept default output set (32/128/128@2x + icon.icns + icon.ico + Windows Square*Logo variants).
- **`tauri.conf.json > bundle.icon` list:** Extend beyond the current 5 entries to include `icons/256x256.png` and `icons/512x512.png` explicitly (ICON-03) so Linux deb/AppImage ships the correct sizes.
- **Regeneration is idempotent:** future upstream icon drift can be fixed by re-running `tauri icon` against the same brand source.

### Windows ICO verification (ICON-02)

- **Verification tool:** ImageMagick `identify -format '%w %h\n' icon.ico`.
- **Assertion:** `identify` output must contain layers at 16, 24, 32, 48, 64, and 256 pixels (six lines minimum, each dimension appearing at least once).
- **Integration:** assertion goes into `verify-sync.sh` (see BRAND-04 below), not a separate script — one entry point for identity integrity.
- **Dev-env dep:** imagemagick becomes a hard requirement for running `verify-sync.sh`. Script fails early with a "brew install imagemagick" message, same pattern as the current `jq` check at line 18.

### Recording filename (BRAND-01)

- **Real code path:** `src-tauri/src/actions.rs:538` — change `format!("handy-{}.wav", …)` → `format!("dictus-{}.wav", …)`.
- **Test fixtures:** `src-tauri/src/managers/history.rs:689` and `src-tauri/src/tray.rs:273` are test-only fixtures using `"handy-{}.wav"` / `"handy-1.wav"`. Update both to `"dictus-{}.wav"` / `"dictus-1.wav"` for consistency and to avoid false positives in the BRAND-04 grep assertion.
- **No migration:** existing `handy-*.wav` files on disk (if any) are left alone. Only new recordings get the Dictus prefix. History panel keeps playing back old files by filename as stored.

### Portable mode marker (BRAND-02) — simplified

- **Approach:** straight string replace. No dual-read, no legacy recognition, no in-place migration.
- **Rationale:** Pierre is effectively the only user at this point — the app has not been publicly communicated yet (as of 2026-04-15). No portable installs in the wild carry a legacy `"Handy Portable Mode"` marker that needs to survive the rename.
- **Explicit override:** this contradicts REQUIREMENTS.md BRAND-02 "still recognizing legacy `Handy Portable Mode` on existing installs". That language was written defensively; decision here is to drop the defensive path because there is no installed base to protect. Planner MUST NOT reintroduce dual-read logic.
- **Code changes:**
  - `src-tauri/src/portable.rs:30` — `std::fs::write(&marker_path, "Handy Portable Mode")` → `std::fs::write(&marker_path, "Dictus Portable Mode")`
  - `src-tauri/src/portable.rs:98` — `s.trim().starts_with("Handy Portable Mode")` → `s.trim().starts_with("Dictus Portable Mode")`
  - `src-tauri/src/portable.rs:5` — doc comment "Portable mode support for Handy" → "Portable mode support for Dictus"
  - 6 test cases at lines 107-165 — update the `write!(f, "Handy Portable Mode")` / `write!(f, "  Handy Portable Mode\n")` strings to `"Dictus Portable Mode"`. Rename test dir names to use `dictus_test_*` for hygiene.

### DebugPaths panel (BRAND-03)

- **Display source:** real filesystem path loaded via Tauri `appDataDir()` API at runtime — NOT tokenized `%APPDATA%/…` or `~/Library/…` strings.
- **Portable-mode aware:** backend command returns the portable `Data/` directory when `portable::is_portable()` is true, else the standard `appDataDir()`. Expose a single new `get_app_data_dir_display` (or similar) Tauri command that encapsulates the portable check.
- **Sub-paths:** frontend receives ONE base dir from the backend; component joins `/models` and `/settings_store.json` suffixes in JSX for display. No separate commands for each sub-path.
- **i18n:** existing translation keys `settings.debug.paths.appData`, `.models`, `.settings` remain. The `eslint-disable-next-line i18next/no-literal-string` directives at DebugPaths.tsx:28, 35, 44 are removed — the displayed path comes from a backend value, not a hardcoded string.
- **Load pattern:** React effect (or existing hook pattern if one covers backend data) on mount, with a placeholder/loading state until the command resolves. Non-blocking — this is a debug panel, not a critical path.

### verify-sync.sh — relocation + brand assertions (BRAND-04 + SYNC-06 pulled forward)

- **New location:** `.github/scripts/verify-sync.sh` — pulled out of `.planning/phases/05-upstream-sync/scripts/`. This absorbs SYNC-06 (originally in Phase 9) into Phase 6.
- **Rationale for pulling SYNC-06 forward:** Phase 9's CI gate (SYNC-08) depends on the script living at its final path. Extending it in-place under `.planning/` then moving it in Phase 9 would churn git history twice. Single move here + in-place edits is cleaner. Reduces Phase 9 scope to SYNC-07/08/09/10/11.
- **New assertions added in Phase 6:**
  - `BRAND-01a` — `grep -rn 'handy-' src-tauri/src/ | grep -v handy_keys` returns zero matches (the `handy-keys` crate ref at `handy_keys.rs` is the legitimate external-crate exception).
  - `BRAND-02a` — `! grep -q '"Handy Portable Mode"' src-tauri/src/portable.rs` (legacy string absent).
  - `BRAND-03a` — `! grep -q '%APPDATA%/handy' src/components/settings/debug/DebugPaths.tsx` (hardcoded string absent).
  - `ICON-02a` — `identify -format '%w\n' src-tauri/icons/icon.ico | sort -u` contains `16`, `24`, `32`, `48`, `64`, `256` (each layer present).
- **SYNC-07 assertions are NOT added in Phase 6** — `plugins.updater.pubkey` present + `endpoints[0]` contains `getdictus/dictus-desktop` remain Phase 9's scope.
- **Execution model:** unconditional — every run checks every assertion. No env-var / label gating. Pierre runs pre-PR, Phase 9 CI gate runs on labeled `upstream-sync` PRs, locally anytime.
- **UPSTREAM.md updates:** all `./.planning/phases/05-upstream-sync/scripts/verify-sync.sh` path references in UPSTREAM.md are updated to `./.github/scripts/verify-sync.sh`.

### Roadmap ripple

- **Phase 9 scope reduced:** SYNC-06 is now complete at end of Phase 6. Phase 9 still owns SYNC-07 (updater assertions added to verify-sync.sh), SYNC-08 (CI workflow wiring), SYNC-09/10/11 (draft-PR flow, PR template, UPSTREAM.md trim).
- **No Phase 9 dependency change:** Phase 9 still depends on Phase 6 (unchanged — just tighter now).
- **Update trigger:** `/gsd:roadmap-update` or equivalent should be run after Phase 6 context to reflect SYNC-06 move. Planner should flag this in PLAN.md so it's not forgotten.

### Claude's Discretion

- Exact wording of the explanatory comment (if any) when SYNC-06 move lands — plan can fold the move into a commit with `chore(sync): relocate verify-sync.sh to .github/scripts/ (SYNC-06)`.
- Whether to introduce a helper function in `verify-sync.sh` for the icon check vs inlining the `identify | sort -u | grep` pipeline. Match existing script style.
- Exact React pattern in DebugPaths.tsx (dedicated `useAppDataDir()` hook vs inline `useEffect`) — planner picks what's idiomatic with existing settings hooks.
- Whether the new `get_app_data_dir_display` command lives in `commands/mod.rs` (general settings) or a new `commands/debug.rs` file. Planner picks based on existing command organization.
- Exact path inside `dictus-brand/` for the new 1024 square PNG — brand repo maintainer (Pierre) decides when adding the file. dictus-desktop plans just reference the final agreed path.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements and scope
- `.planning/REQUIREMENTS.md` §Brand Cleanup — BRAND-01 through BRAND-04 acceptance criteria
- `.planning/REQUIREMENTS.md` §Cross-Platform Icons — ICON-01 through ICON-04 acceptance criteria
- `.planning/REQUIREMENTS.md` §Sync Infrastructure — SYNC-06 (pulled forward from Phase 9 scope)
- `.planning/ROADMAP.md` §"Phase 6: Brand & Icon Polish" — goal + 4 success criteria
- `.planning/PROJECT.md` §"Current Milestone: v1.2 Polish & Automation" — milestone framing

### Files to modify (grounded paths)
- `src-tauri/src/actions.rs:538` — recording filename format
- `src-tauri/src/managers/history.rs:689` — test fixture filename
- `src-tauri/src/tray.rs:273` — test fixture filename
- `src-tauri/src/portable.rs:5,30,98` and tests (lines 107-165) — magic string + doc comment + test strings
- `src/components/settings/debug/DebugPaths.tsx` — full rewrite of the 3 hardcoded path spans + disable-next-line removals
- `src-tauri/tauri.conf.json` §bundle.icon — extend list with 256×256 and 512×512
- `src-tauri/icons/` — regenerated via `tauri icon` (do not hand-edit)
- `.planning/phases/05-upstream-sync/scripts/verify-sync.sh` → `.github/scripts/verify-sync.sh` (git mv + extend)
- `UPSTREAM.md` — update all script-path references to new location
- `src-tauri/src/commands/` — new Tauri command for app-data-dir display (portable-aware)

### Prior phase context (decisions carried forward)
- `.planning/phases/01-bundle-identity/01-CONTEXT.md` — binary name stays `handy` per deferred TECH-03
- `.planning/phases/02-visual-rebrand/02-CONTEXT.md` — icon generation pattern: `tauri icon` + dictus-brand source PNG; acknowledgments block in i18n is legitimately allowed to mention Handy
- `.planning/phases/04-updater-infrastructure/04-CONTEXT.md` — asset-prefix `dictus` already in release.yml (do not revisit)
- `.planning/phases/05-upstream-sync/05-CONTEXT.md` — `verify-sync.sh` pattern and current assertions; SYNC-05d awk-skip of acknowledgments block must be preserved on move

### External tooling
- `tauri icon` CLI — https://v2.tauri.app/reference/cli/#icon — generates platform-specific icon sets from a 1024×1024 source
- ImageMagick `identify` — https://imagemagick.org/script/identify.php — ICO layer introspection

### Brand assets
- `../dictus-brand/source/appicon-light.svg` — reference waveform glyph for the new square 1024 PNG
- `../dictus-brand/source/appicon-dark.svg` — dark variant (not used for desktop icon, here for reference)
- New file to be added: `../dictus-brand/source/appicon-desktop-1024.png` (or dictus-brand/desktop/...) — square 1024×1024 RGBA transparent, source-of-truth for `tauri icon`

### No external ADRs
This project has no `docs/decisions/` ADR directory. Decisions live in `.planning/` and inline in CONTEXT.md.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `tauri icon` CLI already works end-to-end from Phase 2 — same pipeline applies here, just with a square source PNG.
- `portable.rs::is_valid_portable_marker()` function pattern (single-string match + `starts_with` + trim) — keep the same signature, just swap the string.
- `verify-sync.sh` existing 11 assertions (SYNC-05a through SYNC-05k) set the pattern: `check "<label>" "<shell expr>"` with local FAIL counter and early `jq` availability check — mirror the pattern for new BRAND/ICON assertions.
- Tauri `appDataDir()` API via `@tauri-apps/api/path` — already a dependency of the frontend.
- `portable::app_data_dir(app: &tauri::AppHandle)` function in `portable.rs:60-66` — already portable-aware, returns the correct path for both modes. Reuse this inside the new `get_app_data_dir_display` command.

### Established Patterns
- **Tauri commands:** `#[tauri::command]` in `commands/*.rs`, registered via `collect_commands!` in `lib.rs::run()`. Auto-generates TypeScript bindings in `src/bindings.ts` on debug build.
- **i18n:** strings in `src/i18n/locales/{en,es,fr,vi}/translation.json`, accessed via `useTranslation()` + `t('key.path')`. ESLint enforces this via `i18next/no-literal-string`. Removing eslint-disable comments in DebugPaths.tsx means the spans now render backend-provided data (not literal strings), so the rule is satisfied.
- **Commit conventions:** `feat:`, `fix:`, `chore:`, `docs:` — brand cleanup commits fit `chore(brand):` or `fix(brand):`; icon regeneration fits `chore(icons):`; verify-sync move fits `chore(sync): relocate verify-sync.sh (SYNC-06)`.

### Integration Points
- `tauri.conf.json > bundle.icon` array ↔ Tauri bundler ↔ per-platform packages (icns for macOS .app, ico for Windows .exe, PNGs for Linux deb/AppImage).
- `src-tauri/icons/icon.ico` ↔ Windows bundler ↔ taskbar/launcher ICO layers.
- `portable.rs` magic string ↔ exe-adjacent `portable` file ↔ portable-install auto-detection at startup.
- `actions.rs` filename format ↔ history.rs DB row ↔ frontend history panel display.
- New Tauri command `get_app_data_dir_display` ↔ `portable::app_data_dir()` ↔ DebugPaths.tsx UI.
- `verify-sync.sh` @ `.github/scripts/` ↔ Phase 9 `verify-sync.yml` CI workflow (Phase 9 will call the script from its new location).

### Deferred touchpoints (DO NOT modify this phase)
- `src-tauri/Cargo.toml` binary `name = "handy"`, `default-run = "handy"`, `lib.name = "handy_app_lib"` — TECH-01 / TECH-03 deferred
- `handy_keys.rs` module name — TECH-01 deferred (external `handy-keys` crate must NOT be renamed)
- Existing `handy-*.wav` files on disk in user data dirs — no migration, left intact
- SYNC-07 assertions (updater pubkey/endpoint) in verify-sync.sh — Phase 9 owns

</code_context>

<specifics>
## Specific Ideas

- Pierre is effectively the only user as of 2026-04-15 (app not yet publicly communicated). This unlocks the simplified BRAND-02 (no dual-read) and justifies absorbing SYNC-06 without a migration ceremony.
- "Faire au plus simple" — simplest path wins when multiple defensive options exist and there's no installed base to protect.
- Icon visual consistency with iOS (waveform glyph) is non-negotiable — desktop platforms all ship the same glyph, differing only in OS-applied corner masking (macOS rounds, Linux/Windows render square transparent).
- Brand assets belong in `dictus-brand`; any new asset generated for dictus-desktop should be contributed back to that repo as the source-of-truth.

</specifics>

<deferred>
## Deferred Ideas

- **Renaming existing `handy-*.wav` files on disk** — considered and rejected. Non-critical polish, touches user data, no behavioral gain. Left for a future migration phase if ever justified.
- **Dropping the `handy-keys` external crate dependency** — out of scope per PROJECT.md (TECH-01 deferred, `handy-keys` is an external crate and cannot be renamed without a fork).
- **Migrating the on-disk app data dir path** — DATA-01, tracked as deferred in PROJECT.md. Still deferred; Phase 6 only changes what DebugPaths **displays**, not the actual path, which remains `com.dictus.desktop` from Phase 1.
- **Dedicated `verify-icons.sh` script separate from `verify-sync.sh`** — considered and rejected. Single script keeps one entry point for identity integrity, and the ICO check is cheap.
- **Automating `tauri icon` regeneration in CI** — nice-to-have but out of scope. Manual regeneration on brand asset changes is acceptable at current cadence.

</deferred>

---

*Phase: 06-brand-icon-polish*
*Context gathered: 2026-04-15*
