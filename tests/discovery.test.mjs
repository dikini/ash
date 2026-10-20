import test from 'node:test';
import assert from 'node:assert/strict';
import { parseHTML } from 'linkedom';

test('reference metadata reuses frontmatter and retains formal notation as Markdown', async () => {
  const { renderSpecMarkdown } = await import('../scripts/spec-markdown.mjs');
  const markdown = '---\ntitle: Grammar reference\ndescription: Exact grammar and rules.\nauthor: Test Author\nupdated: 2026-10-18\ntags: [grammar, semantics]\ncustom: retained\n---\n# Syntax\n\n- ID: `DN-0100`\n- Status: exploring\n- Related: [Core](../design/core.md)\n\nFirst paragraph.\n\n```ebnf\nterm ::= "true" ;\n```\n';
  const page = await renderSpecMarkdown(markdown, { file: 'syntax.md', chapters: ['syntax.md'], directory: 'spec', revision: 'a'.repeat(40) });
  assert.equal(page.title, 'Grammar reference');
  assert.equal(page.metadata.description, 'Exact grammar and rules.');
  assert.deepEqual(page.metadata.authors, [{ name: 'Test Author' }]);
  assert.equal(page.metadata.updated, '2026-10-18');
  assert.equal(page.metadata.id, 'DN-0100');
  assert.equal(page.metadata.status, 'exploring');
  assert.deepEqual(page.metadata.related, [{ title: 'Core', url: '../design/core.md' }]);
  assert.equal(page.metadata.sourceMetadata.custom, 'retained');
  assert.match(page.markdown, /```ebnf\nterm ::= "true" ;\n```/);
  assert.doesNotMatch(page.html, /custom: retained|description: Exact/);
  const frontmatterOnly = await renderSpecMarkdown('---\ntitle: Frontmatter title\n---\nA document with no repeated H1.\n\n## Details\n\nText.', { file: 'syntax.md', chapters: ['syntax.md'], directory: 'spec', revision: 'a'.repeat(40) });
  assert.equal(frontmatterOnly.title, 'Frontmatter title');
  assert.equal(frontmatterOnly.titleId, 'document-title');
});

test('metadata derives a description but never invents dates or accepts malformed frontmatter', async () => {
  const { readMarkdown } = await import('../scripts/content-metadata.mjs');
  const page = readMarkdown('# Document\n\n- ID: `AD-1`\n- Status: accepted\n\nA useful **description** with `code`.\n\n## Later\n\nOther text.');
  assert.equal(page.metadata.description, 'A useful description with code.');
  assert.equal(page.metadata.updated, undefined);
  assert.throws(() => readMarkdown('---\nupdated: 2026-02-30\n---\n# Bad'), /date/i);
  assert.throws(() => readMarkdown('---\ndraft: no\n---\n# Bad'), /draft/i);
  assert.throws(() => readMarkdown('---\ntitle: [bad]\n---\n# Bad'), /title/i);
  assert.equal(readMarkdown('---\nrobots: noindex, follow\n---\n# Private').metadata.noindex, true);
  assert.equal(readMarkdown('---\npublished: 2026-10-19T00:30:00+03:00\nupdated: 2026-10-18T22:00:00Z\n---\n# Valid').metadata.published, '2026-10-19T00:30:00+03:00');
  assert.throws(() => readMarkdown('---\npublished: 2026-10-18T22:00:00Z\nupdated: 2026-10-19T00:30:00+03:00\n---\n# Bad ordering'), /precedes/i);
});

