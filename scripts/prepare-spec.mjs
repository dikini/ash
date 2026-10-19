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

async function readSnapshot(source, ref, directories, requiredChapters, label) {
  if (!['HEAD', 'FETCH_HEAD'].includes(ref) && !/^[a-f0-9]{40}$/.test(ref)) throw new Error('Invalid specification revision');
  const revision = (await git(source, ['rev-parse', '--verify', `${ref}^{commit}`])).trim();
  if (!/^[a-f0-9]{40}$/.test(revision)) throw new Error('Invalid resolved specification revision');
  const files = (await git(source, ['ls-tree', '-r', '--name-only', revision, '--', ...directories])).trim().split('\n');
  const directory = directories.find(directory => files.includes(`${directory}/README.md`)) ?? directories[0];
  const available = files.filter(file => file.startsWith(`${directory}/`) && file.endsWith('.md')).map(file => file.slice(directory.length + 1));
  for (const file of requiredChapters) if (!available.includes(file)) throw new Error(`Missing ${label} chapter: ${directory}/${file} at ${revision}`);
  const ordered = [...requiredChapters, ...available.filter(file => !requiredChapters.includes(file)).sort()];
  const chapters = [];
  for (const file of ordered) {
    if (!/^[a-zA-Z0-9-]+\.md$/.test(file)) throw new Error(`Unsupported ${label} chapter path: ${file}`);
    chapters.push({ file, markdown: await git(source, ['show', `${revision}:${directory}/${file}`]) });
  }
  return { revision, directory, chapters };
}

export const readSpecSnapshot = (source, ref = 'HEAD') => readSnapshot(source, ref, ['spec', 'docs/spec'], required, 'specification');
export const readToolSnapshot = (source, ref = 'HEAD') => readSnapshot(source, ref, ['docs/tools'], ['README.md', 'ash.md'], 'tool documentation');
const toolRoute = file => file === 'README.md' ? '/docs/tools/' : `/docs/tools/${file.replace(/\.md$/, '')}/`;

async function prepareSpec() {
  const ref = process.env.ASH_SPEC_REF ?? 'main';
  if (ref !== 'main' && !/^[a-f0-9]{40}$/.test(ref)) throw new Error('ASH_SPEC_REF must be main or a full commit SHA');
  let source, selected;
  if (process.env.ASH_SPEC_SOURCE) {
    source = resolve(process.env.ASH_SPEC_SOURCE);
    selected = ref === 'main' ? 'HEAD' : ref;
    console.log(`Reading committed reference documentation from ${source}…`);
  } else {
    source = join(cache, 'source');
    await mkdir(source, { recursive: true });
    try { await access(join(source, '.git')); } catch { await git(source, ['init', '-q']); }
    console.log(`Fetching ash-lang ${ref} for the specification and tool documentation…`);
    await git(source, ['fetch', '--depth=1', 'https://github.com/dikini/ash-lang.git', ref]);
    selected = 'FETCH_HEAD';
  }
  const snapshot = await readSpecSnapshot(source, selected);
  // Resolve both groups from one immutable commit, including local overrides.
  const toolSnapshot = await readToolSnapshot(source, snapshot.revision);
  const routes = Object.fromEntries([
    ...snapshot.chapters.map(({ file }) => [`${snapshot.directory}/${file}`, specRoute(file)]),
    ...toolSnapshot.chapters.map(({ file }) => [`${toolSnapshot.directory}/${file}`, toolRoute(file)]),
  ]);
  async function renderGroup(group, route) {
    const pages = [];
    for (const { file, markdown } of group.chapters) {
      console.log(`Rendering ${group.directory}/${file}…`);
      const rendered = await renderSpecMarkdown(markdown, {
        file, chapters: group.chapters.map(chapter => chapter.file),
        revision: snapshot.revision, directory: group.directory, routes,
      });
      pages.push({ file, route: route(file), ...rendered });
    }
    return { directory: group.directory, pages };
  }
  const publication = { revision: snapshot.revision, ...await renderGroup(snapshot, specRoute), tools: await renderGroup(toolSnapshot, toolRoute) };
  await mkdir(cache, { recursive: true });
  const staged = join(cache, `publication-${process.pid}.json`);
  await writeFile(staged, JSON.stringify(publication) + '\n');
  await rename(staged, join(cache, 'publication.json'));
  console.log(`Prepared ${publication.pages.length} specification chapters and ${publication.tools.pages.length} tool documents from ash-lang ${snapshot.revision}.`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) await prepareSpec();
