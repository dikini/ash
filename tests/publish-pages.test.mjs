import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, readFile, writeFile, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { sha256 } from '../scripts/source-config.mjs';
import { hashPagesArtifact } from '../scripts/prepare-pages.mjs';
const execute = promisify(execFile);

test('publishing pushes verified files to gh-pages and preserves source and publishing history', async () => {
  const { publishVerifiedPages } = await import('../scripts/publish-pages.mjs');
  const folder = await mkdtemp(join(tmpdir(), 'ash-publish-'));
  const root = join(folder, 'source'), remote = join(folder, 'remote.git'), artifact = join(folder, 'artifact');
  const testEnvironment = { ...process.env, GIT_CONFIG_COUNT: '1', GIT_CONFIG_KEY_0: 'commit.gpgsign', GIT_CONFIG_VALUE_0: 'false' };
  const git = async (args, cwd = root) => (await execute('git', args, { cwd, env: testEnvironment })).stdout.trim();
  try {
    await mkdir(root); await mkdir(artifact);
    await git(['init', '-q', '-b', 'main']);
    await git(['config', 'user.name', 'Test']); await git(['config', 'user.email', 'test@example.test']);
    await writeFile(join(root, '.gitignore'), '.cache/\n');
    await writeFile(join(root, 'source.txt'), 'source\n');
    await git(['add', '.']); await git(['commit', '-qm', 'Source']);
    const revision = await git(['rev-parse', 'HEAD']);
    await git(['init', '--bare', '-q', remote]);
    await git(['remote', 'add', 'origin', remote]);
    const manifest = JSON.stringify({ site: { revision, dirty: false }, hosting: { url: 'https://dikini.github.io/ash/' } });
    await writeFile(join(artifact, 'build-manifest.json'), manifest);
    const verify = async () => writeFile(join(artifact, 'refresh-report.json'), JSON.stringify({ buildManifestSha256: sha256(manifest), artifactSha256: await hashPagesArtifact(artifact), checks: ['check', 'build', 'test', 'test:browser', 'prepare:pages'] }));
    await writeFile(join(artifact, 'index.html'), '<h1>Verified</h1>');
    await writeFile(join(artifact, 'obsolete.txt'), 'old');
    await writeFile(join(artifact, '.nojekyll'), '');
    await verify();
    // The temporary publishing checkout inherits the caller's Git identity.
    const identity = { GIT_AUTHOR_NAME: 'Test', GIT_AUTHOR_EMAIL: 'test@example.test', GIT_COMMITTER_NAME: 'Test', GIT_COMMITTER_EMAIL: 'test@example.test' };
    await publishVerifiedPages({ root, artifact, environment: { ...testEnvironment, ...identity } });
    assert.equal(await git(['show', 'gh-pages:index.html'], remote), '<h1>Verified</h1>');
    assert.equal(await git(['show', 'gh-pages:.nojekyll'], remote), '');
    assert.equal(await git(['rev-parse', 'HEAD']), revision);
    const first = await git(['rev-parse', 'gh-pages'], remote);
    await rm(join(artifact, 'obsolete.txt'));
    await writeFile(join(artifact, 'index.html'), '<h1>Next</h1>');
    await assert.rejects(publishVerifiedPages({ root, artifact }), /changed after verification/i);
    await verify();
    await publishVerifiedPages({ root, artifact, environment: { ...testEnvironment, ...identity } });
    assert.equal(await git(['rev-parse', 'gh-pages^'], remote), first);
    await assert.rejects(git(['show', 'gh-pages:obsolete.txt'], remote));
    assert.equal(await readFile(join(root, 'source.txt'), 'utf8'), 'source\n');
    await writeFile(join(root, 'source.txt'), 'uncommitted');
    await assert.rejects(publishVerifiedPages({ root, artifact }), /clean/i);
    await git(['checkout', '--', 'source.txt']);
    await writeFile(join(artifact, 'build-manifest.json'), manifest + '\n');
    await assert.rejects(publishVerifiedPages({ root, artifact }), /receipt|manifest/i);
  } finally { await rm(folder, { recursive: true, force: true }); }
});
