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
  assert.equal(document.querySelector('main img').getAttribute('src'), 'assets/tree.svg');
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


test('study integration exposes shared navigation and static current-page semantics', () => {
  const original = '<html><head></head><body><nav id="quarto-sidebar"><a class="sidebar-link active" href="./chapter.html">Chapter</a><a class="sidebar-link" href="./other.html">Other</a></nav><main id="quarto-document-content"><p>Chapter</p></main></body></html>';
  const { document } = parseHTML(integrateStudyHtml(original, snapshot));
  assert.ok(document.querySelector('link[href="/navigation.css"]'));
  assert.ok(document.querySelector('script[type="module"][src="/navigation.js"]'));
  assert.equal(document.querySelector('.active').getAttribute('aria-current'), 'page');
  assert.equal(document.querySelector('a[href="./other.html"]').hasAttribute('aria-current'), false);
});

test('study integration shares site chrome while preserving native chapter markup', () => {
  const chapter = '<nav id="quarto-sidebar"><details open><summary>Part</summary><a class="sidebar-link active" href="./chapter.html">Chapter</a></details></nav><main id="quarto-document-content"><pre><code>&lt;tag&gt;  \n  main () = "a &amp; b"\n</code></pre><a href="next.html#exercise">Next</a></main>';
  const original = `<!doctype html><html><head></head><body>${chapter}</body></html>`;
  const integrated = integrateStudyHtml(original, snapshot);
  const { document } = parseHTML(integrated);
  assert.ok(document.querySelector('.ash-study-header .ash-tree-mark'));
  assert.ok(document.querySelector('.ash-study-header nav[aria-label="Main navigation"]'));
  assert.ok(document.querySelector('.ash-study-header a[href="/search/"]'));
  assert.ok(document.querySelector('.ash-study-header .source-link'));
  assert.ok(document.querySelector('.ash-study-header details.mobile-nav > summary'));
  assert.equal(document.querySelector('.ash-page-menu a[href="/study/"]').getAttribute('aria-current'), 'true');
  for (const href of ['/diaries/', '/study/', '/contribute/', '/status/']) {
    assert.ok(document.querySelector(`.ash-study-footer a[href="${href}"]`));
  }
  assert.ok(document.querySelector('.ash-study-footer .footer-mark'));
  assert.equal(document.querySelector('.ash-study-footer').getAttribute('data-pagefind-ignore'), '');
  assert.ok(integrated.indexOf('</footer>') < integrated.indexOf('</body>'));
  assert.ok(integrated.includes('<code>&lt;tag&gt;  \n  main () = "a &amp; b"\n</code>'));
  assert.ok(integrated.includes('<details open><summary>Part</summary>'));
  assert.equal(document.querySelector('#quarto-sidebar .active').getAttribute('aria-current'), 'page');
  assert.equal(document.querySelector('main a').getAttribute('href'), 'next.html#exercise');
});

test('study integration rejects output without a supported closing body', () => {
  assert.throws(() => integrateStudyHtml('<html><head></head><body><main id="quarto-document-content">Chapter</main></html>', snapshot), /unsupported.*format/i);
});

test('study snapshot metadata remains escaped in hosted chrome', () => {
  const { document } = parseHTML(integrateStudyHtml('<html><head></head><body><main id="quarto-document-content">Chapter</main></body></html>', { ...snapshot, milestone: '<script>bad</script>' }));
  assert.equal(document.querySelector('.ash-study-banner script'), null);
  assert.ok(document.querySelector('.ash-study-banner').textContent.includes('<script>bad</script>'));
});
