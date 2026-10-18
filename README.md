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

## Run locally

The first version uses Astro, TypeScript, Markdown content collections, ordinary
CSS, and Pagefind. Use Node 24 and npm:

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

The study section links to ash-doc's unpublished source snapshot. No study
edition is embedded, and no production deployment is configured.
