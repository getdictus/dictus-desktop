# Requirements: Dictus Desktop v1.2 Polish & Local-First UX

**Defined:** 2026-04-15
**Renamed/rescoped:** 2026-05-21 — "Polish & Automation" → "Polish & Local-First UX"; SYNC-07..11 and AGENT-01..06 cancelled (see "Future Requirements > Sync Automation" section below)
**Core Value:** L'application doit être identifiable et utilisable comme Dictus Desktop — pas comme Handy — et rester vivante (updates automatiques) sans décrocher du upstream Handy.
**Previous milestones:** v1.0 (rebrand), v1.1 (auto-update & upstream sync)

## v1.2 Requirements

Requirements for the Polish & Local-First UX milestone. Each maps to a roadmap phase.

### Brand Cleanup (BRAND)

<!-- Address Handy brand leaks in user-visible surfaces. -->

- [x] **BRAND-01**: New recording files are named `dictus-{timestamp}.wav` instead of `handy-{timestamp}.wav` (`actions.rs:538`, `history.rs:689`, `tray.rs:273`)
- [x] **BRAND-02**: Portable mode detection marker uses the literal string `"Dictus Portable Mode"` while still recognizing legacy `"Handy Portable Mode"` on existing installs (`portable.rs:30,98`)
- [x] **BRAND-03**: DebugPaths settings panel displays the real data-dir path from Tauri `appDataDir()` API instead of the hardcoded `%APPDATA%/handy` string (`DebugPaths.tsx:29-46`)
- [x] **BRAND-04**: `verify-sync.sh` gains one assertion per BRAND-01/02/03 surface so future upstream syncs cannot silently reintroduce `handy-*.wav`, `"Handy Portable Mode"`, or the hardcoded debug path

### Cross-Platform Icons (ICON)

<!-- Fix Linux black-corners artifact and verify Windows icon integrity. -->

- [x] **ICON-01**: Linux package (deb/AppImage) ships a square 256×256 PNG icon with transparent background (no rounded corners baked in) — no black corners visible in app launcher or taskbar
- [x] **ICON-02**: Windows `.exe` embeds an `icon.ico` containing layers at 16, 24, 32, 48, 64, and 256 pixels (verified via ICO analyzer), with Dictus logo at each layer
- [x] **ICON-03**: `tauri.conf.json > bundle.icon` lists all required platform variants (macOS `.icns`, Windows `.ico`, Linux PNG sizes including 256×256 and 512×512)
- [x] **ICON-04**: Single 1024×1024 square RGBA source PNG committed under `src-tauri/icons/` (or referenced from `dictus-brand` repo) as the source-of-truth for regenerating all platform icons via `bun run tauri icon`

### Sync Infrastructure (SYNC)

<!-- Originally planned automated upstream-sync workflow (draft PRs + CI gate). -->
<!-- Rescoped 2026-05-21: SYNC-06 absorbed by Phase 6; SYNC-07..11 cancelled — manual workflow chosen. See "Future Requirements > Sync Automation" below and todo 2026-05-21-upstream-sync-strategy-review.md. -->

- [x] **SYNC-06**: `verify-sync.sh` relocated to `.github/scripts/verify-sync.sh` (out of `.planning/` tree); all references in UPSTREAM.md updated _(absorbed by Phase 6)_

### macOS Clean Shutdown (SHUT)

<!-- Fix "Dictus quit unexpectedly" dialog on clean quit. -->

- [x] **SHUT-01**: Console.app crash report read and the crashing thread identified (tokio-runtime-worker, main, or Tauri plugin); diagnosis documented in phase plan before fix is chosen
- [x] **SHUT-02**: Based on diagnosis, either (a) explicit cleanup/drop order added before `app.exit(0)` at `lib.rs:254` (tray quit) and `lib.rs:622` (CloseRequested) — e.g., `tauri_plugin_global_shortcut` unregister on main thread via `app_handle.run_on_main_thread`; or (b) `std::process::exit(0)` after `log::logger().flush()` as documented last-resort with upstream bug reference
- [x] **SHUT-03**: Clean quit on macOS (tray menu "Quit Dictus" and post-auto-update relaunch) no longer triggers the "Dictus quit unexpectedly — Reopen / Report / Ignore" OS dialog on macOS Sequoia 15.x

### Privacy / Local-First UX (PRIV)

<!-- Reorder post-process providers and document network surface. -->

