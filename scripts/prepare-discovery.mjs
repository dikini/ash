import assert from 'node:assert/strict';
import { readFile, writeFile, readdir, mkdir, rm, stat } from 'node:fs/promises';
import { resolve, join, relative } from 'node:path';
import { pathToFileURL } from 'node:url';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { parseHTML } from 'linkedom';
import { readMarkdown } from './content-metadata.mjs';
import { expandSourceReferences } from './source-markdown.mjs';
import { readSourceConfig, repositoryWebUrl, sha256 } from './source-config.mjs';
import { createDiscovery, enrichHtml, isReadingPage, readingMarkdown, normalizeOrigin, hasHtmlNoindex } from './discovery.mjs';

async function walk(folder, extension) {
  const files = [];
  for (const item of await readdir(folder, { withFileTypes: true })) {
    const file = join(folder, item.name);
    if (item.isDirectory()) files.push(...await walk(file, extension));
    else if (item.name.endsWith(extension)) files.push(file);
  }
  return files.sort();
}
const json = async path => JSON.parse(await readFile(path, 'utf8'));
const labels = { spec: ['Specification', 'normative-specification'], tools: ['Tools', 'tool-documentation'], design: ['Design notes', 'informative-design'], decisions: ['Architectural decisions', 'architectural-decision'] };

