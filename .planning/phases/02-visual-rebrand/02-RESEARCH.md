# Phase 02: Visual Rebrand - Research

**Researched:** 2026-04-08
**Domain:** React/TypeScript frontend visual identity, Tauri icon generation, CSS design tokens, i18n string replacement, CSS animation
**Confidence:** HIGH

---

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions

**App icon:** Use `AppIcon-1024.png` from `dictus-brand/ios/AppIcon.appiconset/` as source. Run `tauri icon` to generate all platform sizes. Same icon all platforms.

**In-app logo:** Create `DictusLogo` component — 3-bar waveform SVG + "Dictus" wordmark — replacing `HandyTextLogo` and `HandyHand`. Use existing SVGs from `dictus-brand/source/logo-standalone-light.svg` and `appicon-dark.svg`. Light/dark variants.

**Color palette:**

- Primary accent: `#3D7EFF`
- Gradient: `#6BA3FF` → `#2563EB`
- Dark mode background: `#0A1628`
- Design tokens updated in `App.css` via Tailwind v4 `@theme` block
- Replace all hardcoded pink: `#FAA2CA`, `#da5893`, `#ffe5ee`, `#f28cbb`, `#fad1ed`, `#F9C5E8`

**Typography:** Keep system font stack. No DM Sans bundling.

**i18n:** Replace all ~12 "Handy" references per locale in all 4 locales (en, es, fr, vi). Match Dictus iOS tone. Claude translates all locales.

**Recording overlay:**

- Larger: ~300×56px (vs current 172×36px)
- 15–20 waveform bars (vs current 9 bars sliced from 16)
- Bar colors: blue gradient center bars (`#6BA3FF` → `#2563EB`), white/gray edge bars with opacity falloff
- Sinusoidal wave animation during transcribing state (~1.5s loop)
- Recording state: bars animate from real audio energy with smoothing/decay
- State colors: red `#EF4444` for recording/cancel, green `#22C55E` flash on success
- Dark pill background retained
- Cancel hover uses blue accent not pink

### Claude's Discretion

- Exact overlay dimensions and bar spacing (within 15–20 bar constraint)
- Waveform animation parameters (smoothing, decay) — optimize for desktop 60fps
- Wordmark font weight and sizing in DictusLogo
- Exact dark mode color mappings for secondary UI elements (borders, hover states, scrollbar)
- Sidebar HandyHand icon replacement approach (waveform icon vs lucide icon)

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>

## Phase Requirements

| ID      | Description                                                                        | Research Support                                                                                  |
| ------- | ---------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| VISU-01 | App icon generated for all platforms via `tauri icon`                              | `tauri icon` CLI confirmed; source file at `dictus-brand/ios/AppIcon.appiconset/AppIcon-1024.png` |
| VISU-02 | Logo components replaced (HandyTextLogo → DictusLogo, HandyHand → Dictus waveform) | Brand SVGs exist in `dictus-brand/source/`; component pattern documented                          |
| VISU-03 | Sidebar branding updated                                                           | `Sidebar.tsx` lines 4-5, 37, 97 identified; uses HandyTextLogo + HandyHand                        |
| VISU-04 | i18n strings updated in all locale files                                           | 11 "Handy" references confirmed per locale × 4 locales (en, es, fr, vi)                           |
| VISU-05 | Dictus design tokens injected via Tailwind v4 @theme                               | `App.css` @theme block is single source of truth; 6 pink tokens to replace                        |
| VISU-06 | Color palette switched from pink to blue throughout UI                             | 7 files with hardcoded pink identified                                                            |
| VISU-07 | Recording overlay redesigned in Dictus style                                       | Current overlay analyzed; iOS BrandWaveform parameters extracted                                  |
| VISU-08 | Animations aligned with Dictus identity                                            | iOS ProcessingAnimation + BrandWaveformDriver parameters extracted                                |
| ONBR-01 | Onboarding rebranded with Dictus text, visuals, tone                               | `Onboarding.tsx` and `AccessibilityOnboarding.tsx` identified                                     |
| LANG-01 | Force language UX clarified: Auto / Français / English clearly presented           | `AppLanguageSelector.tsx` + `SUPPORTED_LANGUAGES` array identified                                |

