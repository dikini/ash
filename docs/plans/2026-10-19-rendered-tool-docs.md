# Rendered Ash tool documentation

Apply the approved specification integration design to ash-lang `docs/tools/`.
Publish its overview at `/docs/tools/` and executable guide at `/docs/tools/ash/`.
Reuse the existing Markdown pipeline, site reading layout, search index, exact
source links, and source overrides. Keep the website's pinned installation guide
and its compiler checkpoint intact.

Read both reference groups from one fetched ash-lang commit. Resolve links across
the groups locally, preserving fragments. Other repository links continue to use
the exact source commit. Keep upstream prose and code unchanged.

Implementation sequence:

1. Add failing tests for tool snapshot acquisition, committed-source behavior,
   missing tool chapters, and links between the two reference groups.
2. Extend `scripts/prepare-spec.mjs` to read tools at the resolved spec revision
   and publish both groups atomically. Add a `prepare:tools` command using this
   shared preparation. Reuse the existing source environment overrides.
3. Extract the spec page template into `src/layouts/Reference.astro`, with shared
   types and publication loading, and add the tool routes. Reuse the current
   reading styles, parameterizing group navigation and provenance wording.
4. Link the tool guide from Documentation and specs, add static output assertions
   and browser checks for navigation, source identity, examples, search, no-JS
   reading, mobile reflow, and accessibility.
5. Build, run type checks and relevant tests, inspect desktop/mobile screenshots,
   obtain an independent review, and update publication documentation.
