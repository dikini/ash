# Develop and verify the website

The first local version uses Astro 7, strict TypeScript, Markdown content
collections, CSS, and Pagefind. npm's lockfile records exact package versions.
Use Node 24. All reading pages are generated as static HTML.

## Local commands

```sh
npm ci
npm run dev
```

For a production-style local review, including search:

```sh
npm run check
npm run build
npm test
npm run preview
```

`build` fetches ash-lang and ash-doc main, renders the specifications and verifies
the study HTML, generates the
website and Pagefind index, and checks local links, anchors, and assets. It
needs Git, Python 3.11+, tar, and network access; automatic pinned Quarto
installation currently targets Linux x86-64. It needs no sibling ash-lang or
ash-doc checkout. Astro may background a server when run by an agent; use
`npx astro preview status`, `npx astro preview logs`, and
`npx astro preview stop` to manage that server. The browser-test server uses
`--ignore-lock` to stay in the foreground.

Do not add a guessed production origin. Once a real domain is chosen,
`SITE_URL=https://your-domain.example npm run build` enables canonical links.
Replace that illustrative origin with the selected domain. Sitemap, updates
feed, and provider deployment configuration remain publication work.

## Content and examples

Author site-owned Markdown in `content/`. Front matter declares `title`,
`description`, `section`, `route`, and `updated`; `draft: true` removes an entry
from generated pages and updates navigation. Routes omit leading and trailing
slashes. Astro's collection schema checks metadata and route generation rejects
duplicate content routes.

Use the custom home page and tour for their different presentation needs.
The tour and home page import canonical source directly from `examples/`, so
the displayed programs cannot drift from their checked source files. Use
plain-text code presentation until an Ash highlighting grammar is verified.

`src/data/project.json` records the website language target. The generated
`public/study/latest/snapshot.json` records the fetched documentation and its
own Ash target. Review affected claims and rerun examples when changing the
website target. Study content retains its own setup and compiler target.

## Developer diary entries

Write diary entries in `content/diaries/` with section `Developer Diaries` and a
route under `diaries/`. In addition to normal page metadata, provide `author`
and `published` (YYYY-MM-DD). Keep `updated` at or after publication. The index
sorts published entries by publication date, newest first; editing an entry
does not move it to the top. Draft entries are omitted from pages, listings,
and search. `content/diaries/first-entry.md` is an unpublished authoring starter;
replace its placeholders and set real dates before changing `draft` to false.

Use a conversational voice grounded in actual work. Explain uncertainties and
link evidence without turning an informal entry into a language guarantee.
The current serif reading treatment shares the site shell; full narrative
typography and multilingual font work remain planned separately.

## Executable verification

Use an ash-lang checkout at the exact `languageRevision` in the project data.
The default sibling location is `../ash-lang`; override it explicitly:

```sh
ASH_SOURCE=/path/to/ash-lang npm run verify:examples
```

The verifier checks the revision and tracked compiler-input changes, builds
with Rust 1.97.1, and runs every `.ash` file in `examples/`. It checks exit
status, exact successful value/type output, empty successful stderr, and the
structured reason of the intentional rejection. `expectations.json` must cover
every example. Install the toolchain with rustup if needed. Documentation-only
changes in the compiler checkout do not invalidate these compiler checks.

The website baseline is `bade38c635755a608695dd58c8cbef1254aace4c`.
The three positive examples return `42 : Int`. The negative example rejects
Boolean addition with `type.operator-operand`.

## Browser verification

After building:

```sh
npx playwright install chromium
npm run test:browser
```

On a system with an installed Chrome, an alternative is:

```sh
CHROME_BIN=/usr/bin/google-chrome npm run test:browser
```

The suite covers desktop and mobile journeys, search, links into unpublished
study material, navigation without JavaScript, viewport overflow, and serious
or critical axe accessibility findings on representative layouts. This does
not replace a manual accessibility review. Keep generated traces and browser
results out of Git.

## Automation and current evidence

