import React from "react";
import { useTranslation } from "react-i18next";
import { ToggleSwitch } from "../../ui/ToggleSwitch";
import { useSettings } from "../../../hooks/useSettings";

interface LogTranscriptsToggleProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const LogTranscriptsToggle: React.FC<LogTranscriptsToggleProps> = ({
  descriptionMode = "tooltip",
  grouped = false,
}) => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating } = useSettings();

  return (
    <ToggleSwitch
      checked={getSetting("log_transcripts") ?? false}
      onChange={(enabled) => updateSetting("log_transcripts", enabled)}
      isUpdating={isUpdating("log_transcripts")}
      label={t("settings.debug.logTranscripts.label")}
      description={t("settings.debug.logTranscripts.description")}
      descriptionMode={descriptionMode}
      grouped={grouped}
    />
  );
};
