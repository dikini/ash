import { readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';

const file = new URL('../dist/tokens.css', import.meta.url);
const sha256 = createHash('sha256').update(await readFile(file)).digest('hex');
await writeFile(new URL('../dist/design-tokens.json', import.meta.url),
  JSON.stringify({ version: 1, file: 'tokens.css', sha256 }, null, 2) + '\n');
