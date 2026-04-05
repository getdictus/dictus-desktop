# Phase 1: Bundle Identity - Context

**Gathered:** 2026-04-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Set bundle ID, product name, and neutralize inherited Handy build artifacts before any distributed build. The application must identify itself as Dictus Desktop at the OS and build level. No internal code renaming (handy_app_lib, handy-keys, binary name) — those are deferred to V2 (TECH-01–04).

</domain>

<decisions>
## Implementation Decisions

### Version number
- Reset to 0.1.0 in both tauri.conf.json and Cargo.toml
- Fresh start — signals new product, leaves room for 1.0.0 as a meaningful milestone later

### Updater handling
- Set `createUpdaterArtifacts: false` in tauri.conf.json
- Remove `endpoints` array and `pubkey` from updater plugin config (leave empty object)
- Keep `tauri-plugin-updater` dependency in Cargo.toml — easy to re-enable when Dictus has its own release infra (INFR-02)

### Windows signing
- Remove `signCommand` field entirely from `bundle.windows` config
- No placeholder — clean removal until Dictus gets its own signing setup (INFR-03, V2)

### Cargo.toml authorship
- `authors = ["Dictus", "cjpais"]` — both listed, fork attribution in manifest
- `description` updated to describe Dictus Desktop
- `name`, `default-run`, `lib.name` stay as-is (deferred to TECH-01/TECH-03)

### Claude's Discretion
- Exact wording of Cargo.toml description field
- Whether to add a `repository` field pointing to Dictus repo
- Order of fields in tauri.conf.json after edits

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Bundle configuration
- `src-tauri/tauri.conf.json` — Current Handy identity: productName, identifier, updater endpoints, Windows signCommand
- `src-tauri/Cargo.toml` — Current Handy metadata: name, description, authors, default-run, lib.name

### Project requirements
- `.planning/REQUIREMENTS.md` — BNDL-01 through BNDL-05 define exact acceptance criteria
- `.planning/ROADMAP.md` — Phase 1 success criteria (5 conditions that must be TRUE)
- `.planning/PROJECT.md` — Key decisions section for init-time decisions (updater disable, clean-slate install, bundle ID)

</canonical_refs>

<code_context>
## Existing Code Insights

### Files to modify
- `src-tauri/tauri.conf.json` — productName, identifier, bundle.createUpdaterArtifacts, plugins.updater, bundle.windows.signCommand
- `src-tauri/Cargo.toml` — package metadata (version, description, authors) — NOT name, default-run, or lib.name (V2 scope)

### Established Patterns
- Tauri 2.x config structure with `$schema` reference
- Cargo.toml uses platform-specific dependency sections (`[target.'cfg(...)'.dependencies]`)

### Integration Points
- `productName` affects OS window title, tray menu label, macOS dock name
- `identifier` affects OS data paths (app data directory) — clean-slate install decided, no migration needed
- `createUpdaterArtifacts` controls whether build produces .sig and latest.json files

### Deferred touchpoints (V2 — DO NOT modify)
- `name = "handy"` and `default-run = "handy"` in Cargo.toml (TECH-03)
- `lib.name = "handy_app_lib"` in Cargo.toml (TECH-01)
- `[patch.crates-io]` references to `cjpais/tauri.git` (TECH-04)
- `handy-keys` dependency (TECH-01)

</code_context>

<specifics>
## Specific Ideas

No specific requirements — decisions are configuration-focused and well-defined by requirements.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 01-bundle-identity*
*Context gathered: 2026-04-05*
