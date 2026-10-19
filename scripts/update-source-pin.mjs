import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdir, writeFile, rename } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { readSourceConfig, validateSourceConfig, commitPattern } from './source-config.mjs';

const execute = promisify(execFile);
const git = async (source, args) => (await execute('git', args, { cwd: source, encoding: 'utf8', env: { ...process.env, GIT_TERMINAL_PROMPT: '0' } })).stdout.trim();
const validateRef = ref => assert.ok(typeof ref === 'string' && /^[A-Za-z0-9][A-Za-z0-9._/-]*$/.test(ref), 'Invalid source ref');

export async function advanceSourcePin({ config, source, target, ref, reason, now = new Date().toISOString() }) {
  assert.ok(target === 'checkpoint' || Object.values(config.renderers).includes(target), 'Unknown pin target');
  validateRef(ref);
  assert.ok(typeof reason === 'string' && reason.trim(), 'A pin-update reason is required');
  const revision = await git(source, ['rev-parse', '--verify', '--end-of-options', `${ref}^{commit}`]);
  assert.match(revision, commitPattern, 'Invalid resolved revision');
  const next = structuredClone(config);
  const current = target === 'checkpoint' ? config.checkpoint.revision : config.repositories[target].ref;
  if (current === revision) return { config: next, changed: false };
  if (target === 'checkpoint') {
    const toolchain = await git(source, ['show', `${revision}:rust-toolchain.toml`]);
    const channel = toolchain.match(/^\s*channel\s*=\s*"(\d+\.\d+\.\d+)"/m)?.[1];
    assert.ok(channel, 'Candidate must declare an exact Rust toolchain');
    Object.assign(next.checkpoint, { revision, toolchain: channel, reviewed: null, reason: reason.trim(), updatedAt: now });
  } else {
    Object.assign(next.repositories[target], { ref: revision, reason: reason.trim(), updatedAt: now });
  }
  validateSourceConfig(next);
  return { config: next, changed: true };
}

async function main() {
  const args = process.argv.slice(2), options = {};
  for (let index = 0; index < args.length; index++) {
    const name = args[index];
    assert.ok(['--target', '--ref', '--reason', '--checkout', '--dry-run'].includes(name), `Unknown option ${name}`);
    if (name === '--dry-run') options['dry-run'] = true;
    else {
      const value = args[++index];
      assert.ok(value && !value.startsWith('--'), `Missing value for ${name}`);
      options[name.slice(2)] = value;
    }
  }
  const { config } = readSourceConfig();
  const target = options.target ?? 'checkpoint';
  assert.ok(target === 'checkpoint' || Object.values(config.renderers).includes(target), 'Unknown pin target');
  validateRef(options.ref);
  assert.ok(typeof options.reason === 'string' && options.reason.trim(), 'A pin-update reason is required');
  const sourceId = target === 'checkpoint' ? config.checkpoint.source : target;
  const source = options.checkout ? resolve(options.checkout) : resolve('.cache/source-updates', sourceId);
  let ref = options.ref;
  if (!options.checkout) {
    await mkdir(source, { recursive: true });
    await git(source, ['init', '-q']);
    await git(source, ['fetch', '--depth=1', config.repositories[sourceId].repository, ref]);
    ref = 'FETCH_HEAD';
  }
  const result = await advanceSourcePin({ config, source, target, ref, reason: options.reason });
  if (result.changed && !options['dry-run']) {
    const staged = resolve(`.sources-${process.pid}.json`);
    await writeFile(staged, JSON.stringify(result.config, null, 2) + '\n');
    await rename(staged, resolve('sources.json'));
  }
  console.log(JSON.stringify({ target, changed: result.changed, dryRun: Boolean(options['dry-run']), pin: target === 'checkpoint' ? result.config.checkpoint : result.config.repositories[target] }, null, 2));
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) await main();
