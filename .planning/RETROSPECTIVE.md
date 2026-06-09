# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v1.2 — Polish & Local-First UX

**Shipped:** 2026-05-29
**Phases:** 4 (6, 7, 8, 9) | **Plans:** 16 | **Commits:** 110 | **Timeline:** 44 days (2026-04-16 → 2026-05-29)

### What Was Built
- **Brand cleanup complete** — `dictus-*.wav` filenames, "Dictus Portable Mode" marker, runtime app-data path in DebugPaths; verify-sync.sh relocated to `.github/scripts/` and extended to 15 assertions (BRAND-01..04, SYNC-06)
- **Platform icons regenerated** — opaque navy-tile Linux PNG (black-corners impossible), 6-layer Windows ICO, `tauri.conf.json bundle.icon` with 7 entries, 1024×1024 RGBA source-of-truth (ICON-01..04)
- **macOS clean shutdown** — `flush_and_exit` helper releases CGEventTap on main runloop before `app.exit(0)`; debug-mode `simulate_updater_restart` validates updater-relaunch path; multi-day no-crash validation (SHUT-01..03)
- **Privacy / local-first UX** — platform-aware default provider, Local/Cloud tabs replacing cloud opt-in toggle, `docs/PRIVACY.md` documenting network surface, 25 i18n keys propagated across 19 sibling locales (PRIV-01..03)
- **Audit gap closure** — 33 clippy errors resolved across 14 files, retroactive 07-VERIFICATION.md authored, 08-UAT promoted to passed (AUDIT-01..03)

### What Worked
- **Wave-0 verify-sync.sh extension pattern** — Phase 6 Plan 01 extended `verify-sync.sh` with BRAND-01a/02a/03a assertions before any source change, giving downstream plans (02/03) immediate green-make targets. Same pattern as v1.1 SYNC validate.sh.
- **Diagnosis-before-fix gate (SHUT-01)** — Committing `## Diagnosis` to `07-01-PLAN.md` *before* writing code forced the team to rule out wrong suspects (path b: `std::process::exit`) and pick the right one (path a: explicit CGEventTap release on main thread). First commit on the Phase 7 branch was the diagnosis.
- **Single-script i18n propagation (08-10)** — One Node script (JSON.parse/stringify roundtrip + insertion-order preservation) propagated 25 new keys across 19 locales in 1m 56s. All 19 succeeded first try.
- **Retroactive verification backfill (Phase 9)** — Authoring `07-VERIFICATION.md` from `07-01-SUMMARY.md` + Pierre's multi-day observation produced a defensible audit artifact without re-running the original verification work. Pattern reusable for any phase that closed without VERIFICATION.md.
- **Audit-driven gap closure** — The 2026-05-28 audit produced 3 concrete gaps (AUDIT-01/02/03), each scoped tightly to a single deliverable; Phase 9 closed all three in 14 minutes of execution.

### What Was Inefficient
- **Phase 8 UAT iteration cost** — Initial Phase 8 (plans 08-01..05) shipped a cloud opt-in toggle + three-pillar marketing block, both rejected at UAT. Rework spanned plans 08-06..10 (5 additional plans across ~2 weeks), 6 commits per gap. A pre-UAT design contract (UI-SPEC.md was authored but not enforced against the cloud-toggle pattern) might have caught the pivots earlier.
- **Phase 7 crash non-reproducible after fix** — Fix is defensive but the original trigger was never fully isolated under clean env (likely multi-install pollution). Multi-day real-world observation served as final evidence, but a clean reproduction would have been more defensible.
- **Clippy debt accumulated invisibly** — 33 clippy errors had accumulated across 14 files before the v1.2 audit surfaced them. Phase 7 VALIDATION.md *claimed* the clippy gate as a sampling rate but the gate was never run in CI. Phase 9 closure was small (60min) but the deferred cost grew silently.
- **Nyquist VALIDATION.md never finalised** — All 4 phases authored VALIDATION.md at planning time but left them as `status: draft, nyquist_compliant: false`. Pattern repeats from v1.1 (Phase 5 VALIDATION.md draft). Authoring without enforcement = paperwork.

