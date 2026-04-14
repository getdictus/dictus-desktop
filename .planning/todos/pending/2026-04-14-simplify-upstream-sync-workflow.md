---
created: 2026-04-14
title: Simplify upstream sync workflow (post Sync #1 retrospective)
area: tooling
files:
  - .github/workflows/upstream-sync.yml
  - .github/upstream-sha.txt
  - .planning/phases/05-upstream-sync/scripts/verify-sync.sh
  - UPSTREAM.md
---

## Problem

After executing Sync #1 (Phase 5), a retrospective identified that the current sync infrastructure is over-engineered for the actual cadence/risk. What we built:

- Custom weekly detection workflow (`upstream-sync.yml`)
- Custom SHA-tracking flat file (`.github/upstream-sha.txt`)
- Custom issue creation on drift detection
- Dedicated GSD phase per sync (5, then 5.x, then 6 eventually)
- `verify-sync.sh` in `.planning/` tree

Several of these duplicate features GitHub already provides natively or via mature community actions.

## Solution

Proposed minimalist target (from the retrospective):

```
.github/
├── workflows/
│   ├── upstream-sync.yml        # ~30 lines, uses community action
│   │                            # (aormsby/Fork-Sync-With-Upstream-action
│   │                            # or similar), opens draft PR weekly
│   └── verify-sync.yml          # CI gate: runs verify-sync.sh on any
│                                # PR labeled 'upstream-sync'
├── pull_request_template/
│   └── upstream-sync.md         # checklist: cap-at-SHA? risk-rating?
│                                # verify-sync.sh green?
└── scripts/
    └── verify-sync.sh           # moved out of .planning/ (it's CI infra)
UPSTREAM.md                      # runbook preserved, trimmed
```

### What to keep (high value, irreplaceable)

- `verify-sync.sh` — no native GitHub feature does this
- `UPSTREAM.md` — Dictus-specific risk rating + cap-at-SHA rule (e.g. skipping AWS Bedrock for local-first) is unique to us
- The "cap-at-SHA" pattern when a commit violates local-first policy

### What to simplify / remove (cost > benefit)

- Custom detection workflow → replace with community action (opens draft PR directly)
- `upstream-sha.txt` → `git merge-base upstream/main main` gives equivalent info for free
- Custom issue creation → GitHub's native "N commits behind" banner + auto-draft PR suffices
- GSD phase per sync → syncs are routine chores, not projects. A PR checklist is enough

### Gains

- ~80% of the behavior for ~20% of the code
- `verify-sync.sh` becomes an automatic CI gate on any `upstream-sync`-labeled PR (currently manual)
- No GSD phase setup per sync (~30 min saved each time)
- PR is already open when you arrive — you just review + resolve conflicts

### What we lose (acceptable)

- Granular sha.txt history → replaced by history of merged `upstream-sync`-labeled PRs
- GSD plan-per-sync granularity → replaced by PR description + checklist

## Context

Not urgent — the current system works (Sync #1 succeeded). But the refactor gets cheaper the sooner we do it (each sync executed with the current system is another data point of "we didn't actually need sha.txt"). Likely worth doing before Sync #2 if possible.

Requires a small migration: move `verify-sync.sh` out of `.planning/` (it's production CI infra, not planning artifact).
