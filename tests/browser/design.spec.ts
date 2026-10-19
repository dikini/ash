import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

const paths = ['/docs/design/', '/docs/design/dn-0001-reboot-foundations/', '/docs/design/mini-ash/first-stratum/grammar/', '/docs/design/provider-experiments/semantics/'];

test('all reference groups retain readable paragraph spacing', async ({ page }) => {
  for (const path of ['/spec/', '/docs/tools/ash/', '/docs/design/dn-0001-reboot-foundations/']) {
    await page.goto(path);
    const paragraph = page.locator('article.prose > p').first();
    expect(await paragraph.evaluate(element => parseFloat(getComputedStyle(element).marginBottom)), path).toBeGreaterThanOrEqual(20);
  }
});

test('design notes link from documentation with informative status and exact sources', async ({ page }) => {
  await page.goto('/docs/');
  await page.getByRole('link', { name: 'design notes', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'Design notes', exact: true })).toBeVisible();
  await page.locator('article a[href="/docs/design/dn-0001-reboot-foundations/"]').first().click();
  await expect(page.locator('.design-status')).toContainText('Informative design work');
  await expect(page.locator('article')).toContainText('Status: exploring');
  await expect(page.locator('.spec-source a')).toHaveAttribute('href', /\/blob\/[a-f0-9]{40}\/docs\/design\/DN-0001-reboot-foundations\.md$/);
  await page.getByRole('navigation', { name: 'Adjacent design notes', exact: true }).locator('a[rel="next"]').click();
  await expect(page).toHaveURL(/\/docs\/design\/dn-0002-/);
  await page.getByRole('navigation', { name: 'Design notes', exact: true }).getByRole('link', { name: 'All design notes', exact: true }).click();
  await page.getByText(/^Supporting documents \(/).click();
  await page.locator('.design-support a[href="/docs/design/mini-ash/first-stratum/grammar/"]').click();
  await expect(page.locator('.rrd-diagram svg').first()).toBeVisible();
});

test('design overview and nested evidence work without JavaScript', async ({ browser, baseURL }) => {
  const context = await browser.newContext({ javaScriptEnabled: false, viewport: { width: 390, height: 844 } });
  try {
    const page = await context.newPage();
    await page.goto(`${baseURL}/docs/design/`);
    await page.getByText(/^Supporting documents \(/).click();
    await page.locator('.design-support a[href="/docs/design/mini-ash/first-stratum/grammar/"]').click();
    await expect(page.locator('.rrd-diagram svg').first()).toBeVisible();
    await page.locator('.spec-grammar details').first().getByText('Grammar source (EBNF)', { exact: true }).click();
    await expect(page.locator('.spec-grammar details pre').first()).toBeVisible();
    await page.getByRole('navigation', { name: 'Design notes', exact: true }).getByRole('link').click();
    await expect(page).toHaveURL(/\/docs\/design\/$/);
  } finally { await context.close(); }
});

test('design pages reflow at 320px and retain keyboard access to wide content', async ({ page }) => {
  await page.setViewportSize({ width: 320, height: 800 });
  for (const path of paths) {
    await page.goto(path);
    await page.addStyleTag({ content: '* { line-height: 1.5 !important; letter-spacing: .12em !important; word-spacing: .16em !important; } p { margin-bottom: 2em !important; }' });
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth), path).toBe(true);
  }
  const source = page.locator('.spec-notation-source pre');
  await source.focus();
  await expect(source).toBeFocused();
  await page.keyboard.press('ArrowRight');
  await expect.poll(() => source.evaluate(element => element.scrollLeft)).toBeGreaterThan(0);
});

test('representative design pages pass automated WCAG AA checks', async ({ page }) => {
  for (const path of paths) {
    await page.goto(path);
    const result = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa']).analyze();
    expect(result.violations, path).toEqual([]);
  }
});

test('site search finds design notes and nested supporting material', async ({ page }) => {
  await page.goto('/search/');
  const search = page.getByRole('textbox', { name: /search/i });
  await search.fill('reboot foundations');
  await expect(page.locator('.pagefind-ui__result-link[href*="/docs/design/dn-0001-reboot-foundations/"]').first()).toBeVisible({ timeout: 15_000 });
  await search.fill('T-Resource-Instance-Candidate');
  await expect(page.locator('.pagefind-ui__result-link[href*="/docs/design/provider-experiments/semantics/"]').first()).toBeVisible({ timeout: 15_000 });
});
