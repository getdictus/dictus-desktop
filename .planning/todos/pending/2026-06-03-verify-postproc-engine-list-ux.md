---
created: 2026-06-03T00:00:00.000Z
title: Visually verify Post-processing on-device engine list UX (Phase 11 tail)
area: ui / settings / post-processing
files:
  - src/components/settings/post-processing/LlmLibrarySection.tsx
  - src/components/settings/post-processing/PostProcessingSettings.tsx
---

## Context

Phase 11 was closed with two UI tweaks from commit `c360a62` whose **logic is
code-verified** but whose **rendering was not eyeballed in-app** (author was away
from the test machine). Non-blocking for Phase 12, but must be confirmed visually
before the v1.3 release. Run `CMAKE_POLICY_VERSION_MINIMUM=3.5 bun run tauri dev`,
open Settings → Post-traitement.

## To verify

### 1. Custom (Ollama) — Test-connection button placement

- Select **Custom (Ollama)** in the on-device list.
- The Custom card itself should show only the Ollama tip (no Test button).
- The config block below the list should show, in order: **URL de base** field
  (full width) → **Tester la connexion** button directly **under** the URL → the
  **Modèle** dropdown.
- ✅ Pass: the Test button is under the URL, full-width row, **not clipped** at the
  right edge at any window width (the earlier `horizontal` layout overflowed —
  see the pre-fix screenshot). Resize the window narrower to confirm responsive.

### 2. Engine list ordering + no reordering on selection

Expected stable order, top to bottom:
`Apple Intelligence → downloaded models → downloadable models → perso (imported
GGUF) → Custom (Ollama)`.

- ✅ Selecting a **downloaded** GGUF model as the engine does **not** move it to the
  top (it stays in place; only the "Actif" badge moves to it).
- ✅ Starting a **download** of a downloadable model moves it **up** into the
  downloaded zone (this is the only time a card shifts).
- ✅ An **imported custom GGUF** ("perso") sits after the downloadable catalogue
  models and before the Custom/Ollama card.
- ✅ Switching to the **Cloud** tab shows the same "no reordering on selection"
  behavior (consistency was the point of dropping the active-pin).

## Definition of done

- Both behaviors confirmed visually on at least macOS (ideally also check a
  narrow window for the responsive Test button).
- If anything is off, fix in `LlmLibrarySection.tsx` (ordering: `engineSorted`
  rank function) / `PostProcessingSettings.tsx` (Custom config block layout) and
  re-verify.
- Move this todo to `.planning/todos/done/` once confirmed.
</content>
