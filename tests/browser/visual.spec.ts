import { test, expect } from '@playwright/test';

const article = '/learn/tour/';

test('footer closes the page on an ink band with a tree watermark', async ({ page }) => {
  await page.goto('/');
  const footer = page.locator('.site-footer');
  expect(await footer.evaluate(el => getComputedStyle(el).backgroundColor)).toBe('rgb(23, 28, 24)');
  await expect(footer.locator('.footer-mark')).toBeAttached();
  const inner = footer.locator('.footer-inner');
  expect(await inner.evaluate(el => getComputedStyle(el).maxWidth)).toBe('1240px');
});

test('eyebrows carry the branch tick', async ({ page }) => {
  await page.goto('/');
  const mask = await page.locator('.honest-status .eyebrow').evaluate(el =>
    getComputedStyle(el, '::before').maskImage || getComputedStyle(el, '::before').webkitMaskImage);
  expect(mask).toContain('data:image/svg+xml');
});

test('hero code panel sits in a tonal spotlight', async ({ page }) => {
  await page.goto('/');
  const wash = await page.locator('.hero-example').evaluate(el => getComputedStyle(el, '::before').backgroundImage);
  expect(wash).toContain('radial-gradient');
});

test('status panel carries the tree watermark', async ({ page }) => {
  await page.goto('/');
  const mark = page.locator('.status-mark');
  await expect(mark).toBeAttached();
  expect(parseFloat(await mark.evaluate(el => getComputedStyle(el).opacity))).toBeLessThan(0.1);
});

test('article margin contents is sticky and framed', async ({ page }) => {
  await page.goto(article);
  const sticky = page.viewportSize()!.width > 760;
  expect(await page.locator('.article-aside').evaluate(el => getComputedStyle(el).position)).toBe(sticky ? 'sticky' : 'static');
  const nav = page.locator('.article-aside nav');
  expect(await nav.evaluate(el => getComputedStyle(el).borderTopWidth)).toBe('1px');
  expect(await nav.evaluate(el => getComputedStyle(el).backgroundColor)).toBe('rgb(233, 236, 223)');
});

test('callout label is semibold sans', async ({ page }) => {
  await page.goto(article);
  const strong = page.locator('.callout strong').first();
  expect(await strong.evaluate(el => getComputedStyle(el).fontWeight)).toBe('600');
  expect(await strong.evaluate(el => getComputedStyle(el).fontFamily)).toContain('Ash Sans');
});

test('prose code blocks share the panel border', async ({ page }) => {
  await page.goto(article);
  const pre = page.locator('article.prose > pre').first();
  expect(await pre.evaluate(el => getComputedStyle(el).borderTopWidth)).toBe('1px');
  expect(await page.locator('article.prose .code-example pre').first().evaluate(el => getComputedStyle(el).borderTopWidth)).toBe('0px');
});
