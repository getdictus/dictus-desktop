import React, { useEffect, useState } from "react";
import { Trans, useTranslation } from "react-i18next";
import { RefreshCcw } from "lucide-react";

import { Alert } from "../../ui/Alert";
import { SettingContainer, SettingsGroup } from "@/components/ui";
import { Button } from "../../ui/Button";
import { ResetButton } from "../../ui/ResetButton";

import { ProviderPicker } from "../PostProcessingSettingsApi/ProviderPicker";
import { TestConnectionButton } from "../PostProcessingSettingsApi/TestConnectionButton";
import { BaseUrlField } from "../PostProcessingSettingsApi/BaseUrlField";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ApiKeyField } from "../PostProcessingSettingsApi/ApiKeyField";
import { ModelSelect } from "../PostProcessingSettingsApi/ModelSelect";
import { usePostProcessProviderState } from "../PostProcessingSettingsApi/usePostProcessProviderState";
import { useSettings } from "../../../hooks/useSettings";
import { SmartModesSection } from "./SmartModesSection";
import type { PostProcessProvider } from "@/bindings";
import { LlmLibrarySection } from "./LlmLibrarySection";
import type { ProviderEntry } from "./LlmLibrarySection";
import { CustomGgufDropZone } from "./CustomGgufDropZone";
import { useLlmModelStore } from "@/stores/llmModelStore";

const LOCAL_PROVIDER_IDS_SET = new Set([
  "apple_intelligence",
  "custom",
  "embedded",
]);
const RECOMMENDED_PROVIDER_ID = "apple_intelligence";

