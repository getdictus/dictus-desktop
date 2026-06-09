import React, { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import type { TFunction } from "i18next";
import { X } from "lucide-react";
import { listen } from "@tauri-apps/api/event";
import { commands } from "@/bindings";
import { localizeSmartModeName } from "./SmartModeCard";
import {
  formatKeyCombination,
  getKeyName,
  normalizeKey,
} from "@/lib/utils/keyboard";
import { useOsType } from "@/hooks/useOsType";
import { useSettings } from "@/hooks/useSettings";

interface HandyKeysEvent {
  modifiers: string[];
  key: string | null;
  is_key_down: boolean;
  hotkey_string: string;
}

interface SmartModeShortcutChipProps {
  modeId: string;
  currentBinding: string | null;
  disabled?: boolean;
  onBound: () => void;
  onConflictChange?: (message: string | null) => void;
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

/** Re-registers all bound shortcuts. Best-effort — errors are suppressed. */
const resumeAll = () => {
  commands.resumeAllShortcuts().catch(() => {});
};

/**
 * Maps a structured backend binding error payload to a localized string.
 *
 * Backend payloads (from 13-25):
 *   SHORTCUT_CONFLICT|exact_duplicate|<id>|<name>
 *   SHORTCUT_CONFLICT|base_overlap|<id>|<name>|<base>
 *
 * <id> is the conflicting binding id. For smart_mode_* ids we resolve the
 * LOCALIZED mode label (seeded-and-pristine -> t(key), else stored name).
 * Core ids (transcribe/cancel) fall back to the stored <name> verbatim
 * (core binding names are not localized — known limitation).
 *
 * Unknown or legacy errors fall back to the generic conflict message.
 */
function localizeBindingError(
  raw: string | null | undefined,
  t: TFunction,
): string {
  if (raw && raw.startsWith("SHORTCUT_CONFLICT|")) {
    const parts = raw.split("|");
    const code = parts[1] ?? "";
    const id = parts[2] ?? "";
    const name = parts[3] ?? "";
    const base = parts[4] ?? "";
    const displayName = id.startsWith("smart_mode_")
      ? localizeSmartModeName(id.slice("smart_mode_".length), name, t)
      : name;
    if (code === "base_overlap") {
      return t("smartModes.card.shortcutConflictBase", {
        name: displayName,
        base,
      });
    }
    return t("smartModes.card.shortcutConflict", { name: displayName });
  }
  // Unknown / legacy error → fall back to the generic conflict message
  return raw ?? t("smartModes.card.shortcutConflict", { name: "" });
}

export const SmartModeShortcutChip: React.FC<SmartModeShortcutChipProps> = ({
  modeId,
  currentBinding,
  disabled = false,
  onBound,
  onConflictChange,
}) => {
  const { t } = useTranslation();
  const osType = useOsType();
  const { getSetting } = useSettings();

  // Determine which capture path to use based on keyboard implementation
  const useHandyKeys = getSetting("keyboard_implementation") === "handy_keys";

  const [isRecording, setIsRecording] = useState(false);
  // heldModifiers tracks currently pressed modifier keys for live preview
  const heldModifiersRef = useRef<Set<string>>(new Set());
  // Whether a non-modifier (main) key was pressed during this recording. Used
  // to decide between a combo (commit on main keydown) and a modifier-only
  // binding like "option" (commit on modifier release).
  const mainKeyPressedRef = useRef(false);
  // Guards against double-commit (e.g. a keyup firing right after a keydown
  // commit before the listeners are torn down).
  const committedRef = useRef(false);
  // Tracks the current handy-keys hotkey_string (ref avoids stale closure in
  // the async listen callback — mirrors HandyKeysShortcutInput.tsx:41).
  const currentKeysRef = useRef<string>("");
  const [recordedKeys, setRecordedKeys] = useState<string[]>([]);
  const [conflict, setConflict] = useState<string | null>(null);

  const chipRef = useRef<HTMLDivElement>(null);

  const sortModifiers = (mods: string[]) =>
    [...mods].sort(
      (a, b) =>
        MODIFIERS.indexOf(a.toLowerCase()) - MODIFIERS.indexOf(b.toLowerCase()),
    );

  // Reset conflict when binding changes
  useEffect(() => {
    setConflict(null);
  }, [currentBinding]);

  // Report conflict state to parent (hoisted error row)
  useEffect(() => {
    onConflictChange?.(conflict);
  }, [conflict, onConflictChange]);

  const commitCombo = useCallback(
    async (combo: string) => {
      if (committedRef.current) return;
      committedRef.current = true;
      setIsRecording(false);
      heldModifiersRef.current.clear();
      setRecordedKeys([]);

      const res = await commands.setSmartModeBinding(modeId, combo);
      if (res.status === "ok") {
        const response = res.data;
        if (response.success) {
          setConflict(null);
          resumeAll();
          onBound();
        } else {
          setConflict(localizeBindingError(response.error, t));
          resumeAll();
        }
      }
    },
    [modeId, onBound, t],
  );

  // ── Handy-keys capture path (backend stream, preserves left/right modifiers) ──
  // Mirrors HandyKeysShortcutInput.tsx:76-153. Active only when
  // keyboard_implementation === "handy_keys".
  useEffect(() => {
    if (!isRecording || !useHandyKeys) return;

    let cleanup = false;

    const setupListener = async () => {
      const unlisten = await listen<HandyKeysEvent>(
        "handy-keys-event",
        async (event) => {
          if (cleanup) return;
          const { hotkey_string, is_key_down } = event.payload;

          if (is_key_down && hotkey_string) {
            // Live preview — store side-distinct hotkey_string in both ref and
            // state so the chip renders it. Do NOT run through getKeyName.
            currentKeysRef.current = hotkey_string;
            setRecordedKeys(hotkey_string.split("+"));
          } else if (!is_key_down && currentKeysRef.current) {
            // Key released — commit with the raw side-distinct string so that
            // find_conflicting_binding can match "command_right" exactly.
            const keysToCommit = currentKeysRef.current;
            unlisten();
            await commands.stopHandyKeysRecording().catch(() => {});
            await commitCombo(keysToCommit);
          }
        },
      );

      if (cleanup) {
        // Component unmounted between async start and listener attach
        unlisten();
        await commands.stopHandyKeysRecording().catch(() => {});
        return;
      }
    };

    setupListener();

    return () => {
      cleanup = true;
      commands.stopHandyKeysRecording().catch(() => {});
      resumeAll();
    };
  }, [isRecording, useHandyKeys, commitCombo]);

  // ── Webview keydown/keyup capture path (fallback for "tauri" implementation) ──
  // Guarded to NOT run when keyboard_implementation === "handy_keys".
  useEffect(() => {
    if (!isRecording || useHandyKeys) return;

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
        setRecordedKeys(sortModifiers([...heldModifiersRef.current]));
      } else {
        // Main (non-modifier) key pressed — this is the COMMIT trigger
        mainKeyPressedRef.current = true;
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

      if (!isModifier(key)) return;

      // Modifier-only shortcut (e.g. "option" alone): if a modifier is released
      // and no main key was pressed during this recording, commit the held
      // modifier(s) as the binding. Captured BEFORE removing the released key so
      // it's included in the combo.
      if (!mainKeyPressedRef.current && heldModifiersRef.current.size > 0) {
        const combo = sortModifiers([...heldModifiersRef.current]).join("+");
        void commitCombo(combo);
        return;
      }

      // Otherwise just keep the live preview accurate.
      heldModifiersRef.current.delete(key);
      setRecordedKeys(sortModifiers([...heldModifiersRef.current]));
    };

    const handleClickOutside = (e: MouseEvent) => {
      if (cleanup) return;
      if (chipRef.current && !chipRef.current.contains(e.target as Node)) {
        setIsRecording(false);
        heldModifiersRef.current.clear();
        setRecordedKeys([]);
        resumeAll();
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
      // Always resume all shortcuts when recording stops or the component
      // unmounts mid-record; resumeAllShortcuts is idempotent so it is safe
      // to call even if a prior exit path already resumed.
      resumeAll();
    };
  }, [isRecording, useHandyKeys, modeId, osType, commitCombo]);

  const handleClick = () => {
    if (disabled || isRecording) return;
    setConflict(null);
    heldModifiersRef.current.clear();
    mainKeyPressedRef.current = false;
    committedRef.current = false;
    currentKeysRef.current = "";
    setRecordedKeys([]);
    // Suspend ALL global shortcuts so any already-bound combo is captured by
    // the keydown listener (webview path) or the backend stream (handy-keys
    // path) instead of firing that mode's action.
    commands.suspendAllShortcuts().catch(() => {});
    if (useHandyKeys) {
      // Start backend recording before setting isRecording so the useEffect
      // listener is set up with an already-active backend stream.
      commands.startHandyKeysRecording(`smart_mode_${modeId}`).catch(() => {});
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

  return <>{renderChip()}</>;
};
