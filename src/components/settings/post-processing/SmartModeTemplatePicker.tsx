import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import type { SmartMode, SmartModeKind } from "@/bindings";
import { commands } from "@/bindings";
import { SEEDED_MODE_ID_TO_I18N_KEY } from "./SmartModeCard";

interface SmartModeTemplatePickerProps {
  open: boolean;
  kind: SmartModeKind;
  existingModeIds: string[];
  onClose: () => void;
  onCreated: () => void;
  onCustom: () => void;
}

export const SmartModeTemplatePicker: React.FC<
  SmartModeTemplatePickerProps
> = ({ open, kind, existingModeIds, onClose, onCreated, onCustom }) => {
  const { t } = useTranslation();
  const [templates, setTemplates] = useState<SmartMode[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!open) return;
    setLoading(true);
    void commands.smartModeTemplates().then((all) => {
      setTemplates(all.filter((m) => m.kind === kind));
      setLoading(false);
    });
  }, [open, kind]);

  if (!open) return null;

  const title =
    kind === "rewrite"
      ? t("smartModes.picker.rewriteTitle")
      : t("smartModes.picker.translationTitle");

  const handlePickTemplate = async (template: SmartMode) => {
    // Always creates a new mode (new id via addSmartMode) so deleted-then-readded works without id collision.
    const res = await commands.addSmartMode(
      template.name,
      template.kind,
      template.prompt,
      template.target_language ?? null,
    );
    if (res.status === "ok") {
      onCreated();
      onClose();
    }
  };

  const handleCustom = () => {
    onClose();
    onCustom();
  };

  const getTemplatePreview = (template: SmartMode): string => {
    if (template.kind === "rewrite") {
      const slice = template.prompt.slice(0, 80);
      return template.prompt.length > 80 ? slice + "…" : slice;
    }
    return template.target_language?.label ?? "";
  };

  const getTemplateName = (template: SmartMode): string => {
    const i18nKey = SEEDED_MODE_ID_TO_I18N_KEY[template.id];
    return i18nKey ? t(i18nKey) : template.name;
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="bg-background border border-mid-gray/20 rounded-lg p-4 max-w-md w-full max-h-[80vh] overflow-y-auto mx-4 shadow-xl">
        {/* Title */}
        <p className="text-lg font-semibold mb-3">{title}</p>

        {/* Template list */}
        {loading ? (
          <p className="text-sm text-mid-gray py-4 text-center">…</p>
        ) : (
          <div className="flex flex-col gap-1">
            {templates.map((template) => {
              const alreadyAdded = existingModeIds.includes(template.id);
              return (
                <button
                  key={template.id}
                  type="button"
                  className="w-full text-left px-3 py-2 rounded-md hover:bg-logo-primary/5 border border-transparent hover:border-logo-primary/30"
                  onClick={() => void handlePickTemplate(template)}
                >
                  <div className="flex items-center justify-between gap-2">
                    <span className="font-medium text-sm">
                      {getTemplateName(template)}
                    </span>
                    {alreadyAdded && (
                      <span className="text-xs text-mid-gray shrink-0">
                        {t("smartModes.picker.added")}
                      </span>
                    )}
                  </div>
                  <p className="text-xs text-mid-gray mt-0.5 line-clamp-1">
                    {getTemplatePreview(template)}
                  </p>
                </button>
              );
            })}

            {/* Custom entry */}
            <div className="border-t border-mid-gray/20 mt-1 pt-1">
              <button
                type="button"
                className="w-full text-left px-3 py-2 rounded-md hover:bg-logo-primary/5 border border-transparent hover:border-logo-primary/30"
                onClick={handleCustom}
              >
                <span className="font-medium text-sm">
                  {t("smartModes.picker.custom")}
                </span>
                <p className="text-xs text-mid-gray mt-0.5">
                  {t("smartModes.picker.customSubtext")}
                </p>
              </button>
            </div>
          </div>
        )}

        {/* Close affordance */}
        <div className="mt-3 flex justify-end">
          <button
            type="button"
            className="text-sm text-mid-gray hover:text-text focus:outline-none"
            onClick={onClose}
          >
            {t("common.cancel")}
          </button>
        </div>
      </div>
    </div>
  );
};
