import { test, expect } from '@playwright/test';

const cases = [
  { name: 'home', path: '/', fullPage: true },
  { name: 'tour-hero', path: '/learn/tour/', fullPage: false },
  { name: 'tour-body', path: '/learn/tour/#effects-and-handlers', fullPage: false },
  { name: 'status-hero', path: '/status/', fullPage: false },
];

for (const { name, path, fullPage } of cases) {
  test(`${name} matches visual baseline`, async ({ page }) => {
    await page.emulateMedia({ reducedMotion: 'reduce' });
    await page.goto(path, { waitUntil: 'networkidle' });
    await expect(page).toHaveScreenshot(`${name}.png`, { fullPage, maxDiffPixelRatio: 0.02 });
  });
}
