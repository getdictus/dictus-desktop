# Project Research Summary

**Project:** Dictus Desktop v1.2 — Polish & Automation
**Domain:** Desktop app polish, brand cleanup, and CI/CD automation for a Tauri 2.x fork
**Researched:** 2026-04-15
**Confidence:** HIGH (architecture/pitfalls based on direct source analysis); MEDIUM (CI agent patterns, macOS shutdown)

## Executive Summary

v1.2 is a polish and automation milestone — no new user-facing features, but meaningful work across three distinct areas: brand correctness, CI automation, and reliability. The codebase is a maintained fork of Handy (cjpais), and the primary tension throughout this milestone is keeping Dictus identity intact while accepting upstream functional improvements. Every feature in scope either fixes a visible quality gap (broken icons, crash dialog, wrong brand strings) or strengthens the infrastructure that catches identity regressions (verify-sync.sh, CI gate, Claude agents).

The recommended approach is to treat the milestone in three parallel tracks that converge before the agent layer ships. Track A (brand/icon polish) and Track C (macOS shutdown) are fully independent of each other and of CI work — they can proceed simultaneously on separate branches. Track D (sync infra refactor) must complete before Track E (Claude Code agent layer) because the agent workflow depends on the community action generating labeled draft PRs. All five tracks share a single critical safety net: `verify-sync.sh` extended with UPDT-03/UPDT-05 assertions and promoted to a required CI check.

The primary risk in this milestone is not technical complexity — individual tasks are well-scoped — but rather the concentration of identity-critical files that can silently regress. `tauri.conf.json`, the 22 locale files, `llm_client.rs` headers, portable.rs magic string, and recording filenames are all surfaces where a merge, an agent fix, or a brand cleanup could introduce a regression that only manifests at runtime. The mitigation strategy is layered: verify-sync.sh covers config fields, the auditor agent covers the diff, and the human merge gate is the final check.

## Key Findings

### Recommended Stack

No new frontend packages and no new Rust crates are required for v1.2. All additions are workflow files, icon assets, or code pattern changes. Three genuine tooling additions: `peter-evans/create-pull-request@v8` (replaces custom detection workflow), `anthropics/claude-code-action@v1` (Claude agent layer), and `bunx tauri icon` CLI invocation (already installed).

**Core additions:**
- `peter-evans/create-pull-request@v8` (v8.1.1): replaces custom SHA-tracking issue workflow with idempotent draft PR creation — supports `draft: true`, `labels`, handles branch conflicts
- `anthropics/claude-code-action@v1` (v1.0.96): two sequential jobs — adapter (write access, 15 turns) then auditor (read-only, 5 turns, posts PR comment)
- OAuth auth via `CLAUDE_CODE_OAUTH_TOKEN`: use Max subscription quota, not API key billing — correct auth path for the first-party action
- `tokio-util` (conditional): only if crash report confirms background-task shutdown root cause; check `cargo tree | grep tokio-util` first

**Critical constraints:**
- Pin Claude Code action to `v1` major tag; `@v1` floating tag is stable
- `tauri-plugin-updater 2.10.0` already installed — UPDT-03/UPDT-05 assertions verify pubkey/endpoint fields required by this version

### Expected Features

**Must have (P1):**
- Linux/Windows icon fix — visible black-corner artifact on Linux; `icon.ico` multi-resolution for Windows
- Recording filename cleanup (`handy-` → `dictus-`) — user-visible; forward-only rename, no DB migration
- Portable mode magic string update — legacy compatibility fallback required
- DebugPaths display fix — use `appDataDir()` Tauri API, not hardcoded `%APPDATA%/handy`
- verify-sync.sh path move + UPDT-03/UPDT-05 assertions
- verify-sync.yml CI gate (required status check)
- macOS clean shutdown — investigation-first; fix or document upstream bug

**Should have (P2 — if time allows):**
- Community action draft PR workflow
- PR checklist template
- Claude adapter + auditor agent workflow

**Defer to v1.3+:**
- Privacy network surface documentation
- Local-first provider picker redesign
- Data-dir migration (handy → dictus on disk)

### Architecture Approach

v1.2 changes span five non-overlapping zones: Rust brand strings, frontend display, icon assets, Rust shutdown paths, and GitHub Actions workflows. This isolation means all tracks can proceed on separate branches without conflicts.

