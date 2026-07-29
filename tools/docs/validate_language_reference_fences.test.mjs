import assert from 'node:assert/strict';
import { mkdtemp, chmod, lstat, readFile, rm, symlink, writeFile, mkdir } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import test from 'node:test';

const repositoryRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..');
const validator = join(repositoryRoot, 'tools/docs/validate_language_reference_fences.mjs');

async function withFixture(run) {
  const fixture = await mkdtemp(join(tmpdir(), 'ash-language-reference-fences-'));
  const manual = join(fixture, 'docs/reference/language');
  const extractDir = join(fixture, 'extracted');
  await mkdir(manual, { recursive: true });
  try {
    await run({ fixture, manual, extractDir });
  } finally {
    await chmodTreeReadable(fixture);
    await rm(fixture, { recursive: true, force: true });
  }
}

async function chmodTreeReadable(path) {
  const { readdir, stat } = await import('node:fs/promises');
  let entries;
  try {
    entries = await readdir(path);
  } catch {
    return;
  }
  for (const entry of entries) {
    const child = join(path, entry);
    const info = await stat(child);
    if (info.isDirectory()) await chmodTreeReadable(child);
    else await chmod(child, 0o600);
  }
}

async function writeFixture(root, path, content) {
  const destination = join(root, path);
  await mkdir(dirname(destination), { recursive: true });
  await writeFile(destination, content, 'utf8');
  return destination;
}

function runValidator(manual, extractDir) {
  const fixture = resolve(manual, '..', '..', '..');
  return runValidatorArguments([
    '--root', 'docs/reference/language',
    '--extract-dir', extractDir,
  ], fixture);
}

function runValidatorArguments(arguments_, cwd) {
  return spawnSync(process.execPath, [validator, ...arguments_], { cwd, encoding: 'utf8' });
}

function commandOutput(result) {
  return `${result.stdout}\n${result.stderr}`;
}

function assertRejected(result, expected) {
  assert.notEqual(result.status, 0, commandOutput(result));
  assert.match(commandOutput(result), expected, commandOutput(result));
}

test('recurses only under the selected manual, validates both fence kinds, and records extraction provenance', async () => {
  await withFixture(async ({ fixture, manual, extractDir }) => {
    const index = await writeFixture(manual, 'index.md', [
      '# Fixture manual',
      '',
      '```ebnf',
      'entry = "entry" ;',
      '```',
      '',
      '```rust',
      'this is deliberately not a checked fence',
      '```',
      '',
    ].join('\n'));
    const semantics = await writeFixture(manual, 'nested/semantics.md', [
      '# Fixture semantics',
      '',
      '```sequent',
      'Identity :=',
      '  =>',
      '  GAMMA |- A',
      '```',
      '',
    ].join('\n'));
    await writeFixture(fixture, 'outside-manual.md', [
      '```ebnf',
      'outside ::= invalid ;',
      '```',
      '',
    ].join('\n'));

    const before = new Map([
      [index, await readFile(index, 'utf8')],
      [semantics, await readFile(semantics, 'utf8')],
    ]);
    const result = runValidator(manual, extractDir);

    assert.equal(result.status, 0, commandOutput(result));
    assert.match(result.stdout, /index\.md:3.*\[ebnf\]/i, commandOutput(result));
    assert.match(result.stdout, /nested\/semantics\.md:3.*\[sequent\]/i, commandOutput(result));

    const manifestPath = join(extractDir, 'manifest.json');
    const manifest = JSON.parse(await readFile(manifestPath, 'utf8'));
    assert.deepEqual(
      manifest.entries.map(({ kind, source, line }) => ({ kind, source, line })),
      [
        { kind: 'ebnf', source: 'index.md', line: 3 },
        { kind: 'sequent', source: 'nested/semantics.md', line: 3 },
      ],
    );
    for (const entry of manifest.entries) {
      const extractedPath = resolve(extractDir, entry.extracted);
      assert.equal(relative(extractDir, extractedPath).startsWith('..'), false, entry.extracted);
      const source = await readFile(extractedPath, 'utf8');
      assert.notEqual(source.trim(), '', entry.extracted);
    }
    for (const [path, expected] of before) {
      assert.equal(await readFile(path, 'utf8'), expected, path);
    }
  });
});

test('accepts target fences indented by up to three spaces', async () => {
  await withFixture(async ({ manual, extractDir }) => {
    await writeFixture(manual, 'indented.md', [
      '# Indented fences',
      '',
      '   ```ebnf',
      'entry = "entry" ;',
      '   ```',
      '',
      '  ```sequent',
      'Identity :=',
      '  =>',
      '  GAMMA |- A',
      '  ```',
      '',
    ].join('\n'));

    const result = runValidator(manual, extractDir);
    assert.equal(result.status, 0, commandOutput(result));
    const manifest = JSON.parse(await readFile(join(extractDir, 'manifest.json'), 'utf8'));
    assert.deepEqual(
      manifest.entries.map(({ kind, source, line }) => ({ kind, source, line })),
      [
        { kind: 'ebnf', source: 'indented.md', line: 3 },
        { kind: 'sequent', source: 'indented.md', line: 7 },
      ],
    );
  });
});

