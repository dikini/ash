import test from "node:test";
import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { createHash } from 'node:crypto';
import { parseHTML } from 'linkedom';

const pages = [
  "",
  "discover",
  "learn",
  "learn/tour",
  "docs",
  "docs/install",
  "status",
  "updates",
  "diaries",
  "updates/first-version",
  "contribute",
  "study",
  "search",
];

test("developer diaries are discoverable and unpublished drafts stay private", () => {
  assert.ok(existsSync("dist/diaries/index.html"), "Missing diary landing page");
  const html = readFileSync("dist/diaries/index.html", "utf8");
  assert.match(readFileSync("dist/updates/index.html", "utf8"), /href="\/diaries\/"/);
  assert.doesNotMatch(html, /Draft: first developer diary/);
  assert.ok(!existsSync("dist/diaries/first-entry/index.html"));
});

test("all reader entry paths have a statically generated page", () => {
  for (const page of pages) {
    assert.ok(
      existsSync(`dist/${page ? `${page}/` : ""}index.html`),
      `Missing route /${page}`,
    );
  }
});

test("study serves the latest source snapshot with explicit targets and no release claim", () => {
  assert.ok(
    existsSync("dist/study/index.html"),
    "Study landing page must exist",
  );
  const html = readFileSync("dist/study/index.html", "utf8");
  assert.match(html, /github\.com\/dikini\/ash-doc\/tree\/[a-f0-9]{40}/);
  assert.match(html, /not a published book edition/i);
  assert.match(html, /href="\/study\/latest\/"/);
  const snapshot = JSON.parse(readFileSync('dist/study/latest/snapshot.json', 'utf8'));
  assert.match(snapshot.sourceRevision, /^[a-f0-9]{40}$/);
  assert.match(snapshot.ashRevision, /^[a-f0-9]{40}$/);
  assert.ok(snapshot.lessonCount > 0);
  assert.ok(existsSync('dist/study/latest/learn-what-is-an-ash-program.html'));
  assert.doesNotMatch(html, /href="[^"]+\.(?:pdf|epub)"/);
});

test('build manifest records the exact published source and pinned runtime identities', () => {
  const manifest = JSON.parse(readFileSync('dist/build-manifest.json', 'utf8'));
  const config = JSON.parse(readFileSync('sources.json', 'utf8'));
  const reference = JSON.parse(readFileSync('.cache/ash-spec/publication.json', 'utf8'));
  const study = JSON.parse(readFileSync('dist/study/latest/snapshot.json', 'utf8'));
  assert.equal(manifest.version, 1);
  assert.match(manifest.site.revision, /^[a-f0-9]{40}$/);
  assert.equal(typeof manifest.site.dirty, 'boolean');
  assert.equal(manifest.sources[config.reference.source].revision, reference.revision);
  assert.equal(manifest.sources[config.study.source].revision, study.sourceRevision);
  assert.equal(manifest.sources[config.study.source].ashRevision, study.ashRevision);
  assert.equal(manifest.checkpoint.revision, config.checkpoint.revision);
  assert.equal(manifest.inputs.sourceConfigSha256, createHash('sha256').update(readFileSync('sources.json')).digest('hex'));
  for (const [name, source] of Object.entries(config.renderers)) {
    assert.equal(manifest.renderers[name].revision, config.repositories[source].ref);
    assert.match(manifest.renderers[name].archiveSha256, /^[a-f0-9]{64}$/);
  }
  for (const path of ['docs/install', 'status', 'discover', 'contribute', 'docs']) {
    const html = readFileSync(`dist/${path}/index.html`, 'utf8');
    assert.doesNotMatch(html, /\{\{(?:checkpoint|source|repository)\./);
    assert.ok(html.includes(config.checkpoint.revision), `${path}: missing compiler checkpoint`);
  }
});

test('public reading pages have consistent metadata, Markdown exports, and exact source identities', () => {
  const catalogueBytes = readFileSync('dist/content-index.json');
  const catalogue = JSON.parse(catalogueBytes);
  const manifest = JSON.parse(readFileSync('dist/build-manifest.json', 'utf8'));
  assert.ok(catalogue.pages.length > 200);
  assert.equal(manifest.discovery.catalogueSha256, createHash('sha256').update(catalogueBytes).digest('hex'));
  assert.equal(catalogue.sourceConfigSha256, manifest.inputs.sourceConfigSha256);
  for (const page of catalogue.pages) {
    assert.ok(!['/search/', '/404.html', '/diaries/first-entry/'].includes(page.url));
    const html = readFileSync(`dist${page.url}${page.url.endsWith('/') ? 'index.html' : ''}`, 'utf8');
    const document = parseHTML(html).document;
    assert.equal(document.querySelector('meta[name=description]').getAttribute('content'), page.description, page.url);
    assert.equal(document.querySelector('link[type="text/markdown"]')?.getAttribute('href') ?? null, page.markdown);
    assert.ok(document.querySelector('script[type="application/ld+json"]'));
    if (page.source.rawUrl) {
      assert.equal(page.markdown, page.source.rawUrl);
      assert.equal(page.markdownSha256, page.source.sha256);
      assert.match(page.markdownSha256, /^[a-f0-9]{64}$/);
      assert.ok(page.markdown.includes(`/${page.source.revision}/${page.source.path}`));
      const alias = page.url.endsWith('/') ? page.url + 'index.md' : page.url.replace(/\.html$/, '.md');
      assert.ok(!existsSync(`dist${alias}`), `Duplicate Markdown URI: ${alias}`);
    } else if (page.markdown) {
      const md = readFileSync(`dist${page.markdown}`, 'utf8');
      assert.equal(page.markdownSha256, createHash('sha256').update(md).digest('hex'));
      assert.doesNotMatch(md, /\{\{(?:source|checkpoint|route)\./);
    } else {
      assert.equal(page.source.publicMarkdown, false);
      assert.equal(page.markdownSha256, null);
      const alias = page.url.endsWith('/') ? page.url + 'index.md' : page.url.replace(/\.html$/, '.md');
      assert.ok(!existsSync(`dist${alias}`), `Unexpected private-source export: ${alias}`);
    }
    assert.match(page.source.revision, /^[a-f0-9]{40}$/);
  }
  const decision = catalogue.pages.find(page => page.id === 'AD-0001');
  assert.equal(decision.status, 'accepted');
  assert.equal(decision.authority, 'architectural-decision');
  const grammar = catalogue.pages.find(page => page.url === '/spec/syntax/');
  assert.match(grammar.markdown, /raw.githubusercontent.com\/dikini\/ash-lang\/[a-f0-9]{40}\/spec\/syntax.md/);
  const study = catalogue.pages.find(page => page.url.endsWith('/learn-what-is-an-ash-program.html'));
  assert.equal(study.ashRevision, manifest.sources['ash-doc'].ashRevision);
  const sourceConfig = JSON.parse(readFileSync('sources.json', 'utf8'));
  if (sourceConfig.repositories['ash-doc'].publicMarkdown === false) assert.equal(study.markdown, null);
  else assert.match(study.markdown, /raw.githubusercontent.com\/dikini\/ash-doc\/[a-f0-9]{40}\/content\/learn\/what-is-an-ash-program\/README.md/);
  const llms = readFileSync('dist/llms.txt', 'utf8');
  assert.match(llms, /content-index.json/);
  assert.doesNotMatch(llms, /diaries\/first-entry/);
  assert.equal(parseHTML(readFileSync('dist/search/index.html', 'utf8')).document.querySelector('meta[name=robots]').getAttribute('content'), 'noindex, follow');
});
