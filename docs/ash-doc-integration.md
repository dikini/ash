# Ash-doc integration

Updated: 2026-10-18

The user requested that every site build pull the latest ash-doc documentation
and render it within the website. This supersedes the earlier links-only
choice. Ash-doc remains separately maintained; the website serves its current
development material without claiming that a published edition exists.

## Ownership and rendering

Ash-site owns website presentation and the import adapter. Ash-doc owns study
prose, reading order, canonical examples, verification, and book rendering.
Ash-lang remains authoritative for language semantics. The website uses
ash-doc's own source-preserving Quarto pipeline for HTML. It adds scoped site
navigation and a snapshot banner; it does not reinterpret lesson Markdown or
rewrite example text. PDF and EPUB generation remain upstream.

## Build contract

`npm run build` first runs `scripts/prepare-study.mjs`:

1. Fetch `main` from `https://github.com/dikini/ash-doc.git` into the ignored,
   site-owned `.cache/ash-doc/source` checkout.
2. Resolve that fetch to one exact source revision. Read its declared compiler
   target, font checksums, and pinned Quarto manifest.
3. Install the declared Quarto archive after verifying its SHA-256. On a cache
   miss this downloads a Linux x86-64 binary. Git, Python 3.11+, and tar are
   build prerequisites. Node 24 remains the site's runtime for build scripts.
4. Render the complete indexed book as HTML, preserving its setup, family
   introductions, lessons, source links, figures, and notes.
5. Run upstream's source/output verification. This checks presentation
   agreement; it does not execute the Ash examples or replace upstream's
   compiler verification.
6. Stage the HTML tree under generated `public/study/latest/`, add site
   navigation, and record `snapshot.json`, `upstream-build.json`, and
   `upstream-sources.json`. Astro copies that tree into the built site.
7. Index study article bodies with Pagefind and check final internal links,
   anchors, and referenced assets.

The source checkout, renderer tools, and unchanged rendered output are cached.
Every normal build still fetches upstream. Rendering can reuse output only for
the identical documentation revision; upstream output verification runs again.
Fetch, render, or validation errors fail the build instead of silently using a
stale snapshot. A failed preparation leaves the existing preview artifact in place.

`ASH_DOC_REF=<full-commit-SHA> npm run build` explicitly reproduces an older
snapshot. Omitting it restores the default `main` fetch. The manifest records
the requested ref and resolved source revision. To seed a first build from an
existing archive, set `ASH_DOC_QUARTO_ARCHIVE=/path/to/archive.tar.gz`; the same
upstream checksum is required. No sibling ash-doc checkout is needed.

The current indexed book has no generated diagram fences; its checked-in SVG
figures are preserved by the upstream renderer. If upstream adds tool-dependent
diagram jobs, its renderer fails rather than omitting them. Add preparation for
the declared pinned tools when that requirement appears.

## Reader-facing contract

`/study/` displays the actual snapshot metadata and learning entry points.
`/study/latest/` contains the rendered book with its chapter navigation. Every
chapter shows the documentation revision, Ash target, and development status.
The latest route moves when new source is fetched. It is not an immutable
published-edition URL. Published editions can be added later under separate,
immutable routes and through their own publication workflow.

Study content and informational pages can target different Ash revisions.
Study setup and code belong to the snapshot's declared target. Search results
link to the appropriate page, where its snapshot identity stays visible.
The site's short tour and status retain their own checkpoint.

## Verification

Fixture tests check that integration preserves source code and native links,
adds snapshot identification, and rejects unsupported HTML. Upstream verifies
all rendered source selection and code blocks before staging. Site checks cover
the manifest, generated learning routes, assets, and anchors. Browser review
covers navigation into the book, return links, desktop/mobile layout, and study
search. The import adapter never edits ash-doc source. Shared HTML design tokens are
maintained separately through the [design system](design-system.md) sync
command; the hosted theme applies them even to older source snapshots.
