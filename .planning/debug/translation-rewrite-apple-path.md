---
status: diagnosed
trigger: "la traduction ne se fait pas — custom REWRITE mode 'Traduction Espagnole' (prompt 'Translate the following text to Spanish. Return ONLY...') produced no translation"
created: 2026-06-05T00:00:00Z
updated: 2026-06-05T00:00:00Z
symptoms_prefilled: true
goal: find_root_cause_only
---

## Current Focus

hypothesis: A REWRITE-kind smart mode runs through the LLM post-processing provider (apple_intelligence for this user). Either (a) the rewrite prompt lacks the ${output} transcript token so the model never receives the text, OR (b) Apple Intelligence "structured outputs" constrains/truncates the free-text output (26-char bullet evidence).
test: Read actions.rs dispatch + prompt substitution + apple_intelligence structured-outputs path
expecting: Determine which path the rewrite-translation took and why it produced nothing
next_action: Read src-tauri/src/actions.rs

## Symptoms

expected: A translation smart mode runs and outputs translated text
actual: Custom REWRITE mode "Traduction Espagnole" (prompt "Translate the following text to Spanish. Return ONLY...") produced no translation
errors: none surfaced to user; produces no/empty output
reproduction: UAT Test 10 — create a custom rewrite mode with a translate prompt, run it with apple_intelligence as active provider
started: Phase 13 UAT re-test
evidence_from_logs: |
  - 14:08:06 dedicated Translation mode (binding=smart_mode_mode_translate_es) → LLM inference prompt=168 chars → output 53 chars "La traducción se hará con el modelo predeterminado." (WORKS via embedded generic LLM)
  - 14:07:50 Bullet Points REWRITE mode → provider 'apple_intelligence' + "Using structured outputs" → "Apple Intelligence post-processing succeeded. Output length: 26 chars" (truncated/wrong)

## Eliminated

- hypothesis: The rewrite prompt lacks ${output} so the transcript is never received by the model.
  evidence: build_system_prompt (actions.rs:73-82) STRIPS ${output} on purpose and the transcript is passed SEPARATELY as user_content (actions.rs:217, 231). The Apple path calls process_text_with_system_prompt(&system_prompt, &user_content). The transcript IS delivered regardless of whether the prompt contains ${output}. So a missing token is NOT the cause of "no translation". (It is still a real UX gap for the embedded legacy path, tracked separately under UAT gap test 4, but not THIS failure.)
  timestamp: 2026-06-05

## Evidence

- timestamp: 2026-06-05
  checked: actions.rs process_transcription_output → resolve_mode_prompt (lines 482-526, 973-985)
  found: A REWRITE-kind mode resolves via resolve_mode_prompt → Some((mode, prompt)) and runs through post_process_with_prompt. It does NOT go through run_translation. run_translation is reached ONLY for SmartModeKind::Translation. So "Traduction Espagnole" (a Rewrite mode) never touches the working embedded-LLM translation path.
  implication: The reported failure is entirely on the rewrite/post-process provider path, which for this user is apple_intelligence.

- timestamp: 2026-06-05
  checked: post_process_with_prompt provider branch (actions.rs:213-261) + settings.rs:617
  found: apple_intelligence has supports_structured_output: true, so the rewrite runs the structured-output branch. For apple_intelligence it calls apple_intelligence::process_text_with_system_prompt(system_prompt, user_content, token_limit). token_limit = model.trim().parse::<i32>().unwrap_or(0) — i.e. the per-provider "model" string for apple_intelligence is interpreted as a WORD-COUNT cap (actions.rs:230).
  implication: If the apple_intelligence "model" setting holds a small number, output is hard-truncated by word count.

