import React from "react";
import { useTranslation } from "react-i18next";
import type { GroupedProviderOption } from "./usePostProcessProviderState";

interface ProviderPickerProps {
  externalOptions: GroupedProviderOption[];
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
  activeTab: "local" | "cloud";
  onTabChange: (tab: "local" | "cloud") => void;
  /**
   * Fully composed on-device engine list (provider cards + GGUF model cards),
   * rendered as the local tab body. Built by the parent so the active engine
   * can be pinned to the top across both provider and model rows.
   */
  localContent?: React.ReactNode;
}

export const ProviderPicker: React.FC<ProviderPickerProps> = ({
  externalOptions,
  value,
  onChange,
  disabled,
  activeTab,
  onTabChange,
  localContent,
}) => {
  const { t } = useTranslation();

  // Cloud provider card — clickable, selection shown by accent border (no
  // radio), matching the on-device cards for visual consistency.
  const renderRow = (option: GroupedProviderOption) => {
    const checked = value === option.value;
    const selectable = !disabled;
    return (
      <div
        key={option.value}
        role="button"
        aria-pressed={checked}
        tabIndex={selectable ? 0 : undefined}
        onClick={() => selectable && onChange(option.value)}
        onKeyDown={(e) => {
          if (selectable && (e.key === "Enter" || e.key === " ")) {
            e.preventDefault();
            onChange(option.value);
          }
        }}
        className={`flex flex-col gap-1 px-4 py-3 rounded-xl border-2 transition-all ${
          checked
            ? "border-logo-primary/50 bg-logo-primary/10"
            : "border-mid-gray/20 hover:border-logo-primary/50 hover:bg-logo-primary/5"
        } ${disabled ? "opacity-50 cursor-not-allowed" : "cursor-pointer"}`}
      >
        <span
          className={`text-base font-semibold text-text ${
            selectable ? "hover:text-logo-primary" : ""
          } transition-colors`}
        >
          {option.label}
        </span>
        {option.description ? (
          <p className="text-sm text-text/60">{option.description}</p>
        ) : null}
      </div>
    );
  };

  const renderSection = (title: string, options: GroupedProviderOption[]) => (
    <fieldset className="space-y-3 border-0 p-0 m-0">
      <legend className="text-xs font-medium text-mid-gray uppercase tracking-wide mb-2">
        {title}
      </legend>
      {options.map((option) => renderRow(option))}
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
      {activeTab === "local" ? (
        <fieldset className="space-y-3 border-0 p-0 m-0">
          <legend className="text-xs font-medium text-mid-gray uppercase tracking-wide mb-2">
            {t("settings.postProcessing.api.providers.sectionLocal")}
          </legend>
          {localContent}
        </fieldset>
      ) : null}

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
