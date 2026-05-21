# Phase 8: Privacy / Local-First UX - Research

**Researched:** 2026-05-21
**Domain:** Tauri 2.x + React 18 + TypeScript settings UX, i18next, tauri-specta command bindings, markdown docs
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Promote post-processing out of Experimental (framing)**
- Move `<PostProcessingToggle>` out of the `experimentalEnabled && (<SettingsGroup>...)` block in `src/components/settings/advanced/AdvancedSettings.tsx:60-70` into a regular Advanced group. Planner picks: either a standalone `SettingsGroup` titled "Post-processing" / "Smart text", or fold into an existing group — likely its own group since v1.3 Smart Modes will expand it.
- `post_process_enabled` stays **OFF by default** in `default_post_process_enabled()` (`settings.rs`). This phase makes the *toggle* discoverable, not the *feature* default-on.
- Other entries currently in Experimental group (`KeyboardImplementationSelector`, `AccelerationSelector`, `LazyStreamClose`) **stay where they are**. Only `PostProcessingToggle` moves.

**Provider list contents and ordering (PRIV-01)**
- **Keep all 8 current providers as-is.** Do NOT add Ollama as a distinct provider. Do NOT add Gemini. Do NOT drop Z.AI/OpenRouter/Cerebras. The literal text of PRIV-01 ("Ollama, Apple Intelligence, Custom local | OpenAI, Anthropic, Groq, Gemini") was written without checking actual code state and is **explicitly overridden**.
- **Local group order (top):** Apple Intelligence (macOS ARM64 only), `Custom (local)`.
- **External group order (below):** keep current code order — OpenAI, Z.AI, OpenRouter, Anthropic, Groq, Cerebras. Preserves upstream Handy ordering and avoids gratuitous churn.
- **Relabel `Custom` → `Custom (local)`.** Single label change across i18n keys. The provider id (`"custom"`) stays unchanged so persisted settings survive.
- **Default provider when post-processing is first enabled:** Apple Intelligence on `cfg(all(target_os = "macos", target_arch = "aarch64"))`, else `Custom (local)`. Change `default_post_process_provider_id()` (`settings.rs:520`) to be platform-aware. Today it's hardcoded to `"openai"`.

**Custom (local) polish — discoverability for the Ollama path**
- **Inline Ollama hint** under the base URL field when `selectedProvider.id === "custom"`: one-line caption with link, e.g. `"Tip: install Ollama (→ ollama.com) and run \`ollama serve\` to use local models."`
- **Test connection button** next to the base URL field for `Custom (local)` only. Pings `base_url/models`, reports success ("Connected — N models available") or failure ("Could not reach <base_url>"). Non-blocking. Reuse `fetch_post_process_models` infra if possible; otherwise add a new small backend command.
- Both polishes are scoped to `Custom (local)` only — do NOT add Test connection to external providers in Phase 8.

**Section UI — stacked two-block layout (PRIV-01)**
- **Drop the single flat Dropdown** for provider selection. Replace with a **stacked two-section radio-style picker**, both sections visible at once. NOT a sorted dropdown.
- Layout (verbatim from CONTEXT.md):
  ```
   ── On your device ──────────────────────────────
     (●) Apple Intelligence
         Free • on-device • macOS Apple Silicon
     ( ) Custom (local)
         Ollama / LM Studio on localhost
         [Tip: install Ollama (→ ollama.com)…]
         [Test connection]   ← only on Custom (local)

   ── External — data leaves this device ──────────
     ( ) OpenAI
     ( ) Z.AI
     ( ) OpenRouter
     ( ) Anthropic
     ( ) Groq
     ( ) Cerebras
  ```
