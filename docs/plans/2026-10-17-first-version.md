# Ash website first-version implementation proposal

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Deliver a useful public Ash website with a verified short tour and a
clear route into independently versioned study material.

**Architecture:** Generate the informational website as static HTML from
site-owned content. Publish approved ash-doc outputs as separate static edition
trees, preserving their authoring, verification, and rendering pipeline.

**Tech Stack:** Proposed Astro, TypeScript, Markdown content collections,
ordinary CSS, Pagefind, and focused browser checks; existing Quarto Book and
Typst remain responsible for ash-doc outputs.

---

Date: 2026-10-17

Status: proposal for review, not authorization to implement or deploy. The
[ownership model](../site-design.md) is agreed. Stack selection, visual design,
hosting, and the artifact import mechanism remain proposed. Refine this into
code-level tasks against the chosen package versions before implementation.

## Stack alternatives

| Approach | Benefit | Cost for this project |
| --- | --- | --- |
| Custom Astro site plus independent ash-doc artifacts, recommended | Separate layouts for discovery, guides, updates, and study; build-time content validation | We implement article layouts and navigation |
| Astro with Starlight plus ash-doc artifacts | Ready-made documentation navigation and layouts | Adds a documentation framework while the core site has several other reading contexts |
| Quarto website plus Quarto Book | Fewer rendering systems and familiar study tooling | Public presentation becomes coupled to the study renderer's conventions and customization |

Choose custom Astro for the first version. The site is primarily an
informational publication with several entry paths; the detailed study book
already has its renderer. Revisit Starlight if site-owned reference material
grows enough to justify its navigation system.

## Recommended components

- **Astro with static output:** generate the site at build time. Use Astro
  layouts and components for navigation, articles, status, and landing pages.
- **TypeScript:** use strict checking for configuration and application code.
- **Markdown content collections:** validate the metadata actually needed by
  each content type. Add MDX only when a concrete page needs embedded components.
- **CSS:** use shared tokens for typography, spacing, color, focus, and code;
  keep the design small enough to understand without a UI framework.
- **Code presentation:** use Astro's highlighting facilities, with an explicit
  plain-text fallback until an Ash grammar is verified. Correct displayed code
  matters more than approximate coloring.
- **Pagefind:** index site-owned HTML after building. Exclude embedded edition
  trees initially to avoid mixing language targets in results.
- **npm and a lockfile:** pin package dependencies and record a supported Node
  version at implementation time. Do not guess current package versions here.
- **Verification:** framework/type checks, metadata and internal-link checks,
  example verification, and focused Playwright browser journeys.
- **Build automation:** a GitHub Actions workflow is proposed for reproducible
  checks and a static artifact. Select production hosting and preview access
  before adding provider-specific deployment configuration.

The first version needs no request-time application server or database.
Keep full-page navigation and reading usable without client-side JavaScript;
search and a mobile navigation enhancement may use small scripts.

### Technical sources

Official documentation consulted on 2026-10-17:

