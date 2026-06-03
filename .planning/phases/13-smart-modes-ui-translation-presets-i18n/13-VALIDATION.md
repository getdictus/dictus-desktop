---
phase: 13
slug: smart-modes-ui-translation-presets-i18n
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-03
---

# Phase 13 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest / cargo test + structural scripts |
| **Config file** | none — uses existing project scripts |
| **Quick run command** | `bun run check:translations` |
| **Full suite command** | `bun run lint && bun run check:translations && cargo test` |
| **Estimated runtime** | ~60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `bun run check:translations`
- **After every plan wave:** Run `bun run lint && bun run check:translations`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| TBD | TBD | TBD | TBD | TBD | TBD | TBD | ⬜ pending |

*Populated by gsd-planner against final PLAN.md task IDs.*
*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

*Determined by gsd-planner. If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| End-to-end Smart Mode flow (record → transcribe → mode fires → embedded LLM → paste) | MODE-04, TRANS-01 | Requires global shortcut + audio + LLM inference in running app | Build app, record audio, trigger a Smart Mode shortcut, confirm LLM output is pasted |
| Inline shortcut conflict warning at bind time | MODE-05 | Requires UI interaction with live binding registry | Bind a shortcut already in use; confirm inline warning appears and no silent registration failure |
| Translation preset offline run | TRANS-01, TRANS-02 | Requires embedded TranslateGemma model loaded | Disconnect network, run a translation preset, confirm output produced |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
