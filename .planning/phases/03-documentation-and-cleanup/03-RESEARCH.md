# Phase 3: Documentation and Cleanup - Research

**Researched:** 2026-04-09
**Domain:** Markdown documentation rewrite + React/i18n component update
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**README positioning**
- Fork-first introduction: lead with "Dictus Desktop is a fork of Handy" then pivot to Dictus identity and vision
- Strip to essentials: remove sponsors section, Raycast integration, signature verification, Handy-specific roadmap
- Keep: intro, how it works, quick start, architecture summary, updated troubleshooting, license + attribution
- Installation: link to getdictus.com as the main site (note desktop downloads coming soon) + build-from-source via BUILD.md
- Remove manual model download URLs (blob.handy.computer) from README entirely — app handles downloads internally
- Contact/community: GitHub Issues (getdictus/dictus-desktop), getdictus.com, hello@getdictus.com, Telegram https://t.me/getdictus
- Fork attribution in a dedicated Acknowledgments section: "Dictus Desktop is a fork of Handy by cjpais" with link to original repo

**About panel content**
- Donate button: link to getdictus.com/donate (Stripe redirect to be set up separately)
- Source code button: link to https://github.com/getdictus/dictus-desktop
- Add privacy tagline: "Privacy-first speech-to-text. Your voice stays on your device." or similar
- Add ecosystem mention: Dictus is available on iOS, Android, and Desktop
- Link to privacy policy: getdictus.com/en/privacy
- Keep existing sections: version display, app data directory, log directory, acknowledgments (Whisper)

**Developer docs (CLAUDE.md & BUILD.md)**
- BUILD.md: update clone URL to github.com/getdictus/dictus-desktop, update all Handy path references to Dictus equivalents
- CLAUDE.md: replace "Handy" with "Dictus Desktop" in architecture descriptions and all text
- Remove blob.handy.computer URLs from both docs — the VAD model curl in CLAUDE.md dev setup needs a working alternative or a note
- All Handy-branded text (descriptions, headers, explanations) rewritten as Dictus Desktop

**Handy remnants policy**
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

### Deferred Ideas (OUT OF SCOPE)
- CDN migration from blob.handy.computer to Dictus-owned CDN — INFR-01, V2
- Model download from HuggingFace/GitHub instead of custom CDN — worth investigating for INFR-01
- Binary rename from "handy" to "dictus" — TECH-03, V2
- Stripe donate page setup on getdictus.com/donate — user-side task, not code
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| DOCS-01 | README Dictus Desktop (fork assumed, vision, MIT licence, open-source positioning) | README is 457 lines of Handy content — full rewrite required; structure and content map documented below |
| DOCS-02 | CLAUDE.md and BUILD.md updated (Handy references removed) | 3 Handy references in CLAUDE.md (line 32: blob.handy.computer URL, line 37: "Handy is a cross-platform...", line 124: "Handy supports command-line"); BUILD.md full of Handy clone URL and path references |
| DOCS-03 | In-app About section rebranded Dictus | AboutSettings.tsx has 2 hardcoded Handy URLs (lines 32, 69); i18n keys under settings.about.* already say "Dictus" for most strings; new keys needed for privacy tagline, ecosystem, privacy policy link |
</phase_requirements>

## Summary

Phase 3 is a documentation-only phase with one small React component update. The work divides into three distinct concerns: (1) a full rewrite of README.md from 457-line Handy content to Dictus-first documentation, (2) targeted line edits to CLAUDE.md and BUILD.md to remove Handy references, and (3) updating AboutSettings.tsx to replace two hardcoded Handy URLs and add new i18n-backed content (privacy tagline, ecosystem mention, privacy policy link).

The i18n system is already in place and already uses "Dictus" in most settings.about.* keys (Phase 2 completed this). The About panel needs two hardcoded URL fixes in the TSX file plus new i18n keys for the three new content additions (privacy tagline, ecosystem mention, privacy policy link). All 20 locales will need the new keys, but only en and fr require human-quality translations — the other 18 can receive English fallback strings or copy from en.

