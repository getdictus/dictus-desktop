---
status: diagnosed
phase: 08-privacy-local-first-ux
source:
  - 08-01-SUMMARY.md
  - 08-02-SUMMARY.md
  - 08-03-SUMMARY.md
  - 08-04-SUMMARY.md
started: 2026-05-22T00:00:00Z
updated: 2026-05-22T00:00:00Z
diagnosis_note: |
  Root cause is a design pivot rejected by user, not a code bug — skipped
  parallel debug agents (no investigation needed). Closure split per user
  decision (2026-05-22): UI-only items handled as a Phase 8 sub-phase
  (gap-closure plan); embedded LLM runtime items deferred to a new
  milestone "Local-First Models" promoted ahead of v1.3 Smart Modes.
---

## Current Test

[testing closed — tests 10-14 skipped; design gap on test 9 supersedes them.
Remaining UI tests will be redefined by the Phase 8 gap-closure sub-phase.]

## Tests

### 1. Platform-aware default provider (macOS ARM64)

expected: default_post_process_provider_id() returns "apple_intelligence" on macOS ARM64, "custom" elsewhere
result: pass
verified: cargo test --lib settings::tests — 5/5 passed including default_post_process_provider_id_returns_apple_on_macos_arm64

### 2. Custom (local) provider relabel

expected: settings.rs has label: "Custom (local)" with stable id: "custom"
result: pass
verified: grep at src-tauri/src/settings.rs:602 — `label: "Custom (local)".to_string()`

### 3. docs/PRIVACY.md network surface audit

expected: docs/PRIVACY.md exists listing all 11 outbound endpoint families + 16 CDN URLs
result: pass
verified: file exists (9815 bytes), 28 endpoint references found

### 4. README ## Privacy section

expected: README.md has ## Privacy section linking to docs/PRIVACY.md
result: pass
verified: grep README.md:53 — `## Privacy`

### 5. UPSTREAM maintenance hook

expected: UPSTREAM.md has Privacy / Network Surface Hook with grep check
result: pass
verified: grep UPSTREAM.md:300 — `## Privacy / Network Surface Hook`

### 6. i18n parity across 20 locales

expected: All 14 new Phase 8 keys present in all 19 sibling locales + en (bun run check:translations exits 0)
result: pass
verified: check:translations — all 19 languages have complete translations (413/413 keys each)

### 7. ProviderPicker component exists

expected: src/components/settings/PostProcessingSettingsApi/ProviderPicker.tsx exists with stacked sections, renderRowExtras prop, accent styling
result: pass
verified: file exists; reads as expected — fieldset/legend per section, border-logo-primary bg-logo-primary/10 on selected row, renderRowExtras invoked when checked

### 8. TestConnectionButton component exists

expected: src/components/settings/PostProcessingSettingsApi/TestConnectionButton.tsx exists with success/error Alert
result: pass
verified: file exists at expected path

### 9. ProviderPicker visual structure

expected: Open Settings → Post-processing tab. Provider area shows TWO stacked sections (not a flat dropdown): "On your device" section on top with Apple Intelligence (macOS Apple Silicon only) and Custom (local); "External / cloud" section below with OpenAI, Anthropic, Groq, Cerebras, OpenRouter, Z.AI. Each provider appears as a radio row with label + description.
result: issue
reported: "Cloud providers too visible by default — user wants them hidden behind a toggle. Provider × Model selection feels structurally wrong (X axis). Ollama UX is hostile: 'Custom' + terminal commands is not user-friendly. Vision: local models first, in-app LLM downloader (gemma3:4b recommended), cloud behind a toggle. Mockup provided at mockups/local-first-models-vision.png."
severity: major

### 10. Selected provider accent + Ollama tip

expected: Click "Custom (local)" radio. Selected row gets accent border + tinted background. Inside the selected row (indented under the label) an Ollama tip appears with a clickable link to ollama.com and a Test connection button.
result: skipped
reason: Superseded by Test 9 design gap — Ollama tip + Custom row will be replaced by the new local-models surface in the gap-closure sub-phase.

### 11. Test connection button (success + error)

expected: With Ollama running locally, click Test connection → green success Alert with model count. With Ollama NOT running, red error Alert with collapsible details.
result: skipped
reason: Superseded by Test 9 design gap — TestConnectionButton may be removed or repurposed for advanced/Custom flow under the cloud toggle.

### 12. PostProcessingToggle promoted to own group

expected: Settings → Advanced. The post-processing toggle is in its own dedicated SettingsGroup labelled "Post-processing".
result: skipped
reason: Behaviour will be revisited as part of the page reframe — toggle may move out of Advanced entirely if the new page architecture exposes a single "1 modèle prêt" status.

### 13. About panel Network surface link

expected: Settings → About. A "Network surface" row appears below "Privacy" with a button opening docs/PRIVACY.md on GitHub.
result: skipped
reason: Not covered by Test 9 gap but skipped to avoid partial UAT validation while the design pivot is in flight. To be re-tested once gap-closure sub-phase ships.

