import React, { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus } from "lucide-react";
import type { PostProcessProvider, SmartMode, TargetLanguage } from "@/bindings";
import { commands } from "@/bindings";
import { Button } from "@/components/ui/Button";
import { useSettings } from "@/hooks/useSettings";
import { useLlmModelStore } from "@/stores/llmModelStore";
import { SmartModeCard } from "./SmartModeCard";
import { SmartModeTemplatePicker } from "./SmartModeTemplatePicker";
import { TranslationEngineChoiceModal } from "./TranslationEngineChoiceModal";

// Languages supported by TranslateGemma (~40 benchmarked languages)
const TRANSLATE_GEMMA_LANGUAGES: TargetLanguage[] = [
  { code: "en", label: "English" },
  { code: "es", label: "Spanish" },
  { code: "fr", label: "French" },
  { code: "de", label: "German" },
  { code: "it", label: "Italian" },
  { code: "pt", label: "Portuguese" },
  { code: "nl", label: "Dutch" },
  { code: "pl", label: "Polish" },
  { code: "ru", label: "Russian" },
  { code: "ja", label: "Japanese" },
  { code: "ko", label: "Korean" },
  { code: "zh", label: "Chinese (Simplified)" },
  { code: "zh-TW", label: "Chinese (Traditional)" },
  { code: "ar", label: "Arabic" },
  { code: "he", label: "Hebrew" },
  { code: "tr", label: "Turkish" },
  { code: "sv", label: "Swedish" },
  { code: "da", label: "Danish" },
  { code: "fi", label: "Finnish" },
  { code: "no", label: "Norwegian" },
  { code: "cs", label: "Czech" },
  { code: "sk", label: "Slovak" },
  { code: "ro", label: "Romanian" },
  { code: "hu", label: "Hungarian" },
  { code: "bg", label: "Bulgarian" },
  { code: "hr", label: "Croatian" },
  { code: "uk", label: "Ukrainian" },
  { code: "vi", label: "Vietnamese" },
  { code: "th", label: "Thai" },
  { code: "id", label: "Indonesian" },
  { code: "ms", label: "Malay" },
  { code: "hi", label: "Hindi" },
  { code: "bn", label: "Bengali" },
  { code: "ta", label: "Tamil" },
  { code: "te", label: "Telugu" },
  { code: "ur", label: "Urdu" },
  { code: "fa", label: "Persian" },
  { code: "el", label: "Greek" },
  { code: "ca", label: "Catalan" },
  { code: "af", label: "Afrikaans" },
];

// Broader curated list for the generic model path
const GENERIC_LANGUAGES: TargetLanguage[] = [
  ...TRANSLATE_GEMMA_LANGUAGES,
  { code: "lt", label: "Lithuanian" },
  { code: "lv", label: "Latvian" },
  { code: "et", label: "Estonian" },
  { code: "sl", label: "Slovenian" },
  { code: "sr", label: "Serbian" },
  { code: "mk", label: "Macedonian" },
  { code: "sq", label: "Albanian" },
  { code: "be", label: "Belarusian" },
  { code: "ka", label: "Georgian" },
  { code: "hy", label: "Armenian" },
  { code: "az", label: "Azerbaijani" },
  { code: "kk", label: "Kazakh" },
  { code: "uz", label: "Uzbek" },
  { code: "sw", label: "Swahili" },
  { code: "am", label: "Amharic" },
  { code: "cy", label: "Welsh" },
  { code: "ga", label: "Irish" },
  { code: "mt", label: "Maltese" },
  { code: "is", label: "Icelandic" },
  { code: "eu", label: "Basque" },
  { code: "gl", label: "Galician" },
];

