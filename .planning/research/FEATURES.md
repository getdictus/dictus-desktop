# Feature Research

**Domain:** Desktop speech-to-text app (rebranding milestone — Handy fork → Dictus Desktop V1)
**Researched:** 2026-04-05
**Confidence:** HIGH (rebrand scope well-defined; language feature verified in codebase; STT competitive landscape confirmed via multiple sources)

---

## Feature Landscape

### Table Stakes for Rebranding (Missing = Not a Real Rebrand)

Features that must exist for the product to be legitimately called "Dictus Desktop" rather than a reskinned Handy.

| Feature                                           | Why Expected                                                                             | Complexity | Notes                                                                                              |
| ------------------------------------------------- | ---------------------------------------------------------------------------------------- | ---------- | -------------------------------------------------------------------------------------------------- |
| Product name change everywhere visible            | Users see "Handy" = product identity failure                                             | LOW        | tauri.conf.json `productName`, window title, about panel                                           |
| Bundle identifier update                          | `com.handy.*` in system = broken install trust; future com.dictus.desktop packaging      | LOW        | `identifier` in tauri.conf.json; Tauri 2 single field                                              |
| App icon and logo replacement                     | Visual identity is the primary signal of brand                                           | MEDIUM     | Multiple sizes required (macOS .icns, Windows .ico, Linux .png); must match Dictus visual language |
| Onboarding text + visuals rebranded               | First run experience is the brand introduction                                           | LOW        | Text changes + replace HandyHand/HandyTextLogo components                                          |
| All "Handy" labels removed from UI                | Any remaining "Handy" reference is a regression                                          | LOW        | Systematic grep + replace across JSX/translation files                                             |
| i18n strings updated (all locales)                | UI text is what users read — en/es/fr/vi must all be updated                             | MEDIUM     | 4 locale files; must preserve translation completeness                                             |
| README rewritten as Dictus Desktop                | Public repository identity — fork attribution + Dictus positioning                       | LOW        | Markdown only; no code changes                                                                     |
| Internal code references renamed (critical path)  | `handy_app_lib`, `handy.log`, `startHandyKeysRecording` in observable outputs leak brand | MEDIUM     | Rust library name in Cargo.toml, log file path, binary name                                        |
| Documentation cleaned (CLAUDE.md, BUILD.md, etc.) | Dev-facing docs with "Handy" break contributor/maintainer trust                          | LOW        | Text changes only                                                                                  |

### Table Stakes for Desktop STT UX (Users Assume These Exist)

Features that desktop STT users expect regardless of brand. Missing these makes the product feel broken.

| Feature                                             | Why Expected                                                                         | Complexity | Notes                                                               |
| --------------------------------------------------- | ------------------------------------------------------------------------------------ | ---------- | ------------------------------------------------------------------- |
| Global keyboard shortcut to start/stop recording    | STT without a hotkey is unusable in practice; every competitor has this              | LOW        | Already exists; just needs Dictus-branded default                   |
| Recording visual feedback                           | User needs to know when app is listening                                             | LOW        | Overlay already exists                                              |
| Auto-paste or clipboard output                      | Transcription that requires manual paste is friction; kills workflow                 | LOW        | Already exists (multiple paste methods)                             |
| Language selection (Auto + specific languages)      | Multi-language users need control; monolingual users expect auto                     | LOW-MEDIUM | Already exists; "force language" UX needs clarification (see below) |
| System tray presence                                | Desktop apps that vanish are confusing; tray = "it's running" signal                 | LOW        | Already exists                                                      |
| Transcription history                               | Users expect to recover recent dictations                                            | LOW        | Already exists                                                      |
| Model management (download, switch)                 | Local STT requires user-facing model control; cloud apps hide this, local apps can't | MEDIUM     | Already exists                                                      |
| Settings for audio device selection                 | Multi-device users (headset vs built-in mic) need this                               | LOW        | Already exists                                                      |
| Error feedback (mic permission, model load failure) | Silent failures destroy user trust                                                   | LOW        | Already exists via toast/events                                     |

### Differentiators for Dictus (Competitive Advantage in V1 Context)

These set Dictus Desktop apart from Handy and from generic Whisper wrappers like MacWhisper. Not all are V1 work — some are positioning choices that cost nothing to document.

