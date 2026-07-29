import { access, lstat, mkdir, readdir, readFile, realpath, writeFile } from 'node:fs/promises';
import { constants } from 'node:fs';
import { dirname, extname, relative, resolve, sep } from 'node:path';

const RAILROAD_EBNF = '/home/dikini/Projects/railroad/src/ebnf.js';
const SEQUENT_RENDERER = '/home/dikini/Projects/sequent-md/packages/core/src/index.js';
const TARGET_FENCE_OPEN = /^( {0,3})(`{3,})(ebnf|sequent)\s*$/;
const EBNF_STRUCTURAL_CHARACTERS = new Set(['=', ';', '|', '(', ')', '[', ']', '{', '}', '?', '+', '*']);

class ValidationError extends Error {}

function usage() {
  return 'Usage: node tools/docs/validate_language_reference_fences.mjs --root <manual-root> --extract-dir <temporary-directory>';
}

function parseArguments(arguments_) {
  let root;
  let extractDir;

  for (let index = 0; index < arguments_.length; index += 1) {
    const argument = arguments_[index];
    if (argument === '--root' || argument === '--extract-dir') {
      const value = arguments_[index + 1];
      if (!value || value.startsWith('--')) {
        throw new ValidationError(`Missing value for ${argument}\n${usage()}`);
      }
      if (argument === '--root') {
        if (root !== undefined) throw new ValidationError('Duplicate --root argument');
        root = value;
      } else {
        if (extractDir !== undefined) throw new ValidationError('Duplicate --extract-dir argument');
        extractDir = value;
      }
      index += 1;
    } else {
      throw new ValidationError(`Unknown argument: ${argument}\n${usage()}`);
    }
  }

  if (!root || !extractDir) throw new ValidationError(usage());
  if (root !== 'docs/reference/language') {
    throw new ValidationError('Manual root must be exactly docs/reference/language');
  }
  const resolvedRoot = resolve(root);
  const resolvedExtractDir = resolve(extractDir);
  if (resolvedExtractDir === resolvedRoot || resolvedExtractDir.startsWith(`${resolvedRoot}${sep}`)) {
    throw new ValidationError('Extract directory must be outside the manual root to keep Markdown read-only');
  }
  return { root: resolvedRoot, extractDir: resolvedExtractDir };
}

function displayPath(root, path) {
  return relative(root, path).split(sep).join('/');
}

function fileDescription(fence) {
  return `${fence.source}:${fence.line} [${fence.kind}]`;
}

async function collectMarkdownFiles(root) {
  let rootInfo;
  try {
    rootInfo = await lstat(root);
  } catch (error) {
    throw new ValidationError(`Cannot inspect manual root ${root}: ${error.message}`);
  }
  if (!rootInfo.isDirectory()) throw new ValidationError(`Manual root is not a directory: ${root}`);

  const files = [];

  async function visit(directory) {
    let entries;
    try {
      entries = await readdir(directory, { withFileTypes: true });
    } catch (error) {
      throw new ValidationError(`Unreadable directory under manual root: ${displayPath(root, directory)}: ${error.message}`);
    }

    entries.sort((left, right) => left.name.localeCompare(right.name));
    for (const entry of entries) {
      const path = resolve(directory, entry.name);
      if (entry.isSymbolicLink()) {
        throw new ValidationError(`Refusing symlink under manual root: ${displayPath(root, path)}`);
      }
      if (entry.isDirectory()) {
        await visit(path);
      } else if (entry.isFile() && extname(entry.name).toLowerCase() === '.md') {
        files.push(path);
      }
    }
  }

  await visit(root);
  return files;
}

async function readMarkdown(root, path) {
  let info;
  try {
    info = await lstat(path);
    await access(path, constants.R_OK);
  } catch (error) {
    throw new ValidationError(`Unreadable Markdown file ${displayPath(root, path)}: ${error.message}`);
  }

  // The test runner can be privileged, in which case access() accepts an intentionally
  // permissionless file. Treat it as unreadable so the validator reports the condition a
  // normal documentation gate would encounter.
  if ((info.mode & 0o444) === 0) {
    throw new ValidationError(`Unreadable Markdown file ${displayPath(root, path)}: no read permission bits`);
  }

  try {
    return await readFile(path, 'utf8');
  } catch (error) {
    throw new ValidationError(`Unreadable Markdown file ${displayPath(root, path)}: ${error.message}`);
  }
}

function closingFencePattern(markerLength) {
  return new RegExp(`^ {0,3}${String.fromCharCode(96)}{${markerLength},}\\s*$`);
}

function extractFences(source, markdown) {
  const fences = [];
  const lines = markdown.replace(/\r\n?/g, '\n').split('\n');
  let active;

  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    if (active) {
      if (closingFencePattern(active.markerLength).test(line)) {
        fences.push({
          kind: active.kind,
          source,
          line: active.line,
          body: active.body.join('\n'),
        });
        active = undefined;
      } else {
        active.body.push(line);
      }
      continue;
    }

    const opening = line.match(TARGET_FENCE_OPEN);
    if (opening) {
      active = {
        markerLength: opening[2].length,
        kind: opening[3],
        line: index + 1,
        body: [],
      };
    }
  }

  if (active) {
    throw new ValidationError(`Malformed ${active.kind} fence: ${source}:${active.line} has no closing fence`);
  }

  return fences;
}

function outsideQuotedCharacters(source, visitor) {
  let quoted = false;
  let escaped = false;

  for (let index = 0; index < source.length; index += 1) {
    const character = source[index];
    if (quoted) {
      if (escaped) {
        escaped = false;
      } else if (character === '\\') {
        escaped = true;
      } else if (character === '"') {
        quoted = false;
      }
      continue;
    }

    if (character === '"') {
      quoted = true;
    } else {
      visitor(character, index);
    }
  }

  if (quoted) throw new ValidationError('unterminated quoted terminal');
}

function validateProductionSegment(segment) {
  if (segment.trim() === '') return;
  if (!/^\s*[A-Za-z_][A-Za-z0-9_]*\s*=/.test(segment)) {
    throw new ValidationError('production has no rule name followed by =');
  }
}

function preflightEbnf(source) {
  if (source.trim() === '') throw new ValidationError('fence is empty');

  let segment = '';
  let forbiddenAssignment = false;
  let punctuation;
  outsideQuotedCharacters(source, (character, index) => {
    if (source.slice(index, index + 3) === '::=') forbiddenAssignment = true;
    if (character === ';') {
      validateProductionSegment(segment);
      segment = '';
      return;
    }
    segment += character;
    if (
      !punctuation
      && !/[A-Za-z0-9_\s]/.test(character)
      && !EBNF_STRUCTURAL_CHARACTERS.has(character)
    ) {
      punctuation = character;
    }
  });

  if (forbiddenAssignment) throw new ValidationError('production uses forbidden ::= assignment');
  if (segment.trim() !== '') throw new ValidationError('production has no terminal semicolon');
  if (punctuation) throw new ValidationError(`terminal punctuation must be quoted: ${punctuation}`);
}

function isWithin(directory, path) {
  return path === directory || path.startsWith(`${directory}${sep}`);
}

async function existingPath(path, description) {
  try {
    return await lstat(path);
  } catch (error) {
    if (error.code === 'ENOENT') return undefined;
    throw new ValidationError(`Cannot inspect ${description}: ${error.message}`);
  }
}

async function rejectExtractSymlinkAncestors(extractDir) {
  let candidate = extractDir;
  while (true) {
    const info = await existingPath(candidate, 'extract directory');
    if (info?.isSymbolicLink()) {
      throw new ValidationError(`Extract directory contains a symlink: ${candidate}`);
    }
    const parent = dirname(candidate);
    if (parent === candidate) return;
    candidate = parent;
  }
}

async function prepareExtractionDirectory(root, extractDir) {
  await rejectExtractSymlinkAncestors(extractDir);
  const existingExtractDir = await existingPath(extractDir, 'extract directory');
  if (existingExtractDir && !existingExtractDir.isDirectory()) {
    throw new ValidationError(`Extract directory is not a directory: ${extractDir}`);
  }

  try {
    await mkdir(extractDir, { recursive: true });
  } catch (error) {
    throw new ValidationError(`Cannot create extract directory ${extractDir}: ${error.message}`);
  }

  const physicalRoot = await realpath(root);
  const physicalExtractDir = await realpath(extractDir);
  if (isWithin(physicalRoot, physicalExtractDir)) {
    throw new ValidationError('Extract directory resolves inside the manual root and would write Markdown');
  }

  const fencesDir = resolve(extractDir, 'fences');
  const existingFencesDir = await existingPath(fencesDir, 'fences directory');
  if (existingFencesDir?.isSymbolicLink()) {
    throw new ValidationError(`Fences directory is a symlink: ${fencesDir}`);
  }
  if (existingFencesDir && !existingFencesDir.isDirectory()) {
    throw new ValidationError(`Fences path is not a directory: ${fencesDir}`);
  }
  try {
    await mkdir(fencesDir, { recursive: true });
  } catch (error) {
    throw new ValidationError(`Cannot create fences directory ${fencesDir}: ${error.message}`);
  }

  const physicalFencesDir = await realpath(fencesDir);
  if (!isWithin(physicalExtractDir, physicalFencesDir)) {
    throw new ValidationError(`Fences directory resolves outside the extract directory: ${fencesDir}`);
  }
  return { physicalExtractDir, physicalFencesDir };
}

async function rejectOutputSymlink(path, description) {
  const info = await existingPath(path, description);
  if (info?.isSymbolicLink()) {
    throw new ValidationError(`${description} is a symlink: ${path}`);
  }
}

async function writeExtractions(root, extractDir, fences) {
  const { physicalExtractDir, physicalFencesDir } = await prepareExtractionDirectory(root, extractDir);

  const entries = [];
  for (let index = 0; index < fences.length; index += 1) {
    const fence = fences[index];
    const extracted = `fences/${String(index + 1).padStart(4, '0')}.${fence.kind}`;
    const extractedPath = resolve(physicalFencesDir, `${String(index + 1).padStart(4, '0')}.${fence.kind}`);
    if (!isWithin(physicalFencesDir, extractedPath)) {
      throw new ValidationError(`Extraction path escapes fences directory: ${extracted}`);
    }
    await rejectOutputSymlink(extractedPath, 'Extracted fence output');
    try {
      await writeFile(extractedPath, `${fence.body}\n`, 'utf8');
    } catch (error) {
      throw new ValidationError(`Cannot extract ${fileDescription(fence)}: ${error.message}`);
    }
    entries.push({
      kind: fence.kind,
      source: fence.source,
      line: fence.line,
      extracted,
    });
  }

  const manifestPath = resolve(physicalExtractDir, 'manifest.json');
  if (!isWithin(physicalExtractDir, manifestPath)) {
    throw new ValidationError('Extraction manifest path escapes extract directory');
  }
  await rejectOutputSymlink(manifestPath, 'Extraction manifest');
  try {
    await writeFile(manifestPath, `${JSON.stringify({ entries }, null, 2)}\n`, 'utf8');
  } catch (error) {
    throw new ValidationError(`Cannot write extraction manifest: ${error.message}`);
  }
}

async function validateFences(fences) {
  const { compileEbnf } = await import(RAILROAD_EBNF);
  const { render } = await import(SEQUENT_RENDERER);

  for (const fence of fences) {
    if (fence.body.trim() === '') {
      throw new ValidationError(`Malformed ${fence.kind} fence: ${fileDescription(fence)} is empty`);
    }
    if (fence.kind === 'ebnf') {
      try {
        preflightEbnf(fence.body);
      } catch (error) {
        throw new ValidationError(`EBNF preflight ${fileDescription(fence)}: ${error.message}`);
      }
      try {
        compileEbnf(fence.body);
      } catch (error) {
        throw new ValidationError(`EBNF compiler ${fileDescription(fence)}: ${error.message}`);
      }
    } else {
      let result;
      try {
        result = render(fence.body);
      } catch (error) {
        throw new ValidationError(`Sequent renderer ${fileDescription(fence)}: ${error.message}`);
      }
      if (!Array.isArray(result.diagnostics) || result.diagnostics.length > 0) {
        const details = Array.isArray(result.diagnostics)
          ? result.diagnostics.map((diagnostic) => diagnostic.message).join('; ')
          : 'renderer returned no diagnostics collection';
        throw new ValidationError(`Sequent diagnostic ${fileDescription(fence)}: ${details}`);
      }
    }
  }
}

async function main() {
  const { root, extractDir } = parseArguments(process.argv.slice(2));
  const markdownFiles = await collectMarkdownFiles(root);
  const fences = [];

  for (const path of markdownFiles) {
    const markdown = await readMarkdown(root, path);
    fences.push(...extractFences(displayPath(root, path), markdown));
  }

  const ebnfCount = fences.filter((fence) => fence.kind === 'ebnf').length;
  const sequentCount = fences.filter((fence) => fence.kind === 'sequent').length;
  if (markdownFiles.length > 0 && ebnfCount === 0) {
    throw new ValidationError('zero EBNF fences extracted from a nonempty manual');
  }
  if (markdownFiles.length > 0 && sequentCount === 0) {
    throw new ValidationError('zero sequent fences extracted from a nonempty manual');
  }

  await writeExtractions(root, extractDir, fences);
  await validateFences(fences);
  for (const fence of fences) console.log(fileDescription(fence));
}

main().catch((error) => {
  const message = error instanceof Error ? error.message : String(error);
  console.error(`language reference fence validation failed: ${message}`);
  process.exitCode = 1;
});
