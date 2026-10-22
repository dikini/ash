import assert from 'node:assert/strict';
import { test } from 'node:test';
import { mkdtemp, mkdir, readFile, writeFile, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { createHash } from 'node:crypto';

const hash = value => createHash('sha256').update(value).digest('hex');

test('Pages artifact serves links and metadata under the project path without changing source output', async () => {
  const { preparePages } = await import('../scripts/prepare-pages.mjs');
  const root = await mkdtemp(join(tmpdir(), 'ash-pages-'));
  try {
    const source = join(root, 'dist');
    await mkdir(join(source, 'docs'), { recursive: true });
    const html = '<!DOCTYPE html><html><head><link rel="stylesheet" href="/theme.css"><link rel="canonical" href="https://dikini.github.io/docs/"><script type="application/ld+json">{"url":"https://dikini.github.io/docs/"}</script></head><body><a href="/docs/#start">Docs</a><a href="//example.com/">External</a><img src="/icon.svg"><pre>"/example"</pre><script>const example = "/example";</script><h1 id="start">Docs</h1></body></html>';
    await writeFile(join(source, 'index.html'), html);
    await writeFile(join(source, 'docs/index.html'), html);
    await writeFile(join(source, 'theme.css'), '@import url("/tokens.css"); .icon { background: url(/icon.svg); }');
    await writeFile(join(source, 'tokens.css'), ':root { --color: red; }');
    await writeFile(join(source, 'icon.svg'), '<svg/>');
    await writeFile(join(source, 'index.md'), '# Home\n\nPage: https://dikini.github.io/\n\n[Docs](/docs/)\n\n```sh\ncat /example\n```\n');
    await writeFile(join(source, 'llms.txt'), '[Home](https://dikini.github.io/index.md)\n');
    await writeFile(join(source, 'robots.txt'), 'User-agent: *\nAllow: /\nSitemap: https://dikini.github.io/sitemap.xml\n');
    await writeFile(join(source, 'sitemap.xml'), '<urlset><url><loc>https://dikini.github.io/</loc></url></urlset>');
    await writeFile(join(source, 'content-index.json'), JSON.stringify({ baseUrl: 'https://dikini.github.io', buildManifest: '/build-manifest.json', pages: [{ url: '/', canonical: 'https://dikini.github.io/', markdown: '/index.md', markdownSha256: 'old' }] }));
    await writeFile(join(source, 'build-manifest.json'), JSON.stringify({ discovery: {} }));
    const output = await preparePages({ source, destination: join(root, 'pages'), url: 'https://dikini.github.io/ash/' });
    const result = await readFile(join(output, 'index.html'), 'utf8');
    assert.match(result, /href="\/ash\/docs\/#start"/);
    assert.match(result, /src="\/ash\/icon.svg"/);
    assert.match(result, /https:\/\/dikini.github.io\/ash\/docs\//);
    assert.match(result, /href="\/\/example.com\/"/);
    assert.match(result, /<pre>"\/example"<\/pre>/);
    assert.match(result, /const example = "\/example"/);
    assert.equal(await readFile(join(source, 'index.html'), 'utf8'), html);
    assert.match(await readFile(join(output, 'theme.css'), 'utf8'), /url\("\/ash\/tokens.css"\)/);
    const markdown = await readFile(join(output, 'index.md'), 'utf8');
    assert.match(markdown, /\[Docs\]\(\/ash\/docs\/\)/);
    assert.match(markdown, /cat \/example/);
    assert.match(markdown, /Page: https:\/\/dikini.github.io\/ash\//);
    const catalogueBytes = await readFile(join(output, 'content-index.json'));
    const catalogue = JSON.parse(catalogueBytes);
    assert.equal(catalogue.baseUrl, 'https://dikini.github.io/ash');
    assert.equal(catalogue.pages[0].markdownSha256, hash(markdown));
    assert.equal(catalogue.pages[0].url, '/ash/');
    const manifest = JSON.parse(await readFile(join(output, 'build-manifest.json'), 'utf8'));
    assert.equal(manifest.discovery.catalogueSha256, hash(catalogueBytes));
    assert.match(await readFile(join(output, 'sitemap.xml'), 'utf8'), /https:\/\/dikini.github.io\/ash\//);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test('Pages preparation rejects unsafe paths before copying or removing files', async () => {
  const { preparePages } = await import('../scripts/prepare-pages.mjs');
  await assert.rejects(preparePages({ url: 'https://dikini.github.io/../' }), /project path/i);
  await assert.rejects(preparePages({ url: 'https://dikini.github.io/ash/?x=1' }), /Pages URL/i);
});
