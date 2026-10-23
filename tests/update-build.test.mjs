import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fingerprint, validationScope, routeCacheKey, reusableStage } from '../scripts/update-build.mjs';

test('fingerprints include file contents, additions, and deletions', async t => {
  const root = await mkdtemp(join(tmpdir(), 'ash-update-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  await mkdir(join(root, 'content'));
  await writeFile(join(root, 'content/a.md'), 'one');
  const first = await fingerprint(root, ['content']);
  await writeFile(join(root, 'content/a.md'), 'two');
  assert.notEqual(await fingerprint(root, ['content']), first);
  await rm(join(root, 'content/a.md'));
  assert.notEqual(await fingerprint(root, ['content']), first);
});
test('only editorial changes select focused checks; unknown changes widen scope', () => {
  assert.equal(validationScope(['content/diaries/new.md']), 'editorial');
  for (const files of [['src/layouts/Site.astro'], ['sources.json'], ['tests/pages.test.mjs'], ['other']]) assert.equal(validationScope(files), 'full');
});
test('route cache key invalidates body, metadata, configuration and reference navigation', () => {
  const page = { body: 'body', data: { title: 'A' } };
  const key = routeCacheKey(page, { checkpoint: 'one' });
  assert.notEqual(routeCacheKey({ ...page, body: 'new' }, { checkpoint: 'one' }), key);
  assert.notEqual(routeCacheKey({ ...page, data: { title: 'B' } }, { checkpoint: 'one' }), key);
  assert.notEqual(routeCacheKey(page, { checkpoint: 'two' }), key);
  assert.notEqual(routeCacheKey({ group: ['A', 'B'] }, {}), routeCacheKey({ group: ['A', 'C'] }, {}));
});
test('stage reuse rejects changed inputs, missing outputs, and tampered outputs', async t => {
  const root = await mkdtemp(join(tmpdir(), 'ash-stage-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  await writeFile(join(root, 'output'), 'good');
  const receipt = { input: 'key', output: await fingerprint(root, ['output']) };
  assert.equal(await reusableStage(root, receipt, 'key', ['output']), true);
  assert.equal(await reusableStage(root, receipt, 'new', ['output']), false);
  await writeFile(join(root, 'output'), 'bad');
  assert.equal(await reusableStage(root, receipt, 'key', ['output']), false);
  await rm(join(root, 'output'));
  assert.equal(await reusableStage(root, receipt, 'key', ['output']), false);
});

test('unchanged inputs reuse validation, but a changed build context requires full checks', () => {
  assert.equal(validationScope([]), 'unchanged');
  assert.equal(validationScope(['content/a.md'], false), 'full');
});

test('search identity includes configuration and Pagefind environment options', async () => {
  const { searchCacheKey } = await import('../scripts/update-build.mjs');
  const key = searchCacheKey('html', [['pagefind.yml', 'one']], { PAGEFIND_FORCE_LANGUAGE: 'en' });
  assert.notEqual(searchCacheKey('html', [['pagefind.yml', 'two']], { PAGEFIND_FORCE_LANGUAGE: 'en' }), key);
  assert.notEqual(searchCacheKey('html', [['pagefind.yml', 'one']], { PAGEFIND_FORCE_LANGUAGE: 'fr' }), key);
  assert.equal(searchCacheKey('html', [], { UNRELATED: 'one' }), searchCacheKey('html', [], { UNRELATED: 'two' }));
});
test('missing output cannot be reused even if the receipt contains its missing fingerprint', async t => {
  const root = await mkdtemp(join(tmpdir(), 'ash-missing-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  const output = await fingerprint(root, ['missing']);
  assert.equal(await reusableStage(root, { input: 'key', output }, 'key', ['missing']), false);
});

test('all default Pagefind configuration formats affect cache identity independently of Git tracking', async t => {
  const { searchConfiguration } = await import('../scripts/update-build.mjs');
  const root = await mkdtemp(join(tmpdir(), 'ash-search-config-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  await writeFile(join(root, '.gitignore'), 'pagefind.*\n');
  const empty = await searchConfiguration(root);
  for (const name of ['pagefind.json', 'pagefind.yml', 'pagefind.yaml', 'pagefind.toml']) {
    await writeFile(join(root, name), 'force_language = "fr"');
    assert.notEqual(await searchConfiguration(root), empty, name);
    await rm(join(root, name));
    assert.equal(await searchConfiguration(root), empty, name);
  }
});
