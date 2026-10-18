import { test, expect } from '@playwright/test';

async function textStyle(page: import('@playwright/test').Page, selector: string) {
  return page.locator(selector).first().evaluate(element => {
    const style = getComputedStyle(element);
    let parent: Element | null = element;
    let background = 'rgba(0, 0, 0, 0)';
    while (parent && background === 'rgba(0, 0, 0, 0)') {
      background = getComputedStyle(parent).backgroundColor;
      parent = parent.parentElement;
    }
    const luminance = (color: string) => {
      const channels = color.match(/[\d.]+/g)!.slice(0, 3).map(Number).map(v => {
        v /= 255;
        return v <= .04045 ? v / 12.92 : ((v + .055) / 1.055) ** 2.4;
      });
      return channels[0] * .2126 + channels[1] * .7152 + channels[2] * .0722;
    };
    const values = [luminance(style.color), luminance(background)].sort((a, b) => b - a);
    return { color: style.color, background, font: style.fontFamily, size: parseFloat(style.fontSize), ratio: (values[0] + .05) / (values[1] + .05) };
  });
}

test('main-site body copy is comfortably sized and has strong contrast', async ({ page }) => {
  await page.goto('/');
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth)).toBe(true);
  for (const selector of ['.hero-description', '.principle-list p', '.path-card p', '.honest-status p:not(.eyebrow)', '.diary-invitation p:not(.eyebrow)']) {
    const style = await textStyle(page, selector);
    expect(style.size, selector).toBeGreaterThanOrEqual(16);
    expect(style.ratio, selector).toBeGreaterThanOrEqual(7);
  }
});

test('documentation and site share a readable dark code treatment', async ({ page }) => {
  await page.goto('/learn/tour/');
  const siteCode = await textStyle(page, '.code-example pre code');
  await page.goto('/study/latest/learn-what-is-an-ash-program.html');
  const studyCode = await textStyle(page, '#quarto-document-content pre code');
  expect(studyCode.color).toBe(siteCode.color);
  expect(studyCode.background).toBe(siteCode.background);
  expect(studyCode.font).toBe(siteCode.font);
  expect(studyCode.size).toBeGreaterThanOrEqual(14);
  expect(studyCode.ratio).toBeGreaterThanOrEqual(7);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth)).toBe(true);
});

test('body text uses the bundled regular font rather than device font substitutions', async ({ page }) => {
  await page.goto('/');
  await page.evaluate(() => document.fonts.ready);
  expect(await page.locator('.hero-description').evaluate(element => {
    const style = getComputedStyle(element);
    return { family: style.fontFamily.split(',')[0].replaceAll('"', ''), weight: style.fontWeight,
      loaded: document.fonts.check('400 16px "Ash Sans"') && [...document.fonts].some(font => font.family === 'Ash Sans' && font.status === 'loaded') };
  })).toEqual({ family: 'Ash Sans', weight: '400', loaded: true });
});
