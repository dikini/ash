import { defineConfig } from 'astro/config';
import sourceMarkdown from './scripts/source-markdown.mjs';
import sources from './sources.json' with { type: 'json' };
import { normalizeOrigin } from './scripts/discovery.mjs';

const site = normalizeOrigin(process.env.SITE_URL);

export default defineConfig({
  output: 'static',
  trailingSlash: 'always',
  ...(site ? { site } : {}),
  markdown: { processor: sourceMarkdown({ config: sources }), shikiConfig: { theme: 'github-dark' } },
});