</phase_requirements>

---

## Summary

Phase 2 is a pure visual identity swap — no new features. The codebase is well-structured for this work: all colors flow through a single `App.css` @theme block (Tailwind v4), all strings go through i18next, and icon components follow a consistent SVG-component pattern. The work decomposes naturally into five areas: (1) app icon regeneration, (2) logo component replacement, (3) color token swap, (4) i18n string replacement, and (5) recording overlay redesign.

The iOS reference implementation (BrandWaveform.swift, ProcessingAnimation.swift, DictusColors.swift) provides exact numeric parameters for the overlay animation — smoothing factor 0.3, decay factor 0.85, sine formula `0.2 + 0.25 * (sin(2π * (index/N + phase)) + 1.0)`, recording state color `#EF4444`, success green `#22C55E`. These translate directly to CSS/JS.

The LANG-01 requirement is addressed by the existing `AppLanguageSelector` component using `SUPPORTED_LANGUAGES` from `i18n/languages.ts`. The issue is that the dropdown currently shows all 20 languages with the label "Application Language / Change the language of the Handy interface" — both strings need updating to Dictus branding, and the description needs to explain the Auto / Français / English choice clearly.

**Primary recommendation:** Execute in five parallel-safe work streams — icon, logo components, color tokens, i18n strings, overlay — then validate all four locales visually.

---

## Standard Stack

### Core (already in project — no new installs)

| Library                 | Version | Purpose                          | Why Standard                                                 |
| ----------------------- | ------- | -------------------------------- | ------------------------------------------------------------ |
| Tailwind CSS v4         | 4.x     | Design tokens via `@theme`       | Already configured; `@theme` block is single source of truth |
| i18next + react-i18next | current | All user-visible strings         | ESLint-enforced; already handles en/es/fr/vi                 |
| React                   | 18.x    | Component authoring              | Project baseline                                             |
| Tauri CLI               | 2.x     | `tauri icon` for icon generation | Already installed                                            |

### No New Dependencies

This phase requires zero new npm or cargo dependencies. All work is:

- SVG markup in React components
- CSS custom property updates
- JSON string file edits
- Running `tauri icon` CLI command

---

## Architecture Patterns

### Recommended Work Structure

```
Work Stream A — App Icon (VISU-01)
  1. Run: bunx tauri icon ../dictus-brand/ios/AppIcon.appiconset/AppIcon-1024.png
  2. Verify src-tauri/icons/ has new files

Work Stream B — Logo Components (VISU-02, VISU-03)
  1. Create src/components/icons/DictusLogo.tsx (replaces HandyTextLogo)
  2. Create src/components/icons/DictusWaveformIcon.tsx (replaces HandyHand)
  3. Update Sidebar.tsx imports/usages
  4. Update Onboarding.tsx import/usage
  5. Update AccessibilityOnboarding.tsx import/usage

Work Stream C — Color Tokens (VISU-05, VISU-06)
  1. Update App.css @theme block (6 token changes)
  2. Update App.css dark mode block
  3. Update 5 component files with hardcoded pink
  4. Update RecordingOverlay.css bar color and cancel hover

Work Stream D — i18n Strings (VISU-04, ONBR-01, LANG-01)
  1. Replace 11 "Handy" references in en/translation.json with Dictus tone
  2. Replace + translate all 4 locales (es, fr, vi)
  3. Update appLanguage description for LANG-01

Work Stream E — Recording Overlay (VISU-07, VISU-08)
  1. Update RecordingOverlay.css dimensions
  2. Rewrite waveform bars logic in RecordingOverlay.tsx
  3. Add sinusoidal animation for transcribing state
  4. Update icon colors (MicrophoneIcon, CancelIcon, TranscriptionIcon)
```

### Pattern 1: Tailwind v4 @theme Token Replacement

**What:** Single-source design token swap in `App.css`
**When to use:** Any color change — do NOT add CSS variables elsewhere

Current `@theme` block to replace:

```css
/* BEFORE */
--color-background-ui: #da5893;
--color-logo-primary: #faa2ca;
--color-logo-stroke: #382731;

/* AFTER */
--color-accent: #3d7eff;
--color-accent-gradient-start: #6ba3ff;
--color-accent-gradient-end: #2563eb;
--color-logo-primary: #6ba3ff; /* used by .logo-primary */
--color-logo-stroke: #0a1628; /* dark text on logo bars */
--color-text-stroke: #f6f6f6; /* unchanged */
--color-mid-gray: #808080; /* unchanged */
```

Dark mode block:

```css
/* AFTER */
@media (prefers-color-scheme: dark) {
  :root {
    --color-text: #fbfbfb;
    --color-background: #0a1628; /* Dictus dark bg */
    --color-logo-primary: #6ba3ff;
    --color-logo-stroke: #ffffff;
  }
}
```

### Pattern 2: DictusLogo SVG Component

**What:** React component wrapping the 3-bar waveform SVG + "Dictus" wordmark
**Source:** `dictus-brand/source/logo-standalone-light.svg` (3 bars, viewBox 0 0 80 80)

SVG bar structure (from brand files):

```tsx
// Source: dictus-brand/source/logo-standalone-light.svg + appicon-dark.svg
// Light variant: dark-tinted bars
// Bar 1 (left, short): x=19 y=31 w=9 h=18 rx=4.5  fill="#0A1628" opacity=0.45
// Bar 2 (center, tall): x=35.5 y=19 w=9 h=42 rx=4.5  fill=url(#bar-accent)
// Bar 3 (right, medium): x=52 y=26 w=9 h=27 rx=4.5  fill="#0A1628" opacity=0.65

// Dark variant: white bars with blue center
// Bar 1: fill="white" opacity=0.45
// Bar 2: fill=url(#bar-accent)  (#6BA3FF → #2563EB gradient)
// Bar 3: fill="white" opacity=0.65
```

Component interface (matches existing HandyTextLogo pattern):

```tsx
const DictusLogo = ({
  width,
  className,
}: {
  width?: number | string;
  className?: string;
}) => {
  /* ... */
};
export default DictusLogo;
```

### Pattern 3: Waveform Overlay Animation

**What:** CSS animation + requestAnimationFrame loop for 15–20 bar waveform
**Source:** Translated from BrandWaveform.swift + BrandWaveformDriver.swift

Recording state (real audio energy):

```typescript
// From BrandWaveformDriver.swift: smoothingFactor=0.3, decayFactor=0.85
// rise: smoothed[i] = prev + (target - prev) * 0.3
// fall: smoothed[i] = target + (prev - target) * 0.85
// threshold: if smoothed[i] < 0.005, set to 0
```

Transcribing state (sine wave):

```typescript
// From BrandWaveformDriver.processingEnergy():
// energy(index) = 0.2 + 0.25 * (sin(2π * (index/(N-1) + phase)) + 1.0)
// phase increments each frame: phase += dt / 2.0  (iOS link.duration ~= 1/60s)
// Result: ~1.5s for full wave cycle (phase 0→1 takes ~30 frames at 60fps = 0.5s per cycle)
// Desktop: phase += 0.5 * dt (where dt is seconds since last frame)
```

Bar color logic (from BrandWaveform.swift resolvedBarColor):

```typescript
// distanceFromCenter = |index - center| / center
// if distanceFromCenter < 0.4: blue gradient center (#6BA3FF)
// else: white/gray with opacity = (1.0 - distanceFromCenter) * 0.9 + 0.15
//   dark mode: white    light mode: gray
```

Overlay CSS target dimensions:

```css
.recording-overlay {
  height: 56px; /* up from 36px */
  width: 300px; /* up from 172px */
  border-radius: 28px; /* pill: height/2 */
}

.bars-container {
  height: 36px; /* room for taller bars */
  gap: 2px; /* from BrandWaveform.barSpacing */
}
```

### Pattern 4: i18n String Replacement

**What:** Replace "Handy" with "Dictus" + adapt tone to match iOS Localizable.xcstrings warmth

11 locations per locale file (identical keys across en/es/fr/vi):