**Major components:**
1. **CI Pipeline** — `upstream-sync.yml` body replaced; `verify-sync.yml` (new required gate); `claude-agents.yml` (new, two-job sequential: adapter then auditor with `needs: [adapter]`)
2. **Rust shutdown** — explicit cleanup before `app.exit(0)` in tray quit handler (`lib.rs:254`) and `on_window_event` CloseRequested (`lib.rs:622`); diagnosis via Console.app crash report first
3. **Brand cleanup** — targeted replacement at 6 specific file:line locations; `handy_keys`/`handy-keys` explicitly excluded (external crate, must not be renamed)
4. **Icon pipeline** — `bunx tauri icon app-icon.png` from 1024x1024 square RGBA source; add `icons/linux-square-256x256.png` to `bundle.icon` array
5. **Settings UI** — provider reorder in `settings.rs:524-603`; `ProviderSelect.tsx` visual divider if Dropdown supports option groups

**Shutdown crash suspects (priority order):** `tauri_plugin_global_shortcut` (drops first, CGEventTap requires main thread), `AudioRecordingManager`/CPAL CoreAudio stream, `TranscriptionManager`/Metal GPU context.

### Critical Pitfalls

1. **Community action auto-resolves identity conflicts** (C1) — never use `--strategy-option=theirs/ours`; configure for draft-PR-only with raw conflicts intact; `verify-sync.sh` as required CI check before enabling the action

2. **Portable mode magic string breaks existing installs** (C12) — changing detection without legacy fallback silently moves user data to system app data dir; always ship the dual-check with in-place marker upgrade

3. **Prompt injection via upstream commit messages** (C5) — agent adapter reads untrusted content; structure prompts with explicit trust boundaries; `verify-sync.sh` as hardcoded CI step independent of agents

4. **`handy_keys`/`handy-keys` false positives in brand grep** (C14) — external crate; blanket grep+sed breaks the build; use scoped exclusion grep

5. **Recording filename rename misread as data migration** (C11) — DB stores actual filenames; forward-only rename does not break old entries; do not run a DB migration renaming column values without renaming WAV files on disk

6. **Auditor agent race condition** (C9) — adapter and auditor must not run in parallel; use `needs: [adapter]` and `contents: read` permission on auditor job

## Implications for Roadmap

### Phase 1: Brand & Icon Polish
**Rationale:** No dependencies on other phases; highest user-visible impact; lowest risk. Establishes the extended verify-sync.sh baseline that later CI phases depend on.
**Delivers:** Correct platform icons, clean recording filenames, accurate portable mode detection, accurate DebugPaths display, extended verify-sync.sh assertions
**Avoids:** C11, C12, C13, C14, C15
**Research flag:** Standard patterns — all file:line locations confirmed in ARCHITECTURE.md.

### Phase 2: macOS Clean Shutdown
**Rationale:** Independent of all other phases; must diagnose before implementing.
**Delivers:** Clean app quit on macOS; documented root cause or upstream bug report
**Avoids:** Using `std::process::exit(0)` as default; relying on undefined Tauri drop order
**Research flag:** MEDIUM confidence — root cause requires Console.app crash report diagnosis before fix strategy is chosen.

### Phase 3: Privacy/Local-First UX (optional v1.2)
**Rationale:** Independent, low-risk, frontend-only. Lowest urgency — can slip to v1.3.
**Delivers:** Provider picker reordered (local providers first), optional "External" section divider
**Research flag:** MEDIUM confidence — assess current Dropdown component API before committing to option-group rendering.

### Phase 4: Sync Infra Refactor
**Rationale:** Hard prerequisite for Phase 5. Community action must be producing labeled draft PRs before agent workflow has meaningful input.
**Delivers:** Community action, `verify-sync.yml` required check, PR template, `upstream-sha.txt` deprecation
**Avoids:** C1, C2, C3, C4
**Research flag:** MEDIUM confidence — test `peter-evans/create-pull-request@v8` with `workflow_dispatch` before scheduling weekly cron; verify idempotency when sync branch is already open.

