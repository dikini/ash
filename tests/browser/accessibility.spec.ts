import { test, expect } from '@playwright/test';
import { readdirSync } from 'node:fs';
import AxeBuilder from '@axe-core/playwright';
const paths = ['/', '/discover/', '/learn/', '/learn/tour/', '/docs/install/', '/docs/', '/study/', '/status/', '/updates/', '/diaries/', '/contribute/', '/search/', '/study/latest/', '/study/latest/learn-what-is-an-ash-program.html'];
const bookPages = readdirSync('dist/study/latest').filter(file => file.endsWith('.html')).map(file => `/study/latest/${file}`);
for (const path of [...new Set([...paths, ...bookPages])]) {
  test(`site and book meet automated WCAG 2.2 AA checks: ${path}`, async ({ page }) => {
    const response = await page.goto(path);
    expect(response?.status(), path).toBe(200);
    const results = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa']).analyze();
    expect(results.violations, path).toEqual([]);
  });
}
test('keyboard users can skip navigation on the site and book', async ({ page }) => {
  for (const path of ['/', '/study/latest/learn-what-is-an-ash-program.html']) {
    await page.goto(path);
    await page.keyboard.press('Tab');
    const skip = page.getByRole('link', { name: 'Skip to content', exact: true });
    await expect(skip).toBeFocused();
    await page.keyboard.press('Enter');
    await expect(page.getByRole('main')).toBeFocused();
  }
});
test('reading reflows at 320 pixels and with increased text spacing', async ({ page }) => {
  await page.setViewportSize({ width: 320, height: 800 });
  for (const path of paths) {
    await page.goto(path);
    await page.addStyleTag({ content: '* { line-height: 1.5 !important; letter-spacing: .12em !important; word-spacing: .16em !important; } p { margin-bottom: 2em !important; }' });
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth), path).toBe(true);
  }
});
