# Feature Research

**Domain:** Desktop app polish & CI automation (Dictus Desktop v1.2 — Tauri 2.x fork of Handy)
**Researched:** 2026-04-15
**Confidence:** MEDIUM-HIGH (icon platform behavior HIGH; CI agent patterns HIGH; macOS shutdown MEDIUM; privacy UX patterns MEDIUM)

> Scope: v1.2 only. Features shipped in v1.0/v1.1 are not re-researched.

---

## Feature Landscape

### Table Stakes (Users Expect These)

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Platform-native icons (Linux square, Windows multi-size .ico) | Users see the Dictus icon in their app launcher and taskbar. The macOS rounded-corner `.icns` renders with a black backing on Linux — visibly broken and off-brand. | LOW | One new 1024x1024 square source PNG + `bun run tauri icon` regenerates all platform sizes. No per-platform manual work needed. Linux expects square PNG; the `.icns` is macOS-only at bundle time. |
| Brand-clean recording filenames | Users who export recordings or browse transcription history see `handy-TIMESTAMP.wav` filenames. Hard brand leak on a user-visible surface. | LOW-MEDIUM | String replace in `actions.rs:538`, `history.rs:689`, `tray.rs:273`. Decision required: migrate existing DB records (higher scope) or only rename new recordings going forward. |
| Accurate Portable Mode label | `portable.rs:30` and `:98` use the literal `"Handy Portable Mode"` string. User-visible on portable installs; incorrect brand. | LOW | String replace in two locations. Add a migration-window fallback that still recognizes the old string so existing portable installs are not broken immediately. |
| Accurate DebugPaths display | `DebugPaths.tsx:29-46` hardcodes `%APPDATA%/handy`. If the on-disk path is already `dictus`, the display is wrong and misleading to users debugging data locations. | LOW-MEDIUM | Fix display to read actual path from `appDataDir()` Tauri API instead of a hardcoded string. A separate data-dir migration (if the path on disk is still `handy`) is higher scope and deferred. |
| Clean macOS shutdown (no "quit unexpectedly" dialog) | Every clean quit — tray menu "Quit Dictus" or post-update relaunch — triggers an OS-level crash dialog. Erodes user trust even though the app does quit correctly. | MEDIUM-HIGH | Investigation-first: read Console.app crash report to identify the crashing thread. Likely candidates: tokio-runtime-worker audio loop, VAD worker, or a Tauri plugin teardown. Fix strategy determined after diagnosis. Pre-existing upstream (Handy) bug confirmed — not a v1.1 regression. |
| Upstream sync PR already open on arrival | Current flow: weekly cron detects drift → opens GitHub Issue → human manually creates branch + PR. Community action alternatives open a draft PR directly, saving 30+ min of setup per sync. | MEDIUM | Replace custom `upstream-sync.yml` detection with a community action configured for draft-PR-only mode (no auto-merge). Conflict detection varies by action; real merge conflicts may still require manual branch setup. |
| Post-sync identity gate runs automatically | `verify-sync.sh` is currently run manually by the engineer. If forgotten, identity regressions can land on main silently. | LOW | Move `verify-sync.sh` to `.github/scripts/`. Add `verify-sync.yml` CI workflow triggered on PRs labeled `upstream-sync`. Mark as required check in branch protection. |
| UPDT-03/UPDT-05 re-assertion in post-sync gate | Current `verify-sync.sh` (11 assertions) does not check updater pubkey or endpoint config. A sync PR could silently wipe the auto-updater configuration. | LOW | Extend `verify-sync.sh` with two `jq` assertions: `plugins.updater.pubkey` not empty, `plugins.updater.endpoints[0]` contains `getdictus/dictus-desktop`. |

