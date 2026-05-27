import React, { useEffect, useState } from "react";
import { Trans, useTranslation } from "react-i18next";
import { RefreshCcw } from "lucide-react";
import { commands } from "@/bindings";

import { Alert } from "../../ui/Alert";
import {
  Dropdown,
  SettingContainer,
  SettingsGroup,
  Textarea,
} from "@/components/ui";
import { Button } from "../../ui/Button";
import { ResetButton } from "../../ui/ResetButton";
import { Input } from "../../ui/Input";

import { ProviderPicker } from "../PostProcessingSettingsApi/ProviderPicker";
import { TestConnectionButton } from "../PostProcessingSettingsApi/TestConnectionButton";
import { BaseUrlField } from "../PostProcessingSettingsApi/BaseUrlField";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ApiKeyField } from "../PostProcessingSettingsApi/ApiKeyField";
import { ModelSelect } from "../PostProcessingSettingsApi/ModelSelect";
import { usePostProcessProviderState } from "../PostProcessingSettingsApi/usePostProcessProviderState";
import { ShortcutInput } from "../ShortcutInput";
import { useSettings } from "../../../hooks/useSettings";
import type { PostProcessProvider } from "@/bindings";

const LOCAL_PROVIDER_IDS_SET = new Set(["apple_intelligence", "custom"]);
const RECOMMENDED_PROVIDER_ID = "apple_intelligence";

