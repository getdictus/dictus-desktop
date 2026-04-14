---
created: 2026-04-14
title: Privacy / local-first UX audit
area: ui
files:
  - src/components/settings/
  - src-tauri/src/settings.rs:524-600
  - src-tauri/src/llm_client.rs
---

## Problem

Dictus's core value proposition is local-first processing for privacy, but post-processing inherited from Handy upstream exposes multiple cloud providers (OpenAI, Anthropic, Groq, Gemini) alongside local options (Ollama, Apple Intelligence). Current UX has not been audited against the local-first principle.

Surfaced during Phase 5 upstream sync when reviewing AWS Bedrock commit (#1288, commit `aee682f`). That commit was skipped per local-first policy, but the underlying UX question remains: users should see local providers as the obvious/default path, with cloud providers clearly labeled as opt-in external services.

Also need to document the network call surface so users have transparency about what stays local vs what goes to external services.

## Solution

TBD. Likely phases:

1. **Audit current settings UI**: screenshot the post-process provider picker and related settings, note ordering, visual hierarchy, labeling.
2. **Audit network surface**: grep for all reqwest/fetch calls in Rust + frontend, list all external endpoints the app can call, document in a user-facing privacy doc.
3. **Design proposal**: reorder provider list to put local first (Ollama, Apple Intelligence), visually group cloud providers under a "External (advanced)" section with clear disclosure about data leaving the device.
4. **Onboarding copy**: make sure first-run experience emphasizes local transcription as the primary path.

No urgent fix needed — transcription itself is already 100% local and post-processing is opt-in, default off. This is a positioning/clarity improvement, not a privacy regression.
