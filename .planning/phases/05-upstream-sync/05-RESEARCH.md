# Phase 5: Upstream Sync - Research

**Researched:** 2026-04-14
**Domain:** GitHub Actions (upstream detection + idempotency), git merge strategy (fork rebrand), identity integrity verification
**Confidence:** HIGH — based on direct codebase and upstream delta analysis

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Detection action design:**
- Workflow file: `.github/workflows/upstream-sync.yml`
- Triggers: Weekly cron (Monday 08:00 UTC) + manual `workflow_dispatch`
- Issue content: Commit list only — title: "Upstream: N new commits on cjpais/Handy", body: commit hashes + one-line messages + link to upstream compare view. Minimal, scannable.
- Idempotency: Store last-seen upstream SHA in `.github/upstream-sha.txt` committed to repo. Action compares current upstream HEAD against stored SHA before creating issue. No issue if unchanged.
- Label: `upstream-sync` label must be created in repo before first action run

**Merge strategy:**
- Method: Single `git merge upstream/main` on a dedicated branch — one merge commit preserving upstream history
- Branch naming: `upstream/sync-YYYY-MM-DD` (date-stamped, under upstream/ namespace)
- Landing: PR to main — review checkpoint, CI runs, clean merge record
- No develop branch — not needed for solo project. PR to main is the gate.
- Post-merge checks (SYNC-05): grep `productName`, `identifier`, and i18n strings for Dictus (not Handy), plus `cargo build` succeeds. Matches Phase 4 identity verification pattern.
- First merge executor: Claude Code executes the merge in a plan task (creates branch, merges, resolves conflicts, verifies identity, opens PR). Pierre reviews and merges.

**UPSTREAM.md structure:**
- Location: Root of repo (`UPSTREAM.md`) alongside README.md — high visibility
- Detail level: Copy-paste git commands — future-Pierre can follow in 6 months without re-researching (same philosophy as `docs/RUNBOOK-updater-signing.md`)
- Content sections:
  - Fork point info (`85a8ed77`, merge-base `39e855d`)
  - Step-by-step merge process (fetch, branch, merge, check, PR)
  - Conflict resolution guide for known hot zones (tauri.conf.json, i18n files, Cargo.toml — always keep Dictus values)
  - Post-merge checklist (identity grep, cargo build, handy.computer scan)
- SHA tracking: References `.github/upstream-sha.txt` as the source of truth for "where are we"

### Claude's Discretion
- Exact cron expression for Monday 08:00 UTC
- Issue body template formatting (markdown)
- UPSTREAM.md prose and section ordering
- Whether post-merge identity checks are a script or inline commands in UPSTREAM.md
- Conflict resolution order if multiple files conflict simultaneously
- Whether to regenerate Cargo.lock as part of merge or as separate step

### Deferred Ideas (OUT OF SCOPE)
- Phase 6: AI-driven upstream sync pipeline — Claude Code scheduled agent analyzes upstream commits for relevance, adapts code (identity preservation), opens PR. Second agent reviews PR and generates manual test checklist. Pierre approves, tests, releases. Full automation of the merge workflow.
- SYNC-A1: AI-assisted cherry-pick triage — already in REQUIREMENTS.md as future requirement. Subsumed by Phase 6 vision.
- Semi-automatic PR creation in the action — action could auto-create branch + merge + PR on detection. Intermediate step between manual and full AI pipeline. Could be Phase 6 wave 1.
- Develop branch — not needed now, revisit if team grows or release cadence demands a staging buffer.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| SYNC-01 | `.github/workflows/upstream-sync.yml` detects new commits on `cjpais/Handy` main and creates a GitHub issue with commit summary | SHA-based detection pattern, `actions/github-script@v7` issue creation, `upstream-sha.txt` idempotency design fully researched |
| SYNC-02 | Weekly action is idempotent (no duplicate issues for same upstream state) | Idempotency via committed `.github/upstream-sha.txt` is the locked approach; no marketplace action needed |
| SYNC-03 | `UPSTREAM.md` documents fork point (`85a8ed77`), merge-base (`39e855d`), and step-by-step merge process | All conflict zones identified from live `git diff HEAD upstream/main`; exact commands documented |
| SYNC-04 | First upstream merge of 4 post-v0.8.2 commits completed on dedicated branch | Delta is confirmed: 4 commits, files known, conflict zones known, strategy clear |
| SYNC-05 | Post-merge checklist verified (identity fields, version, i18n scan, handy.computer scan, cargo build) | Phase 4 `validate.sh` pattern reusable; all grep assertions derived from actual upstream diff |
</phase_requirements>

