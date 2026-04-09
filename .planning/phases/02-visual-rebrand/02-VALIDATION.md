---
phase: 2
slug: visual-rebrand
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-08
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property               | Value                                                                                                   |
| ---------------------- | ------------------------------------------------------------------------------------------------------- |
| **Framework**          | ESLint + TypeScript type-check (no unit test framework — visual rebrand validated by grep + lint + UAT) |
| **Config file**        | `eslint.config.js` (ESLint), `tsconfig.json` (TypeScript)                                               |
| **Quick run command**  | `bun run lint && bun run format:check`                                                                  |
| **Full suite command** | `bun run lint && bun run format:check`                                                                  |
| **Estimated runtime**  | ~10 seconds                                                                                             |

---

## Sampling Rate

- **After every task commit:** Run `bun run lint && bun run format:check`
- **After every plan wave:** Run `bun run lint && bun run format:check` + grep for "Handy" and pink hex
- **Before `/gsd:verify-work`:** Full suite must be green + grep clean + visual UAT
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID  | Plan | Wave | Requirement | Test Type   | Automated Command                                                | File Exists | Status     |
| -------- | ---- | ---- | ----------- | ----------- | ---------------------------------------------------------------- | ----------- | ---------- |
| 02-01-01 | 01   | 1    | VISU-01     | manual+ls   | `ls -la src-tauri/icons/icon.icns`                               | —           | ⬜ pending |
| 02-01-02 | 01   | 1    | VISU-02     | build       | `bun run lint`                                                   | —           | ⬜ pending |
| 02-01-03 | 01   | 1    | VISU-03     | grep        | `grep -r "HandyTextLogo\|HandyHand" src/`                        | —           | ⬜ pending |
| 02-02-01 | 02   | 1    | VISU-04     | grep        | `grep -r "Handy" src/i18n/locales/`                              | —           | ⬜ pending |
| 02-02-02 | 02   | 1    | ONBR-01     | grep+build  | `grep "Handy" src/components/onboarding/`                        | —           | ⬜ pending |
| 02-02-03 | 02   | 1    | LANG-01     | grep        | `grep "Handy" src/i18n/locales/en/translation.json`              | —           | ⬜ pending |
| 02-03-01 | 03   | 1    | VISU-05     | grep        | `grep -i "FAA2CA\|da5893\|ffe5ee\|f28cbb\|fad1ed" src/App.css`   | —           | ⬜ pending |
| 02-03-02 | 03   | 1    | VISU-06     | grep        | `grep -ri "FAA2CA\|da5893\|faa2ca" src/components/ src/overlay/` | —           | ⬜ pending |
| 02-04-01 | 04   | 2    | VISU-07     | manual-only | Visual launch test                                               | —           | ⬜ pending |
| 02-04-02 | 04   | 2    | VISU-08     | manual-only | Visual launch test                                               | —           | ⬜ pending |

_Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky_

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. No new test framework or test files needed — visual rebrand is validated by grep one-liners, lint, and UAT.

---

## Manual-Only Verifications

| Behavior                                             | Requirement | Why Manual                       | Test Instructions                                        |
| ---------------------------------------------------- | ----------- | -------------------------------- | -------------------------------------------------------- |
| App icon displays Dictus icon on macOS/Windows/Linux | VISU-01     | Platform-specific icon rendering | Build app, verify icon in dock/taskbar/desktop           |
| Overlay width 300px with 15-20 bars                  | VISU-07     | Visual layout validation         | Start recording, verify overlay dimensions and bar count |
| Sine wave animation during transcribing              | VISU-08     | Animation timing/smoothness      | Start transcription, observe waveform animation behavior |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
