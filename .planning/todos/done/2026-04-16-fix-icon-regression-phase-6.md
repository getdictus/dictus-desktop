---
created: 2026-04-16T13:15:00.000Z
title: Fix Phase 6 icon regression — middle blue bar dropped across all platforms
area: branding
files:
  - ../dictus-brand/source/appicon-desktop-1024.png
  - ../dictus-brand/source/github-avatar.svg
  - ../dictus-brand/source/appicon-light.svg
  - src-tauri/icons/icon.png
  - src-tauri/icons/icon.icns
  - src-tauri/icons/icon.ico
  - src-tauri/icons/*.png
  - src-tauri/icons/android/**/*.png
---

## Problem

Phase 6 (commit `6f530d4 chore(icons): regenerate from square opaque-tile source`) broke the app icon across **every platform** — macOS, Windows, Linux, Android. Visually reported by Pierre on macOS 2026-04-16 when testing the Phase 7 build.

**Regression:** The pre-Phase-6 icon had 3 bars with a bright blue gradient middle bar (#6BA3FF→#2563EB) — matching the `dictus-brand/source/github-avatar.svg` (3 bars) and `appicon-light.svg` (3 bars, gradient background, glow stroke). The post-Phase-6 icon has only 2 gray bars — the middle blue bar is missing entirely, and the background is flat opaque navy (#0A1628) instead of the gradient.

**Root cause:** `dictus-brand` commit `24d4c78` claims it rasterized `source/github-avatar.svg` (3 bars + blue accent) to `source/appicon-desktop-1024.png` "via ImageMagick magick at 300dpi". But the output PNG only contains 2 gray bars — ImageMagick's SVG delegate (MSVG/RSVG) dropped the `fill="url(#bar-accent)"` reference with `gradientUnits="userSpaceOnUse"`. Phase 6 then ran `bun run tauri icon` against this broken source, propagating the loss to all platform assets.

**Detection gap:** `verify-sync.sh`'s `ICON-02a` assertion only checks the `.ico` has the right *layer dimensions* (16/24/32/48/64/256). It has no visual-content check, so it passed against a broken icon. Also, Phase 6 did not include a human visual-verification gate after regeneration.

Observed on Dictus v0.1.1 built from `main@8e595d2`, macOS Sequoia 15.x.

## Solution

Address as **Phase 6.1** (decimal gap-closure phase) once Phase 7 finishes. Use this todo as the scope seed when creating the phase.

### Fix steps

1. **Re-render `appicon-desktop-1024.png` in `dictus-brand` repo** from `source/github-avatar.svg` (NOT ImageMagick's SVG delegate). Options:
   - `rsvg-convert -w 1024 -h 1024 source/github-avatar.svg -o source/appicon-desktop-1024.png` (preferred — handles `gradientUnits="userSpaceOnUse"` correctly)
   - Inkscape CLI: `inkscape --export-width=1024 --export-filename=source/appicon-desktop-1024.png source/github-avatar.svg`
   - Chrome headless as last resort
2. **Verify the new PNG has 3 bars with a blue middle before committing in the brand repo.**
3. **Commit in `dictus-brand`** with a message noting the ImageMagick gradient-drop root cause.
4. **Back in `dictus-desktop`:** `bun run tauri icon ../dictus-brand/source/appicon-desktop-1024.png` — accepts default output set.
5. **Fallback regen** of 256x256 and 512x512 (not emitted by `tauri icon`) via ImageMagick, same as Phase 6's Plan 06-03.
6. **Visual verification gate (MANDATORY — Pierre's explicit ask):** before any commit in `dictus-desktop`, Pierre opens each generated PNG by hand and visually confirms the 3-bar blue-middle design is present. List to check:
   - `src-tauri/icons/icon.png`
   - `src-tauri/icons/32x32.png`, `64x64.png`, `128x128.png`, `128x128@2x.png`, `256x256.png`, `512x512.png`
   - Every `Square*Logo.png` (Windows)
   - Every `android/mipmap-*/ic_launcher*.png` (Android)
   - `icon.ico` via Preview or an online ICO viewer (check all 6 layers)
   - `icon.icns` via Preview (check multiple resolutions)
   - If any PNG is wrong, loop back to step 1 — do not commit.
7. **Build + install** the `.app` on macOS, confirm the Finder / Dock icon shows the 3-bar blue-middle design.
8. **Add a visual-content check** to `verify-sync.sh` — one approach: compare a SHA-256 of the regenerated `icon.png` against a committed golden hash, or use `compare -metric AE reference.png icon.png` to fail on pixel drift. Whatever is added, it must fail if the middle bar is dropped again.
9. **Update this todo to `done/`** once Phase 6.1 completes and `/gsd:verify-work 6.1` passes.

### Why this deserves its own phase, not a drive-by fix

- Touches 20+ binary files across 4 platforms (macOS, Windows, Linux, Android).
- Cross-repo (`dictus-brand` + `dictus-desktop`).
- Requires a new verify-sync visual-content assertion so the regression is caught automatically next time.
- Benefits from the GSD audit trail (CONTEXT, PLAN, VERIFICATION) given user-visible surface impact.

### What to preserve from Phase 6

- The opaque-tile decision was correct for the Linux black-corner issue (ICON-01) — the new asset must stay opaque.
- The `bundle.icon` 256/512 addition (ICON-03, commit `a46b1be`) is good — keep it.
- `verify-sync.sh` relocation to `.github/scripts/` — keep.

### Out of scope for this todo

- Changing the icon design itself (this is a pure fidelity fix — the SVG design is correct, only the rasterization broke).
- iOS icons (separately managed, unaffected).
