import { test, expect } from '@playwright/test';

const chapter = '/study/latest/learn-what-is-an-ash-program.html';

test('hosted chapters share site chrome and keep Quarto reading columns', async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.goto(chapter);
  const favicon = page.locator('head link[rel="icon"]');
  await expect(favicon).toHaveAttribute('href', /\/favicon\.svg\?v=ash-tree$/);
  const iconResponse = await page.request.get(new URL((await favicon.getAttribute('href'))!, page.url()).href);
  expect(iconResponse.ok()).toBe(true);
  expect(iconResponse.headers()['content-type']).toContain('image/svg+xml');
  const shell = page.locator('#quarto-content');
  const geometry = await shell.evaluate(el => ({ max: getComputedStyle(el).maxWidth, width: el.getBoundingClientRect().width }));
  expect(geometry.max).toBe('1240px');
  expect(geometry.width).toBeLessThanOrEqual(1240);
  await expect(page.locator('#quarto-document-content')).toBeVisible();
  await expect(page.locator('#quarto-sidebar')).toBeVisible();
  await expect(page.locator('#quarto-margin-sidebar')).toBeVisible();
  expect(await page.locator('#quarto-document-content').evaluate(el => el.getBoundingClientRect().width)).toBeLessThanOrEqual(760);
  const columns = await page.locator('#quarto-content').evaluate(el => {
    const sidebar = el.querySelector('#quarto-sidebar')!.getBoundingClientRect();
    const article = el.querySelector('#quarto-document-content')!.getBoundingClientRect();
    const aside = el.querySelector('#quarto-margin-sidebar')!.getBoundingClientRect();
    return { leftOverlap: sidebar.right - article.left, rightOverlap: article.right - aside.left };
  });
  expect(columns.leftOverlap).toBeLessThanOrEqual(1);
  expect(columns.rightOverlap).toBeLessThanOrEqual(1);
  for (const selector of ['.ash-study-header', '.ash-study-footer']) {
    await expect(page.locator(selector)).toBeAttached();
    expect(await page.locator(selector).evaluate(el => getComputedStyle(el).backgroundColor)).toBe('rgb(23, 28, 24)');
  }
  await expect(page.locator('.ash-study-header nav[aria-label="Main navigation"]')).toBeVisible();
  await expect(page.locator('.ash-study-header').getByRole('link', { name: 'Search', exact: true })).toBeVisible();
  expect(await page.locator('#quarto-document-content h1').first().evaluate(el => getComputedStyle(el).fontFamily)).toContain('Ash Pagella');
  expect(await page.locator('#quarto-document-content p').first().evaluate(el => getComputedStyle(el).fontFamily)).toMatch(/PT Serif|Pagella/);
  expect(await page.locator('#quarto-sidebar').evaluate(el => getComputedStyle(el).fontFamily)).toContain('Ash Sans');
});

test('hosted chapters reflow while keeping native mobile reading and site menus', async ({ page }) => {
  for (const width of [320, 390, 820]) {
    await page.setViewportSize({ width, height: 1180 });
    await page.goto(chapter);
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), `${width}px overflow`).toBe(true);
    await expect(page.getByRole('button', { name: 'Toggle sidebar navigation', exact: true })).toBeVisible();
    await expect(page.locator('details.on-this-page summary')).toBeVisible();
    expect(await page.locator('details.on-this-page summary').evaluate(el => getComputedStyle(el).color))
      .toBe(await page.locator('.ash-study-banner').evaluate(el => getComputedStyle(el).color));
    if (width <= 760) {
      const menu = page.locator('.ash-study-header details.mobile-nav');
      await menu.locator('summary').click();
      await expect(menu.locator('nav')).toBeVisible();
      await expect(menu.getByRole('link', { name: 'Study Ash', exact: true })).toHaveAttribute('aria-current', 'true');
      await menu.locator('summary').press('Enter');
      await expect(menu).not.toHaveAttribute('open');
    }
  }
});

test('hosted chapter navigation and native menu work without JavaScript', async ({ browser }) => {
  const context = await browser.newContext({ javaScriptEnabled: false, viewport: { width: 390, height: 844 } });
  try {
    const page = await context.newPage();
    await page.goto(chapter);
    await expect(page.locator('#quarto-sidebar a[aria-current="page"]')).toBeAttached();
    const menu = page.locator('.ash-study-header details.mobile-nav');
    await menu.locator('summary').click();
    await expect(menu.locator('nav')).toBeVisible();
    await menu.getByRole('link', { name: 'Learn', exact: true }).click();
    await expect(page).toHaveURL(/\/learn\/$/);
  } finally { await context.close(); }
});


test('hosted study keeps reading controls and reflows with doubled text', async ({ page }) => {
  await page.setViewportSize({ width: 820, height: 1180 });
  await page.goto(chapter);
  await page.addStyleTag({ content: 'html { font-size: 200% !important; }' });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await expect(page.locator('details.on-this-page summary')).toBeVisible();
  await expect(page.locator('#quarto-document-content')).toBeVisible();
});

test('full-width study sections keep prose clear of sticky chapter navigation and preserve illustrations', async ({ page }) => {
  const path = '/study/latest/learn-using-more-than-one-input.html#pausing-after-the-first-input';
  for (const width of [1440, 1280, 1100, 992]) {
    await page.setViewportSize({ width, height: 1000 });
    await page.goto(path);
    const section = page.locator('#pausing-after-the-first-input');
    await section.scrollIntoViewIfNeeded();
    expect(await page.evaluate(() => scrollY)).toBeGreaterThan(100);
    await expect(page.locator('#quarto-document-content')).toHaveClass(/page-full/);
    const layout = await page.evaluate(() => {
      const sidebar = document.querySelector('#quarto-sidebar')!.getBoundingClientRect();
      const aside = document.querySelector('#quarto-margin-sidebar')!.getBoundingClientRect();
      const shell = document.querySelector('#quarto-content')!.getBoundingClientRect();
      return {
        left: sidebar.right, right: aside.left, shellRight: shell.right,
        content: [...document.querySelectorAll('#pausing-after-the-first-input > h2, #pausing-after-the-first-input > p, #pausing-after-the-first-input > pre')].map(el => {
          const r = el.getBoundingClientRect(); return { left: r.left, right: r.right };
        }),
        images: [...document.querySelectorAll<HTMLImageElement>('#quarto-document-content img.authored-illustration')].map(el => {
          const r = el.getBoundingClientRect(); return { left: r.left, right: r.right, width: r.width };
        }),
      };
    });
    expect(layout.content.length).toBeGreaterThan(0);
    for (const r of layout.content) {
      expect(r.left, `sidebar overlap at ${width}px`).toBeGreaterThanOrEqual(layout.left - 1);
      expect(r.right, `aside overlap at ${width}px`).toBeLessThanOrEqual(layout.right + 1);
    }
    expect(layout.images.length).toBeGreaterThan(0);
    for (const r of layout.images) {
      expect(r.width, `illustration visible at ${width}px`).toBeGreaterThan(100);
      expect(r.left).toBeGreaterThanOrEqual(layout.right - 1);
      expect(r.right).toBeLessThanOrEqual(layout.shellRight + 1);
    }
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  }
  for (const width of [820, 390]) {
    await page.setViewportSize({ width, height: 1000 });
    await page.goto(path);
    const widths = await page.locator('img.authored-illustration').evaluateAll(images => images.map(el => el.getBoundingClientRect().width));
    expect(widths.length).toBeGreaterThan(0);
    expect(widths.every(width => width > 100)).toBe(true);
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  }
});
