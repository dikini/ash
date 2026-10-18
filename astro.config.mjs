import { defineConfig } from 'astro/config';

export default defineConfig({
  output: 'static',
  trailingSlash: 'always',
  ...(process.env.SITE_URL ? { site: process.env.SITE_URL } : {}),
  markdown: { shikiConfig: { theme: 'github-dark' } },
});
