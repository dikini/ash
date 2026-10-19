import sources from '../../sources.json';

const webUrl = (repository: string) => repository.replace(/\.git$/, '');
export default {
  languageRevision: sources.checkpoint.revision,
  reviewed: sources.checkpoint.reviewed ?? undefined,
  toolchain: sources.checkpoint.toolchain,
  repository: webUrl(sources.repositories[sources.checkpoint.source as keyof typeof sources.repositories].repository),
  referenceRepository: webUrl(sources.repositories[sources.reference.source as keyof typeof sources.repositories].repository),
  studyRepository: webUrl(sources.repositories[sources.study.source as keyof typeof sources.repositories].repository),
  studyRoute: sources.study.route,
  referenceRoutes: Object.fromEntries(sources.reference.groups.map(group => [group.id, group.route])) as Record<string, string>,
};
