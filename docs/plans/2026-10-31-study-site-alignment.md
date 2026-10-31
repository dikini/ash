# Hosted Study Site Alignment Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make hosted ash-doc chapters share ash-site's chrome, component language, and content widths while preserving Quarto's source, main/aside layout, and reading behavior.

**Architecture:** Extend the existing site-side HTML adapter and scoped hosted stylesheet. Do not modify generated chapters, fetched ash-doc source, shared design tokens, or the Astro article layout. Add browser assertions for computed geometry and visual treatments, then run the complete project verification.

**Tech Stack:** Quarto-generated static HTML, CSS custom properties, vanilla JavaScript already loaded by `public/navigation.js`, Node test runner, Astro, Playwright.

**Design reference:** `docs/plans/2026-10-31-study-site-alignment-design.md`

---

### Task 1: Establish hosted chapter regression coverage

**Files:**
- Modify: `tests/browser/visual.spec.ts`
- Modify: `tests/browser/readability.spec.ts`
- Modify: `tests/browser/journeys.spec.ts`
- Modify: `tests/study.test.mjs`

**Step 1: Write failing adapter tests**

In `tests/study.test.mjs`, extend the successful integration assertions to require the same public chrome hooks on every chapter:

```js
assert.ok(document.querySelector('.ash-study-header .ash-tree-mark'));
assert.ok(document.querySelector('.ash-study-header nav[aria-label="Main navigation"]'));
assert.ok(document.querySelector('.ash-study-header a[href="/search/"]'));
assert.ok(document.querySelector('.ash-study-footer'));
```

Keep all existing source-preservation and required-integration assertions. Add the footer requirement to the adapter's unsupported-format guard test expectations.

**Step 2: Run the adapter test and verify failure**

Run: `node --test --test-isolation=none tests/study.test.mjs`

Expected: FAIL because the current adapter has no tree-mark or footer hooks.

**Step 3: Write failing browser geometry tests**

Add a desktop test in `tests/browser/visual.spec.ts` using
`/study/latest/learn-what-is-an-ash-program.html`. Assert:

```ts
const shell = await page.locator('#quarto-content').evaluate(el => {
  const style = getComputedStyle(el);
  return { maxWidth: parseFloat(style.maxWidth), width: el.getBoundingClientRect().width };
});
expect(shell.maxWidth).toBe(1240);
expect(shell.width).toBeLessThanOrEqual(1240);
await expect(page.locator('#quarto-document-content')).toBeVisible();
await expect(page.locator('#quarto-margin-sidebar')).toBeVisible();
```

At 820px and 390px, assert no document overflow and that Quarto's native chapter/on-page controls remain available. Do not require the desktop aside to remain visible at mobile widths. Assert the reading column does not exceed 760px where the current Quarto breakpoint exposes both article and margin columns.

**Step 4: Write failing component and chrome tests**

In the browser suites, assert that the study header/footer use `var(--ink)`, the header includes search and responsive navigation, the display heading uses Pagella, body copy retains the book serif, navigation uses Ash Sans, callouts use the shared surface/accent treatment, and code retains the shared dark panel. Include keyboard focus and native disclosure checks rather than screenshot-only assertions.

**Step 5: Run the focused browser tests and verify failure**

Run: `npx playwright test tests/browser/visual.spec.ts tests/browser/readability.spec.ts tests/browser/journeys.spec.ts`

Expected: the new hosted-chrome and geometry assertions FAIL; existing navigation and source checks remain valid.

**Step 6: Commit the regression coverage**

```bash
git add tests/study.test.mjs tests/browser/visual.spec.ts tests/browser/readability.spec.ts tests/browser/journeys.spec.ts
git commit -m "test: define hosted study design alignment"
```

### Task 2: Add reusable study chrome without changing document source

**Files:**
- Modify: `scripts/study-html.mjs:4-28`
- Modify: `tests/study.test.mjs`

**Step 1: Implement the shared header and footer hooks**

Refactor the existing banner constant into small local header/banner/footer strings. Reuse the current tree geometry as an inline SVG symbol/use or an accessible static asset, with decorative SVGs marked `aria-hidden="true"`. Keep the existing snapshot interpolation and `escape` calls unchanged. Include:

- The existing skip link targeting `#quarto-document-content`.
- Main navigation matching `src/layouts/Site.astro`, including Search and Source.
- A native mobile `details`/`summary` menu using existing shared navigation behavior.
- The current development snapshot aside below the header.
- An ink study footer with the same core destinations as the site footer.

Inject these strings only around the upstream body. Continue the existing attribute-only main/pre/link changes and never rewrite chapter text or relative URLs.

**Step 2: Strengthen the integration guard**

Require `.ash-study-footer` and the new header hooks alongside the existing main, banner, stylesheet, and navigation checks. Failure must remain `Unsupported upstream HTML format`.

