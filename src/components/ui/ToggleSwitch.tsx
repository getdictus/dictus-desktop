import React from "react";
import { SettingContainer } from "./SettingContainer";
import { GlassSwitch } from "./glass/vendor/GlassSwitch";
import {
  behindFor,
  switchTrackFor,
  supportsGlassLens,
  usePrefersDark,
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
  /** Test hook only. */
  testId?: string;
}

/** Board geometry: track 44 x 24. The thumb proportions are the library
 *  example's own, which is the reference for how this control should read. */
const TRACK_WIDTH_PX = 44;
const TRACK_HEIGHT_PX = 24;
const ACCENT_FILL = "#3D7EFF";

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
  testId,
}) => {
  const isDark = usePrefersDark();
  const locked = disabled || isUpdating;

  return (
    <SettingContainer
      title={label}
      description={description}
      descriptionMode={descriptionMode}
      grouped={grouped}
      disabled={disabled}
      tooltipPosition={tooltipPosition}
    >
      <div data-testid={testId}>
        {supportsGlassLens() ? (
          <GlassSwitch
            checked={checked}
            onCheckedChange={onChange}
            disabled={locked}
            width={TRACK_WIDTH_PX}
            height={TRACK_HEIGHT_PX}
            scheme={isDark ? "dark" : "light"}
            trackColor={switchTrackFor(isDark)}
            activeColor={ACCENT_FILL}
            surface={behindFor("switchThumb", isDark)}
            ariaLabel={label}
          />
        ) : (
          // No lens: the plain switch, same track geometry.
          <label
            className={`inline-flex items-center ${locked ? "cursor-not-allowed" : "cursor-pointer"}`}
          >
            <input
              type="checkbox"
              role="switch"
              className="sr-only peer"
              checked={checked}
              disabled={locked}
              aria-label={label}
              onChange={(e) => onChange(e.target.checked)}
            />
            <div className="switch-fallback peer-focus-visible:shadow-[var(--focus-ring)] peer-disabled:opacity-50" />
          </label>
        )}
      </div>
      {isUpdating && (
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="w-4 h-4 border-2 border-accent border-t-transparent rounded-full animate-spin"></div>
        </div>
      )}
    </SettingContainer>
  );
};
