import { readdir, readFile } from 'node:fs/promises';
import { join, relative, sep } from 'node:path';
import { sha256 } from './source-config.mjs';

export async function artifactFiles(directory) {
  const files = [];
  for (const item of await readdir(directory, { withFileTypes: true })) {
    const file = join(directory, item.name);
    if (item.isDirectory()) files.push(...await artifactFiles(file));
    else files.push(file);
  }
  return files;
}

export async function hashArtifact(directory) {
  const records = [];
  for (const file of await artifactFiles(directory)) {
    const path = relative(directory, file).split(sep).join('/');
    if (path !== 'refresh-report.json') records.push([path, sha256(await readFile(file))]);
  }
  records.sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0);
  return sha256(JSON.stringify(records));
}
