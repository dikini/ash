import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, readFile, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';

const sha = 'a'.repeat(40), other = 'b'.repeat(40), hash = 'c'.repeat(64);
const configuration = () => ({
  version: 1,
  repositories: {
    'ash-lang': { repository: 'https://github.com/dikini/ash-lang.git', ref: 'main' },
    'ash-doc': { repository: 'https://github.com/dikini/ash-doc.git', ref: 'main' },
    railroad: { repository: 'https://github.com/dikini/railroad', ref: sha },
    'sequent-md': { repository: 'https://github.com/dikini/sequent-md', ref: other },
  },
  reference: { source: 'ash-lang', groups: [
    { id: 'spec', directories: ['spec', 'docs/spec'], required: ['README.md'], route: '/spec/' },
    { id: 'tools', directories: ['docs/tools'], required: ['README.md', 'ash.md'], route: '/docs/tools/' },
    { id: 'design', directories: ['docs/design'], required: ['README.md'], route: '/docs/design/' },
    { id: 'decisions', directories: ['docs/decisions'], required: ['README.md'], route: '/docs/decisions/' },
  ] },
  study: { source: 'ash-doc', route: '/study/latest/' },
  checkpoint: { source: 'ash-lang', revision: sha, toolchain: '1.97.1', reviewed: '2026-10-18' },
  renderers: { 'rrd-preview': 'railroad', '@sequent-md/core': 'sequent-md', '@sequent-md/rehype': 'sequent-md' },
});

test('source configuration separates floating docs from immutable renderer and compiler pins', async () => {
  const { validateSourceConfig, selectDocumentationSource } = await import('../scripts/source-config.mjs');
  const config = validateSourceConfig(configuration());
  assert.equal(selectDocumentationSource(config, 'reference', {}).ref, 'main');
  assert.equal(selectDocumentationSource(config, 'study', { ASH_DOC_REF: other }).ref, other);
  assert.equal(selectDocumentationSource(config, 'reference', { ASH_SPEC_REF: sha }).ref, sha);
  assert.throws(() => selectDocumentationSource(config, 'reference', { ASH_SPEC_REF: '--upload-pack=bad' }), /ref/i);
  for (const change of [
    config => { config.checkpoint.revision = 'main'; },
    config => { config.repositories.railroad.ref = 'main'; },
    config => { config.reference.groups[0].route = '/docs/tools/'; },
    config => { config.reference.groups[0].directories = ['../outside']; },
    config => { config.study.source = 'absent'; },
    config => { config.renderers = {}; },
  ]) {
    const invalid = configuration(); change(invalid);
    assert.throws(() => validateSourceConfig(invalid));
  }
});

test('site Markdown derives pinned links, commands and review status from source config', async () => {
  const { default: sourceMarkdown } = await import('../scripts/source-markdown.mjs');
  const render = async (config, source = '[Spec](https://github.com/dikini/ash-lang/blob/{{checkpoint.revision}}/spec/README.md)\n\n`{{checkpoint.short}}` · {{checkpoint.reviewed}}\n\n```sh\ngit switch --detach {{checkpoint.revision}}\n```') => {
    const renderer = await sourceMarkdown({ config }).createRenderer({ syntaxHighlight: false });
    return (await renderer.render(source)).code;
  };
  const config = configuration();
  const html = String(await render(config));
  assert.match(html, new RegExp(`/blob/${sha}/spec/README.md`));
  assert.match(html, /18 October 2026/);
  assert.match(html, new RegExp(`git switch --detach ${sha}`));
  config.checkpoint.reviewed = null;
  assert.match(String(await render(config)), /Pending review/);
  config.reference.groups[0].route = '/language/';
  assert.match(String(await render(config, '[Local]({{route.spec}}syntax/)')), /href="\/language\/syntax\/"/);
  await assert.rejects(render(config, '{{checkpoint.unknown}}'), /unknown.*reference/i);
});

