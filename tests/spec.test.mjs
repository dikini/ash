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

test('specification and tool references link locally across source directories', async () => {
  const { renderSpecMarkdown } = await import('../scripts/spec-markdown.mjs');
  const routes = { 'spec/README.md': '/spec/', 'docs/tools/README.md': '/docs/tools/', 'docs/tools/ash.md': '/docs/tools/ash/' };
  const spec = await renderSpecMarkdown('# Spec\n\n[Tool](../docs/tools/ash.md#failures)', { file: 'README.md', chapters: ['README.md'], directory: 'spec', revision, routes });
  assert.equal(parseHTML(spec.html).document.querySelector('a').getAttribute('href'), '/docs/tools/ash/#failures');
  const tools = await renderSpecMarkdown('# Tool\n\n[Overview](README.md) [Spec](../../spec/README.md#current-ash) [Policy](../project/documentation.md)', {
    file: 'ash.md', chapters: ['README.md', 'ash.md'], directory: 'docs/tools', revision, routes,
  });
  assert.deepEqual([...parseHTML(tools.html).document.querySelectorAll('a')].map(a => a.getAttribute('href')), [
    '/docs/tools/', '/spec/#current-ash', `https://github.com/dikini/ash-lang/blob/${revision}/docs/project/documentation.md`,
  ]);
});

test('ordinary source fences retain their text and keyboard scrolling', async () => {
  const source = 'main : Unit -> Int\nmain () = 42\n';
  const { html } = await render(`# Example\n\n\`\`\`ash\n${source}\`\`\``);
  const { document } = parseHTML(html);
  assert.equal(document.querySelector('pre code').textContent, source);
  assert.equal(document.querySelector('pre').getAttribute('tabindex'), '0');
});

test('design routes preserve nested indexes and cross-reference the other publications', async () => {
  const { designRoute } = await import('../scripts/prepare-spec.mjs');
  assert.equal(designRoute('README.md'), '/docs/design/');
  assert.equal(designRoute('DN-0001-foundations.md'), '/docs/design/dn-0001-foundations/');
  assert.equal(designRoute('mini-ash/first-stratum/README.md'), '/docs/design/mini-ash/first-stratum/');
  assert.equal(designRoute('mini-ash/first-stratum/grammar.md'), '/docs/design/mini-ash/first-stratum/grammar/');
  const { renderSpecMarkdown } = await import('../scripts/spec-markdown.mjs');
  const routes = {
    'docs/design/README.md': '/docs/design/',
    'docs/design/mini-ash/first-stratum/README.md': '/docs/design/mini-ash/first-stratum/',
    'docs/design/DN-0001-foundations.md': '/docs/design/dn-0001-foundations/',
    'spec/README.md': '/spec/', 'docs/tools/ash.md': '/docs/tools/ash/',
  };
  const { html } = await renderSpecMarkdown('# Evidence\n\n[Design](../../README.md) [Note](../../DN-0001-foundations.md#question) [Index](./) [Spec](../../../../spec/README.md) [Tool](../../../tools/ash.md) [Lean](Kernel.lean)', {
    file: 'mini-ash/first-stratum/evidence.md', directory: 'docs/design', chapters: [], revision, routes,
  });
  assert.deepEqual([...parseHTML(html).document.querySelectorAll('a')].map(a => a.getAttribute('href')), [
    '/docs/design/', '/docs/design/dn-0001-foundations/#question', '/docs/design/mini-ash/first-stratum/', '/spec/', '/docs/tools/ash/',
    `https://github.com/dikini/ash-lang/blob/${revision}/docs/design/mini-ash/first-stratum/Kernel.lean`,
  ]);
});

test('informative design notes retain unrenderable sequent source while specs stay strict', async () => {
  const { renderSpecMarkdown } = await import('../scripts/spec-markdown.mjs');
  const source = 'not a rule\n';
  const markdown = `# Research\n\n\`\`\`sequent\n${source}\`\`\`\n\n\`\`\`sequent\nIdentity :=\n  [ GAMMA |- A ]\n  ===>\n  GAMMA |- A\n\`\`\``;
  const result = await renderSpecMarkdown(markdown, { file: 'research.md', directory: 'docs/design', chapters: [], revision, allowSequentSource: true });
  const { document } = parseHTML(result.html);
  assert.equal(document.querySelector('.spec-notation-source pre code').textContent, source);
  assert.match(document.querySelector('.spec-notation-source p').textContent, /Inference rule source.*diagram unavailable/);
  assert.ok(document.querySelector('.sequent-proof'));
  assert.equal(result.warnings.length, 1);
  assert.match(result.warnings[0], /research.md.*sequent/i);
  await assert.rejects(renderSpecMarkdown(markdown, { file: 'research.md', directory: 'spec', chapters: [], revision }), /sequent/i);
});