`.github/workflows/check.yml` defines independent website and example jobs.
The website job builds and checks static output, runs browser checks, and
uploads a preview artifact. The example job checks out the exact compiler
revision and runs the canonical example verifier. No job deploys the site.

Local verification on 2026-10-18 produced 13 static pages, passed the four
Node tests and ten desktop/mobile browser tests, and verified all four Ash
programs against the declared checkpoint. Desktop and mobile home-page
screenshots were inspected. The workflow has been reviewed locally; a hosted
GitHub Actions run has not occurred.

## Study publication boundary

The website now renders latest ash-doc HTML as a development snapshot. Each
build fetches main once, records its exact revision, and reuses cached rendered
output only when the source revision is unchanged. `ASH_DOC_REF=<full-SHA>`
selects an explicit revision for reproduction. Quarto's upstream archive is
downloaded and checksum-verified on first use; `ASH_DOC_QUARTO_ARCHIVE` can seed
it from an existing archive. Source, tools, and HTML live in ignored `.cache/`;
generated study assets are also ignored. CI caches them but still fetches main.

Run `npm run prepare:study` to prepare documentation without rebuilding Astro.
This also makes the study assets available to the development server. Fetch or
render errors fail the build, without silently falling back to stale content.
See the [integration contract](ash-doc-integration.md) for provenance and
verification boundaries. PDF and EPUB publication remain upstream.

## Specification publication

Each build fetches ash-lang `main` once and renders every canonical chapter from
`spec/` under `/spec/`, together with `docs/tools/` under `/docs/tools/`,
`docs/design/` under `/docs/design/`, and `docs/decisions/` under
`/docs/decisions/`. Design and decision documents include nested Markdown. The
four groups share the same exact source revision and reading layout. The
importer also recognizes the older `docs/spec/` path.
The source Git repository and generated publication input live in ignored
`.cache/ash-spec/`. Pagefind indexes these pages with the rest of the site.

The specification follows its own source revision. It can describe a newer Ash
than the compiler checkpoint used by the website's tour and executable examples.
Every reference page links to its exact source commit. Links among specification,
tool, design, and decision pages stay on the website; links to other upstream
documents use that same commit on GitHub. The pinned installation guide retains
its checkpoint-specific tool reference for readers following the tour.

```sh
# Reproduce a particular upstream commit (replace with a real full SHA).
ASH_SPEC_REF=<full-SHA> npm run build

# Preview the committed specification at a local checkout's HEAD.
ASH_SPEC_SOURCE=../ash-lang npm run dev

# Prepare all reference groups without rebuilding the whole website.
npm run prepare:spec
# Equivalent entry points for the other reference groups.
npm run prepare:tools
npm run prepare:design
npm run prepare:decisions
```

`dev` prepares all reference groups at startup. Restart the development server
to refresh the snapshot after a new upstream commit. The `ASH_SPEC_REF` and
`ASH_SPEC_SOURCE` overrides apply to all four groups. Local imports read committed
Git blobs and leave the checkout untouched; uncommitted reference edits are excluded
so the displayed revision stays accurate. `ASH_SPEC_REF` can also select a
specific commit available in that local repository.

The HTML pipeline uses railroad's `rrd-preview/rehype` for EBNF and
`@sequent-md/rehype` for inference rules. The existing HTML runtime packages are
included as npm archives in `vendor/`, with upstream revisions, copied-file
hashes, and railroad license notices. See [renderer updates](../vendor/README.md).
Ordinary installs need no renderer source checkouts, Java, sbt, or browser-side
rendering. Long grammar sequences are grouped associatively to bound librrd's
line-partition search; the original EBNF remains available in an expandable
source block. Diagrams scroll within the page and accept keyboard focus.
SVG bounds are measured during preparation with svgdom so taller diagrams retain
their complete content without browser-side layout scripts.

Design notes retain their original status fields and carry an informative-work
notice linking to the current specification. Their upstream overview indexes the
numbered notes, with an expandable list of supporting documents. Public paths
are lowercase, retain nested directories, and map README files to directory
indexes. Links to implementation files, experiment code, and other non-published
sources use the exact source commit on GitHub.