- [x] **PRIV-01**: Post-process provider list in settings UI renders local providers (Ollama, Apple Intelligence, Custom local) at the top of the Dropdown, external providers (OpenAI, Anthropic, Groq, Gemini) grouped under a neutral "External — data leaves this device" section label
- [x] **PRIV-02**: Network surface audit documented as `docs/PRIVACY.md` (or in-app Privacy section) listing every outbound endpoint the app can contact (updater check, LLM post-process, model CDN), what data leaves the device, and how to disable each
- [x] **PRIV-03**: Onboarding copy reviewed — local transcription presented as the primary path, cloud post-processing clearly labeled as opt-in external service (minor i18n additions if needed across 20 locales)

## Future Requirements (deferred beyond v1.2)

### Infrastructure

- **INFR-01**: Dictus-owned CDN for onnxruntime models (replace `blob.handy.computer`)
- **INFR-03**: Windows Azure Trusted Signing setup (macOS Developer ID verified in v1.1, Windows OS-level code signing still pending)

### Tech Debt

- **TECH-01**: Rename `handy_keys` module (requires careful handling — `handy-keys` external crate must NOT be renamed)
- **TECH-03**: Cargo binary rename `handy`→`dictus` (defers macOS permission/scripts risk)
- **DATA-01**: On-disk data directory migration (if current path is still `handy`); requires backup logic

### Sync Automation (cancelled 2026-05-21)

Originally planned in v1.2 but removed as over-engineered relative to actual upstream cadence (~15 commits/month, ~1h manual merge cost). Captured in todo `2026-05-21-upstream-sync-strategy-review.md`:

- ~~**SYNC-07..11**~~: Automated draft PR workflow + required CI gate. **Cancelled.** Manual workflow via UPSTREAM.md runbook is sufficient. UPDT-03/UPDT-05 re-assertion (was SYNC-07) reframed as a small ad-hoc improvement.
- ~~**AGENT-01..06**~~: Claude Code adapter + auditor agent layer for upstream-sync PRs. **Cancelled.** Premature given low merge frequency.
- **SYNC-A1**: AI-assisted cherry-pick triage on larger upstream deltas (Open Cloud agent or similar) — retained as long-term exploration if cadence ever grows.

## Out of Scope

Explicitly excluded from v1.2. Documented to prevent scope creep.

| Feature                                                | Reason                                                                                                     |
| ------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------- |
| Auto-merge upstream PRs                                | Dictus rebrand affects same files upstream modifies — human merge gate is mandatory per local-first policy |
| Global `grep handy \| sed` brand replacement           | `handy_keys` / `handy-keys` is an external crate dep — blanket replace breaks the build                    |
| Per-item modal warning on cloud provider selection     | Intrusive repeated friction; section-level labeling is sufficient                                          |
| Hiding cloud providers entirely                        | Users should be able to find them; opt-in, not invisible                                                   |
| Data-dir migration (handy → dictus on disk)            | User-impacting, requires backup logic — defer until justified                                              |
| API-metered Claude agent billing (`ANTHROPIC_API_KEY`) | Cost unacceptable for weekly runs — OAuth Max subscription is mandated                                     |
| Replacing `handy-keys` external crate                  | Build-breaking; out of scope                                                                               |
| Mobile ↔ desktop sync, cloud accounts, Nostr          | PROJECT.md long-term out of scope                                                                          |

## Traceability

Which phases cover which requirements.

| Requirement | Phase                | Status   |
| ----------- | -------------------- | -------- |
| BRAND-01    | Phase 6              | Complete |
| BRAND-02    | Phase 6              | Complete |
| BRAND-03    | Phase 6              | Complete |
| BRAND-04    | Phase 6              | Complete |
| ICON-01     | Phase 6              | Complete |
| ICON-02     | Phase 6              | Complete |
| ICON-03     | Phase 6              | Complete |
| ICON-04     | Phase 6              | Complete |
| SYNC-06     | Phase 6 _(absorbed)_ | Complete |
| SHUT-01     | Phase 7              | Complete |
| SHUT-02     | Phase 7              | Complete |
| SHUT-03     | Phase 7              | Complete |
| PRIV-01     | Phase 8              | Complete |
| PRIV-02     | Phase 8              | Complete |
| PRIV-03     | Phase 8              | Complete |

**Coverage:**

- v1.2 active requirements: 15 total (after 2026-05-21 rescope: 11 cancelled — SYNC-07..11 + AGENT-01..06)
- Mapped to phases: 15 ✓
- Unmapped: 0 ✓

---

_Requirements defined: 2026-04-15_
_Last updated: 2026-05-21 — milestone renamed to "Polish & Local-First UX"; SYNC-07..11 + AGENT-01..06 cancelled; SHUT-01..03 marked Complete (Phase 7 closed 2026-04-23)_