test('architectural decision routes and references connect all reference groups', async () => {
  const { decisionRoute } = await import('../scripts/prepare-spec.mjs');
  assert.equal(decisionRoute('README.md'), '/docs/decisions/');
  assert.equal(decisionRoute('AD-0003-notation.md'), '/docs/decisions/ad-0003-notation/');
  assert.equal(decisionRoute('evidence/README.md'), '/docs/decisions/evidence/');
  assert.equal(decisionRoute('evidence/Parser.md'), '/docs/decisions/evidence/parser/');
  const { renderSpecMarkdown } = await import('../scripts/spec-markdown.mjs');
  const routes = {
    'docs/decisions/README.md': '/docs/decisions/',
    'docs/decisions/AD-0003-notation.md': '/docs/decisions/ad-0003-notation/',
    'docs/design/DN-0001-foundations.md': '/docs/design/dn-0001-foundations/',
    'docs/tools/ash.md': '/docs/tools/ash/', 'spec/README.md': '/spec/',
  };
  const decision = await renderSpecMarkdown('# Decision\n\n[Overview](./) [Design](../design/DN-0001-foundations.md#question) [Tool](../tools/ash.md) [Spec](../../spec/README.md) [Policy](../project/documentation.md)', {
    file: 'AD-0003-notation.md', directory: 'docs/decisions', chapters: [], revision, routes,
  });
  assert.deepEqual([...parseHTML(decision.html).document.querySelectorAll('a')].map(a => a.getAttribute('href')), [
    '/docs/decisions/', '/docs/design/dn-0001-foundations/#question', '/docs/tools/ash/', '/spec/',
    `https://github.com/dikini/ash-lang/blob/${revision}/docs/project/documentation.md`,
  ]);
  const design = await renderSpecMarkdown('# Design\n\n[Decision](../decisions/AD-0003-notation.md#decision)', {
    file: 'DN-0001-foundations.md', directory: 'docs/design', chapters: [], revision, routes,
  });
  assert.equal(parseHTML(design.html).document.querySelector('a').getAttribute('href'), '/docs/decisions/ad-0003-notation/#decision');
  await assert.rejects(renderSpecMarkdown('# Decision\n\n```sequent\nnot a rule\n```', {
    file: 'AD-0003-notation.md', directory: 'docs/decisions', chapters: [], revision,
  }), /AD-0003-notation.md.*sequent/s);
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
  const { readSpecSnapshot, readToolSnapshot, readDesignSnapshot, readDecisionSnapshot } = await import('../scripts/prepare-spec.mjs');
  const source = await mkdtemp(join(tmpdir(), 'ash-spec-'));
  const git = args => execFileSync('git', args, { cwd: source, encoding: 'utf8' }).trim();
  try {
    git(['init', '-q']);
    await mkdir(join(source, 'spec'));
    await mkdir(join(source, 'docs/tools'), { recursive: true });
    await writeFile(join(source, 'docs/tools/README.md'), '# Ash Tools\n');
    await writeFile(join(source, 'docs/tools/ash.md'), '# The `ash` Executable\n\n```text\nash main.ash\n```\n');
    await mkdir(join(source, 'docs/design/mini-ash'), { recursive: true });
    await writeFile(join(source, 'docs/design/README.md'), '# Design notes\n');
    await writeFile(join(source, 'docs/design/DN-0001-foundations.md'), '# Foundations\n');
    await writeFile(join(source, 'docs/design/mini-ash/README.md'), '# Mini Ash\n');
    await writeFile(join(source, 'docs/design/mini-ash/evidence.md'), '# Evidence\n');
    await writeFile(join(source, 'docs/design/mini-ash/Kernel.lean'), '-- Source only\n');
    await mkdir(join(source, 'docs/decisions'), { recursive: true });
    await writeFile(join(source, 'docs/decisions/README.md'), '# Architectural decisions\n');
    await writeFile(join(source, 'docs/decisions/AD-0001-environment.md'), '# Environment\n\n- Status: accepted\n');
    await writeFile(join(source, 'docs/decisions/AD-0002-previous-choice.md'), '# Previous choice\n\n- Status: superseded\n');
    await writeFile(join(source, 'docs/decisions/template.md'), '# Decision template\n');
    for (const file of ['README.md', 'lexical-structure.md', 'syntax.md', 'name-resolution.md', 'static-semantics.md', 'elaboration.md', 'dynamic-semantics.md']) {
      await writeFile(join(source, 'spec', file), `# ${file}\n\nCommitted.\n`);
    }
    git(['add', 'spec', 'docs']);
    git(['-c', 'user.name=Test', '-c', 'user.email=test@example.com', '-c', 'commit.gpgsign=false', '-c', 'core.hooksPath=/dev/null', 'commit', '-qm', 'spec fixture']);
    const revision = git(['rev-parse', 'HEAD']);
    await writeFile(join(source, 'spec/syntax.md'), '# Uncommitted\n');
    await writeFile(join(source, 'docs/tools/ash.md'), '# Uncommitted tool\n');
    await writeFile(join(source, 'docs/design/mini-ash/evidence.md'), '# Uncommitted evidence\n');
    await writeFile(join(source, 'docs/decisions/AD-0001-environment.md'), '# Uncommitted decision\n');
    const before = git(['status', '--porcelain']);
    const snapshot = await readSpecSnapshot(source);
    assert.equal(snapshot.revision, revision);
    assert.equal(snapshot.directory, 'spec');
    assert.equal(snapshot.chapters.length, 7);
    assert.equal(snapshot.chapters.find(chapter => chapter.file === 'syntax.md').markdown, '# syntax.md\n\nCommitted.\n');
    const tools = await readToolSnapshot(source, snapshot.revision);
    assert.equal(tools.revision, snapshot.revision);
    assert.equal(tools.directory, 'docs/tools');
    assert.deepEqual(tools.chapters.map(chapter => chapter.file), ['README.md', 'ash.md']);
    assert.equal(tools.chapters[1].markdown, '# The `ash` Executable\n\n```text\nash main.ash\n```\n');
    const design = await readDesignSnapshot(source, snapshot.revision);
    assert.equal(design.revision, snapshot.revision);
    assert.equal(design.directory, 'docs/design');
    assert.deepEqual(design.chapters.map(chapter => chapter.file), ['README.md', 'DN-0001-foundations.md', 'mini-ash/README.md', 'mini-ash/evidence.md']);
    assert.equal(design.chapters.at(-1).markdown, '# Evidence\n');
    const decisions = await readDecisionSnapshot(source, snapshot.revision);
    assert.equal(decisions.revision, snapshot.revision);
    assert.equal(decisions.directory, 'docs/decisions');
    assert.deepEqual(decisions.chapters.map(chapter => chapter.file), ['README.md', 'AD-0001-environment.md', 'AD-0002-previous-choice.md', 'template.md']);
    assert.equal(decisions.chapters[1].markdown, '# Environment\n\n- Status: accepted\n');
    assert.equal(decisions.chapters[2].markdown, '# Previous choice\n\n- Status: superseded\n');
    assert.equal(git(['status', '--porcelain']), before);
    assert.equal((await readSpecSnapshot(source, revision)).revision, revision);
    await assert.rejects(readSpecSnapshot(source, '--help'), /revision/);
    await assert.rejects(readSpecSnapshot(source, 'b'.repeat(40)), /git|revision/i);
    git(['rm', '--cached', 'docs/tools/ash.md']);
    git(['-c', 'user.name=Test', '-c', 'user.email=test@example.com', '-c', 'commit.gpgsign=false', '-c', 'core.hooksPath=/dev/null', 'commit', '-qm', 'remove required tool guide']);
    await assert.rejects(readToolSnapshot(source), /Missing.*ash\.md/);
    await writeFile(join(source, 'docs/design/mini-ash.md'), '# Route collision\n');
    git(['add', 'docs/design/mini-ash.md']);
    git(['-c', 'user.name=Test', '-c', 'user.email=test@example.com', '-c', 'commit.gpgsign=false', '-c', 'core.hooksPath=/dev/null', 'commit', '-qm', 'duplicate design route']);
    await assert.rejects(readDesignSnapshot(source), /Duplicate.*route/i);
    await writeFile(join(source, 'docs/decisions/ad-0001-environment.md'), '# Duplicate route\n');
    git(['add', 'docs/decisions/ad-0001-environment.md']);
    git(['-c', 'user.name=Test', '-c', 'user.email=test@example.com', '-c', 'commit.gpgsign=false', '-c', 'core.hooksPath=/dev/null', 'commit', '-qm', 'duplicate decision route']);
    await assert.rejects(readDecisionSnapshot(source), /Duplicate.*route/i);
    assert.equal((await readDecisionSnapshot(source, revision)).revision, revision);
    git(['rm', '--cached', 'docs/decisions/README.md']);
    git(['-c', 'user.name=Test', '-c', 'user.email=test@example.com', '-c', 'commit.gpgsign=false', '-c', 'core.hooksPath=/dev/null', 'commit', '-qm', 'remove decision overview']);
    await assert.rejects(readDecisionSnapshot(source), /Missing.*decisions\/README.md/);
  } finally { await rm(source, { recursive: true, force: true }); }
});
