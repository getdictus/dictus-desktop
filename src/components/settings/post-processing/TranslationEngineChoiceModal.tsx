import React, { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { X } from "lucide-react";
import { commands } from "@/bindings";
import { Button } from "@/components/ui/Button";
import { useLlmModelStore } from "@/stores/llmModelStore";

const TRANSLATE_GEMMA_MODEL_ID = "translate-gemma-4b";

interface TranslationEngineChoiceModalProps {
  open: boolean;
  onClose: () => void;
  onChosen: () => void;
}

export const TranslationEngineChoiceModal: React.FC<
  TranslationEngineChoiceModalProps
> = ({ open, onClose, onChosen }) => {
  const { t } = useTranslation();
  const store = useLlmModelStore();

  useEffect(() => {
    if (!open) return;
    void store.refresh();
    let unlistenFn: (() => void) | null = null;
    void store.initListeners().then((fn) => {
      unlistenFn = fn;
    });
    return () => {
      unlistenFn?.();
    };
  }, [open, store]);

  if (!open) return null;

  const gemmaModel = store.models.find((m) => m.id === TRANSLATE_GEMMA_MODEL_ID);
  const isDownloaded = gemmaModel?.is_downloaded ?? false;
  const isDownloading =
    gemmaModel?.is_downloading ??
    TRANSLATE_GEMMA_MODEL_ID in store.downloadProgress;
  const downloadPercentage =
    store.downloadProgress[TRANSLATE_GEMMA_MODEL_ID]?.percentage;

  const handleDownloadGemma = () => {
    void store.downloadModel(TRANSLATE_GEMMA_MODEL_ID);
  };

  const handleUseGemma = async () => {
    const res = await commands.setTranslationEngineChoice("translate_gemma");
    if (res.status === "ok") {
      onChosen();
      onClose();
    }
  };

  const handleUseGeneric = async () => {
    const res = await commands.setTranslationEngineChoice("generic_model");
    if (res.status === "ok") {
      onChosen();
      onClose();
    }
  };

  return (
    /* Overlay backdrop */
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <div className="relative bg-background border border-mid-gray/20 rounded-xl shadow-xl p-6 max-w-md w-full mx-4 flex flex-col gap-6">
        {/* Option A: Dedicated translation model */}
        <div className="flex flex-col gap-3 p-4 rounded-xl border-2 border-mid-gray/20">
          <div>
            <p className="text-base font-medium">
              {t("smartModes.translation.modal.dedicatedHeading")}
            </p>
            <p className="text-sm text-text/60 mt-1">
              {t("smartModes.translation.modal.dedicatedSubtext")}
            </p>
          </div>

          {isDownloaded ? (
            <Button variant="primary" size="md" onClick={handleUseGemma}>
              {t("smartModes.translation.modal.useButton")}
            </Button>
          ) : isDownloading ? (
            <div className="flex flex-col gap-2">
              <div className="w-full bg-mid-gray/20 rounded-full h-2">
                <div
                  className="bg-logo-primary h-2 rounded-full transition-all"
                  style={{ width: `${downloadPercentage ?? 0}%` }}
                />
              </div>
              <p className="text-xs text-mid-gray/60">
                {Math.round(downloadPercentage ?? 0)}%
              </p>
            </div>
          ) : (
            <Button variant="primary" size="md" onClick={handleDownloadGemma}>
              {t("smartModes.translation.modal.downloadButton")}
            </Button>
          )}
        </div>

        {/* Option B: Generic model */}
        <div className="flex flex-col gap-3 p-4 rounded-xl border-2 border-mid-gray/20">
          <div>
            <p className="text-base font-medium">
              {t("smartModes.translation.modal.genericHeading")}
            </p>
            <p className="text-sm text-text/60 mt-1">
              {t("smartModes.translation.modal.genericSubtext")}
            </p>
          </div>
          <Button variant="secondary" size="md" onClick={handleUseGeneric}>
            {t("smartModes.translation.modal.useGenericButton")}
          </Button>
        </div>

        {/* Footnote */}
        <p className="text-xs text-mid-gray/70">
          {t("smartModes.translation.modal.footnote")}
        </p>

        {/* Close button */}
        <button
          type="button"
          className="absolute top-4 right-4 text-mid-gray/60 hover:text-text focus:outline-none"
          onClick={onClose}
          aria-label={t("common.close")}
        >
          <X className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
};
