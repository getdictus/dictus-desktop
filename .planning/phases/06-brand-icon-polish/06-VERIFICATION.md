---
phase: 06-brand-icon-polish
verified: 2026-04-16T12:00:00Z
status: passed
score: 9/9 must-haves verified
re_verification: false
human_verification:
  - test: "Linux deb/AppImage icon rendering"
    expected: "No black corners on app launcher icon; square navy tile renders cleanly on Linux desktop"
    why_human: "Cannot build Linux packages from macOS. Automated backstop is opaque source PNG (alpha min=1.0, zero transparent pixels), which makes the black-corners artifact physically impossible. Pierre approved this approach and deferred visual confirmation."
  - test: "Windows icon.ico visual fidelity at all scaling levels"
    expected: "Icon renders correctly at 100/125/150/200% DPI scaling on Windows"
    why_human: "No Windows machine available. Automated backstop is ICON-02a verify-sync assertion confirming all 6 required layers (16/24/32/48/64/256) are present in icon.ico. Pierre's stated stance: 'si jamais les icones vont pas, on corrigera ca plus tard.'"
  - test: "DebugPaths panel runtime rendering"
    expected: "Settings > Debug Paths shows the real runtime app data path (e.g., /Users/<name>/Library/Application Support/com.dictus.desktop) — not '%APPDATA%/handy'"
    why_human: "Dynamic backend command result cannot be verified without running the app. Code wiring is fully verified; runtime rendering requires app launch."
---

# Phase 06: Brand & Icon Polish Verification Report

**Phase Goal:** Close the last three user-visible Handy string leaks, fix the Linux icon black-corners artifact, regenerate the platform icon set from a Dictus-owned source, relocate verify-sync.sh to its permanent .github/scripts/ home, and extend it with BRAND/ICON guards.
**Verified:** 2026-04-16
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | No `handy-` filename prefix in src-tauri (except handy_keys) | VERIFIED | `grep -rn 'handy-' src-tauri/src/ | grep -v handy_keys` returns no output; BRAND-01a assertion in verify-sync.sh |
| 2 | Portable mode marker is "Dictus Portable Mode" | VERIFIED | `portable.rs` line 30, 98, 113, 162 all use "Dictus Portable Mode"; zero "Handy Portable Mode" occurrences |
| 3 | DebugPaths.tsx fetches real path via getAppDirPath, no hardcoded `%APPDATA%/handy` | VERIFIED | `grep -q '%APPDATA%/handy' DebugPaths.tsx` returns no match; `commands.getAppDirPath` found in file |
| 4 | verify-sync.sh has 15 assertions including BRAND-01a/02a/03a/ICON-02a | VERIFIED | `grep -c '^check ' .github/scripts/verify-sync.sh` = 15; all four new labels confirmed present |
| 5 | Linux icon source has zero transparent pixels (opaque navy tile) | VERIFIED | `identify` reports alpha min=1.0, max=1.0 on source PNG — fully opaque despite TrueColorAlpha channel type |
| 6 | icon.ico has 6 required layers (16/24/32/48/64/256) | VERIFIED | `identify -format '%w\n' icon.ico | sort -u | grep -cE '^(16|24|32|48|64|256)$'` = 6 |
| 7 | tauri.conf.json bundle.icon has 7 entries including 256x256.png and 512x512.png | VERIFIED | `bundle.icon` array has exactly 7 entries: 32x32, 128x128, 128x128@2x, 256x256, 512x512, icon.icns, icon.ico |
| 8 | 1024x1024 source PNG exists at `../dictus-brand/source/appicon-desktop-1024.png` | VERIFIED | File exists, 36973 bytes, 1024x1024 dimensions confirmed via `identify` |
| 9 | verify-sync.sh at `.github/scripts/` (not `.planning/phases/05-upstream-sync/scripts/`) | VERIFIED | `test -x .github/scripts/verify-sync.sh` passes; old path absent; git history preserved through `git mv` |