### Patterns Established
- **Audit-then-gap-closure-phase** — `/gsd:audit-milestone` produces concrete REQ-IDs (AUDIT-XX), gap-closure becomes its own phase. Cleaner than retro-amending phases or shipping with documented debt.
- **3-source requirement cross-reference** — `VERIFICATION.md status` × `SUMMARY frontmatter` × `REQUIREMENTS.md checkbox` must agree per REQ-ID. Caught Phase 7 (missing VERIFICATION.md) and Phase 8 (stale UAT frontmatter) in v1.2 audit.
- **Retroactive VERIFICATION.md from SUMMARY** — Defensible when SUMMARY is comprehensive + real-world observation exists. Document source-of-evidence transparently in VERIFICATION.md frontmatter.
- **TECH-XX debt naming** — Suppressed lints + deferred refactors get a TECH-XX entry in REQUIREMENTS.md "Future Requirements" with file:line + rationale + size estimate. Visible debt, not silent.
- **Sibling-locale propagation by script** — 19-locale changes done by one-shot Node script with JSON-roundtrip + per-locale failure log+continue. Faster than 19 manual edits, deterministic, audit-friendly.

### Key Lessons
1. **Author the diagnosis before the fix.** SHUT-01 forced this; it caught the wrong suspect early. Make it a gate, not a checkbox.
2. **UI-SPEC contracts must enforce against design pivots, not just describe layout.** Phase 8's cloud-toggle survived the UI-SPEC but failed UAT — the spec described layout, not interaction model. Future UI phases should specify "what the user does" not just "what the user sees."
3. **Don't claim a CI gate you don't run.** Phase 7 VALIDATION.md sampling rate listed `cargo clippy -- -D warnings` but no CI step enforced it. By v1.2 audit, 33 errors had accumulated. Either wire the gate or remove the claim.
4. **Audit-to-gap-closure-to-archive is a clean shipping flow.** Audit produces concrete gaps → gaps become a small surgical phase → re-audit confirms green → archive. Avoids the "ship with debt" trap.
5. **Nyquist VALIDATION.md as planner-stub is a smell.** Three milestones in a row (v1.0/v1.1/v1.2) authored VALIDATION.md but never finalised it. Either drop the artifact or make it a wave-0 gate.

### Cost Observations
- Model mix: not tracked per-session in this project (no telemetry capture configured)
- Sessions: spanned 44 days; longest single execution Phase 9 (14 min, 17 files) — surgical audit closure
- Notable: Phase 8 iteration cost (5 follow-up plans for UAT pivots) was 2× the original Phase 8 plan count

---

## Milestone: v1.3 — Smart Modes & Local LLM

**Shipped:** 2026-06-09
**Phases:** 4 (10, 11, 12, 13) | **Plans:** 36 | **Commits:** 215 | **Timeline:** ~11 days (2026-05-29 → 2026-06-09)

### What Was Built
- **Embedded LLM runtime, all platforms (LLM-01..04, PREP-01)** — in-process GGUF via `llama-cpp-2`, GPU auto-select (Metal embedded / Vulkan / CPU fallback), background-thread inference, idle unload; ggml duplicate-symbol conflict resolved at link via keep-first-definition; CI green on 7 platforms
- **Functional model library (MDL-01..05)** — 4-model curated catalogue with size-before-download, in-app download/cancel/resume + SHA256 from HuggingFace CDN, delete-to-reclaim, custom GGUF drag/drop
- **Smart Modes data layer + UI (MODE-01..06)** — lossless v1.2→v1.3 migration (7 tests), visual card list, create/edit/delete, per-mode shortcut with structured localized conflict detection
- **First-class offline translation (TRANS-01..02)** — runs through the active embedded LLM; TranslateGemma dropped after A/B benchmark; whatlang output-language directive
- **Full 20-locale localization (L10N-01)** — reversed the names-only deferral; `--check-untranslated` guardrail added
- **Prerequisite gate (PREP-01..03)** — Upstream Sync #2 + selective cherry-pick policy, TECH-04 struct refactor (closing v1.2 debt), ggml feasibility

