import type { GlassOptics } from "@samasante/liquid-glass";

/**
 * The approved board's optical constants, verbatim. They come from the
 * library's own `examples/GlassSwitch.tsx`, `GlassSlider.tsx` and
 * `GlassNotification.tsx`, retuned for the Dictus material.
 *
 * Only three controls use the lens — the toggle thumb, the volume handle and
 * the sidebar selection capsule. Everything else in the app is CSS.
 */

export const SWITCH_BASE: Partial<GlassOptics> = {
  mapSize: 256,
  depth: 0.2,
  dispersion: 0.65,
  strength: 0.19,
  clipToShape: true,
  softEdge: true,
  curvature: 0.3,
  splay: 0.6,
  bend: 0.1,
  bendWidth: 0.06,
  frost: 0,
  specular: 1.2,
  sheenAngle: 45,
  glow: 0.05,
  glowSpread: 0.5,
  glowFalloff: 1.5,
  sheen: 0.45,
  sheenWidth: 2,
  sheenFalloff: 1.5,
  edgeShadow: "0 2px 6px rgba(0,0,0,0.16)",
  edgeInsetShadow: "0 -4px 10px rgba(0,0,0,0.12)",
};

export const SWITCH_LIGHT: Partial<GlassOptics> = {
  brightness: -0.02,
  sheenAngle: 30,
  specular: 1.5,
  glow: 0.4,
  glowSpread: 0.5,
  glowFalloff: 2,
  sheen: 1,
  sheenWidth: 1.5,
  sheenFalloff: 1,
};

export const SWITCH_DARK: Partial<GlassOptics> = {
  brightness: 0.12,
  glow: 0.4,
  sheen: 0.5,
};

export const SLIDER_BASE: Partial<GlassOptics> = {
  mapSize: 256,
  depth: 0.2,
  dispersion: 0.5,
  scaleX: 0.06,
  scaleY: 0.06,
  clipToShape: true,
  softEdge: true,
  curvature: 0.55,
  splay: 0.5,
  bend: 0.1,
  bendWidth: 0.05,
  frost: 0,
  brightness: 0.06,
  specular: 1.5,
  sheenAngle: 45,
  glow: 0.4,
  glowSpread: 0.5,
  glowFalloff: 1.5,
  sheen: 0,
  sheenWidth: 3,
  sheenFalloff: 1.5,
  edgeShadow: "0 2px 6px rgba(0,0,0,0.16)",
  edgeInsetShadow: "0 -4px 10px rgba(0,0,0,0.12)",
};

export const SLIDER_LIGHT: Partial<GlassOptics> = {
  restEdgeShadow: "0 1.333px 5.333px rgba(46,15,15,0.12)",
  scaleX: 0.1,
  scaleY: 0.1,
  brightness: -0.02,
  sheenAngle: 30,
  glowFalloff: 2,
  sheen: 1,
  sheenWidth: 1,
  sheenFalloff: 1,
};

export const SLIDER_DARK: Partial<GlassOptics> = {
  restEdgeShadow: "0 1.333px 5.333px rgba(0,0,0,0.5)",
  scaleX: 0.133,
  scaleY: 0.135,
  brightness: 0.12,
  sheenAngle: 45,
  glowFalloff: 1.5,
  sheen: 0.5,
  sheenWidth: 1,
  sheenFalloff: 1.5,
};

export const PANEL_LENS: Partial<GlassOptics> = {
  mapSize: 256,
  clipToShape: true,
  softEdge: true,
  depth: 1,
  curvature: 0.5,
  dispersion: 0.6,
  strength: 0.17,
  bend: 0.7,
  bendWidth: 0.12,
  frost: 4,
  brightness: 0.04,
  specular: 1.3,
  sheenAngle: 50,
  glow: 0.32,
  glowSpread: 1,
  glowFalloff: 1,
  sheen: 1.3,
  sheenWidth: 3,
};

/** The sidebar capsule is PANEL_LENS softened: less frost, a wider lip. */
export const NAV_CAPSULE_LENS: Partial<GlassOptics> = {
  ...PANEL_LENS,
  frost: 2,
  bend: 0.7,
  bendWidth: 0.14,
  strength: 0.14,
  dispersion: 0.5,
  sheen: 1,
  sheenWidth: 2,
};

export const NAV_CAPSULE_LIGHT: Partial<GlassOptics> = { brightness: 0.14 };
export const NAV_CAPSULE_DARK: Partial<GlassOptics> = { brightness: 0.06 };

/** Merge a base look with its per-theme overrides. */
export const opticsFor = (
  base: Partial<GlassOptics>,
  light: Partial<GlassOptics>,
  dark: Partial<GlassOptics>,
  isDark: boolean,
): Partial<GlassOptics> => ({
  ...base,
  ...(isDark ? dark : light),
  // The board sets a dark specular in light theme and a bright one in dark.
  sheenDark: !isDark,
});
