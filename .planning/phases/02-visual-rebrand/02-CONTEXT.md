# Phase 2: Visual Rebrand - Context

**Gathered:** 2026-04-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Replace all visible Handy identity with Dictus across all supported platforms (macOS, Windows, Linux) and all four locales (en, es, fr, vi). This includes: app icon, in-app logo, design tokens (colors), i18n strings, UI components (sidebar, onboarding), and recording overlay. No new features — only visual identity transformation.

</domain>

<decisions>
## Implementation Decisions

### App icon

- Use `AppIcon-1024.png` from `dictus-brand/ios/AppIcon.appiconset/` as the source image
- Feed it to `tauri icon` to generate all platform-specific sizes (icns, ico, PNGs)
- Same icon across all platforms — no desktop-specific variant

### In-app logo

- Waveform icon + "Dictus" wordmark (replacing HandyTextLogo and HandyHand components)
- Create a DictusLogo component that renders the 3-bar waveform SVG alongside the "Dictus" text
- Style reference: `dictus-brand/source/dictus-brand-kit.html` — match the iOS brand kit aesthetic
- Light variant (dark bars on light bg) and dark variant (white/blue bars on dark bg) using existing SVGs from `dictus-brand/source/logo-light.svg` and `logo-dark.svg`

### Color palette

- Replace Handy pink/rose palette with Dictus blue palette from the brand kit
- Primary accent: `#3D7EFF` (used for active sidebar, interactive highlights, glow effects)
- Gradient: `#6BA3FF` → `#2563EB` (waveform bars, progress indicators)
- Dark mode background: `#0A1628` (blue-900 from brand kit)
- Light mode: white/light background with blue accents (keep existing light structure)
- Both light and dark themes supported (follow system preference as currently implemented)
- Design tokens updated in `App.css` via Tailwind v4 `@theme` block — single source of truth
- All hardcoded pink hex values replaced: `#FAA2CA`, `#da5893`, `#ffe5ee`, `#f28cbb`, `#fad1ed`, `#F9C5E8`

### Typography

- Keep system font stack (-apple-system, Segoe UI, etc.) — no DM Sans bundling
- Desktop apps should feel platform-native; the brand kit's DM Sans is for web/marketing only

### i18n strings & tone

- Match the Dictus iOS app tone: warm, direct, privacy-aware
- Examples of the iOS tone to match:
  - "Dictus needs the microphone to transcribe your voice. Your recordings stay on your device."
  - "Dictus is set up and ready to use"
  - "We'd love to skip this step, but iOS requires opening Dictus to activate the microphone."
- Replace all ~12 "Handy" references in translation files with "Dictus" and adapt wording
- Claude translates all 4 locales (en, es, fr, vi) with consistent branding and tone

### Recording overlay

