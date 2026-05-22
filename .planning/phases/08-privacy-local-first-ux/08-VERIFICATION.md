---
phase: 08-privacy-local-first-ux
verified: 2026-05-22T00:00:00Z
status: gaps_found
score: 5/8 must-haves verified
re_verification: false
gaps:
  - truth: "Apple Intelligence unavailability banner appears inline with the Apple Intelligence provider card"
    status: failed
    reason: "Alert rendered after entire SettingContainer/ProviderPicker block (PostProcessingSettings.tsx lines 156-161), appearing below all cloud providers — visually disconnected from the Apple Intelligence radio row"
    artifacts:
      - path: "src/components/settings/post-processing/PostProcessingSettings.tsx"
        issue: "Apple Intelligence Alert at lines 156-161 is positioned after the full ProviderPicker block (lines 103-154), placing it below Anthropic, Groq, Cerebras, OpenRouter, Z.AI"
    missing:
      - "Inline the Alert inside the Apple Intelligence row using the renderRowExtras prop pattern, or render it immediately below the LOCAL section header — do NOT render at page bottom"

  - truth: "The Ollama link in the Custom provider row is visually identifiable as a link without hovering"
    status: failed
    reason: "Ollama anchor at PostProcessingSettings.tsx line 130 uses className 'text-logo-primary hover:underline cursor-pointer' — underline appears only on hover; in small hint-style caption text the color shift from text-logo-primary is imperceptible"
    artifacts:
      - path: "src/components/settings/post-processing/PostProcessingSettings.tsx"
        issue: "Line 130: className='text-logo-primary hover:underline cursor-pointer' — no underline at rest"
    missing:
      - "Add underline at rest: change to 'text-logo-primary underline underline-offset-2 hover:opacity-80 cursor-pointer' so the link is identifiable without hovering"

  - truth: "The API key field for the Custom (local) provider communicates it is optional"
    status: failed
    reason: "API key field renders with placeholder 'sk-...' for all non-Apple providers including Custom/Ollama (PostProcessingSettings.tsx lines 186-204), implying the field is required; no optional indicator exists for the custom provider path"
    artifacts:
      - path: "src/components/settings/post-processing/PostProcessingSettings.tsx"
        issue: "Lines 186-204: ApiKeyField with placeholder t('settings.postProcessing.api.apiKey.placeholder') = 'sk-...' — rendered unconditionally for custom provider"
      - path: "src/i18n/locales/en/translation.json"
        issue: "apiKey.placeholder = 'sk-...' — no optional-aware variant for custom provider"
    missing:
      - "Either hide the API key field when provider is 'custom', or replace placeholder with an optional-aware string (e.g. '(optional — only needed if your endpoint requires auth)') and style the label as secondary/muted"

  - truth: "The provider area uses a tabs pattern (Local / Cloud) to separate local and cloud providers structurally"
    status: failed
    reason: "ProviderPicker uses a cloud enable/disable toggle (enableCloudProviders prop + onToggleCloudProviders), not a tabs pattern. The Local/Cloud distinction is a toggle interaction, not a structural tab layout. User confirmed this as a design pivot requirement."
    artifacts:
      - path: "src/components/settings/PostProcessingSettingsApi/ProviderPicker.tsx"
        issue: "Cloud providers gated behind enableCloudProviders toggle prop — tabs (Local / Cloud) not implemented"
      - path: "src/components/settings/post-processing/PostProcessingSettings.tsx"
        issue: "Lines 115-117: passes enableCloudProviders and onToggleCloudProviders to ProviderPicker — toggle pattern, not tabs"
    missing:
      - "Replace cloud toggle with a tabs control (Local / Cloud); default active tab = Local; switching tabs hides the other section; add i18n keys for tab labels in all 20 locales"

  - truth: "The three-pillar privacy marketing block (Confidentialite / Controle / Experience) is removed from the post-processing page"
    status: failed
    reason: "Three-pillar grid still rendered at PostProcessingSettings.tsx lines 645-664; pillars.privacy, pillars.control, pillars.simplicity i18n keys still present in all 20 locales"
    artifacts:
      - path: "src/components/settings/post-processing/PostProcessingSettings.tsx"
        issue: "Lines 645-664: three-pillar grid with privacy/control/simplicity cards still rendered"
      - path: "src/i18n/locales/en/translation.json"
        issue: "pillars.privacy, pillars.control, pillars.simplicity keys still present"
    missing:
      - "Delete the three-pillar grid from PostProcessingSettings.tsx; drop pillars.* i18n keys from all 20 locales"

  - truth: "The local model library coming-soon placeholder block appears at the top of the post-processing page"
    status: failed
    reason: "Library coming-soon placeholder rendered at PostProcessingSettings.tsx lines 613-638 — positioned AFTER the Hotkey group (line 600) and API group (line 608), mid-page below the provider picker area; user requested it be moved to top"
    artifacts:
      - path: "src/components/settings/post-processing/PostProcessingSettings.tsx"
        issue: "Lines 613-638: library placeholder SettingsGroup rendered mid-page after hotkey and API sections, not at top of page"
    missing:
      - "Move the library placeholder block to above the model selector / status badge section (top of page), reframing it as a top-of-page teaser"
