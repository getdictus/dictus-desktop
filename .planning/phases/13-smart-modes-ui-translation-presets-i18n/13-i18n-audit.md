# i18n English-Fallback Audit — Phase 13

**Date:** 2026-06-08
**Method:** Leaf-key equality scan across all 19 non-English locales vs. `en` source (523 leaf keys).
**Result:** 91 keys have locale value byte-equal to the English source in ALL 19 locales.

This document is the authoritative contract for plan 13-23 (bulk translation).

---

## To translate (English-fallback debt)

Keys equal-to-en in ALL 19 locales, **excluding** the allowlist below.
These are the keys 13-23 must translate into all 19 non-English locales.

### errors.*

| Key | English value |
|-----|---------------|
| `errors.boundary.reload` | "Reload" |
| `errors.boundary.title` | "Something went wrong." |

### settings.debug.simulateUpdaterRestart.*

| Key | English value |
|-----|---------------|
| `settings.debug.simulateUpdaterRestart.button` | "Simulate Restart" |
| `settings.debug.simulateUpdaterRestart.description` | "Relaunches Dictus via the same API tauri-plugin-updater uses after applying an update. Use this to validate SHUT-03 (clean post-update relaunch) without publishing a throwaway test release." |
| `settings.debug.simulateUpdaterRestart.title` | "Simulate Updater Restart" |

### settings.postProcessing.modelsAndLocalProcessing.library.models.*

| Key | English value |
|-----|---------------|
| `settings.postProcessing.modelsAndLocalProcessing.library.models.translateGemma4b.description` | "Dedicated translation model — 55 benchmarked languages" |

### smartModes.card.*

| Key | English value |
|-----|---------------|
| `smartModes.card.addShortcut` | "Add shortcut" |
| `smartModes.card.createCta` | "Create mode" |
| `smartModes.card.discardCta` | "Discard changes" |
| `smartModes.card.discardNewCta` | "Discard" |
| `smartModes.card.nameLabel` | "Name" |
| `smartModes.card.namePlaceholder` | "e.g. Make Concise" |
| `smartModes.card.outputHint` | "Use ${output} where the dictated transcript should be inserted." |
| `smartModes.card.promptLabel` | "Prompt" |
| `smartModes.card.promptPlaceholder` | "Rewrite the following text to be more concise.\n\nText:\n${output}" |
| `smartModes.card.saveCta` | "Save mode" |
| `smartModes.card.shortcutConflict` | "Already used by: {{name}}" |
| `smartModes.card.targetLanguage` | "Target language" |

### smartModes.delete.*

| Key | English value |
|-----|---------------|
| `smartModes.delete.ariaLabel` | "Delete {{name}}" |
| `smartModes.delete.confirm` | "Delete \"{{name}}\"? This will also remove its shortcut binding." |
| `smartModes.delete.title` | "Delete mode" |

### smartModes.edit.*

| Key | English value |
|-----|---------------|
| `smartModes.edit.ariaLabel` | "Edit {{name}}" |

### smartModes.kind.*

| Key | English value |
|-----|---------------|
| `smartModes.kind.rewrite` | "Rewrite" |
| `smartModes.kind.translation` | "Translation" |

### smartModes.noModel.*

| Key | English value |
|-----|---------------|
| `smartModes.noModel.body` | "Select or download a local model above to use Smart Modes." |
| `smartModes.noModel.heading` | "No local model selected" |

### smartModes.picker.*

| Key | English value |
|-----|---------------|
| `smartModes.picker.added` | "Added" |
| `smartModes.picker.custom` | "Custom" |
| `smartModes.picker.customSubtext` | "Create a mode with your own name and prompt" |
| `smartModes.picker.overwriteConfirm` | "\"{{name}}\" already exists. Re-adding it will overwrite the existing mode. Continue?" |
| `smartModes.picker.overwriteTitle` | "Mode already exists" |
| `smartModes.picker.rewriteTitle` | "Add a rewrite mode" |
| `smartModes.picker.translationTitle` | "Add a translation mode" |

### smartModes.sections.*

| Key | English value |
|-----|---------------|
| `smartModes.sections.rewrite` | "Rewrite" |
| `smartModes.sections.translation` | "Translation" |

### smartModes.shortcut.*

| Key | English value |
|-----|---------------|
| `smartModes.shortcut.clear` | "Clear" |
| `smartModes.shortcut.clearAriaLabel` | "Clear shortcut {{combo}}" |
| `smartModes.shortcut.recording` | "Press keys…" |

### smartModes.translation.*

