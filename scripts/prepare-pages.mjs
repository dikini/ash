import assert from 'node:assert/strict';
import { cp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { join, resolve, relative, sep } from 'node:path';
import { pathToFileURL } from 'node:url';
import { parseHTML } from 'linkedom';
import { markdownParser } from './content-metadata.mjs';
import { sha256 } from './source-config.mjs';
import { checkSite } from './check-links.mjs';
import { artifactFiles as filesIn, hashArtifact as hashPagesArtifact } from './artifact-integrity.mjs';

export { hashPagesArtifact };

function rewriteMarkdown(input, rewriteUrl) {
  const edits = [];
  function visit(node) {
    if (node.url && node.position) {
      const start = node.position.start.offset;
      const raw = input.slice(start, node.position.end.offset);
      // Locate destinations after the label; leave fenced and inline code alone.
      let index = raw.indexOf('[') + 1, depth = 1;
      for (; index < raw.length && depth; index++) {
        if (raw[index] === '\\') index++;
        else if (raw[index] === '[') depth++;
        else if (raw[index] === ']') depth--;
      }
      if (raw[index] === '(' || raw[index] === ':') {
        index++;
        while (/\s/.test(raw[index] ?? '') && index < raw.length) index++;
        if (raw[index] === '<') index++;
        // The source spelling may escape parentheses; only the URL prefix changes.
        const prefix = node.url.startsWith('/') && !node.url.startsWith('//') ? '/' : null;
        const value = rewriteUrl(node.url);
        if (value !== node.url) {
          const oldPrefix = prefix ?? new URL(node.url).origin;
          const newPrefix = value.slice(0, value.length - node.url.length + oldPrefix.length);
          edits.push({ start: start + index, end: start + index + oldPrefix.length, value: newPrefix });
        }
      }
    }
    for (const child of node.children ?? []) visit(child);
  }
  visit(markdownParser.parse(input));
  let result = input;
  for (const edit of edits.sort((a, b) => b.start - a.start)) result = result.slice(0, edit.start) + edit.value + result.slice(edit.end);
  return result.replace(/^Page: (\S+)/m, (_, url) => `Page: ${rewriteUrl(url)}`);
}

export async function preparePages({ source = 'dist', destination = '.cache/github-pages', url = process.env.PAGES_URL } = {}) {
  const site = new URL(url);
  assert.ok(site.protocol === 'https:' && !site.username && !site.password && !site.search && !site.hash, 'Pages URL must be HTTPS without credentials, query, or fragment');
  assert.match(site.pathname, /^\/[a-zA-Z0-9_-]+\/$/, 'Pages URL must contain a single project path, e.g. /ash/');
  const prefix = site.pathname.slice(0, -1);
  const sourceRoot = resolve(source);
  const mount = resolve(destination);
  const output = join(mount, prefix);
  assert.ok(sourceRoot !== mount && !mount.startsWith(sourceRoot + sep) && !sourceRoot.startsWith(mount + sep), 'Pages staging directory must be separate from the build');
  let sourceReceipt;
  try { sourceReceipt = JSON.parse(await readFile(join(sourceRoot, 'refresh-report.json'), 'utf8')); }
  catch (error) { if (error.code !== 'ENOENT') throw error; }
  if (sourceReceipt) assert.equal(sourceReceipt.artifactSha256, await hashPagesArtifact(sourceRoot), 'Source output changed after verification; run a new local refresh');
  const rewriteUrl = value => {
    if (value === site.origin) return site.origin + prefix;
    if (value.startsWith(site.origin + '/')) return site.origin + prefix + value.slice(site.origin.length);
    if (value.startsWith('/') && !value.startsWith('//')) return prefix + value;
    return value;
  };
  const rewriteJson = value => {
    if (typeof value === 'string') return rewriteUrl(value);
    if (Array.isArray(value)) return value.map(rewriteJson);
    if (value && typeof value === 'object') return Object.fromEntries(Object.entries(value).map(([key, child]) => [key, rewriteJson(child)]));
    return value;
  };
  const rewriteCss = css => css.replace(/url\(\s*(["']?)(\/[^\s)'";]*)\1\s*\)/g, (_, quote, url) => `url(${quote}${rewriteUrl(url)}${quote})`);
  await rm(output, { recursive: true, force: true });
  await mkdir(mount, { recursive: true });
  await cp(sourceRoot, output, { recursive: true });
  for (const file of await filesIn(output)) {
    if (file.endsWith('.html')) {
      const { document } = parseHTML(await readFile(file, 'utf8'));
      for (const node of document.querySelectorAll('[href], [src], [action], [poster], [data-src]')) {
        for (const attribute of ['href', 'src', 'action', 'poster', 'data-src']) {
          if (node.hasAttribute(attribute)) node.setAttribute(attribute, rewriteUrl(node.getAttribute(attribute)));
        }
      }
      for (const node of document.querySelectorAll('meta[property="og:url"], meta[property="og:image"], meta[name="twitter:image"]')) node.setAttribute('content', rewriteUrl(node.getAttribute('content')));
      for (const node of document.querySelectorAll('script[type="application/ld+json"]')) node.textContent = JSON.stringify(rewriteJson(JSON.parse(node.textContent))).replaceAll('<', '\\u003c');
      for (const node of document.querySelectorAll('style')) node.textContent = rewriteCss(node.textContent);
      for (const node of document.querySelectorAll('[style]')) node.setAttribute('style', rewriteCss(node.getAttribute('style')));
      await writeFile(file, document.toString());
    } else if (file.endsWith('.css')) {
      await writeFile(file, rewriteCss(await readFile(file, 'utf8')));
    } else if (file.endsWith('.md')) {
      await writeFile(file, rewriteMarkdown(await readFile(file, 'utf8'), rewriteUrl));
    }
  }
  for (const name of ['llms.txt', 'robots.txt', 'sitemap.xml']) {
    const file = join(output, name);
    const content = await readFile(file, 'utf8');
    await writeFile(file, content.replaceAll(site.origin + '/', site.origin + prefix + '/'));
  }
  const catalogueFile = join(output, 'content-index.json');
  const catalogue = JSON.parse(await readFile(catalogueFile, 'utf8'));
  assert.equal(catalogue.baseUrl, site.origin, 'Build with SITE_URL set to the Pages origin before preparing Pages');
  catalogue.baseUrl = site.origin + prefix;
  catalogue.buildManifest = rewriteUrl(catalogue.buildManifest);
  for (const page of catalogue.pages) {
    const markdown = page.markdown;
    if (markdown?.startsWith('/')) page.markdownSha256 = sha256(await readFile(join(output, markdown)));
    for (const key of ['url', 'canonical', 'markdown']) if (page[key]) page[key] = rewriteUrl(page[key]);
    for (const related of page.related ?? []) related.url = rewriteUrl(related.url);
  }
  const catalogueBytes = JSON.stringify(catalogue, null, 2) + '\n';
  await writeFile(catalogueFile, catalogueBytes);
  const manifestFile = join(output, 'build-manifest.json');
  const originalManifest = await readFile(manifestFile);
  const manifest = JSON.parse(originalManifest);
  manifest.hosting = { provider: 'github-pages', url: site.href, basePath: prefix };
  Object.assign(manifest.discovery, {
    baseUrl: catalogue.baseUrl,
    catalogueSha256: sha256(catalogueBytes),
    llmsSha256: sha256(await readFile(join(output, 'llms.txt'))),
    markdownSetSha256: sha256(JSON.stringify(catalogue.pages.map(page => [page.url, page.markdown, page.markdownSha256]))),
  });
  await writeFile(manifestFile, JSON.stringify(manifest, null, 2) + '\n');
  await writeFile(join(output, '.nojekyll'), '');
  const errors = await checkSite(mount);
  assert.deepEqual(errors, [], 'Pages links must resolve under the project path');
  const receiptFile = join(output, 'refresh-report.json');
  let receipt;
  try { receipt = JSON.parse(await readFile(receiptFile, 'utf8')); }
  catch (error) { if (error.code !== 'ENOENT') throw error; }
  if (receipt) {
    assert.equal(receipt.buildManifestSha256, sha256(originalManifest), 'Refresh receipt does not match the source build manifest');
    receipt.sourceBuildManifestSha256 = receipt.buildManifestSha256;
    receipt.buildManifestSha256 = sha256(await readFile(manifestFile));
    receipt.checks = [...new Set([...receipt.checks, 'prepare:pages'])];
    receipt.preparedAt = new Date().toISOString();
    receipt.artifactSha256 = await hashPagesArtifact(output);
    await writeFile(receiptFile, JSON.stringify(receipt, null, 2) + '\n');
  }
  console.log(`Prepared GitHub Pages artifact at ${relative(resolve('.'), output)} for ${site.href}; mounted links verified.`);
  return output;
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) await preparePages();
