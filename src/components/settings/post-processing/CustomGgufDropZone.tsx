import React, { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { Alert } from "@/components/ui/Alert";
import { Button } from "@/components/ui/Button";
import { useLlmModelStore } from "@/stores/llmModelStore";

export const CustomGgufDropZone: React.FC = () => {
  const { t } = useTranslation();
  const store = useLlmModelStore();
  const [isDragActive, setIsDragActive] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const unlistenRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    const setupDragDrop = async () => {
      const webview = getCurrentWebview();
      const unlisten = await webview.onDragDropEvent((event) => {
        if (event.payload.type === "enter") {
          setIsDragActive(true);
        } else if (event.payload.type === "leave") {
          setIsDragActive(false);
        } else if (event.payload.type === "drop") {
          setIsDragActive(false);
          const paths = event.payload.paths;
          if (paths && paths.length > 0) {
            void handleImport(paths[0]);
          }
        }
      });
      unlistenRef.current = unlisten;
    };

    void setupDragDrop();

    return () => {
      unlistenRef.current?.();
    };
  }, []); // Mount once — webview drag-drop listener registered on mount only

  const handleImport = async (filePath: string) => {
    setError(null);
    const result = await store.importCustom(filePath);
    if (!result.ok) {
      setError(
        t(
          "settings.postProcessing.modelsAndLocalProcessing.library.invalidGguf",
        ),
      );
    }
  };

  const handleFilePickerOpen = async () => {
    setError(null);
    const selected = await open({
      filters: [{ name: "GGUF", extensions: ["gguf"] }],
      multiple: false,
    });
    if (selected && typeof selected === "string") {
      await handleImport(selected);
    }
  };

  return (
    <div className="space-y-2">
      <Button
        variant="secondary"
        size="md"
        onClick={() => void handleFilePickerOpen()}
      >
        {t(
          "settings.postProcessing.modelsAndLocalProcessing.library.addCustomModel",
        )}
      </Button>

      <div
        role="region"
        aria-label={t(
          "settings.postProcessing.modelsAndLocalProcessing.library.dropZoneLabel",
        )}
        tabIndex={0}
        className={[
          "border border-dashed min-h-[44px] rounded-lg flex flex-col items-center justify-center p-4 transition-colors focus:outline-none focus:ring-2 focus:ring-logo-primary",
          isDragActive
            ? "border-logo-primary/50 bg-logo-primary/5"
            : "border-mid-gray/40",
        ]
          .filter(Boolean)
          .join(" ")}
      >
        <span className="text-sm text-mid-gray">
          {t(
            "settings.postProcessing.modelsAndLocalProcessing.library.dropZoneLabel",
          )}
        </span>
        <span className="text-xs text-mid-gray/70">
          {t(
            "settings.postProcessing.modelsAndLocalProcessing.library.dropZoneSublabel",
          )}
        </span>
      </div>

      {error && (
        <Alert variant="error" contained>
          {error}
        </Alert>
      )}
    </div>
  );
};