### 14. Onboarding has no post-processing surface

expected: Run a clean first-run onboarding. No cloud/API prompting, no provider selection, no API key field. Flow is: model download → permissions → done.
result: skipped
reason: PRIV-03 satisfied by code inspection (Plan 03 SUMMARY confirms zero post_process refs in Onboarding.tsx); skipping interactive re-verification during design pivot.

## Summary

total: 14
passed: 8
issues: 1
pending: 0
skipped: 5

## Gaps

- truth: "Phase 8 frontend surface places local models first with cloud providers as a secondary section, visible by default"
  status: failed
  reason: |
  User reported that the implemented UI is too cloud-friendly and the
  provider × model selection axis is structurally awkward. The desired
  vision (mockup provided) is fundamentally different: 1. Cloud providers MUST be hidden behind an "Activer les modèles cloud"
  toggle (OFF by default) — current implementation shows them in a
  visible second section 2. The page should be reframed as "Modèles et traitement local" with
  a status badge ("1 modèle prêt") and privacy tags (Local, Privé,
  Aucune donnée envoyée, Fonctionne hors ligne) 3. The "Custom (local)" + terminal-instructions Ollama flow is hostile
  UX — Dictus Desktop should embed a local LLM runtime (llama.cpp /
  candle) and provide a Whisper-style downloader for GGUF models
  (recommended default: gemma3:4b, fast + good for translation +
  post-processing) 4. The selected post-processing model should be its own card (separate
  from provider selection); a "Bibliothèque de modèles locaux" section
  lists downloaded models and allows + Ajouter un modèle local
  (drag/drop GGUF or pick file) 5. Three pillars at bottom: Confidentialité par défaut, Contrôle
  utilisateur, Expérience simplifiée
  severity: major
  test: 9
  artifacts:
  - path: ".planning/phases/08-privacy-local-first-ux/mockups/local-first-models-vision.png"
    issue: "Reference mockup — target vision for local-first model UX"
  - path: "src/components/settings/PostProcessingSettingsApi/ProviderPicker.tsx"
    issue: "Current two-section radio picker shows cloud providers by default — needs toggle gating"
  - path: "src/components/settings/post-processing/PostProcessingSettings.tsx"
    issue: "Page structure is provider-first not model-first — needs rearchitecting per mockup"
    missing:
  - "Embedded local LLM runtime (llama.cpp or candle) — connects to memory project_embedded_local_llm_runtime"
  - "In-app GGUF model downloader (Whisper-style) with recommended default gemma3:4b"
  - "Local model library UI: list downloaded models, + Add local model (drag/drop GGUF), use/manage actions"
  - "Cloud toggle gating cloud providers (off by default)"
  - "Page reframing: 'Modèles et traitement local' with status badge + privacy tags"
  - "Three-pillar marketing block at bottom"
    scope_split: |
    Closure split per user decision (2026-05-22):

  IN SCOPE for Phase 8 gap-closure sub-phase (UI-only, current milestone v1.2): 1. Add "Activer les modèles cloud" toggle gating all cloud providers
  (apple_intelligence stays visible as local; OpenAI, Anthropic, Groq,
  Cerebras, OpenRouter, Z.AI hidden until toggle ON) 2. Reframe page title to "Modèles et traitement local" with subtitle
  "Dictus privilégie les modèles locaux pour garantir votre confidentialité" + "En savoir plus" link to docs/PRIVACY.md 3. Add "1 modèle prêt" status badge (top-right of post-processing page) 4. Move selected post-processing model into its own card with privacy
  tags (Local · Privé · Aucune donnée envoyée · Fonctionne hors ligne)
  and a "Recommandé" badge when applicable 5. Add three-pillar marketing block at bottom of page (Confidentialité
  par défaut / Contrôle utilisateur / Expérience simplifiée) 6. i18n: add all new keys to en/translation.json + propagate to 19 locales

  OUT OF SCOPE — deferred to new "Local-First Models" milestone (promoted
  ahead of v1.3 Smart Modes):
  a. "Bibliothèque de modèles locaux" UI (list of downloaded models,
  Utiliser/Gérer/Supprimer actions)
  b. "+ Ajouter un modèle local" — drag/drop GGUF or file picker
  c. Embedded LLM runtime (llama.cpp / candle / mistral.rs choice TBD)
  d. In-app GGUF downloader (Whisper-style UX) with recommended default
  gemma3:4b (~4.3GB) for translation + post-processing
  e. GPU detection per platform (Metal / Vulkan / CUDA)
  f. RAM budget management (Whisper + LLM coexistence)
  g. Quantization presets (Q4/Q5/Q8)
  h. Model unload lifecycle (similar to transcription ModelUnloadTimeout)

  During the gap-closure sub-phase, the "Bibliothèque de modèles locaux"
  section may be shown as a placeholder/coming-soon area, OR omitted
  entirely until the new milestone delivers it — decision deferred to
  the gap-closure planner.
  debug_session: ""