### What Worked
- **A/B benchmark before committing to a specialized model** — `examples/translation_ab.rs` (14 texts × 4 models) gave hard evidence that the generic Gemma-3-4B beat the dedicated TranslateGemma. Killed a plausible-but-wrong assumption with data, not opinion.
- **Structured error code across the FE/BE boundary** — making the backend emit a codified `SHORTCUT_CONFLICT` payload (code + params) and rendering it via frontend `t()` solved both "no English prose leaks" and "fully localizable" in one design. Cleaner than translating backend strings.
- **Migration verified against a real v1.2 settings fixture** — 7 migration tests against an actual JSON fixture caught data-loss edge cases before any user upgrade.
- **TDD on the LlmManager backend (11-01)** — the runtime's load/infer/unload/idle-watcher core was test-first, which paid off when the ggml link conflict forced rework without breaking behavior.

### What Was Inefficient
- **Phase 13 ballooned to 26 plans (4 original + 22 gap-closure across 4 UAT rounds).** The Smart Modes shortcut-conflict UX alone churned through ~8 plans ([B5]→[G11]→[G12]→[G13]→[G16]→[G17]) as each live FR-build UAT surfaced a new layer (cross-mode collision → base-overlap → English prose → localized name → chip overflow). Each was real, but the long tail suggests the conflict UX should have had a dedicated design contract up front.
- **The "names-only / English-fallback" i18n shortcut backfired.** Plans 13-02/13-15 deferred real translation (English values in non-EN locales); live FR UAT [G14] then forced a full re-translation (13-22/23) across 19 locales. The deferral created more work than doing it once.
- **The Phase-10 ggml spike gave a false-negative.** 10-03 concluded "conflict did not manifest" — but the spike only added the dependency without *calling* llama, so the linker never pulled ggml objects. The real conflict surfaced in 11-01 on 5 platforms. A spike must exercise the actual code path it's de-risking.
- **Catalogue churned post-UAT.** Shipped 4 models, not the 3 originally specified (Qwen3-4B reasoning + TranslateGemma both dropped after runtime testing). The ROADMAP/REQUIREMENTS wording was never synced, becoming documented drift.

### Patterns Established
- **Benchmark harness as a decision artifact** — a committed `examples/*.rs` A/B harness produces reproducible evidence for model/engine choices. Reusable for any future model swap.
- **Codified error payloads over translated backend strings** — backend returns `CODE|param|param`, frontend owns all user-facing text via `t()`. Keeps localization single-sourced in the locale files.
- **Spikes must execute the risky path** — adding a dependency ≠ de-risking it; the spike has to call the function that triggers the link/runtime behavior in question.
- **Live localized-build UAT** — testing in an actual FR build (not just EN + a translation check) surfaced English leakage, untranslated conflict names, and layout overflow that automated checks missed.

### Key Lessons
1. **A spike that doesn't exercise the failure mode is theater.** The ggml "did not manifest" finding cost a Phase-11 scramble. De-risking means running the exact code path, not just compiling the dependency.
2. **Don't defer i18n as "English fallback for now."** It reads as done (the check passes) but isn't; the rework to do it properly later exceeded doing it once. For a 20-locale app, translate at the point of adding the string.
3. **A churny UX surface deserves a design contract before plan 1.** The shortcut-conflict feature consumed ~8 gap-closure plans across 4 UAT rounds because each round revealed the next layer. A UI-SPEC enumerating conflict *cases* (exact-dup, base-overlap, localized name, long-string layout) up front would have collapsed the tail.
4. **Sync docs to runtime reality at close.** Catalogue + seed-count drift (3→4 models, 10→1 seeded) was real and defensible but never written back into ROADMAP/REQUIREMENTS — only caught at audit. Wording sync should be part of the post-UAT refresh, not deferred.
5. **Validate-on-data beats validate-on-intuition for model choice.** TranslateGemma *should* have won; it didn't. The benchmark was the difference between shipping the right model and the obvious one.

