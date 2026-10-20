import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test('study chapters retain book navigation and return to the site', async ({ page }) => {
  await page.goto('/study/latest/learn-what-is-an-ash-program.html');
  await expect(page.locator('#quarto-document-content pre').first()).toContainText('main : Unit -> Int');
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth)).toBe(true);
  await page.locator('.nav-page-next').click();
  await expect(page).toHaveURL(/learn-using-another-source-file\.html/);
  await page.getByRole('navigation', { name: 'Ash website' }).getByRole('link', { name: 'Study overview' }).click();
  await expect(page.getByRole('heading', { name: 'Take time to understand the program' })).toBeVisible();
});

test('site search finds imported study chapters', async ({ page }) => {
  await page.goto('/search/');
  await page.getByRole('textbox', { name: /search/i }).fill('cancellable journal');
  await expect(page.locator('.pagefind-ui__result-link[href*="/study/latest/"]').first()).toBeVisible({ timeout: 15000 });
});

test("front page leads directly to developer diaries", async ({ page }) => {
  await page.goto('/');
  await page.getByRole('main').getByRole('link', { name: 'Read developer diaries' }).click();
  await expect(page.getByRole('heading', { name: 'Developer diaries', exact: true })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'No diary entries published yet' })).toBeVisible();
});

test("experienced programmer can explore the tour and reach setup", async ({
  page,
}) => {
  await page.goto("/");
  await page.getByRole("link", { name: "Take the language tour" }).click();
  await expect(
    page.getByRole("heading", { name: "A short tour of Ash" }),
  ).toBeVisible();
  await expect(page.locator("samp")).toHaveCount(3);
  await page
    .getByRole("link", { name: "Build the pinned command-line tool" })
    .click();
  await expect(
    page.getByRole("heading", { name: "Build Ash. Run a program." }),
  ).toBeVisible();
});

test("novice route reaches the rendered development study snapshot", async ({
  page,
}) => {
  await page.goto("/learn/#new-to-programming");
  await page
    .getByRole("link", { name: "Study Ash", exact: true })
    .first()
    .click();
  await page.getByRole('link', { name: 'Read the study material' }).click();
  await expect(page.locator('.ash-study-banner')).toContainText('Development snapshot');
  await expect(page.locator('#quarto-document-content')).toBeVisible();
});

test("static search returns a relevant language page", async ({ page }) => {
  await page.goto("/search/");
  const input = page.getByRole("textbox", { name: /search/i });
  await input.fill("handlers");
  await expect(page.locator(".pagefind-ui__result-link").first()).toBeVisible({
    timeout: 15000,
  });
  await expect(page.locator(".pagefind-ui__results")).toContainText(
    /handler/i,
  );
});

test("representative layouts have no serious accessibility violations or viewport overflow", async ({
  page,
}) => {
  for (const path of ["/", "/learn/tour/", "/study/", "/status/"]) {
    await page.goto(path);
    const results = await new AxeBuilder({ page }).analyze();
    expect(
      results.violations.filter(
        (v) => v.impact === "serious" || v.impact === "critical",
      ),
      path,
    ).toEqual([]);
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= document.documentElement.clientWidth,
      ),
      path,
    ).toBe(true);
  }
});

test("navigation works without JavaScript", async ({ browser, baseURL }) => {
  const context = await browser.newContext({
    javaScriptEnabled: false,
    viewport: { width: 390, height: 844 },
  });
  const page = await context.newPage();
  await page.goto(`${baseURL}/`);
  await page.getByRole("button", { name: "Menu" }).click();
  await page
    .getByRole("navigation", { name: "Mobile navigation" })
    .getByRole("link", { name: "Status", exact: true })
    .click();
  await expect(
    page.getByRole("heading", { name: "A clear view of where Ash stands" }),
  ).toBeVisible();
  await context.close();
});