| Feature                                         | Value Proposition                                                                                                                                                                | Complexity | Notes                                                                                                                 |
| ----------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- | --------------------------------------------------------------------------------------------------------------------- |
| Dictus ecosystem coherence                      | Users with Dictus iOS recognize the brand, palette, tone — reduces onboarding friction                                                                                           | MEDIUM     | Visual alignment work: palette tokens, sidebar labels, section naming (General→Dictation, Postprocessing→Smart Modes) |
| Privacy-first explicit positioning              | "No cloud, no account, audio never leaves device" — this is a real differentiator vs Speechify, Wispr Flow                                                                       | LOW        | Documentation + About panel copy; no code needed                                                                      |
| Forced language guarantee (UX clarity)          | Power users who dictate in French exclusively don't want auto-detect guessing wrong; clear "Auto / Français / English" selector is more trustworthy than a 100-language dropdown | LOW        | Existing LanguageSelector already does this technically; the gap is UX clarity and confidence messaging               |
| Cross-platform parity (macOS + Windows + Linux) | Most polished STT tools are macOS-only (Superwhisper is Mac-only); Linux support is rare                                                                                         | LOW        | Already exists; just needs to be surfaced in documentation                                                            |
| Open source + MIT license                       | Trust, auditability, no vendor lock-in — increasingly valued as privacy concern grows                                                                                            | LOW        | README positioning only                                                                                               |
| LLM post-processing ("Smart Modes")             | Transcription cleanup, formatting, tone adjustment — more than raw STT                                                                                                           | LOW        | Already exists; rename "postprocessing" → "Smart Modes" in UI aligns with Dictus iOS language                         |

### Anti-Features (Do Not Build in V1)

Features that seem reasonable but would harm the V1 milestone scope, quality, or timeline.

| Feature                                            | Why Requested                                    | Why Problematic                                                                                                                                                    | Alternative                                                                               |
| -------------------------------------------------- | ------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------- |
| Pixel-perfect iOS design port                      | "Make it look exactly like the iOS app"          | Desktop ≠ mobile interaction model; window chrome, keyboard navigation, density differ fundamentally; a pixel port would look wrong on desktop and take 10x longer | Align palette + tone + naming; accept desktop-specific layout as correct                  |
| Mobile ↔ desktop sync                             | Natural evolution of Dictus ecosystem            | Zero architecture defined for this yet; cloud/Nostr/local sync are all open questions; V1 with sync hooks would be half-built and unstable                         | Hard defer to V4+; document it as planned, not present                                    |
| User accounts / auth                               | Monetization or cross-device continuity pressure | Privacy-first product with accounts is a trust contradiction; adds infra, auth flows, and security surface in V1                                                   | Stay local-only; frame as a feature, not a limitation                                     |
| Model auto-recommendation (fast/balanced/accurate) | Users want guidance on model selection           | Requires calibration across hardware profiles, model performance benchmarking, and UI to surface it; not a rebrand task                                            | V2 scope; V1 just shows available models with capability metadata                         |
| Additional LLM providers in V1                     | "While we're in there..."                        | Risk of scope creep on a branding milestone; existing providers (OpenAI, Anthropic, Ollama, etc.) already cover needs                                              | V2 feature; existing providers stay as-is                                                 |
| Real-time streaming transcription                  | Users see AI tools doing live captions           | whisper.cpp architecture in this codebase is segment-based, not streaming; adding streaming is a pipeline rewrite, not a V1 task                                   | VAD + fast inference gives low-enough latency for dictation use; call it "near real-time" |
| Windows/Linux code-signing in V1                   | Proper distribution                              | Code signing requires certificates, notarization workflows, CI pipeline work — separate from rebrand                                                               | V2 distribution milestone; V1 is local dev builds                                         |

---

## Feature Dependencies

```
[Dictus brand identity (icons, palette, name)]
    └──required by──> [Onboarding rebrand]
    └──required by──> [About panel rebrand]
    └──required by──> [Bundle identifier update]

[Bundle identifier (com.dictus.desktop)]
    └──required by──> [Installable .dmg / .exe / .deb packaging]
                           └──required by──> [Distribution / release]

[i18n strings updated]
    └──required by──> [Any UI label change in non-English locales]

[Force Language UX clarity]
    └──enhances──> [Language Selector (existing)]
    └──requires──> [Understanding of whisper.cpp language param behavior]

[Settings section renaming (General→Dictation)]
    └──enhances──> [Dictus iOS visual alignment]
    └──depends on──> [i18n strings updated]

[Internal code rename (Cargo.toml, log paths)]
    └──independent of──> [Visible UI rebrand]  ← can be deferred to pass 2
```

