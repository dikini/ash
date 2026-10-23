import { test, expect } from '@playwright/test';

test('article contents follows reading position and fragment navigation', async ({ page, isMobile }) => {
  await page.goto('/learn/tour/');
  if (isMobile) {
    await expect(page.locator('.ash-contents')).toBeHidden();
    return; // The existing mobile layout omits article margin contents.
  }
  const nav = page.getByRole('navigation', { name: 'On this page', exact: true });
  const links = nav.locator('a');
  expect(await links.count()).toBeGreaterThan(1);
  await expect(nav.locator('[aria-current="location"]')).toHaveCount(1);
  const target = links.nth(1);
  const fragment = await target.getAttribute('href');
  await target.click();
  await expect(target).toHaveAttribute('aria-current', 'location');
  await expect(target).toHaveCSS('font-weight', '700');
  const marker = await target.evaluate(el => {
    const style = getComputedStyle(el, '::before');
    return { mask: style.maskImage, width: style.width };
  });
  expect(marker.mask).toContain('ash-leaf-outline.svg');
  expect(marker.width).toBe('14px');
  await page.goto(`/learn/tour/${fragment}`);
  await expect(nav.locator(`[href="${fragment}"]`)).toHaveAttribute('aria-current', 'location');
  await page.evaluate(() => window.scrollTo(0, document.documentElement.scrollHeight));
  await expect(links.last()).toHaveAttribute('aria-current', 'location');
  await page.evaluate(() => window.scrollTo(0, 0));
  await expect(links.first()).toHaveAttribute('aria-current', 'location');
});

test('hierarchical study selection and ancestor emphasis work without JavaScript', async ({ browser, baseURL }) => {
  const context = await browser.newContext({ javaScriptEnabled: false });
  try {
    const page = await context.newPage();
    await page.goto(`${baseURL}/study/latest/learn-what-is-an-ash-program.html`);
    const current = page.locator('#quarto-sidebar .sidebar-link.active');
    await expect(current).toHaveAttribute('aria-current', 'page');
    await expect(current).toHaveCSS('font-weight', '700');
    const ancestors = page.locator('#quarto-sidebar .sidebar-item:has(.sidebar-link.active) > .sidebar-item-container > .sidebar-item-text:not(.active)');
    expect(await ancestors.count()).toBeGreaterThan(0);
    await expect(ancestors.first()).toHaveCSS('font-weight', '600');
    expect(await ancestors.first().evaluate(el => getComputedStyle(el, '::before').content)).toBe('none');
    expect(await current.evaluate(el => getComputedStyle(el, '::before').maskImage)).toContain('ash-leaf-outline.svg');
  } finally { await context.close(); }
});

test('both theme variants are available and vertical menus reflow at narrow widths', async ({ page, request }) => {
  for (const variant of ['outline', 'solid']) {
    const response = await request.get(`/marks/ash-leaf-${variant}.svg`);
    expect(response.ok()).toBe(true);
    expect(await response.text()).toContain('currentColor');
  }
  await page.setViewportSize({ width: 320, height: 800 });
  for (const path of ['/learn/tour/', '/docs/', '/study/latest/learn-what-is-an-ash-program.html']) {
    await page.goto(path);
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth), path).toBe(true);
  }
});

test('location remains distinguishable under forced colors', async ({ page }) => {
  await page.emulateMedia({ forcedColors: 'active', reducedMotion: 'reduce' });
  await page.goto('/learn/tour/');
  const current = page.locator('.ash-contents [aria-current="location"]');
  await expect(current).toHaveCount(1);
  const marker = await current.evaluate(el => {
    const style = getComputedStyle(el, '::before');
    return { mask: style.maskImage, border: style.borderTopWidth };
  });
  expect(marker).toEqual({ mask: 'none', border: '1px' });
});


