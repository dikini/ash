import assert from 'node:assert/strict';
import { access, cp, lstat, mkdir, readFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { sha256 } from './source-config.mjs';

const execute = promisify(execFile);
const repositories = { railroad: 'railroad', sequent: 'sequent-md', split: 'split-md' };
const exists = async file => { try { await access(file); return true; } catch { return false; } };

export function studyRendererInputs(specimen, kinds) {
  return Object.entries(specimen.renderers).filter(([name]) => !kinds || kinds.includes({ railroad: 'ebnf', sequent: 'sequent', split: 'split' }[name])).map(([name, revision]) => {
    assert.ok(Object.hasOwn(repositories, name), `Unsupported study renderer: ${name}`);
    assert.match(revision, /^[a-f0-9]{40}$/, `Invalid study renderer revision: ${name}`);
    const bundle = specimen.renderer_bundles[name];
    assert.ok(bundle && /^[a-zA-Z0-9_./-]+$/.test(bundle.path) && !bundle.path.startsWith('/') && !bundle.path.split('/').includes('..'), `Invalid study renderer bundle path: ${name}`);
    assert.match(bundle.sha256, /^[a-f0-9]{64}$/, `Invalid study renderer bundle checksum: ${name}`);
    return { name, repository: `https://github.com/dikini/${repositories[name]}.git`, revision, bundle };
  });
}

export async function prepareStudyTools({ source, cache, specimen, manifest, kinds }) {
  const args = [];
  for (const input of studyRendererInputs(specimen, kinds)) {
    const directory = join(cache, 'renderers', input.name);
    await mkdir(directory, { recursive: true });
    if (!await exists(join(directory, '.git'))) await execute('git', ['init', '-q'], { cwd: directory });
    console.log(`Preparing study ${input.name} ${input.revision.slice(0, 7)}…`);
    await execute('git', ['fetch', '--depth=1', input.repository, input.revision], { cwd: directory });
    await execute('git', ['checkout', '--detach', '--force', 'FETCH_HEAD'], { cwd: directory });
    // Railroad's distribution bundle is generated rather than checked in.
    if (input.name === 'railroad' && !await exists(join(directory, input.bundle.path))) {
      await execute('npm', ['ci', '--no-audit', '--no-fund'], { cwd: directory });
      await execute('npm', ['run', 'build'], { cwd: directory });
    }
    assert.equal(sha256(await readFile(join(directory, input.bundle.path))), input.bundle.sha256, `Study ${input.name} bundle checksum mismatch`);
    args.push(`--${input.name}`, directory);
  }
  if (!kinds.length) return args;
  const tools = join(source, '.build/toolchain');
  const mermaid = join(tools, 'mermaid');
  const lock = join(source, 'rendering/toolchain/mermaid/package-lock.json');
  assert.equal(sha256(await readFile(lock)), manifest.mermaid_cli.lock_sha256, 'Study diagram npm lock checksum mismatch');
  const browserVersion = manifest.mermaid_cli.browser.match(/^Chrome ([\d.]+)$/)?.[1];
  assert.ok(browserVersion, 'Unsupported study browser pin');
  const browser = join(tools, 'browser-cache', `chrome/linux-${browserVersion}/chrome-linux64/chrome`);
  const installedLock = join(mermaid, 'package-lock.json');
  const matchingLock = await exists(installedLock) && sha256(await readFile(installedLock)) === manifest.mermaid_cli.lock_sha256;
  if (!matchingLock || !await exists(browser) || !await exists(join(mermaid, 'node_modules/puppeteer/lib/puppeteer/puppeteer.js'))) {
    if (await exists(mermaid)) assert.ok(!(await lstat(mermaid)).isSymbolicLink(), 'Cannot install study tools through a symlink');
    await mkdir(mermaid, { recursive: true });
    for (const file of ['package.json', 'package-lock.json']) await cp(join(source, 'rendering/toolchain/mermaid', file), join(mermaid, file));
    console.log('Installing locked study diagram tools and Chrome…');
    await execute('npm', ['ci', '--no-audit', '--no-fund'], { cwd: mermaid, maxBuffer: 10 * 1024 * 1024, env: { ...process.env, PUPPETEER_CACHE_DIR: join(tools, 'browser-cache'), PUPPETEER_SKIP_CHROME_HEADLESS_SHELL_DOWNLOAD: 'true' } });
  }
  assert.ok((await execute(browser, ['--version'])).stdout.includes(browserVersion), 'Study browser version differs from its pin');
  await execute('pdftocairo', ['-v']);
  args.push('--browser', resolve(browser));
  return args;
}