**Score:** 9/9 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `.github/scripts/verify-sync.sh` | Relocated validator with 15 assertions, executable | VERIFIED | Exists, executable, 15 `check` lines, syntax valid (`bash -n` exits 0), git history from Phase 05 preserved |
| `UPSTREAM.md` | Updated path references to `.github/scripts/verify-sync.sh` | VERIFIED | 3 occurrences of new path; zero occurrences of old path `.planning/phases/05-upstream-sync/scripts/verify-sync.sh` |
| `src-tauri/src/actions.rs` | `format!("dictus-{}.wav", ...)` for recording filenames | VERIFIED | Contains `dictus-` format string; no `handy-` format call |
| `src-tauri/src/portable.rs` | "Dictus Portable Mode" throughout; doc comment updated | VERIFIED | All 4 string occurrences use "Dictus Portable Mode"; 5 `dictus_test_` dir names; doc comment says "Dictus"; zero "Handy" occurrences |
| `src-tauri/src/managers/history.rs` | Test fixture using `dictus-` filename | VERIFIED | `format!("dictus-{}.wav", timestamp)` confirmed present |
| `src-tauri/src/tray.rs` | Test fixture with `dictus-1.wav` | VERIFIED | `"dictus-1.wav".to_string()` confirmed present |
| `src/components/settings/debug/DebugPaths.tsx` | Fetches from `commands.getAppDirPath()`, no hardcoded handy strings, no eslint-disable directives | VERIFIED | `commands.getAppDirPath` present; `%APPDATA%/handy` absent; no `eslint-disable-next-line i18next/no-literal-string` directives |
| `src-tauri/icons/256x256.png` | Explicit 256px PNG (ImageMagick fallback) | VERIFIED | Exists, 7807 bytes, April 16 timestamp |
| `src-tauri/icons/512x512.png` | Explicit 512px PNG (ImageMagick fallback) | VERIFIED | Exists, 17216 bytes, April 16 timestamp |
| `src-tauri/icons/icon.ico` | 6-layer Windows ICO | VERIFIED | `identify` confirms 6 layers at required sizes |
| `src-tauri/icons/icon.icns` | Regenerated macOS bundle | VERIFIED | Exists, 40773 bytes, April 16 timestamp |
| `../dictus-brand/source/appicon-desktop-1024.png` | 1024x1024 opaque navy tile (external repo) | VERIFIED | Exists, 1024x1024, fully opaque (alpha min=max=1.0) |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `.github/scripts/verify-sync.sh` | `src-tauri/src/`, `src/components/settings/debug/DebugPaths.tsx`, `src-tauri/icons/icon.ico` | grep + identify assertions | WIRED | BRAND-01a/02a/03a grep assertions present at lines 57-65; ICON-02a identify assertion at line 66 |
| `UPSTREAM.md` | `.github/scripts/verify-sync.sh` | documentation reference | WIRED | Line 216 has `bash .github/scripts/verify-sync.sh`; line 284 has bullet referencing new path; line 239 has historical context note |
| `src/components/settings/debug/DebugPaths.tsx` | `src-tauri/src/commands/mod.rs::get_app_dir_path` | `commands.getAppDirPath()` binding in `src/bindings.ts` | WIRED | Import `{ commands } from "@/bindings"` present; `commands.getAppDirPath()` called in `useEffect`; `useState`/`useEffect` hooks present |
| `src-tauri/src/actions.rs` | Recording filename | `format!("dictus-{}.wav", ...)` | WIRED | Format string confirmed in production code path |
| `src-tauri/src/portable.rs` | Portable marker write/read | Same literal string in `init()` write and `is_valid_portable_marker()` read | WIRED | Both `write(&marker_path, "Dictus Portable Mode")` and `starts_with("Dictus Portable Mode")` present |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| BRAND-01 | 06-02 | New recordings named `dictus-{timestamp}.wav` | SATISFIED | `format!("dictus-{}.wav"` in actions.rs:538, history.rs test fixture, tray.rs test fixture; no `handy-` prefix in src-tauri |
| BRAND-02 | 06-02 | Portable mode marker uses "Dictus Portable Mode" | SATISFIED | All occurrences in portable.rs use "Dictus Portable Mode"; deliberate deviation: no legacy `Handy Portable Mode` dual-read (CONTEXT.md override, Pierre approved) |
| BRAND-03 | 06-03 | DebugPaths shows real path via `commands.getAppDirPath()` | SATISFIED | DebugPaths.tsx rewritten with useEffect fetching backend command; no hardcoded handy strings |
| BRAND-04 | 06-01 | verify-sync.sh has assertions for BRAND-01/02/03 surfaces | SATISFIED | BRAND-01a, BRAND-02a, BRAND-03a all present as `check` lines in verify-sync.sh |
| ICON-01 | 06-04 | Linux PNG icon with no black-corners artifact | SATISFIED (automated backstop) | Source PNG is opaque navy tile (alpha min=max=1.0); transparent-corner variant superseded by Pierre-approved opaque approach |
| ICON-02 | 06-01 + 06-04 | icon.ico with 6 layers (16/24/32/48/64/256) + ICON-02a assertion | SATISFIED | identify confirms 6 layers; ICON-02a check present in verify-sync.sh at line 66 |
| ICON-03 | 06-04 | tauri.conf.json bundle.icon has 7 entries with 256x256 and 512x512 | SATISFIED | bundle.icon has exactly 7 entries including explicit 256x256.png and 512x512.png |
| ICON-04 | 06-04 | 1024x1024 source PNG in dictus-brand repo | SATISFIED | `../dictus-brand/source/appicon-desktop-1024.png` exists, 1024x1024, April 16 — note: REQUIREMENTS.md says "under src-tauri/icons/" but 06-04 plan and SUMMARY correctly place it in dictus-brand external repo; both plan and SUMMARY agree on this location |
| SYNC-06 | 06-01 | verify-sync.sh relocated to `.github/scripts/` | SATISFIED | File at `.github/scripts/verify-sync.sh`, executable, git history preserved; old path absent |

