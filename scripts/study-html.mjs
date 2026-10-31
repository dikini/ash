import { parseHTML } from 'linkedom';
import { readSourceConfig, repositoryWebUrl } from './source-config.mjs';

const escape = value => String(value).replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;');

export function integrateStudyHtml(html, snapshot) {
  const { config } = readSourceConfig();
  const ashRepository = repositoryWebUrl(config.repositories[config.checkpoint.source].repository);
  const docsRepository = repositoryWebUrl(snapshot.repository ?? config.repositories[config.study.source].repository);
  const { document } = parseHTML(html);
  if (!document.querySelector('main#quarto-document-content')) throw new Error('Upstream HTML has no supported chapter body');
  const tree = `<svg class="ash-tree-mark brand-icon" width="32" height="32" viewBox="0 0 64 64" aria-hidden="true"><g fill="currentColor"><path d="M28 7C21 12 22 19 28 23C34 19 35 12 28 7Z"/><path d="M10 15C20 14 25 20 24 29C15 28 10 23 10 15Z"/><path d="M46 15C36 14 31 20 32 29C41 28 46 23 46 15Z"/><path d="M8 30C17 28 25 33 25 42C15 43 9 38 8 30Z"/><path d="M48 30C39 28 31 33 31 42C41 43 47 38 48 30Z"/><path d="M22 53C25 52 26 49 26 45H30C30 49 31 52 34 53Z"/></g><path d="M28 49V21M28 33L18 23M28 33L38 23M28 44L16 35M28 44L40 35" fill="none" stroke="currentColor" stroke-width="3.2" stroke-linecap="round" stroke-linejoin="round"/></svg>`;
  const nav = [['Discover', '/discover/'], ['Learn', '/learn/'], ['Docs', '/docs/'], ['Status', '/status/'], ['Updates', '/updates/']];
  const links = nav.map(([label, href]) => `<a href="${href}">${label}</a>`).join('');
  const header = `<a class="ash-skip-link" href="#quarto-document-content">Skip to content</a><header class="ash-study-header" data-pagefind-ignore><a class="ash-study-brand wordmark" href="/" aria-label="Ash home">${tree}<span>ash</span></a><nav class="desktop-nav" aria-label="Main navigation">${links}</nav><div class="header-tools"><a class="search-link" href="/search/"><svg aria-hidden="true" width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><circle cx="10.5" cy="10.5" r="6.5"/><path d="m16 16 5 5"/></svg><span>Search</span></a><a class="source-link" href="${ashRepository}">Source <img src="/github.svg" width="16" height="16" alt="" aria-hidden="true"></a></div><details class="mobile-nav"><summary role="button" aria-label="Menu"><svg class="menu-icon" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><path class="menu-line menu-line-top" d="M4 7h16"/><path class="menu-line menu-line-mid" d="M4 12h16"/><path class="menu-line menu-line-bottom" d="M4 17h16"/></svg></summary><nav class="ash-page-menu" aria-label="Mobile navigation">${links}<a href="/study/" aria-current="true">Study Ash</a><a href="/contribute/">Contribute</a></nav></details></header>`;
  const banner = `<aside class="ash-study-banner" aria-label="Documentation snapshot" data-pagefind-ignore>Development snapshot · ${escape(snapshot.milestone)} · Ash <a href="${ashRepository}/tree/${escape(snapshot.ashRevision)}">${escape(snapshot.ashRevision.slice(0, 7))}</a> · Docs <a href="${docsRepository}/tree/${escape(snapshot.sourceRevision)}">${escape(snapshot.sourceRevision.slice(0, 7))}</a>. Not a published book edition.</aside>`;
  const footer = `<footer class="ash-study-footer" data-pagefind-ignore><div class="footer-inner"><div><a class="footer-brand" href="/" aria-label="Ash home">${tree}<span>ash</span></a><p>Typed functional programming.<br>Parametric effects and scoped resources.</p></div><div class="footer-links"><a href="/diaries/">Developer diaries</a><a href="/study/">Study Ash</a><a href="/contribute/">Contribute</a><a href="${ashRepository}">GitHub</a><a href="/status/">Project status</a></div><p class="footer-note">Implementation, documentation,<br>and open language research.</p>${tree.replace('ash-tree-mark brand-icon', 'ash-tree-mark footer-mark')}</div></footer>`;
  // Keep upstream source bytes intact, including code, diagnostics, and relative URLs.
  const integrated = html.replace('</head>', '<link rel="stylesheet" href="/theme.css"><link rel="stylesheet" href="/study-chrome.css"><link rel="stylesheet" href="/navigation.css"><script type="module" src="/navigation.js"></script>\n</head>')
    .replace(/<body\b[^>]*>/, tag => `${tag}\n${header}${banner}`)
    .replace('</body>', `${footer}\n</body>`)
    .replace(/<main\b[^>]*\bid="quarto-document-content"[^>]*>/, tag => tag.replace('<main', '<main tabindex="-1" data-pagefind-body'));
  // Attribute-only additions preserve upstream example text and verification bytes.
  const accessible = integrated.replace(/<pre\b([^>]*)>/g, (tag, attributes) => /\btabindex\s*=/.test(attributes) ? tag : `<pre tabindex="0" aria-label="Code or transcript"${attributes}>`);
  const navigable = accessible.replace(/<a\b[^>]*>/g, tag => {
    const classes = tag.match(/\bclass="([^"]*)"/)?.[1].split(/\s+/) ?? [];
    return classes.includes('sidebar-link') && classes.includes('active') && !/\baria-current=/.test(tag)
      ? tag.replace(/>$/, ' aria-current="page">') : tag;
  });
  const result = parseHTML(navigable).document;
  if (!result.querySelector('main#quarto-document-content[data-pagefind-body]') || !result.querySelector('.ash-study-header .ash-tree-mark') || !result.querySelector('.ash-study-header nav[aria-label="Main navigation"]') || !result.querySelector('.ash-study-header details.mobile-nav') || !result.querySelector('.ash-study-footer') || !result.querySelector('.ash-study-banner') || !result.querySelector('link[href="/study-chrome.css"]') || !result.querySelector('link[href="/theme.css"]') || !result.querySelector('link[href="/navigation.css"]') || !result.querySelector('script[src="/navigation.js"]')) {
    throw new Error('Unsupported upstream HTML format: required study integration was not applied');
  }
  return navigable;
}
