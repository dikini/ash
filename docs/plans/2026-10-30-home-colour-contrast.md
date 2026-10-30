# Home Page Colour and Contrast Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Give the site a green-black header and continuous home hero while preserving the approved copy, geometry, typography, sage panels, and ash detail.

**Architecture:** Add an explicit home-page body class in `Site.astro`, reuse the existing inline ash-tree symbol for the header brand, and keep the visual revision in plain CSS. Use existing paper, ink, code-muted, enji, and benihi tokens; extend the dark fields to the viewport edges without changing the content container.

**Tech Stack:** Astro 7, plain CSS, Playwright, existing design tokens.

**Design reference:** `docs/plans/2026-10-30-home-colour-contrast-design.md`

---

### Task 1: Specify the dark opening

**Files:**
- Modify: `tests/browser/visual.spec.ts`

**Step 1: Write the failing browser test**

Add this test to `tests/browser/visual.spec.ts`:

```ts
test('home opening joins ink fields with readable paper and beni actions', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('body')).toHaveClass(/home-page/);
  for (const selector of ['.site-header', '.intro-line', '.hero']) {
    expect(await page.locator(selector).evaluate(el => getComputedStyle(el).backgroundColor))
      .toBe('rgb(23, 28, 24)');
  }
  expect(await page.locator('.hero h1').evaluate(el => getComputedStyle(el).color))
    .toBe('rgb(245, 243, 236)');
  expect(await page.locator('.hero .primary').evaluate(el => getComputedStyle(el).backgroundColor))
    .toBe('rgb(183, 40, 46)');
  await expect(page.locator('.wordmark svg use')).toHaveAttribute('href', '#ash-tree');
});
```

**Step 2: Run the test to verify it fails**

Run: `npm run test:browser -- visual.spec.ts -g "home opening"`

Expected: FAIL because the body has no `home-page` class, the opening is paper, and the header brand is an image.

**Step 3: Commit if requested**

```bash
git add tests/browser/visual.spec.ts
git commit -m "test: specify dark home opening"
```

### Task 2: Implement the header and home colour fields

**Files:**
- Modify: `src/layouts/Site.astro`
- Modify: `src/styles/global.css`

**Step 1: Identify the home page and update browser chrome**

In `src/layouts/Site.astro`, change the theme colour to the existing ink value and add the body class:

```astro
<meta name="theme-color" content="#171c18" />
```

```astro
<body class={path === '/' ? 'home-page' : undefined}>
```

**Step 2: Reuse the bare tree for the header brand**

Replace only the header's black-square `img` with the existing symbol, leaving the favicon and footer unchanged:

```astro
<a class="wordmark" href="/" aria-label="Ash home"><svg class="brand-icon" width="32" height="32" aria-hidden="true"><use href="#ash-tree" /></svg><span>ash</span></a>
```

**Step 3: Add the dark header treatment**

In `src/styles/global.css`, keep the existing dimensions and add the colour treatment. The pseudo-element extends the header colour to the viewport edges without clipping the mobile disclosure:

```css
.site-header {
  position: relative;
  isolation: isolate;
  background: var(--ink);
  color: var(--paper);
  border-bottom-color: var(--code-border);
}
.site-header::before {
  position: absolute;
  z-index: -1;
  inset-block: 0;
  left: 50%;
  width: 100vw;
  translate: -50% 0;
  background: var(--ink);
  content: "";
}
.site-header :focus-visible,
.home-page .hero :focus-visible { outline-color: var(--color-accent-bright); }
.desktop-nav a,
.search-link,
.mobile-nav summary { color: color-mix(in srgb, var(--paper) 76%, transparent); }
.desktop-nav a:hover,
.desktop-nav a[aria-current],
.mobile-nav summary:hover,
.mobile-nav[open] summary { color: var(--color-accent-bright); }
.source-link { border-left-color: var(--code-border); }
.source-link img { filter: brightness(0) invert(1); }
```

