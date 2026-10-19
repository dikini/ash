import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { parseHTML } from 'linkedom';

test('all canonical specification chapters are published with source provenance', () => {
  const publication = JSON.parse(readFileSync('.cache/ash-spec/publication.json', 'utf8'));
  assert.equal(publication.pages.length, 7);
  assert.match(publication.revision, /^[a-f0-9]{40}$/);
  for (const page of publication.pages) {
    const { document } = parseHTML(readFileSync(`dist${page.route}index.html`, 'utf8'));
    assert.equal(document.querySelectorAll('h1').length, 1);
    assert.equal(document.querySelector('h1').textContent, page.title);
    assert.ok(document.querySelector(`a[href="https://github.com/dikini/ash-lang/blob/${publication.revision}/${publication.directory}/${page.file}"]`));
    assert.equal(document.querySelectorAll('.spec-chapters a').length, 7);
    assert.equal(document.querySelectorAll('.rrd-error, .sequent-diagnostic, code.language-ebnf, code.language-sequent').length, 0);
  }
  const syntax = readFileSync('dist/spec/syntax/index.html', 'utf8');
  const semantics = readFileSync('dist/spec/static-semantics/index.html', 'utf8');
  assert.match(syntax, /class="rrd-diagram"/);
  assert.match(semantics, /class="sequent-proof"/);
  assert.match(readFileSync('dist/docs/index.html', 'utf8'), /href="\/spec\/"/);
});

test('tool documentation shares the spec revision and preserves upstream examples', () => {
  const publication = JSON.parse(readFileSync('.cache/ash-spec/publication.json', 'utf8'));
  assert.equal(publication.tools.pages.length, 2);
  for (const page of publication.tools.pages) {
    const { document } = parseHTML(readFileSync(`dist${page.route}index.html`, 'utf8'));
    assert.equal(document.querySelectorAll('h1').length, 1);
    assert.equal(document.querySelector('h1').textContent, page.title);
    assert.ok(document.querySelector(`a[href="https://github.com/dikini/ash-lang/blob/${publication.revision}/docs/tools/${page.file}"]`));
    assert.equal(document.querySelectorAll('nav[aria-label="Tool documentation"] a').length, 2);
    assert.match(document.querySelector('.spec-source').textContent, /tool documentation/);
  }
  const { document: overview } = parseHTML(readFileSync('dist/docs/tools/index.html', 'utf8'));
  assert.ok(overview.querySelector('article a[href="/spec/"]'));
  assert.ok(overview.querySelector('article a[href="/docs/tools/ash/"]'));
  const { document: guide } = parseHTML(readFileSync('dist/docs/tools/ash/index.html', 'utf8'));
  assert.match(guide.querySelector('pre code').textContent, /^ash \[--message-format human\|short\|json\]/);
  assert.match(readFileSync('dist/spec/index.html', 'utf8'), /href="\/docs\/tools\/ash\/"/);
  assert.match(readFileSync('dist/docs/index.html', 'utf8'), /href="\/docs\/tools\/ash\/"/);
  assert.match(readFileSync('dist/docs/install/index.html', 'utf8'), /pinned tool guide/);
});

test('recursive design documents are published as informative work at the shared revision', () => {
  const publication = JSON.parse(readFileSync('.cache/ash-spec/publication.json', 'utf8'));
  assert.ok(publication.design.pages.length > 50);
  for (const page of publication.design.pages) {
    const { document } = parseHTML(readFileSync(`dist${page.route}index.html`, 'utf8'));
    assert.equal(document.querySelectorAll('h1').length, 1);
    assert.equal(document.querySelector('h1').textContent, page.title);
    assert.ok(document.querySelector(`a[href="https://github.com/dikini/ash-lang/blob/${publication.revision}/docs/design/${page.file}"]`));
    assert.match(document.querySelector('.design-status').textContent, /Informative design work/);
    assert.ok(document.querySelector('.design-status a[href="/spec/"]'));
    assert.equal(document.querySelectorAll('nav[aria-label="Design notes"] a').length, 1);
    assert.equal(document.querySelectorAll('.rrd-error, .sequent-diagnostic, code.language-ebnf, code.language-sequent').length, 0);
  }
  const { document: overview } = parseHTML(readFileSync('dist/docs/design/index.html', 'utf8'));
  assert.ok(overview.querySelector('article a[href="/docs/design/dn-0001-reboot-foundations/"]'));
  const supporting = publication.design.pages.filter(page => page.file !== 'README.md' && !/^DN-\d+[^/]*\.md$/.test(page.file));
  assert.equal(overview.querySelectorAll('.design-support a').length, supporting.length);
  const { document: grammar } = parseHTML(readFileSync('dist/docs/design/mini-ash/first-stratum/grammar/index.html', 'utf8'));
  assert.ok(grammar.querySelector('.rrd-diagram svg'));
  assert.ok(grammar.querySelector('article a[href="/docs/design/dn-0016-first-mini-ash-corpus/"]'));
  assert.match(readFileSync('dist/docs/design/provider-experiments/semantics/index.html', 'utf8'), /Inference rule source \(diagram unavailable\)/);
  assert.match(readFileSync('dist/docs/index.html', 'utf8'), /href="\/docs\/design\/"/);
});