- timestamp: 2026-06-05
  checked: swift/apple_intelligence.swift processTextWithSystemPrompt (lines 5-129)
  found: TWO defects. (1) STRUCTURED-OUTPUT SCHEMA MISMATCH: it ALWAYS tries session.respond(to: userContent, generating: CleanedTranscript.self) where @Generable CleanedTranscript { let cleanedText: String } (lines 5-9, 99-103). The Foundation Models guided-generation is biased toward "cleaning a transcript", not translating/rewriting. The @Generable struct's field name/semantics ("cleanedText") and Dictus' JSON schema field name ("transcription", actions.rs:264-274) are unrelated and ignored by the native Swift path. For a translate-to-Spanish instruction the on-device model frequently returns the text essentially unchanged or near-empty because the schema frames the task as transcript cleanup. (2) WORD-COUNT TRUNCATION: truncatedText (lines 25-36, 109-111) cuts output to `tokenLimit` WORDS when tokenLimit>0. The 26-char bullet result is consistent with a tiny word cap and/or the schema returning a near-empty cleanedText.
  implication: Free-form rewrite/translation prompts run through Apple Intelligence are constrained by a transcript-cleanup @Generable schema (semantic mismatch) and an unrelated word-count cap. This explains BOTH the "no translation" report (schema frames task as cleanup → text passes through ≈ untranslated/empty) AND the separately-reported 26-char bullet truncation.

- timestamp: 2026-06-05
  checked: dictus.log evidence vs code paths
  found: 14:08:06 smart_mode_mode_translate_es went through run_translation + embedded generic LLM → correct Spanish (53 chars). 14:07:50 Bullet Points REWRITE → provider apple_intelligence + "Using structured outputs" → "Output length: 26 chars". The two paths are exactly the resolve_mode_prompt branch split confirmed above.
  implication: Translation works ONLY on the dedicated Translation-kind + embedded path. A user who builds translation as a custom Rewrite mode while apple_intelligence is the active provider gets the broken Apple structured-output path.

## Resolution

root_cause: |
  The failing "Traduction Espagnole" is a REWRITE-kind smart mode, not a Translation-kind mode. Rewrite modes route through post_process_with_prompt (actions.rs:503-504), which dispatches to the user's ACTIVE post-process provider — apple_intelligence for this user — NOT through run_translation/the embedded translation path that works.

  apple_intelligence has supports_structured_output: true (settings.rs:617), so the rewrite enters the structured-output branch (actions.rs:213-261) and calls the native Swift processTextWithSystemPrompt. That Swift function ALWAYS attempts guided generation against a hardcoded @Generable struct `CleanedTranscript { let cleanedText: String }` (swift/apple_intelligence.swift:5-9, 99-103). This schema frames every task as "clean a transcript", so a "Translate to Spanish" instruction yields the input essentially unchanged or near-empty — i.e. "the translation does not happen". The same Swift function also word-count-truncates output via truncatedText when the apple_intelligence "model" setting (parsed as a word cap at actions.rs:230) is small — this is the 26-char bullet truncation.

  The transcript token (${output}) is NOT the cause: build_system_prompt deliberately strips ${output} and the transcript is sent separately as user_content, so the model always receives the text. Token-missing is a real but SEPARATE UX gap (UAT test 4), not this failure.

  Net: it is the apple-intelligence-structured-outputs path (hardcoded transcript-cleanup @Generable schema + word-count truncation), NOT a missing token.
fix: ""
verification: ""
files_changed: []

suggested_fix_direction: |
  Primary (root cause of "no translation"): make the Apple Intelligence native path task-agnostic. Either
  (a) drop the hardcoded CleanedTranscript @Generable schema and use plain free-text session.respond(to:) (with system instructions carrying the user's prompt) so translate/rewrite/bullets all work; or
  (b) only use guided generation for the built-in Clean Up mode and fall back to free-text for any other prompt; or
  (c) rename/redesign the @Generable field to a neutral "result"/"text" and verify on-device translation actually transforms text under guided generation (likely still worse than free-text for non-cleanup tasks).
  Recommended: (a) plain free-text response for the smart-mode/rewrite path; reserve structured output for cases that truly need JSON.

  Secondary (the 26-char truncation): stop interpreting the apple_intelligence "model" string as a word-count cap, or default it to 0/unlimited for smart modes. The word-count truncatedText cap (swift lines 25-36) silently mangles legitimate longer output. At minimum default token_limit to 0 (no cap) for rewrite/translation modes (actions.rs:230).

  Also worth doing (product decision): a Translation built as a custom Rewrite mode should arguably be steered to the working embedded translation path, or the picker should not let users hand-roll translation as a rewrite prompt while a structured-output provider is active. But the minimal correct fix is making the Apple path honor arbitrary prompts.
