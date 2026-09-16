import { test, expect, type Page } from "@playwright/test";

const HARNESS = "/tests/glass-harness.html";

/** The lens builds an SVG filter chain; a live lens has a displacement map. */
const displacementMapCount = (page: Page) =>
  page.locator("feDisplacementMap").count();

test.describe("glass lens under React.StrictMode", () => {
  test("survives mount, unmount and remount", async ({ page }) => {
    const failures: string[] = [];
    page.on("console", (message) => {
      if (message.type() === "error") failures.push(message.text());
      // The upstream symptom is a warning, not an error.
      if (message.text().includes("glass-webgl shader")) {
        failures.push(message.text());
      }
    });
    page.on("pageerror", (error) => failures.push(error.message));

    await page.goto(HARNESS);

    // StrictMode already ran mount -> unmount -> mount before this resolves.
    const lens = page.getByTestId("lens");
    await expect(lens).toBeVisible();
    expect(await displacementMapCount(page)).toBeGreaterThan(0);

    // A second, explicit cycle.
    await page.click("#toggle");
    await expect(lens).toHaveCount(0);
    await page.click("#toggle");
    await expect(lens).toBeVisible();

    expect(await displacementMapCount(page)).toBeGreaterThan(0);
    expect(failures).toEqual([]);

    // The dev shim is what makes the remount survivable: the renderer can no
    // longer reach loseContext(). Production keeps real GPU cleanup, verified
    // by grepping the built bundle instead.
    const loseContextReachable = await page.evaluate(() => {
      const gl = document.createElement("canvas").getContext("webgl2");
      return gl ? gl.getExtension("WEBGL_lose_context") !== null : null;
    });
    expect(loseContextReachable).toBe(false);
  });

  test("the effects-off escape hatch renders the CSS material", async ({
    page,
  }) => {
    await page.goto(`${HARNESS}?glass=off`);

    await expect(page.locator("#support")).toHaveAttribute(
      "data-support",
      "disabled",
    );

    // Same box, same shape, no filter chain.
    const lens = page.getByTestId("lens");
    await expect(lens).toBeVisible();
    const box = await lens.boundingBox();
    expect(box?.width).toBe(20);
    expect(box?.height).toBe(20);
    expect(await displacementMapCount(page)).toBe(0);
  });
});
