import React from "react";

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?:
    | "primary"
    | "primary-soft"
    | "secondary"
    | "danger"
    | "danger-ghost"
    | "ghost";
  size?: "sm" | "md" | "lg";
}

export const Button: React.FC<ButtonProps> = ({
  children,
  className = "",
  variant = "primary",
  size = "md",
  ...props
}) => {
  const baseClasses =
    "focus-ring font-medium rounded-lg border transition-colors disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer";

  const variantClasses = {
    primary:
      "text-white bg-accent border-accent hover:bg-accent/80 hover:border-accent/80",
    "primary-soft":
      "text-text bg-accent/20 border-transparent hover:bg-accent/30",
    secondary:
      "bg-control border-control-border glass-specular hover:bg-accent/10 hover:border-accent",
    danger:
      "text-white bg-red-600 border-red-600 hover:bg-red-700 hover:border-red-700",
    "danger-ghost":
      "text-red-400 border-transparent hover:text-red-300 hover:bg-red-500/10",
    ghost:
      "text-current border-transparent hover:bg-control hover:border-hairline",
  };

  const sizeClasses = {
    sm: "px-2 py-1 text-xs",
    md: "px-4 py-[5px] text-sm",
    lg: "px-4 py-2 text-base",
  };

  return (
    <button
      className={`${baseClasses} ${variantClasses[variant]} ${sizeClasses[size]} ${className}`}
      {...props}
    >
      {children}
    </button>
  );
};