```
onboarding.permissions.description
settings.general.shortcut.title   ("Handy Shortcuts" → "Dictus Shortcuts")
settings.advanced.autostart.description
settings.advanced.showTrayIcon.description
settings.debug.updateChecks.description
settings.about.version.description
settings.about.appDataDirectory.description
settings.about.supportDevelopment.description
settings.about.acknowledgments.whisper.details
accessibility.permissionsDescription
appLanguage.description
```

iOS tone examples to match:

- "Dictus needs the microphone to transcribe your voice. Your recordings stay on your device."
- "Dictus is set up and ready to use"
- Warm, direct, privacy-aware — not corporate

### Pattern 5: LANG-01 — Force Language UX

**What:** The `AppLanguageSelector` component already works correctly. LANG-01 requires:

1. Rename "Handy Shortcuts" → "Dictus Shortcuts" (same key in all locales)
2. Update `appLanguage.description` to read clearly: "Choose the language for the Dictus interface. Auto follows your system language."
3. Ensure the dropdown label text `lang.nativeName` renders "Français", "English" etc. clearly — it already does via `SUPPORTED_LANGUAGES`

No code changes needed to `AppLanguageSelector.tsx` beyond string updates.

### Anti-Patterns to Avoid

- **Adding CSS variables outside `App.css` @theme:** Breaks the single source of truth. All new color tokens go in the `@theme` block only.
- **Hardcoding colors in component props:** `MicrophoneIcon`, `CancelIcon`, `TranscriptionIcon` have `color` props defaulting to `#FAA2CA` — change the default, don't override at call sites.
- **Importing both old and new logo components:** Remove `HandyTextLogo` and `HandyHand` imports entirely — don't keep them as aliases.
- **Using `setInterval` for animation:** Use `requestAnimationFrame` for the waveform loop — matches the iOS CADisplayLink approach and avoids frame-doubling.
- **Forgetting the overlay window i18n:** `RecordingOverlay.tsx` uses `t("overlay.transcribing")` and `t("overlay.processing")` — these strings don't contain "Handy" but the overlay syncs language on each show via `syncLanguageFromSettings()`.

---

## Don't Hand-Roll

| Problem                | Don't Build             | Use Instead                               | Why                                                                           |
| ---------------------- | ----------------------- | ----------------------------------------- | ----------------------------------------------------------------------------- |
| Platform icon sizes    | Custom resize script    | `bunx tauri icon <source>`                | Generates all 20+ platform-specific sizes (icns, ico, PNGs) in one command    |
| Color scheme detection | Media query JS listener | CSS `@media (prefers-color-scheme: dark)` | Already implemented; adding JS would duplicate state                          |
| Animation timing       | `setInterval`           | `requestAnimationFrame`                   | Syncs with display refresh; pauses when tab hidden; matches iOS CADisplayLink |
| Translation QA         | Visual scan             | i18next `t()` + ESLint rule               | ESLint already enforces no hardcoded JSX strings                              |

---

## Common Pitfalls

### Pitfall 1: `tauri icon` Overwrites All Icons

**What goes wrong:** Running `tauri icon` replaces all 20+ files in `src-tauri/icons/`. Any manually placed icons are gone.
**Why it happens:** It's a full regeneration, not incremental.
**How to avoid:** Run `tauri icon` first as the single action for VISU-01 — don't manually place any PNG files before or after.
**Warning signs:** After running, check `icon.icns` and `icon.ico` to confirm they changed (file size or date).

### Pitfall 2: Tailwind v4 `@theme` Token Names Must Match Utility Classes

**What goes wrong:** Renaming `--color-logo-primary` to `--color-accent` breaks `bg-logo-primary/80` usage in `Sidebar.tsx` line 107.
**Why it happens:** Tailwind v4 generates utility classes from `@theme` variable names.
**How to avoid:** Keep `--color-logo-primary` name OR update all usage sites together. The Sidebar uses `bg-logo-primary/80` for active state — this is a Tailwind utility class derived from the token name.
**Warning signs:** Active sidebar item loses its highlight color.

### Pitfall 3: RecordingOverlay Runs in a Separate Window