### Differentiators (Competitive Advantage)

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Two-agent Claude Code review on upstream sync PRs | Agent 1 adapts the PR (identity preservation per UPSTREAM.md rules). Agent 2 independently audits Agent 1's work for anything missed. Human approves and merges. Reduces regression risk on each upstream sync without requiring human expertise to re-check every conflict zone. | HIGH | Two sequential `anthropics/claude-code-action@v1` steps in one GitHub Actions job. Different `prompt` per agent. No shared conversation state — each is a fresh Claude Code invocation. Agent 2 must run after Agent 1 (sequential dependency by design). Neither agent can auto-merge; human merge is mandatory. |
| Local-first visual hierarchy in provider settings | Cloud providers (OpenAI, Anthropic, Groq, Gemini) are listed without visual distinction from local providers (Ollama, Apple Intelligence). A "External — data leaves this device" section label makes the local-first posture legible to privacy-sensitive users. | MEDIUM | Frontend-only settings component refactor. No Rust backend changes. Modeled on Obsidian/Logseq pattern: local is the unremarkable default (no badge), cloud is a labeled secondary section. |
| Network surface transparency doc | Users have no way to know what endpoints Dictus contacts and under what conditions. A `PRIVACY.md` or in-app Privacy settings section listing all external calls (updater check, LLM post-processing endpoints, model CDN) provides concrete transparency. | LOW | Grep-based audit of `reqwest` calls in Rust + `fetch` in frontend. Document each endpoint: when triggered, what data leaves the device, how to disable. One-time authoring task. |
| PR checklist template for upstream syncs | Replaces ad-hoc GSD-phase-per-sync with a reusable `.github/pull_request_template/upstream-sync.md`. Items: cap-at-SHA decision, commit risk ratings, `verify-sync.sh` green, Claude agent review complete, human test checklist. | LOW | Static file. High leverage: makes each sync repeatable without a GSD planning phase (~30 min saved per sync). |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Auto-merge upstream PRs | Eliminate human merge step; fully automate sync | Dictus brand assertions conflict with upstream identity fields in the same files (tauri.conf.json, llm_client.rs, 22 locale files). Merge conflicts require human judgment ("keep Dictus pubkey, accept upstream reasoning_effort"). Explicitly out of scope per PROJECT.md. | Automate everything up to the merge commit; keep human approval as the final gate. |
| `std::process::exit(0)` as default shutdown fix | Quick one-liner fix for the macOS crash dialog | Bypasses all Rust destructors — audio buffers not flushed, model state not released, plugin teardown skipped. Acceptable only as a last resort if the root cause is an upstream-unfixable bug. | Diagnose the actual crashing thread via Console.app crash report first. Add explicit drop ordering for managers. Escalate to hard exit only if Tauri bug is confirmed and unactionable. |
| Fixing DebugPaths.tsx as a display-only string change | Cosmetic fix, low effort | If the actual data dir on disk is still `handy` (not `dictus`), fixing only the display string creates a misleading UI that shows a path the user cannot find. | Read the actual value from `appDataDir()` Tauri API. Track data-dir migration (if needed) separately as a scoped future item. |
| Global `grep -r handy \| sed` brand replacement | Complete brand cleanliness in one pass | `handy_keys` / `handy-keys` is the name of an external crate dependency. A blanket replace breaks the build. | Use targeted replacements that explicitly exclude `handy_keys` and `handy-keys` patterns. Extend `verify-sync.sh` exclusion list to match. |

---

## Feature Dependencies

```
[Linux/Windows icon fix]
    └──independent──> pure asset + config change; no runtime dependencies

[Brand cleanup: recording filenames, portable mode string]
    └──independent of other v1.2 features
    └──should extend──> [verify-sync.sh assertions] (catch regressions in future syncs)
    └──may expose──> [data-dir migration decision] (higher scope; defer)

[verify-sync.sh UPDT-03/UPDT-05 extension]
    └──prerequisite──> [verify-sync.sh moved to .github/scripts/]

[verify-sync.yml CI gate]
    └──requires──> [verify-sync.sh at .github/scripts/ path]
    └──triggered by──> [upstream-sync PR label]

[Upstream sync community action (draft PR)]
    └──enables──> [verify-sync.yml CI gate to run on those PRs]
    └──enables──> [Claude agent review workflow trigger]

[Claude Agent 1 — Adaptation]
    └──requires──> [upstream-sync draft PR exists and is open]
    └──requires──> [ANTHROPIC_API_KEY in repo secrets]
    └──produces──> [adapted commits pushed to PR branch]

[Claude Agent 2 — Audit]
    └──requires──> [Claude Agent 1 step complete] (sequential within same job)
    └──requires──> [ANTHROPIC_API_KEY in repo secrets]
    └──produces──> [audit review comment on PR; no commits]

[Privacy UX: provider picker reordering]
    └──independent──> frontend settings component refactor only
    └──informed by──> [network surface audit] (know what calls exist before writing disclosure text)

[macOS clean shutdown fix]
    └──investigation-first──> Console.app crash report diagnosis determines fix strategy
    └──independent──> no dependency on other v1.2 features
```

### Dependency Notes

