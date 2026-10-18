import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const args = process.argv.slice(2);
const check = args.includes('--check');
const paths = args.filter(arg => arg !== '--check');
if (paths.length > 1 || paths.some(arg => arg.startsWith('--'))) {
  throw new Error('Usage: node scripts/sync-design-tokens.mjs [--check] [/path/to/ash-doc]');
}
const result = spawnSync('python3', [fileURLToPath(new URL('./token_sync.py', import.meta.url)),
  ...(check ? ['--check'] : []), ...(paths.length ? ['--peer', paths[0]] : [])], { stdio: 'inherit' });
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