- [Astro content collections](https://docs.astro.build/en/guides/content-collections/)
  describes schema-validated content and static route generation.
- [Astro pages](https://docs.astro.build/en/basics/astro-pages/) describes routes
  and page formats.
- [Starlight customization](https://starlight.astro.build/guides/customization/)
  describes its documentation layouts and customization boundaries.
- [Pagefind](https://pagefind.app/docs/) describes post-build static search;
  its [configuration](https://pagefind.app/docs/config-options/) provides
  indexing controls.

These capabilities support the proposal; the preference for Astro is a project
judgment. Recheck APIs and supported runtimes when implementation begins.

## Proposed source layout

```text
README.md
docs/                         Project design and active planning
content/                      Site-owned Markdown, grouped by reader purpose
src/
  content.config.ts           Schemas and loaders for content/
  pages/                      Astro route generation and landing pages
  layouts/                    Site and article layouts
  components/                 Navigation, example display, status presentation
  styles/                     Tokens and shared CSS
public/                       Site-owned static assets
examples/                     Canonical site programs, fixtures, and checks
scripts/                      Link checks and optional study artifact assembly
tests/                        Focused behavior and browser checks
dist/                         Generated website and selected study editions
```

This is a target layout, not a request to create empty directories. Use
Astro's conventional `src/` tree instead of the earlier conceptual `site/`
folder. Load root-level `content/` explicitly through content collections.
Keep generated publication files out of authored source. Add a publication
manifest and ignored artifact staging directory only with the first import.

## Delivery sequence

### 1. Define the first content checkpoint

Files: `content/discover/overview.md`, `content/status/index.md`, and the initial
example target record under `examples/`.

1. Choose an exact ash-lang revision for the site's first examples and status
   review; inspect its specification and tool instructions.
2. Record implemented, experimental, and aspirational claims with supporting
   sources and a review date.
3. Choose a small program that makes Ash's character concrete without relying
   on an unimplemented capability.

Acceptance: a reviewer can trace the headline claims and run the example at
the declared target. The checkpoint is deliberately independent of ash-doc's.

### 2. Establish the site foundation and visual direction

Files: `package.json`, lockfile, Node version record, `astro.config.mjs`,
`tsconfig.json`, `src/content.config.ts`, and initial layouts/styles/pages.

1. Present a home-page and article-layout design using real draft content.
2. Scaffold the selected supported Astro version and strict TypeScript config.
3. Implement semantic page structure, short primary navigation, footer,
   responsive typography, code overflow, visible focus, and a useful 404 page.
4. Add content schemas and route generation for the content that exists.

Acceptance: a clean install and static build succeed; desktop and narrow
layouts support keyboard navigation and reading without JavaScript.
Proposed command contract: `npm ci`, `npm run check`, `npm run build`.
These commands will be defined during implementation; they do not exist yet.

### 3. Deliver the informational core and executable tour

Files: home route; content under `discover/`, `learn/`, `documentation/`,
`status/`, `updates/`, and `contribute/`; canonical programs and checks in
`examples/`; a reusable example-display component.

1. Write the overview, learning routes, setup/reference entry points, status,
   contribution entry points, and initial dated update.
2. Add a short tour for experienced programmers and a clear novice entry path.
3. Display runnable code from canonical source or check exact agreement.
4. Check promised values, types, output, and failures against the exact target.
5. Add page metadata, canonical URL configuration, sitemap, and an updates feed
   once the production origin is selected.

Acceptance: every public entry point offers a useful next step; no placeholder
release or production-readiness claim is presented as fact. Example checks run
in a separate verification job so reading builds need no installed compiler.

### 4. Deliver the study entry point

Files: `src/pages/study/index.astro`; conditionally, a selected-edition manifest,
`scripts/assemble-study.mjs`, and integration tests for publication paths.

1. Link to the reviewed ash-doc source and explain its study context and target.
2. Determine whether an approved publication actually exists.
3. If it does, obtain explicitly identified artifacts and verification evidence;
   implement the small import interface described in the integration document.
4. Test nested-path assets, chapter links, anchors, downloads, and target labels.
5. If it does not, retain honest source links and leave artifact embedding as
   a subsequent delivery. Do not publish a local review book by implication.

Acceptance: the study landing page is useful in either state. Importing an
edition never rewrites its lessons, weakens its checks, or drops a book format.

### 5. Add discovery and release checks

Files: search component/configuration, internal-link checker, focused browser
tests, package scripts, and `.github/workflows/check.yml`.

1. Index site-owned content after the static build; exclude `/study/<edition>/`.
2. Check relative links, explicit anchors, draft exclusion, target metadata,
   and any selected edition's files. Add focused negative fixtures for checks
   that reject invalid metadata or missing publication artifacts.
3. Exercise home → tour → docs, novice → study, and status → contribution paths
   with browser automation at desktop and mobile sizes.
4. Review headings, focus, contrast, code scrolling, descriptions, and search
   keyboard use. Add automated accessibility checks for representative layouts.
5. Produce a candidate artifact in CI and document local preview and build use.

Acceptance: type/build checks, links, selected examples, and browser journeys
pass on the candidate artifact. Add tests for consequential behavior rather
than snapshots that merely mirror static prose.

### 6. Prepare publication

Files: provider configuration and release instructions, after hosting selection.

1. Select the production domain, static host, preview policy, and asset limits.
2. Verify the complete candidate at its intended paths, including downloads.
3. Review content accuracy, source permissions, and any unresolved limitations.
4. Present the working candidate and deployment details before publishing.

Acceptance: the first version explains Ash, demonstrates verified code, states
its current boundaries, supports learning and participation, and reaches study
material without confusing editions. Deployment is a separate action from
this documentation proposal.

## Decisions still needed

- Accept or revise the recommended stack and first-version scope.
- Choose visual direction, the initial language checkpoint, and production host.
- Decide whether the first launch links to study source or includes an approved
  published edition, based on ash-doc's actual readiness.

Physical repository consolidation, a complete reference rewrite, and an
interactive playground are outside this first-version proposal.
