# Shared design tokens implementation plan

**Goal:** Formalize the approved Ash visual language and synchronize HTML presentation with ash-doc.

**Architecture:** Ash-site owns a portable CSS custom-property contract. Website and hosted book load it through theme.css; standalone ash-doc vendors the same file through an explicit sync command. Existing short token names remain aliases. Book typography and print formats retain their reading roles.

**Tech stack:** CSS custom properties, Astro, Node, Quarto.

1. Add public/tokens.css with semantic colors, type, spacing, layout, borders, radii, shadow, and motion; make theme.css import it.
2. Replace repeated site values with tokens without changing the approved appearance. Apply semantic tokens to hosted book chrome and reading colors.
3. Document component rules, accessibility, ownership, and sync commands in docs/design-system.md; link existing integration documentation.
4. In an isolated ash-doc worktree, vendor tokens.css, load it in HTML, and document the contract. Preserve serif reading typography and PDF/EPUB output.
5. Verify synchronization, CSS variable resolution, site check/build/tests, and representative ash-doc HTML rendering. Inspect diffs and report local worktree state.