test('build manifest uses prepared revisions and rejects stale or conflicting provenance', async () => {
  const { assembleBuildManifest } = await import('../scripts/write-build-manifest.mjs');
  const config = configuration();
  const reference = { repository: config.repositories['ash-lang'].repository, ref: 'main', revision: sha, configHash: hash, mode: 'remote' };
  const study = { repository: config.repositories['ash-doc'].repository, ref: 'main', sourceRevision: other, ashRevision: sha, configHash: hash, toolchain: { quarto: { version: '1.10.19', sha256: hash } } };
  const renderers = Object.fromEntries(Object.entries(config.renderers).map(([name, source]) => [name, { repository: config.repositories[source].repository, revision: config.repositories[source].ref, archive: 'runtime.tgz', archiveSha256: hash, files: { 'runtime.js': hash } }]));
  const inputs = { config, configHash: hash, reference, study, renderers, site: { revision: sha, dirty: false }, lockfileHash: hash, tokenHash: hash };
  const manifest = assembleBuildManifest(inputs);
  assert.equal(manifest.sources['ash-lang'].revision, sha);
  assert.equal(manifest.sources['ash-lang'].ref, 'main');
  assert.equal(manifest.sources['ash-doc'].revision, other);
  assert.equal(manifest.sources['ash-doc'].ashRevision, sha);
  assert.equal(manifest.renderers['rrd-preview'].revision, sha);
  assert.equal(manifest.checkpoint.revision, sha);
  assert.throws(() => assembleBuildManifest({ ...inputs, reference: { ...reference, configHash: 'd'.repeat(64) } }), /stale/i);
  assert.throws(() => assembleBuildManifest({ ...inputs, reference: { ...reference, revision: 'main' } }), /revision/i);
  const wrongRenderers = structuredClone(renderers); wrongRenderers['rrd-preview'].revision = other;
  assert.throws(() => assembleBuildManifest({ ...inputs, renderers: wrongRenderers }), /renderer/i);
});

test('changing a checkpoint invalidates the serialized Astro Markdown cache configuration', async () => {
  const { default: sourceMarkdown } = await import('../scripts/source-markdown.mjs');
  const config = configuration();
  const initial = JSON.stringify(sourceMarkdown({ config }).options);
  config.checkpoint.revision = other;
  config.checkpoint.reviewed = null;
  assert.notEqual(JSON.stringify(sourceMarkdown({ config }).options), initial);
});

test('pin advancement resolves a committed milestone, resets review, and is idempotent', async () => {
  const { advanceSourcePin } = await import('../scripts/update-source-pin.mjs');
  const source = await mkdtemp(join(tmpdir(), 'ash-pin-'));
  const git = args => execFileSync('git', args, { cwd: source, encoding: 'utf8' }).trim();
  try {
    git(['init', '-q']);
    await writeFile(join(source, 'rust-toolchain.toml'), '[toolchain]\nchannel = "1.99.0"\n');
    git(['add', '.']);
    git(['-c', 'user.name=Test', '-c', 'user.email=test@example.com', '-c', 'commit.gpgsign=false', '-c', 'core.hooksPath=/dev/null', 'commit', '-qm', 'milestone']);
    git(['-c', 'tag.gpgsign=false', 'tag', 'milestone-one']);
    const revision = git(['rev-parse', 'HEAD']);
    await writeFile(join(source, 'rust-toolchain.toml'), 'uncommitted changes');
    const config = configuration();
    const result = await advanceSourcePin({ config, source, target: 'checkpoint', ref: 'milestone-one', reason: 'Milestone one', now: '2026-10-19T12:00:00Z' });
    assert.equal(result.config.checkpoint.revision, revision);
    assert.equal(result.config.checkpoint.toolchain, '1.99.0');
    assert.equal(result.config.checkpoint.reviewed, null);
    assert.equal(result.config.checkpoint.reason, 'Milestone one');
    assert.equal(result.config.repositories['ash-lang'].ref, 'main');
    assert.equal(config.checkpoint.revision, sha);
    assert.equal((await advanceSourcePin({ config: result.config, source, target: 'checkpoint', ref: 'milestone-one', reason: 'repeat' })).changed, false);
    await assert.rejects(advanceSourcePin({ config, source, target: 'checkpoint', ref: '--help', reason: 'bad' }), /ref/i);
    await assert.rejects(advanceSourcePin({ config, source, target: 'checkpoint', ref: 'missing', reason: 'bad' }));
    const renderer = await advanceSourcePin({ config, source, target: 'railroad', ref: 'milestone-one', reason: 'renderer update' });
    assert.equal(renderer.config.repositories.railroad.ref, revision);
    await assert.rejects(advanceSourcePin({ config, source, target: 'ash-lang', ref: 'milestone-one', reason: 'bad' }), /target/i);
    const bytes = JSON.stringify(config);
    await writeFile(join(source, 'sources.json'), bytes);
    assert.throws(() => execFileSync(process.execPath, [join(process.cwd(), 'scripts/update-source-pin.mjs'), '--ref', revision, '--reason', '--dry-run', '--checkout', source], { cwd: source, encoding: 'utf8', stdio: 'pipe' }), /Missing value.*--reason/);
    assert.equal(await readFile(join(source, 'sources.json'), 'utf8'), bytes);
  } finally { await rm(source, { recursive: true, force: true }); }
});

