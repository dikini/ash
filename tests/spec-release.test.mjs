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
