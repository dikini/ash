# Site Design Revision Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add restrained visual depth — footer ink band, hero spotlight, staged entrance, tree motif, framed sticky TOC — to homepage, chrome, and article template without changing a single design token or font.

**Architecture:** Pure CSS revision in `src/styles/global.css` plus small markup hooks in `src/layouts/Site.astro` and `src/pages/index.astro`. Palette tokens (`public/tokens.css`) and fonts (`public/theme.css`) are untouched. A Playwright structural spec (computed-style assertions) drives TDD; a separate snapshot spec is the visual regression net, baselined at the end.

**Tech Stack:** Astro 7, plain CSS custom properties, Playwright + @axe-core/playwright.

**Design doc:** `docs/plans/2026-10-20-site-design-revision-design.md` (approved)

**Working directory:** repo root of the worktree `.worktrees/design-revision`, branch `design/site-revision`. Baseline: `npm test` 42 passing, `npm run build` clean.

**Conventions the implementer must know:**

- Playwright serves the *built* site (`webServer: npm run preview`). Always `npm run build` before browser tests, and always run tests with `CI=1` so Playwright starts a fresh preview server instead of reusing a stale one: `CI=1 npx playwright test <file>`.
- The hero entrance must animate `transform` only — the design doc requires text to remain fully opaque (no opacity keyframes).
- Animations fire on page load; snapshot tests must `await page.emulateMedia({ reducedMotion: 'reduce' })` **before** `page.goto()` so the existing reduced-motion rule freezes the final state and screenshots are deterministic.
- Do not commit snapshot PNGs until Task 8 — intermediate baselines would fail on every subsequent CSS change.
- `#171c18` = `rgb(23, 28, 24)` = `var(--ink)`.

---

### Task 1: Failing structural spec + snapshot scaffold

**Files:**
- Create: `tests/browser/visual.spec.ts`
- Create: `tests/browser/snapshots.spec.ts`

**Step 1: Write the failing structural spec**

Create `tests/browser/visual.spec.ts`:

```ts
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
  expect(await mark.evaluate(el => getComputedStyle(el).opacity)).toBeLessThan('0.1');
});

test('article margin contents is sticky and framed', async ({ page }) => {
  await page.goto(article);
  const nav = page.locator('.article-aside nav');
  expect(await nav.evaluate(el => getComputedStyle(el).position)).toBe('sticky');
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
});
```

(rgb(233, 236, 223) = `#e9ecdf` = `--color-surface-subtle`.)

**Step 2: Write the snapshot scaffold**

Create `tests/browser/snapshots.spec.ts`:

```ts
import { test, expect } from '@playwright/test';

const cases = [
  { name: 'home', path: '/', fullPage: true },
  { name: 'tour-hero', path: '/learn/tour/', fullPage: false },
  { name: 'tour-body', path: '/learn/tour/#effects-and-handlers', fullPage: false },
  { name: 'status-hero', path: '/status/', fullPage: false },
];

for (const { name, path, fullPage } of cases) {
  test(`${name} matches visual baseline`, async ({ page }) => {
    await page.emulateMedia({ reducedMotion: 'reduce' });
    await page.goto(path, { waitUntil: 'networkidle' });
    await expect(page).toHaveScreenshot(`${name}.png`, { fullPage, maxDiffPixelRatio: 0.02 });
  });
}
```

**Step 3: Run to verify the structural tests fail and baselines generate**

```bash
npm run build
CI=1 npx playwright test tests/browser/visual.spec.ts
```

Expected: all 7 structural tests FAIL (footer still paper, no `.footer-mark`, no mask, linear-gradient wash, no `.status-mark`, aside not sticky, pre unbordered). Do not run snapshots.spec.ts yet — its baselines will be generated in Task 8 against the finished design.

**Step 4: Commit the spec**

```bash
git add tests/browser/visual.spec.ts tests/browser/snapshots.spec.ts
git commit -m "test: add structural spec for site design revision"
```

---

### Task 2: Footer ink band + tree symbol

**Files:**
- Modify: `src/layouts/Site.astro` (footer block, lines 31–47; body start for symbol)
- Modify: `src/styles/global.css` (width rule lines 35–38; footer rules lines 315–318; mobile footer rule line 367; forced-colors block line 388)

**Step 1: Add the shared tree symbol and restructure the footer**

In `src/layouts/Site.astro`, immediately after `<body>` (before the skip link), add a hidden symbol definition — `currentColor` instead of the favicon's hard `#fff` so the mark can be tinted per context:

