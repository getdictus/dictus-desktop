# Roadmap: Dictus Desktop V1

## Overview

Three phases transform the Handy fork into Dictus Desktop. Phase 1 sets the bundle identity — the configuration that must be correct before any build is distributed. Phase 2 delivers the visible rebrand that users and contributors will see: icons, colors, strings, and UI copy. Phase 3 closes the milestone with documentation, updated developer references, and the in-app About panel. Each phase delivers a coherent, verifiable capability; none require the others to be partially complete first.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Bundle Identity** - Set bundle ID, product name, and neutralize inherited Handy build artifacts before any distributed build (completed 2026-04-05)
- [ ] **Phase 2: Visual Rebrand** - Replace all visible Handy identity with Dictus: icons, design tokens, i18n strings, UI components, and onboarding
- [ ] **Phase 3: Documentation and Cleanup** - Rewrite external docs, update internal developer references, and rebrand the in-app About panel

## Phase Details

### Phase 1: Bundle Identity
**Goal**: The application identifies itself as Dictus Desktop at the OS and build level, with no inherited Handy build artifacts that could cause trust or distribution problems
**Depends on**: Nothing (first phase)
**Requirements**: BNDL-01, BNDL-02, BNDL-03, BNDL-04, BNDL-05
**Success Criteria** (what must be TRUE):
  1. The app installs and registers with the OS as "Dictus" (window title, tray menu, macOS dock) — not "Handy"
  2. The bundle identifier is `com.dictus.desktop` in tauri.conf.json and Cargo.toml, and data paths reflect it
  3. A build produces no updater artifact pointing to the upstream cjpais/Handy releases endpoint
  4. The Windows build configuration contains no reference to the upstream maintainer's signing identity
  5. Cargo.toml metadata (name, description, default-run) describes Dictus Desktop, not Handy
**Plans:** 1/1 plans complete
Plans:
- [x] 01-01-PLAN.md — Set Dictus identity in tauri.conf.json and Cargo.toml

### Phase 2: Visual Rebrand
**Goal**: Every user-visible surface reads and looks like Dictus Desktop across all supported platforms and all four locales
**Depends on**: Phase 1
**Requirements**: VISU-01, VISU-02, VISU-03, VISU-04, VISU-05, VISU-06, VISU-07, VISU-08, ONBR-01, LANG-01
**Success Criteria** (what must be TRUE):
  1. The app icon is a Dictus icon on macOS, Windows, and Linux (no Handy hand icon anywhere)
  2. No visible "Handy" text appears anywhere in the app UI when running in any of the four supported locales (en, es, fr, vi)
  3. The app renders Dictus brand colors (blue palette) throughout — no Handy pink/rose palette visible
  4. A first-run user sees onboarding with Dictus name, logo, and tone — no Handy references
  5. The force language control presents "Auto / Français / English" choices clearly and the selected behavior is consistent with the label
**Plans:** 1/4 plans executed
Plans:
- [ ] 02-01-PLAN.md — Generate Dictus platform icons and swap design tokens from pink to blue
- [ ] 02-02-PLAN.md — Create DictusLogo and DictusWaveformIcon, replace in Sidebar and Onboarding
- [ ] 02-03-PLAN.md — Replace all Handy references in i18n locale files (en, es, fr, vi)
- [ ] 02-04-PLAN.md — Redesign recording overlay with 18-bar waveform and iOS-style animations

### Phase 3: Documentation and Cleanup
**Goal**: External documentation presents Dictus Desktop as a first-class product, internal developer docs contain no Handy references, and the in-app About panel reflects the Dictus identity
**Depends on**: Phase 2
**Requirements**: DOCS-01, DOCS-02, DOCS-03
**Success Criteria** (what must be TRUE):
  1. The repository README describes Dictus Desktop (fork attribution, MIT licence, privacy-first positioning) — not Handy
  2. A developer cloning the repo finds CLAUDE.md and BUILD.md free of Handy-specific instructions and URLs
  3. The in-app About panel shows Dictus branding, open-source positioning, and no Handy references
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Bundle Identity | 1/1 | Complete   | 2026-04-05 |
| 2. Visual Rebrand | 1/4 | In Progress|  |
| 3. Documentation and Cleanup | 0/TBD | Not started | - |
