---
phase: 12
slug: smart-modes-data-layer
status: ready
nyquist_compliant: true
wave_0_complete: false
created: 2026-06-03
---

# Phase 12 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` (cargo test) |
| **Config file** | none — workspace-native; `src-tauri/Cargo.toml` dev-deps (`tempfile` present) |
| **Quick run command** | `cd src-tauri && cargo test settings::` |
| **Full suite command** | `cd src-tauri && cargo test` |
| **Estimated runtime** | ~30-90 seconds (incremental; first build longer) |

---

## Sampling Rate

- **After every task commit:** Run `cd src-tauri && cargo test settings::` (or the module touched)
- **After every plan wave:** Run `cd src-tauri && cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green + `cargo clippy` clean + `bindings.ts` regenerated
- **Max feedback latency:** ~90 seconds

---

## Per-Task Verification Map

*Populated by the planner as tasks are defined. Each task touching migration, data model, CRUD, or routing must map to a `cargo test` assertion or an explicit Manual-Only entry below.*

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 01-T1 | 12-01 | 1 | MODE-01 | unit | `cargo test settings::` (serde default load of v1.2 JSON) | ❌ W0 | ⬜ pending |
| 01-T2 | 12-01 | 1 | MODE-02 | unit | `cargo test settings::default_smart_modes` | ❌ W0 | ⬜ pending |
| 01-T3 | 12-01 | 1 | MODE-01 | unit | `cargo test settings::migration` | ❌ W0 | ⬜ pending |
| 02-T1 | 12-02 | 2 | MODE-03 | unit | `cargo test shortcut::` (crud_delete_last_guard, crud_delete_reassigns_active) | ❌ W0 | ⬜ pending |
| 02-T2 | 12-02 | 2 | MODE-04 | unit | `cargo test shortcut::tests::smart_mode_binding_id_format` | ❌ W0 | ⬜ pending |
| 02-T3 | 12-02 | 2 | MODE-03 | smoke (manual) | `cargo build && grep -q "SmartMode" src/bindings.ts` | ❌ W0 | ⬜ pending |
| 03-T1 | 12-03 | 2 | MODE-04 | unit | `cargo test actions::` (mode_routing_rewrite, mode_routing_translation_returns_none) | ❌ W0 | ⬜ pending |
| 03-T2 | 12-03 | 2 | MODE-04 | unit | `cargo test transcription_coordinator::is_transcribe_binding_smart_mode_prefix` | ❌ W0 | ⬜ pending |
| 03-T3 | 12-03 | 2 | MODE-04 | build | `cargo build` (both init loops compile) | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/src/settings.rs` migration unit tests — v1.2 fixture round-trip (MODE-01): prompt text preserved, shortcut combo transferred, `settings_schema_version` stamped, idempotent re-run
- [ ] v1.2 settings JSON fixture (pristine + edited "Improve Transcriptions" variants) — embedded as test constants or `tests/fixtures/`
- [ ] Default-modes assertion (MODE-02): 10 seeded modes, correct kinds/order, Clean Up active on fresh install
- [ ] CRUD round-trip tests (MODE-03): create→list→update→delete reflected in settings
- [ ] Shortcut-routing unit coverage (MODE-04): `smart_mode_{id}` recognized by `is_transcribe_binding`, conflict returns explicit error

*Existing test patterns in `settings.rs` and `managers/llm.rs` provide the harness; `tempfile` covers store isolation.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live global-shortcut firing registers OS-level hotkey | MODE-04 | OS keyboard-grab requires a running app + real keypress; not unit-testable | Bind a mode to a combo, press it, confirm transcription routes through that mode's prompt |
| tauri-specta `bindings.ts` regenerated with new types | MODE-03 | Generated on app build, not via `cargo test` | Run `bun run tauri dev` (or build), confirm `SmartMode`/`SmartModeKind`/`TargetLanguage` + new commands appear in `src/bindings.ts` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 90s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
