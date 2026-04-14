# Upstream Sync Runbook

Dictus Desktop is forked from [cjpais/Handy](https://github.com/cjpais/Handy). This runbook documents how to pull upstream changes into Dictus without losing our identity (product name, bundle identifier, branded strings, updater config, HTTP headers).

Read this top to bottom before starting a merge. Every command is copy-paste ready.

---

## Fork Point and Current State

| Reference | SHA | Meaning |
|-----------|-----|---------|
| Fork point | `85a8ed77` | Commit where Dictus forked from Handy |
| Merge-base (v0.8.2) | `39e855d` | Last shared ancestor — Handy's v0.8.2 release; seed of `.github/upstream-sha.txt` |
| Last sync | see `.github/upstream-sha.txt` | Updated after each merge lands on main |

The file `.github/upstream-sha.txt` is the single source of truth for "where are we." The weekly detection action (`.github/workflows/upstream-sync.yml`) reads it and opens a tracking issue when upstream has new commits.

**Important:** `upstream-sha.txt` is updated ONLY as part of the merge commit that lands on main. Never update it from the detection workflow — doing so would make the tracking issue disappear before the merge is actually done.

---

## Prerequisites

- `upstream` git remote configured (verify: `git remote -v | grep upstream`). If missing:
  ```bash
  git remote add upstream https://github.com/cjpais/Handy.git
  ```
- `jq` installed (`brew install jq` on macOS; `apt install jq` on Linux)
- Clean working tree (`git status` shows nothing to commit)
- Currently on `main` with latest pulled:
  ```bash
  git checkout main && git pull origin main
  ```

---

## Step-by-Step Merge Process

### 1. Fetch upstream and inspect the delta

```bash
git fetch upstream main --no-tags
STORED=$(cat .github/upstream-sha.txt | tr -d '[:space:]')
echo "Stored SHA: $STORED"
echo "Upstream HEAD: $(git rev-parse upstream/main)"
git log ${STORED}..upstream/main --stat
```

Review each commit. For each one, decide: do we want it? (Answer is usually yes — the filter is "is this a Handy-brand-specific change we must reject?" In practice: no. We accept all functional changes and only reject identity regressions during conflict resolution.)

**Known post-v0.8.2 upstream delta (4 commits):**

| SHA | Title | Risk |
|-----|-------|------|
| `c1697b2` | nix: use symlinkJoin for ALSA_PLUGIN_DIR | NONE — accept upstream |
| `84d88f9` | perf: add reasoning_effort passthrough | MEDIUM — accept reasoning additions, keep Dictus HTTP headers |
| `30b57c4` | fix(issue 522): surface paste errors as UI toast | HIGH — 22 i18n files + App.tsx; accept new keys, keep Dictus strings |
| `fdc8cb7` | correct typo in library name (transcription-rs → transcribe-rs) | LOW — accept upstream |

### 2. Create the sync branch

```bash
git checkout -b upstream/sync-$(date +%Y-%m-%d)
```

Branch naming: `upstream/sync-YYYY-MM-DD` (date-stamped, under `upstream/` namespace).

### 3. Merge upstream/main

```bash
git merge upstream/main --no-ff \
  -m "chore(upstream): merge cjpais/Handy post-v0.8.2 delta (N commits)"
```

Replace `N commits` with the actual count from the delta inspection in Step 1.

Expect conflicts. Do NOT `git merge --abort`. See Section 4 for resolution rules.

### 4. Conflict Resolution (Known Hot Zones)

Work through conflicts in this order — most critical first.

#### 4.1 `src-tauri/tauri.conf.json` — IDENTITY (never regress)

Always keep Dictus values:

- `"productName": "Dictus"` (reject upstream `"Handy"`)
- `"identifier": "com.dictus.desktop"` (reject upstream `"com.cjpais.handy"`)
- `"version": "0.1.0"` (or whatever the current Dictus version is — check `src-tauri/Cargo.toml`)
- `"plugins": { "updater": { "pubkey": "<Dictus pubkey>", "endpoints": ["https://github.com/getdictus/dictus-desktop/releases/latest/download/latest.json"] } }`
- Discard any upstream Windows `signCommand` (we do not code-sign on Windows yet — see INFR-03)

Verify after edit:
```bash
jq -e '.productName == "Dictus"' src-tauri/tauri.conf.json
jq -e '.identifier == "com.dictus.desktop"' src-tauri/tauri.conf.json
jq -e '.plugins.updater.endpoints[0] | test("getdictus/dictus-desktop")' src-tauri/tauri.conf.json
```

> **Do NOT use `git checkout --theirs src-tauri/tauri.conf.json`** — it would accept all upstream identity fields. Resolve this file manually, field by field.

#### 4.2 `src-tauri/src/llm_client.rs` — HTTP HEADERS (Pitfall 1)

Accept all of upstream's `ReasoningConfig` / `reasoning_effort` struct and function additions — those are new features we want.

REJECT upstream's reversion of our HTTP headers. Keep:
- `User-Agent: "Dictus/1.0 (+https://github.com/getdictus/dictus-desktop)"`
- `X-Title: "Dictus"`

The current correct values in `llm_client.rs`:
```rust
headers.insert(
    USER_AGENT,
    HeaderValue::from_static("Dictus/1.0 (+https://github.com/getdictus/dictus-desktop)"),
);
headers.insert("X-Title", HeaderValue::from_static("Dictus"));
```

After resolving:
```bash
grep -q 'Dictus/1.0' src-tauri/src/llm_client.rs && echo "ok User-Agent"
grep -q '"X-Title"' src-tauri/src/llm_client.rs && grep -q '"Dictus"' src-tauri/src/llm_client.rs && echo "ok X-Title"
```

#### 4.3 `src-tauri/src/actions.rs` — handy.log comment (Pitfall 6)

Accept upstream's functional additions (paste-error emit, reasoning_effort plumbing). Then update any comment that says "logged to handy.log on the Rust side" to reference the Dictus log path (or "the app log").

Verify:
```bash
! grep -i 'handy\.log' src-tauri/src/actions.rs && echo "ok no handy.log reference"
! grep 'handy\.computer' src-tauri/src/actions.rs && echo "ok no handy.computer reference"
```

#### 4.4 `src/i18n/locales/en/translation.json` and 21 other locales

Upstream's `30b57c4` adds `pasteFailedTitle` and `pasteFailed` keys in all 22 locales — accept these new keys (text is neutral). The conflict happens on keys we already translated from "Handy" to "Dictus" (permissions description, shortcuts title, etc.).

Rule: for every conflicting key, keep the Dictus value.

After resolving all locales:
```bash
! grep '"Handy"' src/i18n/locales/en/translation.json && echo "ok en locale"
for f in src/i18n/locales/*/translation.json; do
  if grep -q '"Handy"' "$f"; then
    echo "REGRESSION in $f"
  fi
done
echo "locale scan complete"
```

> **Note on acknowledgments:** The `src/i18n/locales/en/translation.json` file may contain a comment or attribution block referencing Handy as the upstream project. This is intentional attribution — it must NOT be removed. The check above looks for `"Handy"` as a value in JSON keys, which is distinct from attribution prose.

#### 4.5 `src/App.tsx`

Upstream adds a `useEffect` that listens for `paste-error` events and shows a toast. This is a clean addition — accept upstream as-is.

#### 4.6 `src/lib/types/events.ts`

Upstream adds a `PasteErrorEvent` interface. This file may already exist in Dictus. Merge both sides: keep existing Dictus exports, add upstream's `PasteErrorEvent` interface. If the file does not exist in Dictus, accept upstream's new file as-is.

#### 4.7 `src-tauri/Cargo.lock` — NEVER manually resolve

After resolving `Cargo.toml` (if it conflicts), regenerate:
```bash
cd src-tauri
cargo generate-lockfile
cd ..
git add src-tauri/Cargo.lock
```

Do not hand-edit `Cargo.lock`. Hand-edits produce subtle version mismatches that can cause build failures or runtime issues.

#### 4.8 `flake.nix`

Accept upstream — Nix config is not Dictus-branded:
```bash
git checkout --theirs flake.nix
git add flake.nix
```

#### 4.9 `README.md`

Accept upstream typo fix (transcription-rs → transcribe-rs). Other Dictus-specific README changes are in different sections and should not conflict. If they do conflict, keep Dictus text:
```bash
# If README.md conflicts and upstream change is clearly a neutral typo fix:
# Manually edit to accept the typo fix while keeping Dictus sections intact.
git add README.md
```

### 5. Finalize the merge

```bash
# Update upstream-sha.txt to the new upstream HEAD
git rev-parse upstream/main > .github/upstream-sha.txt

# Stage all resolved files
git add -A

# Continue the merge (creates the merge commit)
git merge --continue
```

The merge message was set in Step 3. Git will open your editor to confirm — save and close to proceed.

### 6. Post-Merge Verification Checklist

Run the Phase 5 validator:
```bash
bash .planning/phases/05-upstream-sync/scripts/verify-sync.sh
```

Expected output: all check lines show `ok`, ending with `All Phase 5 checks passed.`

If any check fails, fix it before pushing the branch. Common fixes:

| Failing check | Fix |
|---------------|-----|
| `SYNC-05a/b/c FAIL` | Re-resolve `src-tauri/tauri.conf.json` per Section 4.1 |
| `SYNC-05d FAIL` | An i18n locale still has `"Handy"` — check all 22 locales per Section 4.4 |
| `SYNC-05e/f FAIL` | `src-tauri/src/llm_client.rs` headers reverted — re-apply per Section 4.2 |
| `SYNC-05g/h FAIL` | `handy.computer` reference slipped in — remove per Section 4.3 |
| `SYNC-05i FAIL` | `handy.log` comment not updated — fix per Section 4.3 |
| `SYNC-05j FAIL` | `cargo build` broken — read the compiler error; likely a type mismatch from partial conflict resolution |

### 7. Push and open PR

```bash
git push -u origin upstream/sync-$(date +%Y-%m-%d)
gh pr create \
  --base main \
  --title "chore(upstream): merge cjpais/Handy post-v0.8.2 delta" \
  --body "Follows UPSTREAM.md. All SYNC-05 post-merge checks green locally. See .planning/phases/05-upstream-sync/ for context."
```

Review in GitHub UI. After CI passes, merge with **"Create a merge commit"** (preserves upstream history — do not squash or rebase).

---

## Anti-Patterns (Do NOT do these)

| Anti-pattern | Why it's wrong | What to do instead |
|---|---|---|
| `git checkout --theirs src-tauri/tauri.conf.json` | Accepts Handy identity fields (productName, identifier, pubkey, endpoints) | Resolve manually, field by field — keep Dictus values |
| `git cherry-pick c1697b2 84d88f9 30b57c4 fdc8cb7` | Loses the merge relationship; creates separate commits instead of one merge commit preserving upstream history | Use `git merge upstream/main --no-ff` |
| Hand-edit `Cargo.lock` | Machine-generated format; manual edits introduce subtle version conflicts | Run `cargo generate-lockfile` after resolving `Cargo.toml` |
| Update `upstream-sha.txt` from the detection workflow | Causes tracking issue to disappear before the merge is done (Pitfall 2) | Only update `upstream-sha.txt` as part of the merge commit landing on main |
| Skip `verify-sync.sh` before pushing | Identity regressions silently land on main | Always run the validator; fix all failures before pushing |
| Use `git merge --abort` on first conflict | Abandons the entire merge; you lose the branch state | Resolve conflicts file by file — see Section 4 |

---

## Future: Phase 6 Automation

Phase 6 replaces this manual runbook with an AI-driven pipeline: Claude Code agent #1 analyzes upstream commits for relevance, adapts code (identity preservation), and opens a PR. A second Claude Code agent reviews the PR and generates a manual test checklist. Pierre approves, tests, and releases. Until Phase 6 ships, this runbook is the process.

---

## Quick Reference

**Conflict files and their rules:**

| File | Rule | Risk |
|------|------|------|
| `src-tauri/tauri.conf.json` | Keep Dictus: productName, identifier, version, pubkey, endpoints | CRITICAL — identity |
| `src-tauri/src/llm_client.rs` | Accept reasoning additions; keep `Dictus/1.0` User-Agent and `Dictus` X-Title | HIGH — branding |
| `src-tauri/src/actions.rs` | Accept upstream; remove `handy.log` comment reference | MEDIUM — comment cleanup |
| `src/i18n/locales/*/translation.json` | Accept new keys (paste-error); keep Dictus values for conflicting keys | HIGH — 22 files |
| `src/App.tsx` | Accept upstream (paste-error useEffect is clean addition) | LOW |
| `src/lib/types/events.ts` | Merge both sides; keep Dictus exports + add upstream PasteErrorEvent | LOW |
| `src-tauri/Cargo.lock` | Never manually resolve — run `cargo generate-lockfile` | MEDIUM |
| `flake.nix` | `git checkout --theirs` — Nix is not Dictus-branded | NONE |
| `README.md` | Accept upstream typo fix; keep Dictus sections | LOW |

**Key files:**
- `.github/upstream-sha.txt` — source of truth for last synced upstream SHA
- `.github/workflows/upstream-sync.yml` — weekly detection action
- `.planning/phases/05-upstream-sync/scripts/verify-sync.sh` — post-merge identity gate
