---
phase: 3
slug: documentation-and-cleanup
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-09
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property               | Value                                                                                                                                   |
| ---------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| **Framework**          | vitest (frontend) + manual grep verification (docs)                                                                                     |
| **Config file**        | `vite.config.ts`                                                                                                                        |
| **Quick run command**  | `grep -rn "handy\|Handy" README.md BUILD.md CLAUDE.md`                                                                                  |
| **Full suite command** | `bun run lint && grep -rn "handy\.computer\|cjpais/Handy" README.md BUILD.md CLAUDE.md src/components/settings/about/AboutSettings.tsx` |
| **Estimated runtime**  | ~5 seconds                                                                                                                              |

---

## Sampling Rate

- **After every task commit:** Run `grep -rn "handy\|Handy" README.md BUILD.md CLAUDE.md`
- **After every plan wave:** Run `bun run lint && grep -rn "handy\.computer\|cjpais/Handy" README.md BUILD.md CLAUDE.md src/components/settings/about/AboutSettings.tsx`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID  | Plan | Wave | Requirement | Test Type | Automated Command                                                                               | File Exists | Status     |
| -------- | ---- | ---- | ----------- | --------- | ----------------------------------------------------------------------------------------------- | ----------- | ---------- |
| 03-01-01 | 01   | 1    | DOCS-01     | grep      | `grep -c "Dictus Desktop" README.md`                                                            | ✅          | ⬜ pending |
| 03-01-02 | 01   | 1    | DOCS-01     | grep      | `grep -c "fork of Handy" README.md`                                                             | ✅          | ⬜ pending |
| 03-02-01 | 02   | 1    | DOCS-02     | grep      | `grep -c "Handy" CLAUDE.md \| test $(cat) -le 2`                                                | ✅          | ⬜ pending |
| 03-02-02 | 02   | 1    | DOCS-02     | grep      | `grep -c "Handy" BUILD.md \| test $(cat) -eq 0`                                                 | ✅          | ⬜ pending |
| 03-03-01 | 03   | 1    | DOCS-03     | grep      | `grep -c "handy.computer" src/components/settings/about/AboutSettings.tsx \| test $(cat) -eq 0` | ✅          | ⬜ pending |
| 03-03-02 | 03   | 1    | DOCS-03     | lint      | `bun run lint -- src/components/settings/about/AboutSettings.tsx`                               | ✅          | ⬜ pending |

_Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky_

---

## Wave 0 Requirements

_Existing infrastructure covers all phase requirements._

---

## Manual-Only Verifications

| Behavior                      | Requirement | Why Manual                               | Test Instructions                                                                                             |
| ----------------------------- | ----------- | ---------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| About panel visual appearance | DOCS-03     | Layout/design requires visual inspection | Open app → Settings → About; verify Dictus branding, privacy tagline, ecosystem mention, all links functional |
| README readability            | DOCS-01     | Content quality requires human review    | Read README.md end-to-end; verify flow, accuracy, no orphaned references                                      |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