```html
<svg width="0" height="0" style="position:absolute" aria-hidden="true">
  <symbol id="ash-tree" viewBox="0 0 64 64">
    <g fill="currentColor">
      <path d="M28 7C21 12 22 19 28 23C34 19 35 12 28 7Z" />
      <path d="M10 15C20 14 25 20 24 29C15 28 10 23 10 15Z" />
      <path d="M46 15C36 14 31 20 32 29C41 28 46 23 46 15Z" />
      <path d="M8 30C17 28 25 33 25 42C15 43 9 38 8 30Z" />
      <path d="M48 30C39 28 31 33 31 42C41 43 47 38 48 30Z" />
      <path d="M22 53C25 52 26 49 26 45H30C30 49 31 52 34 53Z" />
    </g>
    <path d="M28 49V21M28 33L18 23M28 33L38 23M28 44L16 35M28 44L40 35" fill="none" stroke="currentColor" stroke-width="3.2" stroke-linecap="round" stroke-linejoin="round" />
  </symbol>
</svg>
```

Replace the whole `<footer class="site-footer">…</footer>` block with:

```html
<footer class="site-footer">
  <div class="footer-inner">
    <div><a class="footer-brand" href="/" aria-label="Ash home"><svg class="brand-icon" width="32" height="32" aria-hidden="true"><use href="#ash-tree" /></svg><span>ash</span></a><p>Typed functional programming.<br />Explicit effects and scoped resources.</p></div>
    <div class="footer-links"><a href="/diaries/">Developer diaries</a><a href="/study/">Study Ash</a><a href="/contribute/">Contribute</a><a href={project.repository}>GitHub</a><a href="/status/">Project status</a></div>
    <p class="footer-note">Implementation, documentation,<br />and open language research.</p>
    <svg class="footer-mark" viewBox="0 0 64 64" aria-hidden="true"><use href="#ash-tree" /></svg>
  </div>
</footer>
```

(The footer brand swaps the black-square favicon img for the bare tree in `currentColor` — the black square would disappear against the ink band.)

**Step 2: CSS — width rule, footer band**

In `global.css`, change the shared width rule (line 35) to constrain `.footer-inner` instead of `.site-footer`:

```css
.site-header, .home, .article-shell, .footer-inner {
  width: min(var(--layout-max), calc(100% - 2 * var(--layout-gutter)));
  margin-inline: auto;
}
```

Replace the old `.site-footer` rule and its three sub-rules (lines 315–318) with:

```css
.site-footer { background: var(--ink); color: var(--paper); }
.footer-inner {
  position: relative;
  display: flex;
  align-items: start;
  justify-content: space-between;
  gap: var(--space-8);
  padding-block: var(--space-12) var(--space-16);
  overflow: hidden;
}
.footer-mark {
  position: absolute;
  top: 50%;
  right: 0;
  width: clamp(180px, 22vw, 300px);
  translate: 0 -50%;
  color: var(--paper);
  opacity: 0.07;
  pointer-events: none;
}
.site-footer p { margin-top: var(--space-3); color: color-mix(in srgb, var(--paper) 62%, transparent); font-size: var(--text-sm); line-height: 1.7; }
.footer-links { display: grid; grid-template-columns: repeat(3, auto); gap: var(--space-2) var(--space-6); font-size: var(--text-sm); }
.footer-links a:hover { color: var(--color-accent-bright); }
.footer-note { text-align: right; }
```

(Ink-band links hover with `--color-accent-bright` — `#f48658` on `#171c18` is ~7.6:1; the dark rust accent would fail contrast on ink, so the bright token is required here.)

**Step 3: Mobile + forced-colors adjustments**

In the `max-width: 760px` block, replace `.site-footer { flex-direction: column; … }` (line 367) with:

```css
.footer-inner { flex-direction: column; align-items: stretch; padding-block: var(--space-10); gap: var(--space-6); }
.footer-mark { display: none; }
.footer-links { grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--space-2) var(--space-5); }
```

In the `forced-colors` block (line 388) add:

```css
.footer-mark { display: none; }
```

**Step 4: Verify**

```bash
npm run build && CI=1 npx playwright test tests/browser/visual.spec.ts
```

Expected: footer test PASSES; the other 6 still fail.

**Step 5: Commit**

```bash
git add src/layouts/Site.astro src/styles/global.css
git commit -m "feat: close pages on an ink footer band with tree watermark"
```

---

### Task 3: Branch tick on eyebrows

**Files:**
- Modify: `src/styles/global.css` (`.eyebrow` rule, lines 95–98)

**Step 1: Replace the `.eyebrow` rule**

```css
.eyebrow {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  font: var(--weight-semibold) var(--text-sm)/1.5 var(--sans);
  letter-spacing: 0;
}
.eyebrow::before {
  content: "";
  flex-shrink: 0;
  width: 13px;
  height: 13px;
  background: var(--accent);
  -webkit-mask: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 14 14'%3E%3Cpath d='M7 13V3.5M7 7.5 3.5 4M7 9.5l3.5-3' fill='none' stroke='%23000' stroke-width='1.6' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E") center / contain no-repeat;
  mask: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 14 14'%3E%3Cpath d='M7 13V3.5M7 7.5 3.5 4M7 9.5l3.5-3' fill='none' stroke='%23000' stroke-width='1.6' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E") center / contain no-repeat;
}
```

