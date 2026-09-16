import React from "react";

/**
 * The five materials the design system recognises.
 *
 * - `base` — the app canvas. Flat, no border, nothing floats above it.
 * - `raised` — an inline control sitting on a card (chip, select trigger).
 * - `card` — a settings group or a hero panel. 16 px radius.
 * - `popover` — a dropdown, tooltip or toast. Floats, so it blurs harder.
 * - `overlay` — the recording pill. Always dark: it sits over the user's
 *   desktop rather than over the app canvas, so it does not follow the theme.
 */
export type SurfaceVariant = "base" | "raised" | "card" | "popover" | "overlay";

const VARIANT_CLASSES: Record<SurfaceVariant, string> = {
  base: "bg-background",
  raised:
    "bg-control border border-control-border rounded-lg glass-blur glass-specular",
  card: "bg-card border border-hairline rounded-2xl glass-blur",
  popover:
    "bg-floating border border-hairline rounded-xl glass-blur-floating glass-raised",
  overlay:
    "bg-[var(--surface-overlay)] rounded-[42px] glass-blur-floating shadow-[var(--overlay-rim)]",
};

/**
 * Returns the class string for a variant so surfaces that cannot render a
 * `<Surface>` — a portal root, a third-party menu — still read from the same
 * tokens instead of restating them.
 */
export const surfaceClasses = (
  variant: SurfaceVariant,
  className = "",
): string => `${VARIANT_CLASSES[variant]} ${className}`.trim();

export interface SurfaceProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: SurfaceVariant;
}

/**
 * The single source of truth for the app's surface material. Every variant
 * reads only design tokens, so a theme or fallback change lands in one place.
 */
export const Surface = React.forwardRef<HTMLDivElement, SurfaceProps>(
  ({ variant = "card", className = "", children, ...props }, ref) => (
    <div ref={ref} className={surfaceClasses(variant, className)} {...props}>
      {children}
    </div>
  ),
);

Surface.displayName = "Surface";
