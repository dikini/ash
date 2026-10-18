# Build-time Ash documentation integration

**Goal:** Fetch ash-doc main during every site build and serve its rendered
learning material within the website.

**Architecture:** Resolve one upstream Git revision, render its indexed book
through its own Quarto HTML pipeline, validate the output, and stage it under
`/study/latest/`. Retain the upstream reading order, examples, notes, and figures.
Add site navigation and explicit development-snapshot identity. This changes
the previous links-only policy at the user's request; it does not declare a
published book edition.

**Tech stack:** Git, Python 3.11+, upstream checksum-pinned Quarto, Node, Astro,
Pagefind, and the existing browser checks.

## Delivery

1. Add fixture-based tests for imported HTML preservation and snapshot metadata.
2. Add a fetch/render script with an ignored source/tool cache. Fetch remote
   main on each build; allow an exact revision for reproducible rebuilds.
   Resolve it once, fail on fetch/render/check errors, and record provenance.
3. Reuse upstream HTML generation and source/output verification. Download
   Quarto according to the upstream manifest, checking its archive hash.
4. Copy the validated HTML tree into generated public assets. Add scoped site
   navigation and a snapshot banner without rewriting program text. Include
   study article bodies in the site's Pagefind index.
5. Replace the links-only study landing page with navigation into the rendered
   book and visible documentation/compiler revisions.
6. Update CI prerequisites, source/tool caching, contributor instructions, and
   the ownership document. Keep EPUB/PDF generation upstream and out of the
   website HTML build.
7. Verify latest remote rendering, local links and anchors, representative
   desktop/mobile chapter navigation, search, and code/figure preservation.

The current upstream book has no generated diagram fences; its checked-in SVG
figures are handled by its own renderer. If upstream introduces additional
rendering dependencies, fail with their original error rather than dropping
content or substituting stale output. Extend tool preparation when required.