---

## Summary

Phase 5 delivers three concrete deliverables: a GitHub Actions workflow that detects upstream drift weekly, a human-readable `UPSTREAM.md` runbook, and an executed first merge of the 4 post-v0.8.2 upstream commits. None of these require new dependencies or architectural changes.

The upstream delta has been directly inspected. The 4 commits (`c1697b2`, `84d88f9`, `30b57c4`, `fdc8cb7`) touch 4 functional areas: Nix ALSA config, reasoning_effort passthrough in `llm_client.rs`/`actions.rs`, paste-error toast in `App.tsx`/`actions.rs`/all i18n locales, and a README typo fix. The most conflict-prone commit is `30b57c4` (paste-error feature) because it touches `App.tsx` and 22 i18n locale files — all files Dictus has previously modified for the rebrand. The `tauri.conf.json` diff confirms Dictus identity is safe (upstream has their identity, Dictus already has the correct identity; merge resolution is straightforward).

The detection action design is fully locked by CONTEXT.md. The idempotency mechanism (committed `.github/upstream-sha.txt`, compare before creating issue) avoids GitHub Issues search API complexity and works reliably in CI. The existing `release.yml` + `actions/github-script@v7` pattern is directly reusable.

**Primary recommendation:** Implement in three sequential tasks — (1) workflow + `upstream-sha.txt` + label, (2) UPSTREAM.md + conflict resolution guide, (3) execute first merge on branch + post-merge verification script + open PR.

---

## Standard Stack

### Core

| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| GitHub Actions | N/A | Weekly cron detection | Already 9 workflows in repo; no new platform |
| `actions/checkout@v4` | v4 | Checkout with full git history | Used in all existing workflows |
| `actions/github-script@v7` | v7 | Create GitHub issues via REST API | Used in `release.yml` for release creation |
| `git` (bash) | system | Fetch upstream, compute delta | No marketplace action needed; native git sufficient |

### Supporting

| Tool | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| `upstream-sha.txt` (flat file) | N/A | Idempotency state | Committed to `.github/`, read by action to detect change |
| `upstream-sync` label | N/A | Issue categorization | Must be created manually in repo settings before first run |
| Phase 4 `validate.sh` pattern | N/A | Post-merge grep assertions | Reuse check/FAIL/pass pattern for SYNC-05 script |

### No Marketplace Actions Needed

The locked design uses native git + `actions/github-script@v7`. Marketplace "upstream sync" actions (e.g., `aormsby/Fork-Sync-With-Upstream-action`) do auto-merging — explicitly out of scope for this phase.

**Installation:** No new packages. All tooling is already present.

---

## Architecture Patterns

### Recommended Project Structure

```
.github/
├── upstream-sha.txt          # NEW — tracks last-seen upstream HEAD SHA
└── workflows/
    └── upstream-sync.yml     # NEW — weekly cron detection action
UPSTREAM.md                   # NEW — root of repo, merge runbook
scripts/
└── verify-sync.sh            # NEW (or inline) — post-merge SYNC-05 assertions
```

### Pattern 1: SHA-Based Idempotent Detection

