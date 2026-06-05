---
status: diagnosed
trigger: "Test 11 UAT — (1) translation engine choice 'use current model' does not persist when active model is Apple Foundation; modal reopens showing Gemma. (2) Apple Intelligence bullet post-processing truncated to 26 chars."
created: 2026-06-05T00:00:00Z
updated: 2026-06-05T00:00:00Z
symptoms_prefilled: true
goal: find_root_cause_only
---

## Current Focus

hypothesis: CONFIRMED BOTH. (1) The modal reads "which engine is active" from llmModelStore.activeModelId (GGUF LLM id), NOT from the persisted translation_engine_choice. When Apple Intelligence is the active post-process *provider*, activeModelId is null (Apple is not a GGUF model entry), so "use current" badge never shows. The write DOES persist generic_model, but the modal can't reflect it. (2) Apple structured path hardcodes @Generable CleanedTranscript{cleanedText} + includeSchemaInPrompt default true → biases the on-device model to a short "cleaned text" value, ignoring the bullets prompt → 26-char output. Rust token_limit truncation is NOT the cause (default model string "Apple Intelligence" parses to 0 = no truncation).
test: traced modal state derivation + apple swift structured generation path
expecting: done
next_action: write resolution, return diagnosis

## Symptoms

expected: Selecting translation engine ("use current/active model"=generic, or dedicated Gemma) persists; reopening modal reflects chosen engine. Post-processing output not truncated.
actual: With Apple Foundation/Apple Intelligence active, choosing "use current model" then reopening modal still shows Gemma activated; choice does not persist. Bullet post-processing returned only 26 chars (truncated). Embedded LLM (Qwen) works fine both ways.
errors: "Apple Intelligence post-processing succeeded. Output length: 26 chars"
reproduction: Test 11 in UAT, active model = Apple Foundation/Apple Intelligence
started: Phase 13 UAT re-test

## Eliminated

- hypothesis: "(2) Rust word-count truncation (actions.rs:230 token_limit / Swift truncatedText) cuts the output to 26 chars"
  evidence: "Apple provider default model field = APPLE_INTELLIGENCE_DEFAULT_MODEL_ID = the string \"Apple Intelligence\". actions.rs:230 `model.trim().parse::<i32>().unwrap_or(0)` → parse fails → token_limit=0. Swift truncatedText (apple_intelligence.swift:25) `guard limit > 0 else { return text }` → returns text UNCHANGED when 0. So no Rust/Swift word truncation occurs. The 26 chars is what the model actually produced."
  timestamp: 2026-06-05

- hypothesis: "(2) GenerationOptions.maximumResponseTokens default caps response to ~26 chars"
  evidence: "Per FoundationModels docs, when maximumResponseTokens is unset the model generates up to its context limit (no short artificial cap). The Swift code passes NO GenerationOptions to session.respond, so no token cap applies."
  timestamp: 2026-06-05

- hypothesis: "(1) the setTranslationEngineChoice write is skipped/gated when active_llm_model_id is empty"
  evidence: "enableWith() (modal:65) always calls commands.setTranslationEngineChoice('generic_model') unconditionally; command (llm.rs:77) writes settings.translation_engine_choice with no gating. The write succeeds and persists. The bug is purely in how the modal DISPLAYS the active engine on reopen, not in persistence."
  timestamp: 2026-06-05

## Evidence

- timestamp: 2026-06-05
  checked: TranslationEngineChoiceModal.tsx lines 27-62
  found: "Modal derives all active-engine UI from useLlmModelStore: activeModelId (s.activeModelId), recActive = enabled && activeModelId === 'gemma-3-4b' (line 56), currentActiveSelected = enabled && activeModelId != null && activeModelId !== 'gemma-3-4b' (line 61-62). It NEVER reads settings.translation_engine_choice to decide which badge is shown."
  implication: "Active-engine display is inferred from the GGUF LLM active id, not from the persisted choice. Any provider that is not a GGUF LLM (Apple Intelligence, cloud) leaves activeModelId null."

- timestamp: 2026-06-05
  checked: llmModelStore.ts refresh() lines 51-68
  found: "activeModelId = commands.getActiveLlmModel() → settings.active_llm_model_id (llm.rs:70-73). This is ONLY the embedded GGUF model id. Apple Intelligence is a post_process provider (settings.rs:611-618, id 'apple_intelligence'), NOT an LLM model — it never sets active_llm_model_id."
  implication: "With Apple Intelligence as the active provider, activeModelId is null → recActive=false AND currentActiveSelected=false → neither option shows an 'active' badge. The Gemma card shows as the default/recommended (border on recActive only, but the user perceives Gemma as 'still marked'). The persisted generic_model choice is invisible."

- timestamp: 2026-06-05
  checked: SmartModesSection.tsx lines 93-103, 174-189
  found: "The section header correctly reads translationEnabled from getSetting('translation_engine_choice') (persisted, line 93-94) and shows the 'current engine' label using activeModelName ?? engineGeneric (line 177-179). So the SECTION knows the choice persisted; only the MODAL mis-derives the active badge."
  implication: "Confirms persistence works (translation runs, header updates). The user-visible 'still Gemma' is the modal's badge logic, decoupled from the persisted choice."

- timestamp: 2026-06-05
  checked: actions.rs lines 213-261 (structured-output branch + Apple handling)
  found: "When provider.supports_structured_output is true (Apple Intelligence = true, settings.rs:617), the Apple branch (220-261) calls apple_intelligence::process_text_with_system_prompt(system_prompt, user_content, token_limit). token_limit from model.parse (=0 here, no effect)."
  implication: "Apple path goes through structured Swift generation, distinct from the embedded LLM path that worked (Qwen)."

