/**
 * Why the lens is or is not rendering, in the order the reasons are checked.
 *
 * `disabled` and `reduced-transparency` are deliberate: the first is the
 * reduced-effects escape hatch (`<html data-glass="off">`), the second follows
 * the OS accessibility setting.
 */
export type GlassSupport =
  | "supported"
  | "disabled"
  | "reduced-transparency"
  | "no-displacement-map"
  | "no-webgl2";

let cached: GlassSupport | null = null;

const probe = (): GlassSupport => {
  if (typeof window === "undefined" || typeof document === "undefined") {
    return "no-displacement-map";
  }
  if (document.documentElement.dataset.glass === "off") return "disabled";
  if (window.matchMedia("(prefers-reduced-transparency: reduce)").matches) {
    return "reduced-transparency";
  }
  // The lens the app uses refracts a DOM copy through an SVG
  // `feDisplacementMap`, so that is the gate that actually matters here.
  if (typeof SVGFEDisplacementMapElement === "undefined") {
    return "no-displacement-map";
  }
  // The same component switches to a WebGL2 renderer for video and canvas
  // sources. Nothing in Dictus uses those today, but a webview without WebGL2
  // cannot run the effect at all, so it takes the CSS material everywhere
  // rather than losing the lens on one screen only. WebKitGTK is the risk case.
  try {
    if (!document.createElement("canvas").getContext("webgl2")) {
      return "no-webgl2";
    }
  } catch {
    return "no-webgl2";
  }
  return "supported";
};

/** Probed once per page: none of these can change without a reload. */
export const glassSupport = (): GlassSupport => (cached ??= probe());

export const supportsGlassLens = (): boolean => glassSupport() === "supported";

/** Test hook — forget the probe so a test can flip `data-glass` and re-read. */
export const resetGlassSupport = (): void => {
  cached = null;
};
