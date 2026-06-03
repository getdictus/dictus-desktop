---
phase: 13
slug: smart-modes-ui-translation-presets-i18n
status: ready
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-03
---

# Phase 13 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust unit) + ESLint + structural translation script + manual E2E |
| **Config file** | none — uses existing project scripts |
| **Quick run command** | `bun run check:translations` |
| **Full suite command** | `bun run lint && bun run check:translations && cd src-tauri && cargo test -p dictus-desktop` |
| **Estimated runtime** | ~90 seconds (cargo test dominates) |

---

## Sampling Rate

- **After every task commit:** Run `bun run check:translations` (frontend/i18n tasks) or `cd src-tauri && cargo test -p dictus-desktop` (backend tasks)
- **After every plan wave:** Run `bun run lint && bun run check:translations && cd src-tauri && cargo test -p dictus-desktop`
- **Before `/gsd:verify-work`:** Full suite must be green + manual E2E checkpoint approved
- **Max feedback latency:** 90 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 13-01-T1 | 13-01 | 1 | TRANS-01/02 | Unit (Rust) | `cd src-tauri && cargo test -p dictus-desktop` | ✅ | ⬜ pending |
| 13-01-T2 | 13-01 | 1 | TRANS-01 | Unit (Rust) | `cd src-tauri && cargo test -p dictus-desktop` | ✅ | ⬜ pending |
| 13-01-T3 | 13-01 | 1 | TRANS-02 | Unit (Rust) | `cd src-tauri && cargo test -p dictus-desktop` | ✅ | ⬜ pending |
| 13-02-T1 | 13-02 | 1 | L10N-01 | Structural (JSON parse) | `node -e "require('./src/i18n/locales/en/translation.json')"` | ✅ | ⬜ pending |
| 13-02-T2 | 13-02 | 1 | L10N-01 | Structural | `bun run check:translations` | ✅ | ⬜ pending |
| 13-03-T1 | 13-03 | 2 | MODE-04/05 | Lint + grep contract | `bun run lint` | ✅ | ⬜ pending |
| 13-03-T2 | 13-03 | 2 | MODE-03/06 | Lint + grep contract | `bun run lint` | ✅ | ⬜ pending |
| 13-03-T3 | 13-03 | 2 | TRANS-01, MODE-06 | Lint + grep contract | `bun run lint` | ✅ | ⬜ pending |
| 13-04-T1 | 13-04 | 3 | MODE-06 | Lint + check:translations | `bun run lint && bun run check:translations` | ✅ | ⬜ pending |
| 13-04-T2 | 13-04 | 3 | MODE-04/05, TRANS-01/02 | Manual E2E (checkpoint) | `bun run tauri dev` (human-verify) | ❌ manual | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all automatable phase requirements:
- Rust unit tests run via `cargo test -p dictus-desktop` (actions_tests module exists; 13-01 extends it).
- `bun run check:translations` (scripts/check-translations.ts) covers L10N-01; baseline verified clean 2026-06-03 (all 19 non-English locales complete).
- `bun run lint` enforces the no-hardcoded-JSX-strings rule and unused-import hygiene, which doubles as a contract check that the new components use the correct i18n keys and command paths.

No new test scaffolding required. The only MISSING coverage is the live-app behaviors (shortcut firing, audio capture, LLM inference output) — these are inherently manual and captured as the 13-04 checkpoint, not as a Wave 0 gap.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| End-to-end Smart Mode flow (record → transcribe → mode fires → embedded LLM → paste) | MODE-04, TRANS-01 | Requires global shortcut + audio + LLM inference in running app | 13-04 Task 2 checkpoint: bind a mode shortcut, record audio, trigger, confirm LLM output is pasted |
| Inline shortcut conflict warning at bind time, bind blocked | MODE-05 | Requires UI interaction with live binding registry | 13-04 Task 2 checkpoint: bind a combo already in use; confirm inline red role=alert text appears and the second mode stays unbound |
| Translation preset offline run | TRANS-01, TRANS-02 | Requires embedded LLM loaded; output is non-deterministic | 13-04 Task 2 checkpoint: choose engine, bind a translation mode, record, confirm translated output offline |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or are explicit manual checkpoints
- [x] Sampling continuity: no 3 consecutive tasks without automated verify (only the final 13-04-T2 is manual)
- [x] Wave 0 covers all MISSING references (none — existing infra suffices)
- [x] No watch-mode flags
- [x] Feedback latency < 90s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** ready