### Dependency Notes

- **Icons require brand identity first:** App icons cannot be produced until the Dictus visual language (palette, logomark) is defined or sourced from the iOS app assets.
- **Bundle ID change is safe for a new app:** Since Dictus Desktop has no existing install base (new product, not an update), changing from `com.handy.*` to `com.dictus.desktop` carries zero migration risk.
- **Force language depends on whisper.cpp behavior understanding:** The existing `LanguageSelector` already passes `selected_language` → whisper params. The UX work is labeling and confidence copy, not backend changes. One known edge case: multilingual models may override explicit language params (confirmed whisper.cpp issue #1831). The clarification task is documenting this behavior honestly in the UI.
- **Internal rename (pass 2) is independent:** Renaming `handy_app_lib`, `startHandyKeysRecording`, etc. does not block any user-visible V1 feature. It can be done after the visible rebrand without risk.

---

## MVP Definition

### Launch With (V1 — Dictus Desktop Foundation)

Minimum for the product to legitimately be "Dictus Desktop V1" rather than a Handy fork with a renamed README.

- [ ] Product name "Dictus Desktop" everywhere visible (window title, about, tray menu, onboarding)
- [ ] Bundle identifier `com.dictus.desktop` in tauri.conf.json + Cargo.toml
- [ ] Dictus app icon (all required sizes for macOS/Windows/Linux)
- [ ] All "Handy" user-visible text replaced (UI labels, onboarding, settings)
- [ ] i18n strings updated across all 4 locales (en/es/fr/vi) — or flagged as incomplete
- [ ] Settings sections renamed to Dictus language (Dictation, Smart Modes)
- [ ] Force language UX: Auto / specific language selector with clear behavior description
- [ ] README.md as Dictus Desktop (fork attribution, privacy-first positioning, MIT license)
- [ ] Critical internal renames that surface in logs or filesystem (handy.log → dictus.log, binary name)

### Add After Validation (V1.x)

- [ ] Internal code symbol rename (handy_app_lib → dictus_app_lib, function names) — low user impact, do after V1 ships
- [ ] Complete Dictus iOS palette alignment (if token-level work is needed beyond color overrides)
- [ ] About panel with open-source / privacy-first copy
- [ ] BUILD.md and contributor docs updated

### Future Consideration (V2+)

- [ ] Model recommendation tiers (fast/balanced/accurate) — needs benchmark data
- [ ] Windows/Linux code signing and notarization pipeline
- [ ] Smart model routing (short vs long audio)
- [ ] Mobile ↔ desktop sync (requires architecture decision first)
- [ ] Additional language UI (per-mode language override)

---

## Feature Prioritization Matrix

| Feature                                          | User Value                       | Implementation Cost            | Priority |
| ------------------------------------------------ | -------------------------------- | ------------------------------ | -------- |
| Product name + bundle ID                         | HIGH                             | LOW                            | P1       |
| App icon replacement                             | HIGH                             | MEDIUM (asset production)      | P1       |
| Onboarding rebrand                               | HIGH                             | LOW                            | P1       |
| UI labels + i18n update                          | HIGH                             | MEDIUM (4 locales, systematic) | P1       |
| Settings section rename                          | MEDIUM                           | LOW                            | P1       |
| Force language UX clarity                        | MEDIUM                           | LOW (copy + description only)  | P1       |
| README rewrite                                   | MEDIUM                           | LOW                            | P1       |
| Internal code rename (critical path: log/binary) | LOW (user) / HIGH (maintainer)   | MEDIUM                         | P2       |
| Internal symbol rename (Rust lib, functions)     | LOW (user) / MEDIUM (maintainer) | MEDIUM                         | P2       |
| Dictus iOS palette token alignment               | MEDIUM                           | MEDIUM                         | P2       |
| About panel copy                                 | LOW                              | LOW                            | P2       |
| Documentation cleanup (CLAUDE.md, BUILD.md)      | LOW                              | LOW                            | P3       |

**Priority key:**

- P1: Must have for V1 release — product is incomplete without it
- P2: Should have — ship in V1.x or before first external user
- P3: Nice to have — background work, no user urgency

---

## Force Language: What It Means in Whisper Context

This feature deserves its own section because it's explicitly in scope for V1 and has nuance.

### How whisper.cpp language parameter works

- Default (`selected_language = "auto"`): Whisper auto-detects the language from the first ~30 seconds of audio. Works well for clear single-language recordings.
- Forced (`selected_language = "fr"`): The `language` param is set explicitly in `WhisperInferenceParams`. Whisper skips auto-detection and treats the audio as the specified language.
- Known edge case: With some multilingual models, whisper.cpp has been reported to override explicit language params and auto-detect anyway (GitHub issue #1831). The existing code already handles this by validating language against the model's `supported_languages` list and falling back to `"auto"` if unsupported — this is correct behavior.

### What the codebase already does (HIGH confidence — verified in source)

The `LanguageSelector` component (`src/components/settings/LanguageSelector.tsx`) already:

- Shows "Auto" as the default and reset value (`default_selected_language() = "auto"`)
- Shows a searchable dropdown of all supported languages (filtered to the active model's `supported_languages`)
- Persists the choice to settings (`selected_language`)
- The `TranscriptionManager` reads `settings.selected_language`, validates it against model support, and passes it to the whisper engine as `None` (auto) or `Some(lang_code)` (forced)

### What the V1 UX work actually is (LOW complexity)

The current implementation is technically complete. The V1 task is **UX clarity**, not engineering:

1. Rename the setting section from "General" to "Dictation" so language appears under dictation context
2. Update the description copy to make "Auto" vs forced behavior explicit (the current string is good but generic)
3. Ensure the selector shows "Auto" prominently with a clear label (not just a list item)
4. Verify behavior documentation is accurate about the edge case (multilingual model fallback)

There is no new backend work needed unless the team discovers the forced language is not reliably honored — in that case it becomes a V2 investigation.

---

## Competitor Feature Analysis

| Feature                              | Superwhisper    | MacWhisper      | OpenWhispr | Dictus Desktop V1 Target              |
| ------------------------------------ | --------------- | --------------- | ---------- | ------------------------------------- |
| Local processing (no cloud required) | Yes (default)   | Yes             | Yes        | Yes                                   |
| Global hotkey                        | Yes             | Limited         | Yes        | Yes (existing)                        |
| Recording overlay                    | Yes             | No (file-based) | Yes        | Yes (existing)                        |
| Language auto-detect                 | Yes             | Yes             | Yes        | Yes (existing)                        |
| Force specific language              | Yes (via modes) | Yes             | Yes        | Yes (existing, UX to clarify)         |
| Transcription history                | Yes             | Yes             | Yes        | Yes (existing)                        |
| LLM post-processing                  | Yes (AI modes)  | No              | Partial    | Yes (existing, rename to Smart Modes) |
| Cross-platform (Win/Linux)           | macOS only      | macOS only      | Yes        | Yes                                   |
| Open source                          | No              | No              | Yes        | Yes (MIT)                             |
| Privacy-first messaging              | Partial         | Strong          | Strong     | Strong (V1 README positioning)        |
| App-specific modes                   | Yes (premium)   | No              | No         | No (V2+)                              |
| Mobile companion                     | No              | No              | No         | Planned V4+                           |

---

## Sources

- [OpenWhispr vs SuperWhisper comparison](https://openwhispr.com/compare/superwhisper) — competitor feature landscape
- [Superwhisper documentation](https://superwhisper.com/docs/get-started/introduction) — feature set, language modes, dictation modes
- [whisper.cpp CLI documentation (DeepWiki)](https://deepwiki.com/ggml-org/whisper.cpp/3.1-command-line-interface) — language parameter behavior
- [whisper.cpp language detection issue #1831](https://github.com/ggml-org/whisper.cpp/issues/1831) — known edge case for forced language override
- [Tauri configuration reference](https://v2.tauri.app/reference/config/) — productName, identifier fields
- [Speechify Windows app local models (TechCrunch 2026)](https://techcrunch.com/2026/03/31/speechifys-windows-app-uses-local-models-for-transcription-and-dictation/) — market context
- [Best dictation tools comparison 2026 (OpenWhispr)](https://openwhispr.com/blog/best-dictation-tools-windows-2026) — table stakes feature set
- [11 Best Superwhisper Alternatives 2026 (Voibe)](https://www.getvoibe.com/blog/superwhisper-alternatives/) — competitor positioning
- Verified directly in codebase: `src/components/settings/LanguageSelector.tsx`, `src-tauri/src/managers/transcription.rs`, `src-tauri/src/settings.rs`

---

_Feature research for: Desktop speech-to-text app rebranding (Dictus Desktop V1)_
_Researched: 2026-04-05_