test('Markdown links resolve to published pages or immutable upstream sources without changing code', async () => {
  const { exportMarkdown } = await import('../scripts/discovery.mjs');
  const entry = { title: 'Syntax', description: 'Rules.', route: '/spec/syntax/', authority: 'normative-specification', markdown: '# Syntax\n\n[Tool](../docs/tools/ash.md#run) [Core][core]\n\n[core]: core.md\n\n```sequent\nA :=\n  [ P ]\n  ===>\n  Q\n```\n\n```ash\nmain () = "[unchanged](x.md)"\n```', source: { repository: 'https://github.com/example/ash', revision: 'a'.repeat(40), path: 'spec/syntax.md' } };
  const output = exportMarkdown(entry, { sourceRoutes: { 'spec/core.md': '/spec/core/', 'docs/tools/ash.md': '/docs/tools/ash/' } });
  assert.match(output, /\/docs\/tools\/ash\/#run/);
  assert.match(output, /\/spec\/core\//);
  assert.match(output, /```sequent\nA :=\n  \[ P \]\n  ===>\n  Q\n```/);
  assert.match(output, /main \(\) = "\[unchanged\]\(x.md\)"/);
  assert.match(output, /Source revision: a{40}/);
  const rooted = exportMarkdown({ ...entry, markdown: '# Syntax\n\n[Tool](/docs/tools/ash.md#run) [Source](/src/main.rs)' }, { sourceRoutes: { 'docs/tools/ash.md': '/docs/tools/ash/' } });
  assert.match(rooted, /\]\(\/docs\/tools\/ash\/#run\)/);
  assert.match(rooted, /github.com\/example\/ash\/blob\/a{40}\/src\/main.rs/);
});

test('exports preserve source formatting and handle heading links, reference images, and complex destinations', async () => {
  const { exportMarkdown } = await import('../scripts/discovery.mjs');
  const source = '# [Source title](README.md)\n\nA  deliberately spaced paragraph.\n\n[Label](<a file.md> "A title with ]( brackets")\n\n![Diagram][figure]\n\n[figure]: assets/tree.svg\n\n```ash\nmain () = "a  b"\n\n```\n';
  const output = exportMarkdown({ title: 'Source title', route: '/source/', markdown: source, source: { repository: 'https://github.com/example/repo', path: 'docs/source.md', revision: 'a'.repeat(40) } });
  assert.match(output, /A  deliberately spaced paragraph\./);
  assert.doesNotMatch(output, /\nSource title\]\(/);
  assert.doesNotMatch(output, /README.md\)/);
  assert.match(output, /a%20file.md/);
  assert.match(output, /raw.githubusercontent.com\/example\/repo\/a{40}\/docs\/assets\/tree.svg/);
  assert.match(output, /```ash\nmain \(\) = "a  b"\n\n```/);
});

test('generated page Markdown retains syntax labels and meaningful links without navigation', async () => {
  const { readingMarkdown } = await import('../scripts/discovery.mjs');
  const output = readingMarkdown('<html><body><nav>Shell</nav><main><h1>Tour</h1><p>Read <a href="/spec/">the spec</a>.</p><pre data-language="ash"><code>main () = 40 + 2\n</code></pre><aside>Shell aside</aside></main></body></html>');
  assert.match(output, /```ash\nmain \(\) = 40 \+ 2\n```/);
  assert.match(output, /\[the spec\]\(\/spec\/\)/);
  assert.doesNotMatch(output, /Shell/);
  const example = readingMarkdown('<html><body><main><figure class="code-example"><figcaption>answer.ash<span class="code-language">ASH</span></figcaption><pre><code>main () = 42</code></pre><div class="code-result"><span aria-hidden="true">↳</span><samp>42 : Int</samp><span class="result-label">RESULT</span></div></figure></main></body></html>');
  assert.match(example, /```ash\nmain \(\) = 42\n```/);
  assert.match(example, /```text\n42 : Int\n```/);
  assert.doesNotMatch(example, /ashASH|↳|RESULT/);
});

test('SEO enrichment preserves body bytes and uses one source of metadata', async () => {
  const { enrichHtml } = await import('../scripts/discovery.mjs');
  const body = '<body><main><h1>Chapter</h1><pre><code>x &lt; y\n</code></pre></main></body></html>';
  const html = '<!doctype html><html><head><title>Old</title><meta name="description" content="old"><meta name="author" content="Upstream"></head>' + body;
  const entry = { route: '/docs/chapter/', markdownUrl: '/docs/chapter/index.md', title: 'Chapter', description: 'Use "quotes" & <text>.', section: 'Documentation', type: 'TechArticle', authority: 'normative-specification', authors: [{ name: 'Upstream' }], updated: '2026-10-18' };
  const output = enrichHtml(html, entry, { origin: 'https://ash.example' });
  assert.equal(output.slice(output.indexOf('<body>')), body);
  const d = parseHTML(output).document;
  assert.equal(d.querySelectorAll('meta[name=description]').length, 1);
  assert.equal(d.querySelector('meta[name=description]').getAttribute('content'), entry.description);
  assert.equal(d.querySelector('link[rel=canonical]').getAttribute('href'), 'https://ash.example/docs/chapter/');
  assert.equal(d.querySelector('link[type="text/markdown"]').getAttribute('href'), entry.markdownUrl);
  const data = JSON.parse(d.querySelector('script[type="application/ld+json"]').textContent);
  assert.equal(data['@type'], 'TechArticle');
  assert.equal(data.dateModified, '2026-10-18');
  assert.equal(data.author[0].name, 'Upstream');
  assert.ok(!enrichHtml(html, entry).includes('rel="canonical"'));
});

test('discovery indexes omit private pages and work without a public origin', async () => {
  const { createDiscovery, normalizeOrigin } = await import('../scripts/discovery.mjs');
  const pages = [
    { route: '/spec/', title: 'Spec', description: 'Current language.', authority: 'normative-specification', markdown: '# Spec' },
    { route: '/draft/', title: 'Draft', draft: true, markdown: '# Secret' },
    { route: '/hidden/', title: 'Hidden', noindex: true, markdown: '# Hidden' },
    { route: '/search/', title: 'Search', markdown: '# Search' },
    { route: '/404.html', title: 'Missing', markdown: '# Missing' },
  ];
  const local = createDiscovery(pages);
  assert.deepEqual(local.catalogue.pages.map(p => p.url), ['/spec/']);
  assert.equal(local.catalogue.baseUrl, null);
  assert.equal(local.sitemap, null);
  assert.doesNotMatch(local.llms, /Secret|\/draft\/|\/hidden\//);
  assert.match(local.llms, /\/spec\/index.md/);
  const publicSite = createDiscovery(pages, { origin: 'https://ash.example/' });
  assert.match(publicSite.sitemap, /<loc>https:\/\/ash.example\/spec\/<\/loc>/);
  assert.doesNotMatch(publicSite.sitemap, /lastmod|draft|hidden|search|404/);
  assert.match(publicSite.robots, /Sitemap: https:\/\/ash.example\/sitemap.xml/);
  assert.equal(normalizeOrigin(undefined), null);
  assert.throws(() => normalizeOrigin('https://ash.example/path'), /origin/i);
  assert.throws(() => normalizeOrigin('https://user:pass@ash.example'), /origin/i);
});

test('imported Markdown points at immutable original files without making duplicate exports', async () => {
  const { createDiscovery, enrichHtml } = await import('../scripts/discovery.mjs');
  const source = { repository: 'https://github.com/example/ash', revision: 'a'.repeat(40), path: 'spec/syntax.md', rawUrl: `https://raw.githubusercontent.com/example/ash/${'a'.repeat(40)}/spec/syntax.md`, sha256: 'b'.repeat(64) };
  const page = { route: '/spec/syntax/', title: 'Syntax', description: 'Syntax.', source, markdown: '# Syntax', authority: 'normative-specification' };
  const result = createDiscovery([page], { origin: 'https://ash.example' });
  assert.equal(result.exports.size, 0);
  assert.equal(result.catalogue.pages[0].markdown, source.rawUrl);
  assert.equal(result.catalogue.pages[0].markdownSha256, source.sha256);
  assert.match(result.llms, /\]\(https:\/\/raw.githubusercontent.com\/example\/ash\/a{40}\/spec\/syntax.md\)/);
  const d = parseHTML(enrichHtml('<html><head></head><body>Source</body></html>', page)).document;
  assert.equal(d.querySelector('link[type="text/markdown"]').getAttribute('href'), source.rawUrl);
});

test('sources without public Markdown remain discoverable as HTML without Markdown copies or dead links', async () => {
  const { createDiscovery, enrichHtml } = await import('../scripts/discovery.mjs');
  const source = { repository: 'https://github.com/example/private', publicMarkdown: false, revision: 'a'.repeat(40), path: 'content/lesson.md', sha256: 'b'.repeat(64) };
  const page = { route: '/study/latest/', title: 'Study', description: 'Study.', source: { ...source, generated: true }, markdown: '# Study', authority: 'study-material', section: 'Study' };
  const result = createDiscovery([page]);
  assert.equal(result.exports.size, 0);
  assert.equal(result.catalogue.pages[0].markdown, null);
  assert.equal(result.catalogue.pages[0].markdownSha256, null);
  assert.match(result.llms, /\[Study\]\(\/study\/latest\/\)/);
  const d = parseHTML(enrichHtml('<html><head></head><body>Study</body></html>', page)).document;
  assert.equal(d.querySelector('link[type="text/markdown"]'), null);
  assert.equal(d.querySelector('meta[name=robots]').getAttribute('content'), 'index, follow');
});

test('noindex pages are also removed from local search and have no Markdown alternate', async () => {
  const { enrichHtml } = await import('../scripts/discovery.mjs');
  const html = '<html><head><title>Search</title></head><body><main data-pagefind-body><h1>Search</h1></main></body></html>';
  const d = parseHTML(enrichHtml(html, { route: '/search/', title: 'Search', description: 'Search.' })).document;
  assert.equal(d.querySelectorAll('[data-pagefind-body]').length, 0);
  assert.equal(d.querySelector('link[type="text/markdown"]'), null);
  const originalPolicy = html.replace('<title>Search</title>', '<title>Private</title><meta name="robots" content="noindex, follow">');
  const privatePage = parseHTML(enrichHtml(originalPolicy, { route: '/private/', title: 'Private', description: 'Private.' })).document;
  assert.equal(privatePage.querySelector('meta[name=robots]').getAttribute('content'), 'noindex, follow');
  assert.equal(privatePage.querySelector('link[type="text/markdown"]'), null);
});