---

# Phase 8: Privacy / Local-First UX Verification Report

**Phase Goal:** The settings UI and onboarding flow communicate clearly that Dictus is a local-first app — local post-process providers appear before external ones, the network surface is documented, and onboarding copy presents cloud as opt-in.

**Verified:** 2026-05-22T00:00:00Z
**Status:** gaps_found
**Re-verification:** No — initial verification

**Source of gaps:** Authoritative — taken verbatim from 08-05-SUMMARY.md (UAT session, Pierre Viviere, 2026-05-22). 6 failures confirmed by code inspection.

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Platform-aware default provider: Apple Intelligence on macOS ARM64, Custom elsewhere | VERIFIED | settings.rs lines 526+; test at line 995 |
| 2 | Custom provider relabeled to "Custom (local)" with stable id="custom" | VERIFIED | settings.rs line 608: `label: "Custom (local)".to_string()` |
| 3 | docs/PRIVACY.md exists listing all outbound endpoints | VERIFIED | File exists at 9815 bytes / 75 lines; README ## Privacy at line 53 |
| 4 | Onboarding presents local transcription as primary; no cloud post-processing surface | VERIFIED | Code inspection — no post_process refs in Onboarding.tsx (UAT check 7, 14 passed) |
| 5 | ProviderPicker and TestConnectionButton components exist and are wired | VERIFIED | Both files present; imported in PostProcessingSettings.tsx lines 17, 149 |
| 6 | Apple Intelligence unavailability Alert appears inline with its provider card | FAILED | Alert at lines 156-161 rendered below entire ProviderPicker block — page bottom |
| 7 | Ollama link is visually identifiable as a link at rest | FAILED | Line 130: `hover:underline` only — no underline at rest |
| 8 | API key field communicates optional status for Custom (local) provider | FAILED | Placeholder `sk-...` shown unconditionally for Custom; no optional indicator |
| 9 | Provider area uses tabs (Local / Cloud) instead of toggle | FAILED | Toggle pattern implemented; tabs not present |
| 10 | Three-pillar marketing block removed from post-processing page | FAILED | Lines 645-664: grid still rendered; pillars.* i18n keys still present |
| 11 | Library coming-soon placeholder positioned at top of page | FAILED | Lines 613-638: placeholder mid-page below hotkey and API sections |

