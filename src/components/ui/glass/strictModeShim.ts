let applied = false;

/**
 * Neutralises the destructive half of the glass renderer's teardown, in
 * development only.
 *
 * `GlassWebGLRenderer.dispose()` ends with
 * `gl.getExtension("WEBGL_lose_context")?.loseContext()`. React.StrictMode's
 * dev-only mount → unmount → mount cycle disposes the renderer and then re-runs
 * the effect against the same canvas, which now hands back a permanently lost
 * context: shader compilation fails with `glass-webgl shader: null` and the
 * lens is dead for the component's lifetime.
 *
 * Upstream: https://github.com/samasante/liquid-glass/issues/2 (open as of
 * 0.1.1). Delete this file and its call site when a fixed release lands.
 *
 * We keep `React.StrictMode` in both entry points and make `getExtension`
 * return `null` for that one extension instead, so `loseContext()` is never
 * reached. Every other extension lookup is untouched, and production keeps real
 * GPU cleanup: the whole body is behind `import.meta.env.DEV` and is dropped
 * from the production bundle.
 */
export const applyStrictModeGlassShim = (): void => {
  if (!import.meta.env.DEV) return;
  if (applied || typeof window === "undefined") return;
  applied = true;

  const lostContextExtension = "WEBGL_lose_context";

  const patch = (proto: WebGLRenderingContext | WebGL2RenderingContext) => {
    const original = proto.getExtension;
    proto.getExtension = function patchedGetExtension(
      this: WebGLRenderingContext,
      name: string,
    ) {
      if (name === lostContextExtension) return null;
      return original.call(this, name);
    } as typeof proto.getExtension;
  };

  if (typeof WebGLRenderingContext !== "undefined") {
    patch(WebGLRenderingContext.prototype);
  }
  if (typeof WebGL2RenderingContext !== "undefined") {
    patch(WebGL2RenderingContext.prototype);
  }
};
