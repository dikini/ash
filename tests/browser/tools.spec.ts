import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

test('documentation links to the ash guide with contents and source provenance', async ({ page, isMobile }) => {
  await page.goto('/docs/');
  await page.getByRole('link', { name: 'ash tool guide', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'The ash Executable', exact: true })).toBeVisible();
  await expect(page.locator('article pre code').first()).toContainText('ash [--message-format human|short|json]');
  const toolSource = await page.locator('.spec-source a').getAttribute('href');
  expect(toolSource).toMatch(/\/blob\/[a-f0-9]{40}\/docs\/tools\/ash\.md$/);
  if (!isMobile) {
    await page.getByRole('navigation', { name: 'On this page', exact: true }).getByRole('link', { name: 'Diagnostic formats and locales', exact: true }).click();
    await expect(page).toHaveURL(/#diagnostic-formats-and-locales$/);
  }
  await page.getByRole('link', { name: 'Read the language specification ↗', exact: true }).click();
  const specSource = await page.locator('.spec-source a').getAttribute('href');
  expect(specSource?.match(/\/blob\/([a-f0-9]{40})\//)?.[1]).toBe(toolSource?.match(/\/blob\/([a-f0-9]{40})\//)?.[1]);
  await page.locator('article a[href="/docs/tools/ash/"]').first().click();
  await expect(page).toHaveURL(/\/docs\/tools\/ash\/$/);
});

test('tool overview and guide work without JavaScript', async ({ browser, baseURL }) => {
  const context = await browser.newContext({ javaScriptEnabled: false, viewport: { width: 390, height: 844 } });
  try {
    const page = await context.newPage();
    await page.goto(`${baseURL}/docs/tools/`);
    await expect(page.getByRole('heading', { name: 'Ash Tools', exact: true })).toBeVisible();
    await page.locator('article a[href="/docs/tools/ash/"]').click();
    await expect(page.locator('article pre code').first()).toContainText('--message-format');
    await page.getByRole('navigation', { name: 'Tool documentation', exact: true }).getByRole('link', { name: 'Overview', exact: true }).click();
    await page.locator('article a[href="/spec/"]').click();
    await expect(page.getByRole('heading', { name: 'Ash Specification', exact: true })).toBeVisible();
  } finally { await context.close(); }
});

test('tool docs reflow at 320px with keyboard-scrollable examples', async ({ page }) => {
  await page.setViewportSize({ width: 320, height: 800 });
  for (const path of ['/docs/tools/', '/docs/tools/ash/']) {
    await page.goto(path);
    await page.addStyleTag({ content: '* { line-height: 1.5 !important; letter-spacing: .12em !important; word-spacing: .16em !important; } p { margin-bottom: 2em !important; }' });
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth), path).toBe(true);
  }
  const example = page.locator('article pre').first();
  await expect(example).toHaveAttribute('tabindex', '0');
  await example.focus();
  await expect(example).toBeFocused();
  await page.keyboard.press('ArrowRight');
  await expect.poll(() => example.evaluate(element => element.scrollLeft)).toBeGreaterThan(0);
});

test('tool docs pass automated WCAG AA checks', async ({ page }) => {
  for (const path of ['/docs/tools/', '/docs/tools/ash/']) {
    await page.goto(path);
    const result = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa']).analyze();
    expect(result.violations, path).toEqual([]);
  }
});

test('site search finds the rendered tool guide', async ({ page }) => {
  await page.goto('/search/');
  await page.getByRole('textbox', { name: /search/i }).fill('diagnostic formats locales');
  await expect(page.locator('.pagefind-ui__result-link[href*="/docs/tools/ash/"]').first()).toBeVisible({ timeout: 15_000 });
});