test('accepts target fences when opening and closing indentation differ', async () => {
  await withFixture(async ({ manual, extractDir }) => {
    await writeFixture(manual, 'asymmetric-indentation.md', [
      '# Asymmetric indentation',
      '',
      '   ```ebnf',
      'entry = "entry" ;',
      '```',
      '',
      '```sequent',
      'Identity :=',
      '  =>',
      '  GAMMA |- A',
      '   ```',
      '',
    ].join('\n'));

    const result = runValidator(manual, extractDir);
    assert.equal(result.status, 0, commandOutput(result));
    const manifest = JSON.parse(await readFile(join(extractDir, 'manifest.json'), 'utf8'));
    assert.deepEqual(
      manifest.entries.map(({ kind, source, line }) => ({ kind, source, line })),
      [
        { kind: 'ebnf', source: 'asymmetric-indentation.md', line: 3 },
        { kind: 'sequent', source: 'asymmetric-indentation.md', line: 7 },
      ],
    );
  });
});

test('rejects an extraction directory symlinked to the manual without writing manual files', async (context) => {
  await withFixture(async ({ manual, extractDir }) => {
    const markdown = await writeFixture(manual, 'index.md', '```ebnf\nentry = "entry" ;\n```\n\n```sequent\nIdentity :=\n  =>\n  GAMMA |- A\n```\n');
    const before = await readFile(markdown, 'utf8');
    try {
      await symlink(manual, extractDir, 'dir');
    } catch (error) {
      if (['EACCES', 'EPERM', 'ENOTSUP'].includes(error.code)) {
        context.skip(`symlinks are unavailable: ${error.code}`);
        return;
      }
      throw error;
    }

    assertRejected(runValidator(manual, extractDir), /extract.*symlink|symlink.*extract/i);
    assert.equal(await readFile(markdown, 'utf8'), before);
    await assert.rejects(lstat(join(manual, 'manifest.json')), { code: 'ENOENT' });
    await assert.rejects(lstat(join(manual, 'fences')), { code: 'ENOENT' });
  });
});

test('rejects a pre-existing fences symlink in the extraction directory without writing through it', async (context) => {
  await withFixture(async ({ manual, extractDir }) => {
    const markdown = await writeFixture(manual, 'index.md', '```ebnf\nentry = "entry" ;\n```\n\n```sequent\nIdentity :=\n  =>\n  GAMMA |- A\n```\n');
    const before = await readFile(markdown, 'utf8');
    await mkdir(extractDir, { recursive: true });
    try {
      await symlink(manual, join(extractDir, 'fences'), 'dir');
    } catch (error) {
      if (['EACCES', 'EPERM', 'ENOTSUP'].includes(error.code)) {
        context.skip(`symlinks are unavailable: ${error.code}`);
        return;
      }
      throw error;
    }

    assertRejected(runValidator(manual, extractDir), /fences.*symlink|symlink.*fences/i);
    assert.equal(await readFile(markdown, 'utf8'), before);
    await assert.rejects(lstat(join(manual, '0001.ebnf')), { code: 'ENOENT' });
  });
});

test('rejects a root outside or lexically parent of docs/reference/language', async () => {
  await withFixture(async ({ fixture, manual, extractDir }) => {
    await writeFixture(manual, 'index.md', '```ebnf\nentry = "entry" ;\n```\n\n```sequent\nIdentity :=\n  =>\n  GAMMA |- A\n```\n');
    await writeFixture(fixture, 'outside/index.md', '```ebnf\noutside = "outside" ;\n```\n\n```sequent\nIdentity :=\n  =>\n  GAMMA |- A\n```\n');
    for (const root of ['docs/reference', 'outside']) {
      const result = runValidatorArguments([
        '--root', root,
        '--extract-dir', extractDir,
      ], fixture);
      assertRejected(result, /manual root.*docs\/reference\/language|--root.*docs\/reference\/language/i);
    }
  });
});

