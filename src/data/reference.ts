import { readFile } from 'node:fs/promises';

export interface ReferencePage {
  file: string; route: string; title: string; titleId: string; html: string;
  headings: { depth: number; slug: string; text: string }[];
}
export interface ReferenceGroup { directory: string; pages: ReferencePage[]; }
interface ReferencePublication extends ReferenceGroup { revision: string; tools: ReferenceGroup; }

export async function loadReferencePublication(): Promise<ReferencePublication> {
  try {
    // Astro relocates shared modules into build chunks; read from the project
    // working directory rather than resolving against the bundled module URL.
    const publication: ReferencePublication = JSON.parse(await readFile('.cache/ash-spec/publication.json', 'utf8'));
    if (!publication.tools?.pages.length) throw new Error('Missing tool documentation');
    return publication;
  } catch (cause) {
    throw new Error('Prepare the specification and tool documentation with npm run prepare:spec before running Astro.', { cause });
  }
}
