import { parseHTML } from 'linkedom';
import { markdownParser } from './content-metadata.mjs';
import { repositoryWebUrl, sha256 } from './source-config.mjs';

const localBase = 'https://site.invalid';
const escape = value => String(value).replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;');
const short = value => value.replace(/\s+/g, ' ').trim();
export const markdownUrl = route => route.endsWith('/') ? `${route}index.md` : route.replace(/\.html$/, '.md');
export const contentMarkdownUrl = page => page.source?.publicMarkdown === false ? null : page.source?.rawUrl ?? markdownUrl(page.route);
const publicLink = (url, origin) => origin && url.startsWith('/') ? origin + url : url;
export const isReadingPage = page => !page.draft && !page.noindex && !/\b(?:noindex|none)\b/i.test(page.robots ?? '') && !['/search/', '/404.html', '/404/'].includes(page.route);
export const htmlRobots = document => [...document.head.querySelectorAll('meta[name]')].filter(node => node.getAttribute('name').toLowerCase() === 'robots').map(node => node.getAttribute('content') ?? '').join(', ');
export const hasHtmlNoindex = document => /\b(?:noindex|none)\b/i.test(htmlRobots(document));

export function normalizeOrigin(value) {
  if (!value) return null;
  let url;
  try { url = new URL(value); } catch { throw new Error('SITE_URL must be an HTTP(S) origin'); }
  if (!['https:', 'http:'].includes(url.protocol) || url.username || url.password || url.pathname !== '/' || url.search || url.hash) throw new Error('SITE_URL must be an HTTP(S) origin without credentials, path, query, or fragment');
  return url.origin;
}

