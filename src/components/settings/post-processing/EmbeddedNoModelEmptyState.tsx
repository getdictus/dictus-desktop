import React from "react";
import { useTranslation } from "react-i18next";
import { useLlmModelStore } from "@/stores/llmModelStore";

/**
 * Inline empty-state shown under the "Embedded (local)" provider row
 * when no local LLM model has been downloaded yet.
 *
 * Clicking the link scrolls to LlmLibrarySection and briefly highlights
 * the recommended Qwen2.5-1.5B card.
 *
 * Respects `prefers-reduced-motion: reduce` — when active, the pulse
 * animation is skipped and a static accent border is applied instead.
 */
export const EmbeddedNoModelEmptyState: React.FC = () => {
  const { t } = useTranslation();
  const store = useLlmModelStore();

  const handleDownloadLink = () => {
    // Scroll the library section into view
    const section = document.getElementById("llm-library-section");
    section?.scrollIntoView({ behavior: "smooth" });

    // Highlight the recommended model card
    const prefersReducedMotion = window.matchMedia(
      "(prefers-reduced-motion: reduce)",
    ).matches;

    // Find the recommended model id from the store
    const recommendedModel = store.models.find((m) => m.is_recommended);
    if (!recommendedModel) return;

    const cardId = `llm-card-${recommendedModel.id}`;
    const cardEl = document.getElementById(cardId);
    if (!cardEl) return;

    if (prefersReducedMotion) {
      // Static accent border only — no animation
      cardEl.classList.add("border-logo-primary/50");
    } else {
      // One-cycle pulse animation then settle to static accent border
      cardEl.classList.add("animate-pulse-once", "border-logo-primary/50");
      const onAnimEnd = () => {
        cardEl.classList.remove("animate-pulse-once");
        cardEl.removeEventListener("animationend", onAnimEnd);
      };
      cardEl.addEventListener("animationend", onAnimEnd);
    }
  };

  return (
    <div className="text-sm text-mid-gray space-y-1 mt-2 pb-1">
      <p className="font-medium">
        {t(
          "settings.postProcessing.modelsAndLocalProcessing.embedded.noModelHeading",
        )}
      </p>
      <p className="text-sm text-mid-gray leading-relaxed">
        {t(
          "settings.postProcessing.modelsAndLocalProcessing.embedded.noModelBody",
        )}
      </p>
      <button
        type="button"
        onClick={handleDownloadLink}
        className="text-logo-primary underline underline-offset-2 hover:opacity-80 cursor-pointer bg-transparent border-0 p-0 text-sm"
      >
        {t(
          "settings.postProcessing.modelsAndLocalProcessing.embedded.downloadNowLink",
        )}
      </button>
    </div>
  );
};
