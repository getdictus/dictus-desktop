# Roadmap: Dictus Desktop

## Milestones

- ✅ **v1.0 Handy→Dictus Rebrand** — Phases 1-3 (shipped 2026-04-10) — [archive](milestones/v1.0-ROADMAP.md)
- 🚧 **v1.1 Auto-Update & Upstream Sync** — Phases 4-5 (in progress)

## Phases

<details>
<summary>✅ v1.0 Handy→Dictus Rebrand (Phases 1-3) — SHIPPED 2026-04-10</summary>

- [x] Phase 1: Bundle Identity (1/1 plans) — completed 2026-04-05
- [x] Phase 2: Visual Rebrand (5/5 plans) — completed 2026-04-09
- [x] Phase 3: Documentation and Cleanup (2/2 plans) — completed 2026-04-09

</details>

### 🚧 v1.1 Auto-Update & Upstream Sync (In Progress)

**Milestone Goal:** L'app peut se mettre à jour toute seule et on ne décroche pas du upstream Handy.

- [ ] **Phase 4: Updater Infrastructure** - Wire Ed25519 keypair, tauri.conf.json config, CI fixes, and URL fix; validate with dry-run release
- [ ] **Phase 5: Upstream Sync** - Weekly detection action, UPSTREAM.md, 4-commit merge, post-merge checklist

## Phase Details

### Phase 4: Updater Infrastructure
**Goal**: The auto-updater is fully wired — Dictus can sign, publish, and deliver updates to users automatically
**Depends on**: Phase 3 (v1.0 shipped)
**Requirements**: UPDT-01, UPDT-02, UPDT-03, UPDT-04, UPDT-05, UPDT-06, UPDT-07, UPDT-08, UPDT-09, UPDT-10
**Success Criteria** (what must be TRUE):
  1. Running `bunx tauri signer generate` produces an Ed25519 keypair and the private key is backed up securely
  2. `tauri.conf.json` contains a valid `pubkey`, `endpoints` URL pointing to `getdictus/dictus-desktop` releases, and `createUpdaterArtifacts: true`
  3. `release.yml` and `build.yml` both use `asset-prefix: "dictus"` and `build.yml` has `includeUpdaterJson: true`
  4. `UpdateChecker.tsx` fallback URL references `getdictus/dictus-desktop` (not `cjpais/Handy`)
  5. A dry-run release produces a `latest.json` file that is accessible on GitHub Releases
**Plans**: 4 plans
- [ ] 04-01-PLAN.md — Wave 0 infra: runbook + validator script + UPDT-10 curl wrapper (Wave 1)
- [ ] 04-02-PLAN.md — CI text fixes: asset-prefix, includeUpdaterJson, TECH-03 comment, UpdateChecker URL (Wave 1)
- [ ] 04-03-PLAN.md — Keypair generation + GitHub Secrets + tauri.conf.json updater config (Wave 2, depends on 04-01)
- [ ] 04-04-PLAN.md — Dry-run release trigger + UPDT-10 curl assertion (Wave 3, depends on 04-01/02/03)

### Phase 5: Upstream Sync
**Goal**: New upstream Handy commits are detected automatically each week and the current delta (4 commits) is merged into a dedicated branch with identity integrity verified
**Depends on**: Phase 4
**Requirements**: SYNC-01, SYNC-02, SYNC-03, SYNC-04, SYNC-05
**Success Criteria** (what must be TRUE):
  1. `.github/workflows/upstream-sync.yml` runs weekly and creates a GitHub issue listing new upstream commits when the state has changed
  2. Running the action twice with the same upstream state creates no duplicate issues
  3. `UPSTREAM.md` documents fork point `85a8ed77`, merge-base `39e855d`, and a step-by-step merge process a developer can follow
  4. A dedicated branch contains the 4 post-v0.8.2 upstream commits merged into main
  5. Post-merge grep confirms `productName`, `identifier`, and all i18n strings remain Dictus (no Handy regressions)
**Plans**: TBD

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Bundle Identity | v1.0 | 1/1 | Complete | 2026-04-05 |
| 2. Visual Rebrand | v1.0 | 5/5 | Complete | 2026-04-09 |
| 3. Documentation and Cleanup | v1.0 | 2/2 | Complete | 2026-04-09 |
| 4. Updater Infrastructure | 1/4 | In Progress|  | - |
| 5. Upstream Sync | v1.1 | 0/? | Not started | - |
