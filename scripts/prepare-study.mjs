import { mkdir, readFile, writeFile, access, rm, readdir, cp } from 'node:fs/promises';
import { resolve, join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { spawn } from 'node:child_process';
import assert from 'node:assert/strict';
import { integrateStudyHtml } from './study-html.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const cache = join(root, '.cache/ash-doc');
const source = join(cache, 'source');
const rendered = join(cache, 'rendered');
const repository = 'https://github.com/dikini/ash-doc.git';
const ref = process.env.ASH_DOC_REF ?? 'main';
if (ref !== 'main' && !/^[a-f0-9]{40}$/.test(ref)) throw new Error('ASH_DOC_REF must be main or a full commit SHA');

async function exists(path) { try { await access(path); return true; } catch { return false; } }
function run(command, args, options = {}) {
  return new Promise((accept, reject) => {
    // The fetched source is an exact snapshot, never a local synchronization peer.
    const child = spawn(command, args, { cwd: source, stdio: 'inherit', env: { ...process.env, GIT_TERMINAL_PROMPT: '0', ASH_TOKENS_STANDALONE: '1' }, ...options });
    child.on('error', reject);
    child.on('exit', code => code === 0 ? accept() : reject(new Error(`${command} exited with status ${code}`)));
  });
}
async function readJson(path) { return JSON.parse(await readFile(path, 'utf8')); }
async function walk(folder) {
  const files = [];
  for (const entry of await readdir(folder, { withFileTypes: true })) {
    const path = join(folder, entry.name);
    if (entry.isDirectory()) files.push(...await walk(path));
    else files.push(path);
  }
  return files;
}

await mkdir(source, { recursive: true });
if (!await exists(join(source, '.git'))) await run('git', ['init']);
console.log(`Fetching ash-doc ${ref}…`);
await run('git', ['fetch', '--depth=1', repository, ref]);
await run('git', ['checkout', '--detach', '--force', 'FETCH_HEAD']);
// FETCH_HEAD identifies exactly the remote revision selected for this build.
const sourceRevision = (await readFile(join(source, '.git/FETCH_HEAD'), 'utf8')).split(/\s/)[0];
assert.match(sourceRevision, /^[a-f0-9]{40}$/);
const manifest = await readJson(join(source, 'rendering/toolchain/manifest.json'));
const specimen = await readJson(join(source, 'rendering/specimen.json'));
const tools = join(source, '.build/toolchain');
const quarto = join(tools, manifest.quarto.binary);
await mkdir(tools, { recursive: true });
if (!await exists(quarto)) {
  const archive = join(cache, 'quarto.tar.gz');
  if (process.env.ASH_DOC_QUARTO_ARCHIVE) await cp(resolve(process.env.ASH_DOC_QUARTO_ARCHIVE), archive);
  else {
    console.log(`Downloading checksum-pinned Quarto ${manifest.quarto.version}…`);
    const response = await fetch(manifest.quarto.url);
    if (!response.ok) throw new Error(`Quarto download failed: ${response.status}`);
    await writeFile(archive, new Uint8Array(await response.arrayBuffer()));
  }
  assert.equal(createHash('sha256').update(await readFile(archive)).digest('hex'), manifest.quarto.sha256, 'Quarto archive checksum mismatch');
  await run('tar', ['-xzf', archive, '-C', tools]);
}
for (const [file, expected] of Object.entries(manifest.fonts)) {
  assert.equal(createHash('sha256').update(await readFile(join(source, 'rendering/fonts', file))).digest('hex'), expected, `Upstream font/notice checksum mismatch: ${file}`);
}
let previous;
try { previous = await readJson(join(rendered, 'build.json')); } catch { /* first build */ }
if (previous?.source_commit !== sourceRevision || !await exists(join(rendered, 'output/index.html'))) {
  await rm(rendered, { recursive: true, force: true });
  await run('python3', [join(source, 'rendering/build.py'), '--whole-book', '--formats', 'html', '--quarto', quarto, '--output', rendered]);
} else console.log(`Reusing HTML for unchanged ash-doc ${sourceRevision.slice(0, 7)}.`);
await run('python3', [join(source, 'rendering/checks/verify.py'), rendered]);
const identity = await readJson(join(rendered, 'build.json'));
assert.equal(identity.source_commit, sourceRevision, 'Rendered documentation revision mismatch');
assert.equal(identity.ash_commit, specimen.ash_commit, 'Rendered compiler target mismatch');
const readme = await readFile(join(source, 'README.md'), 'utf8');
const snapshot = {
  repository, ref, sourceRevision,
  ashRevision: identity.ash_commit,
  milestone: readme.match(/targets \*\*`([^`]+)`\*\*/)?.[1] ?? 'development',
  lessonCount: identity.lesson_count,
  fetchedAt: new Date().toISOString(),
};
const stage = join(root, 'public/study/latest');
await rm(stage, { recursive: true, force: true });
await mkdir(dirname(stage), { recursive: true });
await cp(join(rendered, 'output'), stage, { recursive: true });
for (const file of await walk(stage)) {
  if (file.endsWith('.html')) await writeFile(file, integrateStudyHtml(await readFile(file, 'utf8'), snapshot));
}
await cp(join(rendered, 'build.json'), join(stage, 'upstream-build.json'));
await cp(join(rendered, 'sources.json'), join(stage, 'upstream-sources.json'));
await writeFile(join(stage, 'snapshot.json'), JSON.stringify(snapshot, null, 2) + '\n');
console.log(`Prepared ${snapshot.lessonCount} study lessons from ${sourceRevision}, targeting Ash ${snapshot.ashRevision}.`);