export function resolveContentLink(href, entry, sourceRoutes = {}, image = false) {
  if (!href || /^(?:[a-z][a-z\d+.-]*:|\/\/|#)/i.test(href)) return href;
  if (!entry.source?.repository || !entry.source?.path) {
    const target = new URL(href, localBase + entry.route);
    return target.pathname + target.search + target.hash;
  }
  const target = new URL(href, `https://source.invalid/${entry.source.path}`);
  const path = decodeURIComponent(target.pathname.slice(1));
  const route = sourceRoutes[path] ?? sourceRoutes[`${path.replace(/\/$/, '')}/README.md`];
  if (route) return route + target.search + target.hash;
  const repo = repositoryWebUrl(entry.source.repository);
  const base = image && repo.startsWith('https://github.com/') ? repo.replace('https://github.com/', 'https://raw.githubusercontent.com/') : `${repo}/blob`;
  return `${base}/${entry.source.revision}${target.pathname}${target.search}${target.hash}`;
}

export function exportMarkdown(entry, { sourceRoutes = {}, origin } = {}) {
  const tree = markdownParser.parse(entry.markdown);
  const edits = [];
  const first = tree.children.find(node => node.type === 'heading' && node.depth === 1);
  if (first) edits.push({ start: first.position.start.offset, end: first.position.end.offset, value: '' });
  const imageReferences = new Set();
  function collectImages(node) {
    if (node.type === 'imageReference') imageReferences.add(node.identifier);
    for (const child of node.children ?? []) collectImages(child);
  }
  collectImages(tree);
  function visit(node) {
    if (node === first) return;
    if ('url' in node && node.position) {
      const offset = node.position.start.offset;
      const raw = entry.markdown.slice(offset, node.position.end.offset);
      let index = raw.indexOf('[') + 1, depth = 1;
      for (; index < raw.length && depth; index++) {
        if (raw[index] === '\\') index++;
        else if (raw[index] === '[') depth++;
        else if (raw[index] === ']') depth--;
      }
      // URL-bearing nodes are inline links/images or reference definitions.
      if (raw[index] === '(' || raw[index] === ':') {
        index++;
        while (/\s/.test(raw[index] ?? '') && index < raw.length) index++;
        const angled = raw[index] === '<';
        if (angled) index++;
        const start = index;
        depth = 0;
        for (; index < raw.length; index++) {
          const char = raw[index];
          if (char === '\\') { index++; continue; }
          if (angled ? char === '>' : /\s/.test(char) || (char === ')' && depth === 0)) break;
          if (char === '(') depth++;
          if (char === ')') depth--;
        }
        const url = resolveContentLink(node.url, entry, sourceRoutes, node.type === 'image' || imageReferences.has(node.identifier));
        edits.push({ start: offset + start, end: offset + index, value: angled ? url : url.replaceAll(' ', '%20').replaceAll('(', '%28').replaceAll(')', '%29') });
      }
    }
    for (const child of node.children ?? []) visit(child);
  }
  visit(tree);
  let body = entry.markdown;
  for (const edit of edits.sort((a, b) => b.start - a.start)) body = body.slice(0, edit.start) + edit.value + body.slice(edit.end);
  const url = origin ? origin + entry.route : entry.route;
  const details = [`Page: ${url}`, `Authority: ${entry.authority ?? 'website-guidance'}`];
  if (entry.id) details.push(`Document ID: ${entry.id}`);
  if (entry.status) details.push(`Status: ${entry.status}`);
  if (entry.source) {
    if (entry.source.url) details.push(`Source: ${entry.source.url}`);
    if (entry.source.rawUrl) details.push(`Original Markdown: ${entry.source.rawUrl}`);
    details.push(`Source revision: ${entry.source.revision}`);
  }
  if (entry.ashRevision) details.push(`Compiler target: ${entry.ashRevision}`);
  if (entry.published) details.push(`Published: ${entry.published}`);
  if (entry.updated) details.push(`Updated: ${entry.updated}`);
  if (entry.authors?.length) details.push(`Authors: ${entry.authors.map(author => author.name).join(', ')}`);
  return `# ${entry.title}\n\n> ${short(entry.description ?? entry.title)}\n\n${details.join('  \n')}\n\n${body.trim()}\n`;
}

// Generated Astro pages have no Markdown source. Convert only their reading
// region; imported documents always use their source Markdown instead.
export function readingMarkdown(html) {
  const { document } = parseHTML(html);
  const root = document.querySelector('[data-pagefind-body]') ?? document.querySelector('main');
  if (!root) throw new Error('Missing reading content');
  function text(node) {
    if (node.nodeType === 3) return node.textContent.replace(/\s+/g, ' ');
    const tag = node.localName;
    if (['script', 'style', 'nav', 'aside', 'button', 'svg'].includes(tag) || node.hasAttribute?.('data-pagefind-ignore') || node.getAttribute?.('aria-hidden') === 'true' || node.matches?.('.code-language,.result-label')) return '';
    const children = () => [...node.childNodes].map(text).join('');
    if (tag === 'pre') {
      const code = node.querySelector('code') ?? node;
      const lang = node.getAttribute('data-language') ?? code.className.match?.(/language-([\w-]+)/)?.[1] ?? node.closest('figure')?.querySelector('.code-language')?.textContent.toLowerCase() ?? '';
      const value = code.textContent;
      const fence = '`'.repeat(Math.max(3, ...[...value.matchAll(/`+/g)].map(match => match[0].length + 1)));
      return `\n\n${fence}${lang}\n${value.replace(/\n$/, '')}\n${fence}\n\n`;
    }
    if (tag === 'code') {
      const value = node.textContent;
      const fence = '`'.repeat(Math.max(1, ...[...value.matchAll(/`+/g)].map(match => match[0].length + 1)));
      const pad = /^`|`$/.test(value) ? ' ' : '';
      return `${fence}${pad}${value}${pad}${fence}`;
    }
    if (tag === 'samp') return `\n\n\`\`\`text\n${node.textContent}\n\`\`\`\n\n`;
    if (tag === 'figcaption') return `\n\n${children().trim()}\n\n`;
    if (/^h[1-6]$/.test(tag)) return `\n\n${'#'.repeat(Number(tag[1]))} ${short(children())}\n\n`;
    if (tag === 'a') return node.getAttribute('href') ? `[${short(children())}](${node.getAttribute('href')})` : children();
    if (tag === 'img') return `![${node.getAttribute('alt') ?? ''}](${node.getAttribute('src') ?? ''})`;
    if (tag === 'strong' || tag === 'b') return `**${children()}**`;
    if (tag === 'em' || tag === 'i') return `*${children()}*`;
    if (tag === 'li') return `\n- ${children().trim().replace(/\n/g, '\n  ')}\n`;
    if (tag === 'br') return '\n';
    if (tag === 'hr') return '\n\n---\n\n';
    if (tag === 'table') {
      const rows = [...node.querySelectorAll('tr')].map(row => [...row.querySelectorAll('th,td')].map(cell => short(text(cell)).replaceAll('|', '\\|')));
      if (!rows.length) return '';
      rows.splice(1, 0, rows[0].map(() => '---'));
      return '\n\n' + rows.map(row => `| ${row.join(' | ')} |`).join('\n') + '\n\n';
    }
    const value = children();
    return ['p', 'div', 'section', 'article', 'ul', 'ol', 'details', 'summary', 'header', 'blockquote'].includes(tag) ? `\n\n${value}\n\n` : value;
  }
  return text(root).trim() + '\n';
}

export function enrichHtml(html, entry, { origin } = {}) {
  origin = normalizeOrigin(origin);
  const { document } = parseHTML(html);
  const head = document.head;
  const existingRobots = htmlRobots(document);
  const existingNoindex = hasHtmlNoindex(document);
  for (const node of head.querySelectorAll('title, meta[name="description"], meta[name="author"], meta[name="robots"], meta[name^="twitter:"], meta[property^="og:"], link[rel="canonical"], link[type="text/markdown"], link[rel="sitemap"], link[data-ash-discovery], script[data-ash-discovery]')) node.remove();
  const indexable = isReadingPage(entry) && !existingNoindex;
  const canonical = origin && indexable ? origin + entry.route : null;
  const title = /(?:· Ash|^Ash(?:$| —))/.test(entry.title) ? entry.title : `${entry.title} · Ash`;
  let additions = `<title>${escape(title)}</title><meta name="description" content="${escape(entry.description)}">`;
  const directives = new Set(`${existingRobots},${entry.robots ?? ''}`.toLowerCase().split(',').map(value => value.trim()).filter(Boolean));
  if (directives.has('none')) { directives.delete('none'); directives.add('noindex'); directives.add('nofollow'); }
  if (!indexable) { directives.delete('index'); directives.add('noindex'); }
  if (!directives.has('noindex')) directives.add('index');
  if (directives.has('nofollow')) directives.delete('follow'); else directives.add('follow');
  additions += `<meta name="robots" content="${escape([...directives].join(', '))}">`;
  for (const author of entry.authors ?? []) additions += `<meta name="author" content="${escape(author.name)}">`;
  const social = { 'og:title': title, 'og:description': entry.description, 'og:type': entry.type === 'Article' || entry.type === 'TechArticle' ? 'article' : 'website', 'og:site_name': 'Ash', 'twitter:card': 'summary', 'twitter:title': title, 'twitter:description': entry.description };
  if (canonical) social['og:url'] = canonical;
  for (const [name, value] of Object.entries(social)) additions += `<meta ${name.startsWith('og:') ? 'property' : 'name'}="${name}" content="${escape(value)}">`;
  if (canonical) additions += `<link rel="canonical" href="${escape(canonical)}">`;
  if (indexable) {
    const markdown = entry.markdownUrl ?? contentMarkdownUrl(entry);
    if (markdown) additions += `<link rel="alternate" type="text/markdown" href="${escape(markdown)}" title="Markdown">`;
    additions += '<link rel="describedby" type="application/json" href="/content-index.json" data-ash-discovery><link rel="alternate" type="text/plain" href="/llms.txt" title="Agent guide" data-ash-discovery>';
    if (origin) additions += '<link rel="sitemap" type="application/xml" href="/sitemap.xml">';
    const data = {
      '@context': 'https://schema.org', '@type': entry.type ?? 'WebPage',
      name: entry.title, headline: entry.title, description: entry.description, inLanguage: entry.language ?? 'en',
      ...(canonical ? { url: canonical, '@id': canonical + '#document', isPartOf: { '@type': 'WebSite', '@id': origin + '/#website', name: 'Ash', url: origin + '/' } } : {}),
      ...(entry.published ? { datePublished: entry.published } : {}), ...(entry.updated ? { dateModified: entry.updated } : {}),
      ...(entry.authors?.length ? { author: entry.authors } : {}),
      ...(entry.tags?.length ? { keywords: entry.tags } : {}), ...(entry.status ? { creativeWorkStatus: entry.status } : {}),
      ...(entry.source?.url ? { isBasedOn: entry.source.url, version: entry.source.revision } : {}),
    };
    additions += `<script type="application/ld+json" data-ash-discovery>${JSON.stringify(data).replaceAll('<', '\\u003c')}</script>`;
  }
  const enriched = html.replace(/<head\b[^>]*>[\s\S]*?<\/head>/i, head.outerHTML.replace('</head>', additions + '</head>'));
  return indexable ? enriched : enriched.replace(/<[a-z][^>]*>/gi, tag => tag.replace(/\sdata-pagefind-body(?:=(?:"[^"]*"|'[^']*'|[^\s>]+))?/g, ''));
}

export function createDiscovery(entries, { origin, sourceConfigHash } = {}) {
  origin = normalizeOrigin(origin);
  const pages = entries.filter(isReadingPage).sort((a, b) => a.route.localeCompare(b.route, 'en'));
  if (new Set(pages.map(page => page.route)).size !== pages.length) throw new Error('Duplicate catalogue route');
  const exports = new Map();
  const records = pages.map(page => {
    const mdUrl = contentMarkdownUrl(page);
    let markdownHash = null;
    if (mdUrl && !page.source?.rawUrl) {
      const markdown = exportMarkdown(page, { origin, sourceRoutes: page.sourceRoutes });
      exports.set(mdUrl, markdown);
      markdownHash = sha256(markdown);
    } else if (mdUrl) {
      markdownHash = page.source.sha256;
      if (!/^[a-f0-9]{64}$/.test(markdownHash ?? '')) throw new Error(`Missing original Markdown checksum: ${page.route}`);
    }
    const { markdown: _, route, sourceRoutes, draft, noindex, ...record } = page;
    return { ...record, url: route, ...(origin ? { canonical: origin + route } : {}), markdown: mdUrl, markdownSha256: markdownHash, related: (page.related ?? []).map(link => ({ ...link, url: resolveContentLink(link.url, page, page.sourceRoutes) })) };
  });
  const catalogue = { version: 1, baseUrl: origin, ...(sourceConfigHash ? { sourceConfigSha256: sourceConfigHash } : {}), buildManifest: '/build-manifest.json', pages: records };
  let llms = '# Ash\n\n> Ash programming language: current specification, tools, study material, and development records.\n\nThe language specification defines current rules. Design notes are informative research; an accepted architectural decision does not establish a language feature. Imported docs and the website compiler checkpoint can use different revisions. Study pages state their compiler target.\n\n';
  const groups = ['website-guidance', 'normative-specification', 'tool-documentation', 'study-material', 'informative-design', 'architectural-decision'];
  for (const authority of groups) {
    const selected = records.filter(page => page.authority === authority && (authority === 'normative-specification' || authority === 'tool-documentation' || ['/','/discover/','/docs/','/status/','/learn/tour/','/study/'].includes(page.url) || /\/(?:design|decisions)\/$/.test(page.url) || page.source?.generated));
    if (!selected.length) continue;
    llms += `## ${selected[0].section ?? authority}\n\n`;
    for (const page of selected) llms += `- [${page.title.replaceAll('[', '').replaceAll(']', '')}](${publicLink(page.markdown ?? page.url, origin)}): ${short(page.description)}${page.markdown ? '' : ' Read as HTML; original Markdown is not public.'}\n`;
    llms += '\n';
  }
  llms += `## Complete catalogue and provenance\n\n- [Content catalogue](${origin ?? ''}/content-index.json): All public reading pages, Markdown URLs, document status, relationships, and exact source revisions.\n- [Build manifest](${origin ?? ''}/build-manifest.json): Resolved documentation revisions and pinned build inputs.\n`;
  const sitemap = origin ? `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${records.map(page => `  <url><loc>${escape(origin + page.url)}</loc>${page.updated ? `<lastmod>${escape(page.updated)}</lastmod>` : ''}</url>`).join('\n')}\n</urlset>\n` : null;
  const robots = `User-agent: *\nAllow: /\n${origin ? `\nSitemap: ${origin}/sitemap.xml\n` : ''}`;
  return { catalogue, exports, llms, sitemap, robots };
}