test('vertical mobile menu distinguishes a current page from an ancestor section', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/docs/');
  await page.locator('.mobile-nav summary').click();
  const docs = page.locator('.ash-page-menu a[href="/docs/"]');
  await expect(docs).toHaveAttribute('aria-current', 'page');
  expect(await docs.evaluate(el => getComputedStyle(el, '::before').maskImage)).toContain('ash-leaf-outline.svg');
  for (const path of ['/study/', '/contribute/']) {
    await page.goto(path);
    await expect(page.locator(`.ash-page-menu a[href="${path}"]`)).toHaveAttribute('aria-current', 'page');
  }
  await page.goto('/docs/install/');
  await expect(docs).toHaveAttribute('aria-current', 'true');
  await expect(docs).toHaveCSS('font-weight', '600');
  expect(await docs.evaluate(el => getComputedStyle(el, '::before').content)).toBe('none');
});


test('hosted study contents follows section-wrapper headings', async ({ page, isMobile }) => {
  test.skip(isMobile, 'The upstream mobile layout hides margin contents');
  await page.goto('/study/latest/learn-what-is-an-ash-program.html');
  const nav = page.locator('#TOC');
  await expect(nav.locator('[aria-current="location"]')).toHaveCount(1);
  const target = nav.locator('a').nth(1);
  await target.click();
  await expect(target).toHaveAttribute('aria-current', 'location');
  expect(await target.evaluate(el => getComputedStyle(el, '::before').maskImage)).toContain('ash-leaf-outline.svg');
});

test('nested study contents preserves depth and emphasizes the active branch', async ({ page, isMobile }) => {
  test.skip(isMobile, 'The upstream mobile layout hides margin contents');
  const path = '/study/latest/cases-guaranteeing-logarithmic-retrieval.html';
  await page.goto(path);
  const child = page.locator('#TOC ul ul a').first();
  const fragment = await child.getAttribute('href');
  await page.goto(`${path}${new URL(fragment!, page.url()).hash}`);
  await expect(child).toHaveAttribute('aria-current', 'location');
  const parent = page.locator('#TOC a[data-current-ancestor]').first();
  await expect(parent).toHaveCSS('font-weight', '600');
  const childLeft = await child.evaluate(el => el.getBoundingClientRect().left);
  const parentLeft = await parent.evaluate(el => el.getBoundingClientRect().left);
  expect(childLeft - parentLeft).toBeGreaterThanOrEqual(12);
});

test('tablet on-this-page disclosure carries the current leaf and follows fragment navigation', async ({ page }) => {
  await page.setViewportSize({ width: 820, height: 1180 });
  await page.goto('/study/latest/cases-searching-ordered-tree.html');
  const contents = page.locator('details.on-this-page');
  await contents.locator('summary').click();
  const current = contents.locator('a[aria-current="location"]');
  await expect(current).toHaveCount(1);
  expect(await current.evaluate(el => getComputedStyle(el, '::before').maskImage)).toContain('ash-leaf-outline.svg');
  const target = contents.locator('a').nth(1);
  await target.click();
  await expect(target).toHaveAttribute('aria-current', 'location');
  await expect(target).toHaveCSS('font-weight', '700');
});

test('tablet disclosure retains nested context when opened and resized', async ({ page }) => {
  await page.setViewportSize({ width: 820, height: 1180 });
  const path = '/study/latest/cases-guaranteeing-logarithmic-retrieval.html';
  await page.goto(path);
  const child = page.locator('#TOC ul ul a').first();
  const fragment = new URL((await child.getAttribute('href'))!, page.url()).hash;
  const contents = page.locator('details.on-this-page');
  await contents.locator('summary').click();
  const target = contents.locator(`a[href$="${fragment}"]`);
  await target.click();
  await expect(target).toHaveAttribute('aria-current', 'location');
  const current = contents.locator('a[aria-current="location"]');
  await expect(current).toHaveCount(1);
  const ancestor = contents.locator('a[data-current-ancestor]').first();
  await expect(ancestor).toHaveCSS('font-weight', '600');
  expect(await current.evaluate(el => parseFloat(getComputedStyle(el).paddingInlineStart)))
    .toBeGreaterThan(await ancestor.evaluate(el => parseFloat(getComputedStyle(el).paddingInlineStart)));
  await page.setViewportSize({ width: 390, height: 844 });
  await expect(current).toHaveCount(1);
  expect(await current.evaluate(el => getComputedStyle(el, '::before').maskImage)).toContain('ash-leaf-outline.svg');
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
});
