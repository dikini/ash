import { parseDocument } from 'yaml';
import { unified } from 'unified';
import remarkParse from 'remark-parse';
import remarkGfm from 'remark-gfm';

export const markdownParser = unified().use(remarkParse).use(remarkGfm);
export const nodeText = node => node.value ?? (node.children ?? []).map(nodeText).join('');
const compact = value => value.replace(/\s+/g, ' ').trim();

function string(value, name) {
  if (value === undefined || value === null) return undefined;
  if (typeof value !== 'string' || !value.trim()) throw new Error(`Invalid ${name}: expected nonempty text`);
  return value.trim();
}
function date(value, name) {
  const text = string(value, name);
  if (!text) return undefined;
  if (!/^\d{4}-\d{2}-\d{2}(?:T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2}))?$/.test(text) || !Number.isFinite(Date.parse(text)) || new Date(text.slice(0, 10)).toISOString().slice(0, 10) !== text.slice(0, 10)) throw new Error(`Invalid ${name} date: ${text}`);
  return text;
}
function boolean(value, name) {
  if (value !== undefined && typeof value !== 'boolean') throw new Error(`Invalid ${name}: expected boolean`);
  return value ?? false;
}
function authors(value) {
  if (value == null) return [];
  return (Array.isArray(value) ? value : [value]).map(author => {
    if (typeof author === 'string') return { name: string(author, 'author') };
    if (!author || typeof author !== 'object') throw new Error('Invalid author');
    const result = { name: string(author.name, 'author name') };
    if (!result.name) throw new Error('Missing author name');
    if (author.url) {
      const url = new URL(string(author.url, 'author URL'));
      if (!['https:', 'http:'].includes(url.protocol)) throw new Error('Invalid author URL');
      result.url = url.href;
    }
    return result;
  });
}

export function readMarkdown(input) {
  let markdown = input.replace(/^\uFEFF/, '');
  let raw = {};
  if (/^---\r?\n/.test(markdown)) {
    const match = markdown.match(/^---\r?\n([\s\S]*?)\r?\n(?:---|\.\.\.)(?:\r?\n|$)/);
    if (!match) throw new Error('Unclosed Markdown frontmatter');
    const yaml = parseDocument(match[1], { uniqueKeys: true });
    if (yaml.errors.length) throw new Error(`Invalid Markdown frontmatter: ${yaml.errors[0].message}`);
    raw = yaml.toJS({ maxAliasCount: 30 }) ?? {};
    if (typeof raw !== 'object' || Array.isArray(raw)) throw new Error('Markdown frontmatter must be a mapping');
    markdown = markdown.slice(match[0].length);
  }
  const tree = markdownParser.parse(markdown);
  const title = tree.children.find(node => node.type === 'heading' && node.depth === 1);
  const fields = {};
  const related = [];
  for (const node of tree.children) {
    if (node.type === 'heading' && node.depth > 1) break;
    if (node.type !== 'list') continue;
    for (const item of node.children) {
      const paragraph = item.children[0];
      const match = paragraph && nodeText(paragraph).match(/^(ID|Status|Related|Updated|Published|Author):\s*(.+)$/i);
      if (!match) continue;
      const key = match[1].toLowerCase();
      fields[key] = match[2];
      if (key === 'related') for (const child of paragraph.children) if (child.type === 'link') related.push({ title: nodeText(child), url: child.url });
    }
  }
  const lead = tree.children.find(node => node.type === 'paragraph');
  const fallback = compact(lead ? nodeText(lead) : title ? nodeText(title) : 'Ash documentation.');
  const description = string(raw.description ?? raw.summary, 'description') ?? (fallback.length > 220 ? fallback.slice(0, 217).replace(/\s+\S*$/, '') + '…' : fallback);
  const tags = raw.tags ?? raw.keywords ?? [];
  const robots = string(raw.robots, 'robots');
  if (!Array.isArray(tags) && typeof tags !== 'string') throw new Error('Invalid tags');
  const metadata = {
    title: string(raw.title, 'title') ?? (title ? compact(nodeText(title)) : undefined), description,
    authors: authors(raw.authors ?? raw.author ?? fields.author),
    published: date(raw.published ?? raw.date ?? fields.published, 'published'),
    updated: date(raw.updated ?? raw.modified ?? raw.lastmod ?? fields.updated, 'updated'),
    tags: (Array.isArray(tags) ? tags : tags.split(',')).map(tag => string(tag, 'tag')),
    id: string(raw.id ?? fields.id, 'id'), status: string(raw.status ?? fields.status, 'status'),
    related: raw.related ?? related,
    draft: boolean(raw.draft, 'draft'), noindex: boolean(raw.noindex, 'noindex') || /\b(?:noindex|none)\b/i.test(robots ?? ''), robots,
    language: string(raw.lang ?? raw.language, 'language') ?? 'en',
    sourceMetadata: raw,
  };
  if (metadata.published && metadata.updated && Date.parse(metadata.updated) < Date.parse(metadata.published)) throw new Error('Update date precedes publication');
  if (!Array.isArray(metadata.related) || metadata.related.some(item => !item || typeof item.title !== 'string' || typeof item.url !== 'string')) throw new Error('Invalid related links: expected title/url records');
  return { markdown, metadata };
}
