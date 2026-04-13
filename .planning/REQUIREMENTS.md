# Requirements: Dictus Desktop

**Defined:** 2026-04-10
**Core Value:** L'application doit être identifiable et utilisable comme Dictus Desktop — pas comme Handy.

## v1.1 Requirements

Requirements for milestone v1.1 (Auto-Update & Upstream Sync). Each maps to roadmap phases.

### Auto-Updater

- [x] **UPDT-01**: Developer can generate Ed25519 keypair using `bunx tauri signer generate` and back up private key securely
- [x] **UPDT-02**: Developer can add `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` to GitHub Secrets
- [x] **UPDT-03**: `tauri.conf.json` is configured with Ed25519 public key in `plugins.updater.pubkey`
- [x] **UPDT-04**: `tauri.conf.json` is configured with GitHub Releases `latest.json` endpoint in `plugins.updater.endpoints`
- [x] **UPDT-05**: `tauri.conf.json` has `createUpdaterArtifacts: true` in `bundle` section
- [x] **UPDT-06**: `release.yml` asset-prefix changed from `"handy"` to `"dictus"`
- [x] **UPDT-07**: `build.yml` asset-prefix changed from `"handy"` to `"dictus"`
- [x] **UPDT-08**: `build.yml` has `includeUpdaterJson: true` in tauri-action config
- [x] **UPDT-09**: `UpdateChecker.tsx` fallback URL points to `getdictus/dictus-desktop` releases (not `cjpais/Handy`)
- [x] **UPDT-10**: A dry-run release validates that `latest.json` is generated and accessible on GitHub Releases

### Upstream Sync

- [ ] **SYNC-01**: `.github/workflows/upstream-sync.yml` detects new commits on `cjpais/Handy` main and creates a GitHub issue with commit summary
- [ ] **SYNC-02**: Weekly action is idempotent (no duplicate issues for same upstream state)
- [ ] **SYNC-03**: `UPSTREAM.md` documents fork point (`85a8ed77`), merge-base (`39e855d`), and step-by-step merge process
- [ ] **SYNC-04**: First upstream merge of 4 post-v0.8.2 commits completed on dedicated branch
- [ ] **SYNC-05**: Post-merge checklist verified (identity fields, version, i18n scan, handy.computer scan, cargo build)

## Future Requirements

Deferred to future milestones. Tracked but not in current roadmap.

### Infrastructure

- **INFR-01**: CDN modeles Dictus (remplacer blob.handy.computer)
- **INFR-03**: Code signing Dictus (macOS + Windows)

### Internal Cleanup

- **TECH-01**: Renommage module handy_keys
- **TECH-03**: Cargo binary rename handy→dictus
- **SETT-01**: Sections settings renommees

### Advanced Sync

- **SYNC-A1**: AI-assisted cherry-pick triage (Open Cloud agent)

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Binary rename (handy→dictus) | Risk of breaking macOS permissions and scripts, no user-visible value |
| Code signing | Requires Apple Developer Account, separate effort |
| CDN migration | No Dictus-owned infrastructure yet |
| Settings section renames | Cosmetic, not urgent |
| Delta updates | Complexity not justified at current user scale |
| Auto-merge upstream | Dictus rebrand affects same files upstream changes — human review mandatory |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| UPDT-01 | Phase 4 | Complete |
| UPDT-02 | Phase 4 | Complete |
| UPDT-03 | Phase 4 | Complete |
| UPDT-04 | Phase 4 | Complete |
| UPDT-05 | Phase 4 | Complete |
| UPDT-06 | Phase 4 | Complete |
| UPDT-07 | Phase 4 | Complete |
| UPDT-08 | Phase 4 | Complete |
| UPDT-09 | Phase 4 | Complete |
| UPDT-10 | Phase 4 | Complete |
| SYNC-01 | Phase 5 | Pending |
| SYNC-02 | Phase 5 | Pending |
| SYNC-03 | Phase 5 | Pending |
| SYNC-04 | Phase 5 | Pending |
| SYNC-05 | Phase 5 | Pending |

**Coverage:**
- v1.1 requirements: 15 total
- Mapped to phases: 15
- Unmapped: 0

---
*Requirements defined: 2026-04-10*
*Last updated: 2026-04-10 after roadmap creation*