**What goes wrong:** CSS changes in `App.css` don't affect the overlay — it has its own `RecordingOverlay.css` and separate entry point (`src/overlay/main.tsx`).
**Why it happens:** Tauri overlay window is a separate webview with its own HTML/CSS.
**How to avoid:** All overlay color changes must go in `RecordingOverlay.css`, not `App.css`. The overlay doesn't import Tailwind or `App.css`.
**Warning signs:** Main app looks correct, overlay still has pink bars.

### Pitfall 4: Bar Count Mismatch Between Rust and React

**What goes wrong:** The Rust backend emits `mic-level` events with 16 values (`Array(16)`). The React overlay currently slices to 9 bars. With 15–20 target bars, the interpolation logic must handle the count mismatch.
**Why it happens:** `RecordingOverlay.tsx` line 51: `setLevels(smoothed.slice(0, 9))` — it discards values instead of interpolating.
**How to avoid:** Implement linear interpolation to map N Rust energy values to 15–20 display bars (matching `BrandWaveformDriver.targetLevels()` which does exactly this).
**Warning signs:** Some bars are always at minimum height.

### Pitfall 5: i18n ESLint Rule Blocks Hardcoded Test Strings

**What goes wrong:** If a string is written directly in JSX (e.g., `<p>Dictus</p>`), ESLint will fail with the `no-hardcoded-strings` rule.
**Why it happens:** `CLAUDE.md` documents this ESLint rule — all user-facing strings must use `t()`.
**How to avoid:** The "Dictus" wordmark in the logo SVG is not JSX text — it can be an SVG `<text>` element or a `tspan`. If rendered as JSX text, it must use a translation key.

### Pitfall 6: Only 4 Locales Have Translation Files

**What goes wrong:** Assuming all 20 languages in `LANGUAGE_METADATA` have translation files.
**Why it happens:** `languages.ts` defines metadata for 20 languages, but only en/es/fr/vi have `translation.json` files.
**How to avoid:** Only update the 4 existing locale files (en, es, fr, vi). Other languages fall back to English.

---

## Code Examples

### `tauri icon` Command

```bash
# Source: CLAUDE.md + tauri.conf.json conventions
cd /Users/pierreviviere/dev/dictus-desktop
bunx tauri icon ../dictus-brand/ios/AppIcon.appiconset/AppIcon-1024.png
# Generates: src-tauri/icons/32x32.png, 128x128.png, icon.icns, icon.ico, etc.
```

### Waveform Bar Interpolation (VISU-07)

```typescript
// Translate N Rust levels → BAR_COUNT display bars (from BrandWaveformDriver.targetLevels)
// Source: dictus/DictusCore/Sources/DictusCore/Design/BrandWaveform.swift
function interpolateLevels(source: number[], barCount: number): number[] {
  if (source.length === 0) return Array(barCount).fill(0);
  return Array.from({ length: barCount }, (_, index) => {
    const position = index / Math.max(barCount - 1, 1);
    const arrayIndex = position * (source.length - 1);
    const lower = Math.floor(arrayIndex);
    const upper = Math.min(lower + 1, source.length - 1);
    const fraction = arrayIndex - lower;
    const value = source[lower] * (1 - fraction) + source[upper] * fraction;
    return value < 0.05 ? 0 : Math.min(Math.max(value, 0), 1);
  });
}
```

### Sinusoidal Processing Animation (VISU-08)

```typescript
// Source: dictus/DictusCore/Sources/DictusCore/Design/BrandWaveform.swift processingEnergy()
function processingEnergy(
  index: number,
  barCount: number,
  phase: number,
): number {
  const normalizedIndex = index / Math.max(barCount - 1, 1);
  const sineValue = Math.sin(2 * Math.PI * (normalizedIndex + phase));
  return 0.2 + 0.25 * (sineValue + 1.0);
  // Range: 0.2 to 0.7
}
// phase increments: phase += dt * 0.5 (0.5 cycles per second ≈ 2s full cycle)
```

### Bar Color by Position (VISU-07)

