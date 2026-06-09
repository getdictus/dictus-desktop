# Milestones

## v1.3 Smart Modes & Local LLM (Shipped: 2026-06-09)

**Phases completed:** 4 phases (10, 11, 12, 13), 36 plans

**Stats:**
- Files changed: 189 | LOC: +32751/-2381 (includes Upstream Sync #2 merge volume from PREP-03)
- Git range: `15a5d2b` → `934312b` (215 commits, ~62 `feat`)
- Timeline: 2026-05-29 → 2026-06-09 (~11 days)
- Audit status: `tech_debt` — all 21 requirements satisfied, all 5 E2E flows wired, 0 orphaned exports; phases 10/12/13 `passed`, phase 11 `human_needed` (non-blocking Win/Linux GPU runtime smoke, user-accepted); Nyquist 2/4 compliant. FAIL gate not triggered.

> GSD milestones are internal planning units and **do not** produce git tags.
> See `docs/VERSIONING.md` for the tag/version convention.

**Key accomplishments:**
- **Embedded LLM runtime, all platforms (LLM-01..04, PREP-01)** — in-process GGUF inference via `llama-cpp-2` 0.1.146 (no external Ollama), background-thread so the overlay/tray/shortcuts never block; GPU auto-select (Metal embedded via `GGML_METAL_EMBED_LIBRARY=ON` on macOS, Vulkan on Win/Linux) with CPU fallback; idle-timeout unload coexisting with the transcription model. The ggml duplicate-symbol conflict (surfaced at link once `LlamaModel::load_from_file` was called, not at the Phase-10 spike) was resolved in `.cargo/config.toml` with linker "keep first definition" (`-Wl,--allow-multiple-definition` GNU-ld/lld + `/FORCE:MULTIPLE` MSVC); Windows x64 needed `CMAKE_GENERATOR=Ninja` for `vulkan-shaders-gen`. CI green on all 7 platforms.
- **Functional local model library (MDL-01..05)** — curated 4-model catalogue (Qwen2.5-1.5B, Gemma-3-4B, Phi-4-Mini, Llama-3.2-3B) with on-disk size shown before download; in-app download with live progress / cancel / resume, SHA256-verified, served from HuggingFace CDN (never `blob.handy.computer`); delete-to-reclaim; custom GGUF by drag/drop or file-pick; the "Bibliothèque de modèles locaux" placeholder replaced by the real library anchored top-of-page. "Embedded (local)" selectable as a post-process provider alongside Apple Intelligence / Custom→Ollama / cloud (cloud stays opt-in).
- **Smart Modes data layer + lossless migration (MODE-01..04)** — `SmartMode` type + CRUD commands, fixture-tested v1.2→v1.3 prompt migration (7 tests, `settings_schema_version`, no data loss; legacy `transcribe_with_post_process` binding transferred to `smart_mode_{active_id}`), embedded provider routing through the runtime, per-mode global shortcut registration via the `smart_mode_{id}` prefix wired through coordinator + actions + both init loops.
- **Smart Modes UI (MODE-03, MODE-05, MODE-06)** — visual card list replacing the single-prompt dropdown; create / edit / delete in-panel; inline per-mode shortcut bind with structured, fully-localized conflict detection (backend emits a codified `SHORTCUT_CONFLICT` payload — no English prose crosses the boundary — rendered via `t()`; exact-duplicate + base-overlap rules; conflicting mode's localized name surfaced; error hoisted to a full-width card row to avoid chip overflow).
- **First-class offline translation (TRANS-01, TRANS-02)** — translation as a Smart Mode running fully offline through the active embedded LLM. TranslateGemma was dropped after a controlled A/B benchmark (`examples/translation_ab.rs`, 14 texts × 4 models) — a same-size generic (Gemma-3-4B, 0/14 polluted) matched/beat the dedicated translate model (~6/14 polluted, runaway generation); enum variant kept for settings back-compat. Engine choice is recommendation-only (one active model via the main selector). Output-language fix: `whatlang` detection injects an explicit "write in {lang}" directive placed first. `--toggle-post-process` / SIGUSR1 re-routed to the active Smart Mode (`signal_handle.rs`, commit b80d00a).
- **Full 20-locale localization (L10N-01)** — reversed the earlier "names-only / English-fallback" deferral: every new user-facing string (model library, Smart Modes UI, default mode names, translation, error messages) truly translated across all 20 locales; `check-translations.ts` gained an opt-in `--check-untranslated` guardrail (with allowlist for loanwords / Romance "No" / Germanic "Name"); `bun run check:translations` and `:untranslated` both exit 0.

**Known tech debt (non-blocking, carried forward):**
- Phases 10 & 11 VALIDATION.md left `status: draft`, `nyquist_compliant: false` — `/gsd:validate-phase 10` and `11` to close retroactively (same pattern as v1.1 phase 5 and v1.2 phases 6–9, still open)
- Phase 11 Windows-x64 + Linux release-build runtime GPU smoke (download → load → embedded inference with Vulkan offload / CPU fallback) not human-tested; CI build/link green, user-accepted as non-blocking
- Doc wording drift (archived as-planned, outcomes noted in `v1.3-REQUIREMENTS.md`): MDL-01 names 3 models (Qwen3-4B / Qwen2.5-1.5B / TranslateGemma-4B), code ships 4 after a post-UAT refresh; MODE-02 "~10 default modes" → 10 curated modes ship, only Clean Up seeded on first run, the other 9 one-click-addable via the template picker (deliberate 13-05 safe-default decision)
- Bookkeeping: LLM-03 (11-01) and MODE-01 (12-01) absent from `requirements_completed` SUMMARY frontmatter though both VERIFIED + WIRED
- Pre-existing `drop_non_drop` clippy warning at `managers/llm.rs:1221` (predates Phase 12)
- AMD Vulkan driver 25.11.1 known crash with Vulkan SDK 1.4.328.1 — monitor llama.cpp issue #17432 before Windows beta
- Carried from earlier: `blob.handy.computer` CDN for onnxruntime (INFR-01); Windows builds unsigned at OS level (INFR-03)

See `.planning/milestones/v1.3-MILESTONE-AUDIT.md` for the full audit report (21/21 requirements, 5/5 E2E flows traced).

---

## v1.2 Polish & Local-First UX (Shipped: 2026-05-29)

**Phases completed:** 4 phases (6, 7, 8, 9), 16 plans

**Stats:**
- Files changed: 178 | LOC: +16160/-701
- Git range: `24d8e61` → `15a5d2b` (110 commits)
- Timeline: 2026-04-16 → 2026-05-29 (44 days, includes Phase 8 UAT iteration cycle)
- Audit status: `passed` — all 18 requirements satisfied, 4/4 phases with `status: passed` VERIFICATION.md, 7/7 cross-phase wiring + 5/5 E2E flows verified

> GSD milestones are internal planning units and **do not** produce git tags.
> See `docs/VERSIONING.md` for the tag/version convention.

**Key accomplishments:**
- **Brand cleanup complete (BRAND-01..04, SYNC-06)** — recording filenames `dictus-*.wav` (`actions.rs:538`); `"Dictus Portable Mode"` marker (`portable.rs`); `DebugPaths.tsx` rewired through `commands.getAppDirPath()` runtime command; `verify-sync.sh` relocated to `.github/scripts/` and extended with BRAND-01a/02a/03a/ICON-02a assertions (15 checks total)
- **Cross-platform icons regenerated (ICON-01..04)** — opaque navy-tile Linux PNG (alpha min=max=1.0, black-corners artifact physically impossible); 6-layer Windows ICO at 16/24/32/48/64/256px; `tauri.conf.json bundle.icon` extended to 7 entries including 256×256 and 512×512; 1024×1024 RGBA source-of-truth committed to `dictus-brand` repo
- **macOS clean shutdown (SHUT-01..03)** — root-cause diagnosis committed before fix (`tauri-plugin-global-shortcut` Drop releasing CGEventTap from `__cxa_finalize_ranges` on main thread); `flush_and_exit(app, code)` helper in `lib.rs:81` funnels both exit sites (tray quit + `CloseRequested`); macOS branch releases CGEventTap on main runloop via `app_handle.run_on_main_thread` before `app.exit(0)`; debug-mode `simulate_updater_restart` command + UI button validates updater-relaunch path; multi-day daily-use validation (2026-04-16 → 2026-04-23) saw no "Dictus quit unexpectedly" dialog
- **Privacy / local-first UX (PRIV-01..03)** — platform-aware default provider (Apple Intelligence on macOS arm64, Custom (local) elsewhere) at `settings.rs:496-505`; `ProviderPicker` rewritten with Local/Cloud tabs (role=tablist, aria-selected) replacing cloud opt-in toggle; Apple Intelligence unavailability Alert inlined via `renderRowExtras`; API key field hidden for Custom (local); Ollama link visibly underlined at rest; `docs/PRIVACY.md` (75 lines) documents every outbound endpoint; library coming-soon placeholder hoisted to top; three-pillar marketing block removed; 25 new i18n keys propagated across 19 sibling locales
- **v1.2 audit gap closure (AUDIT-01..03)** — 33 `cargo clippy -- -D warnings` errors resolved across 14 Rust files (6 `#[derive(Default)]` substitutions, 2 `to_*` methods take `self` by value, double-ref/needless-borrow cleanup, `Error::other`, `is_some_and`/`is_ok_and`); `07-VERIFICATION.md` retroactively authored from `07-01-SUMMARY.md` with 3/3 truths + 4 commit SHAs; `08-UAT.md` frontmatter promoted to `status: passed` with `## Closure` section listing all 9 Phase 8 closure SHAs

**Known tech debt (non-blocking, recorded in REQUIREMENTS.md or memory):**
- All 4 phase VALIDATION.md files left as `status: draft`, `nyquist_compliant: false` — `/gsd:validate-phase 6` (and 7/8/9) to close retroactively before v1.3
- **TECH-04** — `llm_client.rs:137 send_chat_completion_with_schema` 8-arg refactor deferred (suppressed via `#[allow(clippy::too_many_arguments)]`, 15-30 min estimated)
- Pre-existing lint failure on `src/components/icons/DictusLogo.tsx` (`i18next/no-literal-string` on hardcoded `Dictus` SVG `<text>`); pre-existing Prettier failures on 16 `.planning/*.md` files
- Carried from v1.1: `blob.handy.computer` CDN for onnxruntime (INFR-01); v0.1.0 Windows builds unsigned at OS level (INFR-03)
- macOS quit-unexpectedly crash non-reproducible under clean env post-fix (memory `project_macos_quit_crash_investigation.md`) — fix is defensive; original trigger possibly multi-install pollution

See `.planning/milestones/v1.2-MILESTONE-AUDIT.md` for the full passed audit report and `.planning/v1.2-INTEGRATION-CHECK.md` for cross-phase wiring detail.

---

## v1.1 Auto-Update & Upstream Sync (Shipped: 2026-04-14)

**Phases completed:** 2 phases, 7 plans

**Stats:**
- Files changed: 49 | LOC: +4277/-517
- Git range: `84cc346` → `bedf363` (27 commits)
- Timeline: 2026-04-11 → 2026-04-14 (4 days)
- App releases shipped within this milestone: `v0.1.0` (2026-04-11)
- Audit status: `tech_debt` — all 15 requirements satisfied; non-blocking debt documented below

> GSD milestones are internal planning units and **do not** produce git tags.
> See `docs/VERSIONING.md` for the tag/version convention.

**Key accomplishments:**
- **Auto-updater end-to-end** — Ed25519 signing keypair, tauri.conf.json updater config (pubkey + endpoint + createUpdaterArtifacts), CI asset-prefix fixes, and UpdateChecker.tsx fallback corrected (UPDT-01…UPDT-09)
- **v0.1.0 shipped** — first public Dictus Desktop release on `getdictus/dictus-desktop` with latest.json live and tauri-updater assertions green (UPDT-10)
- **Upstream detection automated** — weekly `upstream-sync.yml` workflow compares `cjpais/Handy` HEAD to committed SHA and files a GitHub issue, idempotent on unchanged state (SYNC-01, SYNC-02)
- **UPSTREAM.md runbook** — fork-point-aware merge procedure with 9 hot-zone conflict rules and post-merge verification checklist (SYNC-03)
- **First upstream sync merged** — 4 post-v0.8.2 commits from `cjpais/Handy` merged via PR #3, capped at `fdc8cb7`, identity integrity preserved through `verify-sync.sh` (SYNC-04, SYNC-05)
- **Triple-backup signing custody** — Ed25519 private key backed up to two independent Bitwarden items + iCloud age-encrypted offline, local copy deleted

**Known tech debt (non-blocking):**
- Post-sync gate in UPSTREAM.md §6 does not re-assert UPDT-03/UPDT-05 — regression risk flagged for v1.2
- Phase 5 VALIDATION.md left in draft (`nyquist_compliant: false`) — `/gsd:validate-phase 5` to close retroactively
- `validate.sh` TECH-03 anti-regression grep window too narrow; 8 substantive config assertions still pass
- `build.yml` still references `blob.handy.computer` for onnxruntime (tracked as INFR-01)
- v0.1.0 Windows builds unsigned at OS level (SmartScreen warning expected, Azure Trusted Signing deferred)

See `.planning/v1.1-MILESTONE-AUDIT.md` (archived to `.planning/milestones/v1.1-MILESTONE-AUDIT.md`) for full audit report.

---

## v1.0 Dictus Desktop V1 (Shipped: 2026-04-10)

**Phases completed:** 3 phases, 8 plans, 0 tasks

**Key accomplishments:**
- (none recorded)

---