**Score:** 5/11 truths verified (5 code-confirmed passing, 6 UAT-confirmed failing)

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/src/settings.rs` | Platform-aware default + Custom (local) label + unit tests | VERIFIED | Lines 526, 608, 954+ |
| `docs/PRIVACY.md` | Network surface documentation | VERIFIED | 9815 bytes, 75 lines |
| `README.md` | ## Privacy section with link | VERIFIED | Line 53 |
| `src/components/settings/PostProcessingSettingsApi/ProviderPicker.tsx` | Stacked two-section picker with renderRowExtras | VERIFIED | 114 lines; fieldset/legend per section |
| `src/components/settings/PostProcessingSettingsApi/TestConnectionButton.tsx` | Success/error alert on connection test | VERIFIED | 81 lines |
| `src/components/settings/post-processing/PostProcessingSettings.tsx` | Correct Apple Intelligence Alert placement | STUB | Alert at wrong position (lines 156-161 vs inline in row) |
| `src/components/settings/post-processing/PostProcessingSettings.tsx` | Ollama link with rest-state underline | STUB | `hover:underline` only at line 130 |
| `src/components/settings/post-processing/PostProcessingSettings.tsx` | API key field optional for Custom provider | STUB | `sk-...` placeholder unconditional (lines 197-199) |
| `src/components/settings/PostProcessingSettingsApi/ProviderPicker.tsx` | Tabs (Local / Cloud) pattern | MISSING | Toggle implemented; tabs not implemented |
| `src/components/settings/post-processing/PostProcessingSettings.tsx` | Pillars block removed | STUB | Lines 645-664 still render three-pillar grid |
| `src/components/settings/post-processing/PostProcessingSettings.tsx` | Library placeholder at page top | STUB | Lines 613-638 place placeholder mid-page |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `default_post_process_provider_id()` | `APPLE_INTELLIGENCE_PROVIDER_ID` | `cfg(all(target_os="macos",target_arch="aarch64"))` | WIRED | settings.rs line 526+ |
| `ProviderPicker` | `PostProcessingSettingsApi` | import + JSX usage | WIRED | PostProcessingSettings.tsx line 17, 110 |
| `TestConnectionButton` | `PostProcessingSettingsApi` | import + JSX usage | WIRED | PostProcessingSettings.tsx line 149 |
| Apple Intelligence Alert | ProviderPicker row | `renderRowExtras` prop | NOT WIRED | Alert at lines 156-161 is outside SettingContainer block |
| Ollama link | `openUrl("https://ollama.com")` | onClick handler | PARTIAL | Call exists; but visually invisible to user (no rest underline) |
| API key field | optional UX for custom provider | conditional render or placeholder | NOT WIRED | No optionality signal for custom provider |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| PRIV-01 | 08-01, 08-03, 08-04 | Local providers rendered above external; platform-aware default | PARTIAL | Backend default + relabel VERIFIED; frontend visual ordering correct; but inline Alert (gap 1a), link style (gap 2a), API key UX (gap 2b), and tabs pattern (gap 5) are unresolved |
| PRIV-02 | 08-02, 08-03, 08-04 | docs/PRIVACY.md with all outbound endpoints + README link | VERIFIED | File exists, README linked, UPSTREAM hook added |
| PRIV-03 | 08-03 | Onboarding presents local as primary, cloud as opt-in | VERIFIED | No cloud post-process surface in onboarding (UAT check 7, 14) |

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `PostProcessingSettings.tsx` | 156-161 | Alert rendered outside provider row context | Blocker | Apple Intelligence error appears below all cloud providers — confusing position |
| `PostProcessingSettings.tsx` | 130 | `hover:underline` only on Ollama link | Blocker | User cannot find the Ollama link without prior knowledge of hover interaction |
| `PostProcessingSettings.tsx` | 186-204 | API key field with `sk-...` placeholder for Custom provider | Blocker | Implies API key is required for local Ollama usage — contradicts local-first UX goal |
| `PostProcessingSettings.tsx` | 645-664 | Three-pillar marketing grid | Warning | User-confirmed visual noise; design pivot requires removal |
| `PostProcessingSettings.tsx` | 613-638 | Library placeholder mid-page | Warning | User-requested repositioning to top of page |
| `ProviderPicker.tsx` | all | Toggle pattern for cloud providers | Warning | User-directed design pivot to tabs (Local / Cloud) |

---

### Human Verification Required

None — all gaps were identified via live UAT (Pierre Viviere, 2026-05-22). The 6 failures are authoritative user-reported outcomes. No additional human verification is needed before gap closure; gap-closure plan output should be verified via a follow-up UAT.

---

### Gaps Summary

**6 gaps block Phase 8 from shipping.** All 6 were confirmed by the product owner during live UAT on 2026-05-22 (macOS Apple Silicon, dev mode).

**3 bugs (targeted, low-risk fixes):**

- Gap 1a: Apple Intelligence unavailability Alert positioned at page bottom instead of inline with the Apple Intelligence provider row. Fix: use `renderRowExtras` prop.
- Gap 2a: Ollama link has `hover:underline` only — invisible at rest. Fix: add `underline underline-offset-2` to className.
- Gap 2b: API key field shows `sk-...` placeholder for Custom/Ollama provider, implying the key is required when it is optional at the backend level. Fix: hide field or replace placeholder with optional-aware text.

**3 design pivots (larger scope, require ProviderPicker restructuring and i18n changes):**

- Gap 5: Replace cloud toggle with a Local/Cloud tabs control. Default: Local tab. Cloud tab shows cloud providers. Associated i18n keys needed in all 20 locales.
- Gap 6: Remove the three-pillar privacy marketing block (Confidentialite / Controle / Experience) from PostProcessingSettings.tsx and drop `pillars.*` i18n keys from all 20 locales.
- Gap 7: Move the "Bibliotheque de modeles locaux" coming-soon placeholder from its current mid-page position (after hotkey and API sections) to the top of the post-processing page.

**Root cause grouping:** Gaps 5, 6, and 7 all affect `PostProcessingSettings.tsx` page architecture and require coordinated changes with i18n propagation. Bugs 1a, 2a, 2b are self-contained fixes in `PostProcessingSettings.tsx` and the English i18n file, with locale propagation as a follow-on step.

---

_Verified: 2026-05-22_
_Verifier: Claude (gsd-verifier) — gaps sourced from authoritative UAT outcome in 08-05-SUMMARY.md_
