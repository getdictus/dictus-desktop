import React, { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { commands } from "@/bindings";
import {
  formatKeyCombination,
  getKeyName,
  normalizeKey,
} from "@/lib/utils/keyboard";
import { useOsType } from "@/hooks/useOsType";

interface SmartModeShortcutChipProps {
  modeId: string;
  currentBinding: string | null;
  disabled?: boolean;
  onBound: () => void;
}

const MODIFIERS = [
  "ctrl",
  "control",
  "shift",
  "alt",
  "option",
  "meta",
  "command",
  "cmd",
  "super",
  "win",
  "windows",
];

export const SmartModeShortcutChip: React.FC<SmartModeShortcutChipProps> = ({
  modeId,
  currentBinding,
  disabled = false,
  onBound,
}) => {
  const { t } = useTranslation();
  const osType = useOsType();

  const [isRecording, setIsRecording] = useState(false);
  const [keyPressed, setKeyPressed] = useState<string[]>([]);
  const [recordedKeys, setRecordedKeys] = useState<string[]>([]);
  const [conflict, setConflict] = useState<string | null>(null);

  const chipRef = useRef<HTMLDivElement>(null);

  // Reset conflict when binding changes
  useEffect(() => {
    setConflict(null);
  }, [currentBinding]);

  useEffect(() => {
    if (!isRecording) return;

    let cleanup = false;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (cleanup) return;
      if (e.repeat) return;
      e.preventDefault();

      const rawKey = getKeyName(e, osType);
      const key = normalizeKey(rawKey);

      setKeyPressed((prev) => (prev.includes(key) ? prev : [...prev, key]));
      setRecordedKeys((prev) => (prev.includes(key) ? prev : [...prev, key]));
    };

    const handleKeyUp = async (e: KeyboardEvent) => {
      if (cleanup) return;
      e.preventDefault();

      const rawKey = getKeyName(e, osType);
      const key = normalizeKey(rawKey);

      const updatedPressed = keyPressed.filter((k) => k !== key);
      setKeyPressed(updatedPressed);

      if (updatedPressed.length === 0 && recordedKeys.length > 0) {
        const sortedKeys = [...recordedKeys].sort((a, b) => {
          const aIsMod = MODIFIERS.includes(a.toLowerCase());
          const bIsMod = MODIFIERS.includes(b.toLowerCase());
          if (aIsMod && !bIsMod) return -1;
          if (!aIsMod && bIsMod) return 1;
          return 0;
        });
        const combo = sortedKeys.join("+");

        setIsRecording(false);
        setKeyPressed([]);
        setRecordedKeys([]);

        const res = await commands.setSmartModeBinding(modeId, combo);
        if (res.status === "ok") {
          const response = res.data;
          if (response.success) {
            setConflict(null);
            onBound();
          } else {
            const errorMsg =
              response.error ??
              t("smartModes.card.shortcutConflict", { name: "" });
            setConflict(errorMsg);
            // Restore the previous binding if mode was bound before
            if (currentBinding) {
              commands
                .resumeBinding("smart_mode_" + modeId)
                .catch(() => {});
            }
          }
        }
      }
    };

    const handleClickOutside = (e: MouseEvent) => {
      if (cleanup) return;
      if (chipRef.current && !chipRef.current.contains(e.target as Node)) {
        setIsRecording(false);
        setKeyPressed([]);
        setRecordedKeys([]);
        // Resume binding if mode was bound before
        if (currentBinding) {
          commands.resumeBinding("smart_mode_" + modeId).catch(() => {});
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("keyup", handleKeyUp);
    window.addEventListener("click", handleClickOutside);

    return () => {
      cleanup = true;
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("keyup", handleKeyUp);
      window.removeEventListener("click", handleClickOutside);
    };
  }, [isRecording, keyPressed, recordedKeys, modeId, currentBinding, osType, t, onBound]);

  const handleClick = () => {
    if (disabled || isRecording) return;
    setConflict(null);
    setKeyPressed([]);
    setRecordedKeys([]);
    // Best-effort suspend — no-op if entry doesn't exist yet
    if (currentBinding) {
      commands.suspendBinding("smart_mode_" + modeId).catch(() => {});
    }
    setIsRecording(true);
  };

  const formatCurrentKeys = (): string => {
    if (recordedKeys.length === 0) {
      return t("settings.general.shortcut.pressKeys");
    }
    return formatKeyCombination(recordedKeys.join("+"), osType);
  };

  const renderChip = () => {
    if (isRecording) {
      return (
        <div
          ref={chipRef}
          dir="ltr"
          aria-label={t("settings.general.shortcut.pressKeys")}
          className="px-2 py-1 text-sm font-medium border border-logo-primary bg-logo-primary/30 rounded-md"
        >
          {formatCurrentKeys()}
        </div>
      );
    }

    if (currentBinding) {
      return (
        <div
          ref={chipRef}
          dir="ltr"
          aria-label={currentBinding}
          aria-disabled={disabled ? "true" : undefined}
          onClick={disabled ? undefined : handleClick}
          className={`px-2 py-1 text-sm font-medium bg-mid-gray/10 border border-mid-gray/80 rounded-md ${disabled ? "opacity-60" : "cursor-pointer hover:border-logo-primary"}`}
        >
          {formatKeyCombination(currentBinding, osType)}
        </div>
      );
    }

    return (
      <div
        ref={chipRef}
        dir="ltr"
        aria-label={t("smartModes.card.addShortcut")}
        aria-disabled={disabled ? "true" : undefined}
        onClick={disabled ? undefined : handleClick}
        className={`px-2 py-1 text-xs font-medium bg-mid-gray/5 border border-mid-gray/30 text-mid-gray/60 rounded-md ${disabled ? "opacity-60" : "cursor-pointer"}`}
      >
        {t("smartModes.card.addShortcut")}
      </div>
    );
  };

  return (
    <div>
      {renderChip()}
      {conflict && (
        <p role="alert" dir="auto" className="text-xs text-red-400 mt-1">
          {conflict}
        </p>
      )}
    </div>
  );
};