for (const [name, ebnf, sequent, expected] of [
  ['unterminated EBNF fence', '```ebnf\nentry = "entry" ;\n```\n\n   ```ebnf\nunterminated = "entry" ;\n', '```sequent\nIdentity :=\n  =>\n  GAMMA |- A\n```\n', /no closing fence/i],
  ['unterminated sequent fence', '```ebnf\nentry = "entry" ;\n```\n', '```sequent\nIdentity :=\n  =>\n  GAMMA |- A\n```\n\n  ```sequent\nBroken :=\n  =>\n  GAMMA |- B\n', /no closing fence/i],
  ['empty EBNF fence', '```ebnf\nentry = "entry" ;\n```\n\n   ```ebnf\n   ```\n', '```sequent\nIdentity :=\n  =>\n  GAMMA |- A\n```\n', /Malformed ebnf fence.*empty/i],
  ['empty sequent fence', '```ebnf\nentry = "entry" ;\n```\n', '```sequent\nIdentity :=\n  =>\n  GAMMA |- A\n```\n\n   ```sequent\n   ```\n', /Malformed sequent fence.*empty/i],
]) {
  test(`rejects ${name}`, async () => {
    await withFixture(async ({ manual, extractDir }) => {
      await writeFixture(manual, 'grammar.md', ebnf);
      await writeFixture(manual, 'semantics.md', sequent);

      assertRejected(runValidator(manual, extractDir), expected);
    });
  });
}

for (const [name, arguments_, expected] of [
  ['a missing extract directory', ['--root', 'docs/reference/language'], /Usage:|missing/i],
  ['a missing root', ['--extract-dir', 'extract'], /Usage:|missing/i],
  ['a duplicate root', ['--root', 'docs/reference/language', '--root', 'docs/reference/language', '--extract-dir', 'extract'], /duplicate/i],
  ['an unknown argument', ['--root', 'docs/reference/language', '--extract-dir', 'extract', '--unknown'], /Unknown argument/i],
]) {
  test(`rejects ${name}`, async () => {
    await withFixture(async ({ fixture }) => {
      assertRejected(runValidatorArguments(arguments_, fixture), expected);
    });
  });
}

for (const [name, grammar] of [
  ['missing equals sign', 'entry "entry" ;'],
  ['missing terminal semicolon', 'entry = "entry"'],
  ['unquoted terminal punctuation', 'entry = : ;'],
  ['forbidden ::= assignment', 'entry ::= "entry" ;'],
]) {
  test(`rejects EBNF preflight: ${name}`, async () => {
    await withFixture(async ({ manual, extractDir }) => {
      await writeFixture(manual, 'grammar.md', `\`\`\`ebnf\n${grammar}\n\`\`\`\n`);
      await writeFixture(manual, 'semantics.md', '```sequent\nIdentity :=\n  =>\n  GAMMA |- A\n```\n');

      assertRejected(runValidator(manual, extractDir), /EBNF preflight/i);
    });
  });
}

test('reports a railroad compiler error after EBNF preflight succeeds', async () => {
  await withFixture(async ({ manual, extractDir }) => {
    await writeFixture(manual, 'grammar.md', '```ebnf\nentry = ( "entry" ;\n```\n');
    await writeFixture(manual, 'semantics.md', '```sequent\nIdentity :=\n  =>\n  GAMMA |- A\n```\n');

    assertRejected(runValidator(manual, extractDir), /EBNF compiler/i);
  });
});

test('rejects sequent fences when the renderer returns diagnostics', async () => {
  await withFixture(async ({ manual, extractDir }) => {
    await writeFixture(manual, 'grammar.md', '```ebnf\nentry = "entry" ;\n```\n');
    await writeFixture(manual, 'semantics.md', '```sequent\nBroken :=\n  no deduction operator\n```\n');

    assertRejected(runValidator(manual, extractDir), /sequent.*diagnostic/i);
  });
});

test('rejects an unreadable Markdown file under the selected manual', async () => {
  await withFixture(async ({ manual, extractDir }) => {
    await writeFixture(manual, 'grammar.md', '```ebnf\nentry = "entry" ;\n```\n');
    await writeFixture(manual, 'semantics.md', '```sequent\nIdentity :=\n  =>\n  GAMMA |- A\n```\n');
    const unreadable = await writeFixture(manual, 'unreadable.md', '# cannot read\n');
    await chmod(unreadable, 0o000);

    assertRejected(runValidator(manual, extractDir), /unreadable/i);
  });
});

for (const [name, content, expectedKind] of [
  ['EBNF', '```sequent\nIdentity :=\n  =>\n  GAMMA |- A\n```\n', 'EBNF'],
  ['sequent', '```ebnf\nentry = "entry" ;\n```\n', 'sequent'],
]) {
  test(`rejects a nonempty manual with zero ${name} fences`, async () => {
    await withFixture(async ({ manual, extractDir }) => {
      await writeFixture(manual, 'index.md', `# Fixture manual\n\n${content}`);

      assertRejected(runValidator(manual, extractDir), new RegExp(`zero ${expectedKind} fences`, 'i'));
    });
  });
}
