import { createHash } from 'node:crypto';
import { readdir, readFile, writeFile, mkdir, rename, rm, cp, access } from 'node:fs/promises';
import { join, resolve, relative } from 'node:path';
import { pathToFileURL } from 'node:url';
import { spawn, execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { performance } from 'node:perf_hooks';

const hash = value => createHash('sha256').update(value).digest('hex');
export const routeCacheKey = (data, config) => hash(JSON.stringify([data, config]));
export async function searchConfiguration(root = '.') {
  return fingerprint(root, ['pagefind.json', 'pagefind.yml', 'pagefind.yaml', 'pagefind.toml']);
}
export function searchCacheKey(html, inputs, environment = process.env) {
  return hash(JSON.stringify([html, inputs, Object.entries(environment).filter(([key]) => key.startsWith('PAGEFIND_')).sort(), process.version]));
}
export function validationScope(files, sameContext = true) {
  if (!sameContext) return 'full';
  if (!files.length) return 'unchanged';
  return files.every(file => /^content\/.*\.md$/.test(file)) ? 'editorial' : 'full';
}
export async function fingerprint(root, paths) {
  const records = [];
  async function visit(path) {
    let entries;
    try { entries = await readdir(join(root, path), { withFileTypes: true }); }
    catch (error) {
      if (error.code === 'ENOENT') { records.push([path, null]); return; }
      if (error.code !== 'ENOTDIR') throw error;
      records.push([path, hash(await readFile(join(root, path)))]); return;
    }
    records.push([path, 'directory']);
    for (const entry of entries.sort((a, b) => a.name.localeCompare(b.name))) {
      if (entry.isSymbolicLink()) throw new Error(`Cannot cache symbolic link: ${path}/${entry.name}`);
      await visit(join(path, entry.name));
    }
  }
  for (const path of [...paths].sort()) await visit(path);
  return hash(JSON.stringify(records));
}
export async function reusableStage(root, receipt, input, outputs) {
  if (!receipt || receipt.input !== input) return false;
  for (const output of outputs) {
    try { await access(join(root, output)); } catch (error) { if (error.code === 'ENOENT') return false; throw error; }
  }
  return receipt.output === await fingerprint(root, outputs);
}
async function json(path) { try { return JSON.parse(await readFile(path, 'utf8')); } catch (error) { if (error.code === 'ENOENT' || error instanceof SyntaxError) return undefined; throw error; } }
async function save(path, data) {
  await mkdir(resolve(path, '..'), { recursive: true });
  const staged = `${path}.${process.pid}.tmp`;
  await writeFile(staged, JSON.stringify(data, null, 2) + '\n');
  await rename(staged, path);
}
async function run(command, args, env) {
  await new Promise((accept, reject) => {
    const child = spawn(command, args, { stdio: 'inherit', env: { ...process.env, ...env } });
    child.on('error', reject);
    child.on('exit', code => code === 0 ? accept() : reject(new Error(`${command} ${args.join(' ')} failed (${code})`)));
  });
}
async function sourceSnapshot() {
  const files = [...new Set((await promisify(execFile)('git', ['ls-files', '--cached', '--others', '--exclude-standard', '-z'], { encoding: 'utf8' })).stdout.split('\0').filter(Boolean))].sort();
  return Object.fromEntries(await Promise.all(files.map(async file => [file, await fingerprint('.', [file])])));
}
export async function build({ update = false } = {}) {
  const root = resolve('.');
  const cache = '.cache/update-build';
  const started = performance.now();
  const timings = [];
  const timed = async (name, action) => {
    const start = performance.now();
    await action();
    const seconds = (performance.now() - start) / 1000;
    timings.push({ stage: name, seconds: Number(seconds.toFixed(3)) });
    console.log(`[build] ${name}: ${seconds.toFixed(2)}s`);
  };
  const node = script => run(process.execPath, [script]);
  const npm = args => process.env.npm_execpath ? run(process.execPath, [process.env.npm_execpath, ...args]) : run('npm', args);
  const bin = (name, args = []) => run(resolve('node_modules/.bin', name), args);
  // A partial update never inherits a full-refresh success receipt.
  await rm('dist/refresh-report.json', { force: true });
  await timed('tokens', () => npm(['run', 'tokens:sync']));
  const inputs = await sourceSnapshot();
  const previous = await json(`${cache}/validated.json`);
  const changed = [...new Set([...Object.keys(inputs), ...Object.keys(previous?.inputs ?? {})])].filter(file => inputs[file] !== previous?.inputs?.[file]);
  let scope = 'full';
  // Conservative stage identity: all tracked/runtime inputs except editorial content,
  // documentation, workflow files and tests. Environment source overrides are explicit.
  const preparationInputs = Object.entries(inputs).filter(([file]) => !/^(content\/|docs\/|tests\/|\.github\/)/.test(file));
  const config = JSON.parse(await readFile('sources.json', 'utf8'));
  const base = [preparationInputs, process.version, process.env.ASH_SPEC_REF, process.env.ASH_DOC_REF, process.env.ASH_SPEC_SOURCE, process.env.ASH_DOC_QUARTO_ARCHIVE];
  for (const [name, script, outputs] of [
    ['reference', 'scripts/prepare-spec.mjs', ['.cache/ash-spec/publication.json']],
    ['study', 'scripts/prepare-study.mjs', ['.cache/ash-doc/publication.json', join('public', config.study.route)]],
  ]) {
    const input = hash(JSON.stringify([name, base]));
    const receiptPath = `${cache}/${name}.json`;
    const receipt = await json(receiptPath);
    // Local source overrides can change without changes to this repository.
    const reuse = update && !process.env.ASH_SPEC_SOURCE && !process.env.ASH_DOC_QUARTO_ARCHIVE && await reusableStage(root, receipt, input, outputs);
    await timed(`${name}${reuse ? ' (reused)' : ''}`, async () => {
      if (!reuse) {
        await rm(receiptPath, { force: true });
        await node(script);
        await save(receiptPath, { input, output: await fingerprint(root, outputs) });
      }
    });
  }
  const searchConfig = await searchConfiguration(root);
  const context = hash(JSON.stringify([await fingerprint(root, ['.cache/ash-spec/publication.json', '.cache/ash-doc/publication.json', join('public', config.study.route)]), process.env.SITE_URL ?? null, searchCacheKey('', []), searchConfig, process.version]));
  scope = previous && update ? validationScope(changed, previous.context === context) : 'full';
  await timed('astro', () => bin('astro', ['build']));
  await timed('token manifest', () => node('scripts/write-token-manifest.mjs'));
  await timed('discovery', () => node('scripts/prepare-discovery.mjs'));
  const html = [];
  async function collect(folder) {
    for (const entry of await readdir(folder, { withFileTypes: true })) {
      const path = join(folder, entry.name);
      if (entry.isDirectory() && entry.name !== 'pagefind') await collect(path);
      else if (entry.name.endsWith('.html')) html.push(relative(root, path));
    }
  }
  await collect(resolve('dist'));
  const searchInput = searchCacheKey(await fingerprint(root, html), [preparationInputs, searchConfig]);
  const searchOutput = [`${cache}/pagefind`];
  const searchReceipt = await json(`${cache}/search.json`);
  const reuseSearch = update && await reusableStage(root, searchReceipt, searchInput, searchOutput);
  await timed(`search${reuseSearch ? ' (reused)' : ''}`, async () => {
    if (reuseSearch) await cp(`${cache}/pagefind`, 'dist/pagefind', { recursive: true });
    else {
      await rm(`${cache}/search.json`, { force: true });
      await bin('pagefind', ['--site', 'dist']);
      await rm(`${cache}/pagefind`, { recursive: true, force: true });
      await cp('dist/pagefind', `${cache}/pagefind`, { recursive: true });
      await save(`${cache}/search.json`, { input: searchInput, output: await fingerprint(root, searchOutput) });
    }
  });
  await timed('links (all routes and inbound anchors)', () => node('scripts/check-links.mjs'));
  await timed('build manifest', () => node('scripts/write-build-manifest.mjs'));
  if (update) {
    console.log(`[update] ${changed.length} changed inputs; ${scope} validation.`);
    if (scope !== 'unchanged') {
      await timed('diagnostics', () => npm(['run', 'check']));
      await timed(`${scope} tests`, () => scope === 'editorial'
        ? run(process.execPath, ['--test', '--test-isolation=none', 'tests/pages.test.mjs', 'tests/discovery.test.mjs', 'tests/links.test.mjs', 'tests/release.test.mjs'])
        : npm(['test']));
    } else console.log('[update] Reusing diagnostics and tests from the matching validated inputs and publication context.');
    await save(`${cache}/validated.json`, { inputs, context });
  }
  const report = { mode: update ? 'update' : 'full', validation: update ? scope : 'not requested', changed, timings, seconds: Number(((performance.now() - started) / 1000).toFixed(3)) };
  await save(`${cache}/last-run.json`, report);
  console.log(`[build] Total: ${report.seconds.toFixed(2)}s; timings: ${cache}/last-run.json`);
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const args = process.argv.slice(2);
  if (args.some(arg => arg !== '--update')) throw new Error('Usage: node scripts/update-build.mjs [--update]');
  await build({ update: args.includes('--update') });
}
