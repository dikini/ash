import { defineConfig } from 'astro/config';
import sourceMarkdown from './scripts/source-markdown.mjs';
import sources from './sources.json' with { type: 'json' };

export default defineConfig({
  output: 'static',
  trailingSlash: 'always',
  ...(process.env.SITE_URL ? { site: process.env.SITE_URL } : {}),
  markdown: { processor: sourceMarkdown({ config: sources }), shikiConfig: { theme: 'github-dark' } },
});
