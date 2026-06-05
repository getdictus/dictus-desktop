---
status: diagnosed
trigger: "Test 4 UAT: picker adds duplicates with no distinction; no overwrite warning; Custom form has no placeholders and no ${output} hint"
created: 2026-06-05T13:00:00Z
updated: 2026-06-05T13:20:00Z
---

## Current Focus

hypothesis: (a/b) Picker duplicate detection is broken because picker-created modes get a fresh timestamp id, never the template's seeded id, so the `existingModeIds.includes(template.id)` check can never match a picker copy. (c) Custom form has no placeholders and no `${output}` hint.
test: Read SmartModeTemplatePicker.tsx, SmartModeCard.tsx, settings.rs templates, shortcut/mod.rs add_smart_mode, actions.rs token consumption.
expecting: confirm id mismatch and absence of placeholders/hint.
next_action: diagnosis complete — return findings.

## Symptoms

expected: (1) picker disables/marks/warns when a template already present (uniqueness by seeded id or name); (2) Custom form has placeholders on name+prompt and an example prompt demonstrating ${output}.
actual: (a) can add an already-present mode with no visual distinction; (b) re-importing an edited default does not warn it exists / will overwrite; (c) Custom form has no placeholders and no indication that ${output} must be included.
errors: none
reproduction: Test 4 in .planning/phases/13-smart-modes-ui-translation-presets-i18n/13-UAT.md
started: Discovered during Phase 13 UAT re-test

## Eliminated

(none — direct read confirmed the mechanism)

## Evidence

- checked: src/components/settings/post-processing/SmartModeTemplatePicker.tsx
  found: line 83 `const alreadyAdded = existingModeIds.includes(template.id)`. The "added" hint is the ONLY duplicate handling. handlePickTemplate (39-51) calls `commands.addSmartMode(...)` unconditionally — no guard, no warn-before-overwrite. Click handler is active even when alreadyAdded.
  implication: duplicate hint depends on existingModeIds containing the template's id.

- checked: src-tauri/src/shortcut/mod.rs add_smart_mode (1072-1091)
  found: id is ALWAYS `format!("mode_{}", chrono::Utc::now().timestamp_millis())`. Template id (mode_clean_up etc.) is discarded.
  implication: a mode created from a template gets id `mode_1733...`, NOT `mode_clean_up`. So existingModeIds for picker copies never contain the template's seeded id → `alreadyAdded` is false for every copy. (It would only be true for the original first-run seed mode_clean_up, which is the single exception.) Duplicate detection is fundamentally broken — by design per Decision [13-07]: "uses addSmartMode (new id) not seeded id — prevents id collision when user deletes a default and recreates it via picker". The collision-avoidance decision directly defeats the id-based duplicate check.

- checked: src/components/settings/post-processing/SmartModeCard.tsx edit/create form (194-255)
  found: Name <Input> (201-206) has NO placeholder. Prompt <Textarea> (215-220) has NO placeholder and no helper text / example. There is no mention of ${output} anywhere in the component. SEEDED_MODE_DEFAULT_NAME map (29-40) already exists and could support name-based duplicate detection.
  implication: confirms (c) — users have no way to discover the ${output} token.

- checked: src-tauri/src/actions.rs token consumption
  found: token string is literally `${output}` (dollar + braces). Two consumption paths:
    - resolve_mode_prompt / build (line 74): `prompt_template.replace("${output}", "")` — strips it because the transcript is sent as the user message in structured-output / chat mode.
    - legacy path (line 333): `prompt.replace("${output}", transcription)` — substitutes the transcript inline.
  implication: The exact token the hint must show is `${output}`. If a custom rewrite prompt omits `${output}`, the legacy path sends the prompt with no transcript inlined; the structured-output path still injects the transcript as the user message, so behavior depends on provider.supports_structured_output. For reliability the hint/example MUST use `${output}`.

- checked: src-tauri/src/settings.rs smart_mode_templates (672-757)
  found: All 6 rewrite templates END their prompt with `...\nText:\n${output}` (or `Transcript:\n${output}` for Clean Up). The 4 translation templates have `prompt: String::new()` (empty) and rely on target_language — translation does not use ${output} (run_translation builds its own prompt). Seeded ids: mode_clean_up, mode_make_formal, mode_make_casual, mode_email, mode_bullet_points, mode_summarize, mode_translate_{en,es,fr,zh}.
  implication: built-in templates already include ${output}; only the Custom (blank) form lacks guidance. Confirms the gap is purely the Custom-form UX, not the seeded prompts.

## Resolution

root_cause: |
  (a) No-distinction duplicate add: The picker's duplicate guard `existingModeIds.includes(template.id)` (SmartModeTemplatePicker.tsx:83) is dead code in practice. add_smart_mode (shortcut/mod.rs:1080) assigns every created mode a fresh `mode_{timestamp}` id and discards the template's seeded id. So a mode created from "Clean Up" gets e.g. mode_1733... not mode_clean_up; the next time the picker opens, existingModeIds contains mode_1733..., which never equals template.id (mode_clean_up). The "added" badge therefore (almost) never shows, and the button stays fully clickable, so the same template can be added repeatedly with no visual distinction. This is the direct consequence of Decision [13-07] (use addSmartMode/new id to avoid id collision on delete-then-recreate), which traded away the only signal id-based detection relied on. There is also no name-based fallback check.

  (b) No overwrite warning for edited default: There is no overwrite path at all. The picker only ever ADDS a new mode (never updates an existing one), so re-importing "Clean Up" after editing it creates a SECOND Clean Up rather than warning/overwriting. The uniqueness check the user expects (by name or seeded mapping) does not exist; SEEDED_MODE_DEFAULT_NAME exists in SmartModeCard but is not used by the picker.

  (c) No ${output} discoverability in Custom form: SmartModeCard.tsx name Input (201) and prompt Textarea (215) have no placeholder attributes and no helper/example text. The component never references ${output}. A user creating a Custom rewrite mode has no way to learn that `${output}` is the transcript token, so their prompt silently fails to inject the transcript on the legacy provider path.

  backend transcript token (authoritative): `${output}` — literal string `${output}` (dollar sign + curly braces). Consumed in src-tauri/src/actions.rs at line 74 (stripped for structured/chat path) and line 333 (`prompt.replace("${output}", transcription)` for legacy path). Translation modes do NOT use it (empty prompt + target_language → run_translation).

fix: ""  # diagnose-only
verification: ""
files_changed: []
