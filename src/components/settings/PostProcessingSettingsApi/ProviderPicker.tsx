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
  enableCloudProviders?: boolean;
  onToggleCloudProviders?: (enabled: boolean) => void;
  isUpdatingCloudToggle?: boolean;
}

export const ProviderPicker: React.FC<ProviderPickerProps> = ({
  localOptions,
  externalOptions,
  value,
  onChange,
  disabled,
  renderRowExtras,
  enableCloudProviders = false,
  onToggleCloudProviders,
  isUpdatingCloudToggle = false,
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
            {checked && renderRowExtras ? (
              <div className="pl-7 mt-1">{renderRowExtras(option)}</div>
            ) : null}
          </label>
        );
      })}
    </fieldset>
  );

  return (
    <div className="space-y-4">
      {localOptions.length > 0
        ? renderSection(
            t("settings.postProcessing.api.providers.sectionLocal"),
            localOptions,
          )
        : null}

      {onToggleCloudProviders ? (
        <div className="flex items-start justify-between gap-3 p-3 rounded-md border border-mid-gray/20 bg-mid-gray/5">
          <div className="flex flex-col gap-1">
            <span className="text-sm font-medium">
              {t("settings.postProcessing.cloudToggle.label")}
            </span>
            <span className="text-xs text-mid-gray">
              {t("settings.postProcessing.cloudToggle.description")}
            </span>
          </div>
          <label
            className={`inline-flex items-center ${isUpdatingCloudToggle ? "cursor-not-allowed" : "cursor-pointer"}`}
          >
            <input
              type="checkbox"
              className="sr-only peer"
              checked={enableCloudProviders}
              disabled={isUpdatingCloudToggle}
              onChange={(e) => onToggleCloudProviders(e.target.checked)}
            />
            <div className="relative w-11 h-6 bg-mid-gray/20 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-logo-primary rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-background-ui peer-disabled:opacity-50"></div>
          </label>
        </div>
      ) : null}

      {enableCloudProviders && externalOptions.length > 0
        ? renderSection(
            t("settings.postProcessing.api.providers.sectionExternal"),
            externalOptions,
          )
        : null}
    </div>
  );
};

ProviderPicker.displayName = "ProviderPicker";