export async function prepareDiscovery(output = 'dist', { origin = process.env.SITE_URL } = {}) {
  origin = normalizeOrigin(origin);
  const root = resolve(output);
  const { config, configHash } = readSourceConfig();
  const publication = await json('.cache/ash-spec/publication.json');
  const study = await json('.cache/ash-doc/publication.json');
  const snapshot = await json(join(root, config.study.route, 'snapshot.json'));
  assert.equal(publication.source.configHash, configHash, 'Stale reference metadata');
  assert.equal(study.configHash, configHash, 'Stale study metadata');
  assert.equal(study.sourceRevision, snapshot.sourceRevision, 'Study metadata revision differs from HTML');
  const execute = promisify(execFile);
  const siteRevision = (await execute('git', ['rev-parse', 'HEAD'])).stdout.trim();
  const siteDirty = Boolean((await execute('git', ['status', '--porcelain'])).stdout.trim());
  const records = new Map();
  const sourceRoutes = {};
  for (const definition of config.reference.groups) {
    const group = definition.id === 'spec' ? publication : publication[definition.id];
    for (const page of group.pages) sourceRoutes[`${group.directory}/${page.file}`] = page.route;
  }
  for (const definition of config.reference.groups) {
    const group = definition.id === 'spec' ? publication : publication[definition.id];
    const [section, authority] = labels[definition.id];
    for (const page of group.pages) {
      assert.ok(page.metadata && typeof page.markdown === 'string', `Missing source metadata: ${page.route}`);
      const path = `${group.directory}/${page.file}`;
      records.set(page.route, { ...page.metadata, title: page.title, markdown: page.markdown, route: page.route, section, authority, type: 'TechArticle', sourceRoutes,
        source: { repository: publication.source.repository, revision: publication.revision, path, sha256: page.sourceSha256, publicMarkdown: config.repositories[config.reference.source].publicMarkdown ?? true, url: `${repositoryWebUrl(publication.source.repository)}/blob/${publication.revision}/${path}`, ...(config.repositories[config.reference.source].publicMarkdown === false ? {} : { rawUrl: `${repositoryWebUrl(publication.source.repository).replace('https://github.com/', 'https://raw.githubusercontent.com/')}/${publication.revision}/${path}` }) },
      });
    }
  }
  const studyRoutes = Object.fromEntries(study.pages.filter(page => !page.metadata.draft && page.source.path).map(page => [page.source.path, page.route]));
  for (const page of study.pages) records.set(page.route, { ...page.metadata, markdown: page.markdown, route: page.route, source: page.source, sourceRoutes: studyRoutes, section: 'Study', authority: 'study-material', type: 'TechArticle', ashRevision: snapshot.ashRevision });
  for (const file of await walk(resolve('content'), '.md')) {
    const input = await readFile(file, 'utf8');
    const page = readMarkdown(expandSourceReferences(input, config));
    const route = `/${page.metadata.sourceMetadata.route}/`;
    assert.ok(!records.has(route), `Duplicate source route: ${route}`);
    records.set(route, { ...page.metadata, markdown: page.markdown, route, section: page.metadata.sourceMetadata.section, authority: 'website-guidance', type: ['Updates', 'Developer Diaries'].includes(page.metadata.sourceMetadata.section) ? 'Article' : 'WebPage',
      source: { kind: 'website', revision: siteRevision, dirty: siteDirty, path: relative(resolve('.'), file), sha256: sha256(input) },
    });
  }
  const entries = [];
  const documents = [];
  for (const file of await walk(root, '.html')) {
    const path = '/' + relative(root, file).split('\\').join('/');
    const route = path.replace(/index\.html$/, '');
    const html = await readFile(file, 'utf8');
    const { document } = parseHTML(html);
    let page = records.get(route);
    if (!page) {
      const input = readingMarkdown(html);
      const metadata = readMarkdown(input).metadata;
      let astroPath;
      for (const candidate of [`src/pages/${route === '/' ? 'index' : route.slice(1, -1)}.astro`, `src/pages/${route.slice(1)}index.astro`]) {
        try { if ((await stat(candidate)).isFile()) { astroPath = candidate; break; } } catch { /* route may be generated by an index component */ }
      }
      page = { ...metadata, title: document.title.replace(/ · Ash$/, ''), description: document.querySelector('meta[name="description"]')?.getAttribute('content') ?? metadata.description, route, markdown: input, section: document.querySelector('.eyebrow')?.textContent.trim() ?? 'Ash', authority: 'website-guidance', type: 'WebPage',
        source: { kind: 'website', revision: siteRevision, dirty: siteDirty, ...(astroPath ? { path: astroPath, sha256: sha256(await readFile(astroPath)) } : {}) },
      };
    }
    if (!page.authors?.length) page.authors = [...document.querySelectorAll('head meta[name="author"]')].map(node => ({ name: node.getAttribute('content') })).filter(author => author.name);
    page.noindex ||= hasHtmlNoindex(document);
    if (page.draft) throw new Error(`Draft unexpectedly published: ${route}`);
    entries.push(page);
    documents.push({ file, html, page });
  }
  for (const page of records.values()) if (!page.draft) assert.ok(entries.some(entry => entry.route === page.route), `Source document missing from rendered output: ${page.route}`);
  // Re-running on an output directory must not leave exports for newly hidden pages.
  try {
    const previous = await json(join(root, 'content-index.json'));
    for (const page of previous.pages) if (typeof page.markdown === 'string' && /^\/[a-zA-Z0-9/.-]+\.md$/.test(page.markdown) && !page.markdown.split('/').includes('..')) await rm(join(root, page.markdown), { force: true });
  } catch (error) { if (error.code !== 'ENOENT') throw error; }
  const discovery = createDiscovery(entries, { origin, sourceConfigHash: configHash });
  for (const [path, markdown] of discovery.exports) {
    await mkdir(join(root, path, '..'), { recursive: true });
    await writeFile(join(root, path), markdown);
  }
  for (const { file, html, page } of documents) await writeFile(file, enrichHtml(html, page, { origin }));
  await writeFile(join(root, 'content-index.json'), JSON.stringify(discovery.catalogue, null, 2) + '\n');
  await writeFile(join(root, 'llms.txt'), discovery.llms);
  await writeFile(join(root, 'robots.txt'), discovery.robots);
  if (discovery.sitemap) await writeFile(join(root, 'sitemap.xml'), discovery.sitemap);
  else await rm(join(root, 'sitemap.xml'), { force: true });
  console.log(`Prepared metadata for ${entries.filter(isReadingPage).length} reading pages, with ${discovery.exports.size} generated Markdown files; public imports link to pinned originals${origin ? ` at ${origin}` : '; set SITE_URL to enable canonical URLs and sitemap'}.`);
  return discovery;
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) await prepareDiscovery(process.argv[2]);
