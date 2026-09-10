import React, { useCallback, useLayoutEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Cog, FlaskConical, History, Info, Sparkles, Cpu } from "lucide-react";
import DictusLogo from "./icons/DictusLogo";
import DictusWaveformIcon from "./icons/DictusWaveformIcon";
import { useSettings } from "../hooks/useSettings";
import {
  GlassLens,
  behindFor,
  opticsFor,
  usePrefersDark,
  NAV_CAPSULE_LENS,
  NAV_CAPSULE_LIGHT,
  NAV_CAPSULE_DARK,
} from "./ui/glass";
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

/* Fallbacks until the first measurement lands. */
const NAV_ITEM_HEIGHT_PX = 40;
const NAV_ITEM_WIDTH_PX = 168;

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
  const isDark = usePrefersDark();
  const itemRefs = useRef<(HTMLButtonElement | null)[]>([]);
  const [capsule, setCapsule] = useState({
    top: 0,
    height: NAV_ITEM_HEIGHT_PX,
    width: NAV_ITEM_WIDTH_PX,
  });

  const availableSections = Object.entries(SECTIONS_CONFIG)
    .filter(([_, config]) => config.enabled(settings))
    .map(([id, config]) => ({ id: id as SidebarSection, ...config }));

  // Smart Modes and Debug appear conditionally, so the capsule indexes the
  // rendered list rather than the static section order.
  const activeIndex = availableSections.findIndex(
    (section) => section.id === activeSection,
  );

  // The board's 44 px stride assumes every item is exactly 40 px tall. A flex
  // item will not shrink below its content, so one pixel of overflow in any
  // item makes the error accumulate down the list. Measuring the rendered item
  // removes the assumption entirely.
  const measure = useCallback(() => {
    const item = itemRefs.current[activeIndex];
    if (!item) return;
    // Whole pixels on purpose: the lens rounds its filter region to integer
    // px, so a fractional box lands the rim about a pixel off the plate.
    // offset* is already rounded; Math.round guards a future switch to
    // getBoundingClientRect.
    setCapsule({
      top: Math.round(item.offsetTop),
      height: Math.round(item.offsetHeight),
      width: Math.round(item.offsetWidth),
    });
  }, [activeIndex]);

  useLayoutEffect(() => {
    measure();
    if (typeof ResizeObserver === "undefined") return;
    const observer = new ResizeObserver(measure);
    itemRefs.current.forEach((item) => item && observer.observe(item));
    return () => observer.disconnect();
  }, [measure, availableSections.length]);

  return (
    <div className="flex flex-col w-[184px] h-full shrink-0 bg-sidebar glass-blur border-e border-hairline items-center px-2">
      <DictusLogo width={120} className="m-4" />
      <div className="relative flex flex-col w-full items-center gap-1 pt-2 border-t border-hairline">
        {activeIndex >= 0 && (
          <div
            aria-hidden="true"
            data-testid="nav-capsule"
            className="nav-capsule-slot absolute inset-x-0 top-0 pointer-events-none"
            style={{
              height: capsule.height,
              transform: `translateY(${capsule.top}px)`,
            }}
          >
            {/* The plate carries the rim and the drop shadow; the lens sits on
                top of it and bends what is behind the sidebar. */}
            <div className="nav-capsule absolute inset-0 rounded-[20px]" />
            <GlassLens
              className="absolute inset-0"
              behind={behindFor("navCapsule", isDark)}
              width={capsule.width}
              height={capsule.height}
              radius={capsule.height / 2}
              optics={opticsFor(
                NAV_CAPSULE_LENS,
                NAV_CAPSULE_LIGHT,
                NAV_CAPSULE_DARK,
                isDark,
              )}
              fallbackStyle={{ background: "transparent" }}
            />
          </div>
        )}
        {availableSections.map((section, index) => {
          const Icon = section.icon;
          const isActive = activeSection === section.id;

          return (
            <button
              key={section.id}
              ref={(node) => {
                itemRefs.current[index] = node;
              }}
              type="button"
              data-testid={`nav-item-${section.id}`}
              aria-current={isActive ? "page" : undefined}
              // min-h-0 lets the fixed 40 px win: a flex item will not shrink
              // below its content otherwise, and one pixel of overflow in a
              // label makes every item below it sit lower.
              className={`focus-ring relative flex gap-2 items-center p-2 h-10 min-h-0 w-full rounded-[20px] cursor-pointer text-start transition-colors ${
                isActive
                  ? "text-accent font-semibold"
                  : "text-[var(--nav-label)] font-medium hover:text-text"
              }`}
              onClick={() => onSectionChange(section.id)}
            >
              <Icon width={24} height={24} className="shrink-0" />
              <p className="text-[13px] truncate" title={t(section.labelKey)}>
                {t(section.labelKey)}
              </p>
            </button>
          );
        })}
      </div>
    </div>
  );
};
