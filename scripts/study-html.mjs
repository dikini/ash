import { parseHTML } from 'linkedom';
import { readSourceConfig, repositoryWebUrl } from './source-config.mjs';

const escape = value => String(value).replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;');

export function integrateStudyHtml(html, snapshot) {
  const { config } = readSourceConfig();
  const ashRepository = repositoryWebUrl(config.repositories[config.checkpoint.source].repository);
  const docsRepository = repositoryWebUrl(snapshot.repository ?? config.repositories[config.study.source].repository);
  const { document } = parseHTML(html);
  if (!document.querySelector('main#quarto-document-content')) throw new Error('Upstream HTML has no supported chapter body');
  const banner = `<a class="ash-skip-link" href="#quarto-document-content">Skip to content</a><header class="ash-study-header" data-pagefind-ignore><a class="ash-study-brand" href="/" aria-label="Ash home"><img class="brand-icon" src="/favicon.svg?v=ash-tree" width="32" height="32" alt="" aria-hidden="true"><span>ash<span class="brand-dot">·</span></span></a><nav aria-label="Ash website"><a href="/learn/">Learn</a><a href="/docs/">Documentation</a><a href="/study/">Study overview</a><a href="/">Back to Ash ↗</a></nav></header><aside class="ash-study-banner" aria-label="Documentation snapshot" data-pagefind-ignore>Development snapshot · ${escape(snapshot.milestone)} · Ash <a href="${ashRepository}/tree/${escape(snapshot.ashRevision)}">${escape(snapshot.ashRevision.slice(0, 7))}</a> · Docs <a href="${docsRepository}/tree/${escape(snapshot.sourceRevision)}">${escape(snapshot.sourceRevision.slice(0, 7))}</a>. Not a published book edition.</aside>`;
  // Keep upstream source bytes intact, including code, diagnostics, and relative URLs.
  const integrated = html.replace('</head>', '<link rel="stylesheet" href="/theme.css"><link rel="stylesheet" href="/study-chrome.css"><link rel="stylesheet" href="/navigation.css"><script type="module" src="/navigation.js"></script>\n</head>')
    .replace(/<body\b[^>]*>/, tag => `${tag}\n${banner}`)
    .replace(/<main\b[^>]*\bid="quarto-document-content"[^>]*>/, tag => tag.replace('<main', '<main tabindex="-1" data-pagefind-body'));
  // Attribute-only additions preserve upstream example text and verification bytes.
  const accessible = integrated.replace(/<pre\b([^>]*)>/g, (tag, attributes) => /\btabindex\s*=/.test(attributes) ? tag : `<pre tabindex="0" aria-label="Code or transcript"${attributes}>`);
  const navigable = accessible.replace(/<a\b[^>]*>/g, tag => {
    const classes = tag.match(/\bclass="([^"]*)"/)?.[1].split(/\s+/) ?? [];
    return classes.includes('sidebar-link') && classes.includes('active') && !/\baria-current=/.test(tag)
      ? tag.replace(/>$/, ' aria-current="page">') : tag;
  });
  const result = parseHTML(navigable).document;
  if (!result.querySelector('main#quarto-document-content[data-pagefind-body]') || !result.querySelector('.ash-study-header') || !result.querySelector('.ash-study-banner') || !result.querySelector('link[href="/study-chrome.css"]') || !result.querySelector('link[href="/theme.css"]') || !result.querySelector('link[href="/navigation.css"]') || !result.querySelector('script[src="/navigation.js"]')) {
    throw new Error('Unsupported upstream HTML format: required study integration was not applied');
  }
  return navigable;
}