- **verify-sync.sh path move is a hard prerequisite for the CI gate.** The file currently lives at `.planning/phases/05-upstream-sync/scripts/verify-sync.sh`. The CI workflow needs a stable non-planning path. Move to `.github/scripts/verify-sync.sh` first; update all UPSTREAM.md references and any direct invocations.
- **Claude Agent 2 must run after Agent 1.** Agent 2's audit prompt reads the PR diff including Agent 1's adaptation commits. Sequential steps within a single GitHub Actions job handle this naturally — no inter-workflow messaging or polling needed.
- **Community action must be configured for draft-PR-only mode.** Every reviewed fork-sync action (aormsby, tgymnich, Upstream Sync marketplace action) either direct-merges or opens a PR. For Dictus, the action must be configured to open a draft PR only. Real merge conflicts (tauri.conf.json, locale files) may still fall back to manual branch setup — action is a convenience, not a conflict-resolver.
- **Privacy UX and network audit are independent but logically ordered.** Do the grep audit first to know what endpoints exist; write disclosure text only after the audit is complete.

---

## MVP Definition

### v1.2 Launch With

- [ ] **Linux/Windows icon fix** — visible quality issue; low risk; asset + config
- [ ] **Brand cleanup: recording filenames + portable mode string** — user-visible leaks; extend `verify-sync.sh` for each
- [ ] **DebugPaths display fix** — show `appDataDir()` value; low scope; no migration
- [ ] **verify-sync.sh moved to `.github/scripts/`** — prerequisite for CI gate
- [ ] **verify-sync.sh extended: UPDT-03/UPDT-05 assertions** — closes updater regression gap
- [ ] **verify-sync.yml CI gate** — automatic identity check on all `upstream-sync` PRs
- [ ] **macOS clean shutdown** — investigate first; fix if root cause found; document if upstream bug

### Add After Validation (still v1.2 if time allows)

- [ ] **Community action draft PR** — replace custom detection workflow; ship before Sync #2
- [ ] **PR checklist template** — static file; pairs naturally with community action
- [ ] **Claude Agent 1 + Agent 2 workflow** — high maintenance value; add once community action flow is proven

### Future Consideration (v2+)

- [ ] **Privacy network surface doc** — no blocking urgency; transcription is local; post-processing opt-in/off by default
- [ ] **Local-first provider picker redesign** — UX improvement; defer until user feedback surfaces actual confusion
- [ ] **Data-dir migration (handy → dictus on disk)** — user-impacting; requires backup logic; defer until justified
- [ ] **Onboarding local-first copy audit** — informational; defer until privacy UX audit complete

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Linux/Windows icon fix | HIGH — visible brand quality gap | LOW — asset + config change | P1 |
| Recording filename cleanup | HIGH — user-visible on export/history | LOW-MEDIUM — targeted string replace | P1 |
| macOS clean shutdown fix | HIGH — erodes trust on every quit | MEDIUM-HIGH — investigation first | P1 |
| UPDT-03/UPDT-05 in verify-sync.sh | HIGH — closes updater regression risk | LOW — two jq assertions | P1 |
| verify-sync.sh path move | MEDIUM — prerequisite unblock | LOW — rename + update refs | P1 |
| verify-sync.yml CI gate | HIGH — auto-enforces identity on sync PRs | LOW — single workflow file | P1 |
| Portable mode string fix | MEDIUM — portable installs only | LOW — string replace + fallback | P1 |
| DebugPaths display fix | MEDIUM — debug panel only | LOW — read actual API value | P2 |
| Community action draft PR | HIGH — saves 30+ min per sync | MEDIUM — action selection + config | P2 |
| PR checklist template | MEDIUM — process quality | LOW — static file | P2 |
| Claude Agent 1 (adaptation) | HIGH — long-term maintenance velocity | HIGH — workflow + prompt engineering | P2 |
| Claude Agent 2 (audit) | HIGH — regression safety net | MEDIUM — second sequential step | P2 |
| Provider picker UX redesign | MEDIUM — positioning clarity | MEDIUM — settings component refactor | P3 |
| Network surface privacy doc | LOW — no user complaint yet | LOW — grep audit + doc authoring | P3 |

**Priority key:**
- P1: Must ship for milestone to be complete
- P2: High value; include if time budget allows in v1.2
- P3: Nice to have; defer to v1.3+

---

## Technical Notes Per Feature Area

### Cross-Platform Icons

**Current state:** `tauri.conf.json` bundle.icon lists `32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.icns`, `icon.ico`. No `512x512.png` in the manifest or on disk. The `icon.icns` is macOS-specific (rounded corners applied by macOS shell at system level) — it must not serve as the Linux source.

