import React from "react";
import { SettingContainer } from "./SettingContainer";
import {
  GlassLens,
  behindFor,
  opticsFor,
  usePrefersDark,
  SLIDER_BASE,
  SLIDER_LIGHT,
  SLIDER_DARK,
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

const TRACK_WIDTH_PX = 176;
const HANDLE_WIDTH_PX = 36;
const HANDLE_HEIGHT_PX = 18;

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
  // At rest the handle is the lens tinted solid white; during a drag the tint
  // drops away and the refraction shows.
  const [isDragging, setIsDragging] = React.useState(false);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange(parseFloat(e.target.value));
  };

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
      <div className="flex items-center gap-2">
        <div
          className={`slider-track relative h-[18px] flex items-center ${
            disabled ? "opacity-50" : ""
          }`}
          style={{ width: TRACK_WIDTH_PX }}
        >
          {/* Rail and fill are painted here; the native input stays on top so
              pointer and keyboard behaviour are untouched. */}
          <div
            className="absolute inset-x-0 h-2 rounded"
            style={{ background: "var(--slider-rail)" }}
          />
          <div
            className="absolute start-0 h-2 rounded"
            style={{ background: "#3D7EFF", width: `${ratio * 100}%` }}
          />
          <GlassLens
            className="slider-handle absolute pointer-events-none"
            style={
              {
                "--slider-ratio": ratio,
              } as React.CSSProperties
            }
            behind={behindFor("sliderHandle", isDark)}
            width={HANDLE_WIDTH_PX}
            height={HANDLE_HEIGHT_PX}
            radius={9}
            optics={opticsFor(SLIDER_BASE, SLIDER_LIGHT, SLIDER_DARK, isDark)}
            tint={{ color: "white", opacity: isDragging ? 0 : 1 }}
          />
          <input
            type="range"
            min={min}
            max={max}
            step={step}
            value={value}
            onChange={handleChange}
            disabled={disabled}
            onPointerDown={() => setIsDragging(true)}
            onPointerUp={() => setIsDragging(false)}
            onPointerCancel={() => setIsDragging(false)}
            onKeyDown={() => setIsDragging(true)}
            onKeyUp={() => setIsDragging(false)}
            onBlur={() => setIsDragging(false)}
            className="slider-input focus-ring absolute inset-0 w-full appearance-none bg-transparent rounded disabled:cursor-not-allowed"
          />
        </div>
        {showValue && (
          <span className="text-[13px] font-semibold text-text w-[34px] text-end">
            {formatValue(value)}
          </span>
        )}
      </div>
    </SettingContainer>
  );
};
