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

`build` generates HTML, builds the Pagefind index, and checks local page links,
fragment destinations, and referenced assets. It does not require ash-lang or
ash-doc checkouts. Astro may background a server when run by an agent; use
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

`src/data/project.json` records the website language target and the separately
reviewed ash-doc source and language targets. Review affected claims and rerun
examples when changing the website target. Study links must continue to point
to the source edition's own setup and language target.

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

The first version links to ash-doc source at its reviewed revision, labeled
unpublished. Do not generate a book, copy lesson prose into a second renderer,
or add download buttons until an edition is ready. Follow the
[integration contract](ash-doc-integration.md) when that publication is available.
