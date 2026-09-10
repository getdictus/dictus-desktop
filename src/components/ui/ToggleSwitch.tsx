import React from "react";
import { SettingContainer } from "./SettingContainer";
import {
  GlassLens,
  behindFor,
  opticsFor,
  usePrefersDark,
  SWITCH_BASE,
  SWITCH_LIGHT,
  SWITCH_DARK,
} from "./glass";

interface ToggleSwitchProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  isUpdating?: boolean;
  label: string;
  description: string;
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
  tooltipPosition?: "top" | "bottom";
}

/** Track 44 x 24, thumb 20 x 20 resting 2 px in, so the travel is 20 px. */
const THUMB_TRAVEL_PX = 20;

export const ToggleSwitch: React.FC<ToggleSwitchProps> = ({
  checked,
  onChange,
  disabled = false,
  isUpdating = false,
  label,
  description,
  descriptionMode = "tooltip",
  grouped = false,
  tooltipPosition = "top",
}) => {
  const isDark = usePrefersDark();

  return (
    <SettingContainer
      title={label}
      description={description}
      descriptionMode={descriptionMode}
      grouped={grouped}
      disabled={disabled}
      tooltipPosition={tooltipPosition}
    >
      <label
        className={`inline-flex items-center ${disabled || isUpdating ? "cursor-not-allowed" : "cursor-pointer"}`}
      >
        <input
          type="checkbox"
          value=""
          className="sr-only peer"
          checked={checked}
          disabled={disabled || isUpdating}
          onChange={(e) => onChange(e.target.checked)}
        />
        <div
          className="toggle-track relative w-11 h-6 rounded-xl transition-colors peer-focus-visible:shadow-[var(--focus-ring)] peer-disabled:opacity-50"
          style={{
            background: checked ? "#3D7EFF" : "var(--toggle-track-off)",
          }}
        >
          {/* The thumb is a lens, not a white circle. */}
          <GlassLens
            className="toggle-thumb absolute top-[2px] start-[2px]"
            style={
              {
                "--toggle-travel": checked ? `${THUMB_TRAVEL_PX}px` : "0px",
              } as React.CSSProperties
            }
            behind={behindFor("switchThumb", isDark)}
            width={20}
            height={20}
            radius={10}
            optics={opticsFor(SWITCH_BASE, SWITCH_LIGHT, SWITCH_DARK, isDark)}
          />
        </div>
      </label>
      {isUpdating && (
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="w-4 h-4 border-2 border-accent border-t-transparent rounded-full animate-spin"></div>
        </div>
      )}
    </SettingContainer>
  );
};
