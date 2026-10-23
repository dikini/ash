# Ash site

Static Astro site for the Ash programming language. Sources for the spec,
design notes, study book, and decision records are fetched from pinned upstream
repositories at build time; read `docs/source-management.md` before changing
how content is sourced.

## Commands

- `npm run check` — token check plus Astro diagnostics.
- `npm run build` — full build: token sync, upstream preparation, Astro,
  Pagefind, link check. Takes about a minute.
- `npm test` — token sync tests plus the content test suite.

## Working in this repository

- **Site copy** (home, discover, learn entries, status, updates, diaries,
  contribute, interface text): load the `site-voice` skill, or read
  `docs/site-voice.md`, before writing or editing. The specification, docs
  publications, and study book keep their own voices — do not apply site
  voice there.
- **Visual and layout work**: follow `docs/design-system.md` and load the
  `frontend-design` skill. Design tokens live in `public/tokens.css`; edit
  that file and run `npm run tokens:sync` to propagate — never edit
  `.design-tokens-base.css` directly; synchronization rewrites it.
- Never hand-edit `dist/` or fetched upstream sources; extend the
  preparation scripts instead.
- Verify with the commands above before committing; `git diff --check` for
  whitespace.

- **Navigation changes**: follow the location/context rationale in
  `docs/design-system.md`. The outline leaf marks the exact current item;
  ancestors use stronger neutral text. Keep both outline and solid leaf assets;
  the solid variant is reserved for deliberate stronger-emphasis contexts.
  Reuse the shared navigation treatment and preserve keyboard focus and native
  study disclosure behavior rather than inventing another selection marker.
