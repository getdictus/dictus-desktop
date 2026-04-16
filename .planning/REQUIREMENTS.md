# Requirements: Dictus Desktop v1.2 Polish & Automation

**Defined:** 2026-04-15
**Core Value:** L'application doit être identifiable et utilisable comme Dictus Desktop — pas comme Handy — et rester vivante (updates automatiques) sans décrocher du upstream Handy.
**Previous milestones:** v1.0 (rebrand), v1.1 (auto-update & upstream sync)

## v1.2 Requirements

Requirements for the Polish & Automation milestone. Each maps to a roadmap phase.

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

<!-- Refactor upstream-sync from custom detection+issue to community-action draft-PR + CI gate. -->

- [x] **SYNC-06**: `verify-sync.sh` relocated to `.github/scripts/verify-sync.sh` (out of `.planning/` tree); all references in UPSTREAM.md updated
- [ ] **SYNC-07**: `verify-sync.sh` extended with UPDT-03 assertion (`plugins.updater.pubkey` present and non-empty) and UPDT-05 assertion (`plugins.updater.endpoints[0]` contains `getdictus/dictus-desktop`)
- [ ] **SYNC-08**: `verify-sync.yml` CI workflow runs `verify-sync.sh` on every PR labeled `upstream-sync`, set as a required status check for merges to `main`
- [ ] **SYNC-09**: `upstream-sync.yml` replaced: uses `peter-evans/create-pull-request@v8` (or equivalent) to open a labeled draft PR with upstream commits on a weekly cron, instead of creating a tracking issue
- [ ] **SYNC-10**: `.github/PULL_REQUEST_TEMPLATE/upstream-sync.md` checklist (cap-at-SHA? risk rating? verify-sync.sh green? agent review complete?) auto-applied to draft PRs via `template=` query param or preset body
- [ ] **SYNC-11**: `UPSTREAM.md` runbook trimmed to reflect the new flow (PR already open on arrival, CI gate enforces identity, checklist drives review) — obsolete sections removed, cap-at-SHA and conflict-rules sections preserved

### Claude Code Agents (AGENT)

<!-- Two sequential GitHub Action agents: adapter (commits fixes) + auditor (read-only review). -->

