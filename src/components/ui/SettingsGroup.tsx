import React from "react";
import { Surface } from "./Surface";

interface SettingsGroupProps {
  title?: string;
  description?: string;
  children: React.ReactNode;
}

export const SettingsGroup: React.FC<SettingsGroupProps> = ({
  title,
  description,
  children,
}) => {
  return (
    <div className="space-y-1.5">
      {title && (
        <div className="px-4">
          <h2 className="text-[11px] font-semibold uppercase tracking-[0.09em] text-text-muted">
            {title}
          </h2>
          {description && (
            <p className="text-xs text-text-muted mt-1">{description}</p>
          )}
        </div>
      )}
      {/* overflow stays visible so a row's dropdown can escape the card */}
      <Surface variant="card" className="overflow-visible">
        <div className="divide-y divide-divider">{children}</div>
      </Surface>
    </div>
  );
};