**How Tauri 2 bundles icons per platform (HIGH confidence — official docs + GitHub issues):**
- **Linux deb/AppImage:** Uses PNG files only. AppImage build scripts reference `usr/share/icons/hicolor/<size>/apps/`. A 256x256 PNG is the practical minimum. Tauri's AppImage bundler has historically expected a 256x256 PNG (Issue #749). Linux desktop environments accept square PNGs with transparent backgrounds — no rounded corners expected or desired.
- **Windows:** Uses `icon.ico` embedded in the `.exe`. The `.ico` must contain layers at 16, 24, 32, 48, 64, 256 pixels. BMP format for small layers, PNG format for 256px layer. Inspect the existing `icon.ico` before deciding to regenerate — it may already be correct.
- **macOS:** Uses `icon.icns` only. macOS applies rounded corners at OS level from a square source. The ICNS is not used on other platforms.

**What to do:** Create a 1024x1024 square source PNG (Dictus logo centered, transparent or solid white background — not the macOS rounded-corner version). Run `bun run tauri icon [source.png]` — this CLI regenerates all sizes for all platforms from one file. Add `512x512.png` to the `bundle.icon` array in `tauri.conf.json` after generation.

**Confidence:** HIGH for process. MEDIUM for whether existing `icon.ico` layers are sufficient without regeneration — inspect with an ICO analyzer tool before deciding.

### Two-Agent Claude Code Review

**Pattern (MEDIUM confidence — verified via official `anthropics/claude-code-action` docs + community examples):**

Two sequential steps in one GitHub Actions job, both using `anthropics/claude-code-action@v1`, with distinct `prompt` values:

Step 1 — Adaptation agent (can push commits, high turn budget):
```yaml
- uses: anthropics/claude-code-action@v1
  with:
    anthropic_api_key: ${{ secrets.ANTHROPIC_API_KEY }}
    prompt: |
      You are reviewing an upstream sync PR for Dictus Desktop (a fork of Handy).
      Follow the rules in UPSTREAM.md exactly. Verify: tauri.conf.json has
      productName=Dictus, identifier=com.dictus.desktop, and the correct updater
      pubkey/endpoint. llm_client.rs has Dictus User-Agent and X-Title headers.
      All 22 locale files have no "Handy" values in i18n keys. If regressions
      found, push corrective commits to the PR branch.
    claude_args: "--max-turns 15"
```

Step 2 — Audit agent (read-only, lower turn budget):
```yaml
- uses: anthropics/claude-code-action@v1
  with:
    anthropic_api_key: ${{ secrets.ANTHROPIC_API_KEY }}
    prompt: |
      You are an independent auditor on an upstream sync PR for Dictus Desktop.
      A prior adaptation agent has already committed fixes. Your job: audit for
      anything missed. Check recording filenames (should be dictus-*, not handy-*),
      portable mode string, DebugPaths display, and any remaining Handy brand
      strings in user-visible surfaces. Post findings as a PR review comment.
      Do NOT push commits — report only.
    claude_args: "--max-turns 5"
```