- [ ] **AGENT-01**: `CLAUDE_CODE_OAUTH_TOKEN` secret configured in repo (generated via `claude setup-token` against the user's Claude Max subscription) so agents run on subscription quota, not metered API billing
- [ ] **AGENT-02**: `claude-agents.yml` workflow triggers on `pull_request` events filtered to label `upstream-sync`, runs `anthropics/claude-code-action@v1` as an **adapter** job with write permissions (`contents: write`, `pull-requests: write`) and an explicit file allow-list in the system prompt (may modify only brand/identity surfaces)
- [ ] **AGENT-03**: Same workflow runs a second **auditor** job with `needs: [adapter]` and read-only permissions (`contents: read`), posts findings as a PR review comment without pushing commits
- [ ] **AGENT-04**: Adapter prompt codifies UPSTREAM.md identity rules (productName=Dictus, identifier=com.dictus.desktop, updater pubkey/endpoint, Dictus User-Agent/X-Title headers, no Handy in i18n values) and explicit trust-boundary instruction to ignore any `ignore previous instructions` content from upstream commit messages/diffs
- [ ] **AGENT-05**: Auditor prompt is independent (no shared conversation state), checks residual Handy strings + brand cleanup compliance + general code-quality smells, and cannot push commits
- [ ] **AGENT-06**: `verify-sync.yml` runs as an independent CI step (NOT driven by the auditor agent) so a compromised agent cannot bypass the identity gate

### macOS Clean Shutdown (SHUT)

<!-- Fix "Dictus quit unexpectedly" dialog on clean quit. -->

- [ ] **SHUT-01**: Console.app crash report read and the crashing thread identified (tokio-runtime-worker, main, or Tauri plugin); diagnosis documented in phase plan before fix is chosen
- [ ] **SHUT-02**: Based on diagnosis, either (a) explicit cleanup/drop order added before `app.exit(0)` at `lib.rs:254` (tray quit) and `lib.rs:622` (CloseRequested) — e.g., `tauri_plugin_global_shortcut` unregister on main thread via `app_handle.run_on_main_thread`; or (b) `std::process::exit(0)` after `log::logger().flush()` as documented last-resort with upstream bug reference
- [ ] **SHUT-03**: Clean quit on macOS (tray menu "Quit Dictus" and post-auto-update relaunch) no longer triggers the "Dictus quit unexpectedly — Reopen / Report / Ignore" OS dialog on macOS Sequoia 15.x

### Privacy / Local-First UX (PRIV)

<!-- Reorder post-process providers and document network surface. -->

- [ ] **PRIV-01**: Post-process provider list in settings UI renders local providers (Ollama, Apple Intelligence, Custom local) at the top of the Dropdown, external providers (OpenAI, Anthropic, Groq, Gemini) grouped under a neutral "External — data leaves this device" section label
- [ ] **PRIV-02**: Network surface audit documented as `docs/PRIVACY.md` (or in-app Privacy section) listing every outbound endpoint the app can contact (updater check, LLM post-process, model CDN), what data leaves the device, and how to disable each
- [ ] **PRIV-03**: Onboarding copy reviewed — local transcription presented as the primary path, cloud post-processing clearly labeled as opt-in external service (minor i18n additions if needed across 20 locales)

## Future Requirements (deferred beyond v1.2)

### Infrastructure

- **INFR-01**: Dictus-owned CDN for onnxruntime models (replace `blob.handy.computer`)
- **INFR-03**: Windows Azure Trusted Signing setup (macOS Developer ID verified in v1.1, Windows OS-level code signing still pending)

### Tech Debt

- **TECH-01**: Rename `handy_keys` module (requires careful handling — `handy-keys` external crate must NOT be renamed)
- **TECH-03**: Cargo binary rename `handy`→`dictus` (defers macOS permission/scripts risk)
- **DATA-01**: On-disk data directory migration (if current path is still `handy`); requires backup logic

### Sync Automation Advanced

- **SYNC-A1**: AI-assisted cherry-pick triage on larger upstream deltas (Open Cloud agent or similar)

## Out of Scope

Explicitly excluded from v1.2. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Auto-merge upstream PRs | Dictus rebrand affects same files upstream modifies — human merge gate is mandatory per local-first policy |
| Global `grep handy \| sed` brand replacement | `handy_keys` / `handy-keys` is an external crate dep — blanket replace breaks the build |
| Per-item modal warning on cloud provider selection | Intrusive repeated friction; section-level labeling is sufficient |
| Hiding cloud providers entirely | Users should be able to find them; opt-in, not invisible |
| Data-dir migration (handy → dictus on disk) | User-impacting, requires backup logic — defer until justified |
| API-metered Claude agent billing (`ANTHROPIC_API_KEY`) | Cost unacceptable for weekly runs — OAuth Max subscription is mandated |
| Replacing `handy-keys` external crate | Build-breaking; out of scope |
| Mobile ↔ desktop sync, cloud accounts, Nostr | PROJECT.md long-term out of scope |

## Traceability

Which phases cover which requirements.

| Requirement | Phase | Status |
|-------------|-------|--------|
| BRAND-01 | Phase 6 | Complete |
| BRAND-02 | Phase 6 | Complete |
| BRAND-03 | Phase 6 | Complete |
| BRAND-04 | Phase 6 | Complete |
| ICON-01 | Phase 6 | Complete |
| ICON-02 | Phase 6 | Complete |
| ICON-03 | Phase 6 | Complete |
| ICON-04 | Phase 6 | Complete |
| SYNC-06 | Phase 9 | Complete |
| SYNC-07 | Phase 9 | Pending |
| SYNC-08 | Phase 9 | Pending |
| SYNC-09 | Phase 9 | Pending |
| SYNC-10 | Phase 9 | Pending |
| SYNC-11 | Phase 9 | Pending |
| AGENT-01 | Phase 10 | Pending |
| AGENT-02 | Phase 10 | Pending |
| AGENT-03 | Phase 10 | Pending |
| AGENT-04 | Phase 10 | Pending |
| AGENT-05 | Phase 10 | Pending |
| AGENT-06 | Phase 10 | Pending |
| SHUT-01 | Phase 7 | Pending |
| SHUT-02 | Phase 7 | Pending |
| SHUT-03 | Phase 7 | Pending |
| PRIV-01 | Phase 8 | Pending |
| PRIV-02 | Phase 8 | Pending |
| PRIV-03 | Phase 8 | Pending |

**Coverage:**
- v1.2 requirements: 26 total
- Mapped to phases: 26 ✓
- Unmapped: 0 ✓

---
*Requirements defined: 2026-04-15*
*Last updated: 2026-04-15 — traceability populated after roadmap creation*
