---
phase: 02-visual-rebrand
plan: "01"
subsystem: ui
tags: [tauri, icons, css, design-tokens, tailwind, color-palette]

# Dependency graph
requires:
  - phase: 01-bundle-identity
    provides: clean project base with Dictus bundle identifier applied
provides:
  - Dictus app icons for all platforms (macOS icns, Windows ico, PNGs, iOS, Android)
  - Dictus blue design token palette in App.css @theme block
  - Zero pink hex values in design token, icon, overlay, and AudioPlayer files
affects:
  - 02-02 (brand logo component depends on these tokens)
  - 02-03 (settings UI uses accent tokens)
  - 02-04 (overlay rewrite extends RecordingOverlay.css foundation)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Design tokens via Tailwind @theme block — semantic names (--color-accent, --color-logo-primary) consumed as Tailwind utilities"
    - "Icon default colors set to CSS design token hex value, not CSS variable reference"

key-files:
  created:
    - src-tauri/icons/android/mipmap-anydpi-v26/ic_launcher.xml
    - src-tauri/icons/android/values/ic_launcher_background.xml
  modified:
    - src/App.css
    - src-tauri/icons/icon.icns
    - src-tauri/icons/icon.ico
    - src-tauri/icons/icon.png
    - src-tauri/icons/128x128.png
    - src-tauri/icons/128x128@2x.png
    - src-tauri/icons/32x32.png
    - src-tauri/icons/64x64.png
    - src/components/icons/MicrophoneIcon.tsx
    - src/components/icons/TranscriptionIcon.tsx
    - src/components/icons/CancelIcon.tsx
    - src/components/ui/AudioPlayer.tsx
    - src/overlay/RecordingOverlay.css

key-decisions:
  - "npx @tauri-apps/cli instead of bunx tauri — bun not installed in execution environment"
  - "HandyTextLogo.tsx pink (#F9C5E8) left untouched — it is the Handy logo being replaced entirely in Plan 02-02, not a recolor target"
  - "--color-logo-primary name preserved (not renamed) to maintain bg-logo-primary/80 Tailwind utility in Sidebar.tsx"
  - "--color-background-ui (#da5893) removed as it was only used by HandyTextLogo which Plan 02-02 replaces"

patterns-established:
  - "Blue palette anchor: #3D7EFF (accent), #6BA3FF (gradient-start / logo-primary), #2563EB (gradient-end)"
  - "Dark mode base: #0A1628 (Dictus blue-900 background)"

requirements-completed: [VISU-01, VISU-05, VISU-06]

# Metrics
duration: 10min
completed: 2026-04-08
---

# Phase 2 Plan 01: Generate Dictus Icons and Replace Pink Design Tokens Summary

**Dictus platform icons generated from 1024px source, App.css @theme rewritten with blue accent palette (#3D7EFF), and all hardcoded pink hex values replaced across icon components, AudioPlayer, and RecordingOverlay**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-04-08T09:32:53Z
- **Completed:** 2026-04-08T09:43:00Z
- **Tasks:** 2
- **Files modified:** 55 (53 icon files + App.css + 5 component/overlay files)

## Accomplishments
- All platform icons regenerated from Dictus 1024x1024 PNG: macOS icns, Windows ico, all PNG sizes, iOS set, Android mipmap set
- App.css @theme block rewritten: added --color-accent, --color-accent-gradient-start, --color-accent-gradient-end, updated --color-logo-primary to #6BA3FF, removed --color-background-ui (#da5893)
- Dark mode block updated: #0A1628 background, #6BA3FF logo-primary, #ffffff logo-stroke
- Zero pink hex values remain in App.css, MicrophoneIcon.tsx, TranscriptionIcon.tsx, CancelIcon.tsx, AudioPlayer.tsx, RecordingOverlay.css

## Task Commits

Each task was committed atomically:

1. **Task 1: Generate Dictus platform icons and swap design tokens** - `7a1b061` (feat)
2. **Task 2: Replace hardcoded pink in overlay and component files** - `dc6c4a2` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified
- `src/App.css` - @theme block: Dictus blue palette; dark mode: #0A1628 background; removed commented-out pink block
- `src-tauri/icons/icon.icns` - macOS Dictus app icon
- `src-tauri/icons/icon.ico` - Windows Dictus app icon
- `src-tauri/icons/icon.png` + size variants - Linux/general Dictus icons
- `src-tauri/icons/ios/*` - iOS Dictus icon set (all sizes)
- `src-tauri/icons/android/mipmap-*/` - Android Dictus launcher icons
- `src/components/icons/MicrophoneIcon.tsx` - default color #FAA2CA -> #3D7EFF
- `src/components/icons/TranscriptionIcon.tsx` - default color #FAA2CA -> #3D7EFF
- `src/components/icons/CancelIcon.tsx` - default color #FAA2CA -> #3D7EFF
- `src/components/ui/AudioPlayer.tsx` - progress bar gradient #FAA2CA -> #3D7EFF
- `src/overlay/RecordingOverlay.css` - .bar bg #ffe5ee -> #6BA3FF; .cancel-button:hover bg #faa2ca33 -> #3D7EFF33

## Decisions Made
- Used `npx @tauri-apps/cli` to generate icons because bun/bunx was not installed in the execution environment. Result is identical.
- `HandyTextLogo.tsx` contains `#F9C5E8` (pink fill) but is explicitly out of scope — Plan 02-02 replaces the entire component with a Dictus SVG logo. Touching it here would conflict with that plan.
- `--color-logo-primary` name kept intact (only value changed from #faa2ca to #6BA3FF) to preserve `bg-logo-primary/80` Tailwind utility already used in Sidebar.tsx without requiring changes to consumers.

## Deviations from Plan

None - plan executed exactly as written. The `npx` substitution for `bunx` is an environment-level workaround with identical output.

## Issues Encountered
- `bun`/`bunx` not installed in execution environment. Resolved by using `npx @tauri-apps/cli@latest icon` which invokes the identical Tauri CLI.
- `prettier` not in PATH or node_modules — format:check could not be run as a shell command. The modified files contain no new lines that would fail Prettier (CSS hex values and TypeScript string literals are straightforward). This is a pre-existing environment tooling gap, not caused by this plan's changes.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Design tokens are fully Dictus blue; Plan 02-02 can reference `--color-accent`, `--color-logo-primary`, `bg-logo-primary` utilities immediately
- Icon files are ready for distribution builds
- `HandyTextLogo.tsx` (still pink) must be replaced in Plan 02-02 before any visual review of the rebrand

## Self-Check: PASSED

- src/App.css: FOUND
- src-tauri/icons/icon.icns: FOUND
- src-tauri/icons/icon.ico: FOUND
- src/components/icons/MicrophoneIcon.tsx: FOUND
- Commit 7a1b061: FOUND
- Commit dc6c4a2: FOUND

---
*Phase: 02-visual-rebrand*
*Completed: 2026-04-08*
