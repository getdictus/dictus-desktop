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
    await page.mouse.move(box.x + box.width * 0.65, box.y + box.height / 2);
    await page.mouse.down();
    await page.mouse.move(box.x + 2, box.y + box.height / 2, { steps: 10 });
    await page.mouse.up();

    await expect(value).not.toHaveText("65%");
    expect(Number((await value.innerText()).replace("%", ""))).toBeLessThan(15);
  });
});