- timestamp: 2026-06-05
  checked: apple_intelligence.swift lines 5-9, 96-107
  found: "Hardcoded `@Generable private struct CleanedTranscript { let cleanedText: String }`. session.respond(to: userContent, generating: CleanedTranscript.self) with default includeSchemaInPrompt:true → Apple injects the schema (single field named 'cleanedText', described implicitly as cleaned transcript) into the prompt. On any structured success, output = structured.content.cleanedText. Fallback to plain respond only on throw."
  implication: "The schema is hardcoded to a 'cleaned transcript' shape regardless of the actual smart-mode prompt. For a Bullet Points mode (prompt 'Restructure this text as a concise bulleted list...', settings.rs:706), the model is told (via injected schema) to fill a single 'cleanedText' field — biasing it to a terse value and fighting the multi-line bullet instruction. Structured generation succeeds and returns a 26-char value. The mismatch between the fixed schema name/intent and the mode's real prompt is the truncation mechanism."

- timestamp: 2026-06-05
  checked: git log apple_intelligence.swift / actions.rs
  found: "Structured outputs landed via upstream PR #706 ('implement structured outputs for post-processing providers'); Apple integration via #391. The CleanedTranscript schema predates Smart Modes — it was designed for the single 'cleanup' use case, never updated for arbitrary rewrite/bullets/translation prompts."
  implication: "The Apple structured path was never adapted to Smart Modes' free-form prompts; it assumes a cleanup task."

## Resolution

root_cause: |
  TWO independent root causes, both specific to the Apple Intelligence (non-GGUF) provider path.

  (1) ENGINE-CHOICE PERSISTENCE — IT ACTUALLY PERSISTS; THE MODAL MIS-DISPLAYS IT.
  setTranslationEngineChoice('generic_model') is written and persisted correctly
  (commands/llm.rs:77-85), and SmartModesSection reads it correctly. But
  TranslationEngineChoiceModal decides which engine is "active" by inspecting
  useLlmModelStore.activeModelId (the embedded GGUF model id), NOT the persisted
  translation_engine_choice. Apple Intelligence is a post-process *provider*, not a
  GGUF LLM, so activeModelId is null whenever Apple is active. With activeModelId
  null: recActive=false and currentActiveSelected=false, so the "use current model"
  option never gets the active badge and the modal looks like nothing was chosen /
  Gemma is still the default. The choice is invisible, not unpersisted. (With the
  embedded Qwen model active, activeModelId IS a real GGUF id, so currentActiveSelected
  works — which is exactly why it "works with Qwen but not Apple Foundation".)

  (2) APPLE INTELLIGENCE TRUNCATION (26 chars) — HARDCODED STRUCTURED SCHEMA, NOT A TOKEN CAP.
  The Apple path (actions.rs:220-261) routes through Swift structured generation with a
  hardcoded `@Generable CleanedTranscript { let cleanedText: String }` and the default
  includeSchemaInPrompt:true. Apple injects that single-field "cleaned transcript" schema
  into the prompt, which biases the on-device model to emit a short cleaned-text value and
  ignore the actual Smart-Mode prompt (e.g. Bullet Points). Structured generation succeeds
  and returns the terse 26-char value (output = structured.content.cleanedText). It is NOT
  the Rust word-limit (token_limit=0 → no-op) nor maximumResponseTokens (never set). The
  CleanedTranscript schema was built for the original single cleanup use case and was never
  generalized to arbitrary Smart-Mode prompts.

fix: |
  (1) Make the modal reflect the PERSISTED translation_engine_choice, not the GGUF activeModelId.
  Read translation_engine_choice (and the active provider id) and:
    - show "use current model" as active whenever choice == generic_model AND the active
      engine is NOT the recommended Gemma model (covers Apple Intelligence, cloud, custom,
      and non-Gemma embedded models alike);
    - stop requiring activeModelId != null / activeDownloaded for the "use current" badge
      and the enable button when the active engine is a non-GGUF provider like Apple.
  Pass the active provider id / a resolved active-engine descriptor into the modal so it
  doesn't depend solely on llmModelStore. Suggested: have SmartModesSection (which already
  knows engineChoice + active provider) own the "active engine" determination and pass it down.

  (2) Stop forcing the hardcoded CleanedTranscript schema for Apple Intelligence Smart Modes.
  Options (pick one):
    - Simplest/most robust: for the Apple path, do NOT use structured generation — call the
      plain session.respond(to:) with the full prompt (the embedded LLM path already works
      this way and produced correct bullets with Qwen). i.e. skip the @Generable schema, or
      gate it to only the cleanup mode.
    - Or pass includeSchemaInPrompt:false and/or a schema whose field is described as the
      verbatim post-processed output (not "cleaned transcript"), so the model isn't biased.
  Note: Apple has supports_structured_output:true (settings.rs:617) which forces it down the
  structured branch in actions.rs:213; consider routing Apple through a non-structured branch
  for Smart-Mode (free-form) prompts.

verification: |
  NOT VERIFIED (diagnose-only mode, goal: find_root_cause_only). No fix applied.
  Suggested verification once fixed:
  (1) With Apple Intelligence active: open modal → choose "use current model" → reopen →
      "use current model" shows the active badge (not Gemma). Repeat after Qwen switch to
      confirm no regression.
  (2) With Apple Intelligence active: run Bullet Points mode on a multi-sentence dictation →
      full multi-line bullet list returned (not 26 chars). Check log "Apple Intelligence
      post-processing succeeded. Output length: N chars" with N proportional to input.
files_changed: []

