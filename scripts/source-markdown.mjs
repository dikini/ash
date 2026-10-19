import { readSourceConfig, repositoryWebUrl, validateSourceConfig, sha256 } from './source-config.mjs';
import { satteri } from '@astrojs/markdown-satteri';

export function expandSourceReferences(text, config = readSourceConfig().config) {
  const values = {
    'checkpoint.revision': config.checkpoint.revision,
    'checkpoint.short': config.checkpoint.revision.slice(0, 7),
    'checkpoint.toolchain': config.checkpoint.toolchain,
    'checkpoint.reviewState': config.checkpoint.reviewed ? 'a reviewed snapshot' : 'a snapshot awaiting review',
    'checkpoint.reviewed': config.checkpoint.reviewed
      ? new Date(`${config.checkpoint.reviewed}T12:00:00Z`).toLocaleDateString('en-GB', { day: 'numeric', month: 'long', year: 'numeric', timeZone: 'UTC' }) : 'Pending review',
  };
  for (const [id, source] of Object.entries(config.repositories)) {
    values[`source.${id}`] = repositoryWebUrl(source.repository);
    values[`repository.${id}`] = source.repository;
  }
  for (const group of config.reference.groups) values[`route.${group.id}`] = group.route;
  return text.replace(/\{\{([a-zA-Z0-9_.-]+)\}\}/g, (_, name) => {
    if (!(name in values)) throw new Error(`Unknown source reference: ${name}`);
    return values[name];
  });
}

export default function sourceMarkdown({ config } = {}) {
  const processor = satteri();
  const sourceConfig = validateSourceConfig(config ?? readSourceConfig().config);
  return {
    // Astro hashes processor options to invalidate its content store.
    name: 'source-references', options: { ...processor.options, sourceConfigHash: sha256(JSON.stringify(sourceConfig)) },
    async createRenderer(shared) {
      const renderer = await processor.createRenderer(shared);
      return { async render(content, options) { return renderer.render(expandSourceReferences(content, sourceConfig), options); } };
    },
  };
}