| Key | English value |
|-----|---------------|
| `smartModes.translation.changeEngine` | "Change engine" |
| `smartModes.translation.chooseCta` | "Choose translation engine" |
| `smartModes.translation.currentEngine` | "Using: {{engine}}" |
| `smartModes.translation.enableBody` | "Translation runs fully offline through the embedded LLM. Choose an engine to get started." |
| `smartModes.translation.enableHeading` | "Enable offline translation" |
| `smartModes.translation.engineGeneric` | "Active model" |
| `smartModes.translation.genericEngineNote` | "Best-effort — quality varies by model" |
| `smartModes.translation.modal.activeBadge` | "Currently used" |
| `smartModes.translation.modal.activeButton` | "Currently used" |
| `smartModes.translation.modal.applyFailed` | "Could not change the translation engine." |
| `smartModes.translation.modal.applying` | "Applying…" |
| `smartModes.translation.modal.currentHeading` | "Use your current model" |
| `smartModes.translation.modal.currentNone` | "No local model is active yet. Download one from the model library first." |
| `smartModes.translation.modal.currentSubtext` | "Translate with {{model}} (your active model). Quality varies by model." |
| `smartModes.translation.modal.downloadBackgroundNote` | "You can close this window — the download continues in the background." |
| `smartModes.translation.modal.downloadRecommended` | "Download Gemma 3 4B" |
| `smartModes.translation.modal.downloadingPercent` | "Downloading… {{percent}}%" |
| `smartModes.translation.modal.enableCta` | "Enable offline translation" |
| `smartModes.translation.modal.footnote` | "Both options run fully offline. You can change this choice later in settings." |
| `smartModes.translation.modal.recommendOnlyBody` | "For the best translation quality, select Gemma 3 4B in the model selector above. Translation runs through your single active model." |
| `smartModes.translation.modal.recommendOnlyHeading` | "Gemma 3 4B recommended for translation" |
| `smartModes.translation.modal.recommendedHeading` | "Recommended: Gemma 3 4B" |
| `smartModes.translation.modal.recommendedSubtext` | "A versatile 4B model with clean, reliable translation across languages and idioms. ~2.5 GB download." |
| `smartModes.translation.modal.title` | "Set up offline translation" |
| `smartModes.translation.modal.useCurrent` | "Use current model" |
| `smartModes.translation.modal.useRecommended` | "Use Gemma 3 4B" |
| `smartModes.translation.modal.verifying` | "Verifying download…" |
| `smartModes.translation.recommendNote` | "Gemma 3 4B is recommended for translation. Choose it in the model selector above." |

### smartModes top-level

| Key | English value |
|-----|---------------|
| `smartModes.createRewrite` | "New Rewrite" |
| `smartModes.createTranslation` | "New Translation" |

**Total keys to translate: 69**
- `smartModes.*` (excluding `defaultModes.*` which were translated in 13-15): **62**
- `errors.boundary.*`: **2**
- `settings.debug.simulateUpdaterRestart.*`: **3**
- `settings.postProcessing.modelsAndLocalProcessing.library.models.translateGemma4b.description`: **1**
- `smartModes.kind.*`: **2** (included above in smartModes count)

Note: `smartModes.defaultModes.*` (cleanUp, makeFormal, makeCasual, writeAsEmail, bulletPoints, summarize, translateToEnglish, translateToSpanish, translateToFrench, translateToChinese) were translated in plan 13-15 and do NOT appear here.

---

## Allowlist (legitimately identical — DO NOT translate)

These keys have identical values across locales by design. They must NOT be flagged by `--check-untranslated`.

### Model names (proper nouns / brand names)

All keys matching `onboarding.models.*.name`:
- `onboarding.models.breeze-asr.name` — "Breeze ASR"
- `onboarding.models.canary-180m-flash.name` — "Canary 180M Flash"
- `onboarding.models.canary-1b-v2.name` — "Canary 1B v2"
- `onboarding.models.cohere-int8.name` — "Cohere"
- `onboarding.models.gigaam-v3-e2e-ctc.name` — "GigaAM v3"
- `onboarding.models.large.name` — "Whisper Large"
- `onboarding.models.medium.name` — "Whisper Medium"
- `onboarding.models.moonshine-base.name` — "Moonshine Base"
- `onboarding.models.moonshine-medium-streaming-en.name` — "Moonshine V2 Medium"
- `onboarding.models.moonshine-small-streaming-en.name` — "Moonshine V2 Small"
- `onboarding.models.moonshine-tiny-streaming-en.name` — "Moonshine V2 Tiny"
- `onboarding.models.parakeet-tdt-0.6b-v2.name` — "Parakeet V2"
- `onboarding.models.parakeet-tdt-0.6b-v3.name` — "Parakeet V3"
- `onboarding.models.sense-voice-int8.name` — "SenseVoice"
- `onboarding.models.small.name` — "Whisper Small"
- `onboarding.models.turbo.name` — "Whisper Turbo"

### Acknowledgment titles (proper nouns)

- `settings.about.acknowledgments.handy.title` — "Handy" (project name)
- `settings.about.acknowledgments.whisper.title` — "Whisper.cpp" (project name)

### Keyboard literals

All keys matching `settings.advanced.autoSubmit.options.{cmdEnter,ctrlEnter,superEnter}`:
- `settings.advanced.autoSubmit.options.cmdEnter` — "Cmd+Enter"
- `settings.advanced.autoSubmit.options.ctrlEnter` — "Ctrl+Enter"
- `settings.advanced.autoSubmit.options.superEnter` — "Super+Enter"

### Placeholder strings (UI input hints — must remain identical)

- `settings.postProcessing.api.apiKey.placeholder` — "sk-..."
- `settings.postProcessing.api.baseUrl.placeholder` — "https://api.openai.com/v1"

---

## Partial (equal-to-en in SOME but not all locales)

81 keys are equal to the English source in at least one but not all locales. These are out of [G14] scope — the locales that translated them did so; the ones that didn't may have chosen an equivalent form or will be covered opportunistically by 13-23 for keys that fall within `smartModes.*`.

Notable partial-equal clusters (all equal in 18/19 locales — only `fr` translated them):
- `settings.postProcessing.modelsAndLocalProcessing.library.*` (13 keys) — library UI translated in `fr` only
- `settings.postProcessing.modelsAndLocalProcessing.embedded.*` (5 keys) — embedded UI translated in `fr` only
- `settings.postProcessing.modelsAndLocalProcessing.library.models.{qwen25_1b5,gemma3_4b,phi4_mini,llama32_3b}.description` — translated in `fr` only

These partial keys (library/embedded section) are within the [G14] scope for 13-23 since they represent English fallback for 18 locales. 13-23 should translate them alongside the `smartModes.*` keys.

**Total partial keys: 81** (count only; details available from the audit scan script if needed)
