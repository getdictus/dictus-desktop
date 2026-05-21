import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { Alert } from "../../ui/Alert";
import { Button } from "../../ui/Button";
import { useSettings } from "../../../hooks/useSettings";

interface TestConnectionButtonProps {
  baseUrl: string;
}

type Status =
  | { kind: "idle" }
  | { kind: "loading" }
  | { kind: "success"; count: number }
  | { kind: "error"; message: string };

export const TestConnectionButton: React.FC<TestConnectionButtonProps> = ({
  baseUrl,
}) => {
  const { t } = useTranslation();
  const { fetchPostProcessModels } = useSettings();
  const [status, setStatus] = useState<Status>({ kind: "idle" });

  const handleClick = async () => {
    setStatus({ kind: "loading" });
    try {
      const models = await fetchPostProcessModels("custom");
      if (Array.isArray(models) && models.length > 0) {
        setStatus({ kind: "success", count: models.length });
      } else {
        setStatus({
          kind: "error",
          message: "No models returned",
        });
      }
    } catch (error) {
      setStatus({ kind: "error", message: String(error) });
    }
  };

  return (
    <div className="space-y-2">
      <Button
        onClick={handleClick}
        disabled={status.kind === "loading"}
        variant="secondary"
        size="sm"
      >
        {t("settings.postProcessing.api.custom.testConnection.button")}
      </Button>
      {status.kind === "success" ? (
        <Alert variant="success" contained>
          {t("settings.postProcessing.api.custom.testConnection.success", {
            count: status.count,
          })}
        </Alert>
      ) : null}
      {status.kind === "error" ? (
        <Alert variant="error" contained>
          <span>
            {t("settings.postProcessing.api.custom.testConnection.errorTitle", {
              baseUrl,
            })}
          </span>
          <details className="mt-1 text-xs opacity-80">
            <summary className="cursor-pointer">
              {t(
                "settings.postProcessing.api.custom.testConnection.errorDetails",
              )}
            </summary>
            <pre className="mt-1 whitespace-pre-wrap break-all">
              {status.message}
            </pre>
          </details>
        </Alert>
      ) : null}
    </div>
  );
};

TestConnectionButton.displayName = "TestConnectionButton";
