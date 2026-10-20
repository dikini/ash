import { mkdir, writeFile, rename, access } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { renderSpecMarkdown } from './spec-markdown.mjs';
import { readSourceConfig, selectDocumentationSource, referenceRoute } from './source-config.mjs';
import { readMarkdown } from './content-metadata.mjs';

const execute = promisify(execFile);
const root = fileURLToPath(new URL('../', import.meta.url));
const cache = join(root, '.cache/ash-spec');
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
    if (!/^(?:[a-zA-Z0-9-]+\/)*[a-zA-Z0-9-]+\.md$/.test(file)) throw new Error(`Unsupported ${label} chapter path: ${file}`);
    chapters.push({ file, markdown: await git(source, ['show', `${revision}:${directory}/${file}`]) });
  }
  return { revision, directory, chapters };
}

async function readGroupSnapshot(source, ref, id, config = readSourceConfig().config) {
  const definition = config.reference.groups.find(group => group.id === id);
  const snapshot = await readSnapshot(source, ref, definition.directories, definition.required, `${id} documentation`);
  const routes = new Set();
  for (const { file } of snapshot.chapters) {
    const route = referenceRoute(config, id, file);
    if (routes.has(route)) throw new Error(`Duplicate ${id} route: ${route} (${file})`);
    routes.add(route);
  }
  return snapshot;
}
export const readSpecSnapshot = (source, ref = 'HEAD') => readGroupSnapshot(source, ref, 'spec');
export const readToolSnapshot = (source, ref = 'HEAD') => readGroupSnapshot(source, ref, 'tools');
export const readDesignSnapshot = (source, ref = 'HEAD') => readGroupSnapshot(source, ref, 'design');
export const readDecisionSnapshot = (source, ref = 'HEAD') => readGroupSnapshot(source, ref, 'decisions');
export const designRoute = file => referenceRoute(readSourceConfig().config, 'design', file);
export const decisionRoute = file => referenceRoute(readSourceConfig().config, 'decisions', file);

async function prepareSpec() {
  const { config, configHash } = readSourceConfig();
  const { repository, ref } = selectDocumentationSource(config, 'reference');
  let source, selected;
  if (process.env.ASH_SPEC_SOURCE) {
    source = resolve(process.env.ASH_SPEC_SOURCE);
    selected = ref === 'main' ? 'HEAD' : ref;
    console.log(`Reading committed reference documentation from ${source}…`);
  } else {
    source = join(cache, 'source');
    await mkdir(source, { recursive: true });
    try { await access(join(source, '.git')); } catch { await git(source, ['init', '-q']); }
    console.log(`Fetching ash-lang ${ref} for specifications, tools, design notes, and decisions…`);
    await git(source, ['fetch', '--depth=1', repository, ref]);
    selected = 'FETCH_HEAD';
  }
  const snapshot = await readGroupSnapshot(source, selected, 'spec', config);
  const groups = new Map([['spec', snapshot]]);
  // Resolve every configured group from one immutable commit.
  for (const { id } of config.reference.groups) if (id !== 'spec') groups.set(id, await readGroupSnapshot(source, snapshot.revision, id, config));
  for (const [id, group] of groups) {
    const required = config.reference.groups.find(definition => definition.id === id).required;
    group.chapters = group.chapters.filter(chapter => {
      if (!readMarkdown(chapter.markdown).metadata.draft) return true;
      if (required.includes(chapter.file)) throw new Error(`Required ${id} document is a draft: ${chapter.file}`);
      return false;
    });
  }
  const routes = Object.fromEntries([...groups].flatMap(([id, group]) => group.chapters.map(({ file }) => [`${group.directory}/${file}`, referenceRoute(config, id, file)])));
  async function renderGroup(id, group) {
    const pages = [];
    for (const { file, markdown } of group.chapters) {
      console.log(`Rendering ${group.directory}/${file}…`);
      const rendered = await renderSpecMarkdown(markdown, {
        file, chapters: group.chapters.map(chapter => chapter.file),
        revision: snapshot.revision, directory: group.directory, routes,
        repository,
        allowSequentSource: Boolean(config.reference.groups.find(definition => definition.id === id).allowSequentSource),
      });
      for (const warning of rendered.warnings) console.warn(warning);
      pages.push({ file, route: referenceRoute(config, id, file), ...rendered });
    }
    return { directory: group.directory, pages };
  }
  const publication = { revision: snapshot.revision, source: { repository, ref, revision: snapshot.revision, configHash, mode: process.env.ASH_SPEC_SOURCE ? 'local' : 'remote' } };
  for (const [id, group] of groups) {
    const rendered = await renderGroup(id, group);
    if (id === 'spec') Object.assign(publication, rendered); else publication[id] = rendered;
  }
  await mkdir(cache, { recursive: true });
  const staged = join(cache, `publication-${process.pid}.json`);
  await writeFile(staged, JSON.stringify(publication) + '\n');
  await rename(staged, join(cache, 'publication.json'));
  console.log(`Prepared ${publication.pages.length} specification chapters, ${publication.tools.pages.length} tool documents, ${publication.design.pages.length} design documents, and ${publication.decisions.pages.length} decision documents from ash-lang ${snapshot.revision}.`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) await prepareSpec();
