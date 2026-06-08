import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { X } from "lucide-react";
import { commands } from "@/bindings";
import { Button } from "@/components/ui/Button";
import { useLlmModelStore } from "@/stores/llmModelStore";

// Model recommended for translation. A general-purpose 4B model gives clean,
// reliable translation across languages/idioms (benchmarked to match or beat a
// dedicated translation model, without its runaway-output issues).
const RECOMMENDED_MODEL_ID = "gemma-3-4b";

interface TranslationEngineChoiceModalProps {
  open: boolean;
  /** Whether translation is currently enabled (engine already chosen). */
  enabled?: boolean;
  onClose: () => void;
  onChosen: () => void;
}

export const TranslationEngineChoiceModal: React.FC<
  TranslationEngineChoiceModalProps
> = ({ open, enabled = false, onClose, onChosen }) => {
  const { t } = useTranslation();
  // Select only the slices we read (subscribing to the whole store would cause
  // a refresh→re-render loop).
  const models = useLlmModelStore((s) => s.models);
  const downloadProgress = useLlmModelStore((s) => s.downloadProgress);
  const verifyingModels = useLlmModelStore((s) => s.verifyingModels);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    const { refresh, initListeners } = useLlmModelStore.getState();
    void refresh();
    let unlistenFn: (() => void) | null = null;
    void initListeners().then((fn) => {
      unlistenFn = fn;
    });
    return () => {
      unlistenFn?.();
    };
  }, [open]);

  if (!open) return null;

  const recModel = models.find((m) => m.id === RECOMMENDED_MODEL_ID);
  const recDownloaded = recModel?.is_downloaded ?? false;
  const recVerifying = verifyingModels[RECOMMENDED_MODEL_ID] ?? false;
  const recDownloading =
    (recModel?.is_downloading ?? false) ||
    RECOMMENDED_MODEL_ID in downloadProgress;
  const recPct = downloadProgress[RECOMMENDED_MODEL_ID]?.percentage;

  const handleDownloadRec = () => {
    void useLlmModelStore.getState().downloadModel(RECOMMENDED_MODEL_ID);
  };

  // Enable translation: persist generic_model choice without mutating the active model/provider.
  const handleEnable = async () => {
    if (enabled) {
      onChosen();
      onClose();
      return;
    }
    setError(null);
    setBusy(true);
    try {
      const res = await commands.setTranslationEngineChoice("generic_model");
      if (res.status === "ok") {
        onChosen();
        onClose();
      } else {
        setError(res.error ?? t("smartModes.translation.modal.applyFailed"));
      }
    } catch (e) {
      setError(
        e instanceof Error
          ? e.message
          : t("smartModes.translation.modal.applyFailed"),
      );
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <div className="relative bg-background border border-mid-gray/20 rounded-xl shadow-xl p-6 max-w-md w-full mx-4 flex flex-col gap-6">
        <p className="text-base font-medium pr-6">
          {t("smartModes.translation.modal.title")}
        </p>

        {/* Recommendation block */}
        <div className="flex flex-col gap-3 p-4 rounded-xl border-2 border-mid-gray/20">
          <div>
            <p className="text-base font-medium">
              {t("smartModes.translation.modal.recommendOnlyHeading")}
            </p>
            <p className="text-sm text-text/60 mt-1">
              {t("smartModes.translation.modal.recommendOnlyBody")}
            </p>
          </div>

          {recVerifying ? (
            <div className="flex flex-col gap-2">
              <div className="w-full bg-mid-gray/20 rounded-full h-2 overflow-hidden">
                <div className="bg-logo-primary h-2 rounded-full w-1/3 animate-pulse" />
              </div>
              <p className="text-xs text-mid-gray/60">
                {t("smartModes.translation.modal.verifying")}
              </p>
            </div>
          ) : recDownloading ? (
            <div className="flex flex-col gap-2">
              <div className="w-full bg-mid-gray/20 rounded-full h-2">
                <div
                  className="bg-logo-primary h-2 rounded-full transition-all"
                  style={{ width: `${recPct ?? 0}%` }}
                />
              </div>
              <p className="text-xs text-mid-gray/60">
                {t("smartModes.translation.modal.downloadingPercent", {
                  percent: Math.round(recPct ?? 0),
                })}
              </p>
              <p className="text-xs text-mid-gray/60">
                {t("smartModes.translation.modal.downloadBackgroundNote")}
              </p>
            </div>
          ) : !recDownloaded ? (
            <Button variant="primary" size="md" onClick={handleDownloadRec}>
              {t("smartModes.translation.modal.downloadRecommended")}
            </Button>
          ) : null}
        </div>

        {/* Enable CTA */}
        {!enabled && (
          <Button
            variant="primary"
            size="md"
            disabled={busy}
            onClick={() => void handleEnable()}
          >
            {busy
              ? t("smartModes.translation.modal.applying")
              : t("smartModes.translation.modal.enableCta")}
          </Button>
        )}

        {error && (
          <p role="alert" className="text-xs text-red-400">
            {error}
          </p>
        )}

        <p className="text-xs text-mid-gray/70">
          {t("smartModes.translation.modal.footnote")}
        </p>

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
