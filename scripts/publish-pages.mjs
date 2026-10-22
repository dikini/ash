import assert from 'node:assert/strict';
import { cp, mkdir, mkdtemp, readFile, readdir, rm } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { execFile, spawn } from 'node:child_process';
import { promisify } from 'node:util';
import { preparePages, hashPagesArtifact } from './prepare-pages.mjs';
import { sha256 } from './source-config.mjs';

const execute = promisify(execFile);

export async function publishVerifiedPages({ root = resolve('.'), artifact = resolve('.cache/github-pages/ash'), environment = process.env } = {}) {
  const git = async (args, cwd = root) => (await execute('git', args, { cwd, env: environment })).stdout.trim();
  assert.equal(await git(['status', '--porcelain']), '', 'Publishing requires a clean source checkout');
  const revision = await git(['rev-parse', 'HEAD']);
  const manifestBytes = await readFile(join(artifact, 'build-manifest.json'));
  const manifest = JSON.parse(manifestBytes);
  const receipt = JSON.parse(await readFile(join(artifact, 'refresh-report.json'), 'utf8'));
  assert.equal(manifest.site.revision, revision, 'Prepared pages belong to a different source commit');
  assert.equal(manifest.site.dirty, false, 'Prepared pages were built from uncommitted changes');
  assert.equal(receipt.buildManifestSha256, sha256(manifestBytes), 'Verification receipt does not match the manifest');
  for (const check of ['check', 'build', 'test', 'test:browser', 'prepare:pages']) assert.ok(receipt.checks.includes(check), `Missing local publication check: ${check}`);
  assert.equal(receipt.artifactSha256, await hashPagesArtifact(artifact), 'Prepared pages changed after verification');
  const remote = await git(['remote', 'get-url', 'origin']);
  const existing = await git(['ls-remote', '--heads', remote, 'refs/heads/gh-pages']);
  const cache = join(root, '.cache');
  await mkdir(cache, { recursive: true });
  const checkout = await mkdtemp(join(cache, 'publish-pages-'));
  try {
    await git(['init', '-q', '-b', 'gh-pages'], checkout);
    await git(['remote', 'add', 'origin', remote], checkout);
    if (existing) {
      await git(['fetch', '--depth=1', 'origin', 'gh-pages'], checkout);
      await git(['checkout', '-B', 'gh-pages', 'FETCH_HEAD'], checkout);
    }
    for (const name of await readdir(checkout)) if (name !== '.git') await rm(join(checkout, name), { recursive: true, force: true });
    await cp(artifact, checkout, { recursive: true });
    await git(['add', '--all'], checkout);
    const changed = await git(['diff', '--cached', '--name-only'], checkout);
    if (changed) await git(['commit', '-m', `Publish website from ${revision.slice(0, 7)}`], checkout);
    await git(['push', 'origin', 'HEAD:refs/heads/gh-pages'], checkout);
    console.log(`Published locally verified ${revision.slice(0, 7)} to origin/gh-pages for ${manifest.hosting.url}.`);
  } finally { await rm(checkout, { recursive: true, force: true }); }
}

async function main() {
  const args = process.argv.slice(2);
  assert.ok(args.length === 0 || (args.length === 1 && args[0] === '--prepared'), 'Usage: npm run publish:pages -- [--prepared]');
  const url = process.env.PAGES_URL ?? 'https://dikini.github.io/ash/';
  const environment = { ...process.env, SITE_URL: new URL(url).origin, PAGES_URL: url };
  assert.equal((await execute('git', ['status', '--porcelain'])).stdout.trim(), '', 'Commit source changes before publishing');
  if (!args.includes('--prepared')) {
    await new Promise((accept, reject) => {
      const child = spawn(process.execPath, ['scripts/refresh-docs.mjs', '--browser', '--event', 'manual', '--reason', 'Publish website update'], { stdio: 'inherit', env: environment });
      child.on('error', reject);
      child.on('exit', code => code === 0 ? accept() : reject(new Error(`Local publication checks failed (${code})`)));
    });
    await preparePages({ url });
  }
  await publishVerifiedPages({ environment });
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) await main();
