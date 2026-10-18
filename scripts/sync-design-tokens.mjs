import { readFile, writeFile, stat } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const args = process.argv.slice(2);
const check = args.includes('--check');
const paths = args.filter(arg => arg !== '--check');
if (paths.length !== 1 || paths[0].startsWith('--')) {
  throw new Error('Usage: node scripts/sync-design-tokens.mjs [--check] /path/to/ash-doc');
}
const target = resolve(paths[0]);
await stat(join(target, 'rendering/styles/html.scss'));
const source = await readFile(fileURLToPath(new URL('../public/tokens.css', import.meta.url)), 'utf8');
const destination = join(target, 'rendering/styles/ash-tokens.css');
if (check) {
  if (await readFile(destination, 'utf8') !== source) throw new Error('ash-doc design tokens have drifted; run the sync command');
  console.log('ash-doc design tokens match');
} else {
  await writeFile(destination, source);
  console.log(`Synchronized ${destination}`);
}