**Context isolation:** Each step is a fresh Claude Code invocation. No shared conversation state. Agent 2 reads the PR diff (including Agent 1's commits) as fresh input — structurally sound for independent audit.

**Constraints:**
- Neither agent can auto-merge. Human merge is mandatory (same-file conflicts require judgment).
- Trigger: `on: pull_request` filtered to label `upstream-sync`.
- Token cost: each invocation proportional to codebase context loaded. Set `--max-turns` conservatively. Agent 2 should be read-only (lower count).
- Requires `ANTHROPIC_API_KEY` secret and the Claude GitHub App installed on the repo.

### macOS Clean Shutdown

**Root cause categories (MEDIUM confidence — Tauri GitHub issues #4159, #12978, Apple docs on SIGKILL):**

1. `EXC_CRASH (SIGKILL)` — macOS killed a thread that did not respond to SIGTERM within the grace period. Most common cause in Tauri: tokio-runtime-worker blocked on an audio recording loop, VAD inference, or Whisper model unload that takes too long.
2. `EXC_CRASH (SIGABRT)` — Rust panic during destructor teardown. Drop order for Tauri managed state is not guaranteed; a manager's Drop impl may panic if it tries to access already-dropped state.
3. Tauri 2.x does not implement `applicationShouldTerminate` (macOS app delegate method for pre-exit cleanup). This is a confirmed open feature request (tauri-apps/tauri #12978, closed as duplicate of #9198). As of April 2026, macOS apps built with Tauri 2 cannot intercept termination requests to run cleanup before exit.
4. Plugin teardown race: `tauri-plugin-global-shortcut` or `tauri-plugin-single-instance` socket file not cleaned up before process exit on macOS.

**Diagnosis path:** Console.app → User Reports → Dictus → most recent crash report → read `Exception Type` and `Thread N Crashed` stack trace. If `tokio-runtime-worker`, audio/VAD/ML is suspect. If `main thread`, it is Tauri's shutdown sequence. RUST_BACKTRACE=1 on a debug build provides additional context.

**Fix options in priority order:**
1. Add explicit stop/cleanup calls for each manager (audio, transcription, model) before `app.exit(0)` in the shutdown handler in `lib.rs`.
2. Use `tauri::async_runtime::block_on` to await all pending async tasks before triggering exit.
3. If the crash is traced to a specific Tauri plugin: file upstream issue on tauri-apps/plugins-workspace; consider disabling or replacing the plugin.
4. `std::process::exit(0)` as last resort only — acceptable if root cause is confirmed as an upstream unfixable bug and the tradeoff (no graceful cleanup) is acceptable. Document the decision explicitly if this path is taken.

### Post-Sync Regression Gate

**Standard GitHub Actions pattern (HIGH confidence):**

Trigger:
```yaml
on:
  pull_request:
    types: [opened, synchronize, labeled]
```

Gate condition:
```yaml
if: contains(github.event.pull_request.labels.*.name, 'upstream-sync')
```

Gate step:
```yaml
- run: bash .github/scripts/verify-sync.sh
```

Branch protection enforcement: mark the status check as "required" in repo settings for the `main` branch. This prevents merging any upstream sync PR where identity assertions fail — no manual memory required.

### Local-First Provider UX

**Pattern (MEDIUM confidence — inferred from Obsidian and Logseq product design, no official design spec):**

Both Obsidian and Logseq use the same structural principle: local storage is the baseline, unremarkable default with no badge. Cloud sync is a separate, clearly labeled opt-in section:
- **Obsidian:** "Sync" is a named plugin section in settings, labeled "Obsidian Sync." No badge on local vault files — they are simply the default.
- **Logseq:** Local graph needs no label; sync options appear under a distinct section.

**For Dictus provider picker:** Group the list into two visually distinct sections — "Local" (Ollama, Apple Intelligence) at top with no badge, and "External providers" (OpenAI, Anthropic, Groq, Gemini) below with a section label "External — transcription data leaves this device." Consider a small cloud icon or neutral color shift rather than a warning-red badge (which implies danger rather than information).

**What to avoid:**
- Per-item modal warning dialogs on every cloud provider selection — intrusive, repeated friction
- Hiding cloud providers entirely — users should be able to find them; just not defaulted into them
- Warning-red color for cloud providers — implies security problem rather than privacy preference

---

## Sources

- Tauri 2.x official icon docs: https://v2.tauri.app/develop/icons/
- Freedesktop hicolor icon theme specification: https://specifications.freedesktop.org/icon-theme/latest/
- Tauri GitHub issue #749 (AppImage 256x256 icon requirement): https://github.com/tauri-apps/tauri/issues/749
- Tauri GitHub issue #12978 (applicationShouldTerminate feature request): https://github.com/tauri-apps/tauri/issues/12978
- Tauri GitHub issue #4159 (segfault during termination on macOS): https://github.com/tauri-apps/tauri/issues/4159
- Claude Code GitHub Actions official docs: https://code.claude.com/docs/en/github-actions
- anthropics/claude-code-action repository: https://github.com/anthropics/claude-code-action
- Three-body agent orchestration pattern: https://leocardz.com/2026/04/08/orchestrating-agents-with-github-actions
- GitHub Marketplace — Upstream Sync action: https://github.com/marketplace/actions/upstream-sync
- GitHub Marketplace — aormsby/Fork-Sync-With-Upstream-action: https://github.com/aormsby/Fork-Sync-With-Upstream-action
- Microsoft ICO icon construction guide: https://learn.microsoft.com/en-us/windows/apps/design/iconography/app-icon-construction
- Project context files read: `.planning/PROJECT.md`, `UPSTREAM.md`, `src-tauri/tauri.conf.json`, `.planning/todos/pending/*.md`

---

*Feature research for: Dictus Desktop v1.2 Polish & Automation milestone*
*Researched: 2026-04-15*
