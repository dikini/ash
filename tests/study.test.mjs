import test from 'node:test';
import assert from 'node:assert/strict';
import { parseHTML } from 'linkedom';
import { integrateStudyHtml } from '../scripts/study-html.mjs';

const snapshot = { sourceRevision: 'a'.repeat(40), ashRevision: 'b'.repeat(40), milestone: 'ffi-base' };
test('study integration preserves examples and native links inside site navigation', () => {
  const original = '<!doctype html><html lang="en"><head><title>A chapter</title></head><body><main id="quarto-document-content"><h1>A chapter</h1><pre><code>main () = 40 + 2\n</code></pre><a href="next.html#exercise">Next</a><img src="assets/tree.svg" alt="Tree"></main></body></html>';
  const { document } = parseHTML(integrateStudyHtml(original, snapshot));
  assert.equal(document.querySelector('pre').getAttribute('tabindex'), '0');
  assert.equal(document.querySelector('main').getAttribute('tabindex'), '-1');
  assert.equal(document.querySelector('.ash-skip-link').getAttribute('href'), '#quarto-document-content');
  assert.equal(document.querySelector('pre').textContent, 'main () = 40 + 2\n');
  assert.equal(document.querySelector('main a').getAttribute('href'), 'next.html#exercise');
  assert.equal(document.querySelector('img').getAttribute('src'), 'assets/tree.svg');
  assert.ok(document.querySelector('main[data-pagefind-body]'));
  assert.match(document.querySelector('.ash-study-banner').textContent, /Development snapshot/);
  assert.ok(document.querySelector('a[href="/study/"]'));
});

test('study integration rejects unsupported output instead of publishing an empty chapter', () => {
  assert.throws(() => integrateStudyHtml('<html><body>No chapter</body></html>', snapshot), /chapter body/);
});

test('study integration rejects valid HTML if required integration cannot be applied', () => {
  assert.throws(() => integrateStudyHtml("<html><head></head><body><main id='quarto-document-content'><p>Chapter</p></main></body></html>", snapshot), /unsupported.*format/i);
});
