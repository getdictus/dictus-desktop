---
phase: 8
slug: privacy-local-first-ux
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-21
---

# Phase 8 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | ESLint + Prettier + TypeScript strict + `check-translations.ts` + `cargo test` + manual UAT (Playwright 1.58.0 only used for smoke) |
| **Config file** | `eslint.config.js`, `tsconfig.json`, `scripts/check-translations.ts`, `src-tauri/Cargo.toml`, `playwright.config.ts` (verify exists; if not, no Wave 0 install needed — Phase 8 doesn't introduce new automated UI tests) |
| **Quick run command** | `bun run lint && bun run format:check` |
| **Full suite command** | `bun run lint && bun run format:check && bun run check:translations && bun run build && cd src-tauri && cargo fmt -- --check && cargo clippy --all-targets -- -D warnings && cargo test` |
| **Estimated runtime** | ~5s quick / ~90s full (build dominates) |

---

## Sampling Rate

- **After every task commit:** Run `bun run lint && bun run format:check`
- **After every plan wave:** Run full suite command above
- **Before `/gsd:verify-work`:** Full suite must be green + manual UAT screenshot of new picker on macOS captured
- **Max feedback latency:** ~5 seconds (lint), ~90 seconds (full suite)

---

## Per-Task Verification Map

> Filled by planner after PLAN.md tasks are defined. Reference verification map in `08-RESEARCH.md` §"Validation Architecture > Phase Requirements → Test Map" — copy task IDs here once plans land.

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| TBD-by-planner | — | 0 | PRIV-01 | Rust unit | `cd src-tauri && cargo test default_post_process_provider_id` | ❌ W0 | ⬜ pending |
| TBD-by-planner | — | 0 | PRIV-01 | Rust unit | `cd src-tauri && cargo test default_post_process_providers_custom_id_stable` | ❌ W0 | ⬜ pending |
| TBD-by-planner | — | — | PRIV-01 | Static | `bun run lint` | ✅ | ⬜ pending |
| TBD-by-planner | — | — | PRIV-01 | Static | `bun run check:translations` | ✅ | ⬜ pending |
| TBD-by-planner | — | — | PRIV-01 | Visual UAT | Manual: Settings → Post Process → verify stacked layout, local section above external | N/A | ⬜ pending |
| TBD-by-planner | — | — | PRIV-01 | Visual UAT | Manual: select Custom (local) → Ollama tip + Test connection button visible | N/A | ⬜ pending |
| TBD-by-planner | — | — | PRIV-01 | Visual UAT | Manual: enable Post Process toggle in new location → Sidebar "Post Process" tab appears | N/A | ⬜ pending |
| TBD-by-planner | — | — | PRIV-02 | Static | `test -f docs/PRIVACY.md && grep -cE 'api\.(openai\|z\.ai\|anthropic\|groq\|cerebras)\.com\|openrouter\.ai\|blob\.handy\.computer\|localhost:11434\|apple-intelligence://local\|getdictus/dictus-desktop/releases' docs/PRIVACY.md` (≥ 10) | ❌ W0 | ⬜ pending |
| TBD-by-planner | — | — | PRIV-02 | Static | `grep -c 'docs/PRIVACY.md' README.md` (≥ 1) | ✅ | ⬜ pending |
| TBD-by-planner | — | — | PRIV-02 | Static | `grep -c 'PRIVACY.md' UPSTREAM.md` (≥ 1) | ✅ | ⬜ pending |
| TBD-by-planner | — | — | PRIV-02 | Visual UAT | Manual: About panel → "Privacy & network surface" → browser opens to GitHub blob URL | N/A | ⬜ pending |
| TBD-by-planner | — | — | PRIV-03 | Code review | Grep `src/components/onboarding/Onboarding.tsx` for `post_process` → 0 matches | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/src/settings.rs` — add `#[cfg(test)] mod tests` block with:
  - `default_post_process_provider_id_returns_apple_on_macos_arm64()` — platform-correct id assertion
  - `default_post_process_providers_includes_custom_with_stable_id()` — relabeled provider keeps `id == "custom"`
- [ ] `docs/PRIVACY.md` — new file (does not exist yet)
- [ ] `README.md` — add Privacy section linking to `docs/PRIVACY.md`
- [ ] `UPSTREAM.md` — add one-liner: new outbound HTTP endpoints in upstream sync require PRIVACY.md row update
- [ ] No new test framework install needed — existing lint + translation parity + cargo test + manual UAT covers Phase 8

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Stacked picker visual layout (local above external, neutral section labels) | PRIV-01 | Visual hierarchy is the deliverable; no Vitest/Jest in project | Run `bun run tauri dev` → Settings → Post Process → screenshot, verify "On your device" group renders above "External — data leaves this device" |
| Test connection button feedback (success + failure paths) | PRIV-01 | Requires live Ollama / no-Ollama states; reuses runtime `fetch_post_process_models` | With Ollama running: click → "Connected — N models" Alert. Without: click → "Could not reach <base_url>" Alert |
| Inline Ollama tip with clickable link | PRIV-01 | Tauri opener invocation can't be unit-tested in current setup | Select Custom (local) → click ollama.com link → system browser opens to https://ollama.com |
| Platform-aware default first-run experience | PRIV-01 | Requires fresh settings.json; cargo test covers logic but not full integration | Delete settings.json on macOS ARM64 → launch app → enable Post Process → provider preselected to Apple Intelligence. On non-macOS-ARM64 → preselected to Custom (local) |
| Sidebar tab visibility after toggle relocation | PRIV-01 | UI state is the contract; covered by code review of `Sidebar.tsx:63` reading `post_process_enabled` directly | Toggle on/off in new location → Sidebar "Post Process" tab appears/disappears unchanged |
| About panel link opens correct URL | PRIV-02 | Tauri opener behavior; verify URL string in code is enough | Click "Privacy & network surface" in About panel → browser opens `github.com/getdictus/dictus-desktop/blob/main/docs/PRIVACY.md` |
| PRIVACY.md scannability / honesty | PRIV-02 | Document quality is human-judged | Read PRIVACY.md, confirm: 11 endpoint families present, deferred-cleanup notes on `blob.handy.computer` and OpenRouter Referer, factual tone |
| Onboarding shows no cloud surface | PRIV-03 | Verifies absence — best confirmed visually | Run onboarding flow → confirm only transcription model picker, no post-process / cloud copy |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify OR documented manual UAT step
- [ ] Sampling continuity: lint runs after every commit; no 3 consecutive task commits without lint+format check
- [ ] Wave 0 covers all MISSING references (cargo tests + new docs files)
- [ ] No watch-mode flags in CI commands
- [ ] Feedback latency < 90s (full suite)
- [ ] `nyquist_compliant: true` set in frontmatter after plan-checker pass

**Approval:** pending
