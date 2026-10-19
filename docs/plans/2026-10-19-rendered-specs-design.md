# Render the Ash specification

Approved: 2026-10-19.

Publish the seven canonical Markdown chapters from ash-lang's `spec/` under
`/spec/`, using the existing site shell, article typography, and Pagefind index.
The upstream path is now `spec/`, rather than the former `docs/spec/`.

Each build fetches ash-lang main once and records the exact source revision.
`ASH_SPEC_REF` can select a full commit, and `ASH_SPEC_SOURCE` can select a local
checkout for previews. Local previews read committed spec content so the source
revision remains accurate. The specification snapshot is independent of the
website's executable-example checkpoint. Each page identifies its source.

Render `ebnf` fences through railroad's `rrd-preview/rehype` plugin and `sequent`
fences through `@sequent-md/rehype`. Package the existing HTML renderer runtimes
locally with their source revisions and notices so ordinary installs and CI need
no sibling renderer checkouts, Java, or sbt. Render on the build machine.

Keep upstream prose and examples unchanged. Rewrite intra-spec links to local
routes, preserving fragments; resolve other repository links against the exact
GitHub source revision. Add chapter navigation, an on-page contents list, and
previous/next links. Diagrams scroll within the article on narrow screens and
remain available without JavaScript. Railroad captions identify productions;
proofs retain the renderer's semantic HTML and text.

Fail on source acquisition errors, missing specification chapters, renderer
diagnostics, and broken local links. Verify source provenance, link rewriting,
actual diagram rendering, search discovery, keyboard scrolling, mobile reflow,
and automated accessibility. Reuse the existing study publication workflow.

Alternatives considered: pin specs to the website's older compiler checkpoint,
or keep source-only GitHub links. The user selected the latest-spec build.