**What:** Read `.github/upstream-sha.txt` (committed to repo), fetch `upstream/main`, compare current upstream HEAD to stored SHA. Create issue only when SHA differs. After issue creation, update the file (commit to repo or use as output artifact depending on implementation).

**When to use:** Always — this is the locked idempotency mechanism.

**Key design point:** The stored SHA should be updated **only after Pierre merges the sync branch to main**, not after issue creation. This ensures the issue persists until the sync is actually done, not just detected.

**Workflow skeleton:**
```yaml
# Source: locked in CONTEXT.md, pattern derived from release.yml
name: Upstream Sync Check

on:
  schedule:
    - cron: '0 8 * * 1'   # Monday 08:00 UTC
  workflow_dispatch:

permissions:
  contents: read
  issues: write

jobs:
  check-upstream:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Fetch upstream
        run: |
          git remote add upstream https://github.com/cjpais/Handy.git
          git fetch upstream main

      - name: Compare upstream SHA
        id: compare
        run: |
          STORED_SHA=$(cat .github/upstream-sha.txt 2>/dev/null || echo "")
          CURRENT_SHA=$(git rev-parse upstream/main)
          echo "stored=${STORED_SHA}" >> $GITHUB_OUTPUT
          echo "current=${CURRENT_SHA}" >> $GITHUB_OUTPUT
          if [ "$STORED_SHA" = "$CURRENT_SHA" ]; then
            echo "changed=false" >> $GITHUB_OUTPUT
          else
            echo "changed=true" >> $GITHUB_OUTPUT
          fi

      - name: Build commit list
        id: commits
        if: steps.compare.outputs.changed == 'true'
        run: |
          STORED="${{ steps.compare.outputs.stored }}"
          CURRENT="${{ steps.compare.outputs.current }}"
          if [ -z "$STORED" ]; then
            # First run — use known fork point
            STORED="85a8ed77"
          fi
          COUNT=$(git log ${STORED}..upstream/main --oneline | wc -l | tr -d ' ')
          COMMIT_LIST=$(git log ${STORED}..upstream/main --oneline --no-walk=unsorted | head -20)
          echo "count=${COUNT}" >> $GITHUB_OUTPUT
          {
            echo "commit_list<<EOF"
            echo "$COMMIT_LIST"
            echo "EOF"
          } >> $GITHUB_OUTPUT

      - name: Create tracking issue
        if: steps.compare.outputs.changed == 'true'
        uses: actions/github-script@v7
        with:
          script: |
            const count = '${{ steps.commits.outputs.count }}';
            const commitList = `${{ steps.commits.outputs.commit_list }}`;
            const storedSha = '${{ steps.compare.outputs.stored }}'.substring(0, 8);
            const currentSha = '${{ steps.compare.outputs.current }}'.substring(0, 8);
            const compareUrl = `https://github.com/cjpais/Handy/compare/${storedSha}...${currentSha}`;

            await github.rest.issues.create({
              owner: context.repo.owner,
              repo: context.repo.repo,
              title: `Upstream: ${count} new commits on cjpais/Handy`,
              labels: ['upstream-sync'],
              body: [
                `## Upstream Sync Available`,
                ``,
                `**${count} new commit(s)** on [\`cjpais/Handy\`](https://github.com/cjpais/Handy) since last sync (\`${storedSha}\`).`,
                ``,
                `**Compare:** ${compareUrl}`,
                ``,
                `### Commits`,
                `\`\`\``,
                commitList,
                `\`\`\``,
                ``,
                `Follow \`UPSTREAM.md\` to merge.`,
              ].join('\n')
            });
