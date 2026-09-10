/**
 * The Dictus glass boundary.
 *
 * `@samasante/liquid-glass` is reachable only from inside this directory. No
 * feature screen imports it, so the effect can be retuned, downgraded to the
 * CSS material or dropped entirely without touching a screen.
 */
export { GlassLens, usePrefersDark } from "./GlassLens";
export type { GlassLensProps } from "./GlassLens";
export {
  glassSupport,
  supportsGlassLens,
  resetGlassSupport,
} from "./capabilities";
export type { GlassSupport } from "./capabilities";
export {
  SWITCH_BASE,
  SWITCH_LIGHT,
  SWITCH_DARK,
  SLIDER_BASE,
  SLIDER_LIGHT,
  SLIDER_DARK,
  PANEL_LENS,
  NAV_CAPSULE_LENS,
  NAV_CAPSULE_LIGHT,
  NAV_CAPSULE_DARK,
  opticsFor,
  LENS_BEHIND,
  behindFor,
} from "./optics";