Architectural decisions use the same collection layout and route conventions,
with adjacent links between numbered decisions. Their recorded status, context,
consequences, and verification requirements remain upstream-owned. The notice on
each page distinguishes non-language architecture from current language rules.
Authoring templates and other supporting documents are published and searchable,
but do not appear as adjacent numbered decisions.

Source-fetch errors, missing required documents, duplicate collection routes,
invalid EBNF, and renderer diagnostics fail preparation. Design notes alone allow an
unrenderable sequent block to remain as its unchanged source, labelled “Inference
rule source (diagram unavailable),” with a build warning. Specs, tools, and
architectural decisions reject unrenderable notation. The publication input is
replaced only after all documents render. The full build also checks every local
reference link and fragment.

## Shared reading and code styles

`public/theme.css` owns the paper, ink, secondary-text, accent, and code palette
tokens used by both the site and imported study pages. Keep these tokens shared
rather than adding a second hardcoded palette in the documentation overlay.
`src/styles/global.css` owns site layout; `public/study-chrome.css` applies the
site palette and dark code panels around the native book reading layout.
These changes survive fetching a newer ash-doc revision without modifying its
canonical prose, code, or independently generated PDF/EPUB outputs.

Main descriptive copy stays at least 16px, and narrative prose uses a larger
reading size. Focused browser checks require at least 7:1 contrast for selected
body-copy sections and matching code foreground/background/font across the
site and documentation. Code blocks use at least 14px text and retain their own
horizontal scrolling on narrow screens. Check phone screenshots as well as
contrast numbers; passing an accessibility threshold alone does not establish
comfortable reading.

## Accessibility

The website and imported HTML study material target WCAG 2.2 Level AA:
https://www.w3.org/TR/WCAG22/ . Accessibility is a requirement for new content,
components, and documentation imports.

Use semantic headings and landmarks, descriptive links, meaningful image alternatives,
and the correct page language. Preserve visible keyboard focus, skip navigation,
keyboard scrolling of code, reduced-motion preferences, and browser zoom. Avoid
fixed overlays that cover content or focused controls. Keep normal text contrast at
least 4.5:1 and large text/UI boundaries at least 3:1 where required. Aim for 44px
navigation touch targets; meet the WCAG 24px minimum or its spacing exceptions.
Do not communicate status through color alone. New translations must set their
language so assistive technology uses the appropriate pronunciation.

`npm run test:browser` includes axe checks against WCAG A/AA rules on representative
site and book layouts, keyboard skip links, contrast checks, and reflow at 320 CSS
pixels with increased text spacing. CI runs these tests. Study accessibility fixes
belong in the import adapter and `public/study-chrome.css`, preserving canonical
examples and upstream book outputs.

Before publishing new interactive components, manually check keyboard order, visible
and unobscured focus, 200% text enlargement, 400% zoom/reflow, text spacing, and
screen-reader announcements (NVDA/Firefox or VoiceOver/Safari). Automated checks
cannot establish full WCAG conformance; screen-reader and assistive-technology
review remains part of release QA.

The shared theme self-hosts Source Sans 3 regular and semibold from Adobe's
`adobe-fonts/source-sans` release (`WOFF2/OTF`). Files and the SIL Open Font
License are in `public/fonts/`. Do not depend on locally installed fonts for
body text: device substitutions can produce visibly thin strokes even when
computed color contrast passes. Readability tests verify that the bundled
regular face loads. Keep visual phone review alongside numerical contrast checks.

## Local token workflow

Site development and builds synchronize shared design tokens with a sibling
ash-doc checkout before running. `npm run check` checks drift without writing.
Use `npm run tokens:sync` explicitly after editing tokens while a dev server is
already running. Python 3 is required for these commands. See the
[design system](design-system.md#local-bidirectional-synchronization) for peer
selection, worktrees, standalone builds, conflicts, and build hashes.