const PostProcessingSettingsApiComponent: React.FC = () => {
  const { t } = useTranslation();
  const state = usePostProcessProviderState();
  const llmStore = useLlmModelStore();

  // The embedded provider is no longer a standalone radio — the local GGUF
  // models ARE the engine choices (see UI-SPEC R2). Selecting a downloaded
  // model sets provider=embedded + active model in one action.
  const isEmbeddedSelected = state.selectedProviderId === "embedded";
  const handleSelectEmbeddedModel = async (modelId: string) => {
    await llmStore.setActiveModel(modelId);
    await state.handleProviderSelect("embedded");
  };

  // Name of the active local model, used to label the "active engine" summary
  // when embedded is selected (e.g. "Qwen2.5 1.5B" instead of a generic label).
  const activeLlmName =
    llmStore.models.find((m) => m.id === llmStore.activeModelId)?.name ??
    t(
      "settings.postProcessing.modelsAndLocalProcessing.embedded.providerLabel",
    );

  // Non-model on-device engines (Apple Intelligence, Ollama/Custom) interleaved
  // with the GGUF model cards. Apple leads (top), Custom trails (bottom) so it
  // stays anchored next to its config block; the active engine is shown by its
  // "Actif" badge, not by reordering.
  const providerEntries: ProviderEntry[] =
    state.groupedProviderOptions.local.map((opt) => ({
      id: opt.value,
      label: opt.label,
      description: opt.description,
      checked: state.selectedProviderId === opt.value,
      position: opt.value === "custom" ? "trail" : "lead",
      onSelect: (id: string) => void state.handleProviderSelect(id),
      extras:
        opt.value === "apple_intelligence" ? (
          state.appleIntelligenceUnavailable ? (
            <Alert variant="error" contained>
              {t("settings.postProcessing.api.appleIntelligence.unavailable")}
            </Alert>
          ) : undefined
        ) : opt.value === "custom" ? (
          <p className="text-xs text-mid-gray/80">
            <Trans
              i18nKey="settings.postProcessing.api.custom.ollamaTip"
              components={{
                link: (
                  <a
                    role="link"
                    tabIndex={0}
                    className="text-logo-primary underline underline-offset-2 hover:opacity-80 cursor-pointer"
                    onClick={(e) => {
                      e.preventDefault();
                      e.stopPropagation();
                      void openUrl("https://ollama.com");
                    }}
                    onKeyDown={(e) => {
                      if (e.key === "Enter" || e.key === " ") {
                        e.preventDefault();
                        e.stopPropagation();
                        void openUrl("https://ollama.com");
                      }
                    }}
                  />
                ),
                code: (
                  <code className="font-mono text-xs bg-mid-gray/10 px-1 rounded" />
                ),
              }}
            />
          </p>
        ) : undefined,
    }));

  // Full on-device tab body: provider cards + model cards in stable order,
  // import zone last.
  const localContent = (
    <LlmLibrarySection
      engineMode
      embeddedSelected={isEmbeddedSelected}
      onSelectAsEngine={(id) => void handleSelectEmbeddedModel(id)}
      providerEntries={providerEntries}
      footer={<CustomGgufDropZone />}
    />
  );

  const selectedIsCloud =
    state.selectedProviderId !== "" &&
    !LOCAL_PROVIDER_IDS_SET.has(state.selectedProviderId);

  // Embedded is an in-process provider with no external config (no API key,
  // base URL, or model dropdown) — same as Apple Intelligence. Because
  // "embedded" is synthetic, state.selectedProvider falls back to providers[0],
  // so gate these sections on the id directly rather than the resolved object.
  const isEmbedded = isEmbeddedSelected;

  const initialTab: "local" | "cloud" =
    state.selectedProviderId !== "" &&
    !LOCAL_PROVIDER_IDS_SET.has(state.selectedProviderId)
      ? "cloud"
      : "local";
  const [activeTab, setActiveTab] = useState<"local" | "cloud">(initialTab);

  // Remember the last selected provider per tab so re-clicking a tab restores
  // the user's prior choice in that tab instead of leaving them on a
  // cross-tab selection that triggered the old "cloud selected" warning.
  const [lastLocalId, setLastLocalId] = useState<string>(
    initialTab === "local" ? state.selectedProviderId : "",
  );
  const [lastCloudId, setLastCloudId] = useState<string>(
    initialTab === "cloud" ? state.selectedProviderId : "",
  );

  // Keep tab in sync when the selected provider changes (e.g. user clicks a
  // provider radio inside the active tab) AND remember per-tab last selection.
  useEffect(() => {
    if (state.selectedProviderId === "") return;
    const isLocal = LOCAL_PROVIDER_IDS_SET.has(state.selectedProviderId);
    setActiveTab(isLocal ? "local" : "cloud");
    if (isLocal) {
      setLastLocalId(state.selectedProviderId);
    } else {
      setLastCloudId(state.selectedProviderId);
    }
  }, [state.selectedProviderId]);

  const handleTabChange = (nextTab: "local" | "cloud") => {
    if (nextTab === activeTab) return;
    setActiveTab(nextTab);
    const tabOptions =
      nextTab === "local"
        ? state.groupedProviderOptions.local
        : state.groupedProviderOptions.external;
    if (tabOptions.length === 0) return;
    const preferred = nextTab === "local" ? lastLocalId : lastCloudId;
    const targetId =
      preferred && tabOptions.some((o) => o.value === preferred)
        ? preferred
        : tabOptions[0].value;
    if (targetId !== state.selectedProviderId) {
      void state.handleProviderSelect(targetId);
    }
  };

  return (
    <>
      {/* Selected model card */}
      <div className="rounded-md border border-mid-gray/20 bg-background p-4 space-y-3">
        <div className="flex items-start justify-between gap-3">
          <div>
            <h3 className="text-sm font-medium">
              {t(
                "settings.postProcessing.modelsAndLocalProcessing.selectedModel.title",
              )}
            </h3>
            {isEmbedded ? (
              <p className="text-base font-medium mt-1">{activeLlmName}</p>
            ) : state.selectedProvider ? (
              <p className="text-base font-medium mt-1">
                {state.selectedProvider.label}
              </p>
            ) : null}
          </div>
          {state.selectedProvider?.id === RECOMMENDED_PROVIDER_ID ? (
            <span className="text-xs font-medium px-2 py-0.5 rounded-full border border-logo-primary text-logo-primary">
              {t(
                "settings.postProcessing.modelsAndLocalProcessing.selectedModel.recommendedBadge",
              )}
            </span>
          ) : null}
        </div>
        {!selectedIsCloud ? (
          <div className="flex flex-wrap gap-1.5">
            <span className="text-xs px-2 py-0.5 rounded-full bg-mid-gray/10 text-mid-gray">
              {t(
                "settings.postProcessing.modelsAndLocalProcessing.selectedModel.tags.local",
              )}
            </span>
            <span className="text-xs px-2 py-0.5 rounded-full bg-mid-gray/10 text-mid-gray">
              {t(
                "settings.postProcessing.modelsAndLocalProcessing.selectedModel.tags.private",
              )}
            </span>
            <span className="text-xs px-2 py-0.5 rounded-full bg-mid-gray/10 text-mid-gray">
              {t(
                "settings.postProcessing.modelsAndLocalProcessing.selectedModel.tags.noDataSent",
              )}
            </span>
            <span className="text-xs px-2 py-0.5 rounded-full bg-mid-gray/10 text-mid-gray">
              {t(
                "settings.postProcessing.modelsAndLocalProcessing.selectedModel.tags.offlineCapable",
              )}
            </span>
          </div>
        ) : null}
      </div>

      <SettingContainer
        title={t("settings.postProcessing.api.provider.title")}
        description={t("settings.postProcessing.api.provider.description")}
        descriptionMode="tooltip"
        layout="stacked"
        grouped={true}
      >
        <ProviderPicker
          externalOptions={state.groupedProviderOptions.external}
          value={state.selectedProviderId}
          onChange={state.handleProviderSelect}
          activeTab={activeTab}
          onTabChange={handleTabChange}
          localContent={localContent}
        />
      </SettingContainer>

      {!state.isAppleProvider && !isEmbedded && (
        <>
          {state.selectedProvider?.id === "custom" && (
            <SettingContainer
              title={t("settings.postProcessing.api.baseUrl.title")}
              description={t("settings.postProcessing.api.baseUrl.description")}
              descriptionMode="tooltip"
              layout="stacked"
              grouped={true}
            >
              <div className="flex flex-col items-start gap-2">
                <BaseUrlField
                  value={state.baseUrl}
                  onBlur={state.handleBaseUrlChange}
                  placeholder={t(
                    "settings.postProcessing.api.baseUrl.placeholder",
                  )}
                  disabled={state.isBaseUrlUpdating}
                  className="w-full"
                />
                <TestConnectionButton baseUrl={state.baseUrl} />
              </div>
            </SettingContainer>
          )}

          {state.selectedProvider?.id !== "custom" && (
            <SettingContainer
              title={t("settings.postProcessing.api.apiKey.title")}
              description={t("settings.postProcessing.api.apiKey.description")}
              descriptionMode="tooltip"
              layout="horizontal"
              grouped={true}
            >
              <div className="flex items-center gap-2">
                <ApiKeyField
                  value={state.apiKey}
                  onBlur={state.handleApiKeyChange}
                  placeholder={t(
                    "settings.postProcessing.api.apiKey.placeholder",
                  )}
                  disabled={state.isApiKeyUpdating}
                  className="min-w-[320px]"
                />
              </div>
            </SettingContainer>
          )}
        </>
      )}

      {!state.isAppleProvider && !isEmbedded && (
        <SettingContainer
          title={t("settings.postProcessing.api.model.title")}
          description={
            state.isCustomProvider
              ? t("settings.postProcessing.api.model.descriptionCustom")
              : t("settings.postProcessing.api.model.descriptionDefault")
          }
          descriptionMode="tooltip"
          layout="stacked"
          grouped={true}
        >
          <div className="flex items-center gap-2">
            <ModelSelect
              value={state.model}
              options={state.modelOptions}
              disabled={state.isModelUpdating}
              isLoading={state.isFetchingModels}
              placeholder={
                state.modelOptions.length > 0
                  ? t(
                      "settings.postProcessing.api.model.placeholderWithOptions",
                    )
                  : t("settings.postProcessing.api.model.placeholderNoOptions")
              }
              onSelect={state.handleModelSelect}
              onCreate={state.handleModelCreate}
              onBlur={() => {}}
              className="flex-1 min-w-[380px]"
            />
            <ResetButton
              onClick={state.handleRefreshModels}
              disabled={state.isFetchingModels}
              ariaLabel={t("settings.postProcessing.api.model.refreshModels")}
              className="flex h-10 w-10 items-center justify-center"
            >
              <RefreshCcw
                className={`h-4 w-4 ${state.isFetchingModels ? "animate-spin" : ""}`}
              />
            </ResetButton>
          </div>
        </SettingContainer>
      )}
    </>
  );
};

