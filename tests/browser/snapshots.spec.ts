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
    // Ubuntu's font metrics differ from workstation Linux on narrow layouts.
    const snapshot = process.env.ASH_VISUAL_BASELINE === 'ubuntu' && test.info().project.name === 'mobile'
      ? ['ubuntu', `${name}.png`] : `${name}.png`;
    await expect(page).toHaveScreenshot(snapshot, { fullPage, maxDiffPixelRatio: 0.02 });
  });
}
