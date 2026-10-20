# Search engines and agent discovery

The production build generates HTML metadata and a content catalogue from the
same document records. This covers the website,
specification, tool documentation, design notes, decisions, and Study Ash.
No running API, account, or JavaScript execution is required to read these files.

## Public URLs

The public domain is configured at build time:

```sh
SITE_URL=https://your-chosen-domain.example npm run build
```

`SITE_URL` must be an HTTP(S) origin without credentials, a path, query, or
fragment. A final slash is accepted. With it, builds generate self-canonical
HTML links, absolute social URLs, and `sitemap.xml`; `robots.txt` advertises
that sitemap. Include the same setting when using `docs:refresh`.

Without it, local builds still generate metadata, Markdown, the catalogue,
`llms.txt`, and an allow-crawling `robots.txt`. Links use root-relative paths.
Canonical links and the sitemap are omitted. No placeholder domain is published.
Set the actual origin before deploying. A configured build is intended for that
origin; local preview URLs do not change its canonical links.

## Discovery outputs

- `/llms.txt` is a concise guide to useful starting points and document authority.
- `/content-index.json` lists every public reading page, its description, HTML
  and Markdown URLs, metadata, source identity, and Markdown checksum.
- Reading pages with available Markdown link to it from their HTML head. Public imported documents
  point directly to their original Markdown at the exact upstream commit.
  There is no second local Markdown URI for the same public source content.
- Pages without a committed, published Markdown source get a generated local
  representation. Directory routes use `index.md`, for example
  `/learn/tour/index.md`. Study pages currently remain HTML-only, as described below.
- `/build-manifest.json` records build inputs and discovery-output hashes.
- `/robots.txt` allows public crawling. With `SITE_URL`, `/sitemap.xml` lists
  the canonical reading URLs and reliable source update dates when available.

Drafts have no published page or Markdown export. `noindex` documents may retain
an HTML page but are omitted from the catalogue, Markdown exports, sitemap,
`llms.txt`, and local search. Search and error pages are also excluded.
These exclusions are publishing choices, not access controls.

The specification defines current language rules. Informative design notes and
architectural decisions carry their own status and authority; neither acquires
language-specification authority by appearing in the catalogue. Study records
also state their own compiler target, which can differ from the website's
checkpoint. The guide and catalogue explain these distinctions to agents.

## Reusing source Markdown

Public imported pages point to the original Markdown from the exact
source revision selected for the build. Those original files retain their
frontmatter, prose, examples, transcripts, EBNF, sequent fences, and repository
links. The site does not rewrite, prepend to, or copy them to another public URI.
An agent can resolve relative links against the original Markdown URI.

Catalogue records for public sources link to both the original GitHub document
and its raw Markdown, with a full revision and source checksum. The catalogue's
`markdown` field and the HTML alternate use that same raw Markdown URI.
Generated study index and setup pages are labelled as generated. Pages authored
as Astro components, such as the short tour, get
Markdown extracted from their rendered reading content, retaining code and
transcript labels and omitting site navigation. Site-owned Markdown pages have
no configured public repository yet and receive local Markdown representations.
These local files include provenance, with working links to the published pages.

The `ash-doc` repository is currently private. Its source configuration explicitly
sets `publicMarkdown: false`: study records have `markdown: null`, agent links
use the existing HTML, and no study Markdown copies or alternate links are
published. This includes generated study index and setup pages. Source revisions
and checksums remain recorded independently of that access policy.

After the repository becomes public and its pinned raw URLs are verified, change
`repositories.ash-doc.publicMarkdown` to `true` in `sources.json` and rebuild.
The original committed lessons will then use their pinned raw URLs. The same
flag is `true` for the public ash-lang source. A false value means HTML-only
access; it never silently authorizes a duplicate local Markdown publication.

## Metadata precedence

Frontmatter is parsed as YAML. Site-owned content also follows the existing
Astro collection schema. Import metadata supports these fields:

```yaml
title: A descriptive document title
description: A concise description of this document.
author: Contributor name
# authors: [{name: Contributor name, url: https://example.org/}]
published: 2026-10-18
updated: 2026-10-19
tags: [effects, handlers]
id: DN-0123
status: exploring
related:
  - title: Related document
    url: ../decisions/AD-0001-implementation-environment.md
draft: false
noindex: false
lang: en
```

`summary` is a fallback for `description`; `date` aliases `published`;
`modified` and `lastmod` alias `updated`; `keywords` aliases `tags`.
`robots: noindex, follow` also excludes a page. Dates must be real calendar
dates or ISO timestamps with an explicit timezone. Boolean flags must be YAML
booleans. Invalid metadata fails preparation with a diagnostic.

Without explicit fields, the importer reads the H1 title and leading document
fields such as `- ID:`, `- Status:`, and `- Related:`. A description comes from
the first prose paragraph, shortened without generating new claims. Dates are
never inferred from fetch time or filesystem timestamps. Existing upstream
author metadata is reused when source Markdown supplies none. Additional
frontmatter remains available in each record's `sourceMetadata` field.

Source metadata does not override the website's public origin. Upstream canonical
or other additional fields remain source metadata rather than changing site
canonical URLs. HTML contains descriptions, Open Graph and Twitter metadata,
and JSON-LD describing the actual page; authors and dates are omitted when unknown.

## Build and verification

`scripts/prepare-discovery.mjs` runs after Astro renders and before Pagefind,
link checks, and the final build manifest. It enriches HTML heads while retaining
study body bytes, then writes all discovery files. Re-running it removes stale
local Markdown files and an obsolete sitemap. An imported metadata record from a
different source configuration or study revision is rejected.

Run `npm run check`, `npm run build`, and `npm test`. The discovery tests cover
metadata precedence, stable original Markdown URIs, local export formatting,
relative links, exclusions,
HTML body preservation, origin validation, and consistency of generated outputs.
Once hosting is configured, verify the live robots/sitemap and response headers,
then submit the sitemap through the chosen search engine's webmaster tools.

The implementation follows [Google's crawlability and AI-feature guidance](https://developers.google.com/search/docs/appearance/ai-features),
[sitemap guidance](https://developers.google.com/search/docs/crawling-indexing/sitemaps/build-sitemap),
and the [`llms.txt` proposal](https://llmstxt.org/). `llms.txt` is an additional
agent discovery convention; ordinary crawlable HTML remains the foundation for
search discovery.