```typescript
// Source: dictus/DictusCore/Sources/DictusCore/Design/BrandWaveform.swift resolvedBarColor()
function getBarColor(index: number, barCount: number, isDark: boolean): string {
  const center = (barCount - 1) / 2;
  const distanceFromCenter = Math.abs(index - center) / center;
  if (distanceFromCenter < 0.4) {
    return "#6BA3FF"; // dictusGradientStart — inner 40%
  }
  const opacity = (1.0 - distanceFromCenter) * 0.9 + 0.15;
  return isDark
    ? `rgba(255,255,255,${opacity})`
    : `rgba(128,128,128,${opacity})`;
}
```

### Smoothing/Decay Loop (VISU-07, VISU-08)

```typescript
// Source: BrandWaveformDriver.tickLevels() — smoothingFactor=0.3, decayFactor=0.85
function tickLevels(current: number[], targets: number[]): number[] {
  return current.map((prev, i) => {
    const target = targets[i] ?? 0;
    let next: number;
    if (target > prev) {
      next = prev + (target - prev) * 0.3; // rise: smooth
    } else {
      next = target + (prev - target) * 0.85; // fall: decay
    }
    return next < 0.005 ? 0 : next;
  });
}
```

### App.css @theme After Swap (VISU-05)

```css
/* Source: dictus-brand/source/dictus-brand-kit.html + DictusColors.swift */
@theme {
  --color-text: #0f0f0f;
  --color-background: #fbfbfb;
  --color-accent: #3d7eff;
  --color-accent-gradient-start: #6ba3ff;
  --color-accent-gradient-end: #2563eb;
  --color-logo-primary: #6ba3ff; /* Tailwind: bg-logo-primary */
  --color-logo-stroke: #0a1628;
  --color-text-stroke: #f6f6f6;
  --color-mid-gray: #808080;
}

@media (prefers-color-scheme: dark) {
  :root {
    --color-text: #fbfbfb;
    --color-background: #0a1628; /* Dictus blue-900 dark bg */
    --color-logo-primary: #6ba3ff;
    --color-logo-stroke: #ffffff;
  }
}
```

---

## State of the Art

| Old Approach            | Current Approach                  | When Changed                   | Impact                                                            |
| ----------------------- | --------------------------------- | ------------------------------ | ----------------------------------------------------------------- |
| Multiple CSS vars files | Tailwind v4 `@theme` single block | v4 (project already uses this) | All color changes in one place                                    |
| `setInterval` animation | `requestAnimationFrame`           | —                              | Must use rAF for waveform smoothness                              |
| ForEach SVG rects       | JS-driven height style            | —                              | Direct DOM style mutation per frame (no React re-render overhead) |

---

## Open Questions

1. **DictusLogo wordmark rendering**
   - What we know: Brand kit uses "Dictus" text alongside 3-bar waveform
   - What's unclear: Whether to use SVG `<text>` element (no i18n issue, scales with SVG) or HTML text next to SVG (easier font weight control but two elements to align)
   - Recommendation: Use SVG `<text>` with `font-family="-apple-system, ..."` for platform-native feel; falls under Claude's Discretion

2. **Sidebar HandyHand replacement**
   - What we know: `HandyHand` is the icon for the "General" nav item in the sidebar (not a logo)
   - What's unclear: Whether to use the waveform SVG icon (brand-consistent) or a generic lucide icon like `Mic` or `Radio`
   - Recommendation: A small 3-bar waveform icon (scaled from brand kit) maintains Dictus identity; falls under Claude's Discretion

3. **`bg-logo-primary/80` token renaming risk**
   - What we know: Sidebar.tsx uses `bg-logo-primary/80` for active item background
   - What's unclear: Whether to rename the token to `bg-accent` or keep `logo-primary` as a semantic alias
   - Recommendation: Keep `--color-logo-primary` as the Tailwind token name to avoid updating all call sites; just update its value from `#faa2ca` to `#6BA3FF`

---

## Validation Architecture

### Test Framework

| Property           | Value                                                                                  |
| ------------------ | -------------------------------------------------------------------------------------- |
| Framework          | None detected — visual rebrand; ESLint + TypeScript type-check are the automated gates |
| Config file        | `eslint.config.js` (ESLint), `tsconfig.json` (TypeScript)                              |
| Quick run command  | `bun run lint && bun run format:check`                                                 |
| Full suite command | `bun run lint && bun run format:check`                                                 |

