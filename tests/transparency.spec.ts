import { test, expect, type Page } from "@playwright/test";

const SPECIMEN = "/tests/specimen.html";

const tokens = (page: Page) =>
  page.evaluate(() => {
    const style = getComputedStyle(document.documentElement);
    const read = (name: string) => style.getPropertyValue(name).trim();
    return {
      canvas: read("--glass-canvas"),
      card: read("--surface-card"),
      sidebar: read("--surface-sidebar"),
      blur: read("--blur-surface"),
    };
  });

/** The transparent canvas is macOS-only, and main.tsx stamps the platform.
 *  The specimen takes it from the URL: an init script would run before
 *  documentElement exists. */
const url = (platform: string, extra = "") =>
  `${SPECIMEN}?platform=${platform}${extra}`;

test.describe("macOS transparent canvas", () => {
  test("windows and linux are untouched", async ({ page }) => {
    for (const platform of ["windows", "linux"]) {
      await page.goto(url(platform));
      const t = await tokens(page);
      expect(t.canvas).not.toBe("transparent");
      expect(t.blur).toContain("blur(");
      // The card keeps the translucent value the board specified.
      expect(t.card).toContain("rgba");
    }
  });

  test("macOS clears the canvas and keeps text surfaces opaque", async ({
    page,
  }) => {
    await page.goto(url("macos"));
    const t = await tokens(page);

    expect(t.canvas).toBe("transparent");
    // Opaque: no alpha channel anywhere on a surface that carries text.
    expect(t.card).not.toContain("rgba");
    expect(t.blur).toBe("none");
    // The sidebar is the one surface left translucent.
    expect(t.sidebar).toContain("rgba");
  });

  // Playwright 1.58 cannot emulate prefers-reduced-transparency, so this
  // asserts the rule exists and is scoped correctly rather than that the
  // browser honours it. Toggling the macOS setting is on the hand-test list.
  test("the reduced-transparency rule is present and scoped to macOS", async ({
    page,
  }) => {
    await page.goto(url("macos"));

    const declarations = await page.evaluate(() => {
      const found: Record<string, string> = {};
      const walk = (rules: CSSRuleList) => {
        for (const rule of Array.from(rules)) {
          if (rule instanceof CSSMediaRule) {
            if (rule.conditionText.includes("prefers-reduced-transparency")) {
              for (const inner of Array.from(rule.cssRules)) {
                if (
                  inner instanceof CSSStyleRule &&
                  inner.selectorText.includes('[data-platform="macos"]')
                ) {
                  found["--glass-canvas"] =
                    inner.style.getPropertyValue("--glass-canvas");
                  found["--surface-sidebar"] =
                    inner.style.getPropertyValue("--surface-sidebar");
                }
              }
            }
            walk(rule.cssRules);
          }
        }
      };
      for (const sheet of Array.from(document.styleSheets)) {
        try {
          walk(sheet.cssRules);
        } catch {
          // cross-origin sheet, not ours
        }
      }
      return found;
    });

    expect(declarations["--glass-canvas"]).toBe("var(--surface-opaque)");
    expect(declarations["--surface-sidebar"]).toBe("var(--surface-opaque)");
  });

  test("the effects-off hatch also paints it back in", async ({ page }) => {
    await page.goto(url("macos", "&glass=off"));

    const t = await tokens(page);
    expect(t.canvas).not.toBe("transparent");
    expect(t.sidebar).not.toContain("rgba");
  });
});