**Step 3: Run adapter tests**

Run: `node --test --test-isolation=none tests/study.test.mjs`

Expected: PASS, including unchanged code bytes, links, image URLs, and active-chapter `aria-current`.

**Step 4: Commit the adapter**

```bash
git add scripts/study-html.mjs tests/study.test.mjs
git commit -m "feat: add site chrome to hosted study"
```

### Task 3: Align chrome, components, and Quarto width geometry

**Files:**
- Modify: `public/study-chrome.css`
- Modify: `public/navigation.css` only if an existing shared selector must support the study mobile menu
- Modify: `docs/design-system.md`
- Modify: `docs/ash-doc-integration.md`

**Step 1: Implement the header, banner, and footer treatment**

Replace obsolete fixed-header declarations in `public/study-chrome.css` rather than layering another compensating rule set. Use semantic tokens and site spacing. Keep chrome in normal flow, use ink/paper contrast, bright-accent dark-surface focus/hover states, and 44px coarse-pointer targets. Do not add animation.

**Step 2: Apply the site's width contract to Quarto**

Scope overrides to hosted Quarto containers. Center `#quarto-content` with:

```css
#quarto-content {
  width: min(var(--layout-max), calc(100% - 2 * var(--layout-gutter)));
  max-width: var(--layout-max);
  margin-inline: auto;
}
```

Adjust Quarto's existing grid/flex column sizing so the article is capped by `var(--layout-article)` and its prose measure remains readable. Keep the chapter sidebar and `#quarto-margin-sidebar` in Quarto's layout at wide widths. Do not use fixed positioning to imitate the Astro aside. Preserve existing sticky offsets and native collapses. At `max-width: 760px`, allow one-column flow without changing DOM order.

**Step 3: Align hosted reading components**

Use Ash Pagella for display headings, retain the book body serif, and use Ash Sans for navigation/metadata/controls. Scope all rules beneath Quarto or study-chrome selectors. Match the existing prose link, inline-code, code-block, callout, table, figure, breadcrumb, and framed contents treatments. Preserve syntax-highlighting colors, code scrolling/focus, native summary behavior, current-page leaf markers, and ancestor emphasis.

**Step 4: Regenerate the prepared study output through the standard path**

Run: `npm run prepare:study`

Expected: upstream rendering and source/output verification PASS; staged chapters contain the new adapter hooks and unchanged source content. Never edit `public/study/latest/*.html` directly.

**Step 5: Run focused browser verification**

Run: `npx playwright test tests/browser/visual.spec.ts tests/browser/readability.spec.ts tests/browser/journeys.spec.ts tests/browser/navigation.spec.ts tests/browser/accessibility.spec.ts`

Expected: PASS on desktop and mobile. Inspect at least one content-rich chapter at 1440px, 820px, and 390px. If a test exposes a real layout defect, adjust the scoped CSS and rerun; do not weaken the geometry or accessibility assertions.

**Step 6: Update the ownership documentation**

In `docs/design-system.md`, replace any statement that hosted study retains separate margin/page geometry with the approved shared-width behavior, while retaining book body fonts and Quarto navigation. In `docs/ash-doc-integration.md`, record that the hosted adapter owns site chrome and width/component overrides without changing upstream source.

**Step 7: Commit the presentation**

```bash
git add public/study-chrome.css public/navigation.css docs/design-system.md docs/ash-doc-integration.md
git commit -m "feat: align hosted study with site design"
```

Omit `public/navigation.css` from the commit if no change was needed. Prepared study output is generated and should follow the repository's existing ignore/tracking rules, not be force-added.

### Task 4: Verify the complete deliverable

**Files:**
- Verify all modified files; no new implementation files expected.

**Step 1: Run static checks**

Run: `npm run check`

Expected: token check and Astro diagnostics PASS.

**Step 2: Run content and integration tests**

Run: `npm test`

Expected: all token, source, integration, publication, and content tests PASS.

**Step 3: Run the standard build**

Run: `npm run build`

Expected: upstream preparation, Astro build, Pagefind, link checking, and study source/output verification PASS.

**Step 4: Run the full browser suite on the built deliverable**

Run: `npm run test:browser`

Expected: desktop and mobile projects PASS, including journeys, accessibility, navigation, readability, and visual checks.

**Step 5: Review the final diff and workspace**

Run: `git diff --check && git status --short`

Expected: no whitespace errors. Remove only test/build artifacts created by this work that are not part of the deliverable. Leave unrelated existing untracked skill/config paths untouched.

**Step 6: Commit any final test-only corrections**

```bash
git add tests docs public scripts
git commit -m "test: verify hosted study alignment"
```

Skip this commit if there are no uncommitted deliverable changes.
