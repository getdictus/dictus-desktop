import React from "react";
import { SettingContainer } from "./SettingContainer";
import { GlassSlider } from "./glass/vendor/GlassSlider";
import {
  behindFor,
  controlTrackFor,
  supportsGlassLens,
  usePrefersDark,
} from "./glass";

interface SliderProps {
  value: number;
  onChange: (value: number) => void;
  min: number;
  max: number;
  step?: number;
  disabled?: boolean;
  label: string;
  description: string;
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
  showValue?: boolean;
  formatValue?: (value: number) => string;
}

/** Board geometry: track 176 x 8, handle 36 x 18. */
const TRACK_WIDTH_PX = 176;
const TRACK_HEIGHT_PX = 8;
const HANDLE_WIDTH_PX = 36;
const HANDLE_HEIGHT_PX = 18;
const ACCENT_FILL = "#3D7EFF";

export const Slider: React.FC<SliderProps> = ({
  value,
  onChange,
  min,
  max,
  step = 0.01,
  disabled = false,
  label,
  description,
  descriptionMode = "tooltip",
  grouped = false,
  showValue = true,
  formatValue = (v) => v.toFixed(2),
}) => {
  const isDark = usePrefersDark();
  const ratio = max === min ? 0 : (value - min) / (max - min);

  return (
    <SettingContainer
      title={label}
      description={description}
      descriptionMode={descriptionMode}
      grouped={grouped}
      layout="horizontal"
      disabled={disabled}
    >
      <div className="flex items-center gap-2" data-testid="volume-track">
        {supportsGlassLens() ? (
          <GlassSlider
            value={value}
            onValueChange={onChange}
            min={min}
            max={max}
            step={step}
            disabled={disabled}
            width={TRACK_WIDTH_PX}
            height={TRACK_HEIGHT_PX}
            thumbWidth={HANDLE_WIDTH_PX}
            thumbHeight={HANDLE_HEIGHT_PX}
            scheme={isDark ? "dark" : "light"}
            trackColor={controlTrackFor(isDark)}
            activeColor={ACCENT_FILL}
            surface={behindFor("sliderHandle", isDark)}
            ariaLabel={label}
          />
        ) : (
          // No lens: the plain control, same geometry, same behaviour.
          <input
            type="range"
            min={min}
            max={max}
            step={step}
            value={value}
            disabled={disabled}
            aria-label={label}
            onChange={(e) => onChange(parseFloat(e.target.value))}
            className="slider-fallback focus-ring"
            style={
              {
                width: TRACK_WIDTH_PX,
                "--slider-ratio": ratio,
              } as React.CSSProperties
            }
          />
        )}
        {showValue && (
          <span
            data-testid="volume-value"
            className="text-[13px] font-semibold text-text w-[34px] text-end"
          >
            {formatValue(value)}
          </span>
        )}
      </div>
    </SettingContainer>
  );
};
