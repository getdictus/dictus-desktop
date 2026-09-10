import { test, expect } from "@playwright/test";

const SPECIMEN = "/tests/specimen.html";

test.describe("glass controls stay operable", () => {
  test("the volume handle can be dragged", async ({ page }) => {
    await page.goto(SPECIMEN);

    const value = page.getByTestId("volume-value");
    await expect(value).toHaveText("65%");

    const track = page.getByTestId("volume-track");
    const box = await track.boundingBox();
    if (!box) throw new Error("no slider track");

    // Grab the handle where it currently sits and drag it to the far left.
    await page.mouse.move(box.x + box.width * 0.6, box.y + box.height / 2);
    await page.mouse.down();
    await page.mouse.move(box.x + 2, box.y + box.height / 2, { steps: 10 });
    await page.mouse.up();

    await expect(value).not.toHaveText("65%");
    expect(Number((await value.innerText()).replace("%", ""))).toBeLessThan(15);
  });

  test("the toggle flips on click and on keyboard", async ({ page }) => {
    await page.goto(SPECIMEN);

    const host = page.getByTestId("toggle-push-to-talk");
    const control = host.locator("[role='switch']");
    await expect(control).toBeChecked();

    await host.click();
    await expect(control).not.toBeChecked();

    await control.focus();
    await page.keyboard.press("Space");
    await expect(control).toBeChecked();
  });

  // 1 covers the plain case; 1.25 and 1.5 are where a fractional layout box
  // would show up, and the lens rounds its filter region to whole pixels.
  for (const scale of [1, 1.25, 1.5, 2]) {
    test(`the nav capsule tracks the active item at ${scale}x`, async ({
      browser,
    }) => {
      const context = await browser.newContext({ deviceScaleFactor: scale });
      const page = await context.newPage();
      await page.goto(SPECIMEN);

      const capsule = page.getByTestId("nav-capsule");
      for (const id of ["general", "models", "advanced", "history", "about"]) {
        const item = page.getByTestId(`nav-item-${id}`);
        await item.click();
        await page.waitForTimeout(600);
        const itemBox = await item.boundingBox();
        const capsuleBox = await capsule.boundingBox();
        if (!itemBox || !capsuleBox) throw new Error(`no box for ${id}`);
        expect(Math.abs(capsuleBox.y - itemBox.y)).toBeLessThan(1);
        expect(Math.abs(capsuleBox.height - itemBox.height)).toBeLessThan(1);
      }

      // The lens rounds its filter region to integer px, so a fractional box
      // would land the rim about a pixel off. Assert the box is whole.
      const box = await capsule.boundingBox();
      expect(box!.height % 1).toBeCloseTo(0, 5);
      expect(box!.width % 1).toBeCloseTo(0, 5);
      await context.close();
    });
  }

  test("the nav capsule tracks the active item exactly", async ({ page }) => {
    await page.goto(SPECIMEN);

    const capsule = page.getByTestId("nav-capsule");

    // Walk down the list. The last item is where a hard-coded stride drifts.
    for (const id of ["general", "models", "advanced", "history", "about"]) {
      const item = page.getByTestId(`nav-item-${id}`);
      await item.click();
      await expect(item).toHaveAttribute("aria-current", "page");
      // Let the slide settle before measuring.
      await page.waitForTimeout(600);

      const itemBox = await item.boundingBox();
      const capsuleBox = await capsule.boundingBox();
      if (!itemBox || !capsuleBox) throw new Error(`no box for ${id}`);

      expect(Math.abs(capsuleBox.y - itemBox.y)).toBeLessThan(1);
      expect(Math.abs(capsuleBox.x - itemBox.x)).toBeLessThan(1);
      expect(Math.abs(capsuleBox.height - itemBox.height)).toBeLessThan(1);
      expect(Math.abs(capsuleBox.width - itemBox.width)).toBeLessThan(1);
    }
  });

  // French is the reported case: "Post-traitement" was rendering as
  // "Post-traite…". Czech and Portuguese still ellipsize; see the PR.
  for (const lang of ["en", "fr", "de", "ru"]) {
    test(`nav labels are not truncated in ${lang}`, async ({ page }) => {
      await page.goto(`${SPECIMEN}?lang=${lang}`);

      const labels = page.locator("[data-testid^='nav-item-'] p");
      const count = await labels.count();
      expect(count).toBe(7);

      for (let i = 0; i < count; i += 1) {
        const label = labels.nth(i);
        const overflow = await label.evaluate(
          (node) => node.scrollWidth - node.clientWidth,
        );
        expect(
          overflow,
          `"${await label.innerText()}" overflows by ${overflow}px`,
        ).toBeLessThanOrEqual(0);
      }
    });
  }
});