### Phase 5: Claude Code Agent Layer
**Rationale:** Highest value per future sync; highest complexity. Builds on Phase 4's labeled PR infrastructure.
**Delivers:** Adapter agent (commits identity fixes) + auditor agent (independent PR comment review)
**Avoids:** C5, C6, C7, C8, C9, C10
**Research flag:** MEDIUM-LOW confidence — OAuth billing status has one non-official source; validate `claude setup-token` before building full workflow. Agent prompt engineering for UPSTREAM.md rules is untested — plan for iteration.

### Phase Ordering Rationale
- Phases 1, 2, 3 are structurally independent; can run on separate branches simultaneously
- Phase 4 before Phase 5 is a hard dependency: no labeled PRs = agent never fires
- Phase 1 should land before Phase 4 so the CI gate runs the extended verify-sync.sh from day one
- Phase 3 can slip to v1.3 without affecting any other phase

### Research Flags

Needs deeper validation:
- **Phase 4:** Community action idempotency — test with `workflow_dispatch` before live weekly cron
- **Phase 5:** OAuth token validity — verify `claude setup-token` before building agent workflow; iterate on prompt engineering after first real upstream sync

Standard patterns (skip research-phase):
- **Phase 1:** All file:line locations confirmed by direct source analysis; `bunx tauri icon` behavior in official docs
- **Phase 2:** Diagnosis-first pattern well-understood; fix strategies documented with specific code patterns

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All additions verified against official docs and GitHub release pages |
| Features | HIGH | Priority ratings from direct code inspection; dependency analysis confirmed in ARCHITECTURE.md |
| Architecture | HIGH | All conclusions from direct source file analysis with specific file:line locations |
| Pitfalls | HIGH (brand/icon/shutdown) / MEDIUM (agent behavior) | Brand pitfalls from source analysis; agent pitfalls extrapolated from LLM-in-CI patterns |

**Overall confidence:** HIGH for Phases 1-4. MEDIUM for Phase 5 agent layer.

### Gaps to Address

- **OAuth billing status:** Single non-official source (April 2026). Validate `claude setup-token` before Phase 5 implementation.
- **macOS crash root cause:** Cannot confirm fix strategy without Console.app crash report. Three suspects identified but priority is inference, not confirmed diagnosis.
- **Community action conflict behavior:** How `peter-evans/create-pull-request@v8` handles merge conflicts in identity files needs live testing.
- **`upstream-sha.txt` replacement signal:** Idempotency gap documented in C4 — resolve design decision (keep file or document new signal) during Phase 4.

## Sources

### Primary (HIGH confidence)
- [Tauri v2 Updater plugin docs](https://v2.tauri.app/plugin/updater/) — pubkey/endpoint config
- [Tauri 2.x Icon docs](https://v2.tauri.app/develop/icons/) — `bunx tauri icon` CLI, output files
- [peter-evans/create-pull-request GitHub](https://github.com/peter-evans/create-pull-request) — v8.1.1 confirmed
- [anthropics/claude-code-action GitHub](https://github.com/anthropics/claude-code-action) — v1.0.96, trigger modes confirmed
- [claude-code-action docs/setup.md](https://github.com/anthropics/claude-code-action/blob/main/docs/setup.md) — OAuth auth mode
- Direct source analysis: `lib.rs`, `actions.rs`, `portable.rs`, `settings.rs`, `tray.rs`, `managers/history.rs`
- Direct source analysis: `.github/workflows/`, `src-tauri/tauri.conf.json`, `src/components/settings/`
- [Tauri issue #12534](https://github.com/tauri-apps/tauri/issues/12534) — macOS cleanup crash root cause
- [Tauri issue #12978](https://github.com/tauri-apps/tauri/issues/12978) — `applicationShouldTerminate` missing in Tauri 2.x

### Secondary (MEDIUM confidence)
- [Tauri issue #4159](https://github.com/tauri-apps/tauri/issues/4159) — macOS termination segfault
- [Freedesktop hicolor icon spec](https://specifications.freedesktop.org/icon-theme/latest/) — Linux icon size requirements
- [tauri-apps/tauri Discussion #10206](https://github.com/orgs/tauri-apps/discussions/10206) — GitHub Releases `latest.json` endpoint

### Tertiary (LOW confidence)
- [KissAPI blog April 2026](https://kissapi.ai/blog/claude-code-github-actions-setup-guide-2026.html) — OAuth vs API key billing post-restriction; needs validation

---
*Research completed: 2026-04-15*
*Ready for roadmap: yes*
