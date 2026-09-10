import React from "react";
import { Glass, type GlassOptics } from "@samasante/liquid-glass";
import { applyStrictModeGlassShim } from "./strictModeShim";
import { supportsGlassLens } from "./capabilities";

// Runs once, before any lens is constructed. No-op in production.
applyStrictModeGlassShim();

export interface GlassLensProps
  extends Omit<React.HTMLAttributes<HTMLDivElement>, "children"> {
  /**
   * Solid colour of the surface directly under the lens. The design board can
   * hard-code this per control because it knows exactly what sits behind each
   * one; in the app the caller passes the surface it is placed on, so the same
   * control works on a card, on the sidebar or on the canvas.
   */
  behind: string;
  width: number;
  height: number;
  /** Defaults to a pill: half the shorter side. */
  radius?: number;
  optics?: Partial<GlassOptics>;
  /**
   * What the lens refracts. Defaults to a flat fill of `behind`, which is what
   * every current caller sits on.
   */
  refract?: React.ReactNode;
  /** Extra CSS for the fallback material only. */
  fallbackStyle?: React.CSSProperties;
  children?: React.ReactNode;
}

/** True while the OS asks for a dark appearance. */
export const usePrefersDark = (): boolean => {
  const [isDark, setIsDark] = React.useState(
    () =>
      typeof window !== "undefined" &&
      window.matchMedia("(prefers-color-scheme: dark)").matches,
  );

  React.useEffect(() => {
    const query = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = (event: MediaQueryListEvent) => setIsDark(event.matches);
    query.addEventListener("change", onChange);
    setIsDark(query.matches);
    return () => query.removeEventListener("change", onChange);
  }, []);

  return isDark;
};

/**
 * The only place `@samasante/liquid-glass` is imported. Feature screens use
 * this wrapper, so the effect can be reduced, replaced or removed without
 * touching a single screen.
 *
 * When the lens cannot render — no WebGL2, no displacement-map filter, reduced
 * transparency, or the `data-glass="off"` escape hatch — it falls back to a CSS
 * material of the same size and shape rather than disappearing.
 */
export const GlassLens: React.FC<GlassLensProps> = ({
  behind,
  width,
  height,
  radius,
  optics,
  refract,
  className = "",
  style,
  fallbackStyle,
  children,
  ...rest
}) => {
  const cornerRadius = radius ?? Math.min(width, height) / 2;
  const geometry: React.CSSProperties = {
    width,
    height,
    borderRadius: cornerRadius,
  };

  if (!supportsGlassLens()) {
    return (
      <div
        className={`glass-specular ${className}`.trim()}
        style={{
          ...geometry,
          background: behind,
          ...style,
          ...fallbackStyle,
        }}
        {...rest}
      >
        {children}
      </div>
    );
  }

  return (
    <Glass
      className={className}
      style={{ ...geometry, ...style }}
      width={width}
      height={height}
      radius={cornerRadius}
      behind={behind}
      optics={optics}
      // Chromium-only supersample; the library forces it back to 1 in WebKit.
      filterResolution={2}
      refract={
        refract ?? (
          <div style={{ ...geometry, background: behind, borderRadius: 0 }} />
        )
      }
      {...rest}
    >
      {children}
    </Glass>
  );
};
