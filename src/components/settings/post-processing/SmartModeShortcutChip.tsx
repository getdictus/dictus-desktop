import React, { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { X } from "lucide-react";
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

const isModifier = (k: string) => MODIFIERS.includes(k.toLowerCase());

export const SmartModeShortcutChip: React.FC<SmartModeShortcutChipProps> = ({
  modeId,
  currentBinding,
  disabled = false,
  onBound,
}) => {
  const { t } = useTranslation();
  const osType = useOsType();

  const [isRecording, setIsRecording] = useState(false);
  // heldModifiers tracks currently pressed modifier keys for live preview
  const heldModifiersRef = useRef<Set<string>>(new Set());
  const [recordedKeys, setRecordedKeys] = useState<string[]>([]);
  const [conflict, setConflict] = useState<string | null>(null);

  const chipRef = useRef<HTMLDivElement>(null);

  // Reset conflict when binding changes
  useEffect(() => {
    setConflict(null);
  }, [currentBinding]);

  const commitCombo = useCallback(
    async (combo: string) => {
      setIsRecording(false);
      heldModifiersRef.current.clear();
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
    },
    [modeId, currentBinding, onBound, t],
  );

  useEffect(() => {
    if (!isRecording) return;

    let cleanup = false;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (cleanup) return;
      if (e.repeat) return;
      e.preventDefault();

      const rawKey = getKeyName(e, osType);
      const key = normalizeKey(rawKey);

      if (isModifier(key)) {
        // Track held modifier; update live preview
        heldModifiersRef.current.add(key);
        const sortedMods = [...heldModifiersRef.current].sort((a, b) => {
          const aIdx = MODIFIERS.indexOf(a.toLowerCase());
          const bIdx = MODIFIERS.indexOf(b.toLowerCase());
          return aIdx - bIdx;
        });
        setRecordedKeys(sortedMods);
      } else {
        // Main (non-modifier) key pressed — this is the COMMIT trigger
        const sortedKeys = [...heldModifiersRef.current, key].sort((a, b) => {
          const aIsMod = isModifier(a);
          const bIsMod = isModifier(b);
          if (aIsMod && !bIsMod) return -1;
          if (!aIsMod && bIsMod) return 1;
          return 0;
        });
        const combo = sortedKeys.join("+");
        // Commit immediately on keydown (fixes macOS Cmd+keyup swallow issue)
        void commitCombo(combo);
      }
    };

    const handleKeyUp = (e: KeyboardEvent) => {
      if (cleanup) return;
      e.preventDefault();

      const rawKey = getKeyName(e, osType);
      const key = normalizeKey(rawKey);

      // Only track modifier releases to keep live preview accurate
      if (isModifier(key)) {
        heldModifiersRef.current.delete(key);
        const sortedMods = [...heldModifiersRef.current].sort((a, b) => {
          const aIdx = MODIFIERS.indexOf(a.toLowerCase());
          const bIdx = MODIFIERS.indexOf(b.toLowerCase());
          return aIdx - bIdx;
        });
        setRecordedKeys(sortedMods);
      }
      // No commit on keyup — the commit happens on main-key keydown
    };

    const handleClickOutside = (e: MouseEvent) => {
      if (cleanup) return;
      if (chipRef.current && !chipRef.current.contains(e.target as Node)) {
        setIsRecording(false);
        heldModifiersRef.current.clear();
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
  }, [isRecording, modeId, currentBinding, osType, commitCombo]);

  const handleClick = () => {
    if (disabled || isRecording) return;
    setConflict(null);
    heldModifiersRef.current.clear();
    setRecordedKeys([]);
    // Best-effort suspend — no-op if entry doesn't exist yet
    if (currentBinding) {
      commands.suspendBinding("smart_mode_" + modeId).catch(() => {});
    }
    setIsRecording(true);
  };

  const handleClear = async (e: React.MouseEvent) => {
    e.stopPropagation();
    await commands.clearSmartModeBinding(modeId);
    setConflict(null);
    onBound();
  };

  const formatCurrentKeys = (): string => {
    if (recordedKeys.length === 0) {
      return t("smartModes.shortcut.recording");
    }
    return formatKeyCombination(recordedKeys.join("+"), osType);
  };

  const renderChip = () => {
    if (isRecording) {
      return (
        <div
          ref={chipRef}
          dir="ltr"
          aria-label={t("smartModes.shortcut.recording")}
          className="px-2 py-1 text-sm font-medium border border-logo-primary bg-logo-primary/30 rounded-md"
        >
          {formatCurrentKeys()}
        </div>
      );
    }

    if (currentBinding) {
      return (
        <span className="inline-flex items-center gap-1">
          <div
            ref={chipRef}
            dir="ltr"
            aria-label={currentBinding}
            aria-disabled={disabled ? "true" : undefined}
            onClick={disabled ? undefined : handleClick}
            className={`px-2 py-1 text-sm font-medium bg-mid-gray/10 border border-mid-gray/80 rounded-md ${disabled ? "opacity-60" : "cursor-pointer hover:bg-logo-primary/10 hover:border-logo-primary"}`}
          >
            {formatKeyCombination(currentBinding, osType)}
          </div>
          {!disabled && (
            <button
              type="button"
              className="p-0.5 rounded focus:outline-none"
              onClick={handleClear}
              aria-label={t("smartModes.shortcut.clearAriaLabel", {
                combo: formatKeyCombination(currentBinding, osType),
              })}
            >
              <X className="w-4 h-4 text-mid-gray/50 hover:text-red-400" />
            </button>
          )}
        </span>
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