The VAD model URL in CLAUDE.md (`blob.handy.computer/silero_vad_v4.onnx`) is the one technically non-trivial decision: the CDN migration is deferred (INFR-01), so the note in dev docs should acknowledge the URL is from the original Handy CDN and flag it as a V2 migration target, rather than replacing it with a broken or non-existent URL.

**Primary recommendation:** Three separate tasks — README rewrite, developer docs line edits, About panel TSX+i18n update — in that order (all parallelizable since they touch independent files).

## Standard Stack

### Core (already in project)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| i18next / react-i18next | Existing | All user-facing strings in About panel | ESLint enforces no literal strings in JSX — existing pattern |
| @tauri-apps/plugin-opener | Existing | `openUrl()` for external links | Used in current AboutSettings.tsx |
| @tauri-apps/api/app | Existing | `getVersion()` for version display | Used in current AboutSettings.tsx |
| SettingsGroup / SettingContainer / Button | Existing components | About panel layout | All currently used in AboutSettings.tsx |

No new dependencies required for this phase.

## Architecture Patterns

### Files Modified

```
README.md                                       # Full rewrite
BUILD.md                                        # Targeted edits (clone URL, binary paths)
CLAUDE.md                                       # Targeted edits (3 Handy references)
src/components/settings/about/AboutSettings.tsx # URL fixes + new SettingContainer blocks
src/i18n/locales/en/translation.json           # New keys under settings.about.*
src/i18n/locales/fr/translation.json           # New keys (French translations)
src/i18n/locales/{ar,bg,cs,de,es,he,it,ja,ko,pl,pt,ru,sv,tr,uk,vi,zh,zh-TW}/translation.json
                                                # New keys (English fallback acceptable)
```

### Pattern 1: Adding new i18n keys to About panel

**What:** Add keys to translation.json, consume with `t('key.path')` in TSX.

**When to use:** Any new user-facing string in About panel.

**Example:**
```typescript
// Source: existing pattern in AboutSettings.tsx
<SettingContainer
  title={t("settings.about.privacy.title")}
  description={t("settings.about.privacy.description")}
  grouped={true}
>
  <Button
    variant="secondary"
    size="md"
    onClick={() => openUrl("https://getdictus.com/en/privacy")}
  >
    {t("settings.about.privacy.button")}
  </Button>
</SettingContainer>
```

### Pattern 2: Updating hardcoded URLs in TSX

**What:** Two existing hardcoded URLs in AboutSettings.tsx need replacement.

**Current (Handy) → Target (Dictus):**
- Line 32: `https://handy.computer/donate` → `https://getdictus.com/donate`
- Line 69: `https://github.com/cjpais/Handy` → `https://github.com/getdictus/dictus-desktop`

### Pattern 3: Binary name discrepancy notation in docs

**What:** The binary is currently named `handy` (TECH-03 deferred to V2). Docs use "Dictus" but CLI examples still reference `handy` binary.

**Recommended approach:** Add a note in CLAUDE.md and BUILD.md CLI sections:

```markdown
> **Note:** The binary is currently named `handy` (a V2 rename is planned).
> Use `handy --toggle-transcription` until the binary rename is complete.
```

This avoids confusing a developer who types `dictus --toggle-transcription` and gets "command not found."

### Anti-Patterns to Avoid
- **Don't hardcode new Dictus URLs directly in TSX**: All user-facing content must go through i18n keys, then the URL lives in the TSX `onClick`. The URL itself is fine hardcoded in `openUrl()` — only visible text needs i18n.
- **Don't leave blob.handy.computer in CLAUDE.md without a note**: Simply deleting the VAD model curl command leaves developers unable to set up the dev environment. Keep the URL with a V2 migration note.
- **Don't translate new i18n keys into all 20 locales manually**: For non-EN/FR locales, English fallback is acceptable for V1. The i18n system falls back to EN automatically.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| External link opening | Manual window.open or anchor | `openUrl()` from @tauri-apps/plugin-opener | Already in project; handles platform differences |
| New About panel layout | Custom divs | SettingContainer + SettingsGroup | Existing components maintain visual consistency |
| i18n string management | Direct JSX text | `t('key')` via react-i18next | ESLint will flag any literal strings in JSX |

