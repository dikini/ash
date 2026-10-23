// Shared progressive enhancement for site and hosted-book contents navigation.
export function collectEntries(nav, document) {
  return [...nav.querySelectorAll('a[href]')].flatMap(link => {
    let id;
    try {
      let fragment = link.getAttribute('href');
      if (!fragment.startsWith('#')) {
        const base = new URL(document.baseURI);
        const destination = new URL(fragment, base);
        if (destination.origin !== base.origin || destination.pathname !== base.pathname || destination.search !== base.search) return [];
        fragment = destination.hash;
      }
      id = decodeURIComponent(fragment.slice(1));
    }
    catch { return []; }
    const target = document.getElementById(id);
    const heading = target && (/^H[1-6]$/.test(target.tagName) ? target
      : target.matches('section') ? target.querySelector(':scope > h1, :scope > h2, :scope > h3, :scope > h4, :scope > h5, :scope > h6') : null);
    return heading && /^H[1-6]$/.test(heading.tagName)
      ? [{ link, heading, target, id, depth: Number(heading.tagName.slice(1)) }] : [];
  });
}

export function chooseEntry(entries, threshold = 96, atEnd = false) {
  if (atEnd) return entries.at(-1);
  let current = entries[0];
  for (const entry of entries) {
    if (entry.target.getBoundingClientRect().top <= threshold) current = entry;
  }
  return current;
}

export function markCurrent(nav, entries, current) {
  for (const link of nav.querySelectorAll('[aria-current="location"], [data-current-ancestor]')) {
    link.removeAttribute('aria-current');
    link.removeAttribute('data-current-ancestor');
  }
  if (!current) return;
  const branch = [];
  for (const entry of entries) {
    while (branch.length && branch.at(-1).depth >= entry.depth) branch.pop();
    if (entry === current) break;
    branch.push(entry);
  }
  for (const ancestor of branch) ancestor.link.setAttribute('data-current-ancestor', '');
  current.link.setAttribute('aria-current', 'location');
}

const initializedContents = new WeakSet();
function initializeContents() {
  for (const nav of document.querySelectorAll('.ash-contents, #TOC, details.on-this-page')) {
    if (initializedContents.has(nav)) continue;
    const entries = collectEntries(nav, document);
    if (!entries.length) continue;
    initializedContents.add(nav);
    if (nav.matches('details.on-this-page')) {
      const baseDepth = Math.min(...entries.map(entry => entry.depth));
      for (const entry of entries) entry.link.style.setProperty('--ash-nav-depth', String(entry.depth - baseDepth));
    }
    let current;
    const select = entry => {
      if (entry === current) return;
      current = entry;
      markCurrent(nav, entries, entry);
      // Keep the location visible inside long contents lists without moving
      // the document or keyboard focus (especially when the mobile list flows).
      if (entry && nav.scrollHeight > nav.clientHeight) {
        const frame = nav.getBoundingClientRect();
        const row = entry.link.getBoundingClientRect();
        if (row.top < frame.top) nav.scrollTop += row.top - frame.top;
        else if (row.bottom > frame.bottom) nav.scrollTop += row.bottom - frame.bottom;
      }
    };
    const fromHash = () => {
      let id;
      try { id = decodeURIComponent(location.hash.slice(1)); } catch { return; }
      const entry = entries.find(entry => entry.id === id);
      if (entry) select(entry);
    };
    let pending = false;
    const update = () => {
      pending = false;
      const atEnd = window.scrollY > 0 && window.scrollY + window.innerHeight >= document.documentElement.scrollHeight - 2;
      select(chooseEntry(entries, 96, atEnd));
    };
    const schedule = () => {
      if (!pending) { pending = true; window.requestAnimationFrame(update); }
    };
    update();
    fromHash();
    window.addEventListener('scroll', schedule, { passive: true });
    window.addEventListener('resize', schedule);
    window.addEventListener('load', schedule, { once: true });
    window.addEventListener('hashchange', fromHash);
    window.addEventListener('popstate', schedule);
  }
}

if (typeof window !== 'undefined') {
  // Upstream builds the tablet/mobile disclosure in a DOMContentLoaded
  // handler. Initialize again after all of those synchronous handlers run.
  // The WeakSet keeps listeners unique for already initialized contents.
  if (document.readyState !== 'complete') {
    document.addEventListener('DOMContentLoaded', () => queueMicrotask(initializeContents), { once: true });
    window.addEventListener('load', initializeContents, { once: true });
  }
  initializeContents();
}
