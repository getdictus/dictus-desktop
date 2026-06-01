import React, { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { ask } from "@tauri-apps/plugin-dialog";
import type { ModelInfo } from "@/bindings";
import type { LlmModelInfo } from "@/bindings";
import type { ModelCardStatus } from "@/components/onboarding/ModelCard";
import ModelCard from "@/components/onboarding/ModelCard";
import { SettingsGroup } from "@/components/ui";
import { useLlmModelStore } from "@/stores/llmModelStore";
import { CustomGgufDropZone } from "./CustomGgufDropZone";

/**
 * Adapt LlmModelInfo to ModelInfo so we can reuse ModelCard verbatim.
 * Whisper-only fields are filled with safe zero/empty defaults.
 */
function toModelCardModel(m: LlmModelInfo): ModelInfo {
  return {
    id: m.id,
    name: m.name,
    description: m.description,
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

export const LlmLibrarySection: React.FC = () => {
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
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const getModelStatus = (modelId: string): ModelCardStatus => {
    if (modelId in store.downloadProgress) {
      return "downloading";
    }
    if (modelId in store.verifyingModels) {
      return "verifying";
    }
    if (modelId === store.activeModelId) {
      return "active";
    }
    const model = store.models.find((m) => m.id === modelId);
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
    } else if (status === "available") {
      void store.setActiveModel(modelId);
    }
    // active: no-op
  };

  // Split models: downloaded (active + available) vs catalogue (not downloaded, not downloading)
  const downloadedModels = store.models
    .filter(
      (m) =>
        m.is_downloaded ||
        m.is_downloading ||
        m.id in store.downloadProgress ||
        m.is_custom,
    )
    .sort((a, b) => {
      // Active model pinned to top
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
                    {downloadedModels.map((model) => {
                      const status = getModelStatus(model.id);
                      const progress = store.downloadProgress[model.id];
                      const stats = store.downloadStats[model.id];
                      return (
                        <ModelCard
                          key={model.id}
                          model={toModelCardModel(model)}
                          status={status}
                          onSelect={handleSelect}
                          onDownload={(id) => void store.downloadModel(id)}
                          onCancel={(id) => void store.cancelDownload(id)}
                          onDelete={handleDelete}
                          downloadProgress={progress?.percentage}
                          downloadSpeed={stats?.speedMbps}
                          showRecommended={true}
                        />
                      );
                    })}
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
                    {availableModels.map((model) => {
                      const status = getModelStatus(model.id);
                      return (
                        <ModelCard
                          key={model.id}
                          model={toModelCardModel(model)}
                          status={status}
                          onSelect={handleSelect}
                          onDownload={(id) => void store.downloadModel(id)}
                          showRecommended={true}
                        />
                      );
                    })}
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