## Common Pitfalls

### Pitfall 1: ESLint i18n enforcement will reject literal strings
**What goes wrong:** Adding any user-visible text directly in JSX (e.g., `<span>Privacy-first speech-to-text</span>`) triggers the `i18next/no-literal-string` ESLint rule and breaks the lint step.
**Why it happens:** CLAUDE.md explicitly states ESLint enforces i18n for all user-facing strings.
**How to avoid:** All new text content in AboutSettings.tsx must be wrapped in `t('key.path')`. Add the key to translation.json files first.
**Warning signs:** `bun run lint` fails with "i18next/no-literal-string" errors.

### Pitfall 2: Missing new i18n keys in non-EN locales breaks builds
**What goes wrong:** If a new key is added to EN but not other locales, TypeScript/i18n may warn. At minimum, i18next will silently fall back to EN — acceptable for V1 but should be consistent.
**How to avoid:** Add the new keys to all 20 locales. For non-EN/FR, copy the English string — i18next will use it as fallback anyway, but explicit keys prevent missing-key warnings.

### Pitfall 3: README troubleshooting section references blob.handy.computer model URLs
**What goes wrong:** The existing README has a full manual model installation section (lines 302–395) with blob.handy.computer URLs. Per CONTEXT.md decisions, remove manual model download URLs from README.
**How to avoid:** Remove the entire manual model installation section from README. The app handles downloads. Proxy/network restriction users will need to wait for INFR-01 (V2).

### Pitfall 4: Linux install instructions in BUILD.md use "Handy" binary paths
**What goes wrong:** BUILD.md lines 82–99 have `handy` binary references throughout Linux install steps. These are technically correct (binary IS named handy) but appear to be Handy branding without a note.
**How to avoid:** Keep the `handy` binary references accurate but add the V2 rename note. Do not change `handy` to `dictus` in binary path examples — that would make the instructions wrong.

### Pitfall 5: README Acknowledgments section vs. inline attribution
**What goes wrong:** Placing fork attribution ("based on Handy") in multiple places (intro, footer, every section) makes the document feel like it's still Handy's README.
**How to avoid:** Single dedicated Acknowledgments section with the fork attribution. Lead with Dictus identity throughout, consolidate Handy attribution at the end.

## Code Examples

### Current AboutSettings.tsx hardcoded URLs (lines to replace)

```typescript
// Line 32 — change to:
await openUrl("https://getdictus.com/donate");

// Line 69 — change to:
onClick={() => openUrl("https://github.com/getdictus/dictus-desktop")}
```

### New i18n keys to add under settings.about in EN translation.json

```json
"privacy": {
  "title": "Privacy Policy",
  "description": "Your voice stays on your device. Read our privacy policy.",
  "button": "View Privacy Policy"
},
"ecosystem": {
  "title": "Dictus Ecosystem",
  "description": "Dictus is available on iOS, Android, and Desktop."
}
```

### CLAUDE.md line to update (line 32)

```bash
# Current (remove this):
curl -o src-tauri/resources/models/silero_vad_v4.onnx https://blob.handy.computer/silero_vad_v4.onnx

# Replace with:
curl -o src-tauri/resources/models/silero_vad_v4.onnx https://blob.handy.computer/silero_vad_v4.onnx
# Note: This URL is from the upstream Handy CDN. A Dictus-owned CDN migration is planned (V2 INFR-01).
```

### BUILD.md clone URL to replace (line 55-56)

