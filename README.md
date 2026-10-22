# Ash website

Ash-site is the evolving public home of the Ash programming language. It helps
people understand Ash, evaluate its current capabilities, learn to use it,
follow development, and contribute.

The primary audience is experienced programmers exploring Ash. The site also
serves language researchers, contributors, and people learning to program.

The agreed ownership model is one public website with independently versioned
study publications embedded within it:

- **ash-site** owns public presentation, informational content, navigation,
  short introductions, status reporting, and updates.
- **ash-doc** owns coherent study material, canonical executable examples,
  verification, and HTML, EPUB, and PDF book outputs.
- **ash-lang** owns the authoritative language specification and implementation.

Ash-doc influences the site's teaching and examples. Its book structure does
not determine the website's entire information architecture. Repository
consolidation remains possible without changing these ownership boundaries.

## Project documents

- [Purpose, audiences, and content structure](docs/site-design.md) — agreed
  direction and proposed navigation.
- [Ash-doc integration](docs/ash-doc-integration.md) — ownership, versioning,
  publication, and preservation requirements.
- [First-version stack and implementation proposal](docs/plans/2026-10-17-first-version.md)
  — alternatives, recommendation, delivery steps, and acceptance criteria.

- [Design system and shared tokens](docs/design-system.md) — visual roles,
  component rules, and ash-doc synchronization.

## Run locally

The first version uses Astro, TypeScript, Markdown content collections, ordinary
CSS, and Pagefind. Use Node 24, npm, and Python 3:

```sh
npm ci
npm run dev
```

For the complete static site, including search:

```sh
npm run check
npm run build
npm test
npm run preview
```

Open the local URL printed by Astro. Search is generated during `build` and
works through `preview`; the development server does not serve that index.

See [development and verification](docs/development.md) for browser tests,
executable-example checks, content authoring, and the CI workflow.

Repository locations, publication mappings, default documentation refs, and
compiler/renderer pins are centralised in `sources.json`. Every build writes
`dist/build-manifest.json` with its exact source revisions and input hashes.
Use `npm run docs:refresh -- --event milestone --reason "Milestone name"` for
a local rebuild with verification, and `npm run sources:pin -- --target checkpoint
--ref TAG-OR-SHA --reason "Milestone name" --dry-run` to inspect a pin update.
See [source management](docs/source-management.md) for the complete local
process, renderer updates, reproducible snapshots, and later PR automation.

Builds also publish a [content catalogue and Markdown exports](docs/discovery.md)
for agents, reuse source metadata for SEO, and generate `llms.txt`. Set
`SITE_URL` to the chosen public origin to enable canonical URLs and a sitemap;
local builds work without a selected domain.

Each build fetches ash-doc `main`, renders its indexed book with its pinned
Quarto HTML renderer, and serves a labeled development snapshot under
`/study/latest/`. The exact source and Ash target are recorded with the output.
The first build needs network access, Git, Python 3.11+, and tar in addition to
Node; automatic Quarto provisioning currently targets Linux x86-64. See the
[integration guide](docs/ash-doc-integration.md).

## GitHub Pages

The site is published at <https://dikini.github.io/ash/> from the `gh-pages`
branch. Build and verify locally when you decide to publish an update:

```sh
npm run publish:pages
```

Start from a clean, committed source checkout. The command runs the local
documentation refresh with Astro checks, the full build, content/token tests,
and browser checks. Pending compiler checkpoints also require example
verification, following the normal refresh policy. It prepares the `/ash/`
artifact, records its verification receipt and file hashes, and pushes only
those static files to `origin/gh-pages`. The source branch stays checked out.
Publishing commits preserve branch history and use ordinary, non-forced pushes.

To publish an already prepared and verified artifact without rebuilding:

```sh
npm run publish:pages -- --prepared
```

This requires the artifact to match the current clean source commit and its
verification receipt, including a hash of every published file. Changed or
unverified output is rejected. `build-manifest.json` records source revisions;
`refresh-report.json` records completed checks and artifact integrity.

GitHub Pages is configured to **Deploy from a branch**, using `gh-pages` and
`/(root)`. The branch includes `.nojekyll`, so GitHub's built-in lightweight
deployment serves the prebuilt files without rebuilding or running tests.
Ordinary pushes to `main` do not publish. The expensive verification workflow
in `.github/workflows/check.yml` remains available only through a manual run.

To review the Pages artifact locally:

```sh
SITE_URL=https://dikini.github.io npm run build
PAGES_URL=https://dikini.github.io/ash/ npm run prepare:pages
python3 -m http.server 8000 --directory .cache/github-pages
```

Open <http://localhost:8000/ash/>. Preparation copies `dist/` to
`.cache/github-pages/ash/`, adapts site links and metadata to `/ash/`, updates
discovery hashes, and checks links at that mounted path. This preview alone does
not create a verification receipt. The publishing command pushes the contents
of that directory; GitHub Pages supplies the `/ash/` mount.

The specification at `/spec/`, tool documentation at `/docs/tools/`, design
notes at `/docs/design/`, and architectural decisions at `/docs/decisions/`
import ash-lang `main` from the corresponding source directories at build time,
using one source revision. Design notes include nested Markdown evidence and are
labelled as informative research. Decisions preserve their recorded status and
govern non-language choices. Grammar fences become static railroad diagrams;
sequent fences become proof trees. Each page links to its exact source revision,
which is independent of the website's executable-example checkpoint. The existing
railroad and sequent-md HTML runtimes are packaged in `vendor/` for reproducible
installs without sibling renderer checkouts. See
[specification publication](docs/development.md#specification-publication)
for pinned revisions and local previews. `npm run prepare:tools`,
`npm run prepare:design`, `npm run prepare:decisions`, and `npm run prepare:spec`
refresh all four reference groups together.