The tick is a two-stroke branch derived from the favicon's branch geometry, rendered in the rust accent via a CSS mask — no markup change, works on every existing eyebrow (article headings, status panel, aside).

**Step 2: Verify**

```bash
npm run build && CI=1 npx playwright test tests/browser/visual.spec.ts
```

Expected: footer + eyebrow tests PASS (5 remain).

**Step 3: Commit**

```bash
git add src/styles/global.css
git commit -m "feat: mark eyebrows with a branch tick"
```

---

### Task 4: Hero spotlight + staged entrance

**Files:**
- Modify: `src/styles/global.css` (`.hero-example::before` lines 171–178; `.code-example` lines 187–193; animation block lines 320–324)

**Step 1: Replace the hero wash**

Replace `.hero-example::before` with a centered radial spotlight:

```css
.hero-example::before {
  position: absolute;
  z-index: -1;
  inset: -40px -48px 40px;
  background: radial-gradient(58% 62% at 52% 45%, var(--color-surface-subtle) 0%, transparent 72%);
  content: "";
  pointer-events: none;
}
```

Add the panel shadow to `.code-example`:

```css
.code-example { box-shadow: var(--shadow-code); }
```

**Step 2: Staged entrance — transform only**

Replace the old `@keyframes ash-arrive` block and its media query (lines 320–324) with:

```css
@keyframes ash-arrive { from { transform: translateY(8px); } to { transform: translateY(0); } }
@media (prefers-reduced-motion: no-preference) {
  .intro-line { animation: ash-arrive 420ms ease-out both; }
  .hero-copy > * { animation: ash-arrive 420ms ease-out both; }
  .hero-copy > *:nth-child(2) { animation-delay: 80ms; }
  .hero-copy > *:nth-child(3) { animation-delay: 150ms; }
  .hero-copy > *:nth-child(4) { animation-delay: 210ms; }
  .hero-example { animation: ash-arrive 480ms 240ms ease-out both; }
}
```

These selectors exist only on the homepage, which keeps the entrance within the documented "one brief homepage movement" budget. No opacity keyframes — text stays fully opaque per the design doc.

In the `max-width: 1100px` and `760px` blocks, delete the now-obsolete `hero-example::before` inset overrides (lines 333, 353) or leave them — they still position the wash; keep them, they remain correct.

**Step 3: Verify**

```bash
npm run build && CI=1 npx playwright test tests/browser/visual.spec.ts
```

Expected: footer, eyebrow, spotlight tests PASS (4 remain).

**Step 4: Commit**

```bash
git add src/styles/global.css
git commit -m "feat: spotlight the hero program and stage its entrance"
```

---

### Task 5: Status watermark + section rhythm

**Files:**
- Modify: `src/pages/index.astro` (honest-status section, lines 73–82)
- Modify: `src/styles/global.css` (`.honest-status` lines 253–262; `.principles`/`.paths` padding lines 226, 236)

**Step 1: Watermark in the status band**

In `index.astro`, inside the `<section class="honest-status" …>` add the mark as the section's last child:

```html
<svg class="status-mark" viewBox="0 0 64 64" aria-hidden="true"><use href="#ash-tree" /></svg>
```

CSS — make the panel a positioning context and place the mark:

```css
.honest-status {
  --status-inset: min(var(--space-8), var(--layout-gutter));
  position: relative;
  overflow: hidden;
  margin-inline: calc(-1 * var(--status-inset));
  margin-block: var(--space-4);
  padding: var(--space-10) var(--status-inset);
  background: var(--color-surface-subtle);
  border-radius: var(--radius-control);
}
.status-mark {
  position: absolute;
  top: 50%;
  right: var(--space-6);
  width: clamp(160px, 20vw, 260px);
  translate: 0 -50%;
  color: var(--ink);
  opacity: 0.055;
  pointer-events: none;
}
```

(Replace the existing `.honest-status` rule wholesale; keep its remaining sub-rules `.honest-status .eyebrow`, `> div + div`, `p:not(.eyebrow)` unchanged.)

In the `760px` block, inside the existing `.honest-status` mobile rule, add `overflow: hidden` is inherited; add `.status-mark { display: none; }`. In the `forced-colors` block add `.status-mark { display: none; }`.

**Step 2: Even the section rhythm**

So the tonal moments read as a progression, give the plain sections equal vertical weight and the band breathing room:

```css
.principles { padding-block: var(--space-16); border-top: 1px solid var(--line); }
.paths { padding-block: var(--space-16); border-top: 1px solid var(--line); }
```

(`.principles` currently `space-12 → space-16`; `.paths` already `space-16` — only the principles top padding changes.)

