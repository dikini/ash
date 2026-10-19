import { mkdir, writeFile, rename, access } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { renderSpecMarkdown, specRoute } from './spec-markdown.mjs';

const execute = promisify(execFile);
const root = fileURLToPath(new URL('../', import.meta.url));
const cache = join(root, '.cache/ash-spec');
const required = ['README.md', 'lexical-structure.md', 'syntax.md', 'name-resolution.md', 'static-semantics.md', 'elaboration.md', 'dynamic-semantics.md'];
async function git(source, args) {
  const { stdout } = await execute('git', args, { cwd: source, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024, env: { ...process.env, GIT_TERMINAL_PROMPT: '0' } });
  return stdout;
}

export async function readSpecSnapshot(source, ref = 'HEAD') {
  if (!['HEAD', 'FETCH_HEAD'].includes(ref) && !/^[a-f0-9]{40}$/.test(ref)) throw new Error('Invalid specification revision');
  const revision = (await git(source, ['rev-parse', '--verify', `${ref}^{commit}`])).trim();
  if (!/^[a-f0-9]{40}$/.test(revision)) throw new Error('Invalid resolved specification revision');
  const files = (await git(source, ['ls-tree', '-r', '--name-only', revision, '--', 'spec', 'docs/spec'])).trim().split('\n');
  const directory = files.includes('spec/README.md') ? 'spec' : 'docs/spec';
  const available = files.filter(file => file.startsWith(`${directory}/`) && file.endsWith('.md')).map(file => file.slice(directory.length + 1));
  for (const file of required) if (!available.includes(file)) throw new Error(`Missing specification chapter: ${directory}/${file} at ${revision}`);
  const ordered = [...required, ...available.filter(file => !required.includes(file)).sort()];
  const chapters = [];
  for (const file of ordered) {
    if (!/^[a-zA-Z0-9-]+\.md$/.test(file)) throw new Error(`Unsupported specification chapter path: ${file}`);
    chapters.push({ file, markdown: await git(source, ['show', `${revision}:${directory}/${file}`]) });
  }
  return { revision, directory, chapters };
}

async function prepareSpec() {
  const ref = process.env.ASH_SPEC_REF ?? 'main';
  if (ref !== 'main' && !/^[a-f0-9]{40}$/.test(ref)) throw new Error('ASH_SPEC_REF must be main or a full commit SHA');
  let source, selected;
  if (process.env.ASH_SPEC_SOURCE) {
    source = resolve(process.env.ASH_SPEC_SOURCE);
    selected = ref === 'main' ? 'HEAD' : ref;
    console.log(`Reading committed specifications from ${source}…`);
  } else {
    source = join(cache, 'source');
    await mkdir(source, { recursive: true });
    try { await access(join(source, '.git')); } catch { await git(source, ['init', '-q']); }
    console.log(`Fetching ash-lang ${ref} for the specification…`);
    await git(source, ['fetch', '--depth=1', 'https://github.com/dikini/ash-lang.git', ref]);
    selected = 'FETCH_HEAD';
  }
  const snapshot = await readSpecSnapshot(source, selected);
  const pages = [];
  for (const { file, markdown } of snapshot.chapters) {
    console.log(`Rendering specification ${file}…`);
    const rendered = await renderSpecMarkdown(markdown, {
      file, chapters: snapshot.chapters.map(chapter => chapter.file),
      revision: snapshot.revision, directory: snapshot.directory,
    });
    pages.push({ file, route: specRoute(file), ...rendered });
  }
  const publication = { revision: snapshot.revision, directory: snapshot.directory, pages };
  await mkdir(cache, { recursive: true });
  const staged = join(cache, `publication-${process.pid}.json`);
  await writeFile(staged, JSON.stringify(publication) + '\n');
  await rename(staged, join(cache, 'publication.json'));
  console.log(`Prepared ${pages.length} specification chapters from ash-lang ${snapshot.revision}.`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) await prepareSpec();
