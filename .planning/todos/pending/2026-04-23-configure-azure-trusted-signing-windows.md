---
created: 2026-04-23T22:55:00.000Z
title: Configure Azure Trusted Signing for Windows code signing
area: release-infrastructure
files:
  - .github/workflows/build.yml
  - .github/workflows/release.yml
  - docs/RUNBOOK-updater-signing.md
---

## Problem

Windows `.msi` and `.exe` installers produced by the release workflow are **not code-signed**. The plumbing exists in `.github/workflows/build.yml` (trusted-signing-cli is installed on Windows runners, AZURE_* env vars are wired into the `tauri-apps/tauri-action@v0` step), but the three required GitHub Secrets are missing:

- `AZURE_CLIENT_ID`
- `AZURE_CLIENT_SECRET`
- `AZURE_TENANT_ID`

Confirmed via `gh secret list --repo getdictus/dictus-desktop` on 2026-04-23 — only `APPLE_*`, `KEYCHAIN_PASSWORD`, and `TAURI_SIGNING_PRIVATE_KEY*` are present.

## User impact on unsigned Windows builds

1. **SmartScreen warning** ("Windows protected your PC" blue screen) on download — user must click "More info" → "Run anyway"
2. **UAC shows "Unknown publisher"** in yellow instead of verified "Dictus"
3. Some antivirus products may flag unsigned installers as suspicious
4. Estimated 30–50% drop-off at the SmartScreen gate for general audience, 10–20% for tech-savvy early adopters

This was acceptable for v0.1.2 soft-launch (2026-04-23) but **must be fixed before any broader public communication**.

## Interim mitigation (v0.1.2)

Release notes and README must call out the SmartScreen friction and explain the workaround to Windows users. This is a soft-launch concession — not a long-term solution.

## Solution

### Prerequisites (Azure side — takes 1-7 days)

1. Create / verify a Microsoft Partner Center account tied to a legal entity (Pierre solo or via a company)
2. Subscribe to Azure Trusted Signing (~$10/month)
3. Complete Microsoft identity verification — this is the step that gates everything else
4. Create an Azure AD app registration (Service Principal) with access to the Trusted Signing account
5. Generate `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `AZURE_TENANT_ID`

### GitHub side (takes ~10 minutes once Azure is ready)

1. Add the 3 Azure secrets to `getdictus/dictus-desktop` secrets
2. Verify `build.yml` correctly picks them up on next release
3. Ship a test release (e.g. v0.1.2.1 or v0.1.3) and verify on Windows that the installer shows "Dictus" as verified publisher and SmartScreen stops warning
4. Update `docs/RUNBOOK-updater-signing.md` with an "Azure Trusted Signing" section documenting custody, rotation, and recovery — mirror the Apple section

### Alternative considered (and rejected for now)

**EV certificate from DigiCert / Sectigo** (~$300-500/year, no reputation delay, works immediately) — faster to stand up but more expensive long-term and less integrated than Azure. Revisit only if Azure Partner Center onboarding drags past 2 weeks.

## Priority / timing

- **Blocks:** broader Dictus public launch — must be done before mainstream communication
- **Does NOT block:** v0.1.2 soft-launch to early adopters
- **Good candidate for a dedicated GSD phase** given runbook work + Azure onboarding + test release cycle

## Definition of done

- Windows installer downloaded from a published release shows no SmartScreen warning
- UAC prompt shows "Dictus" (or legal-entity name) as verified publisher
- `docs/RUNBOOK-updater-signing.md` has an "Azure Trusted Signing" section mirroring the existing Apple procedures
- This todo moves to `done/`
