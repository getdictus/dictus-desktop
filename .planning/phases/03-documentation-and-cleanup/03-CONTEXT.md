# Phase 3: Documentation and Cleanup - Context

**Gathered:** 2026-04-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Rewrite external documentation (README, BUILD.md) to present Dictus Desktop as a first-class product, update internal developer references (CLAUDE.md) to remove Handy-specific instructions, and rebrand the in-app About panel with Dictus identity, ecosystem links, and privacy positioning.

</domain>

<decisions>
## Implementation Decisions

### README positioning

- Fork-first introduction: lead with "Dictus Desktop is a fork of Handy" then pivot to Dictus identity and vision
- Strip to essentials: remove sponsors section, Raycast integration, signature verification, Handy-specific roadmap
- Keep: intro, how it works, quick start, architecture summary, updated troubleshooting, license + attribution
- Installation: link to getdictus.com as the main site (note desktop downloads coming soon) + build-from-source via BUILD.md
- Remove manual model download URLs (blob.handy.computer) from README entirely — app handles downloads internally
- Contact/community: GitHub Issues (getdictus/dictus-desktop), getdictus.com, hello@getdictus.com, Telegram https://t.me/getdictus
- Fork attribution in a dedicated Acknowledgments section: "Dictus Desktop is a fork of Handy by cjpais" with link to original repo

### About panel content

- Donate button: link to getdictus.com/donate (Stripe redirect to be set up separately)
- Source code button: link to https://github.com/getdictus/dictus-desktop
- Add privacy tagline: "Privacy-first speech-to-text. Your voice stays on your device." or similar
- Add ecosystem mention: Dictus is available on iOS, Android, and Desktop
- Link to privacy policy: getdictus.com/en/privacy
- Keep existing sections: version display, app data directory, log directory, acknowledgments (Whisper)

### Developer docs (CLAUDE.md & BUILD.md)

- BUILD.md: update clone URL to github.com/getdictus/dictus-desktop, update all Handy path references to Dictus equivalents
- CLAUDE.md: replace "Handy" with "Dictus Desktop" in architecture descriptions and all text
- Remove blob.handy.computer URLs from both docs — the VAD model curl in CLAUDE.md dev setup needs a working alternative or a note
- All Handy-branded text (descriptions, headers, explanations) rewritten as Dictus Desktop

### Handy remnants policy

- Docs should use Dictus names even where technical paths haven't been renamed yet (binary still "handy", some internal paths) — prepares docs for the eventual full rename (V2 TECH-01 through TECH-04)
- Where current technical reality differs from Dictus naming, note the discrepancy briefly so developers aren't confused
- Fork attribution via Acknowledgments section, not scattered throughout docs

### Claude's Discretion

- Exact README structure and section ordering
- How to handle technical path discrepancies (binary name "handy" vs docs saying "Dictus") — balance clarity with forward-looking naming
- Exact wording of privacy tagline and ecosystem mention in About panel
- Whether to keep or simplify the troubleshooting section in README
- Tone calibration between English and French content
- Acknowledgments section content beyond Handy attribution (Whisper, Tauri, etc.)

</decisions>

<canonical_refs>

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Files to modify

- `README.md` — Current Handy README (450 lines), full rewrite needed for Dictus Desktop
- `BUILD.md` — Build instructions with Handy clone URLs and paths
- `CLAUDE.md` — Developer guide with Handy architecture descriptions
- `src/components/settings/about/AboutSettings.tsx` — In-app About panel (links to handy.computer/donate, github.com/cjpais/Handy)

### Project requirements

- `.planning/REQUIREMENTS.md` — DOCS-01, DOCS-02, DOCS-03 define acceptance criteria
- `.planning/ROADMAP.md` — Phase 3 success criteria (3 conditions that must be TRUE)

### Brand & ecosystem references

- `getdictus.com` — Main Dictus website (currently mobile downloads only, desktop coming)
- `getdictus.com/en/privacy` — Privacy policy page
- `getdictus.com/donate` — Donate page (Stripe redirect, to be set up by user)
- `https://github.com/getdictus/dictus-desktop` — Dictus Desktop GitHub repo
- `https://t.me/getdictus` — Telegram community
- `hello@getdictus.com` — Contact email

### Prior context (tone & decisions)

- `.planning/phases/02-visual-rebrand/02-CONTEXT.md` — Dictus iOS tone reference: warm, direct, privacy-aware
- `.planning/phases/01-bundle-identity/01-CONTEXT.md` — Fork attribution style, version 0.1.0, authors = ["Dictus", "cjpais"]

</canonical_refs>

<code_context>

## Existing Code Insights

### Reusable Assets

- `src/components/settings/about/AboutSettings.tsx` — Existing About panel structure with SettingsGroup, SettingContainer, Button components
- i18n keys under `settings.about.*` — existing translation structure for About section content
- `src/components/ui/SettingsGroup.tsx`, `SettingContainer.tsx`, `Button.tsx` — UI components already used in About

### Established Patterns

- All user-facing strings via i18next (`t('key.path')`) — ESLint enforced, no hardcoded strings in JSX
- Links opened via `openUrl()` from `@tauri-apps/plugin-opener`
- Version fetched via `getVersion()` from `@tauri-apps/api/app`

### Integration Points

- About panel i18n keys in `src/i18n/locales/{en,es,fr,vi,...}/translation.json` — all locales need updated strings
- README.md, BUILD.md, CLAUDE.md — standalone markdown files, no code dependencies

</code_context>

<specifics>
## Specific Ideas

- Dictus ecosystem: iOS, Android, and Desktop — not just iOS. About panel should reflect this
- Privacy policy has its own page: getdictus.com/en/privacy — link to it from About panel
- Telegram community (t.me/getdictus) and email (hello@getdictus.com) as contact channels
- User wants docs to be forward-looking with Dictus naming even where technical paths haven't caught up yet

</specifics>

<deferred>
## Deferred Ideas

- CDN migration from blob.handy.computer to Dictus-owned CDN — INFR-01, V2
- Model download from HuggingFace/GitHub instead of custom CDN — worth investigating for INFR-01
- Binary rename from "handy" to "dictus" — TECH-03, V2
- Stripe donate page setup on getdictus.com/donate — user-side task, not code

</deferred>

---

_Phase: 03-documentation-and-cleanup_
_Context gathered: 2026-04-09_