**Step 3: Verify**

```bash
npm run build && CI=1 npx playwright test tests/browser/visual.spec.ts
```

Expected: footer, eyebrow, spotlight, status-mark tests PASS (3 remain).

**Step 4: Commit**

```bash
git add src/pages/index.astro src/styles/global.css
git commit -m "feat: watermark the status panel and even section rhythm"
```

---

### Task 6: Article aside frame + code cohesion

**Files:**
- Modify: `src/styles/global.css` (`.article-aside nav` line 297; `.callout strong` line 294; `.prose pre` line 289)

**Step 1: Framed, sticky margin contents**

Replace `.article-aside nav` (line 297) with:

```css
.article-aside nav {
  position: sticky;
  top: var(--space-6);
  display: flex;
  flex-direction: column;
  padding: var(--space-5);
  border: 1px solid var(--line);
  border-radius: var(--radius-control);
  background: var(--color-surface-subtle);
  font-size: var(--text-sm);
  line-height: 1.55;
}
```

The old rule's `border-left` + `padding-left` are replaced by the full frame; its `align-self: start` parent rule (line 296) stays.

**Step 2: Callout label**

Replace `.callout strong` (line 294):

```css
.callout strong { display: block; margin-bottom: var(--space-1); font: var(--weight-semibold) var(--text-base)/1.5 var(--sans); }
```

**Step 3: Prose code panel border**

Add `border: 1px solid var(--code-border);` to the `.prose pre` rule (line 289) so article code blocks match the `CodeExample` component.

**Step 4: Verify**

```bash
npm run build && CI=1 npx playwright test tests/browser/visual.spec.ts
```

Expected: all 7 structural tests PASS.

**Step 5: Commit**

```bash
git add src/styles/global.css
git commit -m "feat: frame the margin contents and unify code block borders"
```

---

### Task 7: Update the design-system doc

**Files:**
- Modify: `docs/design-system.md` ("Website presentation (October 2026)" section, after line 147)

**Step 1: Append the revision record**

Add a new paragraph at the end of that section:

```markdown
The November 2026 revision adds restrained depth without changing tokens
or fonts: the footer is a full-bleed ink band with the tree mark as a
low-opacity watermark in paper (`currentColor` at 7%); eyebrows carry a
two-stroke branch tick in the accent via a CSS mask; the homepage code
panel sits in a radial surface-subtle spotlight with a token shadow; the
homepage entrance staggers translateY-only arrivals (under 700ms, text
never fades); the status panel carries the same watermark at 5.5%
opacity; the article margin contents is a sticky, fine-framed block on
the subtle surface at reading scale; callout labels are semibold sans;
prose code blocks share the component border. Watermarks are decorative
(`aria-hidden`) and hidden under forced colors. The single-entrance-
animation budget still holds: no scroll-linked or per-section motion.
```

**Step 2: Commit**

```bash
git add docs/design-system.md
git commit -m "docs: record the restrained-depth presentation revision"
```

---

### Task 8: Full verification + visual baselines + comparison

**Files:**
- Commit: `tests/browser/snapshots.spec.ts-snapshots/` (generated)

**Step 1: Full build and test suite**

```bash
npm run build
npm test
```

Expected: 42 passing (same as baseline), token check clean.

**Step 2: Generate final snapshot baselines**

```bash
CI=1 npx playwright test tests/browser/snapshots.spec.ts --update-snapshots
```

Expected: 8 snapshots written (4 cases × desktop/mobile projects). Eyeball the generated PNGs in `tests/browser/snapshots.spec.ts-snapshots/` for obvious rendering defects before committing.

**Step 3: Full browser suite**

```bash
CI=1 npx playwright test
```

Expected: all specs pass including `visual.spec.ts`, `snapshots.spec.ts`, and the pre-existing accessibility/journeys/readability specs (the ink footer and aside frame must not introduce axe violations).

**Step 4: Screenshot comparison pass**

Serve `dist/` and capture the same angles as the before-images:

```bash
npm run preview -- --port 4321 --ignore-lock &
```

Capture homepage desktop (1440×1000), tour article, and home mobile (390) and compare against `.cache/frontend-design-review-e273371/browser/ash-frontend-home-desktop.png`, `ash-frontend-article.png`, and `ash-frontend-home-mobile.png` (from the main checkout — path `../../.cache/...` relative to the worktree). Confirm: footer band renders, watermarks are quiet not muddy, tick reads at small size, spotlight is intentional, mobile stacks cleanly.

**Step 5: Commit baselines**

```bash
git add tests/browser/snapshots.spec.ts-snapshots
git commit -m "test: baseline visual snapshots for revised templates"
```

**Step 6: Report**

Report to the owner: list of commits, before/after assessment against the design doc's seven sections, and any deviations discovered during implementation.
