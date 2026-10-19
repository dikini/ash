# Rendered Specification Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Publish the latest canonical Ash specifications with static railroad
diagrams and proof trees inside the website.

**Architecture:** A preparation script reads one committed ash-lang snapshot,
renders Markdown with the upstream Rehype plugins, and writes an ignored JSON
publication input. An Astro route renders each chapter using the site shell;
Pagefind indexes it with the rest of the site.

**Tech Stack:** Astro, Unified/Remark/Rehype, rrd-preview, sequent-md, Node tests,
Playwright and axe.

### Task 1: Renderer packaging and publication tests

- Add `tests/spec.test.mjs` covering static diagrams, fragment-preserving links,
  ordinary code preservation, and rejection of invalid notation.
- Run `node --test tests/spec.test.mjs`; observe the missing integration fail.
- Package the upstream HTML runtime modules under `vendor/`, with provenance,
  notices, and an explicit update script. Add the Markdown pipeline dependencies.
- Implement `scripts/spec-markdown.mjs`; rerun the targeted tests.

### Task 2: Snapshot acquisition

- Test committed snapshot reading and exclusion of uncommitted local edits with
  a temporary Git fixture. Test full-SHA selection and invalid source failure.
- Implement `scripts/prepare-spec.mjs` using Git reads from a fetched or explicit
  local checkout. Generate `.cache/ash-spec/publication.json` atomically.
- Add `prepare:spec`, and run preparation before dev/build. Include the fetch
  cache in CI. Run preparation against the local checkout and the remote default.

### Task 3: Reading pages

- Add static output assertions for all seven `/spec/` routes and source metadata.
- Implement `src/pages/spec/[...chapter].astro` and `src/styles/spec.css`, with
  chapter navigation, per-page headings, provenance, and previous/next links.
- Update `content/docs.md` with local reference links while retaining the older
  compiler-checkpoint link for readers following the tour.
- Build and run `npm run check`, `npm test`, and the local link checker.

### Task 4: Browser verification and documentation

- Add `tests/browser/spec.spec.ts`: navigation, diagrams without JavaScript,
  search results, 320px reflow, keyboard scrolling, and WCAG AA checks.
- Inspect desktop and phone screenshots. Run the relevant browser tests and
  existing suite. Fix issues and rerun affected verification.
- Update README/development documentation with source overrides, renderer
  updates, and failure behavior. Request an independent code review, address
  actionable findings, and report the verified result.
