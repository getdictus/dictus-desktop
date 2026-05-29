# Phase 10: Prerequisite Gate - Context

**Gathered:** 2026-05-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Clear all v1.3 build blockers and leave the codebase in a known-clean state **before any feature code is written**. Three non-user-facing work items, executed in this fixed order:

1. **PREP-03** — Upstream Sync #2 (catch up the 17-commit delta, drop AWS Bedrock, transition fork policy)
2. **PREP-02** — TECH-04 refactor (`llm_client.rs` 8-arg fn → params struct, drop `#[allow]`, clippy clean)
3. **PREP-01** — ggml feasibility spike (`llama-cpp-2` + `transcribe-rs` coexistence on all 7 CI platforms)

No feature code, no LLM runtime, no Smart Modes. This phase only removes blockers. New capabilities belong to Phases 11–13.

</domain>

<decisions>
## Implementation Decisions

### Execution order (carried from STATE.md, reaffirmed)
- PREP-03 → PREP-02 → PREP-01. Sync #2 may touch `llm_client.rs` / `managers/`; refactor on the up-to-date base; spike last on the clean tree.

### PREP-03 — Upstream Sync #2 method
- **Merge full, then revert Bedrock.** `git merge upstream/main` (brings all 17 commits, preserves merge history and lands the correct upstream HEAD in `upstream-sha.txt`), then `git revert aee682f` in the same sync branch to neutralize the AWS Bedrock provider.
- Bedrock (`aee682f`, PR #1288) is **excluded** — sits mid-delta (commit 12 of 17), so a clean pre-Bedrock cap is impossible without losing the newer v0.8.3 release + nix/overlay fixes. Revert is the honest way to keep everything else.
- Follow the existing `UPSTREAM.md` merge flow: dedicated `upstream/sync-YYYY-MM-DD` branch → PR to main → CI gate → Pierre merges. Per-commit triage (SYNC-A1) documented in the PR.
- Identity integrity preserved: `verify-sync.sh` (15 assertions) must exit 0; honor the 9 documented hot-zone conflict guides, especially `llm_client.rs` headers (§4.2) and `Cargo.lock` (never hand-resolve, §4.7).

### PREP-03 — Fork policy going forward
- After this catch-up merge, future syncs transition to **selective cherry-pick** (no more bulk merges by default).
- Record the policy + rationale as a **new "Fork Policy: Selective Cherry-Pick" section in `UPSTREAM.md`**, including an **Exclusion Log** table. Bedrock `aee682f` is the first exclusion entry (reason: local-first, cloud provider not owned by Dictus).
- This is the last planned bulk catch-up merge; subsequent upstream deltas are triaged commit-by-commit.

### PREP-02 — TECH-04 refactor shape
- `send_chat_completion_with_schema` (8 positional args) takes a **plain public params struct** instead. Claude names it (candidate: `ChatCompletionParams` — NOT `ChatCompletionRequest`, which is already the private wire-body struct at `llm_client.rs:36`).
- Struct should derive `Default` so callers use struct-literal + `..Default::default()` (matches existing `ReasoningConfig` Default-derive style). No builder pattern — only 2 call sites, builder isn't justified.
- Update both callers: `actions.rs:207` (`send_chat_completion_with_schema`) and `actions.rs:265` (`send_chat_completion`). `send_chat_completion` may stay a thin convenience wrapper or be folded in — Claude's discretion.
- Remove `#[allow(clippy::too_many_arguments)]` (line 137). `cargo clippy --all-targets -- -D warnings` must exit 0 on all platforms.

### PREP-01 — ggml spike scope & contingency
- **Commit to `llama-cpp-2` first.** Spike adds `llama-cpp-2`, runs `cargo tree | grep ggml` against the live Cargo.lock with both crates present, and attempts to compile/link on all 7 CI platforms.
- If a symbol conflict appears, resolve via `[patch.crates-io]` (proven pattern — Cargo.toml already patches tauri forks the same way) or equivalent single-shared-ggml approach.
- **Contingency (Plan B):** Only if `llama-cpp-2` cannot be made to coexist even with patching → fall back to **candle or mistral.rs** (pure-Rust, no ggml static-link collision with `whisper-cpp`). This keeps the in-process promise intact. Do NOT bake off alternatives upfront — pay for the fallback evaluation only if the primary path fails.
- Sidecar / `llama-server` is explicitly out-of-scope and is NOT a fallback. Per-platform engine mixing is rejected (contradicts "all platforms ship together").

### Phase 10 completion gate
- **Strict: all 7 CI platforms green.** Phase 10 only closes when Sync #2 + refactor + ggml spike are green across all 7 CI targets. No local-only or subset close. Rationale: this is a foundation phase — a late-discovered Linux/Windows link conflict after feature code exists would force expensive rework.

### Claude's Discretion
- Exact `ChatCompletionParams` struct name and field ordering.
- Whether `send_chat_completion` stays a wrapper or is inlined.
- Spike timeboxing tactics and how many `[patch.crates-io]` resolution attempts before declaring failure.
- Cargo.lock regeneration timing during the merge.
- PR body / per-commit triage table formatting.
- Which of the 16 kept upstream commits (nix refactors, docs, translation fixes) warrant a triage note vs. pass-through.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Upstream sync (PREP-03)
- `UPSTREAM.md` — Full merge runbook: fork point (`85a8ed77`, merge-base `39e855d`), step-by-step merge, 9 hot-zone conflict guides (§4.1–4.9), post-merge checklist (§6), anti-patterns (§256). The new Fork Policy + Exclusion Log section is added here.
- `.github/scripts/verify-sync.sh` — 15 identity assertions; must exit 0 post-merge.
- `.github/upstream-sha.txt` — Current synced SHA `fdc8cb712dc247931099359a2d3bc8a413cf33ec`; updated only in the merge commit that lands on main.
- `.planning/phases/05-upstream-sync/05-CONTEXT.md` — Prior sync decisions (merge strategy, branch naming `upstream/sync-YYYY-MM-DD`, PR-to-main gate, no develop branch).

### TECH-04 refactor (PREP-02)
- `src-tauri/src/llm_client.rs` §111–188 — `send_chat_completion` + `send_chat_completion_with_schema`; private `ChatCompletionRequest` struct at line 36 (name collision to avoid).
- `src-tauri/src/actions.rs` §207, §265 — the only 2 call sites.

### ggml spike (PREP-01)
- `src-tauri/Cargo.toml` §73–111 — `transcribe-rs` per-platform features (`whisper-cpp`/`whisper-metal`/`whisper-vulkan`) + existing `[patch.crates-io]` block (tauri forks) to mirror for ggml.
- Memory `project_embedded_local_llm_runtime` — engine = `llama-cpp-2` over candle/mistral.rs; ggml-conflict spike is the top risk.

### Project-level
- `.planning/PROJECT.md` — Local-first constraint (Bedrock exclusion rationale), prior key decisions.
- `.planning/REQUIREMENTS.md` — PREP-01/02/03 acceptance criteria, "selective cherry-pick going forward" policy statement.
- `.planning/ROADMAP.md` §"Phase 10" — Success criteria + research spike flags.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`[patch.crates-io]` pattern** (`Cargo.toml:111`) — already used for 3 tauri forks; the proven mechanism for forcing a single ggml build if the spike hits a symbol conflict.
- **`UPSTREAM.md`** — mature 17KB runbook with hot-zone conflict guides; Sync #2 follows it, doesn't reinvent it.
- **`verify-sync.sh`** — 15-assertion identity gate, reused as the post-merge check.
- **`ReasoningConfig` Default derive** (`llm_client.rs:27`) — the style template for the new params struct's `..Default::default()` ergonomics.

### Established Patterns
- **Sync flow**: dedicated dated branch → PR to main → CI gate → solo merge (Phase 5 precedent, now on `feat/v1.3-smart-modes` working branch).
- **Per-platform Cargo features**: `transcribe-rs` already feature-gates whisper backend by OS; `llama-cpp-2` GPU features will follow the same per-target pattern in Phase 11.
- **Identity preservation**: `X-Title: Dictus`, `User-Agent: Dictus/...`, REFERER intentionally kept as upstream Handy URL (`llm_client.rs:63–76`) — do not "fix" during merge.

### Integration Points
- `llm_client.rs` ↔ `actions.rs` — refactor blast radius is exactly 2 calls; low risk.
- `Cargo.toml` / `Cargo.lock` — where the ggml spike lands; Cargo.lock conflicts during merge must use upstream-then-regenerate, never hand-edit (UPSTREAM.md §4.7).
- Upstream commit `966ff99` (async GPU query) in the synced delta is **relevant to Phase 11** GPU detection — flag forward, don't act on it here.

</code_context>

<specifics>
## Specific Ideas

- Working branch is `feat/v1.3-smart-modes` (created 2026-05-29) — all Phase 10–13 code work lands here; planning docs stay on `main`. Sync #2's own dated branch (`upstream/sync-YYYY-MM-DD`) should branch from / PR into the working branch context appropriately.
- Spike philosophy: "don't pay for the bake-off unless needed" — llama-cpp-2 is the bet, candle/mistral.rs is the insurance, evaluated only on failure.
- Sync #2 is framed as the **last bulk catch-up**; the deliverable is as much the documented policy shift as the merge itself.

</specifics>

<deferred>
## Deferred Ideas

- **Phase 6 sync automation** (AI-driven cherry-pick triage pipeline) — `UPSTREAM.md` already references this as future; SYNC-A1 in this phase is manual per-commit triage, not the automated pipeline.
- **candle/mistral.rs as primary engine** — only revisited if PREP-01 fails; not the default path.
- Upstream `966ff99` async GPU query adoption — relevant to Phase 11, not Phase 10.

</deferred>

---

*Phase: 10-prerequisite-gate*
*Context gathered: 2026-05-29*
