import React from "react";
import { useTranslation } from "react-i18next";
import { Cog, FlaskConical, History, Info, Sparkles, Cpu } from "lucide-react";
import DictusLogo from "./icons/DictusLogo";
import DictusWaveformIcon from "./icons/DictusWaveformIcon";
import { useSettings } from "../hooks/useSettings";
import {
  GeneralSettings,
  AdvancedSettings,
  HistorySettings,
  DebugSettings,
  AboutSettings,
  PostProcessingSettings,
  ModelsSettings,
} from "./settings";

export type SidebarSection = keyof typeof SECTIONS_CONFIG;

interface IconProps {
  width?: number | string;
  height?: number | string;
  size?: number | string;
  className?: string;
  [key: string]: any;
}

interface SectionConfig {
  labelKey: string;
  icon: React.ComponentType<IconProps>;
  component: React.ComponentType;
  enabled: (settings: any) => boolean;
}

export const SECTIONS_CONFIG = {
  general: {
    labelKey: "sidebar.general",
    icon: DictusWaveformIcon,
    component: GeneralSettings,
    enabled: () => true,
  },
  models: {
    labelKey: "sidebar.models",
    icon: Cpu,
    component: ModelsSettings,
    enabled: () => true,
  },
  advanced: {
    labelKey: "sidebar.advanced",
    icon: Cog,
    component: AdvancedSettings,
    enabled: () => true,
  },
  history: {
    labelKey: "sidebar.history",
    icon: History,
    component: HistorySettings,
    enabled: () => true,
  },
  postprocessing: {
    labelKey: "sidebar.postProcessing",
    icon: Sparkles,
    component: PostProcessingSettings,
    enabled: (settings) => settings?.post_process_enabled ?? false,
  },
  debug: {
    labelKey: "sidebar.debug",
    icon: FlaskConical,
    component: DebugSettings,
    enabled: (settings) => settings?.debug_mode ?? false,
  },
  about: {
    labelKey: "sidebar.about",
    icon: Info,
    component: AboutSettings,
    enabled: () => true,
  },
} as const satisfies Record<string, SectionConfig>;

/** 40 px item + 4 px gap. The capsule slides one stride per rendered item, so
 *  the two must stay in step with the `h-10` and `gap-1` below. */
const NAV_STRIDE_PX = 44;

interface SidebarProps {
  activeSection: SidebarSection;
  onSectionChange: (section: SidebarSection) => void;
}

export const Sidebar: React.FC<SidebarProps> = ({
  activeSection,
  onSectionChange,
}) => {
  const { t } = useTranslation();
  const { settings } = useSettings();

  const availableSections = Object.entries(SECTIONS_CONFIG)
    .filter(([_, config]) => config.enabled(settings))
    .map(([id, config]) => ({ id: id as SidebarSection, ...config }));

  // Smart Modes and Debug appear conditionally, so the capsule indexes the
  // rendered list rather than the static section order.
  const activeIndex = availableSections.findIndex(
    (section) => section.id === activeSection,
  );

  return (
    <div className="flex flex-col w-40 h-full shrink-0 bg-sidebar glass-blur border-e border-hairline items-center px-2">
      <DictusLogo width={120} className="m-4" />
      <div className="relative flex flex-col w-full items-center gap-1 pt-2 border-t border-hairline">
        {activeIndex >= 0 && (
          <div
            aria-hidden="true"
            className="nav-capsule absolute top-2 start-0 w-36 h-10 rounded-[20px] pointer-events-none"
            style={{
              transform: `translateY(${activeIndex * NAV_STRIDE_PX}px)`,
            }}
          />
        )}
        {availableSections.map((section) => {
          const Icon = section.icon;
          const isActive = activeSection === section.id;

          return (
            <button
              key={section.id}
              type="button"
              aria-current={isActive ? "page" : undefined}
              className={`focus-ring relative flex gap-2 items-center p-2 h-10 w-full rounded-[20px] cursor-pointer text-start transition-colors ${
                isActive
                  ? "text-accent font-semibold"
                  : "text-[var(--nav-label)] font-medium hover:text-text"
              }`}
              onClick={() => onSectionChange(section.id)}
            >
              <Icon width={24} height={24} className="shrink-0" />
              <p className="text-sm truncate" title={t(section.labelKey)}>
                {t(section.labelKey)}
              </p>
            </button>
          );
        })}
      </div>
    </div>
  );
};
