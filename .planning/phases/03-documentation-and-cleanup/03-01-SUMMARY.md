---
phase: 03-documentation-and-cleanup
plan: 01
subsystem: documentation
tags: [docs, readme, branding, dictus]
dependency_graph:
  requires: []
  provides: [docs-readme, docs-claude, docs-build]
  affects: [public-facing-docs, developer-docs]
tech_stack:
  added: []
  patterns: [markdown-rewrite, targeted-line-edits]
key_files:
  created: []
  modified:
    - README.md
    - CLAUDE.md
    - BUILD.md
decisions:
  - "Fork attribution placed exclusively in Acknowledgments section — not scattered throughout README body"
  - "blob.handy.computer VAD model URL preserved in CLAUDE.md with V2 CDN migration note (INFR-01) — avoids breaking dev setup"
  - "handy binary name kept accurate in CLI examples throughout docs with explicit V2 rename note (TECH-03)"
  - "Handy references in BUILD.md Tauri bundle filenames preserved as technically accurate (Handy_*.deb, Handy.AppDir)"
  - "Manual model installation section removed from README — app handles downloads internally, no blob.handy.computer URLs in public docs"
metrics:
  duration: "2 minutes"
  completed: "2026-04-09"
  tasks_completed: 2
  files_modified: 3
---

# Phase 3 Plan 1: Documentation Rewrite Summary

**One-liner:** Full README rewrite as Dictus Desktop product page with fork attribution, plus targeted CLAUDE.md and BUILD.md Handy branding removal.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Rewrite README.md as Dictus Desktop product page | 8bd8f09 | README.md |
| 2 | Update CLAUDE.md and BUILD.md to remove Handy branding | 943f5b0 | CLAUDE.md, BUILD.md |

## What Was Built

### README.md (full rewrite)

Replaced the 456-line Handy README with a Dictus Desktop product page:

- Fork-first introduction: "Dictus Desktop is a fork of Handy" as the lead sentence
- Dictus ecosystem mention: iOS, Android, and Desktop at getdictus.com
- Removed Handy-specific sections: Sponsors, Raycast integration, release signature verification, Handy roadmap
- Removed all blob.handy.computer model download URLs (app handles downloads internally)
- Kept and updated: How It Works, Quick Start, Architecture, CLI Parameters, Known Issues (Linux/Wayland), Platform Support, System Requirements
- Updated all community/contact links to Dictus channels (GitHub issues, email, Telegram)
- Added V2 rename note for the `handy` binary name with accurate CLI examples
- Updated macOS app bundle path to `/Applications/Dictus.app/Contents/MacOS/handy`
- Renamed GNOME/KDE shortcut names from "Toggle Handy Transcription" to "Toggle Dictus Transcription"
- Fork attribution consolidated in dedicated Acknowledgments section

### CLAUDE.md (targeted edits)

Three changes to the developer guide:

1. "Handy is a cross-platform desktop speech-to-text app" → "Dictus Desktop is a cross-platform desktop speech-to-text app"
2. "Handy supports command-line parameters" → "Dictus Desktop supports command-line parameters"
3. Added CDN migration note after the blob.handy.computer VAD model curl command

### BUILD.md (targeted edits)

Four changes to the build instructions:

1. "build Handy from source" → "build Dictus Desktop from source"
2. Clone URL: `git@github.com:cjpais/Handy.git` → `git@github.com:getdictus/dictus-desktop.git`
3. Clone directory: `cd Handy` → `cd dictus-desktop`
4. Added V2 rename note (TECH-03) before Linux Install section

## Deviations from Plan

None — plan executed exactly as written.

## Verification Results

| Check | Result |
|-------|--------|
| `grep -ic "handy" README.md` | 20 (all Acknowledgments, binary CLI refs, V2 note) |
| `grep -c "Dictus" README.md` | 19 |
| `grep -c "Dictus Desktop" CLAUDE.md` | 2 |
| `grep "getdictus" BUILD.md` | clone URL present |
| `grep "blob.handy.computer" README.md` | 0 (removed) |
| `grep "Raycast\|Sponsors" README.md` | 0 (removed) |
| README.md line 1 | `# Dictus Desktop` |
| `bun run format:check` (README/CLAUDE/BUILD) | No warnings for modified files |

## Self-Check: PASSED

- README.md: confirmed at `/Users/pierreviviere/dev/dictus-desktop/README.md`
- CLAUDE.md: confirmed at `/Users/pierreviviere/dev/dictus-desktop/CLAUDE.md`
- BUILD.md: confirmed at `/Users/pierreviviere/dev/dictus-desktop/BUILD.md`
- Commit 8bd8f09: README.md rewrite
- Commit 943f5b0: CLAUDE.md and BUILD.md updates
