import assert from 'node:assert/strict';
import { readFile, writeFile, rename } from 'node:fs/promises';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { createHash } from 'node:crypto';
import { resolve, join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { parseHTML } from 'linkedom';
import { readSourceConfig, commitPattern, sha256, repositoryWebUrl } from './source-config.mjs';

export async function verifyReferenceOutput(root, publication) {
  const repository = repositoryWebUrl(publication.source.repository);
  for (const group of [publication, publication.tools, publication.design, publication.decisions].filter(Boolean)) {
    for (const page of group.pages) {
      const { document } = parseHTML(await readFile(join(root, page.route, 'index.html'), 'utf8'));
      assert.equal(document.querySelector('.spec-source a')?.getAttribute('href'), `${repository}/blob/${publication.revision}/${group.directory}/${page.file}`, `Reference page provenance differs from prepared input: ${page.route}`);
    }
  }
}

export function assembleBuildManifest({ config, configHash, reference, study, renderers, site, lockfileHash, tokenHash, discovery, trigger = { event: 'manual', reason: null } }) {
  for (const [kind, prepared] of [['reference', reference], ['study', study]]) {
    assert.equal(prepared.configHash, configHash, `Stale ${kind} preparation: source configuration changed`);
    assert.equal(prepared.repository, config.repositories[config[kind].source].repository, `Stale ${kind} repository`);
    const revision = kind === 'reference' ? prepared.revision : prepared.sourceRevision;
    assert.match(revision, commitPattern, `Invalid ${kind} revision`);
    assert.ok(prepared.ref === 'main' || commitPattern.test(prepared.ref), `Invalid ${kind} requested ref`);
    if (prepared.ref !== 'main') assert.equal(prepared.ref, revision, `${kind} revision differs from requested pin`);
  }
  assert.match(study.ashRevision, commitPattern, 'Invalid study compiler revision');
  assert.ok(study.toolchain?.quarto?.version && study.toolchain.quarto.sha256, 'Missing study renderer identity');
  for (const [name, source] of Object.entries(config.renderers)) {
    assert.equal(renderers[name]?.revision, config.repositories[source].ref, `Renderer ${name} differs from configured pin; repackage it`);
    assert.equal(renderers[name]?.repository, config.repositories[source].repository, `Renderer ${name} repository mismatch`);
  }
  return {
    version: 1, generatedAt: new Date().toISOString(), site, trigger,
    inputs: { sourceConfigSha256: configHash, packageLockSha256: lockfileHash, designTokensSha256: tokenHash, node: process.version },
    sources: {
      [config.reference.source]: { repository: reference.repository, ref: reference.ref, revision: reference.revision, mode: reference.mode, publications: config.reference.groups.map(({ id, directories, route }) => ({ id, directories, route })) },
      [config.study.source]: { repository: study.repository, ref: study.ref, revision: study.sourceRevision, ashRevision: study.ashRevision, route: config.study.route, toolchain: study.toolchain },
    },
    checkpoint: config.checkpoint, renderers, ...(discovery ? { discovery } : {}),
  };
}

export async function collectRendererProvenance(root, config) {
  const provenance = JSON.parse(await readFile(join(root, 'vendor/renderers.json'), 'utf8'));
  const lock = JSON.parse(await readFile(join(root, 'package-lock.json'), 'utf8'));
  const result = {};
  for (const name of Object.keys(config.renderers)) {
    const definition = provenance[name];
    assert.ok(definition, `Missing renderer provenance: ${name}`);
    assert.match(definition.archive, /^[A-Za-z0-9.-]+\.tgz$/, 'Invalid renderer archive');
    for (const [file, expected] of Object.entries(definition.files)) {
      assert.ok(!file.startsWith('/') && !file.split('/').includes('..'), 'Invalid renderer file path');
      const actual = sha256(await readFile(join(root, 'node_modules', name, file)));
      assert.equal(actual, expected, `Installed renderer ${name}/${file} differs from recorded source`);
    }
    const archive = await readFile(join(root, 'vendor', definition.archive));
    const locked = lock.packages?.[`node_modules/${name}`];
    assert.equal(locked?.resolved, `file:vendor/${definition.archive}`, `Renderer ${name} archive differs from npm lockfile`);
    const integrity = `sha512-${createHash('sha512').update(archive).digest('base64')}`;
    assert.equal(integrity, locked.integrity, `Renderer ${name} archive integrity differs from npm lockfile`);
    result[name] = { ...definition, archiveSha256: sha256(archive), archiveIntegrity: integrity };
  }
  return result;
}

async function main() {
  const root = resolve('.');
  const { config, configHash } = readSourceConfig();
  const publication = JSON.parse(await readFile('.cache/ash-spec/publication.json', 'utf8'));
  assert.equal(publication.source?.revision, publication.revision, 'Reference provenance differs from rendered input');
  await verifyReferenceOutput(resolve('dist'), publication);
  const study = JSON.parse(await readFile(join('dist', config.study.route, 'snapshot.json'), 'utf8'));
  const execute = promisify(execFile);
  const revision = (await execute('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' })).stdout.trim();
  assert.match(revision, commitPattern, 'Invalid website revision');
  const dirty = Boolean((await execute('git', ['status', '--porcelain'], { encoding: 'utf8' })).stdout.trim());
  const event = process.env.ASH_BUILD_EVENT ?? 'manual';
  assert.ok(['manual', 'milestone', 'release', 'commit'].includes(event), 'Invalid build event');
  const catalogueBytes = await readFile('dist/content-index.json');
  const catalogue = JSON.parse(catalogueBytes);
  assert.equal(catalogue.sourceConfigSha256, configHash, 'Stale discovery catalogue: source configuration changed');
  const manifest = assembleBuildManifest({
    config, configHash, reference: publication.source, study,
    renderers: await collectRendererProvenance(root, config), site: { revision, dirty },
    lockfileHash: sha256(await readFile('package-lock.json')),
    tokenHash: sha256(await readFile('dist/tokens.css')),
    discovery: { baseUrl: catalogue.baseUrl, pageCount: catalogue.pages.length, generatedMarkdownCount: catalogue.pages.filter(page => page.markdown?.startsWith('/')).length, catalogueSha256: sha256(catalogueBytes), llmsSha256: sha256(await readFile('dist/llms.txt')), markdownSetSha256: sha256(JSON.stringify(catalogue.pages.map(page => [page.url, page.markdown, page.markdownSha256]))) },
    trigger: { event, reason: process.env.ASH_BUILD_REASON ?? null },
  });
  const staged = resolve(`dist/.build-manifest-${process.pid}.json`);
  await writeFile(staged, JSON.stringify(manifest, null, 2) + '\n');
  await rename(staged, resolve('dist/build-manifest.json'));
  console.log(`Build manifest recorded Ash ${manifest.sources[config.reference.source].revision.slice(0, 7)}, study ${study.sourceRevision.slice(0, 7)}, and all pinned inputs.`);
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) await main();
