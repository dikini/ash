# Study and site design alignment

## Context

Ash-site imports ash-doc through its source-preserving Quarto HTML pipeline. The
hosted chapters already share design tokens and code colors with the website,
but their chrome, typography, navigation surfaces, and page geometry still read
as a separate publication. The whole hosted study page should feel compatible
with ash-site without changing ash-doc source or sacrificing Quarto's reading
features.

## Approach

Keep the existing ownership boundary. Ash-doc owns content and rendered document
structure; ash-site owns hosted presentation through `scripts/study-html.mjs`,
`public/study-chrome.css`, and the shared navigation assets. Extend that adapter
rather than changing upstream book styles or rebuilding chapters as Astro pages.

## Design

### Site chrome

Use the website's green-black header, tree wordmark, navigation, search, and
responsive menu treatment on hosted chapters. Place the development snapshot
notice below the header with quiet secondary typography. Close the page with the
site's ink footer. Preserve the existing snapshot identity, source links, skip
link, keyboard focus, and native disclosure behavior.

### Quarto document and navigation

Retain Quarto's source-preserving HTML, main article, chapter sidebar, on-page
aside, breadcrumbs, search, chapter links, collapsible navigation, and mobile
`On this page` disclosure. Do not replace these with the Astro article layout or
rewrite lesson content.

Retain the book's serif body text and reading character. Use the site's Pagella
display treatment for headings and Ash Sans for navigation, metadata, controls,
and callout labels. Align links, inline code, dark code panels, callouts, tables,
figures, and vertical rhythm with existing site components. Preserve the shared
current-page leaf, ancestor emphasis, focus behavior, and forced-colors support.

### Width and responsive geometry

Apply the site's layout contract to the complete Quarto page shell:

- Constrain the overall content shell to `var(--layout-max)` (1240px).
- Use `var(--layout-gutter)` at the existing responsive breakpoints.
- Align the main reading column with `var(--layout-article)` and keep prose near
  `var(--measure-reading)` where Quarto's column model permits.
- Keep the chapter sidebar and on-page aside present and usable at wide widths;
  do not collapse, reorder, or absolutely position them into a different layout.
- Retain Quarto's existing responsive disclosures and verify tablet and mobile
  behavior, including horizontal overflow and enlarged text.

The width changes are hosted overrides to Quarto's geometry, not a new shared
layout token contract and not a change to ash-doc's PDF, EPUB, or standalone
book output.

## Verification

Extend the existing integration and browser coverage to check site chrome,
shared typography and component treatments, and the 1240px/gutter geometry.
Exercise representative chapters at desktop, tablet, and mobile sizes. Confirm
chapter navigation, search, native disclosures, current-location markers, and
navigation without JavaScript still work. Run accessibility and overflow checks,
the content test suite, Astro diagnostics, and a full build. Existing upstream
source/output verification must continue to pass without generated-file edits.
