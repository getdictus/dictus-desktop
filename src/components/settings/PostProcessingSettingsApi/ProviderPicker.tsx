import React from "react";
import { useTranslation } from "react-i18next";
import type { GroupedProviderOption } from "./usePostProcessProviderState";

interface ProviderPickerProps {
  localOptions: GroupedProviderOption[];
  externalOptions: GroupedProviderOption[];
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
  renderRowExtras?: (option: GroupedProviderOption) => React.ReactNode;
  activeTab: "local" | "cloud";
  onTabChange: (tab: "local" | "cloud") => void;
  /**
   * On-device GGUF model list, injected into the local tab right after the
   * Apple Intelligence row (or at the top of the local list when Apple
   * Intelligence is unavailable). This is the "the model is the engine" block.
   */
  localModelSlot?: React.ReactNode;
  /**
   * Rendered at the very bottom of the local tab, after every provider row
   * (e.g. the custom-GGUF import zone, kept below the Ollama/Custom row).
   */
  localFooterSlot?: React.ReactNode;
}

export const ProviderPicker: React.FC<ProviderPickerProps> = ({
  localOptions,
  externalOptions,
  value,
  onChange,
  disabled,
  renderRowExtras,
  activeTab,
  onTabChange,
  localModelSlot,
  localFooterSlot,
}) => {
  const { t } = useTranslation();

  const renderRow = (option: GroupedProviderOption) => {
    const checked = value === option.value;
    // Scale matches ModelCard (rounded-xl, border-2, px-4 py-3, text-base name,
    // text-sm description) so provider rows and model cards read as siblings in
    // the same on-device list.
    return (
      <label
        key={option.value}
        className={`flex flex-col gap-1 px-4 py-3 rounded-xl border-2 transition-all ${
          checked
            ? "border-logo-primary/50 bg-logo-primary/10"
            : "border-mid-gray/20 hover:border-logo-primary/50 hover:bg-logo-primary/5"
        } ${disabled ? "opacity-50 cursor-not-allowed" : "cursor-pointer"}`}
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
          <span className="text-base font-semibold text-text">
            {option.label}
          </span>
        </div>
        {option.description ? (
          <p className="text-sm text-text/60 pl-7">{option.description}</p>
        ) : null}
        {renderRowExtras ? (
          <div className="pl-7 mt-1">{renderRowExtras(option)}</div>
        ) : null}
      </label>
    );
  };

  const renderSection = (
    title: string,
    options: GroupedProviderOption[],
    modelSlot?: React.ReactNode,
    footerSlot?: React.ReactNode,
  ) => {
    const hasApple = options.some((o) => o.value === "apple_intelligence");
    return (
      <fieldset className="space-y-3 border-0 p-0 m-0">
        <legend className="text-xs font-medium text-mid-gray uppercase tracking-wide mb-2">
          {title}
        </legend>
        {modelSlot && !hasApple ? modelSlot : null}
        {options.map((option) => (
          <React.Fragment key={option.value}>
            {renderRow(option)}
            {modelSlot && option.value === "apple_intelligence"
              ? modelSlot
              : null}
          </React.Fragment>
        ))}
        {footerSlot ?? null}
      </fieldset>
    );
  };

  return (
    <div className="space-y-4">
      {/* Tabs control */}
      <div
        role="tablist"
        aria-label={t("settings.postProcessing.api.provider.title")}
        className="inline-flex rounded-md border border-mid-gray/20 p-0.5 bg-mid-gray/5"
      >
        <button
          type="button"
          role="tab"
          aria-selected={activeTab === "local"}
          onClick={() => onTabChange("local")}
          className={`px-3 py-1 text-sm font-medium rounded-md transition-colors ${
            activeTab === "local"
              ? "bg-background text-text shadow-sm"
              : "text-mid-gray hover:text-text"
          }`}
        >
          {t("settings.postProcessing.tabs.local")}
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={activeTab === "cloud"}
          onClick={() => onTabChange("cloud")}
          className={`px-3 py-1 text-sm font-medium rounded-md transition-colors ${
            activeTab === "cloud"
              ? "bg-background text-text shadow-sm"
              : "text-mid-gray hover:text-text"
          }`}
        >
          {t("settings.postProcessing.tabs.cloud")}
        </button>
      </div>

      {/* Active section */}
      {activeTab === "local" && localOptions.length > 0
        ? renderSection(
            t("settings.postProcessing.api.providers.sectionLocal"),
            localOptions,
            localModelSlot,
            localFooterSlot,
          )
        : null}

      {activeTab === "cloud" && externalOptions.length > 0
        ? renderSection(
            t("settings.postProcessing.api.providers.sectionExternal"),
            externalOptions,
          )
        : null}
    </div>
  );
};

ProviderPicker.displayName = "ProviderPicker";
