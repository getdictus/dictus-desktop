import React from "react";
import { useTranslation } from "react-i18next";
import { PlayIcon } from "lucide-react";
import type { SoundTheme } from "@/bindings";
import { SettingContainer } from "../ui/SettingContainer";
import { useSettingsStore } from "../../stores/settingsStore";
import { useSettings } from "../../hooks/useSettings";

/** The packs offered to everyone. Marimba, Pop and Custom have no recorded
 * licence or need hand-placed files: they are only listed while selected. */
const PICKER_PACKS: SoundTheme[] = ["bois", "synthe", "minimal"];

export const SoundPackPicker: React.FC<{ disabled?: boolean }> = ({
  disabled = false,
}) => {
  const { t } = useTranslation();
  const { getSetting, updateSetting } = useSettings();
  const previewSoundPack = useSettingsStore((state) => state.previewSoundPack);
  const selected = getSetting("sound_theme") ?? "marimba";
  const packs = PICKER_PACKS.includes(selected)
    ? PICKER_PACKS
    : [...PICKER_PACKS, selected];

  return (
    <SettingContainer
      title={t("settings.sound.pack.title")}
      description={t("settings.sound.pack.description")}
      descriptionMode="tooltip"
      grouped
      layout="horizontal"
      disabled={disabled}
    >
      <div role="radiogroup" className="flex flex-wrap justify-end gap-2">
        {packs.map((pack) => {
          const label = t(`settings.sound.pack.packs.${pack}`);
          const isSelected = pack === selected;
          return (
            <div
              key={pack}
              className={`flex items-center rounded-lg border text-sm transition-colors ${
                isSelected
                  ? "border-logo-primary bg-logo-primary/20"
                  : "border-mid-gray/20 bg-mid-gray/10"
              } ${disabled ? "opacity-50" : ""}`}
            >
              <button
                type="button"
                role="radio"
                aria-checked={isSelected}
                disabled={disabled}
                onClick={() => updateSetting("sound_theme", pack)}
                className="py-1 ps-3 pe-1 font-medium cursor-pointer disabled:cursor-not-allowed"
              >
                {label}
              </button>
              <button
                type="button"
                disabled={disabled}
                onClick={() => previewSoundPack(pack)}
                title={t("settings.sound.pack.preview", { pack: label })}
                aria-label={t("settings.sound.pack.preview", { pack: label })}
                className="p-1.5 pe-2 rounded-e-lg cursor-pointer hover:text-logo-primary disabled:cursor-not-allowed"
              >
                <PlayIcon className="h-3.5 w-3.5" />
              </button>
            </div>
          );
        })}
      </div>
    </SettingContainer>
  );
};