- **Section labels are neutral**, no warning banner, no `Learn more →` link. Visual separation does the work.
- Per-row description text in i18n. Apple Intelligence: "Free • on-device • macOS Apple Silicon". Custom (local): "Ollama / LM Studio on localhost".
- **Implementation choice (Claude's Discretion):** either (a) extend `Dropdown.tsx` with grouped sections + radio rendering, or (b) build a new dedicated `ProviderPicker` component. Planner picks based on existing component patterns. **Recommendation in `## Architecture Patterns` below: option (b).**
- **Platform behavior:** on platforms without Apple Intelligence, "On your device" section shows only `Custom (local)`. No greyed-out row, no "coming soon" copy.
- All new strings require i18n keys across **20 locales**.

**PRIVACY.md (PRIV-02)**
- **Location:** `docs/PRIVACY.md` — standalone markdown file, sibling to `RUNBOOK-updater-signing.md` and `VERSIONING.md`. **No** in-app Privacy settings section. **No** in-app summary or link from the post-processing footer or onboarding.
- **Shape:** Short prose preamble + per-endpoint table with columns **Endpoint | When triggered | Data sent | How to disable | Default**.
- **Rows to include (exhaustive):**
  1. `https://github.com/getdictus/dictus-desktop/releases/latest/download/latest.json` — updater check via `tauri-plugin-updater`.
  2. `https://blob.handy.computer/*` — transcription model CDN (16 distinct URLs in `managers/model.rs`).
  3. `https://github.com/getdictus/dictus-desktop/releases/latest` — opened in browser when user clicks "View releases" (`UpdateChecker.tsx:206`).
  4. `https://api.openai.com/v1` — OpenAI post-processing.
  5. `https://api.z.ai/api/paas/v4` — Z.AI post-processing.
  6. `https://openrouter.ai/api/v1` — OpenRouter (note: `Referer: https://github.com/cjpais/Handy` upstream legacy at `llm_client.rs:70`, document honestly).
  7. `https://api.anthropic.com/v1` — Anthropic post-processing.
  8. `https://api.groq.com/openai/v1` — Groq post-processing.
  9. `https://api.cerebras.ai/v1` — Cerebras post-processing.
  10. `http://localhost:11434/v1` (default Custom base_url) — local-only, traffic doesn't leave device.
  11. `apple-intelligence://local` — Apple Intelligence on-device. No network.
- **Tone:** factual, scannable, audit-friendly. No marketing language.
- **Linking:** add link from About panel ("Privacy & network surface →" near existing Handy acknowledgment) and from `README.md`. Use Tauri opener (`openUrl()` from `@tauri-apps/plugin-opener`) for the About panel link — opens in browser to GitHub blob, not in-app webview.
- **Maintenance hook:** add a one-liner to `UPSTREAM.md` noting that any upstream commit adding a new HTTP endpoint requires a PRIVACY.md row.

**Onboarding (PRIV-03)**
- **No code change.** The onboarding flow (`src/components/onboarding/Onboarding.tsx`) currently shows only the transcription-model picker — cloud post-processing is not surfaced at all, so local transcription is already the implicit primary path.
- **PRIV-03 is satisfied by existing state.** Document this in the PLAN so downstream agents understand the requirement was reviewed and met without modification.
- If the planner finds a copy nit (e.g., subtitle benefits from one extra word), that's Claude's discretion — default position is **no change**.

### Claude's Discretion

- Exact i18n key paths and naming (e.g., `settings.postProcessing.providers.sectionLocal` vs `settings.postProcessing.local.title`) — planner picks consistent with existing keys.
- Whether to extend `Dropdown.tsx` or build a new `ProviderPicker` — based on existing component patterns.
- Whether the Test connection button reuses `fetch_post_process_models` directly or gets a new dedicated command — planner picks lowest-friction path.
- Exact phrasing of section headers and row descriptions in English (translations follow) — must be in i18n, must be factual.
- Exact column structure of the PRIVACY.md table within the agreed columns (Endpoint / When / Data / Disable / Default).
- Where the "Privacy & network surface →" link sits inside the About panel (above/below Handy acknowledgment, with/without icon).
- Where in `AdvancedSettings.tsx` the promoted `PostProcessingToggle` lands (own group vs folded into Output group).

### Deferred Ideas (OUT OF SCOPE)

- **Smart Modes** — Associate each post-process prompt with its own shortcut. Already scoped as milestone v1.3 "Smart Mode & Translation". Phase 8 sectioned UI should be friendly to this expansion but does NOT implement it.
- **Post-process waveform overlay animation** — Pierre wants a waveform animation during post-processing similar to the recording overlay. Track as a todo for v1.3.
- **Embedded local LLM runtime** — Whisper-style downloadable LLM picker inside Dictus (no separate Ollama process). Own milestone after v1.3.
- **Dictus-owned model CDN** — Replace `blob.handy.computer` (INFR-01). Phase 8 only documents current state.
- **Scrub OpenRouter `Referer: github.com/cjpais/Handy` header** — Tiny one-line fix in `llm_client.rs:70`. Could land as a small follow-up PR; NOT folded into Phase 8.
- **In-app updater opt-out toggle** — Documented in PRIVACY.md as "no in-app toggle today".
- **Test connection on external providers** — Phase 8 ships it only for Custom (local).
- **In-app Privacy settings page** — Considered and rejected for Phase 8 (i18n cost across 20 locales + duplication risk vs `docs/PRIVACY.md`).
- **Onboarding rework** — PRIV-03 satisfied without changes today. Future onboarding revision (e.g., when Smart Modes ships) may want to surface the privacy story explicitly. NOT in Phase 8.
- **Adding/removing providers** — No new Ollama-as-distinct-entry, no new Gemini, no dropping Z.AI/OpenRouter/Cerebras.
- **Full Dictus CDN migration (INFR-01)** — Phase 8 only *documents* `blob.handy.computer`.

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| PRIV-01 | Post-process provider list renders local providers above an "External — data leaves this device" section | Existing code reuse: `usePostProcessProviderState` hook exposes `providerOptions`, `selectedProviderId`, `handleProviderSelect`. The hook needs a grouped-options derivation (local vs external). `default_post_process_providers()` in `settings.rs:524-603` defines the 8 providers; relabel `"Custom" → "Custom (local)"` + platform-aware default in `default_post_process_provider_id()` (line 520). Stacked two-section radio picker built either as new `ProviderPicker` or by extending `Dropdown.tsx`. |
| PRIV-02 | `docs/PRIVACY.md` lists every outbound endpoint, what data leaves, and how to disable each | 16 `blob.handy.computer` URLs enumerated below in §"Network Surface Audit". `tauri-plugin-updater` endpoint from Phase 4. 6 external LLM endpoints from `default_post_process_providers()`. OpenRouter `Referer` legacy at `llm_client.rs:70`. Linked from About panel via `openUrl()` (`@tauri-apps/plugin-opener` 2.5.2 — verified). Linked from README. |
| PRIV-03 | Onboarding presents local transcription as primary; cloud opt-in | **Already satisfied.** `Onboarding.tsx` shows ONLY the transcription model picker — cloud post-processing is not surfaced at all. No code change required; document in PLAN. |

</phase_requirements>

## Summary

This phase is **UX + docs**, not a feature build. Three deliverables, all in surfaces that already exist:

1. **Settings — Post-Processing screen:** replace the flat `Dropdown` provider selector with a stacked two-section radio-style picker (local providers on top, external below). Add an inline Ollama tip + Test connection button to the `Custom (local)` row. Promote `PostProcessingToggle` out of the Experimental group. Change the platform default from `"openai"` to `"apple_intelligence"` on macOS ARM64 / `"custom"` elsewhere.
2. **`docs/PRIVACY.md`:** a new standalone markdown file enumerating ~11 distinct outbound endpoint families with what data leaves and how to disable.
3. **Onboarding (PRIV-03):** no code change. The flow already surfaces only the local transcription-model picker.

**Primary recommendation:** Build a new dedicated `ProviderPicker` component (Option B) instead of overloading `Dropdown.tsx`. The codebase has **no existing radio-group UI** to follow as precedent, so the new component is small (~80 lines) and the existing `Dropdown` stays single-purpose. Consume the same `providerOptions` shape from `usePostProcessProviderState` — extend the hook with a derived `groupedProviderOptions: { local: DropdownOption[]; external: DropdownOption[] }` instead of changing call sites. Reuse `fetch_post_process_models` directly for the Test connection button (no new backend command needed). Use `<Trans>` for the inline Ollama tip with embedded `<a>` rendered as a clickable label that calls `openUrl()`.

## Standard Stack

### Core (verified in-repo)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| React | 18.3.1 | UI framework | Project convention |
| TypeScript | 5.6.3 | Type safety | Strict mode, no `any` per CLAUDE.md |
| react-i18next | 16.4.1 | i18n | ESLint enforces `no-literal-string` in JSX |
| i18next | 25.7.2 | i18n core | Paired with react-i18next |
| Zustand | 5.0.8 | State | `settingsStore.ts` already manages post-process state |
| tauri-specta (auto) | — | Type bindings | Auto-generates `src/bindings.ts` from `#[specta::specta]` Rust commands |
| @tauri-apps/plugin-opener | 2.5.2 | External URL open | `openUrl()` pattern already used in `AboutSettings.tsx`, `UpdateChecker.tsx` |
| Tailwind CSS | 4.1.16 | Styling | Project convention |
| lucide-react | 0.542.0 | Icons | Already used (`Cog`, `Sparkles`, `RefreshCcw`, etc.) |

### Supporting (verified in-repo)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| react-select | 5.8.0 | Combobox/Select | Already used in `ModelSelect.tsx`; do NOT use for the new radio picker (wrong primitive) |
| sonner | 2.0.7 | Toasts | Existing pattern (`toast.error(...)` in `Onboarding.tsx:50`) — useful for Test connection feedback if Alert pattern is rejected |
| @playwright/test | 1.58.0 | E2E tests | Only one smoke test (`tests/app.spec.ts`); see Validation Architecture section below |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| New `ProviderPicker` component | Extending `Dropdown.tsx` with grouped sections | Pollutes single-purpose primitive; only the post-process screen needs grouping today. Smart Modes (v1.3) MAY want grouping again — but premature abstraction. **Recommendation: new component.** |
| `Alert` for Test connection result | Inline status text under the button | `Alert` is already used in `PostProcessingSettings.tsx:49` for Apple Intelligence unavailability — visually consistent. **Recommendation: Alert (variant="success"\|"error", contained).** |
| Reuse `fetch_post_process_models` | New dedicated `test_post_process_connection` command | Reuse: zero new code in `lib.rs`/`shortcut/mod.rs`, no new Tauri binding. The hook already exposes `fetchPostProcessModels(providerId)` from Zustand store. **Recommendation: reuse.** |
| `<Trans>` with embedded `<a>` for Ollama tip | Plain `t(...)` + separate button | `<Trans>` is already used (`PostProcessingSettings.tsx:317-321` for the prompt tip). Pattern established. **Recommendation: `<Trans>` with `components={{ link: <a onClick={() => openUrl(...)} /> }}`.** |

**No new dependencies required.**

## Architecture Patterns

### Recommended File Layout
```
src/
├── components/
│   ├── settings/
│   │   ├── PostProcessingSettingsApi/
│   │   │   ├── ProviderPicker.tsx          # NEW — stacked radio picker
│   │   │   ├── ProviderSelect.tsx          # KEEP for backwards-compat or delete (only one caller)
│   │   │   ├── TestConnectionButton.tsx    # NEW — small wrapper, Custom (local) only
│   │   │   ├── usePostProcessProviderState.ts  # EXTEND: add groupedProviderOptions
│   │   │   └── ...
│   │   ├── post-processing/
│   │   │   └── PostProcessingSettings.tsx  # MODIFY: replace <ProviderSelect>, add tip/test
│   │   ├── advanced/
│   │   │   └── AdvancedSettings.tsx        # MODIFY: move <PostProcessingToggle> out of experimental
│   │   └── about/
│   │       └── AboutSettings.tsx           # MODIFY: add "Privacy & network surface →" entry
│   └── ...
├── i18n/locales/{en,es,fr,...}/translation.json  # MODIFY: 20 locales, new keys
src-tauri/src/
├── settings.rs                              # MODIFY:
│                                            #   - default_post_process_provider_id() platform-aware
│                                            #   - default_post_process_providers() relabel "Custom" → "Custom (local)"
docs/
└── PRIVACY.md                               # NEW
README.md                                    # MODIFY: link to docs/PRIVACY.md
UPSTREAM.md                                  # MODIFY: one-liner maintenance hook
```

### Pattern 1: New ProviderPicker component (stacked radio)
**What:** Custom radio group with two visually separated sections; consumes grouped options derived from `usePostProcessProviderState`.
**When to use:** Only on the Post-Processing settings screen. NOT a general-purpose primitive — kept colocated with `usePostProcessProviderState`.
**Sketch (planner adjusts):**
```typescript
// Source: synthesized from existing Dropdown.tsx + SettingsGroup.tsx patterns

import { useTranslation } from "react-i18next";

export interface GroupedProviderOption {
  value: string;
  label: string;          // from provider.label
  description?: string;   // from i18n: settings.postProcessing.api.providers.descriptions.<id>
}

interface ProviderPickerProps {
  localOptions: GroupedProviderOption[];
  externalOptions: GroupedProviderOption[];
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
  renderRowExtras?: (option: GroupedProviderOption) => React.ReactNode;
}

export const ProviderPicker: React.FC<ProviderPickerProps> = ({
  localOptions, externalOptions, value, onChange, disabled, renderRowExtras,
}) => {
  const { t } = useTranslation();
  const renderSection = (title: string, options: GroupedProviderOption[]) => (
    <fieldset className="space-y-1">
      <legend className="text-xs font-medium text-mid-gray uppercase tracking-wide mb-2">
        {title}
      </legend>
      {options.map((option) => {
        const checked = value === option.value;
        return (
          <label
            key={option.value}
            className={`flex flex-col gap-1 p-3 rounded-md border cursor-pointer transition-colors ${
              checked
                ? "border-logo-primary bg-logo-primary/10"
                : "border-mid-gray/20 hover:bg-mid-gray/5"
            } ${disabled ? "opacity-50 cursor-not-allowed" : ""}`}
          >
            <div className="flex items-center gap-3">
              <input
                type="radio"
                name="post-process-provider"
                value={option.value}
                checked={checked}
                onChange={() => onChange(option.value)}
                disabled={disabled}
                className="accent-logo-primary"
              />
              <span className="text-sm font-medium">{option.label}</span>
            </div>
            {option.description && (
              <p className="text-xs text-mid-gray pl-7">{option.description}</p>
            )}
            {checked && renderRowExtras && (
              <div className="pl-7 mt-1">{renderRowExtras(option)}</div>
            )}
          </label>
        );
      })}
    </fieldset>
  );

  return (
    <div className="space-y-4">
      {localOptions.length > 0 &&
        renderSection(t("settings.postProcessing.api.providers.sectionLocal"), localOptions)}
      {externalOptions.length > 0 &&
        renderSection(t("settings.postProcessing.api.providers.sectionExternal"), externalOptions)}
    </div>
  );
};
```

### Pattern 2: Hook extension (grouped options + local-id detector)
**What:** Add a `LOCAL_PROVIDER_IDS` Set and a derived `groupedProviderOptions` memo to `usePostProcessProviderState`.
**When to use:** Always — single source of truth for which providers are "local".
**Sketch:**
```typescript
// Source: extension of existing usePostProcessProviderState.ts:68-73

const LOCAL_PROVIDER_IDS = new Set(["apple_intelligence", "custom"]);

const groupedProviderOptions = useMemo(() => {
  const local: GroupedProviderOption[] = [];
  const external: GroupedProviderOption[] = [];
  for (const provider of providers) {
    const opt: GroupedProviderOption = {
      value: provider.id,
      label: provider.label,
      // Description keys: settings.postProcessing.api.providers.descriptions.<id>
      description: t(
        `settings.postProcessing.api.providers.descriptions.${provider.id}`,
        { defaultValue: "" },
      ),
    };
    (LOCAL_PROVIDER_IDS.has(provider.id) ? local : external).push(opt);
  }
  // Order within local: apple_intelligence first (if present), then custom
  local.sort((a, b) => {
    if (a.value === "apple_intelligence") return -1;
    if (b.value === "apple_intelligence") return 1;
    return 0;
  });
  // External: preserve provider array order (already correct in settings.rs)
  return { local, external };
}, [providers, t]);
```

### Pattern 3: Platform-aware Rust default
**What:** `cfg(all(target_os = "macos", target_arch = "aarch64"))` gate for the default provider id. Same gate already used at `settings.rs:580` for Apple Intelligence availability — no new conditional pattern.
**Sketch:**
```rust
// Source: extension of settings.rs:520

fn default_post_process_provider_id() -> String {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        APPLE_INTELLIGENCE_PROVIDER_ID.to_string() // "apple_intelligence"
    }
    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    {
        "custom".to_string()
    }
}
```

### Pattern 4: Test connection — reuse fetchPostProcessModels
**What:** A small button colocated with `BaseUrlField` that calls `fetchPostProcessModels("custom")` and surfaces the result via `Alert`.
**Why reuse:** The Tauri command already exists, already handles the `"custom"` skip-when-no-api-key branch (`shortcut/mod.rs:1020`), and returns `Result<Vec<String>, String>`. The success case gives us the model count for the "Connected — N models available" message for free.
**Sketch:**
```typescript
// Source: synthesized from existing PostProcessingSettings.tsx patterns

const TestConnectionButton: React.FC = () => {
  const { t } = useTranslation();
  const { fetchPostProcessModels } = useSettings();
  const [status, setStatus] = useState<
    | { kind: "idle" }
    | { kind: "loading" }
    | { kind: "success"; count: number }
    | { kind: "error"; message: string }
  >({ kind: "idle" });

  const handleClick = async () => {
    setStatus({ kind: "loading" });
    try {
      const models = await fetchPostProcessModels("custom");
      setStatus({ kind: "success", count: models.length });
    } catch (error) {
      setStatus({ kind: "error", message: String(error) });
    }
  };

  return (
    <div className="space-y-2">
      <Button
        onClick={handleClick}
        disabled={status.kind === "loading"}
        variant="secondary"
        size="sm"
      >
        {t("settings.postProcessing.api.custom.testConnection.button")}
      </Button>
      {status.kind === "success" && (
        <Alert variant="success" contained>
          {t("settings.postProcessing.api.custom.testConnection.success", {
            count: status.count,
          })}
        </Alert>
      )}
      {status.kind === "error" && (
        <Alert variant="error" contained>
          {t("settings.postProcessing.api.custom.testConnection.error", {
            message: status.message,
          })}
        </Alert>
      )}
    </div>
  );
};
```
**Note:** `fetchPostProcessModels` throws on Err per `settingsStore.ts:528-540` — verify in implementation; if it returns `Result` without throwing, switch to `.status === "ok"` branching.

### Pattern 5: Inline Ollama tip with embedded link
**What:** `<Trans i18nKey="..." components={{ link: <a /> }} />` with the `<a>` calling `openUrl()`.
**Sketch:**
```typescript
// i18n key (en/translation.json):
//   "ollamaTip": "Tip: install <link>Ollama</link> and run `ollama serve` to use local models."

<Trans
  i18nKey="settings.postProcessing.api.custom.ollamaTip"
  components={{
    link: (
      <a
        className="text-logo-primary hover:underline cursor-pointer"
        onClick={(e) => {
          e.preventDefault();
          openUrl("https://ollama.com");
        }}
      />
    ),
    code: <code className="font-mono text-xs bg-mid-gray/10 px-1 rounded" />,
  }}
/>
```

### Pattern 6: About panel link entry
**What:** A `SettingContainer` row matching the existing Privacy/Ecosystem entries in `AboutSettings.tsx:77-103`, with `openUrl()` to the GitHub blob URL for `docs/PRIVACY.md`.
**Sketch:**
```typescript
<SettingContainer
  title={t("settings.about.privacyNetworkSurface.title")}
  description={t("settings.about.privacyNetworkSurface.description")}
  grouped={true}
>
  <Button
    variant="secondary"
    size="md"
    onClick={() =>
      openUrl("https://github.com/getdictus/dictus-desktop/blob/main/docs/PRIVACY.md")
    }
  >
    {t("settings.about.privacyNetworkSurface.button")}
  </Button>
</SettingContainer>
```
**Placement:** above the existing Handy acknowledgment in the "Acknowledgments" group, OR directly under the existing `settings.about.privacy` button (which links to the marketing privacy page `https://getdictus.com/en/privacy`). Planner picks — the **technical** privacy doc is distinct from the marketing privacy policy, so the entry deserves its own slot, not folded into Acknowledgments.

### Anti-Patterns to Avoid
- **Don't add a separate `is_local` field to the `PostProcessProvider` Rust struct.** The set is small (2 ids: `apple_intelligence`, `custom`), stable, and frontend-only. Adding a backend field forces a settings migration for no benefit. Use a frontend `LOCAL_PROVIDER_IDS` Set.
- **Don't change provider ids.** `"custom"` is the persisted key in `post_process_provider_id`, `post_process_api_keys`, `post_process_models` settings. Only the *label* changes.
- **Don't introduce react-select for the picker.** It's a combobox, not a radio group. Mismatched primitive.
- **Don't add warning banners or "Learn more →" links** in the External section. CONTEXT.md explicitly forbids paternalistic tone.
- **Don't gate Sidebar's "Post Process" tab on anything new.** `Sidebar.tsx:63` reads `settings?.post_process_enabled ?? false` — unchanged. Promoting the toggle changes only its location in `AdvancedSettings.tsx`, not the downstream chain.
- **Don't translate brand names.** Per `CONTRIBUTING_TRANSLATIONS.md:123`: do NOT translate "Ollama", "Apple Intelligence", "OpenAI", etc.
- **Don't add an in-app Privacy settings page.** CONTEXT.md explicitly rejected this; `docs/PRIVACY.md` only.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Opening external URLs | Custom `window.open`, raw `<a href>` | `openUrl()` from `@tauri-apps/plugin-opener` | Already in use everywhere (`AboutSettings.tsx`, `UpdateChecker.tsx:206`). Handles Tauri webview properly. |
| Provider list source-of-truth | Hardcoded TS array | Read from `settings.post_process_providers` (already loaded) | The 8 providers are defined in Rust (`settings.rs:524-603`) and surfaced via `useSettings`. Duplicating in TS = drift risk. |
| Test connection HTTP plumbing | New `reqwest` wrapper in Rust | Reuse `fetch_post_process_models` (already calls `llm_client::fetch_models`) | Same code path the dropdown's auto-fetch already uses. Free for re-use. |
| New backend command | `test_post_process_connection` Tauri command | Reuse `fetchPostProcessModels(providerId)` | Zero new bindings/specta regen. Success = `Ok(Vec<String>)` with count; error = `Err(String)`. |
| Apple Intelligence availability check | New code | `commands.checkAppleIntelligenceAvailable()` (already at `usePostProcessProviderState.ts:84`) | Existing pattern. |
| Settings migration for the new default | Custom serde migration | `serde_default` (already in place — see `#[serde(default = "default_post_process_provider_id")]`) | Existing users with non-default `post_process_provider_id` keep their choice; only new installs see the new default. |
| Markdown rendering in-app | mdx loader, markdown-it | DO NOT render PRIVACY.md in-app | CONTEXT.md mandates external link via `openUrl()` to GitHub blob. |

**Key insight:** This phase is 95% reorganization + new markdown. The only genuinely new code is the `ProviderPicker` component (~80 lines) and the `TestConnectionButton` (~40 lines). Everything else is moving existing pieces around, adding i18n keys, and writing prose.

## Common Pitfalls

### Pitfall 1: Forgetting i18n keys in some of the 20 locales
**What goes wrong:** Adding `settings.postProcessing.api.providers.sectionLocal` only to `en/translation.json` ships untranslated key paths in 19 other locales (i18next falls back to the key string by default — but it looks broken).
**Why it happens:** 20 locale files: `ar bg cs de en es fr he it ja ko pl pt ru sv tr uk vi zh zh-TW`. Easy to miss.
**How to avoid:**
- Add keys to **all 20 locales** with at minimum English fallback values (per CONTRIBUTING_TRANSLATIONS.md, fallback to English is acceptable; do NOT machine-translate technical UI strings).
- Run `bun run check:translations` (script at `scripts/check-translations.ts`) before commit — confirms key parity vs `en` source.
- Add keys to `en/translation.json` first, then copy-paste into siblings (preserving JSON structure).
**Warning signs:** UI showing raw key paths like `settings.postProcessing.api.providers.sectionLocal` instead of the section title.

### Pitfall 2: ESLint `i18next/no-literal-string` blocking commit
**What goes wrong:** Any literal string inside JSX (not in `t(...)`, `<Trans>`, or `aria-*`/`className`/`style` attributes) fails lint.
**Why it happens:** `eslint.config.js:18-35` enforces this strictly. The Custom-row "Tip:" text, Test connection button label, section headers — all must come from `t(...)` or `<Trans>`.
**How to avoid:**
- Run `bun run lint` before commit (the project requires this — see `package.json:11`).
- For inline links use `<Trans i18nKey="..." components={{ link: <a/> }} />` per the existing prompt-tip pattern (`PostProcessingSettings.tsx:317-321`).
- The `eslint-disable-next-line i18next/no-literal-string` escape hatch exists (`AboutSettings.tsx:47-48` uses it for the version `v{version}` span) but should be avoided for new code.

### Pitfall 3: Apple Intelligence default on non-eligible macOS systems
**What goes wrong:** A new install on macOS ARM64 running macOS < 26.0 (Tahoe) has Apple Intelligence as the default provider, but the OS check fails at runtime → user sees the "Apple Intelligence not available" Alert as their first experience.
**Why it happens:** `default_post_process_providers()` always pushes the Apple Intelligence provider on `cfg(all(target_os = "macos", target_arch = "aarch64"))` (line 580), and `default_post_process_provider_id()` will (post-change) return `"apple_intelligence"` on that same platform — but the actual availability is gated on macOS version + System Settings, checked at runtime via `commands.checkAppleIntelligenceAvailable()`.
**How to avoid:** CONTEXT.md accepts this: "Users on macOS Intel / Windows / Linux without Ollama running will see a connection error when they actually try to use it — that's acceptable because it surfaces the local path instead of the cloud path." The Apple Intelligence path is the same: if unavailable, the existing `<Alert variant="error">` at `PostProcessingSettings.tsx:48-52` surfaces the limitation. **No code change needed beyond the default switch** — the existing UX handles the unavailability path.

### Pitfall 4: Existing users with `post_process_provider_id: "openai"` after the default change
**What goes wrong:** A user who previously had post-processing enabled and selected OpenAI gets switched silently to Apple Intelligence after upgrade.
**Why it happens:** Misunderstanding of `serde_default`.
**How to avoid:** `serde_default` only fires when the field is MISSING from the persisted JSON. Existing users already have `post_process_provider_id: "openai"` (or whatever they picked) written to settings → that value persists. Only new installs (or users who never enabled post-processing → never wrote the field) see the new default. **Verify by inspecting the settings.json file pre/post upgrade.**

### Pitfall 5: Custom provider id label change breaking persisted settings
**What goes wrong:** Renaming `id: "custom"` to anything else orphans all existing per-provider keys (`post_process_api_keys["custom"]`, `post_process_models["custom"]`, `post_process_providers[].id == "custom"`).
**Why it happens:** Confusing label (rendered text) with id (persisted key).
**How to avoid:** **Only change the `label` field.** The `id` MUST stay `"custom"`. The CONTEXT.md decision states this explicitly. In the UI, render the label from i18n: instead of using `provider.label` directly, look up `t("settings.postProcessing.api.providers.labels.custom")` and fall back to `provider.label` for the others (or for simplicity, just change the hardcoded Rust label string from `"Custom"` to `"Custom (local)"` and rely on it being a brand-style name not subject to translation per `CONTRIBUTING_TRANSLATIONS.md:123`).

### Pitfall 6: blob.handy.computer URL drift between PRIVACY.md and code
**What goes wrong:** Upstream Handy adds a new model with a `blob.handy.computer/foo.tar.gz` URL; we sync the commit; PRIVACY.md doesn't get updated.
**Why it happens:** No automated check.
**How to avoid:** CONTEXT.md mandates adding a maintenance hook line to `UPSTREAM.md`. Suggested wording: *"Any upstream commit adding a new outbound HTTP endpoint (model URL, API base URL, telemetry, etc.) requires a corresponding row in `docs/PRIVACY.md`. Verify by `grep -E 'https?://' src-tauri/src/managers/model.rs src-tauri/src/settings.rs src-tauri/src/llm_client.rs` against PRIVACY.md table."*
**Optional automation (NOT in Phase 8):** extend `.github/scripts/verify-sync.sh` to grep `blob.handy.computer` URLs from `managers/model.rs` and check each appears in `docs/PRIVACY.md`. Defer to follow-up todo.

### Pitfall 7: Sidebar "Post Process" tab disappearing when toggle is moved
**What goes wrong:** Moving `<PostProcessingToggle>` out of the experimental block accidentally breaks the sidebar tab gating.
**Why it happens:** Misreading the dependency.
**How to avoid:** **The dependency is on `settings.post_process_enabled`, not on the toggle's parent group.** `Sidebar.tsx:63` reads the setting directly. The toggle merely *renders* the setting — its location in the tree doesn't affect the setting's value. Verified: moving the toggle is purely a UI relocation; no side effects on the Sidebar's tab visibility or on `shortcut/handy_keys.rs:437` (which also reads the same setting).

### Pitfall 8: `<Trans>` component swallowing escape characters in i18n strings
**What goes wrong:** The Ollama tip uses backticks for code: ``"Tip: install <link>Ollama</link> and run `ollama serve` to use local models."`` — but i18next may not render the backticks as `<code>` automatically.
**Why it happens:** `<Trans>` only replaces named components from the `components` prop; literal backticks render as-is unless you wrap them yourself.
**How to avoid:** Either (a) include a `<code>` wrapper in the i18n key: `"Tip: install <link>Ollama</link> and run <code>ollama serve</code> to use local models."` and pass `components={{ link: <a/>, code: <code/> }}`, mirroring `PostProcessingSettings.tsx:317-321`; or (b) keep backticks literal in the i18n string and style them via CSS.

## Code Examples

### Reading existing post-processing settings (already in repo)
```typescript
// Source: src/components/settings/PostProcessingSettingsApi/usePostProcessProviderState.ts:34-50
const {
  settings,
  isUpdating,
  setPostProcessProvider,
  updatePostProcessBaseUrl,
  fetchPostProcessModels,
} = useSettings();

const providers = settings?.post_process_providers || [];
const selectedProviderId = useMemo(() => {
  return settings?.post_process_provider_id || providers[0]?.id || "openai";
}, [providers, settings?.post_process_provider_id]);
```

### Opening external URL (existing pattern)
```typescript
// Source: src/components/settings/about/AboutSettings.tsx:69-71
import { openUrl } from "@tauri-apps/plugin-opener";

<Button onClick={() => openUrl("https://github.com/getdictus/dictus-desktop")}>
  {t("settings.about.sourceCode.button")}
</Button>
```

### Embedded link via Trans (existing pattern)
```typescript
// Source: src/components/settings/post-processing/PostProcessingSettings.tsx:317-322
<p className="text-xs text-mid-gray/70">
  <Trans
    i18nKey="settings.postProcessing.prompts.promptTip"
    components={{ code: <code /> }}
  />
</p>
```

### Apple Intelligence availability check (existing pattern)
```typescript
// Source: src/components/settings/PostProcessingSettingsApi/usePostProcessProviderState.ts:82-90
if (providerId === APPLE_PROVIDER_ID) {
  const available = await commands.checkAppleIntelligenceAvailable();
  if (!available) {
    setAppleIntelligenceUnavailable(true);
  }
}
```

### Alert for status feedback (existing pattern)
```typescript
// Source: src/components/settings/post-processing/PostProcessingSettings.tsx:48-52
{state.appleIntelligenceUnavailable ? (
  <Alert variant="error" contained>
    {t("settings.postProcessing.api.appleIntelligence.unavailable")}
  </Alert>
) : null}
```

## Network Surface Audit (for PRIVACY.md authoring)

### Outbound endpoint families found in the codebase (exhaustive)

| # | Endpoint / Family | Source path | Trigger | Data sent | Disable mechanism | Default state |
|---|---|---|---|---|---|---|
| 1 | `https://github.com/getdictus/dictus-desktop/releases/latest/download/latest.json` | `tauri-plugin-updater` configured via `tauri.conf.json` (Phase 4 work) | App start (when enabled) + manual "Check for updates" | HTTP GET; standard headers (User-Agent, Accept) — no transcribed text, no PII | `settings.update_checks_enabled` — see `UpdateChecker.tsx:28-29`. Toggle in Debug settings. | **Enabled** (default per Phase 4) |
| 2 | `https://blob.handy.computer/*` — **16 distinct URLs** in `managers/model.rs` | `src-tauri/src/managers/model.rs` lines 133, 161, 188, 215, 243, 271, 308, 335, 363, 392, 421, 457, 487, 521, 558, 593 | User clicks "Download" on a model card (in Models settings or Onboarding) | HTTP GET only; standard headers | Don't download models | **Off by default** — no model downloads happen without user action |
| 3 | `https://github.com/getdictus/dictus-desktop/releases/latest` | `src/components/update-checker/UpdateChecker.tsx:206` | User clicks "View releases" in portable-update dialog | Browser open; no app HTTP | N/A (manual user action) | N/A |
| 4 | `https://api.openai.com/v1/{chat/completions,models}` | `src-tauri/src/settings.rs:529` + `src-tauri/src/llm_client.rs` | (a) Auto-fetch models when API key + provider selected; (b) User triggers post-process shortcut | Auth header `Bearer <api_key>` + transcribed text + selected prompt | Toggle off `post_process_enabled` OR switch provider OR delete API key | **Off by default** (`post_process_enabled = false`) |
| 5 | `https://api.z.ai/api/paas/v4/{chat/completions,models}` | `settings.rs:537` | Same as #4 | Same as #4 | Same as #4 | Same as #4 |
| 6 | `https://openrouter.ai/api/v1/{chat/completions,models}` | `settings.rs:545` | Same as #4 | Same as #4 + **`Referer: https://github.com/cjpais/Handy` header** (upstream legacy, `llm_client.rs:70`) — **document honestly; flagged for follow-up scrubbing** | Same as #4 | Same as #4 |
| 7 | `https://api.anthropic.com/v1/{messages,models}` | `settings.rs:553` | Same as #4 | `x-api-key: <api_key>` + `anthropic-version: 2023-06-01` + transcribed text + selected prompt | Same as #4 | Same as #4 |
| 8 | `https://api.groq.com/openai/v1/{chat/completions,models}` | `settings.rs:561` | Same as #4 | Same as #4 | Same as #4 | Same as #4 |
| 9 | `https://api.cerebras.ai/v1/{chat/completions,models}` | `settings.rs:569` | Same as #4 | Same as #4 | Same as #4 | Same as #4 |
| 10 | `http://localhost:11434/v1/{chat/completions,models}` (default Custom base_url) | `settings.rs:596` | User selects Custom provider + (a) auto-fetch models; (b) post-process shortcut; (c) Test connection button (Phase 8 new) | Local-only; traffic does not leave the device | N/A — local-only | **Off by default** (`post_process_enabled = false` and user must manually pick Custom) |
| 11 | `apple-intelligence://local` | `settings.rs:585` | User selects Apple Intelligence + post-process shortcut | On-device; no network | N/A — on-device | **Off by default** |

### Exact `blob.handy.computer` URLs (for PRIVACY.md table or footnote)
```
https://blob.handy.computer/ggml-small.bin                        (Whisper Small)
https://blob.handy.computer/whisper-medium-q4_1.bin               (Whisper Medium)
https://blob.handy.computer/ggml-large-v3-turbo.bin               (Whisper Turbo)
https://blob.handy.computer/ggml-large-v3-q5_0.bin                (Whisper Large)
https://blob.handy.computer/breeze-asr-q5_k.bin                   (Breeze ASR)
https://blob.handy.computer/parakeet-v2-int8.tar.gz               (Parakeet V2)
https://blob.handy.computer/parakeet-v3-int8.tar.gz               (Parakeet V3)
https://blob.handy.computer/moonshine-base.tar.gz                 (Moonshine Base)
https://blob.handy.computer/moonshine-tiny-streaming-en.tar.gz    (Moonshine V2 Tiny)
https://blob.handy.computer/moonshine-small-streaming-en.tar.gz   (Moonshine V2 Small)
https://blob.handy.computer/moonshine-medium-streaming-en.tar.gz  (Moonshine V2 Medium)
https://blob.handy.computer/sense-voice-int8.tar.gz               (SenseVoice)
https://blob.handy.computer/giga-am-v3-int8.tar.gz                (GigaAM v3)
https://blob.handy.computer/canary-180m-flash.tar.gz              (Canary 180M Flash)
https://blob.handy.computer/canary-1b-v2.tar.gz                   (Canary 1B v2)
https://blob.handy.computer/cohere-int8.tar.gz                    (Cohere)
```
Recommend rendering as a `blob.handy.computer/*` wildcard row in the main table, with the full list in a collapsed `<details>` block or a short appendix at the bottom of PRIVACY.md.

### Headers attached to all LLM provider calls (per `llm_client.rs:63-95`)
- `Content-Type: application/json`
- `Referer: https://github.com/cjpais/Handy` ← **legacy, flag in PRIVACY.md**
- `User-Agent: Dictus/1.0 (+https://github.com/getdictus/dictus-desktop)`
- `X-Title: Dictus`
- `Authorization: Bearer <api_key>` (for non-Anthropic) OR `x-api-key: <api_key>` + `anthropic-version: 2023-06-01` (for Anthropic)

### Endpoints explicitly NOT contacted by Dictus
- No telemetry/analytics endpoint
- No license validation endpoint
- No crash reporter / Sentry / Bugsnag
- No `getdictus.com` API calls (only browser-opens for marketing pages from About panel)

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Flat dropdown listing all providers (`<ProviderSelect>` → `<Dropdown>`) | Stacked two-section radio picker, visually separated | Phase 8 (this phase) | Local providers become primary; cloud is opt-in via section labeling |
| `Custom` label (ambiguous: cloud-or-local?) | `Custom (local)` label | Phase 8 | Disambiguates intent without changing the provider id |
| Post-processing toggle buried in Experimental group (3 clicks to find) | Top-level Advanced group | Phase 8 | Discoverability of the local Apple Intelligence + Ollama paths |
| Default provider `openai` everywhere | `apple_intelligence` on macOS ARM64, `custom` elsewhere | Phase 8 | Honors local-first; cloud requires explicit opt-in |
| No public network-surface documentation | `docs/PRIVACY.md` enumerating ~11 endpoint families | Phase 8 | Audit-friendly; serves as the "complete outbound surface" reference |
| In-app Privacy section (proposed early in discussion) | Standalone `docs/PRIVACY.md` only | Phase 8 (decided in CONTEXT.md) | Reduces i18n cost (20 locales × structured page) and duplication risk |

**Deprecated/outdated (NOT in scope of Phase 8 but flagged):**
- `Referer: https://github.com/cjpais/Handy` header in `llm_client.rs:70` — upstream Handy legacy. **Document honestly; defer scrubbing to follow-up PR.**
- `blob.handy.computer` CDN — upstream Handy infrastructure. INFR-01 (own CDN migration) is deferred to a future milestone. **Document as-is.**

## Open Questions

1. **Should the Apple Intelligence "Free • on-device • macOS Apple Silicon" line be shown on platforms where Apple Intelligence is hidden (Windows/Linux/macOS Intel)?**
   - What we know: CONTEXT.md says "on platforms without Apple Intelligence, the 'On your device' section shows only `Custom (local)` — no greyed-out Apple Intelligence row." So the row doesn't render at all on those platforms.
   - What's unclear: whether the section header "On your device" stays even when only Custom (local) is in it (one item under a header looks lonely). The CONTEXT.md mockup implies yes.
   - Recommendation: keep the section header always. The visual structure carries the local-first promise even with one item. Don't conditionally hide the header.

2. **Where exactly does the "Privacy & network surface →" link sit in the About panel?**
   - What we know: CONTEXT.md lists this as Claude's Discretion ("Where the 'Privacy & network surface →' link sits inside the About panel").
   - What's unclear: should it be a new entry in the first SettingsGroup (next to the existing `settings.about.privacy` button that links to the marketing privacy page) or in the Acknowledgments group?
   - Recommendation: new entry in the **first SettingsGroup** (the main About group), directly under the existing "Privacy" button. The two are conceptually paired: marketing privacy policy + technical network-surface doc. Use different button labels to disambiguate.

3. **Should the platform-aware default change in `default_post_process_provider_id()` be conditional on Apple Intelligence runtime availability, or just on `cfg(target_os = "macos", target_arch = "aarch64")`?**
   - What we know: `default_post_process_providers()` (line 580) uses just the `cfg` gate for inclusion, without checking macOS version at startup (per the comment at line 576: "This prevents crashes on macOS 26.x beta where accessing SystemLanguageModel.default during early app initialization causes SIGABRT").
   - What's unclear: whether the default selection should follow the same pattern (always Apple Intelligence on macOS ARM64) or actually probe `checkAppleIntelligenceAvailable` at first-run.
   - Recommendation: **Same `cfg` gate, no runtime probe.** The settings default fires at deserialization time (potentially before app initialization). A runtime probe would create the same startup-time race the comment warns about. If Apple Intelligence is unavailable, the existing `<Alert variant="error">` UX surfaces it when the user actually tries to use it.

4. **Test connection error message format — verbatim vs sanitized?**
   - What we know: `fetch_post_process_models` returns `Err(String)` with messages like `"Failed to fetch models: connection refused (os error 61)"` or `"Model list request failed (401 Unauthorized): {...}"`. Useful for debugging but ugly for users.
   - What's unclear: whether to show the raw error or a friendly "Could not reach <base_url>" message with the raw error in a tooltip.
   - Recommendation: friendly main message + raw error in a collapsible `<details>` or tooltip. The CONTEXT.md mockup shows "Could not reach <base_url>" — match that.

5. **PRIVACY.md table format — wide HTML table or markdown table?**
   - What we know: 5 columns × 11 rows. Wide columns ("How to disable" can be a paragraph).
   - What's unclear: whether GitHub-rendered markdown handles the width gracefully.
   - Recommendation: markdown table for the headers/triggers/disables; use a separate `<details>` block under the `blob.handy.computer/*` row for the full 16-URL enumeration. Keep PRIVACY.md to ~150-200 lines max.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Playwright 1.58.0 (E2E only) + manual UAT |
| Config file | `playwright.config` (implied by `bun run test:playwright`) — verify exists or add in Wave 0 |
| Quick run command | `bun run lint && bun run format:check` (runs in seconds) |
| Full suite command | `bun run lint && bun run format:check && bun run check:translations && bun run build` |
| Cargo (Rust) | `cd src-tauri && cargo fmt -- --check && cargo clippy --all-targets` |

**No Rust unit test framework configured** beyond `cargo test` defaults. No Vitest/Jest for frontend. The codebase relies on:
- ESLint (i18next/no-literal-string enforcement)
- TypeScript strict mode (compile-time validation)
- Prettier (formatting)
- `check-translations.ts` (key parity across 20 locales)
- Manual UAT via `bun run tauri dev`

This is acceptable for a UX phase: most acceptance criteria are visual and require the app running. **Phase 8 is heavy on visual UAT and light on automated tests.**

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| PRIV-01 | Stacked picker renders with local section above external | Visual UAT | Manual: `bun run tauri dev` → Settings → Post Process → verify layout | N/A (manual) |
| PRIV-01 | `default_post_process_provider_id()` returns `"apple_intelligence"` on macOS ARM64, `"custom"` elsewhere | Rust unit test | `cd src-tauri && cargo test default_post_process_provider_id` | ❌ Wave 0 — add `#[test]` in `settings.rs` with `#[cfg(test)]` modules |
| PRIV-01 | Custom (local) row shows Ollama tip + Test connection button when selected | Visual UAT | Manual | N/A (manual) |
| PRIV-01 | Test connection button success/error states render via Alert | Visual UAT | Manual: with Ollama running → success; without → error | N/A (manual) |
| PRIV-01 | Selecting OpenAI persists provider id "openai" in settings.json | Integration (manual) | Inspect `~/Library/Application Support/com.dictus.desktop/settings.json` after selection | N/A (manual) |
| PRIV-01 | Provider id `"custom"` unchanged after label change "Custom" → "Custom (local)" | Rust unit test | `cd src-tauri && cargo test default_post_process_providers_custom_id_stable` | ❌ Wave 0 — add `#[test]` |
| PRIV-01 | Sidebar "Post Process" tab still appears when `post_process_enabled=true` after toggle relocation | Visual UAT | Manual: enable toggle in new location → verify sidebar tab appears | N/A (manual) |
| PRIV-01 | `bun run lint` passes (no literal strings in new JSX) | Automated | `bun run lint` | ✅ ESLint config exists |
| PRIV-01 | `bun run check:translations` passes (all 20 locales have new keys) | Automated | `bun run check:translations` | ✅ Script exists at `scripts/check-translations.ts` |
| PRIV-02 | `docs/PRIVACY.md` exists and contains all 11 endpoint families | Static check | `grep -E '(api\\.openai\\.com\|api\\.z\\.ai\|openrouter\\.ai\|api\\.anthropic\\.com\|api\\.groq\\.com\|api\\.cerebras\\.ai\|blob\\.handy\\.computer\|localhost:11434\|apple-intelligence://local\|getdictus/dictus-desktop/releases)' docs/PRIVACY.md \| wc -l` | ❌ Wave 0 — file doesn't exist yet |
| PRIV-02 | README.md links to docs/PRIVACY.md | Static check | `grep -c 'docs/PRIVACY.md' README.md` (≥ 1) | N/A — README exists, needs edit |
| PRIV-02 | About panel "Privacy & network surface" button opens GitHub blob URL | Visual UAT | Manual: click button → browser opens to `github.com/getdictus/dictus-desktop/blob/main/docs/PRIVACY.md` | N/A (manual) |
| PRIV-02 | UPSTREAM.md mentions PRIVACY.md row requirement | Static check | `grep -c 'PRIVACY.md' UPSTREAM.md` (≥ 1) | N/A — UPSTREAM.md exists, needs edit |
| PRIV-03 | Onboarding shows only transcription model picker (no cloud post-process surface) | Code review / Visual UAT | Inspection of `Onboarding.tsx` — verify no `post_process_*` references | ✅ Already true — no change needed |
| PRIV-03 | Plan documents PRIV-03 as "satisfied by existing state, no code change" | Doc check | Read PLAN.md after planning phase | N/A (planning artifact) |

### Sampling Rate
- **Per task commit:** `bun run lint && bun run format:check` (~5 seconds)
- **Per wave merge:** `bun run lint && bun run format:check && bun run check:translations && bun run build && cd src-tauri && cargo fmt -- --check && cargo clippy --all-targets -- -D warnings`
- **Phase gate:** Above full suite green + manual UAT walkthrough (record screenshot of the new picker on macOS for the verify-work artifact) + `cargo test` for the Rust default test before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src-tauri/src/settings.rs` — add a `#[cfg(test)] mod tests` block with two tests:
  - `default_post_process_provider_id_returns_apple_on_macos_arm64()` — uses `cfg!(...)` to assert the platform-correct id
  - `default_post_process_providers_includes_custom_with_stable_id()` — asserts the relabeled provider still has `id == "custom"`
- [ ] `docs/PRIVACY.md` — the file itself doesn't exist yet
- [ ] `README.md` — add a "Privacy & network surface" section pointing to `docs/PRIVACY.md`
- [ ] `UPSTREAM.md` — add one-liner about new HTTP endpoints requiring PRIVACY.md updates
- [ ] (Optional, deferred) `.github/scripts/verify-sync.sh` — extend to assert PRIVACY.md is updated when new outbound endpoints appear in code. Recommend filing as a follow-up todo, NOT in Phase 8.

*(No new test framework install needed; the project's existing lint + translation-parity + manual UAT model fits Phase 8.)*

## Sources

### Primary (HIGH confidence — verified by reading files in this repo)
- `.planning/phases/08-privacy-local-first-ux/08-CONTEXT.md` — authoritative for all locked decisions
- `.planning/REQUIREMENTS.md` §"Privacy / Local-First UX (PRIV)" — PRIV-01/02/03 (with CONTEXT.md override of literal text)
- `.planning/ROADMAP.md` §"Phase 8" — goal + 3 success criteria
- `src-tauri/src/settings.rs` lines 506, 520-603 — provider defaults, including platform `cfg` gate for Apple Intelligence
- `src-tauri/src/llm_client.rs` lines 63-95, 221-277 — header construction (incl. legacy Referer at line 70), `fetch_models` implementation
- `src-tauri/src/managers/model.rs` lines 100-600 — full enumeration of 16 `blob.handy.computer` URLs (verified by grep)
- `src-tauri/src/shortcut/mod.rs` lines 987-1028 — `fetch_post_process_models` Tauri command (reusable for Test connection)
- `src/components/settings/post-processing/PostProcessingSettings.tsx` — current screen structure
- `src/components/settings/PostProcessingSettingsApi/usePostProcessProviderState.ts` — current hook shape
- `src/components/settings/PostProcessingSettingsApi/ProviderSelect.tsx` — current dropdown wrapper (deletable / wrappable)
- `src/components/settings/advanced/AdvancedSettings.tsx` lines 60-70 — current experimental gating
- `src/components/settings/about/AboutSettings.tsx` — About panel structure + `openUrl` import + Button patterns
- `src/components/onboarding/Onboarding.tsx` — confirmed PRIV-03 satisfied: no `post_process_*` references
- `src/components/Sidebar.tsx` lines 59-64 — confirmed sidebar gating is on `post_process_enabled` setting, not on the toggle's parent group
- `src/components/update-checker/UpdateChecker.tsx` line 206 — `openUrl()` from `@tauri-apps/plugin-opener` pattern verified
- `src/components/ui/Dropdown.tsx` — current Dropdown shape (no native section support)
- `src/components/ui/SettingsGroup.tsx`, `SettingContainer.tsx` — settings primitives
- `src/i18n/locales/en/translation.json` — existing keys `settings.postProcessing.*` (line 363) + `settings.debug.postProcessingToggle.*` (line 483)
- `package.json` — verified `@tauri-apps/plugin-opener: ^2.5.2`, no Vitest/Jest, Playwright 1.58.0 only
- `eslint.config.js` — verified `i18next/no-literal-string` error rule
- `CONTRIBUTING_TRANSLATIONS.md` — verified "Don't translate brand names" + fallback-to-English convention
- `docs/RUNBOOK-updater-signing.md`, `docs/VERSIONING.md` — sibling location for PRIVACY.md confirmed

### Secondary (MEDIUM confidence)
- N/A — all critical claims verified directly from in-repo files.

### Tertiary (LOW confidence)
- N/A — no WebSearch/Context7 lookups needed for this phase. The work is entirely in-repo (UX rearrangement + markdown authoring). No new library APIs to verify.

## Metadata

**Confidence breakdown:**
- Standard Stack: HIGH — all libraries and versions verified directly from `package.json` and existing code
- Architecture: HIGH — patterns synthesized from existing in-repo code; no novel architecture
- Pitfalls: HIGH — pitfalls grounded in specific file references (settings persistence model, ESLint rule, Sidebar dependency chain)
- Network surface audit: HIGH — full enumeration via grep across `model.rs`, `settings.rs`, `llm_client.rs`, `UpdateChecker.tsx`
- Validation: HIGH — current test infrastructure (lint + translations + manual UAT + cargo) explicitly inventoried

**Research date:** 2026-05-21
**Valid until:** 2026-06-20 (30 days — stable codebase; only invalidated if upstream Handy adds new endpoints before Phase 8 ships)
