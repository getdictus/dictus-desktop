import { test, expect } from "@playwright/test";

/**
 * Screenshot coverage for the shared component specimen, in both themes and at
 * the app's real 680 x 570 window size. The images land in test-results/ and
 * are the before/after evidence attached to the PR.
 */
for (const scheme of ["light", "dark"] as const) {
  test(`component specimen renders in ${scheme}`, async ({ page }) => {
    await page.emulateMedia({ colorScheme: scheme });
    await page.setViewportSize({ width: 680, height: 570 });
    await page.goto("/tests/specimen.html");

    // The card material has to resolve to the theme's value, not a fallback.
    const card = page.locator(".bg-card").first();
    await expect(card).toBeVisible();

    await expect(page).toHaveScreenshot(`specimen-${scheme}.png`, {
      fullPage: true,
      maxDiffPixelRatio: 0.02,
    });
  });
}
