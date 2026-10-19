import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { readFileSync } from 'node:fs';

const publication = JSON.parse(readFileSync('.cache/ash-spec/publication.json', 'utf8'));
const paths = ['/docs/decisions/', '/docs/decisions/ad-0003-formal-notation-fences/', '/docs/decisions/ad-0012-proof-development-feedback-and-build-reuse/', '/docs/decisions/template/'];

test('decisions are discoverable and linked locally with their original status and source', async ({ page }) => {
  await page.goto('/docs/');
  await page.getByRole('link', { name: 'architectural decisions', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'Architectural decisions', exact: true })).toBeVisible();
  await page.locator('article').getByRole('link', { name: 'AD-0004: Small-Step Semantic Method', exact: true }).click();
  await expect(page.locator('article')).toContainText('Status:');
  await expect(page.locator('.decision-status')).toContainText('non-language choices');
  await expect(page.locator('.spec-source a')).toHaveAttribute('href', `https://github.com/dikini/ash-lang/blob/${publication.revision}/docs/decisions/AD-0004-small-step-semantic-method.md`);
  await page.locator('article a[href="/docs/design/dn-0001-reboot-foundations/"]').click();
  await expect(page.locator('.spec-source a')).toHaveAttribute('href', `https://github.com/dikini/ash-lang/blob/${publication.revision}/docs/design/DN-0001-reboot-foundations.md`);
  await page.locator('article').getByRole('link', { name: 'AD-0004: Small-Step Semantic Method', exact: true }).click();
  await page.getByRole('navigation', { name: 'Adjacent architectural decisions', exact: true }).locator('a[rel="next"]').click();
  await expect(page).toHaveURL(/\/docs\/decisions\/ad-0005-parser-architecture\/$/);
  await page.getByRole('navigation', { name: 'Architectural decisions', exact: true }).getByRole('link', { name: 'All architectural decisions', exact: true }).click();
  await expect(page).toHaveURL(/\/docs\/decisions\/$/);
});

test('decision notation and authoring template work without JavaScript', async ({ browser, baseURL }) => {
  const context = await browser.newContext({ javaScriptEnabled: false, viewport: { width: 390, height: 844 } });
  try {
    const page = await context.newPage();
    await page.goto(`${baseURL}/docs/decisions/ad-0003-formal-notation-fences/`);
    await expect(page.locator('.rrd-diagram svg').first()).toBeVisible();
    await expect(page.locator('.sequent-proof')).toBeVisible();
    await page.getByText('Grammar source (EBNF)', { exact: true }).click();
    await expect(page.locator('.spec-grammar details code')).toContainText('expression ::= term');
    await page.getByRole('navigation', { name: 'Architectural decisions', exact: true }).getByRole('link').click();
    await page.locator('article a[href="/docs/decisions/template/"]').first().click();
    await expect(page.getByRole('heading', { name: 'AD-NNNN: Short title', exact: true })).toBeVisible();
    await expect(page.getByRole('navigation', { name: 'Adjacent architectural decisions', exact: true })).toHaveCount(0);
  } finally { await context.close(); }
});

test('decision pages reflow at 320px and diagrams support keyboard scrolling', async ({ page }) => {
  await page.setViewportSize({ width: 320, height: 800 });
  for (const path of paths) {
    await page.goto(path);
    await page.addStyleTag({ content: '* { line-height: 1.5 !important; letter-spacing: .12em !important; word-spacing: .16em !important; } p { margin-bottom: 2em !important; }' });
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth), path).toBe(true);
  }
  await page.goto('/docs/decisions/ad-0003-formal-notation-fences/');
  const diagram = page.locator('.rrd-diagram').first();
  await diagram.focus();
  await expect(diagram).toBeFocused();
  await page.keyboard.press('ArrowRight');
  await expect.poll(() => diagram.evaluate(element => element.scrollLeft)).toBeGreaterThan(0);
  expect(await page.locator('article.prose > p').first().evaluate(element => parseFloat(getComputedStyle(element).marginBottom))).toBeGreaterThanOrEqual(20);
});

test('all architectural decision pages pass automated WCAG AA checks', async ({ page }) => {
  for (const { route } of publication.decisions.pages) {
    await page.goto(route);
    const result = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa']).analyze();
    expect(result.violations, route).toEqual([]);
  }
});

test('search finds architectural decisions', async ({ page }) => {
  await page.goto('/search/');
  await page.getByRole('textbox', { name: /search/i }).fill('ordered binder authority');
  await expect(page.locator('.pagefind-ui__result-link[href*="/docs/decisions/ad-0009-ordered-resolved-global-binder-authority/"]').first()).toBeVisible({ timeout: 15_000 });
});