```

### Pattern 2: Merge Resolution Rules (Known Conflict Zones)

Based on direct `git diff HEAD upstream/main` analysis, these files will conflict during `git merge upstream/main`:

| File | Conflict Type | Resolution Rule |
|------|--------------|-----------------|
| `src-tauri/tauri.conf.json` | Identity fields (productName, identifier, version), pubkey, endpoints, Windows signCommand | Always keep Dictus values: `productName: "Dictus"`, `identifier: "com.dictus.desktop"`, `version: "0.1.0"` (or current), Dictus pubkey, Dictus endpoints. Discard upstream Windows `signCommand`. |
| `src/i18n/locales/en/translation.json` | Upstream adds `pasteFailedTitle`/`pasteFailed` keys with neutral text; also reverts some Dictus strings to Handy | Accept new keys verbatim (text is generic). Revert any `"Handy"` values back to `"Dictus"`. |
| `src/i18n/locales/*/translation.json` (21 locales) | Same paste-error keys added in all locales | Accept all new keys (text is neutral). Check for Handy regressions. |
| `src/App.tsx` | Upstream adds paste-error `useEffect` | Accept upstream addition. It is clean, non-conflicting with Dictus changes, and the event name `"paste-error"` is neutral. |
| `src-tauri/src/actions.rs` | Upstream adds reasoning_effort logic + paste-error emit | Accept upstream additions. No Dictus-specific changes to this file. Verify no `handy.log` references slipped in (comment says "logged to handy.log"). |
| `src-tauri/src/llm_client.rs` | Upstream adds `ReasoningConfig`, changes function signatures; also reverts `User-Agent` and `X-Title` headers to Handy | Accept upstream API additions. **Reject** header regressions: keep `User-Agent: "Dictus/1.0 (+https://github.com/getdictus/dictus-desktop)"` and `X-Title: "Dictus"`. |
| `src-tauri/Cargo.lock` | Both sides modified; never manually edit | After resolving `Cargo.toml`, run `cargo generate-lockfile` to regenerate. Do not manually resolve. |
| `flake.nix` | Upstream changes ALSA_PLUGIN_DIR to symlinkJoin | Accept upstream change — Nix config is not Dictus-branded. |
| `README.md` | Upstream fixes transcribe-rs typo | Accept upstream change — neutral text edit. |

### Pattern 3: Post-Merge Verification (SYNC-05)

Reuse the `check()`/`FAIL` pattern from `validate.sh`. Assertions to run after merge:

```bash
# Identity fields
grep -q '"productName": "Dictus"' src-tauri/tauri.conf.json
grep -q '"identifier": "com.dictus.desktop"' src-tauri/tauri.conf.json
grep -q '"version": "0.1.0"' src-tauri/tauri.conf.json  # or current Dictus version

# Updater endpoint (Phase 4 assertion)
jq -e '.plugins.updater.endpoints[0] | test("getdictus/dictus-desktop")' src-tauri/tauri.conf.json

# No Handy regressions in i18n
! grep -r '"Handy"' src/i18n/locales/en/translation.json

# No Handy regressions in llm_client User-Agent/X-Title
grep -q 'Dictus/1.0' src-tauri/src/llm_client.rs
grep -q '"Dictus"' src-tauri/src/llm_client.rs

# No handy.computer references in new code
! grep -r 'handy\.computer' src-tauri/src/actions.rs
! grep -r 'handy\.computer' src-tauri/src/llm_client.rs

# Build succeeds
cargo build --manifest-path src-tauri/Cargo.toml
```

### Anti-Patterns to Avoid

- **Do not use `git checkout --theirs src-tauri/tauri.conf.json`** — this would accept all upstream identity fields.
- **Do not resolve Cargo.lock manually** — always regenerate with `cargo generate-lockfile`.
- **Do not create a GitHub issue on every run** — idempotency via `upstream-sha.txt` prevents noise.
- **Do not update `upstream-sha.txt` on issue creation** — only update when the sync branch is merged to main, otherwise the issue disappears before the work is done.
- **Do not use `git cherry-pick` for these 4 commits** — the locked decision is `git merge upstream/main` (one merge commit, preserving upstream history). Cherry-pick creates separate commits and loses the merge relationship.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Duplicate issue detection | Title-based issue search API | SHA comparison with `upstream-sha.txt` | File comparison is O(1), reliable, no API rate-limit risk, survives issue renames |
| GitHub issue creation | curl + jq GitHub API calls | `actions/github-script@v7` | Already used in `release.yml`; handles auth automatically via `GITHUB_TOKEN` |
| Cargo.lock conflict resolution | Manual merge conflict edit | `cargo generate-lockfile` | Cargo.lock format is machine-generated; manual edits introduce subtle version conflicts |
| Upstream diff computation | Complex git scripting | `git log STORED..upstream/main --oneline` | Standard git; no marketplace action needed |

**Key insight:** The entire detection mechanism fits in ~60 lines of YAML. No marketplace "sync" actions are warranted — they do auto-merging, which is out of scope.

---

## Common Pitfalls

### Pitfall 1: Handy Identity Regression in llm_client.rs

**What goes wrong:** The `84d88f9` commit (reasoning_effort) also reverts `User-Agent` and `X-Title` HTTP headers in `llm_client.rs` back to Handy values. A merge that accepts upstream's reasoning_effort additions (correct) will also accept the header regressions (incorrect) unless the conflict is resolved field-by-field.

**Why it happens:** Both sides modified `llm_client.rs` — Dictus for the rebrand headers, upstream for reasoning_effort. A merge conflict appears. Accepting `--theirs` to get the new feature also takes the Handy headers.

**How to avoid:** Resolve `llm_client.rs` manually. Accept all reasoning_effort struct/function additions. Manually set `User-Agent` back to `"Dictus/1.0 (+https://github.com/getdictus/dictus-desktop)"` and `X-Title` back to `"Dictus"`.

**Warning signs:** `grep "Handy/1.0\|X-Title.*Handy" src-tauri/src/llm_client.rs` returns results after merge.

### Pitfall 2: `upstream-sha.txt` Updated Too Early

**What goes wrong:** If the action updates `upstream-sha.txt` to the new SHA immediately after creating the issue (on the same run), the next weekly run sees "no change" — but the merge was never done. The issue goes stale without a re-notification.

**Why it happens:** Conflating "detected" with "synced."

**How to avoid:** `upstream-sha.txt` is updated **only** when the upstream sync branch is merged into main (done by Claude Code or Pierre as part of the branch merge commit). The detection action is read-only — it never commits.

**Warning signs:** `upstream-sha.txt` contains the current upstream SHA but no `upstream/sync-*` branch or PR exists.

### Pitfall 3: `upstream-sync` Label Not Created Before First Run

**What goes wrong:** `actions/github-script@v7` throws a 422 error on issue creation if the label `upstream-sync` does not exist in the repo. The action fails with an obscure REST API error.

**Why it happens:** Labels must be pre-created; the API does not create missing labels automatically.

**How to avoid:** Create the `upstream-sync` label in GitHub repo settings before merging the workflow. This is a manual pre-flight step — Wave 0 task.

**Warning signs:** Action log shows `HttpError: Label does not exist` or `422 Unprocessable Entity`.

### Pitfall 4: i18n Handy Values Slipping Through

**What goes wrong:** The `30b57c4` paste-error commit touches all 22 locale files. The new `pasteFailedTitle`/`pasteFailed` keys have neutral text ("Failed to Paste Text" / "Text could not be pasted..."). However, the same commit shows that upstream's `en/translation.json` contains other keys with "Handy" values (permissions description, shortcuts title, etc.) — these exist due to the Dictus rebrand already in place. Git merge will produce conflicts on those keys.

**Why it happens:** Dictus already changed "Handy" → "Dictus" in `en/translation.json`. Upstream modified the same keys. Both sides changed the same lines: guaranteed conflict.

**How to avoid:** For every conflicting i18n key, keep Dictus's value. Accept upstream's new keys (paste-error). Run `grep -r '"Handy"' src/i18n/locales/en/translation.json` after resolution — must return empty (or only attribution/acknowledgment text).

**Warning signs:** `grep '"Handy"' src/i18n/locales/en/translation.json` returns non-attribution hits.

### Pitfall 5: GitHub Actions `permissions` Missing for Issue Creation

**What goes wrong:** The default `GITHUB_TOKEN` permissions in a repository may not include `issues: write`. The action silently fails or throws a permissions error when calling `github.rest.issues.create`.

**Why it happens:** GitHub's default token permissions can be read-only depending on repo settings.

**How to avoid:** Add explicit `permissions: issues: write` (and `contents: read`) to the job block. The existing `release.yml` uses `contents: write` as a pattern — same explicit approach.

**Warning signs:** Action log shows `Resource not accessible by integration` or `403 Forbidden` on issue creation step.

### Pitfall 6: `actions.rs` Comment References `handy.log`

**What goes wrong:** The upstream `30b57c4` commit adds a comment in `actions.rs` saying "logged to handy.log on the Rust side." This is a code comment, not functional code, but it's a Handy reference that would persist in the codebase.

**Why it happens:** Upstream developers refer to their own log file by their product name.

**How to avoid:** During conflict resolution of `actions.rs`, update the comment to reference `dictus.log` (or the generic "the app log").

---

## Code Examples

### Detection Action — SHA Idempotency Logic

```bash
# Source: derived from locked CONTEXT.md design + codebase analysis
# Read stored SHA (file created by this workflow or by merge task)
STORED_SHA=$(cat .github/upstream-sha.txt 2>/dev/null || echo "")

# Fetch current upstream HEAD
git remote add upstream https://github.com/cjpais/Handy.git
git fetch upstream main --no-tags
CURRENT_SHA=$(git rev-parse upstream/main)

# Compare — only proceed if changed
if [ "$STORED_SHA" = "$CURRENT_SHA" ]; then
  echo "No upstream changes since last sync (${CURRENT_SHA:0:8}). Exiting."
  exit 0
fi
```

### Merge Execution — Conflict-Aware Sequence

```bash
# Source: locked CONTEXT.md, derived from git diff analysis

# 1. Create branch
git checkout -b upstream/sync-$(date +%Y-%m-%d)

# 2. Merge — expect conflicts, do not abort
git merge upstream/main --no-ff -m "chore(upstream): merge cjpais/Handy post-v0.8.2 delta (4 commits)"
# If conflicts: git status to see which files need resolution

# 3. Resolve conflicts (see UPSTREAM.md rules):
# - tauri.conf.json: always keep Dictus identity
# - llm_client.rs: accept reasoning additions, keep Dictus headers
# - App.tsx: accept paste-error useEffect
# - i18n locales: accept new keys, revert Handy values to Dictus

# 4. Regenerate Cargo.lock (never manually resolve)
cargo generate-lockfile --manifest-path src-tauri/Cargo.toml

# 5. Stage all resolved files
git add -A
git merge --continue

# 6. Update upstream-sha.txt to reflect the merged state
echo "$(git rev-parse upstream/main)" > .github/upstream-sha.txt
git add .github/upstream-sha.txt
git commit --amend --no-edit  # or separate commit
```

### Post-Merge Identity Verification Script

```bash
# Source: Phase 4 validate.sh pattern (reuse check()/FAIL design)
FAIL=0
check() {
  local label="$1"; shift
  if eval "$@" >/dev/null 2>&1; then
    printf "  ok  %s\n" "$label"
  else
    printf "  FAIL %s\n" "$label"
    FAIL=1
  fi
}

check "SYNC-05a productName is Dictus" \
  "grep -q '\"productName\": \"Dictus\"' src-tauri/tauri.conf.json"

check "SYNC-05b identifier is com.dictus.desktop" \
  "grep -q '\"identifier\": \"com.dictus.desktop\"' src-tauri/tauri.conf.json"

check "SYNC-05c no Handy in en i18n" \
  "! grep '\"Handy\"' src/i18n/locales/en/translation.json"

check "SYNC-05d llm_client User-Agent is Dictus" \
  "grep -q 'Dictus/1.0' src-tauri/src/llm_client.rs"

check "SYNC-05e no handy.computer in actions.rs" \
  "! grep 'handy\.computer' src-tauri/src/actions.rs"

check "SYNC-05f updater endpoint is getdictus" \
  "jq -e '.plugins.updater.endpoints[0] | test(\"getdictus/dictus-desktop\")' src-tauri/tauri.conf.json"

check "SYNC-05g cargo build succeeds" \
  "cargo build --manifest-path src-tauri/Cargo.toml 2>&1"

[ $FAIL -eq 0 ] && echo "All SYNC-05 checks passed." || exit 1
```

---

## Upstream Delta: Confirmed 4 Commits

Directly verified with `git log 39e855d..upstream/main --stat`:

| SHA | Title | Files Changed | Conflict Risk | Resolution |
|-----|-------|--------------|--------------|------------|
| `c1697b2` | nix: use symlinkJoin for ALSA_PLUGIN_DIR | `flake.nix` | NONE — Dictus has no Nix changes | Accept upstream |
| `84d88f9` | perf: add reasoning_effort passthrough | `src-tauri/src/actions.rs`, `src-tauri/src/llm_client.rs` | MEDIUM — `llm_client.rs` has Dictus header changes | Accept reasoning additions; keep Dictus headers |
| `30b57c4` | fix(issue 522): surface paste errors as UI toast | `src-tauri/src/actions.rs`, `src/App.tsx`, `src/lib/types/events.ts` (new?), `src/i18n/locales/*/translation.json` (22 files) | HIGH — 22 i18n files, App.tsx, actions.rs all touched | Accept new code/keys; revert Handy strings to Dictus |
| `fdc8cb7` | correct typo in library name (transcription-rs → transcribe-rs) | `README.md` | LOW — README.md Dictus changes are in different sections | Accept upstream, verify no Handy branding slipped in |

**Total files affected by merge:** ~27 files (22 locale files + actions.rs + llm_client.rs + App.tsx + README.md + flake.nix + Cargo.lock).

---

## State of the Art

| Old Approach | Current Approach | Notes |
|--------------|-----------------|-------|
| Marketplace "upstream sync" action | Native git + github-script | Marketplace actions auto-merge; we want notification-only |
| Title-based issue dedup | SHA comparison with committed file | More reliable; O(1); no API rate limits |
| Manual conflict resolution each time | UPSTREAM.md with explicit rules per hot zone | Codifies knowledge so future syncs are mechanical |

---

## Open Questions

1. **`src/lib/types/events.ts` — does it exist in Dictus?**
   - What we know: The `30b57c4` commit adds `PasteErrorEvent` interface. The diff stat shows this file but its Dictus state is unknown.
   - What's unclear: Whether Dictus already has this file and whether it conflicts.
   - Recommendation: Check `src/lib/types/events.ts` existence before merge. If it exists, inspect for conflicts. If not, accept upstream's new file.

2. **`upstream-sha.txt` initial value — fork point or merge-base?**
   - What we know: Fork point is `85a8ed77`, merge-base is `39e855d` (v0.8.2 release). After Phase 5 merge, `upstream-sha.txt` should contain the SHA of `upstream/main` at time of merge (currently `fdc8cb7...`).
   - What's unclear: What value to seed the file with before the first action run.
   - Recommendation: Seed with `39e855d` (merge-base / v0.8.2) so the first detection run reports the 4 known commits as "new." After the Phase 5 merge is complete, update to current `upstream/main` HEAD SHA.

3. **Cargo.lock regeneration timing**
   - What we know: After resolving `Cargo.toml` conflicts, `cargo generate-lockfile` produces the correct lock file.
   - What's unclear: Whether running this as a separate commit after `--continue` is cleaner than as part of the merge commit.
   - Recommendation: Regenerate before `git merge --continue`; include in the merge commit. Cleaner history.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | bash scripts (project uses shell-based validators, no test framework for infra) |
| Config file | none — see Wave 0 |
| Quick run command | `bash .planning/phases/05-upstream-sync/scripts/verify-sync.sh` |
| Full suite command | same + `cargo build --manifest-path src-tauri/Cargo.toml` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SYNC-01 | upstream-sync.yml exists and is valid YAML | smoke | `cat .github/workflows/upstream-sync.yml | python3 -c "import sys,yaml; yaml.safe_load(sys.stdin)"` | ❌ Wave 0 |
| SYNC-02 | Idempotency: action does not create duplicate when SHA unchanged | manual | Trigger `workflow_dispatch` twice with same upstream state; verify only one issue | manual-only |
| SYNC-03 | UPSTREAM.md exists and contains fork point + merge-base SHAs | smoke | `grep -q '85a8ed77' UPSTREAM.md && grep -q '39e855d' UPSTREAM.md` | ❌ Wave 0 |
| SYNC-04 | Merge branch exists with 4 upstream commits | smoke | `git log upstream/sync-* --oneline \| grep -E "c1697b2\|84d88f9\|30b57c4\|fdc8cb7"` | ❌ Wave 0 |
| SYNC-05 | Post-merge identity checks pass | unit | `bash .planning/phases/05-upstream-sync/scripts/verify-sync.sh` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `grep -q '"productName": "Dictus"' src-tauri/tauri.conf.json && grep -q '"identifier": "com.dictus.desktop"' src-tauri/tauri.conf.json`
- **Per wave merge:** `bash .planning/phases/05-upstream-sync/scripts/verify-sync.sh`
- **Phase gate:** All SYNC-05 assertions green + PR opened before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `.planning/phases/05-upstream-sync/scripts/verify-sync.sh` — covers SYNC-05 identity checks
- [ ] `.github/upstream-sha.txt` — initial value `39e855d` (merge-base / v0.8.2)
- [ ] `upstream-sync` label created in GitHub repo settings (manual pre-flight)

---

## Sources

### Primary (HIGH confidence)

- Direct `git log 39e855d..upstream/main --stat` — 4 upstream commits confirmed, files and conflict zones identified
- Direct `git diff HEAD upstream/main -- src-tauri/tauri.conf.json` — identity field conflict confirmed, resolution clear
- Direct `git diff HEAD upstream/main -- src-tauri/src/llm_client.rs` — header regression confirmed, resolution documented
- Direct `git diff HEAD upstream/main -- src/i18n/locales/en/translation.json` — 22 locale conflict pattern confirmed
- `.planning/phases/05-upstream-sync/05-CONTEXT.md` — locked decisions (all implementation choices)
- `.github/workflows/release.yml` — `actions/github-script@v7` issue creation pattern
- `.planning/phases/04-updater-infrastructure/scripts/validate.sh` — check()/FAIL pattern for SYNC-05

### Secondary (MEDIUM confidence)

- `.planning/research/PITFALLS.md` (2026-04-10) — B1-B7 pitfalls mapped to actual diff findings
- `.planning/research/ARCHITECTURE.md` (2026-04-10) — upstream sync workflow design, confirmed against current state

### Tertiary (LOW confidence)

- None identified — all findings verified against live codebase and direct git analysis.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies, all tooling verified against existing workflows
- Architecture: HIGH — locked by CONTEXT.md + verified against live upstream delta
- Pitfalls: HIGH — derived from actual `git diff HEAD upstream/main` output, not hypothetical

**Research date:** 2026-04-14
**Valid until:** 2026-05-14 (stable domain; would only change if upstream adds more commits)
