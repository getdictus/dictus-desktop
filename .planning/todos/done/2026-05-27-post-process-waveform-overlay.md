---
created: 2026-05-27T13:00:00.000Z
title: Animate the post-processing pill in the recording overlay
area: ui/overlay
files:
  - src/overlay/RecordingOverlay.tsx
  - src/overlay/recording-overlay.css
  - src/i18n/locales/en/translation.json
related:
  - .planning/phases/08-privacy-local-first-ux/08-CONTEXT.md (L226 — defer note)
  - .planning/phases/08-privacy-local-first-ux/08-RESEARCH.md (L101 — track-as-todo note)
deferred_from: phase-08-privacy-local-first-ux
informally_attached_to: milestone-v1.2
---

## Problem

When post-processing runs (LLM clean-up of the transcript), the recording overlay
shows only a static text indicator (`overlay.processing` i18n key). The recording
and transcribing states both render a lively bars-container waveform animation
(`RecordingOverlay.tsx:161-174`), but the post-processing state has no equivalent
visual feedback — it looks frozen and the user cannot tell whether the app is
actually working or hung.

Pierre flagged this during Phase 8 planning. The deferral was documented in
`08-CONTEXT.md:226` and `08-RESEARCH.md:101` ("track as a todo for v1.3 milestone
or later") but the todo file was never actually created. Captured now post-Phase-8
and informally rolled into the v1.2 polish window — to be executed before
`/gsd:complete-milestone` flips v1.2 to ✅.

## Scope (kept narrow on purpose)

**In scope:**

- Replace the static `t("overlay.processing")` text with an animated indicator
  when `state === "processing"` in `RecordingOverlay.tsx:175-177`
- Keep the existing window dimensions and position — no resize, no new window
- Visual idiom should read as "we are working on it" without screaming
  (think: subtle pulsing dots, slow shimmer bar, or a low-amplitude waveform —
  NOT the recording bars which imply audio capture)
- Respect brand: animation accent color uses `--logo-primary` / `text-logo-primary`
- Honour `prefers-reduced-motion` — fall back to a static dot or label when set
- Keep i18n: any text changes propagate via the standard 19-locale sweep

**Out of scope:**

- New Rust events — the `processing` state already flows end-to-end
  (`OverlayState` type at `RecordingOverlay.tsx:10`, set via `setState(overlayState)`
  at line 95)
- Changing overlay window chrome, position, draggability, or close behaviour
- Animation for any state other than `processing`
- Backend timing instrumentation (separate concern if we ever want to expose
  expected duration to drive a determinate progress bar)

## Open questions for grilling

These are the questions `/grill-with-docs` should crystallise before code is written:

1. **Visual idiom** — pulsing dots vs shimmer bar vs low-amplitude waveform vs spinner?
   What signals "AI thinking" without implying audio capture?
2. **Flash-prevention threshold** — post-processing for a short transcript can finish
   in <300ms. Does the animation appear immediately, or after a 150ms delay to avoid
   a visual flicker on fast turns?
3. **Cancel affordance** — `recording` state shows a cancel pill (line 181). Should
   `processing` also be cancellable? (Backend cancel during post-process — supported?)
4. **Failure surface** — if post-processing errors out, where does the user see it?
   Today: silently falls back to raw transcript. Should the overlay flash an error
   state before closing?
5. **Reduced motion** — what's the static fallback? A dim "Post-traitement en cours…"
   text reads as frozen; a single static dot reads as broken. Find a third option.
6. **Branding consistency** — does this animation set precedent for other "AI
   working" states elsewhere in the app (model download, future translation mode)?
   Worth defining a reusable component, or premature?

## Acceptance criteria (draft — refine during grilling)

- [ ] When `state === "processing"`, the overlay renders an animation (not a
      static text)
- [ ] Animation uses brand `--logo-primary` accent
- [ ] Animation respects `prefers-reduced-motion`
- [ ] No visual flicker for post-processing runs shorter than the chosen threshold
- [ ] No new i18n keys without 19-locale propagation
- [ ] `bun run build` + `bun run check:translations` + `cargo fmt --check` all green
- [ ] Manual smoke test in `bun run tauri dev`: trigger transcribe+post-process,
      confirm animation visible and pleasant in both light and dark themes

## Priority / timing

- **Blocks:** clean ship of v1.2 "Polish & Local-First UX" — the milestone name
  promises polish, and a frozen post-processing pill is the most-visible UX rough
  edge left
- **Does NOT block:** v0.1.x patch releases (post-process UX is already shipped
  with the static indicator)
- **Good candidate for:** ~2 hour focused session post-grilling

## Definition of done

- Animation lands on `feat/phase-08-local-first-ux` (or follow-up branch) with a
  single coherent commit `feat(overlay): animate post-processing pill`
- This todo file moves to `.planning/todos/done/`
- `/gsd:audit-milestone v1.2` and `/gsd:complete-milestone` run cleanly afterwards
