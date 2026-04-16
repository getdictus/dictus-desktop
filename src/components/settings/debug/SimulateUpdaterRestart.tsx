import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { commands } from "@/bindings";
import { SettingContainer } from "../../ui/SettingContainer";
import { Button } from "../../ui/Button";

interface SimulateUpdaterRestartProps {
  descriptionMode?: "tooltip" | "inline";
  grouped?: boolean;
}

/**
 * SHUT-03 manual validation trigger. Invokes the backend
 * `simulate_updater_restart` Tauri command, which calls
 * `AppHandle::restart()` — the same entrypoint
 * `tauri-plugin-updater` uses after applying an update.
 *
 * Rendered only when `settings.debug_mode === true` (gated by
 * `DebugSettings.tsx`). The Rust symbol is built unconditionally
 * so this works in production builds too.
 */
export const SimulateUpdaterRestart: React.FC<SimulateUpdaterRestartProps> = ({
  descriptionMode = "tooltip",
  grouped = false,
}) => {
  const { t } = useTranslation();
  const [pending, setPending] = useState(false);

  const handleClick = async () => {
    if (pending) return;
    setPending(true);
    try {
      await commands.simulateUpdaterRestart();
    } catch (err) {
      // restart() schedules an exit; we usually never reach here.
      console.error("simulateUpdaterRestart failed:", err);
      setPending(false);
    }
  };

  return (
    <SettingContainer
      title={t("settings.debug.simulateUpdaterRestart.title")}
      description={t("settings.debug.simulateUpdaterRestart.description")}
      descriptionMode={descriptionMode}
      grouped={grouped}
      layout="horizontal"
    >
      <Button
        variant="secondary"
        size="sm"
        onClick={handleClick}
        disabled={pending}
      >
        {t("settings.debug.simulateUpdaterRestart.button")}
      </Button>
    </SettingContainer>
  );
};
