# SEO and agent discovery implementation plan

**Goal:** Publish consistent metadata, a static content catalogue, and usable Markdown for all public reading pages, keeping the production origin configurable.

**Approved design:** One metadata record per document feeds HTML metadata, structured data, and discovery indexes. Reuse frontmatter and existing document fields before deriving descriptions. Preserve the distinction between normative specifications, informative design work, decisions, and versioned study material. No public domain has been selected. User clarification: link imported documents directly to their commit-pinned original Markdown URI; do not publish duplicate local Markdown URIs. Generate Markdown only where there is no published source Markdown URI.

**Architecture:** Normalize source metadata while preparing imports. After Astro renders the site, combine those records with site Markdown and generated page content, enrich HTML heads without rewriting study body bytes, and emit Markdown, `content-index.json`, `llms.txt`, robots, and an origin-dependent sitemap. Keep the build manifest as provenance and record discovery output hashes there.

**Stack:** Astro, unified/remark, YAML, linkedom, Node's test runner, Playwright.

**Private-source clarification:** User selected HTML-only study pages until ash-doc is public. Store `publicMarkdown: false` in its central repository configuration, omit study Markdown links and copies (including generated study pages), and retain provenance. Public ash-lang documents link to their original Markdown.

## 1. Normalize and retain source metadata

- Add `scripts/content-metadata.mjs` and `tests/discovery.test.mjs`.
- First test frontmatter precedence, leading ID/status/related fields, real dates, draft/noindex, and unchanged code-fence content.
- Implement YAML parsing and normalized records with source metadata retained. Derive descriptions only when absent.
- Integrate with `scripts/spec-markdown.mjs`, `scripts/prepare-spec.mjs`, and `scripts/prepare-study.mjs`; retain original Markdown and the immutable source mapping. Study index/setup use upstream generated Markdown.
- Declare already installed, locked YAML as a direct dependency. Use Markdown AST positions to edit link destinations while preserving source prose and fence bytes.

## 2. Generate catalogue and Markdown

- Extend failing tests with source-relative links, full source revisions, authority, draft exclusion, and code/notation preservation.
- Add `scripts/discovery.mjs` for URL handling, Markdown transformations, catalogue records, and generated-page extraction; add `scripts/prepare-discovery.mjs` for build orchestration.
- Link to the original pinned Markdown for imported documents; generate local Markdown for website/generated pages without a published original. Publish a complete JSON catalogue and concise `llms.txt` linking collection indexes and key starting points.
- Keep generated navigation, scripts, and styles out of exported reading content. Keep code language labels and textual diagram sources.

## 3. Enrich SEO and crawler outputs

- Test metadata reuse in ordinary/imported/study pages, HTML body preservation, valid structured data, search/error exclusion, and builds both with and without an origin.
- Generate page descriptions, social metadata, accurate JSON-LD, alternate Markdown links, and discovery links from the same records.
- Emit `robots.txt`; emit canonical links and `sitemap.xml` only with a validated `SITE_URL`. Omit dates without reliable source evidence. Reject unsupported origins rather than inventing URLs.
- Run discovery generation before Pagefind/link checking and manifest generation; include catalogue and export digests in the manifest.

## 4. Verify and document

- Add release checks for complete catalogue coverage, metadata consistency, export/link validity, and draft privacy.
- Run targeted Node tests, Astro check, complete build, unit/publication suite, and relevant browser checks. Use a temporary output copy to exercise an explicit test origin without changing the chosen public domain.
- Request independent review, resolve actionable findings, and document configuration, metadata precedence, URLs, and later hosting/Search Console steps in `docs/discovery.md` and the development guide.
- Leave implementation changes uncommitted for the user's normal review/commit workflow.
