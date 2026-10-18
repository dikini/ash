import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

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

test("novice route reaches explicitly unpublished study source", async ({
  page,
}) => {
  await page.goto("/learn/#new-to-programming");
  await page
    .getByRole("link", { name: "Study Ash", exact: true })
    .first()
    .click();
  await expect(
    page.getByText("Publication status: unpublished.", { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Read Learn Ash" }),
  ).toHaveAttribute("href", /github\.com\/dikini\/ash-doc\/tree\/[a-f0-9]{40}/);
});

test("static search returns a relevant language page", async ({ page }) => {
  await page.goto("/search/");
  const input = page.getByRole("textbox", { name: /search/i });
  await input.fill("handlers");
  await expect(page.locator(".pagefind-ui__result-link").first()).toBeVisible({
    timeout: 15000,
  });
  await expect(page.locator(".pagefind-ui__results")).toContainText(
    /Ash|tour|language/,
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
        () => document.documentElement.scrollWidth <= innerWidth,
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
  await page.getByText("Menu", { exact: true }).click();
  await page
    .getByRole("navigation", { name: "Mobile navigation" })
    .getByRole("link", { name: "Status", exact: true })
    .click();
  await expect(
    page.getByRole("heading", { name: "A clear view of where Ash stands" }),
  ).toBeVisible();
  await context.close();
});
