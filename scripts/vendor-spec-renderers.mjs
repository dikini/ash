// Package the existing HTML runtimes without the editor extensions or SVG fonts.
import { mkdtemp, mkdir, cp, readFile, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve, join } from 'node:path';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const destination = join(root, 'vendor');
const temporary = await mkdtemp(join(tmpdir(), 'ash-renderers-'));
await mkdir(destination, { recursive: true });
const provenance = {};
const definitions = [
  { name: 'rrd-preview', source: process.env.RAILROAD_SOURCE ?? '../railroad', exports: { './rehype': './dist/rehype.js' },
    dependencies: { linkedom: '^0.18.13' }, files: ['dist/rehype.js', 'LICENSE', 'THIRD_PARTY_NOTICES.md'] },
  { name: '@sequent-md/core', source: process.env.SEQUENT_SOURCE ?? '../sequent-md', subdir: 'packages/core',
    exports: './src/index.js', files: ['src/index.js', 'src/symbols.js'] },
  { name: '@sequent-md/rehype', source: process.env.SEQUENT_SOURCE ?? '../sequent-md', subdir: 'packages/rehype',
    exports: './src/index.js', dependencies: { '@sequent-md/core': '0.1.0' }, files: ['src/index.js'] },
];
try {
  for (const definition of definitions) {
    const repository = resolve(root, definition.source);
    const source = join(repository, definition.subdir ?? '');
    const trackedChanges = execFileSync('git', ['status', '--porcelain', '--untracked-files=no', '--',
      ...(definition.subdir ? [definition.subdir] : ['src', 'scripts', 'package.json', 'package-lock.json', 'vendor'])], { cwd: repository, encoding: 'utf8' }).trim();
    if (trackedChanges) throw new Error(`Commit renderer source changes before packaging ${definition.name}:\n${trackedChanges}`);
    const stage = join(temporary, definition.name.replaceAll('/', '-'));
    await mkdir(stage, { recursive: true });
    const checksums = {};
    for (const file of definition.files) {
      await mkdir(join(stage, file, '..'), { recursive: true });
      await cp(join(source, file), join(stage, file));
      checksums[file] = createHash('sha256').update(await readFile(join(stage, file))).digest('hex');
    }
    await writeFile(join(stage, 'package.json'), JSON.stringify({
      name: definition.name, version: '0.1.0', type: 'module', exports: definition.exports,
      ...(definition.dependencies ? { dependencies: definition.dependencies } : {}),
      ...(definition.name === 'rrd-preview' ? { license: 'MIT' } : {}),
    }, null, 2) + '\n');
    const packed = JSON.parse(execFileSync('npm', ['pack', '--json', '--ignore-scripts', '--pack-destination', destination], { cwd: stage, encoding: 'utf8' }))[0];
    provenance[definition.name] = {
      repository: `https://github.com/dikini/${definition.name === 'rrd-preview' ? 'railroad' : 'sequent-md'}`,
      revision: execFileSync('git', ['rev-parse', 'HEAD'], { cwd: repository, encoding: 'utf8' }).trim(),
      archive: packed.filename, files: checksums,
    };
  }
  await writeFile(join(destination, 'renderers.json'), JSON.stringify(provenance, null, 2) + '\n');
} finally { await rm(temporary, { recursive: true, force: true }); }