### Phase Requirements → Test Map

| Req ID  | Behavior                                           | Test Type    | Automated Command                                                | File Exists? |
| ------- | -------------------------------------------------- | ------------ | ---------------------------------------------------------------- | ------------ |
| VISU-01 | `src-tauri/icons/icon.icns` exists and changed     | manual-only  | `ls -la src-tauri/icons/icon.icns` (visual confirm)              | —            |
| VISU-02 | `DictusLogo` component renders without error       | build        | `bun run lint` catches unused imports                            | Wave 0       |
| VISU-03 | Sidebar has no `HandyTextLogo`/`HandyHand` imports | lint/grep    | `grep -r "HandyTextLogo\|HandyHand" src/`                        | —            |
| VISU-04 | No "Handy" in any translation.json                 | lint/grep    | `grep -r "Handy" src/i18n/locales/`                              | —            |
| VISU-05 | No pink hex values in App.css @theme               | grep         | `grep -i "FAA2CA\|da5893\|ffe5ee\|f28cbb\|fad1ed" src/App.css`   | —            |
| VISU-06 | No hardcoded pink in component files               | grep         | `grep -ri "FAA2CA\|da5893\|faa2ca" src/components/ src/overlay/` | —            |
| VISU-07 | Overlay CSS width 300px, bars 15-20                | manual-only  | Visual launch test                                               | —            |
| VISU-08 | Sine wave animation plays during transcribing      | manual-only  | Visual launch test                                               | —            |
| ONBR-01 | Onboarding renders DictusLogo, no "Handy" text     | build + grep | `grep "Handy" src/components/onboarding/`                        | —            |
| LANG-01 | AppLanguageSelector shows correct description      | grep         | `grep "Handy" src/i18n/locales/en/translation.json`              | —            |

### Sampling Rate

- **Per task commit:** `bun run lint && bun run format:check`
- **Per wave merge:** `bun run lint && bun run format:check` + grep for "Handy" and pink hex
- **Phase gate:** Full lint green + grep clean + visual UAT before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] No unit tests exist for this phase — intentional. Visual rebrand is validated by grep + lint + UAT.
- [ ] Grep scripts for "Handy" and pink hex can be run ad-hoc; no test file setup needed.

_(All automated checks are one-liners, not test files. Wave 0 = no new test infrastructure required.)_

---

## Sources

### Primary (HIGH confidence)

- `dictus-brand/source/appicon-light.svg`, `appicon-dark.svg`, `logo-standalone-light.svg` — exact SVG bar coordinates and color values
- `dictus/DictusCore/Sources/DictusCore/Design/BrandWaveform.swift` — smoothing=0.3, decay=0.85, bar color thresholds, sine formula
- `dictus/DictusCore/Sources/DictusCore/Design/ProcessingAnimation.swift` — 3-bar pulse animation parameters
- `dictus/DictusCore/Sources/DictusCore/Design/DictusColors.swift` — canonical hex values for all brand colors
- `src/App.css` — current @theme tokens; confirmed pink values to replace
- `src/overlay/RecordingOverlay.tsx` + `RecordingOverlay.css` — current overlay dimensions and structure
- `src/components/Sidebar.tsx` — confirmed HandyTextLogo + HandyHand usage locations
- `src/i18n/locales/en/translation.json` — 11 "Handy" occurrences confirmed by grep
- `src/components/settings/AppLanguageSelector.tsx` — LANG-01 implementation confirmed

### Secondary (MEDIUM confidence)

- `CLAUDE.md` — confirms `tauri icon` CLI usage and system font stack decision

### Tertiary (LOW confidence)

- None

---

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — all libraries already in project, no new deps
- Architecture: HIGH — all files inspected directly, exact locations confirmed
- iOS animation parameters: HIGH — read directly from Swift source files
- Pitfalls: HIGH — confirmed from code inspection (bar count, separate webview, @theme token names)

**Research date:** 2026-04-08
**Valid until:** 2026-05-08 (stable codebase, no active upstream changes expected)