export const PostProcessingSettingsApi = React.memo(
  PostProcessingSettingsApiComponent,
);
PostProcessingSettingsApi.displayName = "PostProcessingSettingsApi";

export const PostProcessingSettings: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting } = useSettings();
  const providers =
    (getSetting("post_process_providers") as
      | PostProcessProvider[]
      | undefined) ?? [];
  // Count of LOCAL providers that are present and considered "ready" (visible by default).
  // For Phase 8 gap-closure: a local provider is "ready" if it exists in settings (Apple Intelligence
  // is only inserted on macOS ARM64; custom is always present). Runtime availability of Apple Intelligence
  // is not probed here to avoid the SIGABRT race documented in settings.rs.
  const readyCount = providers.filter((p) =>
    LOCAL_PROVIDER_IDS_SET.has(p.id),
  ).length;

  const handleLearnMore = () => {
    void openUrl(
      "https://github.com/getdictus/dictus-desktop/blob/main/docs/PRIVACY.md",
    );
  };

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      {/* Header — title + status badge */}
      <div className="flex items-start justify-between gap-3">
        <div>
          <h1 className="text-xl font-medium">
            {t("settings.postProcessing.modelsAndLocalProcessing.title")}
          </h1>
          <p className="text-sm text-mid-gray mt-1">
            {t("settings.postProcessing.modelsAndLocalProcessing.subtitle")}{" "}
            <a
              role="link"
              tabIndex={0}
              className="text-logo-primary hover:underline cursor-pointer"
              onClick={(e) => {
                e.preventDefault();
                handleLearnMore();
              }}
              onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  handleLearnMore();
                }
              }}
            >
              {t("settings.postProcessing.modelsAndLocalProcessing.learnMore")}
            </a>
          </p>
        </div>
        <span className="shrink-0 text-xs font-medium px-2 py-1 rounded-full border border-logo-primary/40 bg-logo-primary/10 text-logo-primary">
          {readyCount === 1
            ? t(
                "settings.postProcessing.modelsAndLocalProcessing.statusBadge.ready_one",
                { count: readyCount },
              )
            : t(
                "settings.postProcessing.modelsAndLocalProcessing.statusBadge.ready_other",
                { count: readyCount },
              )}
        </span>
      </div>

      {/* API (includes selected model card + ProviderPicker with tabs) */}
      <SettingsGroup title={t("settings.postProcessing.api.title")}>
        <PostProcessingSettingsApi />
      </SettingsGroup>

      {/* Smart Modes */}
      <SmartModesSection />
    </div>
  );
};