- Significantly larger than current Handy overlay (~300x56px vs current 172x36px)
- 15-20 waveform bars (up from current 3-4) — closer to iOS's 30-bar waveform feel
- Bar colors: blue gradient center bars (#6BA3FF → #2563EB), white/gray edge bars with opacity falloff — matching iOS BrandWaveform color scheme
- Sinusoidal wave animation during transcription state — replicate iOS ProcessingAnimation behavior (traveling sine wave across bars, ~1.5s loop)
- Recording state: bars animate from real audio energy with smoothing/decay
- State colors matching iOS: red (#EF4444) for recording/cancel, green (#22C55E) flash on success
- Dark pill background retained, just larger and with more visual richness
- Cancel button hover uses blue accent instead of pink

### Claude's Discretion

- Exact overlay dimensions and bar spacing (within the "larger, 15-20 bars" constraint)
- Waveform animation parameters (smoothing, decay factors) — optimize for desktop 60fps
- Wordmark font weight and sizing in the DictusLogo component
- Exact dark mode color mappings for secondary UI elements (borders, hover states, scrollbar)
- How to handle the sidebar HandyHand icon replacement (waveform icon vs lucide icon)

</decisions>

<canonical_refs>

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Brand assets & design system

- `../../../dictus-brand/source/dictus-brand-kit.html` — Complete Dictus brand kit: color palette (blue-100 through blue-900), accent values, typography specs, logo showcase
- `../../../dictus-brand/source/appicon-light.svg` — Waveform logo for light backgrounds (dark bars)
- `../../../dictus-brand/source/appicon-dark.svg` — Waveform logo for dark backgrounds (white/blue bars)
- `../../../dictus-brand/source/logo-light.svg` — Standalone logo for light surfaces
- `../../../dictus-brand/source/logo-dark.svg` — Standalone logo for dark surfaces
- `../../../dictus-brand/ios/AppIcon.appiconset/AppIcon-1024.png` — Source icon for `tauri icon` generation

### iOS reference implementation (recording UI)

- `../../../dictus/DictusCore/Sources/DictusCore/Design/BrandWaveform.swift` — 30-bar waveform: colors, animation parameters (smoothing 0.3, decay 0.85), bar sizing, center-gradient logic
- `../../../dictus/DictusCore/Sources/DictusCore/Design/ProcessingAnimation.swift` — Sinusoidal transcription animation: sine wave formula, phase increment, energy range 0.2-0.7
- `../../../dictus/DictusCore/Sources/DictusCore/Design/AnimatedMicButton.swift` — State colors: recording red #EF4444, success green #22C55E, accent blue #3D7EFF
- `../../../dictus/DictusCore/Sources/DictusCore/Design/DictusColors.swift` — Full color palette definitions
- `../../../dictus/DictusApp/Views/RecordingView.swift` — Recording view layout and state opacity transitions

### iOS i18n reference

- `../../../dictus/DictusApp/Localizable.xcstrings` — iOS string tone reference for matching desktop copy style

</canonical_refs>

<code_context>

## Existing Code Insights

### Reusable Assets

- `src/App.css` @theme block: Single source of truth for design tokens — swap pink values for blue here
- `src/components/icons/HandyTextLogo.tsx`: SVG component to replace with DictusLogo (same pattern, different SVG)
- `src/components/icons/HandyHand.tsx`: SVG icon component to replace with Dictus waveform icon
- `src/overlay/RecordingOverlay.css`: Overlay styling — expand dimensions, update colors
- `src-tauri/icons/`: All platform icons generated by `tauri icon` — replace source and regenerate

### Established Patterns

- Tailwind v4 @theme for design tokens — all color changes flow through CSS variables
- i18n via i18next with ESLint enforcement — all user-facing strings through translation keys
- SVG icon components as React components (default export, width/height/className props)
- Dark mode via `prefers-color-scheme` media query in App.css

### Integration Points

- `src/components/Sidebar.tsx` lines 4-5, 37, 97: Imports and uses HandyTextLogo + HandyHand
- `src/components/onboarding/Onboarding.tsx` line 7, 94: Uses HandyTextLogo
- `src/components/onboarding/AccessibilityOnboarding.tsx` line 13, 311: Uses HandyTextLogo
- `src/components/icons/MicrophoneIcon.tsx` line 13: Default color `#FAA2CA`
- `src/components/icons/TranscriptionIcon.tsx` line 13: Default color `#FAA2CA`
- `src/components/icons/CancelIcon.tsx` line 13: Default color `#FAA2CA`
- `src/components/ui/AudioPlayer.tsx` line 263: Hardcoded pink in progress bar gradient
- `src/i18n/locales/{en,es,fr,vi}/translation.json`: ~12 Handy references per locale

</code_context>

<specifics>
## Specific Ideas

- "I want to reproduce the iOS user experience as much as possible" — especially for recording/transcription overlay
- The overlay must be "significantly bigger" than current Handy — user finds it too small
- The sinusoidal wave during transcription is a key visual element to replicate
- The waveform bar motif (3 bars in logo, 30 in iOS, 15-20 in desktop overlay) should be a consistent Dictus visual identity element
- Brand assets live in `dictus-brand` repo (local at `../dictus-brand`). Any generated assets should also be contributed back to that repo.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

_Phase: 02-visual-rebrand_
_Context gathered: 2026-04-08_
