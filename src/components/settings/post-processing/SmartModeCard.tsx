import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import type { TFunction } from "i18next";
import { Pencil, Trash2 } from "lucide-react";
import { ask } from "@tauri-apps/plugin-dialog";
import type { SmartMode, SmartModeKind, TargetLanguage } from "@/bindings";
import { commands } from "@/bindings";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Textarea } from "@/components/ui/Textarea";
import { useSettings } from "@/hooks/useSettings";
import { SmartModeShortcutChip } from "./SmartModeShortcutChip";

// Seeded mode ID -> i18n key map; verified against settings.rs default_smart_modes
export const SEEDED_MODE_ID_TO_I18N_KEY: Record<string, string> = {
  mode_clean_up: "smartModes.defaultModes.cleanUp",
  mode_make_formal: "smartModes.defaultModes.makeFormal",
  mode_make_casual: "smartModes.defaultModes.makeCasual",
  mode_email: "smartModes.defaultModes.writeAsEmail",
  mode_bullet_points: "smartModes.defaultModes.bulletPoints",
  mode_summarize: "smartModes.defaultModes.summarize",
  mode_translate_en: "smartModes.defaultModes.translateToEnglish",
  mode_translate_es: "smartModes.defaultModes.translateToSpanish",
  mode_translate_fr: "smartModes.defaultModes.translateToFrench",
  mode_translate_zh: "smartModes.defaultModes.translateToChinese",
};

// Seeded mode ID -> stored default English name (from settings.rs smart_mode_templates)
// Used to detect whether a seeded mode's name has been edited by the user.
// Exported so SmartModeTemplatePicker can perform name-based dedup.
export const SEEDED_MODE_DEFAULT_NAME: Record<string, string> = {
  mode_clean_up: "Clean Up",
  mode_make_formal: "Make Formal",
  mode_make_casual: "Make Casual",
  mode_email: "Write as Email",
  mode_bullet_points: "Bullet Points",
  mode_summarize: "Summarize",
  mode_translate_en: "Translate → English",
  mode_translate_es: "Translate → Spanish",
  mode_translate_fr: "Translate → French",
  mode_translate_zh: "Translate → Chinese",
};

/**
 * Localized display name for a Smart Mode.
 * Seeded-and-pristine (stored name still equals the English seed default) -> t(i18nKey).
 * Renamed seeded mode or user-created mode -> the literal stored name.
 */
export function localizeSmartModeName(
  modeId: string,
  storedName: string,
  t: TFunction,
): string {
  const i18nKey = SEEDED_MODE_ID_TO_I18N_KEY[modeId];
  const seedDefault = SEEDED_MODE_DEFAULT_NAME[modeId];
  if (i18nKey && seedDefault && storedName === seedDefault) return t(i18nKey);
  return storedName;
}

interface SmartModeCardProps {
  mode: SmartMode | null;
  kind: SmartModeKind;
  disabled?: boolean;
  availableLanguages: TargetLanguage[];
  genericEngineNote?: boolean;
  onChanged: () => void;
  onDiscardNew?: () => void;
}