Set the existing mobile menu's text colour explicitly so its paper surface remains legible:

```css
.mobile-nav nav { color: var(--ink); }
```

**Step 4: Join the introductory line and hero into the dark field**

Use the established full-bleed CSS technique on these two elements only. The clip prevents the background extension from covering adjacent sections:

```css
.home-page .intro-line,
.home-page .hero {
  background: var(--ink);
  box-shadow: 0 0 0 100vmax var(--ink);
  clip-path: inset(0 -100vmax);
}
.home-page .intro-line,
.home-page .hero-description,
.home-page .hero-footnote,
.home-page .example-caption,
.home-page .example-explanation { color: var(--code-muted); }
.home-page .hero h1 { color: var(--paper); }
.home-page .status-pill { color: var(--code-muted); }
.home-page .status-pill > span { background: var(--code-muted); }
.home-page .hero .primary { background: var(--accent); color: var(--color-on-accent); }
.home-page .hero .primary:hover { background: var(--paper); color: var(--ink); }
.home-page .hero .text-link { color: var(--color-accent-bright); }
```

Do not change hero spacing, the grid, font metrics, content, animations, or the code panel.

**Step 5: Give the existing sage panels a clearer tone step**

Keep their dimensions and borders, but make the home panels slightly deeper using existing tokens:

```css
.home .principle-list,
.home .honest-status {
  background: color-mix(in srgb, var(--color-surface-subtle) 88%, var(--color-status-dot));
}
```

**Step 6: Run the focused test**

Run: `npm run test:browser -- visual.spec.ts -g "home opening"`

Expected: PASS.

**Step 7: Review the implementation against the approved design**

Confirm that no copy, geometry, typography, section order, or lower-page ash detail changed. Remove any rule that adds decoration rather than serving the dark opening or colour separation.

**Step 8: Commit if requested**

```bash
git add src/layouts/Site.astro src/styles/global.css tests/browser/visual.spec.ts
git commit -m "feat: add ink opening and sage contrast"
```

### Task 3: Document and verify the delivered design

**Files:**
- Modify: `docs/design-system.md`
- Update: `tests/browser/snapshots.spec.ts-snapshots/home-desktop-linux.png`
- Update: `tests/browser/snapshots.spec.ts-snapshots/home-mobile-linux.png`
- Update affected shared-header snapshots in `tests/browser/snapshots.spec.ts-snapshots/`

**Step 1: Update the design-system record**

Add a short paragraph to `docs/design-system.md` under the current home-page revision describing the green-black header and home opening, paper/sage lower page, bare-tree header brand, red primary action, and bright-benihi dark-surface focus states. Record that copy and geometry remain unchanged.

**Step 2: Run Astro diagnostics**

Run: `npm run check`

Expected: token check and Astro diagnostics pass with no errors.

**Step 3: Review desktop and mobile rendering**

Build and serve the production site, then inspect `/` at 1440px and 390px. Confirm the continuous dark opening, readable code panel, paper transition at Discover, visible sage panels, preserved ash detail, and a legible mobile menu. Check keyboard focus on the dark header and hero.

Run: `npm run build`

Expected: the full production build, Pagefind, and link check pass.

**Step 4: Update intentional visual baselines**

Run: `npm run test:browser -- snapshots.spec.ts --update-snapshots`

Expected: home and shared-header snapshots update; no layout or content changes appear.

**Step 5: Run the focused visual tests**

Run: `npm run test:browser -- visual.spec.ts`

Expected: all tests in this file pass on desktop and mobile.

**Step 6: Check the final diff**

Run: `git diff --check && git status --short`

Expected: no whitespace errors. Only the intended source, documentation, tests, and snapshots are modified; remove generated logs or temporary inspection output.

**Step 7: Commit if requested**

```bash
git add docs/design-system.md src tests/browser
git commit -m "test: verify home colour revision"
```
