import React, { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { ask } from "@tauri-apps/plugin-dialog";
import { Check } from "lucide-react";
import type { ModelInfo } from "@/bindings";
import type { LlmModelInfo } from "@/bindings";
import type { ModelCardStatus } from "@/components/onboarding/ModelCard";
import ModelCard from "@/components/onboarding/ModelCard";
import Badge from "@/components/ui/Badge";
import { SettingsGroup } from "@/components/ui";
import { useLlmModelStore } from "@/stores/llmModelStore";
import { CustomGgufDropZone } from "./CustomGgufDropZone";

/**
 * A non-model on-device engine (Apple Intelligence, Ollama/Custom) rendered as a
 * peer card in the unified engine list. `position` sets its slot before
 * (`lead`) or after (`trail`) the GGUF model cards. The active engine is
 * signalled by its "Actif" badge, not by reordering.
 */
export interface ProviderEntry {
  id: string;
  label: string;
  description?: string;
  checked: boolean;
  disabled?: boolean;
  position: "lead" | "trail";
  onSelect: (id: string) => void;
  extras?: React.ReactNode;
}

/**
 * Maps a backend catalogue model id to its i18n description key suffix.
 * Custom/unmapped models fall back to the backend-provided description.
 */
const MODEL_ID_TO_I18N_KEY: Record<string, string> = {
  "qwen2.5-1.5b": "qwen25_1b5",
  "gemma-3-4b": "gemma3_4b",
  "phi-4-mini": "phi4_mini",
  "llama-3.2-3b": "llama32_3b",
  "translate-gemma-4b": "translateGemma4b",
};

/**
 * Adapt LlmModelInfo to ModelInfo so we can reuse ModelCard verbatim.
 * Whisper-only fields are filled with safe zero/empty defaults.
 * `description` is passed in (localized) so card text follows the UI language.
 */
function toModelCardModel(m: LlmModelInfo, description: string): ModelInfo {
  return {
    id: m.id,
    name: m.name,
    description,
    filename: m.filename,
    url: m.url ?? "",
    sha256: m.sha256 ?? null,
    size_mb: m.size_mb,
    is_downloaded: m.is_downloaded,
    is_downloading: m.is_downloading,
    partial_size: m.partial_size,
    is_custom: m.is_custom,
    is_recommended: m.is_recommended,
    // Whisper-only fields — safe defaults
    is_directory: false,
    engine_type: "Whisper",
    accuracy_score: 0,
    speed_score: 0,
    supports_translation: false,
    supported_languages: [],
    supports_language_selection: false,
  };
}

interface LlmLibrarySectionProps {
  /**
   * Engine mode: render the full unified on-device engine list (provider cards
   * + GGUF model cards) bare, no SettingsGroup wrapper. Selecting a downloaded
   * model picks it as the post-processing engine (via `onSelectAsEngine`).
   */
  engineMode?: boolean;
  /** Whether the embedded provider is the currently selected engine. */
  embeddedSelected?: boolean;
  /** Select a downloaded model as the embedded engine (sets active + provider). */
  onSelectAsEngine?: (modelId: string) => void;
  /** Non-model engines (Apple Intelligence, Ollama) to interleave in the list. */
  providerEntries?: ProviderEntry[];
  /** Rendered at the very bottom (e.g. the custom-GGUF import zone). */
  footer?: React.ReactNode;
}

export const LlmLibrarySection: React.FC<LlmLibrarySectionProps> = ({
  engineMode = false,
  embeddedSelected = false,
  onSelectAsEngine,
  providerEntries = [],
  footer,
}) => {
  const { t } = useTranslation();
  const store = useLlmModelStore();

  useEffect(() => {
    void store.refresh();
    let unlistenFn: (() => void) | null = null;
    void store.initListeners().then((fn) => {
      unlistenFn = fn;
    });
    return () => {
      unlistenFn?.();
    };
  }, []); // Mount once — store actions are stable Zustand references

  // Catalogue models get their description from i18n (so it follows the UI
  // language); custom models keep their backend-provided description.
  const localizedDescription = (m: LlmModelInfo): string => {
    const key = MODEL_ID_TO_I18N_KEY[m.id];
    if (!key) return m.description;
    return t(
      `settings.postProcessing.modelsAndLocalProcessing.library.models.${key}.description`,
      { defaultValue: m.description },
    );
  };

  const getModelStatus = (modelId: string): ModelCardStatus => {
    if (modelId in store.downloadProgress) {
      return "downloading";
    }
    if (modelId in store.verifyingModels) {
      return "verifying";
    }
    const model = store.models.find((m) => m.id === modelId);
    const isActive = modelId === store.activeModelId;
    // In engine mode a model only reads as the selected engine when the
    // embedded provider is actually the chosen engine — otherwise it's just a
    // downloaded model available to pick.
    if (isActive && (!engineMode || embeddedSelected)) {
      return "active";
    }
    if (model?.is_downloaded) {
      return "available";
    }
    return "downloadable";
  };

  const handleDelete = async (modelId: string) => {
    const model = store.models.find((m) => m.id === modelId);
    const modelName = model?.name ?? modelId;
    const isActive = modelId === store.activeModelId;

    const confirmed = await ask(
      isActive
        ? t("settings.models.deleteActiveConfirm", { modelName })
        : t("settings.models.deleteConfirm", { modelName }),
      {
        title: t("settings.models.deleteTitle"),
        kind: "warning",
      },
    );

    if (confirmed) {
      await store.deleteModel(modelId);
    }
  };

  const handleSelect = (modelId: string) => {
    const status = getModelStatus(modelId);
    if (status === "downloadable") {
      void store.downloadModel(modelId);
      return;
    }
    if (status === "available" || status === "active") {
      if (engineMode && onSelectAsEngine) {
        // Pick this model AS the post-processing engine (active + embedded).
        onSelectAsEngine(modelId);
      } else {
        void store.setActiveModel(modelId);
      }
    }
  };

  const renderCard = (model: LlmModelInfo, withActions: boolean) => (
    <ModelCard
      key={model.id}
      model={toModelCardModel(model, localizedDescription(model))}
      status={getModelStatus(model.id)}
      onSelect={handleSelect}
      onDownload={(id) => void store.downloadModel(id)}
      onCancel={withActions ? (id) => void store.cancelDownload(id) : undefined}
      onDelete={withActions ? handleDelete : undefined}
      downloadProgress={store.downloadProgress[model.id]?.percentage}
      downloadSpeed={store.downloadStats[model.id]?.speedMbps}
      showRecommended={true}
    />
  );

  // Engine-list order by zone: downloaded → downloadable → perso (imported
  // custom GGUF). Active state does NOT affect position — selecting a model must
  // not move it; a model only shifts up into the downloaded zone when its
  // download starts. sort() is stable, so catalogue order holds within a zone.
  const engineSorted = [...store.models].sort((a, b) => {
    const rank = (m: LlmModelInfo) => {
      if (m.is_custom) return 2; // perso / imported GGUF — after the catalogue
      if (m.is_downloaded || m.is_downloading || m.id in store.downloadProgress)
        return 0; // downloaded (or download in flight)
      return 1; // downloadable
    };
    return rank(a) - rank(b);
  });

  // Two-section split (legacy / standalone mode).
  const downloadedModels = store.models
    .filter(
      (m) =>
        m.is_downloaded ||
        m.is_downloading ||
        m.id in store.downloadProgress ||
        m.is_custom,
    )
    .sort((a, b) => {
      if (a.id === store.activeModelId) return -1;
      if (b.id === store.activeModelId) return 1;
      return 0;
    });

  const availableModels = store.models.filter(
    (m) =>
      !m.is_downloaded &&
      !m.is_downloading &&
      !(m.id in store.downloadProgress) &&
      !m.is_custom,
  );

  // Provider card (Apple Intelligence, Ollama) — same shape as ModelCard, with
  // an "Actif" badge when selected, so every row in the list is uniform.
  const renderProviderRow = (p: ProviderEntry) => {
    const selectable = !p.disabled;
    return (
      <div
        key={`provider:${p.id}`}
        role="button"
        aria-pressed={p.checked}
        tabIndex={selectable ? 0 : undefined}
        onClick={() => selectable && p.onSelect(p.id)}
        onKeyDown={(e) => {
          if (selectable && (e.key === "Enter" || e.key === " ")) {
            e.preventDefault();
            p.onSelect(p.id);
          }
        }}
        className={`flex flex-col gap-1 px-4 py-3 rounded-xl border-2 transition-all ${
          p.checked
            ? "border-logo-primary/50 bg-logo-primary/10"
            : "border-mid-gray/20 hover:border-logo-primary/50 hover:bg-logo-primary/5"
        } ${p.disabled ? "opacity-50 cursor-not-allowed" : "cursor-pointer"}`}
      >
        <div className="flex items-center gap-3 flex-wrap">
          <span className="text-base font-semibold text-text">{p.label}</span>
          {p.checked && (
            <Badge variant="primary">
              <Check className="w-3 h-3 mr-1" />
              {t("modelSelector.active")}
            </Badge>
          )}
        </div>
        {p.description ? (
          <p className="text-sm text-text/60">{p.description}</p>
        ) : null}
        {p.extras ? <div className="mt-1">{p.extras}</div> : null}
      </div>
    );
  };

  // ── Engine mode: one unified list — providers + models — active pinned top ──
  if (engineMode) {
    type Item = { key: string; node: React.ReactNode };
    const leads = providerEntries.filter((p) => p.position === "lead");
    const trails = providerEntries.filter((p) => p.position === "trail");

    // Stable order: Apple (lead) → GGUF models (downloaded-first via
    // engineSorted) → Custom/Ollama (trail). The active engine is signalled by
    // its "Actif" badge, NOT by reordering — this keeps Custom anchored at the
    // bottom next to its config block and stays consistent with the Cloud tab,
    // which never reorders on selection.
    const items: Item[] = [
      ...leads.map((p) => ({
        key: `provider:${p.id}`,
        node: renderProviderRow(p),
      })),
      ...engineSorted.map((m) => ({
        key: `model:${m.id}`,
        node: renderCard(m, true),
      })),
      ...trails.map((p) => ({
        key: `provider:${p.id}`,
        node: renderProviderRow(p),
      })),
    ];

    return (
      <div id="llm-library-section" className="space-y-3">
        {store.isLoading ? (
          <div className="py-10 flex justify-center">
            <div className="w-7 h-7 border-2 border-logo-primary border-t-transparent rounded-full animate-spin" />
          </div>
        ) : (
          <>
            {items.map((i) => (
              <React.Fragment key={i.key}>{i.node}</React.Fragment>
            ))}
            {footer}
          </>
        )}
      </div>
    );
  }

  // ── Standalone mode (legacy two-section layout) ──────────────────────────
  return (
    <div id="llm-library-section">
      <SettingsGroup
        title={t(
          "settings.postProcessing.modelsAndLocalProcessing.library.title",
        )}
      >
        <div className="p-4 space-y-6">
          <p className="text-sm text-mid-gray leading-relaxed">
            {t(
              "settings.postProcessing.modelsAndLocalProcessing.library.description",
            )}
          </p>

          {store.isLoading ? (
            <div className="py-16 flex justify-center">
              <div className="w-8 h-8 border-2 border-logo-primary border-t-transparent rounded-full animate-spin" />
            </div>
          ) : (
            <>
              {downloadedModels.length > 0 && (
                <div className="space-y-3">
                  <p className="text-sm font-medium text-text/60">
                    {t(
                      "settings.postProcessing.modelsAndLocalProcessing.library.yourModels",
                    )}
                  </p>
                  <div className="space-y-3">
                    {downloadedModels.map((model) => renderCard(model, true))}
                  </div>
                </div>
              )}

              {availableModels.length > 0 && (
                <div className="space-y-3">
                  <p className="text-sm font-medium text-text/60">
                    {t(
                      "settings.postProcessing.modelsAndLocalProcessing.library.availableModels",
                    )}
                  </p>
                  <div className="space-y-3">
                    {availableModels.map((model) => renderCard(model, false))}
                  </div>
                </div>
              )}

              <CustomGgufDropZone />
            </>
          )}
        </div>
      </SettingsGroup>
    </div>
  );
};
