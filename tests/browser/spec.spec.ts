import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

const paths = ['/spec/', '/spec/lexical-structure/', '/spec/syntax/', '/spec/name-resolution/', '/spec/static-semantics/', '/spec/elaboration/', '/spec/dynamic-semantics/'];

test('documentation leads to rendered specs and chapter navigation', async ({ page }) => {
  await page.goto('/docs/');
  await page.getByRole('link', { name: 'Ash specification', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'Ash Specification', exact: true })).toBeVisible();
  await page.getByRole('navigation', { name: 'Specification chapters', exact: true }).getByRole('link', { name: 'Syntax', exact: true }).click();
  await expect(page.locator('.rrd-diagram svg').first()).toBeVisible();
  await page.getByRole('navigation', { name: 'Adjacent specification chapters', exact: true }).getByRole('link', { name: /Name Resolution/ }).click();
  await expect(page).toHaveURL(/\/spec\/name-resolution\//);
});

test('specification diagrams and chapter links work without JavaScript', async ({ browser, baseURL }) => {
  const context = await browser.newContext({ javaScriptEnabled: false, viewport: { width: 390, height: 844 } });
  try {
    const page = await context.newPage();
    await page.goto(`${baseURL}/spec/syntax/`);
    await expect(page.locator('.rrd-diagram svg').first()).toBeVisible();
    await page.getByRole('navigation', { name: 'Specification chapters', exact: true }).getByRole('link', { name: 'Static Semantics', exact: true }).click();
    await expect(page.locator('.sequent-proof').first()).toBeVisible();
  } finally { await context.close(); }
});

test('specs reflow at 320px and their diagrams support keyboard scrolling', async ({ page }) => {
  for (const path of ['/spec/syntax/', '/spec/static-semantics/']) {
    await page.goto(path);
    const diagram = page.locator('.rrd-diagram, .sequent-proof-container').first();
    await expect(diagram).toHaveAttribute('tabindex', '0');
    await page.setViewportSize({ width: 320, height: 800 });
    await page.addStyleTag({ content: '* { line-height: 1.5 !important; letter-spacing: .12em !important; word-spacing: .16em !important; } p { margin-bottom: 2em !important; }' });
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth), path).toBe(true);
    await diagram.focus();
    await expect(diagram).toBeFocused();
    await page.keyboard.press('ArrowRight');
    expect(await diagram.evaluate(element => element.scrollLeft)).toBeGreaterThan(0);
  }
});

test('all specification chapters pass automated WCAG AA checks', async ({ page }) => {
  test.setTimeout(120_000);
  for (const path of paths) {
    await page.goto(path);
    const result = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa']).analyze();
    expect(result.violations, path).toEqual([]);
  }
});

test('site search finds the rendered specification', async ({ page }) => {
  await page.goto('/search/');
  await page.getByRole('textbox', { name: /search/i }).fill('constraint satisfaction');
  await expect(page.locator('.pagefind-ui__result-link[href*="/spec/"]').first()).toBeVisible({ timeout: 15_000 });
});

test('railroad stations have visible text and bounds include the whole diagram', async ({ page }) => {
  await page.goto('/spec/syntax/');
  const dimensions = await page.locator('.rrd-diagram svg').evaluateAll(elements => elements.map(element => {
    const svg = element as SVGSVGElement;
    const rect = svg.querySelector('rect')!;
    const path = svg.querySelector('.librrd-rail')!;
    const bounds = svg.getBBox();
    const view = svg.viewBox.baseVal;
    return { fill: rect ? getComputedStyle(rect).fill : 'none', stroke: getComputedStyle(path).stroke,
      width: svg.width.baseVal.value, height: svg.height.baseVal.value,
      inside: view.width > 0 && bounds.x >= view.x && bounds.y >= view.y && bounds.x + bounds.width <= view.x + view.width && bounds.y + bounds.height <= view.y + view.height };
  }));
  for (const diagram of dimensions) {
    expect(diagram.fill).toBe('none');
    expect(diagram.stroke).not.toBe('none');
    expect(diagram.width).toBeGreaterThan(0);
    expect(diagram.height).toBeGreaterThan(0);
    expect(diagram.inside).toBe(true);
  }
});
