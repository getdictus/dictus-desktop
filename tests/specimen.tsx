import React from "react";
import ReactDOM from "react-dom/client";
import i18n from "../src/i18n";
import { useSettingsStore } from "../src/stores/settingsStore";

// Render every conditional nav section, and honour ?lang= so the truncation
// test can load the locale with the longest labels.
useSettingsStore.setState({
  settings: {
    post_process_enabled: true,
    debug_mode: true,
  } as never,
  isLoading: false,
});
const forcedLanguage = new URLSearchParams(location.search).get("lang");
if (forcedLanguage) void i18n.changeLanguage(forcedLanguage);
import { Button } from "../src/components/ui/Button";
import { Dropdown } from "../src/components/ui/Dropdown";
import { Input } from "../src/components/ui/Input";
import { SettingsGroup } from "../src/components/ui/SettingsGroup";
import { SettingContainer } from "../src/components/ui/SettingContainer";
import { Slider } from "../src/components/ui/Slider";
import { Surface } from "../src/components/ui/Surface";
import { ToggleSwitch } from "../src/components/ui/ToggleSwitch";
import { Sidebar, type SidebarSection } from "../src/components/Sidebar";
import "../src/App.css";
import "../src/overlay/RecordingOverlay.css";

/**
 * The shared-component comparison screen from the issue's validation plan.
 *
 * It renders the primitives that carry the new material, with no Tauri
 * dependency, so Playwright can screenshot them in both themes. Screens that
 * need the settings store are covered by hand instead — see the PR body.
 */
const BUTTON_VARIANTS = [
  "primary",
  "primary-soft",
  "secondary",
  "danger",
  "danger-ghost",
  "ghost",
] as const;

const Specimen: React.FC = () => {
  const [checked, setChecked] = React.useState(true);
  const [volume, setVolume] = React.useState(0.65);
  const [choice, setChoice] = React.useState<string | null>("balanced");
  const [section, setSection] = React.useState<SidebarSection>("general");

  return (
    <div className="min-h-screen flex gap-4 p-0">
      <div className="h-screen sticky top-0" data-testid="sidebar-host">
        <Sidebar activeSection={section} onSectionChange={setSection} />
      </div>
      <div className="w-full max-w-[520px] flex flex-col gap-4 p-4">
        <SettingsGroup title="Recording" description="Rows, dividers, controls">
          <ToggleSwitch
            grouped
            testId="toggle-push-to-talk"
            checked={checked}
            onChange={setChecked}
            label="Push to talk"
            description="Hold the shortcut while you speak."
          />
          <ToggleSwitch
            grouped
            checked={false}
            onChange={() => {}}
            label="Mute while recording"
            description="Silences other audio during a take."
          />
          <Slider
            grouped
            value={volume}
            onChange={setVolume}
            min={0}
            max={1}
            label="Feedback volume"
            description="Volume of the start and stop sounds."
            formatValue={(v) => `${Math.round(v * 100)}%`}
          />
          <SettingContainer
            grouped
            title="Model"
            description="Which model transcribes your audio."
          >
            <Dropdown
              options={[
                { value: "balanced", label: "Whisper Turbo" },
                { value: "accurate", label: "Whisper Large" },
              ]}
              selectedValue={choice}
              onSelect={setChoice}
            />
          </SettingContainer>
          <SettingContainer
            grouped
            title="Custom words"
            description="Words the model should prefer."
          >
            <Input defaultValue="Dictus" />
          </SettingContainer>
        </SettingsGroup>

        <div className="flex flex-wrap gap-2">
          {BUTTON_VARIANTS.map((variant) => (
            <Button key={variant} variant={variant}>
              {variant}
            </Button>
          ))}
        </div>

        {/* The overlay pill's material, over a strip dark enough to show the
            translucency. The bars are static here: the real waveform is driven
            by requestAnimationFrame and is not part of this change. */}
        <div
          className="flex items-center gap-2 p-4 rounded-2xl"
          style={{ background: "linear-gradient(90deg,#2b3550,#5b6b8f)" }}
        >
          <div className="recording-overlay fade-in">
            <div className="overlay-middle">
              <div className="bars-container">
                {Array.from({ length: 30 }, (_, i) => {
                  const centre = 14.5;
                  const distance = Math.abs(i - centre) / centre;
                  const height = Math.max(2, (1 - distance * 0.9) * 52);
                  const colour =
                    distance < 0.4
                      ? "#6BA3FF"
                      : `rgba(255,255,255,${((1 - distance) * 0.9 + 0.15).toFixed(2)})`;
                  return (
                    <div
                      key={i}
                      className="bar"
                      style={{ height, backgroundColor: colour }}
                    />
                  );
                })}
              </div>
            </div>
          </div>
          <div className="cancel-pill" />
        </div>

        <div
          className="flex items-center gap-2 p-4 rounded-2xl"
          style={{ background: "linear-gradient(90deg,#2b3550,#5b6b8f)" }}
        >
          <div className="recording-overlay fade-in overlay-state-error">
            <div className="overlay-middle">
              <div className="overlay-error">
                <svg
                  className="overlay-error-icon"
                  width="20"
                  height="20"
                  viewBox="0 0 24 24"
                  fill="none"
                  aria-hidden="true"
                >
                  <path
                    d="M12 9v4m0 4h.01M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  />
                </svg>
                <span className="overlay-error-text">
                  Smart Mode could not run: no model available.
                </span>
              </div>
            </div>
          </div>
        </div>

        <div className="flex flex-wrap gap-3">
          <Surface variant="raised" className="px-3 py-2 text-sm">
            raised
          </Surface>
          <Surface variant="card" className="px-3 py-2 text-sm">
            card
          </Surface>
          <Surface variant="popover" className="px-3 py-2 text-sm">
            popover
          </Surface>
          <Surface
            variant="overlay"
            className="px-4 py-2 text-sm text-white flex items-center"
          >
            overlay
          </Surface>
        </div>
      </div>
    </div>
  );
};

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Specimen />
  </React.StrictMode>,
);
