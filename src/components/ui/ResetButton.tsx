import React from "react";
import ResetIcon from "../icons/ResetIcon";

interface ResetButtonProps {
  onClick: () => void;
  disabled?: boolean;
  className?: string;
  ariaLabel?: string;
  children?: React.ReactNode;
}

export const ResetButton: React.FC<ResetButtonProps> = React.memo(
  ({ onClick, disabled = false, className = "", ariaLabel, children }) => (
    <button
      type="button"
      aria-label={ariaLabel}
      className={`focus-ring p-1 rounded-lg border border-transparent transition-all duration-150 ${
        disabled
          ? "opacity-50 cursor-not-allowed text-text-faint"
          : "text-text-faint hover:text-accent hover:bg-accent/10 active:bg-accent/20 active:translate-y-[1px] hover:cursor-pointer hover:border-accent"
      } ${className}`}
      onClick={onClick}
      disabled={disabled}
    >
      {children ?? <ResetIcon />}
    </button>
  ),
);
