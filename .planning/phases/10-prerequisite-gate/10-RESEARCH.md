# Phase 10: Prerequisite Gate - Research

**Researched:** 2026-05-29
**Domain:** Upstream Git merge / Rust struct refactoring / Cargo ggml linking
**Confidence:** HIGH (all critical claims verified against live code, git log, and build scripts)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Execution order: PREP-03 (Sync #2) first → PREP-02 (TECH-04 refactor) → PREP-01 (ggml spike). Rationale: sync may touch llm_client.rs; refactor on up-to-date base.
- PREP-03 merge method: `git merge upstream/main` (full merge, preserves history), then `git revert aee682f` in the same sync branch to neutralize AWS Bedrock. No cherry-pick strategy.
- Sync branch naming: `upstream/sync-YYYY-MM-DD` → PR to `main` (or working branch) → CI gate → Pierre merges.
- Post-merge: `verify-sync.sh` (15 assertions) must exit 0. All 9 hot-zone conflict guides in UPSTREAM.md must be honored.
- After Sync #2, fork policy transitions to **selective cherry-pick** going forward. New section "Fork Policy: Selective Cherry-Pick" + Exclusion Log table added to UPSTREAM.md. Bedrock `aee682f` is entry #1.
- PREP-02 refactor shape: `send_chat_completion_with_schema` 8-arg → plain public params struct. Name must NOT be `ChatCompletionRequest` (collision with existing private wire-body struct at line 36). Candidate: `ChatCompletionParams`. No builder pattern. Struct derives `Default`.
- Both callers in `actions.rs` (~207, ~265) must be updated. `send_chat_completion` may stay as thin wrapper or be inlined — Claude's discretion.
- Remove `#[allow(clippy::too_many_arguments)]` at line 137. `cargo clippy --all-targets -- -D warnings` must exit 0 on all platforms.
- PREP-01 spike: commit to `llama-cpp-2` first. Only fall back to candle/mistral.rs if llama-cpp-2 coexistence is impossible even with patching. Sidecar/llama-server is explicitly out of scope.
- Phase 10 completion gate: all 7 CI platforms green. No local-only or subset close.

### Claude's Discretion
- Exact `ChatCompletionParams` struct name and field ordering.
- Whether `send_chat_completion` stays a wrapper or is inlined.
- Spike timeboxing tactics and how many `[patch.crates-io]` resolution attempts before declaring failure.
- Cargo.lock regeneration timing during the merge.
- PR body / per-commit triage table formatting.
- Which of the 16 kept upstream commits warrant a triage note vs. pass-through.

### Deferred Ideas (OUT OF SCOPE)
- Phase 6 sync automation (AI-driven cherry-pick triage pipeline).
- candle/mistral.rs as primary engine — only if PREP-01 fails.
- Upstream `966ff99` async GPU query adoption — relevant to Phase 11, not Phase 10 (flag forward, do not implement).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| PREP-03 | Upstream Sync #2 merged with per-commit triage (SYNC-A1); identity integrity preserved (`verify-sync.sh` green); fork policy transitions to documented selective cherry-pick going forward (AWS Bedrock `aee682f` decision recorded) | Full 17-commit delta enumerated; aee682f identified as commit #6 (not #12 as CONTEXT said — see correction below); verify-sync.sh 15 assertions documented; all hot-zone files confirmed |
| PREP-02 | TECH-04 resolved — `llm_client.rs send_chat_completion_with_schema` refactored from 8 args to a request struct; `#[allow(clippy::too_many_arguments)]` removed; `cargo clippy --all-targets -- -D warnings` passes clean | Exact current signature verified (8 params, lines 138–147); private `ChatCompletionRequest` name collision confirmed at line 36; both call sites verified at ~207 and ~265 in actions.rs |
| PREP-01 | `llama-cpp-2` compiles cleanly alongside the existing `transcribe-rs` on all 7 CI platforms — ggml symbol conflict resolved (feasibility spike; `[patch.crates-io]` fallback ready) | ggml conflict confirmed as real (both whisper-rs-sys 0.15.0 and llama-cpp-sys-2 statically link libggml, libggml-base, libggml-cpu); `system-ggml` feature in llama-cpp-sys-2 identified as primary resolution path; 7 CI platforms enumerated from main-build.yml |
</phase_requirements>

---

## Summary

Phase 10 clears three non-user-facing blockers that must be resolved before any v1.3 feature code is written. All three work items have been researched against live code and verified files.

**PREP-03** is an upstream Git merge of a 17-commit delta. The current `upstream-sha.txt` is `fdc8cb712dc247931099359a2d3bc8a413cf33ec`. The upstream HEAD is now further ahead. The full delta has been enumerated. Commit `aee682f` (AWS Bedrock) is commit #6 in the sequence — not #12 as the CONTEXT estimated. The revert-after-merge strategy is sound: `aee682f` changes only `src-tauri/src/settings.rs` (adds a `PostProcessProvider` push block — 10 lines), making the revert clean and low-risk. Upstream commit `966ff99` (async GPU query) is commit #3; it changes `lib.rs`, `shortcut/mod.rs`, and `bindings.ts` and is relevant to Phase 11 GPU detection — accept it in the merge but do not act on it in Phase 10. The `verify-sync.sh` script contains exactly 15 assertions (SYNC-05a through SYNC-05k, BRAND-01a, BRAND-02a, BRAND-03a, ICON-02a).

**PREP-02** is a pure Rust refactor with a blast radius of exactly 3 items: one function definition and two call sites. The function currently lives at lines 138–147 of `llm_client.rs`, takes 8 positional arguments, and is guarded by `#[allow(clippy::too_many_arguments)]` at line 137. The `ReasoningConfig` struct at line 27 already derives `Default` — this is the exact pattern to replicate for the new `ChatCompletionParams` struct. The private wire-body `ChatCompletionRequest` struct sits at line 36 — any new public struct must not use that name.

**PREP-01** is a feasibility spike with a confirmed risk. `whisper-rs-sys` 0.15.0 (the transitive dependency under `transcribe-rs`) statically links `libggml`, `libggml-base`, and `libggml-cpu` via its CMake build (verified in the live build.rs). `llama-cpp-sys-2` does the same by default. This creates duplicate symbol definitions (`ggml_backend_buft_name`, etc.) — a known upstream issue. The resolution path is `llama-cpp-sys-2`'s `system-ggml` feature, which sets `LLAMA_USE_SYSTEM_GGML=ON` and links against system-installed GGML instead of building its own. The `[patch.crates-io]` mechanism already proven in the project (3 tauri forks) is the secondary resolution approach if needed.

**Primary recommendation:** Execute in strict PREP-03 → PREP-02 → PREP-01 order. For PREP-01, start the spike by adding `llama-cpp-2` with `features = ["metal"]` (macOS) / `features = ["vulkan"]` (Windows/Linux), run `cargo tree | grep ggml` to confirm the conflict, then attempt `features = ["metal", "system-ggml"]` as the first resolution attempt before any `[patch.crates-io]` work.

---

## Standard Stack

### Core (all three work items)

| Library / Tool | Version | Purpose | Why Standard |
|---------------|---------|---------|--------------|
| `llama-cpp-2` | 0.1.146 (2026-04-30) | Rust wrapper for llama.cpp, GGUF inference | Chosen over candle/mistral.rs per memory context; has `system-ggml` feature critical for conflict resolution |
| `llama-cpp-sys-2` | (transitive) | Low-level FFI bindings to llama.cpp | Ships with llama-cpp-2; has `system-ggml` + `system-ggml-static` features |
| `whisper-rs-sys` | 0.15.0 | FFI bindings to whisper.cpp (transitive under transcribe-rs) | Already present; its build.rs statically links ggml — source of the conflict |
| `cargo clippy` | stable toolchain | Lint enforcement | Project convention: `cargo clippy --all-targets -- -D warnings` |
| `cargo fmt` | stable toolchain | Format enforcement | CLAUDE.md requirement before commit |
| `verify-sync.sh` | — | 15-assertion post-merge identity gate | Existing script at `.github/scripts/verify-sync.sh` |

### GPU Features in llama-cpp-2

| Feature Flag | Platform | What It Enables |
|-------------|----------|-----------------|
| `metal` | macOS (aarch64 + x86_64) | Metal GPU acceleration |
| `vulkan` | Windows x64/ARM64, Linux x64/ARM64 | Vulkan GPU acceleration |
| `system-ggml` | All | Links system GGML instead of building own copy — **key for coexistence** |
| `system-ggml-static` | All | Same as system-ggml but forces static linking |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `llama-cpp-2` | `candle` / `mistral.rs` | Pure-Rust, no ggml conflict, but lower Metal/Vulkan maturity; only if llama-cpp-2 fails |
| merge + revert | cherry-pick 16 commits | Loses merge history; UPSTREAM.md anti-patterns §"Do NOT cherry-pick"; rejected |
| `ChatCompletionParams` struct | builder pattern | Builder unjustified for 2 call sites; adds boilerplate |

---

## Architecture Patterns

### PREP-03: Upstream Sync Flow

```
feat/v1.3-smart-modes (working branch)
└── upstream/sync-2026-05-29 (dated sync branch, from working branch or main)
    ├── git merge upstream/main --no-ff  (brings all 17 commits)
    ├── Resolve conflicts per §4.1–4.9 hot zones
    ├── git revert aee682f               (neutralize Bedrock)
    ├── cargo generate-lockfile          (never hand-edit Cargo.lock)
    ├── Update .github/upstream-sha.txt  (full 40-char upstream HEAD SHA)
    ├── bash .github/scripts/verify-sync.sh  (must exit 0)
    └── gh pr create --base main         (CI gate, Pierre merges)
```

**UPSTREAM.md update required:** Add "Fork Policy: Selective Cherry-Pick" section + Exclusion Log table (first entry: `aee682f`, reason: local-first, cloud provider not owned by Dictus, date: 2026-05-29).

### PREP-02: Struct Refactor Pattern

Follow the `ReasoningConfig` style (line 27 of `llm_client.rs`) exactly:

```rust
// Source: src-tauri/src/llm_client.rs:27-33 (verified live)
#[derive(Debug, Serialize, Clone, Default)]
pub struct ReasoningConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<bool>,
}
```

New struct follows same derive pattern:

```rust
#[derive(Debug, Clone, Default)]
pub struct ChatCompletionParams {
    pub provider: PostProcessProvider,   // or reference — Claude's discretion
    pub api_key: String,
    pub model: String,
    pub user_content: String,
    pub system_prompt: Option<String>,
    pub json_schema: Option<Value>,
    pub reasoning_effort: Option<String>,
    pub reasoning: Option<ReasoningConfig>,
}
```

Call-site pattern with `..Default::default()`:

```rust
// actions.rs call site 1 (~207): struct literal, all fields populated
crate::llm_client::send_chat_completion_with_schema(ChatCompletionParams {
    provider: provider.clone(),
    api_key: api_key.clone(),
    model: model.clone(),
    user_content,
    system_prompt: Some(system_prompt),
    json_schema: Some(json_schema),
    reasoning_effort: reasoning_effort.clone(),
    reasoning: reasoning.clone(),
}).await
```

### PREP-01: ggml Spike Decision Tree

```
Step 1: Add llama-cpp-2 to Cargo.toml (per-platform features)
        macOS:    features = ["metal"]
        Windows:  features = ["vulkan"]
        Linux:    features = ["vulkan"]

Step 2: cargo tree | grep ggml
        → If two separate ggml trees appear: conflict confirmed, proceed to Step 3
        → If single ggml tree: no conflict, proceed to link test

Step 3: Add "system-ggml" feature to llama-cpp-2
        macOS:    features = ["metal", "system-ggml"]
        etc.
        Then: cargo build --manifest-path src-tauri/Cargo.toml
        → If links clean: spike succeeds, add per-platform target blocks to Cargo.toml

Step 4 (if Step 3 fails): Try [patch.crates-io] to pin single ggml version
        Mirror the existing pattern (tauri forks at line 111-114)

Step 5 (fallback declaration):
        If Steps 3-4 fail after 2-3 attempts: document failure → evaluate candle/mistral.rs
```

### Recommended Project Structure (no new dirs needed)

All Phase 10 changes land in existing files:
- `src-tauri/Cargo.toml` — llama-cpp-2 dependency (PREP-01) + new params struct (PREP-02 no file change)
- `src-tauri/src/llm_client.rs` — params struct + signature change (PREP-02)
- `src-tauri/src/actions.rs` — call site updates (PREP-02)
- `UPSTREAM.md` — Fork Policy section + Exclusion Log (PREP-03)
- `.github/upstream-sha.txt` — updated upstream HEAD (PREP-03, only in merge commit)

### Anti-Patterns to Avoid

- **Merging via cherry-pick:** Loses merge history; forbidden by UPSTREAM.md anti-patterns section.
- **Hand-editing Cargo.lock:** `cargo generate-lockfile` is the only correct approach (§4.7).
- **`git checkout --theirs src-tauri/tauri.conf.json`:** Accepts Handy identity fields — always resolve manually.
- **Adding `ChatCompletionRequest` as the new struct name:** Collides with private wire-body struct at line 36.
- **Builder pattern for ChatCompletionParams:** Only 2 call sites; plain struct-literal is sufficient.
- **Sidecar / llama-server as conflict fallback:** Explicitly rejected in CONTEXT.md.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Identity verification post-merge | Custom grep script | `.github/scripts/verify-sync.sh` | 15 assertions already written and tested in Phase 5 |
| ggml symbol deduplication | Manual symbol export map / linker scripts | `llama-cpp-2 system-ggml` feature or `[patch.crates-io]` | CMake-level flag or Cargo-level patch is the only safe approach; linker scripts are fragile across 7 platforms |
| Cargo.lock conflict resolution | Manual merge of Cargo.lock | `cargo generate-lockfile` | Machine-generated format; manual edits break subtly |
| Per-platform ggml feature gating | Single feature set for all platforms | Per-target Cargo feature blocks (mirrors existing `transcribe-rs` pattern at Cargo.toml:82–109) | Different GPU backends per OS; same pattern already proven |

**Key insight:** The `[patch.crates-io]` mechanism already proven in this project (lines 111–114 of Cargo.toml patch 3 tauri forks) is exactly the right tool to force a single shared ggml build if the `system-ggml` feature path fails.

---

## Common Pitfalls

### Pitfall 1: Incorrect aee682f Position in Sequence
**What goes wrong:** CONTEXT.md states aee682f is "commit 12 of 17" but git log shows it is commit #6 of 17 (counting oldest-first). A planner who acts on position #12 will be confused about what follows the revert.
**Why it happens:** CONTEXT was written from a reverse-order perspective or estimated.
**How to avoid:** Use the enumerated table in this document (verified from live git log, oldest-first).
**Warning signs:** If you see 11 commits between the merge-base and aee682f, something is wrong.

### Pitfall 2: Updating upstream-sha.txt Before the Merge Lands
**What goes wrong:** The upstream detection workflow reads `upstream-sha.txt`; if updated before the merge commit lands on main, the tracking issue disappears and the merge is never reviewed.
**Why it happens:** Temptation to update the file early during branch work.
**How to avoid:** `upstream-sha.txt` is updated only as part of the merge commit (Step 5 of UPSTREAM.md). Never update it from the sync branch before PR merge.

### Pitfall 3: Using `git checkout --theirs` on tauri.conf.json
**What goes wrong:** Accepts all Handy identity fields (productName: "Handy", identifier: "com.cjpais.handy") — causes verify-sync.sh SYNC-05a/b/c to fail.
**Why it happens:** Easiest conflict resolution shortcut.
**How to avoid:** Always resolve `tauri.conf.json` manually, field by field, keeping Dictus values. Verify with `jq` commands in §4.1 of UPSTREAM.md.

### Pitfall 4: ChatCompletionRequest Name Collision
**What goes wrong:** Using `ChatCompletionRequest` for the new public params struct — Rust will fail to compile because a private `ChatCompletionRequest` already exists at line 36 of `llm_client.rs`.
**Why it happens:** Obvious naming choice for a "request" struct.
**How to avoid:** Use `ChatCompletionParams` (or another non-colliding name). The private `ChatCompletionRequest` is the internal HTTP wire body — the new public struct is call parameters.

### Pitfall 5: ggml Conflict Not Manifesting on macOS But Breaking on Linux/Windows
**What goes wrong:** The `system-ggml` approach may resolve locally on macOS (where GGML might be system-installed via Homebrew) but fail on Linux CI (no system GGML package) or Windows (different linker).
**Why it happens:** `system-ggml` requires a pre-installed system GGML. CI environments may not have it.
**How to avoid:** During the spike, test with `cargo build` on all 3 OS families. If `system-ggml` requires a system-installed GGML not available in CI, the `[patch.crates-io]` approach (shared source-built ggml) is the correct path. The Phase 10 gate requires all 7 CI platforms green.

### Pitfall 6: send_chat_completion Thin Wrapper Breaking After Refactor
**What goes wrong:** If `send_chat_completion` is kept as a thin wrapper calling `send_chat_completion_with_schema`, the wrapper needs updating after the signature change — easy to forget.
**Why it happens:** Wrapper looked untouched in the refactor plan.
**How to avoid:** `send_chat_completion` (lines 111–130) delegates to `send_chat_completion_with_schema` with `None, None` for system_prompt and json_schema. After the params struct change, it must construct a `ChatCompletionParams` and pass it. Both options (keep wrapper, inline it) require an update.

### Pitfall 7: AGENTS.md / CLAUDE.md Merge Conflict (Commit 564fbc8)
**What goes wrong:** Upstream commit `564fbc8` rewrites `CLAUDE.md` almost entirely (160 lines removed, 160 added) and creates/rewrites `AGENTS.md`. The Dictus `CLAUDE.md` (this project's project instructions) is completely different from upstream's — accepting upstream's `CLAUDE.md` would destroy Dictus project instructions.
**Why it happens:** Upstream unified their CLAUDE.md and AGENTS.md into a single source; Dictus CLAUDE.md is a different document.
**How to avoid:** For `CLAUDE.md`: keep Dictus version entirely. For `AGENTS.md`: this is a new upstream file with no Dictus equivalent — accept upstream's version as-is (it documents AI workflow rules for upstream's repo; harmless to include).

---

## Code Examples

Verified from live files:

### Current send_chat_completion_with_schema signature (exact, verified)
```rust
// Source: src-tauri/src/llm_client.rs:137-147 (verified 2026-05-29)
#[allow(clippy::too_many_arguments)]   // line 137 — REMOVE this
pub async fn send_chat_completion_with_schema(
    provider: &PostProcessProvider,    // arg 1
    api_key: String,                   // arg 2
    model: &str,                       // arg 3
    user_content: String,              // arg 4
    system_prompt: Option<String>,     // arg 5
    json_schema: Option<Value>,        // arg 6
    reasoning_effort: Option<String>,  // arg 7
    reasoning: Option<ReasoningConfig>, // arg 8
) -> Result<Option<String>, String>
```

### send_chat_completion wrapper (exact, verified)
```rust
// Source: src-tauri/src/llm_client.rs:111-130 (verified 2026-05-29)
pub async fn send_chat_completion(
    provider: &PostProcessProvider,
    api_key: String,
    model: &str,
    prompt: String,
    reasoning_effort: Option<String>,
    reasoning: Option<ReasoningConfig>,
) -> Result<Option<String>, String> {
    send_chat_completion_with_schema(
        provider, api_key, model, prompt, None, None, reasoning_effort, reasoning,
    ).await
}
```

### Call site 1 in actions.rs (~207, verified)
```rust
// Source: src-tauri/src/actions.rs:207-216 (verified 2026-05-29)
match crate::llm_client::send_chat_completion_with_schema(
    &provider,
    api_key.clone(),
    &model,
    user_content,
    Some(system_prompt),
    Some(json_schema),
    reasoning_effort.clone(),
    reasoning.clone(),
).await
```

### Call site 2 in actions.rs (~265, verified)
```rust
// Source: src-tauri/src/actions.rs:265-272 (verified 2026-05-29)
match crate::llm_client::send_chat_completion(
    &provider,
    api_key,
    &model,
    processed_prompt,
    reasoning_effort,
    reasoning,
).await
```

### Private ChatCompletionRequest (name to avoid, verified)
```rust
// Source: src-tauri/src/llm_client.rs:35-45 (verified 2026-05-29)
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {    // PRIVATE, wire-body — do NOT reuse this name
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning: Option<ReasoningConfig>,
}
```

### ReasoningConfig Default-derive style (template for new struct)
```rust
// Source: src-tauri/src/llm_client.rs:27-33 (verified 2026-05-29)
#[derive(Debug, Serialize, Clone, Default)]
pub struct ReasoningConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<bool>,
}
```

### Existing [patch.crates-io] pattern (model for ggml fix)
```toml
# Source: src-tauri/Cargo.toml:111-114 (verified 2026-05-29)
[patch.crates-io]
tauri-runtime = { git = "https://github.com/cjpais/tauri.git", branch = "handy-2.10.2" }
tauri-runtime-wry = { git = "https://github.com/cjpais/tauri.git", branch = "handy-2.10.2" }
tauri-utils = { git = "https://github.com/cjpais/tauri.git", branch = "handy-2.10.2" }
```

### Per-platform transcribe-rs feature pattern (model for llama-cpp-2)
```toml
# Source: src-tauri/Cargo.toml:82-109 (verified 2026-05-29)
# Default (unix fallback):
transcribe-rs = { version = "0.3.8", features = ["whisper-cpp", "onnx"] }

[target.'cfg(windows)'.dependencies]
transcribe-rs = { version = "0.3.3", features = ["whisper-vulkan", "ort-directml"] }

[target.'cfg(target_os = "macos")'.dependencies]
transcribe-rs = { version = "0.3.3", features = ["whisper-metal"] }

[target.'cfg(target_os = "linux")'.dependencies]
transcribe-rs = { version = "0.3.3", features = ["whisper-vulkan"] }
```

---

## PREP-03 Detail: Full 17-Commit Delta

Current `upstream-sha.txt`: `fdc8cb712dc247931099359a2d3bc8a413cf33ec`

Commits from stored SHA to upstream HEAD (oldest-first, verified via `git log`):

| # | SHA | Title | Risk | Action |
|---|-----|-------|------|--------|
| 1 | `f26fe0d` | Fix (linux) overlay problem in kde (#1121) | LOW — `overlay.rs` improvement | Accept |
| 2 | `0392b7b` | docs(readme): add Linux startup troubleshooting section (#1266) | NONE | Accept |
| 3 | `966ff99` | query gpu async (#1246) | MEDIUM — changes `lib.rs`, `shortcut/mod.rs`, `bindings.ts`; flag for Phase 11 | Accept; FLAG for Phase 11 |
| 4 | `11311be` | fix(overlay): parse HANDY_NO_GTK_LAYER_SHELL as boolean (#1269) | LOW — `overlay.rs` only | Accept |
| 5 | `564fbc8` | docs: unify CLAUDE.md and AGENTS.md into single source of truth (#1272) | HIGH — rewrites CLAUDE.md; keep Dictus CLAUDE.md | Keep Dictus CLAUDE.md; accept AGENTS.md as new file |
| 6 | `aee682f` | feat: add AWS Bedrock (Mantle) as post-processing provider (#1288) | LOW revert — only `settings.rs` 10-line push block | **REVERT after merge** |
| 7 | `a4d671a` | fix: improve German translation quality (#1292) | LOW — `de/translation.json` only | Accept |
| 8 | `c1e11fa` | refactor(nix): replace manual bindgen env with rustPlatform.bindgenHook (#1255) | NONE | Accept |
| 9 | `af6ec6c` | chore(nix): add bun node_modules normalization scripts (#1256) | NONE | Accept |
| 10 | `4b7bb4e` | docs(audio): clarify what the mic-init timing log actually measures (#1330) | NONE | Accept |
| 11 | `8346bc2` | fix(nix): Fix for macOS build for nixpkgs (#1316) | NONE | Accept |
| 12 | `085cd53` | release v0.8.3 | MEDIUM — bumps version in tauri.conf.json, Cargo.toml; reject upstream version numbers | Keep Dictus version (`0.1.3`) |
| 13 | `a385371` | refactor(nix): rely on cargo-tauri.hook standard phases (#1335) | NONE | Accept |
| 14 | `1d042f3` | docs(agents): make GitHub workflow rules actionable for AI assistants (#1393) | NONE — AGENTS.md update | Accept |
| 15 | `e3206aa` | refactor(nix): drop redundant LD_LIBRARY_PATH wrapper prefix (#1392) | NONE | Accept |
| 16 | `933a525` | docs: add overlay/pasting issue workaround for Linux to Known Issues (#1426) | NONE | Accept |
| 17 | `10a4c31` | docs: complete Portuguese translation for portable updates (#1422) | LOW — `pt/translation.json` | Accept |

**Key correction from CONTEXT.md:** aee682f is commit #6 (not #12) when ordered oldest-first.

**Additional hot zone discovered:** Commit `085cd53` (v0.8.3 release) bumps `package.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, and `tauri.conf.json` version fields. Reject the upstream version numbers — keep `"version": "0.1.3"` in Cargo.toml and `tauri.conf.json`. This is an additional conflict site not in the original UPSTREAM.md §4 hot zones.

**Additional hot zone: CLAUDE.md (commit 564fbc8):** Upstream rewrites `CLAUDE.md` entirely. The Dictus `CLAUDE.md` (project instructions, checked into repo) must be kept verbatim. Use `git checkout --ours CLAUDE.md` on this file — it is safe here because the file is entirely Dictus-specific content (not upstream branding).

**`build.rs` changes (commit 8346bc2 / `085cd53` range):** Upstream modifies `src-tauri/build.rs` to support non-Xcode toolchains (adds `SDKROOT`/`SWIFTC` env var overrides). Accept upstream as-is — no Dictus-specific content in build.rs.

---

## PREP-03 Detail: verify-sync.sh Assertions (15 total, enumerated)

| ID | What It Checks |
|----|---------------|
| SYNC-05a | `productName` is `"Dictus"` in `tauri.conf.json` |
| SYNC-05b | `identifier` is `"com.dictus.desktop"` in `tauri.conf.json` |
| SYNC-05c | Updater endpoint contains `"getdictus/dictus-desktop"` |
| SYNC-05d | No `"Handy"` value in `en/translation.json` (outside acknowledgments block) |
| SYNC-05e | `llm_client.rs` contains `Dictus/1.0` (User-Agent) |
| SYNC-05f | `llm_client.rs` contains `"X-Title"` and `"Dictus"` |
| SYNC-05g | No `handy.computer` reference in `actions.rs` |
| SYNC-05h | No `handy.computer` reference in `llm_client.rs` |
| SYNC-05i | No `handy.log` reference in any `src/` or `src-tauri/src/` file |
| SYNC-05j | `cargo build --manifest-path src-tauri/Cargo.toml` succeeds |
| SYNC-05k | `upstream-sha.txt` is a full 40-char SHA |
| BRAND-01a | No `handy-` filename prefix in `src-tauri/src/` (except `handy_keys` crate) |
| BRAND-02a | No `"Handy Portable Mode"` string in `portable.rs` |
| BRAND-03a | No `%APPDATA%/handy` path in `DebugPaths.tsx` |
| ICON-02a | `icon.ico` contains all 6 required layer sizes (16, 24, 32, 48, 64, 256) |

---

## PREP-01 Detail: ggml Conflict Analysis

### The Conflict
Both `whisper-rs-sys` 0.15.0 and `llama-cpp-sys-2` 0.1.146 independently build and statically link their own copies of the ggml library. The `whisper-rs-sys` build.rs (verified from cargo cache) links:
```
cargo:rustc-link-lib=static=whisper
cargo:rustc-link-lib=static=ggml
cargo:rustc-link-lib=static=ggml-base
cargo:rustc-link-lib=static=ggml-cpu
```
`llama-cpp-sys-2` links its own `ggml`, `ggml-base`, `ggml-cpu` by default. Result: duplicate symbol linker errors (e.g., `ggml_backend_buft_name already defined`). This is a confirmed upstream issue (github.com/ggml-org/llama.cpp/issues/9267, github.com/ggml-org/llama.cpp/issues/11303), marked stale without upstream resolution as of May 2026.

### Resolution Path 1: llama-cpp-2 system-ggml Feature (TRY FIRST)
Add `features = ["metal", "system-ggml"]` (macOS) to llama-cpp-sys-2. This sets `LLAMA_USE_SYSTEM_GGML=ON` in CMake, causing llama.cpp to link against a system-installed GGML instead of building its own. **Caveat:** This requires a system-installed GGML package in CI environments. Ubuntu CI already installs `libopenblas-dev` but not ggml specifically. May require adding `libggml-dev` (if available) to CI apt steps or pre-building and caching GGML. This path has unresolved CI feasibility — the spike must test it.

### Resolution Path 2: [patch.crates-io] Single Build (TRY SECOND)
Mirror the existing tauri fork pattern. If a crate can be created (or found) that exposes a single shared ggml build that both `whisper-rs-sys` and `llama-cpp-sys-2` link against, the `[patch.crates-io]` table forces both crates to use it. This is more complex and requires a compatible shared ggml crate.

### Resolution Path 3: /FORCE:MULTIPLE Rustflag (Windows MSVC workaround only)
Adding `-C link-arg=/FORCE:MULTIPLE` to RUSTFLAGS ignores duplicate symbols on Windows MSVC. This is not a real fix and will not work on macOS/Linux. Do not use as primary approach.

### Resolution Path 4 (Contingency): candle / mistral.rs
Pure-Rust inference engines with no ggml static link. Only evaluate if Paths 1–2 both fail. Not the default path per CONTEXT.md.

### 7 CI Platforms (verified from main-build.yml)

| Platform | Runner | Target | GPU Backend |
|----------|--------|--------|-------------|
| macOS ARM64 | `macos-26` | `aarch64-apple-darwin` | Metal |
| macOS x64 | `macos-latest` | `x86_64-apple-darwin` | Metal |
| Ubuntu 22.04 x64 | `ubuntu-22.04` | `x86_64-unknown-linux-gnu` | Vulkan |
| Ubuntu 24.04 x64 | `ubuntu-24.04` | `x86_64-unknown-linux-gnu` | Vulkan |
| Ubuntu 24.04 ARM64 | `ubuntu-24.04-arm` | `aarch64-unknown-linux-gnu` | Vulkan |
| Windows x64 | `windows-latest` | `x86_64-pc-windows-msvc` | Vulkan |
| Windows ARM64 | `windows-11-arm` | `aarch64-pc-windows-msvc` | Vulkan |

**Important:** The CI already installs Vulkan SDK for Windows (1.4.309.0) and Ubuntu platforms. For Ubuntu 24.04 x64, Vulkan SDK 1.3.290 is installed via lunarg apt repo. For Ubuntu ARM64, jakoch/install-vulkan-sdk-action@v1 installs 1.4.335.0. These Vulkan SDK steps will need to remain in place for llama-cpp-2 Vulkan builds.

**Existing CI note on ggml:** `build.yml` already sets `GGML_NATIVE=OFF`, `GGML_AVX=OFF`, `GGML_AVX2=OFF`, `GGML_FMA=OFF`, `GGML_F16C=OFF` for x86_64 non-macOS non-ARM targets. This is for whisper's ggml — llama-cpp-2 will need the same treatment to avoid SIGILL on older CPUs.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact for Phase 10 |
|--------------|------------------|--------------|---------------------|
| UPSTREAM.md §1 had 4-commit delta | Delta is now 17 commits | 2026-04-10 to 2026-05-12 | Sync is larger than original runbook described |
| Bulk merge each sync | Selective cherry-pick going forward | After Phase 10 | PREP-03 is the last bulk merge |
| `send_chat_completion_with_schema` with 8 positional args + `#[allow]` | Params struct | PREP-02 delivers this | Eliminates clippy suppression |
| No in-process LLM | llama-cpp-2 (Phase 11) | PREP-01 gates it | Spike must pass before Phase 11 begins |

**Deprecated/outdated in UPSTREAM.md:**
- §1 "Known post-v0.8.2 upstream delta (4 commits)" — that table is now stale; it was accurate at Phase 5 time. The new delta is 17 commits. UPSTREAM.md will be updated in PREP-03 with the new Fork Policy section; the old 4-commit table can be left as historical context or updated — Claude's discretion.

---

## Open Questions

1. **Does `system-ggml` work in CI without a pre-installed GGML package?**
   - What we know: Ubuntu CI installs `libopenblas-dev` but no explicit GGML package; macOS runners may have Homebrew GGML from whisper-cpp formula.
   - What's unclear: Whether `LLAMA_USE_SYSTEM_GGML=ON` requires a system package or can be satisfied by a GGML built by whisper-rs-sys's own CMake output.
   - Recommendation: First spike attempt should test locally with `cargo build` before CI; add explicit ggml detection step.

2. **Does `git revert aee682f` cleanly apply after a full merge?**
   - What we know: `aee682f` changes only 10 lines in `settings.rs` (adds one `providers.push()` block). The diff is clean.
   - What's unclear: Whether any of the later 11 commits (7–17) touch the same area of `settings.rs`.
   - Recommendation: Run `git log aee682f..upstream/main -- src-tauri/src/settings.rs` to verify no later commit modifies the Bedrock block. Based on the delta stat review, no other commit touches `settings.rs`.

3. **Does `966ff99` async GPU query conflict with existing Dictus transcription manager changes?**
   - What we know: `966ff99` changes `lib.rs` (adds spawn_blocking pre-warm), `shortcut/mod.rs` (makes `get_available_accelerators` async), and `bindings.ts`. The Phase 7/8 shutdown and privacy changes also touch `lib.rs`.
   - What's unclear: Whether there are merge conflicts in `lib.rs` at the shutdown handler sites.
   - Recommendation: Expect a merge conflict in `lib.rs`; resolve per hot-zone rules (accept upstream's GPU pre-warm addition, keep Dictus shutdown handlers). This is not in the original UPSTREAM.md §4 hot zones — treat it as a new hot zone.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) + ESLint/TypeScript (frontend) |
| Config file | `src-tauri/Cargo.toml` (Rust); `src-tauri` workspace |
| Quick run command | `cargo test --manifest-path src-tauri/Cargo.toml` |
| Full suite command | `cargo test --manifest-path src-tauri/Cargo.toml --all-targets && bun run lint && cargo clippy --all-targets -- -D warnings` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PREP-03 | Identity assertions after merge | Script | `bash .github/scripts/verify-sync.sh` | ✅ |
| PREP-03 | Build succeeds after merge | Build smoke | `cargo build --manifest-path src-tauri/Cargo.toml` | ✅ (via verify-sync SYNC-05j) |
| PREP-03 | upstream-sha.txt is full 40-char SHA | Script | `bash .github/scripts/verify-sync.sh` (SYNC-05k) | ✅ |
| PREP-02 | Clippy clean, no `#[allow(too_many_arguments)]` | Lint | `cargo clippy --all-targets -- -D warnings` | ✅ (cargo toolchain) |
| PREP-02 | Both call sites compile with new params struct | Build | `cargo build --manifest-path src-tauri/Cargo.toml` | ✅ |
| PREP-01 | llama-cpp-2 + transcribe-rs coexist without linker error | Build | `cargo build --manifest-path src-tauri/Cargo.toml` | ❌ Wave 0 (llama-cpp-2 not yet in Cargo.toml) |
| PREP-01 | No duplicate ggml symbols | cargo tree | `cargo tree --manifest-path src-tauri/Cargo.toml \| grep ggml` | ❌ Wave 0 |
| All | All 7 CI platforms green | CI | Full main-build.yml matrix | ✅ (CI exists, coverage added by PREP-01) |

### Sampling Rate
- **Per task commit:** `cargo build --manifest-path src-tauri/Cargo.toml`
- **Per wave merge:** `bash .github/scripts/verify-sync.sh && cargo clippy --all-targets -- -D warnings && bun run lint`
- **Phase gate:** All 7 CI platforms green in main-build.yml before closing Phase 10

### Wave 0 Gaps
- [ ] `src-tauri/Cargo.toml` — add `llama-cpp-2` dependency entry (enables PREP-01 test commands)
- None for PREP-02 or PREP-03 — existing test infrastructure covers all behavior

---

## Sources

### Primary (HIGH confidence)
- Live `git log fdc8cb712..upstream/main` — full 17-commit delta enumerated 2026-05-29
- `src-tauri/src/llm_client.rs` (lines 27–188) — exact function signatures, struct names, line numbers
- `src-tauri/src/actions.rs` (lines 190–290) — both call sites verified
- `src-tauri/Cargo.toml` (lines 73–114) — per-platform features, existing `[patch.crates-io]`
- `.github/scripts/verify-sync.sh` — all 15 assertions enumerated
- `.github/upstream-sha.txt` — confirmed `fdc8cb712dc247931099359a2d3bc8a413cf33ec`
- `.github/workflows/main-build.yml` — 7 CI platform/target matrix verified
- `.github/workflows/build.yml` — Vulkan SDK versions, GGML env vars confirmed
- `~/.cargo/registry/.../whisper-rs-sys-0.15.0/build.rs` — static ggml linking confirmed
- `https://raw.githubusercontent.com/utilityai/llama-cpp-rs/main/llama-cpp-2/Cargo.toml` — llama-cpp-2 features including `system-ggml`, `metal`, `vulkan`, version 0.1.146

### Secondary (MEDIUM confidence)
- `docs.rs/crate/llama-cpp-sys-2/latest/source/build.rs` — `system-ggml` feature behavior (sets `LLAMA_USE_SYSTEM_GGML=ON`, links system GGML)
- `github.com/ggml-org/whisper.cpp/discussions/3544` — shared GGML approach confirmed viable via `talk-llama` example
- `users.rust-lang.org/t/bindings-linking-conflicts/116919` — llama-cpp-2 + whisper-rs duplicate symbol description and proposed solutions

### Tertiary (LOW confidence — needs validation in spike)
- Claim that `system-ggml` feature resolves the CI build conflict — unverified until spike executes on all 7 platforms
- `GGML_NATIVE=OFF` env var behavior with llama-cpp-2 (inferred from existing whisper pattern in build.yml)

---

## Metadata

**Confidence breakdown:**
- Standard stack (PREP-03 process): HIGH — verified against live UPSTREAM.md and verify-sync.sh
- Standard stack (PREP-02 refactor): HIGH — verified against live llm_client.rs and actions.rs
- Standard stack (PREP-01 llama-cpp-2): HIGH for library existence and features; MEDIUM for conflict resolution approach
- Architecture patterns: HIGH for PREP-03/02; MEDIUM for PREP-01 (system-ggml unverified in CI)
- Pitfalls: HIGH (derived from verified live code discrepancies with CONTEXT.md claims)

**Research date:** 2026-05-29
**Valid until:** 2026-06-28 (stable libraries, 30-day window; re-verify llama-cpp-2 version if > 30 days pass)
