# Rendered Ash design notes

Extend the approved reference publication to `ash-lang/docs/design/`, under
`/docs/design/`. Import every committed Markdown document recursively from the
same latest-main revision as specs and tools. Preserve upstream prose, examples,
status fields, and evidence links. Lowercase public routes retain nested paths;
README documents become directory indexes. Other source files remain links to
the exact GitHub revision.

Use the upstream overview as the collection index, with an expandable list of
supporting documents. Keep per-page navigation compact; adjacent-note links cover
the numbered design notes. Label every page as informative research and proposals
and link to the current specification. Include all pages in Pagefind search.

The source audit found one sequent block in provider-experiments/semantics.md
that the current renderer cannot parse. Design documents may preserve such blocks
as clearly labelled original source with a build warning. Specification and tool
publication retain strict rendering diagnostics. Do not repair upstream notation
or infer semantic changes in the website importer.

Implementation and validation:

1. Add failing tests for recursive committed snapshots, stable nested routes,
   cross-group/directory links, and the design-only source fallback.
2. Extend the shared snapshot and atomic publication, layout, and Astro routes.
   Add a prepare:design alias, Documentation entry, and contributor instructions.
3. Test published pages, provenance, diagrams, source preservation, navigation,
   search, reading without JavaScript, mobile reflow, and accessibility.
4. Build, type-check, run the relevant tests, inspect screenshots, and request
   independent review. Leave the changes ready for the user's commit request.
