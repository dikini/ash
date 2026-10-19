# Rendered architectural decisions implementation plan

**Goal:** Publish ash-lang architectural decisions at `/docs/decisions/` using
the established reference publication design.

**Architecture:** Import all committed Markdown from `docs/decisions/` at the
same immutable latest-main revision as specifications, tools, and design notes.
Reuse static Markdown rendering, the reading layout, local crosslinks, Pagefind,
and revision links. Preserve status and scope: accepted decisions govern
non-language architecture; the specification defines current language rules.

**Tech stack:** Astro, TypeScript, Unified, railroad_md, sequent_md, Pagefind,
Node test runner, Playwright, axe-core.

1. Add failing tests in `tests/spec.test.mjs` for decision snapshots, committed
   source isolation, lowercase/nested routes, duplicate-route rejection, missing
   overview rejection, and crosslinks among all reference groups. Run
   `node --test --test-isolation=none tests/spec.test.mjs` and confirm the new
   cases fail because the decision importer is missing.
2. Extend `scripts/prepare-spec.mjs` with a shared collection route/snapshot
   helper and a decisions group. Keep one fetch, immutable source selection,
   atomic publication, and strict notation diagnostics for decisions. Add
   `prepare:decisions` to `package.json`, then rerun the importer tests.
3. Extend `src/data/reference.ts`, `src/layouts/Reference.astro`, and
   `src/styles/spec.css`; add `src/pages/docs/decisions/[...chapter].astro`.
   Use compact overview navigation, adjacent numbered decisions, preserved
   source status, contents links, and a notice explaining decision authority.
   Keep templates outside adjacent-decision navigation.
4. Link the collection from `content/docs.md` and the other reference groups.
   Update `README.md` and `docs/development.md` to describe four groups and
   their shared source overrides.
5. Add output assertions in `tests/spec-release.test.mjs` and browser coverage
   in `tests/browser/decisions.spec.ts` for navigation, status, crosslinks,
   diagrams, no-JavaScript reading, mobile reflow, search, and accessibility.
   Run `npm run build`, `npm run check`, `npm test`, and the affected reference
   browser suites with `CHROME_BIN=/usr/bin/google-chrome`. Inspect desktop and
   mobile screenshots and request independent review. Leave the changes ready
   for the user's separate commit request.
