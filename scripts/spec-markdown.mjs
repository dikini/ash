import { unified } from 'unified';
import remarkParse from 'remark-parse';
import remarkGfm from 'remark-gfm';
import remarkRehype from 'remark-rehype';
import rehypeSlug from 'rehype-slug';
import rehypeStringify from 'rehype-stringify';
import rehypeRailroad from 'rrd-preview/rehype';
import { rehypeSequent } from '@sequent-md/rehype';
import { createSVGWindow } from 'svgdom';

export const specRoute = file => file === 'README.md' ? '/spec/' : `/spec/${file.replace(/\.md$/, '')}/`;
const textContent = node => node.type === 'text' ? node.value : (node.children ?? []).map(textContent).join('');
const hasClass = (node, name) => node.properties?.className?.includes(name);

// Librrd enumerates every possible line partition of a sequence (2^(n-1)).
// Associative groups bound that search without changing terminals, references,
// branches, or repetition. This receives only RRD emitted by the EBNF compiler.
function groupRailroadSequences(rrd) {
  const stack = [[]];
  for (const token of rrd.match(/"[^"]*"|'[^']*'|\[[^\]]*\]|[()]|[^\s()]+/g) ?? []) {
    if (token === '(') { const group = []; stack.at(-1).push(group); stack.push(group); }
    else if (token === ')') stack.pop();
    else stack.at(-1).push(token);
  }
  function format(node) {
    if (typeof node === 'string') return node;
    let parts = node.map(format);
    if (!['+', '-', '-?', '-!'].includes(parts[0])) {
      while (parts.length > 8) {
        const groups = [];
        for (let index = 0; index < parts.length; index += 8) groups.push(`(${parts.slice(index, index + 8).join(' ')})`);
        parts = groups;
      }
    }
    return `(${parts.join(' ')})`;
  }
  return stack[0].map(format).join(' ');
}

async function renderBoundedRailroad(rrd, settings) {
  const tree = { type: 'root', children: [{ type: 'element', tagName: 'pre', properties: {}, children: [
    { type: 'element', tagName: 'code', properties: { className: ['language-rrd'] }, children: [{ type: 'text', value: groupRailroadSequences(rrd) }] },
  ] }] };
  await rehypeRailroad({ width: settings.width, stylesheet: settings.stylesheet })(tree, { message(reason) { throw new Error(reason); } });
  const svg = tree.children[0].children.find(node => node.tagName === 'svg');
  // The upstream Node plugin supplies only width. Compute a complete viewBox
  // at build time; browser defaults otherwise clip diagrams taller than 150px.
  const root = createSVGWindow().document.documentElement;
  root.innerHTML = unified().use(rehypeStringify).stringify({ type: 'root', children: [svg] });
  const bounds = root.firstElementChild.getBBox();
  const x = Math.floor(bounds.x) - 4, y = Math.floor(bounds.y) - 4;
  const width = Math.ceil(bounds.x + bounds.width) + 4 - x;
  const height = Math.ceil(bounds.y + bounds.height) + 4 - y;
  Object.assign(svg.properties, { viewBox: `${x} ${y} ${width} ${height}`, width, height });
  return svg;
}

// Keep a text equivalent beside SVG, whose accessible name identifies only
// the production. The copied fence uses plain text to avoid rendering twice.
function preserveGrammarSource() {
  return function visit(tree) {
    tree.children = (tree.children ?? []).map(node => {
      if (node.tagName === 'pre' && hasClass(node.children?.[0], 'language-ebnf')) {
        const code = node.children[0];
        return { type: 'element', tagName: 'div', properties: { className: ['spec-grammar'] }, children: [node, {
          type: 'element', tagName: 'details', properties: {}, children: [
            { type: 'element', tagName: 'summary', properties: {}, children: [{ type: 'text', value: 'Grammar source (EBNF)' }] },
            { type: 'element', tagName: 'pre', properties: {}, children: [{ ...code, properties: {} }] },
          ],
        }] };
      }
      if (node.children) visit(node);
      return node;
    });
  };
}

export async function renderSpecMarkdown(markdown, { file, chapters, revision, directory }) {
  let title, titleId;
  const headings = [];
  const transform = () => (tree, vfile) => {
    function visit(parent) {
      parent.children = (parent.children ?? []).filter(node => {
        if (node.type !== 'element') return true;
        if (hasClass(node, 'rrd-error') || hasClass(node, 'sequent-diagnostic')) {
          vfile.fail(`${file}: ${hasClass(node, 'sequent-diagnostic') ? 'sequent: ' : ''}${textContent(node)}`);
        }
        if (node.tagName === 'h1') {
          if (title) vfile.fail(`${file}: multiple top-level titles`);
          title = textContent(node);
          titleId = node.properties.id;
          return false;
        }
        if (/^h[2-6]$/.test(node.tagName)) headings.push({ depth: Number(node.tagName[1]), slug: node.properties.id, text: textContent(node) });
        if (node.tagName === 'a' || node.tagName === 'img') {
          const attribute = node.tagName === 'a' ? 'href' : 'src';
          const href = node.properties[attribute];
          if (href && !/^(?:[a-z][a-z\d+.-]*:|\/\/|#)/i.test(href)) {
            const target = new URL(href, `https://source.invalid/${directory}/${file}`);
            const chapter = decodeURIComponent(target.pathname.slice(directory.length + 2));
            if (target.pathname.startsWith(`/${directory}/`) && chapters.includes(chapter)) {
              node.properties[attribute] = specRoute(chapter) + target.search + target.hash;
            } else {
              node.properties[attribute] = node.tagName === 'img'
                ? `https://raw.githubusercontent.com/dikini/ash-lang/${revision}${target.pathname}${target.search}${target.hash}`
                : `https://github.com/dikini/ash-lang/blob/${revision}${target.pathname}${target.search}${target.hash}`;
            }
          }
        }
        if (node.tagName === 'pre' || hasClass(node, 'rrd-diagram') || hasClass(node, 'sequent-proof-container')) {
          node.properties.tabIndex = 0;
          if (node.tagName !== 'pre') {
            node.properties.role = 'region';
            node.properties.ariaLabel = hasClass(node, 'rrd-diagram') ? `Railroad diagram for ${textContent(node.children[0])}` : 'Inference rule';
          }
        }
        if (hasClass(node, 'rrd-diagram')) {
          const caption = node.children.find(child => child.tagName === 'figcaption');
          const svg = node.children.find(child => child.tagName === 'svg');
          if (caption && svg) svg.properties['aria-label'] = `Railroad diagram for ${textContent(caption)}`;
        }
        visit(node);
        return true;
      });
    }
    visit(tree);
    if (!title) vfile.fail(`${file}: missing specification title`);
  };
  const result = await unified().use(remarkParse).use(remarkGfm).use(remarkRehype)
    .use(rehypeSlug).use(preserveGrammarSource).use(rehypeRailroad, { width: 760, renderer: renderBoundedRailroad }).use(rehypeSequent)
    .use(transform).use(rehypeStringify).process({ value: markdown, path: file });
  if (result.messages.length) throw new Error(`${file}: ${result.messages.map(message => message.reason).join('\n')}`);
  return { html: String(result), title, titleId, headings };
}