**Note on BRAND-02 legacy recognition:** REQUIREMENTS.md states "still recognizing legacy `Handy Portable Mode` on existing installs." The implementation deliberately omits this — per CONTEXT.md explicit override: "straight string replace. No dual-read, no legacy recognition." Pierre is the sole user with no installed base. This deviation is documented in 06-02-PLAN.md and is acceptable.

**Note on REQUIREMENTS.md ICON-04 location:** REQUIREMENTS.md says the source PNG may be "under `src-tauri/icons/`" but the actual implementation places it in `../dictus-brand/source/` (external repo). Both the plan and summary are consistent on this location. The requirement is satisfied by the external repo placement.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| None found | — | — | — | — |

No anti-patterns detected in modified files. No TODO/FIXME/placeholder comments, no empty implementations, no hardcoded test data leaked into production paths.

### Human Verification Required

#### 1. Linux Icon Rendering (ICON-01 visual confirmation)

**Test:** Build a Linux deb or AppImage package and inspect the app launcher icon on a Linux desktop environment (GNOME, KDE, or similar).
**Expected:** Square navy Dictus icon with no black corners visible in app launcher, taskbar, or window title bar.
**Why human:** Cannot build Linux packages from macOS. Automated backstop is fully satisfactory per Pierre: source PNG has alpha min=max=1.0 (fully opaque, no transparent pixels to render incorrectly as black). Visual confirmation is deferred to future Linux CI or manual testing.

#### 2. Windows Icon Visual Fidelity (ICON-02 visual confirmation)

**Test:** Install the Windows MSI/NSIS build and inspect the app icon at 100%, 125%, 150%, and 200% DPI scaling settings.
**Expected:** Dictus logo renders clearly at all scaling levels from the 6-layer ICO file.
**Why human:** No Windows machine available. Automated backstop: ICON-02a confirms all 6 required layers are data-correct. Visual fidelity at Windows display scaling levels is unverified.

#### 3. DebugPaths Runtime Rendering (BRAND-03 runtime confirmation)

**Test:** Launch the app, navigate to Settings, find the Debug section, and verify the Debug Paths panel shows a real filesystem path.
**Expected:** macOS: `/Users/<name>/Library/Application Support/com.dictus.desktop`; Windows: `C:\Users\<name>\AppData\Roaming\com.dictus.desktop`; or the portable `Data/` dir if running in portable mode.
**Why human:** The `commands.getAppDirPath()` call is wired correctly in code, but the actual rendered value requires a running app instance to verify. Code inspection confirms the logic is correct; runtime rendering is the remaining gap.

### Gaps Summary

No gaps. All automated checks pass for all 9 observable truths and all 9 required requirements. The three human verification items above are explicitly deferred per Pierre's instructions ("si jamais les icones vont pas, on corrigera ca plus tard et on fera des tests plus poussés plus tard") and represent acceptable deferred visual confirmation, not implementation gaps.

---

_Verified: 2026-04-16_
_Verifier: Claude (gsd-verifier)_
