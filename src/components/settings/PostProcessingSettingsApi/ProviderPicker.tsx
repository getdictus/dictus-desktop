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
}) => {
  const { t } = useTranslation();

  const renderSection = (title: string, options: GroupedProviderOption[]) => (
    <fieldset className="space-y-1 border-0 p-0 m-0">
      <legend className="text-xs font-medium text-mid-gray uppercase tracking-wide mb-2">
        {title}
      </legend>
      {options.map((option) => {
        const checked = value === option.value;
        return (
          <label
            key={option.value}
            className={`flex flex-col gap-1 p-3 rounded-md border transition-colors ${
              checked
                ? "border-logo-primary bg-logo-primary/10"
                : "border-mid-gray/20 hover:bg-mid-gray/5"
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
              <span className="text-sm font-medium">{option.label}</span>
            </div>
            {option.description ? (
              <p className="text-xs text-mid-gray pl-7">{option.description}</p>
            ) : null}
            {renderRowExtras ? (
              <div className="pl-7 mt-1">{renderRowExtras(option)}</div>
            ) : null}
          </label>
        );
      })}
    </fieldset>
  );

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
