import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdir, readFile, rm, writeFile, rename } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { readSourceConfig, sha256 } from './source-config.mjs';

export function parseRefreshOptions(args) {
  const options = { event: 'manual', reason: 'Local documentation refresh', browser: false, verifyExamples: false };
  for (let index = 0; index < args.length; index++) {
    const option = args[index];
    if (option === '--browser') options.browser = true;
    else if (option === '--verify-examples') options.verifyExamples = true;
    else {
      assert.ok(['--event', '--reason', '--compiler-source'].includes(option), `Unknown option: ${option}`);
      const value = args[++index];
      assert.ok(value && !value.startsWith('--'), `Missing value for ${option}`);
      options[option === '--compiler-source' ? 'compilerSource' : option.slice(2)] = value;
    }
  }
  assert.ok(['manual', 'milestone', 'release', 'commit'].includes(options.event), 'Invalid refresh event');
  assert.ok(options.reason.trim(), 'A refresh reason is required');
  return options;
}

export function refreshPlan(config, options) {
  return {
    scripts: ['check', 'build', 'test', ...(options.browser ? ['test:browser'] : [])],
    verifyExamples: Boolean(options.verifyExamples || !config.checkpoint.reviewed),
  };
}

async function main() {
  const options = parseRefreshOptions(process.argv.slice(2));
  const { config, configHash } = readSourceConfig();
  const plan = refreshPlan(config, options);
  const environment = { ...process.env, ASH_BUILD_EVENT: options.event, ASH_BUILD_REASON: options.reason, GIT_TERMINAL_PROMPT: '0' };
  const run = (command, args, cwd = process.cwd()) => new Promise((accept, reject) => {
    const child = spawn(command, args, { cwd, stdio: 'inherit', env: environment });
    child.on('error', reject);
    child.on('exit', code => code === 0 ? accept() : reject(new Error(`${command} ${args.join(' ')} failed (${code})`)));
  });
  const npm = script => process.env.npm_execpath
    ? run(process.execPath, [process.env.npm_execpath, 'run', script]) : run('npm', ['run', script]);
  await rm('dist/refresh-report.json', { force: true });
  const verified = [];
  if (plan.verifyExamples) {
    const source = resolve(options.compilerSource ?? '.cache/ash-compiler/source');
    if (!options.compilerSource) {
      await mkdir(source, { recursive: true });
      await run('git', ['init', '-q'], source);
      await run('git', ['fetch', '--depth=1', config.repositories[config.checkpoint.source].repository, config.checkpoint.revision], source);
      await run('git', ['checkout', '--detach', '--force', 'FETCH_HEAD'], source);
    }
    await run('rustup', ['toolchain', 'install', config.checkpoint.toolchain, '--profile', 'minimal']);
    environment.ASH_SOURCE = source;
    await npm('verify:examples');
    verified.push('verify:examples');
  }
  for (const script of plan.scripts) { await npm(script); verified.push(script); }
  assert.equal(readSourceConfig().configHash, configHash, 'Source config changed during refresh');
  const manifestBytes = await readFile('dist/build-manifest.json');
  assert.equal(JSON.parse(manifestBytes).inputs.sourceConfigSha256, configHash, 'Stale build manifest');
  const receipt = { version: 1, completedAt: new Date().toISOString(), event: options.event, reason: options.reason, checks: verified, buildManifestSha256: sha256(manifestBytes) };
  const staged = resolve(`dist/.refresh-report-${process.pid}.json`);
  await writeFile(staged, JSON.stringify(receipt, null, 2) + '\n');
  await rename(staged, resolve('dist/refresh-report.json'));
  console.log(`Documentation refresh verified. Output: dist/; build-manifest.json records exact inputs; refresh-report.json records completed checks.`);
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) await main();