### Cost Observations
- Model mix: not tracked per-session (no telemetry capture configured)
- Sessions: ~11 days wall-clock; dominated by Phase 13's 4-round UAT loop
- Notable: Phase 13 gap-closure plans (22) were 5.5× the original plan count (4) — the inverse of v1.2's Phase 8 (2× rework); the conflict-UX tail is the main driver

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Phases | Plans | Key Change |
|-----------|--------|-------|------------|
| v1.0 | 3 | 8 | First milestone — rebrand baseline established |
| v1.1 | 2 | 7 | Wave-0 pattern (validate.sh FAIL-first) introduced; triple-backup signing custody |
| v1.2 | 4 | 16 | Audit-driven gap-closure phase pattern; 3-source requirement cross-reference; UAT pivots motivating UI-SPEC enforcement gap |
| v1.3 | 4 | 36 | First feature-heavy milestone (embedded LLM + Smart Modes); benchmark-as-decision-artifact; live localized-build UAT; longest gap-closure tail (22 plans in Phase 13) |

### Cumulative Tech Debt

| Item | Origin | Status |
|------|--------|--------|
| Nyquist VALIDATION.md draft state | v1.0 | Carries through v1.0–v1.3 (7 phases in draft: 5, 6, 7, 8, 9, 10, 11) |
| `blob.handy.computer` CDN (INFR-01) | v1.0 | Documented in `docs/PRIVACY.md` v1.2; LLM weights moved to HuggingFace CDN in v1.3; onnxruntime migration still pending |
| Windows OS-level signing (INFR-03) | v1.1 | Deferred; SmartScreen warning accepted |
| Cargo binary rename `handy`→`dictus` (TECH-03) | v1.0 | Deferred; macOS permission risk |
| Module rename `handy_keys` (TECH-01) | v1.0 | Deferred; external crate `handy-keys` must not be touched |
| `llm_client.rs:137` 8-arg refactor (TECH-04) | v1.2 | ✓ Resolved v1.3 (PREP-02) |
| `DictusLogo.tsx` i18next lint failure | pre-v1.2 | Pre-existing; out of scope |
| Phase 11 Win/Linux runtime GPU smoke | v1.3 | CI build/link green; runtime inference not human-tested (user-accepted) |
| `drop_non_drop` clippy warning (`managers/llm.rs:1221`) | v1.3 | Pre-existing; low priority |
| Doc wording drift (MDL-01 catalogue, MODE-02 seed count) | v1.3 | Capability satisfied; ROADMAP/REQUIREMENTS wording not synced (noted in archive) |

### Top Lessons (Verified Across Milestones)

1. **Wave-0 / FAIL-first scripts make downstream plans testable from commit 1.** Used by v1.1 (validate.sh), v1.2 Phase 6 (verify-sync.sh assertions added before source change). Worth promoting as a workflow default.
2. **Identity integrity needs script-enforced guards through fork-sync merges.** v1.0 established the principle; v1.1 made it a CI gate (verify-sync.sh); v1.2 extended assertions when surfaces grew (BRAND-01a/02a/03a/ICON-02a). Without the script, every upstream sync would silently regress brand surfaces.
3. **VALIDATION.md without a CI hook is paperwork.** Verified across all 4 milestones: 7 of 13 phase VALIDATION.md files (phases 5–11) are in draft state. The backlog only grows — either wire them as gates or stop authoring them.
4. **Audit-before-archive surfaces hidden debt cheaply.** v1.1 audit `tech_debt`; v1.2 caught 31 clippy errors + missing VERIFICATION.md; v1.3 caught catalogue/seed drift + the CLI post-process regression (fixed before close). Cheap insurance against "ship and discover."
5. **Decide model/engine choices on benchmark data, not intuition.** v1.3 dropped the "obvious" specialized translation model after an A/B harness proved a generic model beat it. The harness is a committable decision artifact.
6. **A de-risking spike must execute the path it de-risks.** v1.3's ggml spike compiled the dependency but never called it, giving a false "no conflict" that surfaced a phase later. Compiling ≠ exercising.
