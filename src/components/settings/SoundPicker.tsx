import React from "react";
import { useTranslation } from "react-i18next";
import type { SoundTheme } from "@/bindings";
import { Button } from "../ui/Button";
import { Dropdown, DropdownOption } from "../ui/Dropdown";
import { PlayIcon } from "lucide-react";
import { SettingContainer } from "../ui/SettingContainer";
import { useSettingsStore } from "../../stores/settingsStore";
import { useSettings } from "../../hooks/useSettings";

interface SoundPickerProps {
  label: string;
  description: string;
}

/** Every sound theme, including the legacy and custom ones, for the Debug page. */
export const SoundPicker: React.FC<SoundPickerProps> = ({
  label,
  description,
}) => {
  const { t } = useTranslation();
  const { getSetting, updateSetting } = useSettings();
  const previewSoundPack = useSettingsStore((state) => state.previewSoundPack);
  const customSounds = useSettingsStore((state) => state.customSounds);

  const selectedTheme = getSetting("sound_theme") ?? "marimba";

  const themes: SoundTheme[] = ["bois", "synthe", "minimal", "marimba", "pop"];
  // Only add Custom if both custom sound files exist
  if (customSounds.start && customSounds.stop) {
    themes.push("custom");
  }
  const options: DropdownOption[] = themes.map((theme) => ({
    value: theme,
    label: t(`settings.sound.pack.packs.${theme}`),
  }));

  return (
    <SettingContainer
      title={label}
      description={description}
      grouped
      layout="horizontal"
    >
      <div className="flex items-center gap-2">
        <Dropdown
          selectedValue={selectedTheme}
          onSelect={(value) =>
            updateSetting("sound_theme", value as SoundTheme)
          }
          options={options}
        />
        <Button
          variant="ghost"
          size="sm"
          onClick={() => previewSoundPack(selectedTheme)}
          title={t("settings.sound.pack.preview", {
            pack: t(`settings.sound.pack.packs.${selectedTheme}`),
          })}
        >
          <PlayIcon className="h-4 w-4" />
        </Button>
      </div>
    </SettingContainer>
  );
};
