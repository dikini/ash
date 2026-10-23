import test from 'node:test';
import assert from 'node:assert/strict';
import { parseHTML } from 'linkedom';
import { collectEntries, chooseEntry, markCurrent } from '../public/navigation.js';

function fixture() {
  const { document } = parseHTML('<html><body><nav><a href="#guide">Guide</a><a href="#setup">Setup</a><a href="#install%3Atools">Install</a><a href="#usage">Usage</a><a href="#missing">Missing</a></nav><h2 id="guide">Guide</h2><h3 id="setup">Setup</h3><h4 id="install:tools">Install</h4><h2 id="usage">Usage</h2></body></html>');
  const nav = document.querySelector('nav');
  const entries = collectEntries(nav, document);
  entries.forEach((entry, index) => { entry.heading.getBoundingClientRect = () => ({ top: [20, 70, 140, 500][index] }); });
  return { document, nav, entries };
}

test('contents resolves encoded fragments and skips missing headings', () => {
  const { entries } = fixture();
  assert.equal(entries.length, 4);
  assert.equal(entries[2].heading.id, 'install:tools');
});

test('reading position selects the latest crossed heading, including large jumps', () => {
  const { entries } = fixture();
  assert.equal(entries.length, 4);
  assert.equal(chooseEntry(entries, 96), entries[1]);
  assert.equal(chooseEntry(entries, 550), entries[3]);
  assert.equal(chooseEntry(entries, -10), entries[0]);
  assert.equal(chooseEntry(entries, 96, true), entries[3]);
  assert.equal(chooseEntry([], 96), undefined);
});

test('current item and all ancestors are distinguished without selecting siblings', () => {
  const { nav, entries } = fixture();
  markCurrent(nav, entries, entries[2]);
  assert.equal(nav.querySelectorAll('[aria-current="location"]').length, 1);
  assert.equal(entries[2].link.getAttribute('aria-current'), 'location');
  assert.deepEqual([...nav.querySelectorAll('[data-current-ancestor]')].map(a => a.textContent), ['Guide', 'Setup']);
  markCurrent(nav, entries, entries[3]);
  assert.equal(nav.querySelectorAll('[data-current-ancestor]').length, 0);
  assert.equal(entries[2].link.hasAttribute('aria-current'), false);
});

test('hosted book contents resolves section-wrapper targets to their headings', () => {
  const { document } = parseHTML('<nav><a href="#guide">Guide</a><a href="#detail">Detail</a></nav><section id="guide" class="level2"><h2>Guide</h2><section id="detail" class="level3"><h3>Detail</h3></section></section>');
  const nav = document.querySelector('nav');
  const entries = collectEntries(nav, document);
  assert.equal(entries.length, 2);
  assert.deepEqual(entries.map(entry => entry.depth), [2, 3]);
  markCurrent(nav, entries, entries[1]);
  assert.equal(entries[0].link.hasAttribute('data-current-ancestor'), true);
  assert.equal(entries[1].link.getAttribute('aria-current'), 'location');
});

test('contents accepts same-page absolute links after Quarto normalizes URLs', () => {
  const { document } = parseHTML('<nav><a href="https://ash.test/book.html#guide">Guide</a><a href="https://ash.test/other.html#guide">Other</a><a href="https://elsewhere.test/book.html#guide">External</a></nav><h2 id="guide">Guide</h2>');
  Object.defineProperty(document, 'baseURI', { value: 'https://ash.test/book.html' });
  const entries = collectEntries(document.querySelector('nav'), document);
  assert.equal(entries.length, 1);
  assert.equal(entries[0].link.textContent, 'Guide');
});

test('wrapped section reading position uses the section boundary rather than heading margins', () => {
  const { document } = parseHTML('<nav><a href="#one">One</a><a href="#two">Two</a></nav><section id="one"><h2>One</h2></section><section id="two"><h2>Two</h2></section>');
  const entries = collectEntries(document.querySelector('nav'), document);
  document.getElementById('one').getBoundingClientRect = () => ({ top: -500 });
  document.getElementById('two').getBoundingClientRect = () => ({ top: 41 });
  entries[0].heading.getBoundingClientRect = () => ({ top: -415 });
  entries[1].heading.getBoundingClientRect = () => ({ top: 126 });
  assert.equal(chooseEntry(entries, 96)?.id, 'two');
});
