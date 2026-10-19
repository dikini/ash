import test from 'node:test';
import assert from 'node:assert/strict';
import { parseHTML } from 'linkedom';
import { mkdtemp, mkdir, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { execFileSync } from 'node:child_process';

const revision = 'a'.repeat(40);
const chapters = ['README.md', 'syntax.md', 'static-semantics.md'];
const render = async (markdown, file = 'syntax.md') => {
  const { renderSpecMarkdown } = await import('../scripts/spec-markdown.mjs');
  return renderSpecMarkdown(markdown, { file, chapters, revision, directory: 'spec' });
};

test('spec Markdown renders railroad productions and semantic inference rules', async () => {
  const { html, headings, title } = await render('# Syntax\n\n## Grammar\n\n```ebnf\nterm ::= "true" | "false" ;\n```\n\n```sequent\nIdentity :=\n  [ GAMMA |- A ]\n  ===>\n  GAMMA |- A\n```');
  const { document } = parseHTML(html);
  assert.equal(title, 'Syntax');
  assert.deepEqual(headings, [{ depth: 2, slug: 'grammar', text: 'Grammar' }]);
  assert.ok(document.querySelector('figure.rrd-diagram svg[role="img"]'));
  assert.equal(document.querySelector('.rrd-diagram figcaption').textContent, 'term');
  assert.equal(document.querySelector('svg').getAttribute('aria-label'), 'Railroad diagram for term');
  assert.ok(Number(document.querySelector('svg').getAttribute('height')) > 0);
  assert.ok(document.querySelector('svg').getAttribute('viewBox'));
  assert.equal(document.querySelector('details summary').textContent, 'Grammar source (EBNF)');
  assert.equal(document.querySelector('details code').textContent, 'term ::= "true" | "false" ;\n');
  assert.ok(document.querySelector('.sequent-proof .sequent-premises'));
  assert.match(document.querySelector('.sequent-conclusion').textContent, /Γ ⊢ A/);
  assert.equal(document.querySelectorAll('h1, code.language-ebnf, code.language-sequent').length, 0);
});

test('spec links preserve fragments and resolve other docs at the source revision', async () => {
  const { html } = await render('# Links\n\n[Home](README.md#local-newtypes) [Types](static-semantics.md#checking) [Tool](../docs/tools/ash.md) [Here](#local) [Remote](https://example.com/)\n\n## Local');
  const { document } = parseHTML(html);
  assert.deepEqual([...document.querySelectorAll('a')].map(a => a.getAttribute('href')), [
    '/spec/#local-newtypes', '/spec/static-semantics/#checking',
    `https://github.com/dikini/ash-lang/blob/${revision}/docs/tools/ash.md`, '#local', 'https://example.com/',
  ]);
});

test('ordinary source fences retain their text and keyboard scrolling', async () => {
  const source = 'main : Unit -> Int\nmain () = 42\n';
  const { html } = await render(`# Example\n\n\`\`\`ash\n${source}\`\`\``);
  const { document } = parseHTML(html);
  assert.equal(document.querySelector('pre code').textContent, source);
  assert.equal(document.querySelector('pre').getAttribute('tabindex'), '0');
});

test('long grammar sequences render without enumerating every possible line break', async () => {
  const terms = Array.from({ length: 40 }, (_, index) => `"token${index}"`);
  const { html } = await render(`# Long\n\n\`\`\`ebnf\nlong ::= ${terms.join(' ')} ;\n\`\`\``);
  const { document } = parseHTML(html);
  const svg = document.querySelector('.rrd-diagram svg');
  assert.ok(svg);
  const text = [...svg.querySelectorAll('text')].map(node => node.textContent);
  assert.deepEqual(text, terms.map(term => term.slice(1, -1)));
});

test('invalid formal notation rejects the publication with a chapter diagnostic', async () => {
  await assert.rejects(render('# Bad\n\n```ebnf\ninvalid ::= (\n```'), /syntax\.md.*[Rr]ailroad/s);
  await assert.rejects(render('# Bad\n\n```sequent\nnot a rule\n```'), /syntax\.md.*sequent/s);
});

test('local snapshots read committed chapters without changing the checkout', async () => {
  const { readSpecSnapshot } = await import('../scripts/prepare-spec.mjs');
  const source = await mkdtemp(join(tmpdir(), 'ash-spec-'));
  const git = args => execFileSync('git', args, { cwd: source, encoding: 'utf8' }).trim();
  try {
    git(['init', '-q']);
    await mkdir(join(source, 'spec'));
    for (const file of ['README.md', 'lexical-structure.md', 'syntax.md', 'name-resolution.md', 'static-semantics.md', 'elaboration.md', 'dynamic-semantics.md']) {
      await writeFile(join(source, 'spec', file), `# ${file}\n\nCommitted.\n`);
    }
    git(['add', 'spec']);
    git(['-c', 'user.name=Test', '-c', 'user.email=test@example.com', '-c', 'commit.gpgsign=false', '-c', 'core.hooksPath=/dev/null', 'commit', '-qm', 'spec fixture']);
    const revision = git(['rev-parse', 'HEAD']);
    await writeFile(join(source, 'spec/syntax.md'), '# Uncommitted\n');
    const before = git(['status', '--porcelain']);
    const snapshot = await readSpecSnapshot(source);
    assert.equal(snapshot.revision, revision);
    assert.equal(snapshot.directory, 'spec');
    assert.equal(snapshot.chapters.length, 7);
    assert.equal(snapshot.chapters.find(chapter => chapter.file === 'syntax.md').markdown, '# syntax.md\n\nCommitted.\n');
    assert.equal(git(['status', '--porcelain']), before);
    assert.equal((await readSpecSnapshot(source, revision)).revision, revision);
    await assert.rejects(readSpecSnapshot(source, '--help'), /revision/);
    await assert.rejects(readSpecSnapshot(source, 'b'.repeat(40)), /git|revision/i);
  } finally { await rm(source, { recursive: true, force: true }); }
});