```bash
# Current:
git clone git@github.com:cjpais/Handy.git
cd Handy

# Replace with:
git clone git@github.com:getdictus/dictus-desktop.git
cd dictus-desktop
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| All i18n about.* keys said "Handy" | Most about.* keys now say "Dictus" | Phase 2 (VISU-04) | Only URL fixes + new keys needed; no key renaming required |
| AboutSettings.tsx had Handy links | Still has Handy URLs (lines 32, 69) | Not yet changed | Two targeted replacements needed |

**Observation:** The English and French translation.json files already have `settings.about.*` keys in Dictus form (confirmed by reading lines 512–539). Phase 2 (VISU-04) handled i18n string replacement. The About panel TSX was not part of Phase 2 scope. This means:
- No i18n key renaming needed
- Only URL fixes + new content keys for privacy/ecosystem additions

## Open Questions

1. **VAD model dev setup alternative**
   - What we know: `blob.handy.computer/silero_vad_v4.onnx` is the only documented source in CLAUDE.md
   - What's unclear: Whether the file is available from an authoritative open-source source (Silero's HuggingFace repo) that could replace the CDN URL
   - Recommendation: Keep the existing URL with a V2 migration note; do not break dev setup for the sake of docs polish. Planner can optionally add a secondary source note pointing to Silero's HuggingFace repo as an alternative.

2. **Troubleshooting section in README**
   - What we know: The existing troubleshooting section (Linux, Wayland, custom models) is broadly accurate for Dictus Desktop since it's the same technical stack
   - What's unclear: Whether to keep it in full, slim it down, or drop it
   - Recommendation: Marked as Claude's Discretion — keep the technically-valid content (Linux Wayland, signal flags), drop sections that reference blob.handy.computer URLs or Handy-specific paths.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | None detected — no test config files, no test directories found in project |
| Config file | none |
| Quick run command | `bun run lint` (ESLint + i18n enforcement) |
| Full suite command | `bun run lint && bun run format:check` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DOCS-01 | README describes Dictus Desktop with no Handy references in main content | manual | grep audit: `grep -i "handy" README.md` — should return only Acknowledgments section | n/a |
| DOCS-02 | CLAUDE.md and BUILD.md free of Handy branding text | manual | `grep -i "handy" CLAUDE.md BUILD.md` — should return only binary path references with disclaimer notes | n/a |
| DOCS-03 | About panel shows Dictus branding, new links, no Handy URLs | lint + manual | `bun run lint` validates no literal strings; manual URL check | ❌ Wave 0 — no test files |

### Sampling Rate
- **Per task commit:** `bun run lint` — validates no i18n literal strings in TSX
- **Per wave merge:** `bun run lint && bun run format:check`
- **Phase gate:** Lint green + manual grep audit of docs before `/gsd:verify-work`

### Wave 0 Gaps
- No automated test infrastructure needed for documentation changes
- The `bun run lint` command provides the only automatable check (i18n enforcement on TSX)
- Manual grep audits are the verification method for markdown files

## Sources

### Primary (HIGH confidence)
- Direct file read: `src/components/settings/about/AboutSettings.tsx` — confirmed Handy URLs at lines 32, 69
- Direct file read: `src/i18n/locales/en/translation.json` lines 512–539 — confirmed existing about.* keys structure
- Direct file read: `src/i18n/locales/fr/translation.json` lines 512–539 — confirmed FR translations already use "Dictus"
- Direct file read: `CLAUDE.md` — confirmed 3 Handy references (lines 32, 37, 124)
- Direct file read: `BUILD.md` — confirmed Handy clone URL and binary path references
- Direct file read: `README.md` — confirmed 457-line Handy document, full rewrite scope

### Secondary (MEDIUM confidence)
- CONTEXT.md canonical refs section — brand URLs and ecosystem links

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries already in project, zero new dependencies
- Architecture: HIGH — files directly inspected, exact line numbers confirmed
- Pitfalls: HIGH — ESLint rules confirmed in CLAUDE.md, i18n patterns confirmed in existing code

**Research date:** 2026-04-09
**Valid until:** Stable — no external dependencies; valid until INFR-01 or TECH-03 land