test('manifest collection checks the installed renderer bytes against their provenance', async () => {
  const { collectRendererProvenance } = await import('../scripts/write-build-manifest.mjs');
  const { sha256 } = await import('../scripts/source-config.mjs');
  const root = await mkdtemp(join(tmpdir(), 'ash-runtime-'));
  try {
    const config = configuration(); config.renderers = { 'rrd-preview': 'railroad' };
    await mkdir(join(root, 'vendor'));
    await mkdir(join(root, 'node_modules/rrd-preview'), { recursive: true });
    await writeFile(join(root, 'node_modules/rrd-preview/runtime.js'), 'original');
    await writeFile(join(root, 'vendor/runtime.tgz'), 'archive');
    const archiveIntegrity = `sha512-${createHash('sha512').update('archive').digest('base64')}`;
    await writeFile(join(root, 'package-lock.json'), JSON.stringify({ packages: { 'node_modules/rrd-preview': { resolved: 'file:vendor/runtime.tgz', integrity: archiveIntegrity } } }));
    await writeFile(join(root, 'vendor/renderers.json'), JSON.stringify({ 'rrd-preview': { repository: config.repositories.railroad.repository, revision: sha, archive: 'runtime.tgz', files: { 'runtime.js': sha256('original') } } }));
    assert.equal((await collectRendererProvenance(root, config))['rrd-preview'].archiveSha256, sha256('archive'));
    await writeFile(join(root, 'node_modules/rrd-preview/runtime.js'), 'changed');
    await assert.rejects(collectRendererProvenance(root, config), /installed renderer/i);
    await writeFile(join(root, 'node_modules/rrd-preview/runtime.js'), 'original');
    await writeFile(join(root, 'vendor/runtime.tgz'), 'corrupt archive');
    await assert.rejects(collectRendererProvenance(root, config), /archive.*integrity/i);
  } finally { await rm(root, { recursive: true, force: true }); }
});

test('local refresh validates event inputs and verifies examples after a pin update', async () => {
  const { parseRefreshOptions, refreshPlan } = await import('../scripts/refresh-docs.mjs');
  const options = parseRefreshOptions(['--event', 'milestone', '--reason', 'Milestone one', '--browser']);
  const config = configuration();
  assert.deepEqual(refreshPlan(config, options).scripts, ['check', 'build', 'test', 'test:browser']);
  assert.equal(refreshPlan(config, options).verifyExamples, false);
  config.checkpoint.reviewed = null;
  assert.equal(refreshPlan(config, options).verifyExamples, true);
  assert.throws(() => parseRefreshOptions(['--event', 'random']), /event/i);
  assert.throws(() => parseRefreshOptions(['--reason']), /value/i);
  assert.throws(() => parseRefreshOptions(['--unknown']), /option/i);
});

test('manifest generation rejects reference metadata that differs from the generated pages', async () => {
  const { verifyReferenceOutput } = await import('../scripts/write-build-manifest.mjs');
  const root = await mkdtemp(join(tmpdir(), 'ash-provenance-'));
  const publication = { revision: sha, directory: 'spec', source: { repository: 'https://github.com/dikini/ash-lang.git' }, pages: [{ file: 'README.md', route: '/spec/' }] };
  try {
    await mkdir(join(root, 'spec'));
    const write = revision => writeFile(join(root, 'spec/index.html'), `<p class="spec-source"><a href="https://github.com/dikini/ash-lang/blob/${revision}/spec/README.md">Source</a></p>`);
    await write(sha);
    await verifyReferenceOutput(root, publication);
    await write(other);
    await assert.rejects(verifyReferenceOutput(root, publication), /reference.*provenance/i);
  } finally { await rm(root, { recursive: true, force: true }); }
});