const PostProcessingSettingsApiComponent: React.FC = () => {
  const { t } = useTranslation();
  const state = usePostProcessProviderState();

  const selectedIsCloud =
    state.selectedProviderId !== "" &&
    !LOCAL_PROVIDER_IDS_SET.has(state.selectedProviderId);

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
            {state.selectedProvider ? (
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
          localOptions={state.groupedProviderOptions.local}
          externalOptions={state.groupedProviderOptions.external}
          value={state.selectedProviderId}
          onChange={state.handleProviderSelect}
          activeTab={activeTab}
          onTabChange={handleTabChange}
          renderRowExtras={(option) => {
            if (option.value === "apple_intelligence") {
              if (!state.appleIntelligenceUnavailable) return null;
              return (
                <Alert variant="error" contained>
                  {t(
                    "settings.postProcessing.api.appleIntelligence.unavailable",
                  )}
                </Alert>
              );
            }
            if (option.value !== "custom") return null;
            return (
              <div className="space-y-2 mt-2">
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
                            void openUrl("https://ollama.com");
                          }}
                          onKeyDown={(e) => {
                            if (e.key === "Enter" || e.key === " ") {
                              e.preventDefault();
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
                <TestConnectionButton baseUrl={state.baseUrl} />
              </div>
            );
          }}
        />
      </SettingContainer>

      {!state.isAppleProvider && (
        <>
          {state.selectedProvider?.id === "custom" && (
            <SettingContainer
              title={t("settings.postProcessing.api.baseUrl.title")}
              description={t("settings.postProcessing.api.baseUrl.description")}
              descriptionMode="tooltip"
              layout="horizontal"
              grouped={true}
            >
              <div className="flex items-center gap-2">
                <BaseUrlField
                  value={state.baseUrl}
                  onBlur={state.handleBaseUrlChange}
                  placeholder={t(
                    "settings.postProcessing.api.baseUrl.placeholder",
                  )}
                  disabled={state.isBaseUrlUpdating}
                  className="min-w-[380px]"
                />
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

      {!state.isAppleProvider && (
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

const PostProcessingSettingsPromptsComponent: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating, refreshSettings } =
    useSettings();
  const [isCreating, setIsCreating] = useState(false);
  const [draftName, setDraftName] = useState("");
  const [draftText, setDraftText] = useState("");

  const prompts = getSetting("post_process_prompts") || [];
  const selectedPromptId = getSetting("post_process_selected_prompt_id") || "";
  const selectedPrompt =
    prompts.find((prompt) => prompt.id === selectedPromptId) || null;

  useEffect(() => {
    if (isCreating) return;

    if (selectedPrompt) {
      setDraftName(selectedPrompt.name);
      setDraftText(selectedPrompt.prompt);
    } else {
      setDraftName("");
      setDraftText("");
    }
  }, [
    isCreating,
    selectedPromptId,
    selectedPrompt?.name,
    selectedPrompt?.prompt,
  ]);

  const handlePromptSelect = (promptId: string | null) => {
    if (!promptId) return;
    updateSetting("post_process_selected_prompt_id", promptId);
    setIsCreating(false);
  };

  const handleCreatePrompt = async () => {
    if (!draftName.trim() || !draftText.trim()) return;

    try {
      const result = await commands.addPostProcessPrompt(
        draftName.trim(),
        draftText.trim(),
      );
      if (result.status === "ok") {
        await refreshSettings();
        updateSetting("post_process_selected_prompt_id", result.data.id);
        setIsCreating(false);
      }
    } catch (error) {
      console.error("Failed to create prompt:", error);
    }
  };

  const handleUpdatePrompt = async () => {
    if (!selectedPromptId || !draftName.trim() || !draftText.trim()) return;

    try {
      await commands.updatePostProcessPrompt(
        selectedPromptId,
        draftName.trim(),
        draftText.trim(),
      );
      await refreshSettings();
    } catch (error) {
      console.error("Failed to update prompt:", error);
    }
  };

  const handleDeletePrompt = async (promptId: string) => {
    if (!promptId) return;

    try {
      await commands.deletePostProcessPrompt(promptId);
      await refreshSettings();
      setIsCreating(false);
    } catch (error) {
      console.error("Failed to delete prompt:", error);
    }
  };

  const handleCancelCreate = () => {
    setIsCreating(false);
    if (selectedPrompt) {
      setDraftName(selectedPrompt.name);
      setDraftText(selectedPrompt.prompt);
    } else {
      setDraftName("");
      setDraftText("");
    }
  };

  const handleStartCreate = () => {
    setIsCreating(true);
    setDraftName("");
    setDraftText("");
  };

  const hasPrompts = prompts.length > 0;
  const isDirty =
    !!selectedPrompt &&
    (draftName.trim() !== selectedPrompt.name ||
      draftText.trim() !== selectedPrompt.prompt.trim());

  return (
    <SettingContainer
      title={t("settings.postProcessing.prompts.selectedPrompt.title")}
      description={t(
        "settings.postProcessing.prompts.selectedPrompt.description",
      )}
      descriptionMode="tooltip"
      layout="stacked"
      grouped={true}
    >
      <div className="space-y-3">
        <div className="flex gap-2">
          <Dropdown
            selectedValue={selectedPromptId || null}
            options={prompts.map((p) => ({
              value: p.id,
              label: p.name,
            }))}
            onSelect={(value) => handlePromptSelect(value)}
            placeholder={
              prompts.length === 0
                ? t("settings.postProcessing.prompts.noPrompts")
                : t("settings.postProcessing.prompts.selectPrompt")
            }
            disabled={
              isUpdating("post_process_selected_prompt_id") || isCreating
            }
            className="flex-1"
          />
          <Button
            onClick={handleStartCreate}
            variant="primary"
            size="md"
            disabled={isCreating}
          >
            {t("settings.postProcessing.prompts.createNew")}
          </Button>
        </div>

        {!isCreating && hasPrompts && selectedPrompt && (
          <div className="space-y-3">
            <div className="space-y-2 flex flex-col">
              <label className="text-sm font-semibold">
                {t("settings.postProcessing.prompts.promptLabel")}
              </label>
              <Input
                type="text"
                value={draftName}
                onChange={(e) => setDraftName(e.target.value)}
                placeholder={t(
                  "settings.postProcessing.prompts.promptLabelPlaceholder",
                )}
                variant="compact"
              />
            </div>

            <div className="space-y-2 flex flex-col">
              <label className="text-sm font-semibold">
                {t("settings.postProcessing.prompts.promptInstructions")}
              </label>
              <Textarea
                value={draftText}
                onChange={(e) => setDraftText(e.target.value)}
                placeholder={t(
                  "settings.postProcessing.prompts.promptInstructionsPlaceholder",
                )}
              />
              <p className="text-xs text-mid-gray/70">
                <Trans
                  i18nKey="settings.postProcessing.prompts.promptTip"
                  components={{ code: <code /> }}
                />
              </p>
            </div>

            <div className="flex gap-2 pt-2">
              <Button
                onClick={handleUpdatePrompt}
                variant="primary"
                size="md"
                disabled={!draftName.trim() || !draftText.trim() || !isDirty}
              >
                {t("settings.postProcessing.prompts.updatePrompt")}
              </Button>
              <Button
                onClick={() => handleDeletePrompt(selectedPromptId)}
                variant="secondary"
                size="md"
                disabled={!selectedPromptId || prompts.length <= 1}
              >
                {t("settings.postProcessing.prompts.deletePrompt")}
              </Button>
            </div>
          </div>
        )}

        {!isCreating && !selectedPrompt && (
          <div className="p-3 bg-mid-gray/5 rounded-md border border-mid-gray/20">
            <p className="text-sm text-mid-gray">
              {hasPrompts
                ? t("settings.postProcessing.prompts.selectToEdit")
                : t("settings.postProcessing.prompts.createFirst")}
            </p>
          </div>
        )}

        {isCreating && (
          <div className="space-y-3">
            <div className="space-y-2 block flex flex-col">
              <label className="text-sm font-semibold text-text">
                {t("settings.postProcessing.prompts.promptLabel")}
              </label>
              <Input
                type="text"
                value={draftName}
                onChange={(e) => setDraftName(e.target.value)}
                placeholder={t(
                  "settings.postProcessing.prompts.promptLabelPlaceholder",
                )}
                variant="compact"
              />
            </div>

            <div className="space-y-2 flex flex-col">
              <label className="text-sm font-semibold">
                {t("settings.postProcessing.prompts.promptInstructions")}
              </label>
              <Textarea
                value={draftText}
                onChange={(e) => setDraftText(e.target.value)}
                placeholder={t(
                  "settings.postProcessing.prompts.promptInstructionsPlaceholder",
                )}
              />
              <p className="text-xs text-mid-gray/70">
                <Trans
                  i18nKey="settings.postProcessing.prompts.promptTip"
                  components={{ code: <code /> }}
                />
              </p>
            </div>

            <div className="flex gap-2 pt-2">
              <Button
                onClick={handleCreatePrompt}
                variant="primary"
                size="md"
                disabled={!draftName.trim() || !draftText.trim()}
              >
                {t("settings.postProcessing.prompts.createPrompt")}
              </Button>
              <Button
                onClick={handleCancelCreate}
                variant="secondary"
                size="md"
              >
                {t("settings.postProcessing.prompts.cancel")}
              </Button>
            </div>
          </div>
        )}
      </div>
    </SettingContainer>
  );
};

export const PostProcessingSettingsApi = React.memo(
  PostProcessingSettingsApiComponent,
);
PostProcessingSettingsApi.displayName = "PostProcessingSettingsApi";

export const PostProcessingSettingsPrompts = React.memo(
  PostProcessingSettingsPromptsComponent,
);
PostProcessingSettingsPrompts.displayName = "PostProcessingSettingsPrompts";

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

      {/* Local model library — coming-soon placeholder (hoisted to top per Gap 7) */}
      <SettingsGroup
        title={t(
          "settings.postProcessing.modelsAndLocalProcessing.library.title",
        )}
      >
        <div className="rounded-md border border-dashed border-mid-gray/40 bg-mid-gray/5 p-4 space-y-2">
          <div className="flex items-center gap-2">
            <span className="text-xs font-medium px-2 py-0.5 rounded-full bg-logo-primary/10 text-logo-primary border border-logo-primary/40">
              {t(
                "settings.postProcessing.modelsAndLocalProcessing.library.comingSoonBadge",
              )}
            </span>
          </div>
          <p className="text-sm text-mid-gray">
            {t(
              "settings.postProcessing.modelsAndLocalProcessing.library.comingSoonBody",
            )}
          </p>
          <p className="text-xs text-mid-gray/80">
            {t(
              "settings.postProcessing.modelsAndLocalProcessing.library.currentBridge",
            )}
          </p>
        </div>
      </SettingsGroup>

      {/* Hotkey */}
      <SettingsGroup title={t("settings.postProcessing.hotkey.title")}>
        <ShortcutInput
          shortcutId="transcribe_with_post_process"
          descriptionMode="tooltip"
          grouped={true}
        />
      </SettingsGroup>

      {/* API (includes selected model card + ProviderPicker with tabs) */}
      <SettingsGroup title={t("settings.postProcessing.api.title")}>
        <PostProcessingSettingsApi />
      </SettingsGroup>

      {/* Prompts */}
      <SettingsGroup title={t("settings.postProcessing.prompts.title")}>
        <PostProcessingSettingsPrompts />
      </SettingsGroup>
    </div>
  );
};