export const SmartModesSection: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, refreshSettings } = useSettings();

  const [modes, setModes] = useState<SmartMode[]>([]);
  const [creatingRewrite, setCreatingRewrite] = useState(false);
  const [creatingTranslation, setCreatingTranslation] = useState(false);
  const [modalOpen, setModalOpen] = useState(false);
  const [pickerOpen, setPickerOpen] = useState<"rewrite" | "translation" | null>(null);

  const engineChoice = getSetting("translation_engine_choice") ?? "not_chosen";
  const translationEnabled = engineChoice !== "not_chosen";

  // Translation always runs through the active generic LLM model now.
  const availableLanguages = GENERIC_LANGUAGES;
  const genericEngineNote = translationEnabled;

  const activeModelId = useLlmModelStore((s) => s.activeModelId);
  const llmModels = useLlmModelStore((s) => s.models);
  const activeModelName =
    llmModels.find((m) => m.id === activeModelId)?.name ?? null;

  // Resolve the active-engine descriptor for the translation modal.
  // Uses the GGUF model name when one is active; otherwise falls back to the
  // active post-process provider's display label (covers Apple Intelligence,
  // cloud providers, etc.).
  const providerId = (getSetting("post_process_provider_id") as string | undefined) ?? "";
  const providers = (getSetting("post_process_providers") as PostProcessProvider[] | undefined) ?? [];
  const activeEngineName: string | null =
    activeModelName ??
    providers.find((p) => p.id === providerId)?.label ??
    (providerId ? providerId : null);
  // True only when the active GGUF model is the recommended Gemma 3 4B
  const activeIsRecommended = activeModelId === "gemma-3-4b";

  const refetchModes = useCallback(async () => {
    const r = await commands.listSmartModes();
    if (r.status === "ok") {
      setModes(r.data);
    }
    // Shortcut bindings live in the settings store (not in SmartMode), so any
    // mode/binding change must refresh settings or the chip display goes stale.
    await refreshSettings?.();
  }, [refreshSettings]);

  useEffect(() => {
    void refetchModes();
  }, [refetchModes]);

  const rewrites = modes.filter((m) => m.kind === "rewrite");
  const translations = modes.filter((m) => m.kind === "translation");

  return (
    <div className="space-y-6">
      {/* Rewrite section */}
      <div className="space-y-2">
        <div className="px-4">
          <h2 className="text-xs font-medium text-mid-gray uppercase tracking-wide">
            {t("smartModes.sections.rewrite")}
          </h2>
        </div>
        <div className="flex flex-col gap-2">
          {rewrites.map((mode) => (
            <SmartModeCard
              key={mode.id}
              mode={mode}
              kind="rewrite"
              availableLanguages={availableLanguages}
              genericEngineNote={genericEngineNote}
              onChanged={refetchModes}
            />
          ))}
          {creatingRewrite && (
            <SmartModeCard
              mode={null}
              kind="rewrite"
              availableLanguages={availableLanguages}
              genericEngineNote={genericEngineNote}
              onChanged={refetchModes}
              onDiscardNew={() => setCreatingRewrite(false)}
            />
          )}
          <div>
            <Button
              variant="primary-soft"
              size="sm"
              disabled={creatingRewrite}
              onClick={() => setPickerOpen("rewrite")}
            >
              <Plus className="w-3 h-3 mr-1 inline" />
              {t("smartModes.createRewrite")}
            </Button>
          </div>
        </div>
      </div>

      {/* Translation section */}
      <div className="space-y-2">
        <div className="px-4 flex items-center justify-between gap-2">
          <h2
            className={`text-xs font-medium uppercase tracking-wide ${translationEnabled ? "text-mid-gray" : "text-amber-500"}`}
          >
            {t("smartModes.sections.translation")}
          </h2>
          {translationEnabled && (
            <div className="flex items-center gap-2">
              <span className="text-xs text-mid-gray">
                {t("smartModes.translation.currentEngine", {
                  engine: activeModelName ?? t("smartModes.translation.engineGeneric"),
                })}
              </span>
              <Button
                variant="secondary"
                size="sm"
                onClick={() => setModalOpen(true)}
              >
                {t("smartModes.translation.changeEngine")}
              </Button>
            </div>
          )}
        </div>
        <div className="flex flex-col gap-2">
          {translations.map((mode) => (
            <SmartModeCard
              key={mode.id}
              mode={mode}
              kind="translation"
              disabled={!translationEnabled}
              availableLanguages={availableLanguages}
              genericEngineNote={genericEngineNote}
              onChanged={refetchModes}
            />
          ))}

          {/* Enable CTA box when translation not yet configured */}
          {!translationEnabled && (
            <div className="bg-amber-500/5 border border-amber-500/20 rounded-xl p-4 flex flex-col gap-3">
              <div>
                <p className="text-sm font-medium">
                  {t("smartModes.translation.enableHeading")}
                </p>
                <p className="text-sm text-text/60 mt-1">
                  {t("smartModes.translation.enableBody")}
                </p>
              </div>
              <div>
                <Button
                  variant="primary-soft"
                  size="md"
                  onClick={() => setModalOpen(true)}
                >
                  {t("smartModes.translation.chooseCta")}
                </Button>
              </div>
            </div>
          )}

          {/* New translation card (only when translation is enabled) */}
          {creatingTranslation && translationEnabled && (
            <SmartModeCard
              mode={null}
              kind="translation"
              availableLanguages={availableLanguages}
              genericEngineNote={genericEngineNote}
              onChanged={refetchModes}
              onDiscardNew={() => setCreatingTranslation(false)}
            />
          )}

          <div>
            <Button
              variant="primary-soft"
              size="sm"
              disabled={!translationEnabled || creatingTranslation}
              onClick={() => setPickerOpen("translation")}
            >
              <Plus className="w-3 h-3 mr-1 inline" />
              {t("smartModes.createTranslation")}
            </Button>
          </div>
        </div>
      </div>

      <TranslationEngineChoiceModal
        open={modalOpen}
        enabled={translationEnabled}
        engineChoice={engineChoice as string}
        activeEngineName={activeEngineName}
        activeIsRecommended={activeIsRecommended}
        onClose={() => setModalOpen(false)}
        onChosen={() => {
          void refreshSettings?.();
          void refetchModes();
        }}
      />

      <SmartModeTemplatePicker
        open={pickerOpen === "rewrite"}
        kind="rewrite"
        existingModeIds={rewrites.map((m) => m.id)}
        onClose={() => setPickerOpen(null)}
        onCreated={refetchModes}
        onCustom={() => setCreatingRewrite(true)}
      />

      <SmartModeTemplatePicker
        open={pickerOpen === "translation"}
        kind="translation"
        existingModeIds={translations.map((m) => m.id)}
        onClose={() => setPickerOpen(null)}
        onCreated={refetchModes}
        onCustom={() => setCreatingTranslation(true)}
      />
    </div>
  );
};