export const SmartModeCard: React.FC<SmartModeCardProps> = ({
  mode,
  kind,
  disabled = false,
  availableLanguages,
  genericEngineNote = false,
  onChanged,
  onDiscardNew,
}) => {
  const { t } = useTranslation();
  const { getSetting } = useSettings();

  const isNew = mode === null;
  const [isEditing, setIsEditing] = useState(isNew);
  const [shortcutConflict, setShortcutConflict] = useState<string | null>(null);

  // Derive displayed name via shared helper (seeded-and-pristine -> t(key), else stored name).
  const displayName = mode ? localizeSmartModeName(mode.id, mode.name, t) : "";

  // Draft state for the edit form
  const [draftName, setDraftName] = useState(mode?.name ?? "");
  const [draftPrompt, setDraftPrompt] = useState(mode?.prompt ?? "");
  const [draftLang, setDraftLang] = useState<TargetLanguage | null>(
    mode?.target_language ?? null,
  );

  // Derive current shortcut binding from settings
  const bindings = getSetting("bindings") ?? {};
  const bindingKey = mode ? "smart_mode_" + mode.id : null;
  const currentBinding = bindingKey
    ? (bindings[bindingKey]?.current_binding ?? null)
    : null;

  // Validation: primary CTA disabled when required fields empty
  const isValid = (() => {
    if (draftName.trim() === "") return false;
    if (kind === "rewrite" && draftPrompt.trim() === "") return false;
    if (kind === "translation" && draftLang === null) return false;
    return true;
  })();

  const handleSave = async () => {
    if (!isValid) return;
    if (mode) {
      const res = await commands.updateSmartMode(
        mode.id,
        draftName.trim(),
        kind === "rewrite" ? draftPrompt.trim() : "",
        kind === "translation" ? draftLang : null,
      );
      if (res.status === "ok") {
        setIsEditing(false);
        onChanged();
      }
    }
  };

  const handleCreate = async () => {
    if (!isValid) return;
    const res = await commands.addSmartMode(
      draftName.trim(),
      kind,
      kind === "rewrite" ? draftPrompt.trim() : "",
      kind === "translation" ? draftLang : null,
    );
    if (res.status === "ok") {
      onChanged();
      onDiscardNew?.();
    }
  };

  const handleDiscard = () => {
    if (isNew) {
      onDiscardNew?.();
    } else {
      setDraftName(mode!.name);
      setDraftPrompt(mode!.prompt);
      setDraftLang(mode!.target_language ?? null);
      setIsEditing(false);
    }
  };

  const handleDelete = async () => {
    if (!mode) return;
    const name = displayName;
    const ok = await ask(t("smartModes.delete.confirm", { name }), {
      title: t("smartModes.delete.title"),
      kind: "warning",
    });
    if (ok) {
      await commands.deleteSmartMode(mode.id);
      onChanged();
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      if (!disabled) setIsEditing(true);
    }
  };

  // Kind badge classes
  const kindBadgeClass =
    kind === "rewrite"
      ? "inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-mid-gray/10 text-mid-gray"
      : "inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-logo-primary/10 text-logo-primary border border-logo-primary/30";

  // Card frame classes
  const cardBase =
    "flex flex-col gap-2 px-4 py-3 rounded-xl border-2 transition-all";
  const cardCollapsed = disabled
    ? `${cardBase} border-mid-gray/20 opacity-60 bg-mid-gray/5`
    : isEditing
      ? `${cardBase} border-logo-primary/40 bg-logo-primary/5`
      : `${cardBase} border-mid-gray/20 hover:border-logo-primary/50 hover:bg-logo-primary/5`;

  if (isEditing || isNew) {
    return (
      <div className={`${cardBase} border-logo-primary/40 bg-logo-primary/5`}>
        {/* Header row in edit mode (only for existing cards) */}
        {!isNew && mode && (
          <div className="flex justify-between gap-2 items-start">
            <div className="flex items-center gap-2 flex-wrap">
              <span className={kindBadgeClass}>
                {kind === "rewrite"
                  ? t("smartModes.kind.rewrite")
                  : t("smartModes.kind.translation")}
              </span>
              <span className="text-base font-medium text-text">
                {displayName}
              </span>
            </div>
          </div>
        )}

        {/* Edit form */}
        <div className="flex flex-col gap-3">
          {/* Name field */}
          <div className="flex flex-col gap-1">
            <label className="text-sm font-medium">
              {t("smartModes.card.nameLabel")}
            </label>
            <Input
              variant="compact"
              value={draftName}
              onChange={(e) => setDraftName(e.target.value)}
              placeholder={t("smartModes.card.namePlaceholder")}
              className="w-full"
            />
          </div>

          {/* Rewrite: prompt field */}
          {kind === "rewrite" && (
            <div className="flex flex-col gap-1">
              <label className="text-sm font-medium">
                {t("smartModes.card.promptLabel")}
              </label>
              <Textarea
                variant="compact"
                value={draftPrompt}
                onChange={(e) => setDraftPrompt(e.target.value)}
                placeholder={t("smartModes.card.promptPlaceholder")}
                className="w-full"
              />
              <p className="text-xs text-mid-gray/70">
                {t("smartModes.card.outputHint")}
              </p>
            </div>
          )}

          {/* Translation: language selector */}
          {kind === "translation" && (
            <div className="flex flex-col gap-1">
              <label className="text-sm font-medium">
                {t("smartModes.card.targetLanguage")}
              </label>
              <select
                className="px-2 py-1 text-sm font-medium bg-mid-gray/10 border border-mid-gray/80 rounded-md focus:outline-none focus:border-logo-primary hover:border-logo-primary"
                value={draftLang?.code ?? ""}
                onChange={(e) => {
                  const found = availableLanguages.find(
                    (l) => l.code === e.target.value,
                  );
                  setDraftLang(found ?? null);
                }}
              >
                <option value="">{t("smartModes.card.targetLanguage")}</option>
                {availableLanguages.map((lang) => (
                  <option key={lang.code} value={lang.code}>
                    {lang.label}
                  </option>
                ))}
              </select>
              {genericEngineNote && (
                <p className="text-xs text-mid-gray/60">
                  {t("smartModes.translation.genericEngineNote")}
                </p>
              )}
            </div>
          )}

          {/* Action row */}
          <div className="flex gap-2">
            <Button
              variant="primary"
              size="md"
              disabled={!isValid}
              onClick={isNew ? handleCreate : handleSave}
            >
              {isNew
                ? t("smartModes.card.createCta")
                : t("smartModes.card.saveCta")}
            </Button>
            <Button variant="secondary" size="md" onClick={handleDiscard}>
              {isNew
                ? t("smartModes.card.discardNewCta")
                : t("smartModes.card.discardCta")}
            </Button>
          </div>
        </div>
      </div>
    );
  }

  // Collapsed view
  if (!mode) return null;

  return (
    <div
      className={cardCollapsed}
      role="button"
      tabIndex={disabled ? undefined : 0}
      aria-disabled={disabled ? "true" : undefined}
      onKeyDown={disabled ? undefined : handleKeyDown}
    >
      {/* Top row */}
      <div className="flex justify-between gap-2 items-start">
        <div className="flex items-center gap-2 flex-wrap min-w-0">
          <span className={kindBadgeClass}>
            {kind === "rewrite"
              ? t("smartModes.kind.rewrite")
              : t("smartModes.kind.translation")}
          </span>
          <span className="text-base font-medium text-text">{displayName}</span>
          {/* Preview */}
          {kind === "rewrite" ? (
            <span className="text-sm text-text/60 truncate">
              {mode.prompt.slice(0, 80)}
              {mode.prompt.length > 80 ? "…" : ""}
            </span>
          ) : (
            <span className="text-sm text-text/60">
              {mode.target_language?.label ?? ""}
            </span>
          )}
        </div>

        {/* Shortcut chip + edit/delete (hidden when disabled) */}
        <div className="flex items-center gap-1 shrink-0">
          <SmartModeShortcutChip
            modeId={mode.id}
            currentBinding={currentBinding}
            disabled={disabled}
            onBound={onChanged}
            onConflictChange={setShortcutConflict}
          />
          {!disabled && (
            <>
              <button
                type="button"
                aria-label={t("smartModes.edit.ariaLabel", {
                  name: displayName,
                })}
                onClick={(e) => {
                  e.stopPropagation();
                  setIsEditing(true);
                }}
                className="p-0.5 rounded focus:outline-none"
              >
                <Pencil className="w-4 h-4 text-mid-gray/50 hover:text-text" />
              </button>
              <button
                type="button"
                aria-label={t("smartModes.delete.ariaLabel", {
                  name: displayName,
                })}
                onClick={(e) => {
                  e.stopPropagation();
                  void handleDelete();
                }}
                className="p-0.5 rounded focus:outline-none"
              >
                <Trash2 className="w-4 h-4 text-mid-gray/50 hover:text-red-400" />
              </button>
            </>
          )}
        </div>
      </div>
      {shortcutConflict && (
        <p
          role="alert"
          dir="auto"
          className="w-full text-xs text-red-400 whitespace-normal break-words"
        >
          {shortcutConflict}
        </p>
      )}
    </div>
  );
};
