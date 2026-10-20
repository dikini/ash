import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { createHash } from 'node:crypto';

export const commitPattern = /^[a-f0-9]{40}$/;
const sourcePath = /^(?:[A-Za-z0-9-]+\/)*[A-Za-z0-9.-]+$/;
export const sha256 = value => createHash('sha256').update(value).digest('hex');
export const repositoryWebUrl = repository => repository.replace(/\.git$/, '');

export function validateSourceConfig(config) {
  assert.equal(config.version, 1, 'Unsupported source configuration version');
  for (const [id, source] of Object.entries(config.repositories)) {
    assert.match(id, /^[a-z][a-z0-9-]*$/, 'Invalid source ID');
    assert.match(source.repository, /^https:\/\/[^\s]+$/, `Invalid repository for ${id}`);
    assert.ok(source.ref === 'main' || commitPattern.test(source.ref), `Invalid source ref for ${id}`);
    assert.ok(source.publicMarkdown === undefined || typeof source.publicMarkdown === 'boolean', `Invalid publicMarkdown policy for ${id}`);
  }
  assert.ok(config.repositories[config.reference.source], 'Unknown reference source');
  assert.ok(config.repositories[config.study.source], 'Unknown study source');
  assert.ok(config.repositories[config.checkpoint.source], 'Unknown checkpoint source');
  assert.match(config.checkpoint.revision, commitPattern, 'Invalid compiler revision');
  assert.match(config.checkpoint.toolchain, /^\d+\.\d+\.\d+$/, 'Pin an exact Rust toolchain');
  assert.ok(config.checkpoint.reviewed === null || /^\d{4}-\d{2}-\d{2}$/.test(config.checkpoint.reviewed), 'Invalid review date');
  const routes = new Set(), ids = new Set();
  for (const group of config.reference.groups) {
    assert.match(group.id, /^[a-z][a-z0-9-]*$/);
    assert.ok(!ids.has(group.id), 'Duplicate reference group'); ids.add(group.id);
    assert.match(group.route, /^\/(?:[a-z0-9-]+\/)+$/, 'Invalid publication route');
    assert.ok(!routes.has(group.route), 'Duplicate publication route'); routes.add(group.route);
    for (const path of [...group.directories, ...group.required]) {
      assert.match(path, sourcePath, 'Invalid source path');
      assert.ok(!path.split('/').some(segment => segment === '.' || segment === '..'), 'Invalid source path');
    }
    assert.ok(group.directories.length && group.required.includes('README.md'), 'Require a source overview');
    assert.ok(!group.allowSequentSource || group.id === 'design', 'Only design notes allow sequent source fallback');
  }
  assert.deepEqual([...ids].sort(), ['decisions', 'design', 'spec', 'tools'], 'Missing reference group');
  assert.match(config.study.route, /^\/(?:[a-z0-9-]+\/)+$/);
  assert.ok(!routes.has(config.study.route), 'Duplicate study route');
  assert.deepEqual(Object.keys(config.renderers).sort(), ['@sequent-md/core', '@sequent-md/rehype', 'rrd-preview'], 'Require every renderer runtime mapping');
  for (const source of Object.values(config.renderers)) assert.match(config.repositories[source]?.ref ?? '', commitPattern, 'Renderer revision must be pinned');
  return config;
}

export function readSourceConfig(path = resolve('sources.json')) {
  const bytes = readFileSync(path);
  return { config: validateSourceConfig(JSON.parse(bytes)), configHash: sha256(bytes) };
}

export function selectDocumentationSource(config, kind, env = process.env) {
  const id = config[kind].source;
  const source = config.repositories[id];
  const ref = env[kind === 'reference' ? 'ASH_SPEC_REF' : 'ASH_DOC_REF'] ?? source.ref;
  assert.ok(ref === 'main' || commitPattern.test(ref), 'Documentation ref must be main or a full commit SHA');
  return { id, repository: source.repository, ref };
}

export function referenceRoute(config, id, file) {
  const group = config.reference.groups.find(group => group.id === id);
  assert.ok(group, `Unknown reference group: ${id}`);
  const path = file.replace(/(?:^|\/)README\.md$/, '').replace(/\.md$/, '');
  return group.route + (path ? `${id === 'design' || id === 'decisions' ? path.toLowerCase() : path}/` : '');
}
