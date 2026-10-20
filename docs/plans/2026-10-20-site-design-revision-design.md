# Site design revision: restrained depth within the fixed identity

Date: 2026-10-20
Status: approved by project owner; ready for implementation planning
Branch: `design/site-revision` (worktree `.worktrees/design-revision`)

## Decision record

Owner decisions that bound this revision:

1. Polish within the fixed identity. Palette tokens and bundled fonts stay
   exactly as documented in `docs/design-system.md`. No token changes, no
   font changes, no cross-repo token sync.
2. Scope: homepage, shared chrome (header/footer), and the article template.
   Spec/docs/study renderers are untouched.
3. Restrained depth: layered paper tones within existing surface tokens, the
   tree mark as a quiet recurring motif, one orchestrated homepage entrance.
   No scroll-linked animation, no new decoration vocabulary beyond this.

Alternatives considered and rejected: editorial-instrument direction
(too cold, loses the identity's warmth) and narrative-scroll direction
(violates the documented single-entrance-animation budget).

## Problem statement

The current execution is faithful to the documented system but flat below
the hero: sections are separated only by hairlines on bare paper, the hero
gradient reads as a smudge, the brand mark appears only in chrome, and the
article aside has no visual anchor. The revision fixes execution quality
without altering the documented identity.

## Changes

### Chrome

- Header unchanged in structure: hairline rule, sentence-case nav,
  current-page underline. Minor spacing refinement for the wordmark lockup
  on mobile.
- Footer becomes a full-bleed ink band (`--color-text-primary` background,
  paper text, rust hover accents) — the ink-on-paper reversal the primary
  button already establishes. Tree mark appears large, right-aligned, at
  very low opacity as a decorative watermark (`aria-hidden`). Footer content
  and links unchanged.

### Homepage hero

- Replace the corner-gradient smudge behind the code panel with an
  intentional spotlight: a soft radial wash in `--color-surface-subtle`
  centered behind the panel, keeping the existing border; add a gentle
  shadow.
- Orchestrate the existing `ash-arrive` animation into a staged entrance:
  header contents, intro line, headline, description, actions, code panel;
  60–90ms stagger steps, under ~700ms total. Respects the existing
  `prefers-reduced-motion` rules. Homepage only; no other entrance
  animations are added.

### Homepage sections

- Eyebrows gain a small branch tick (two-stroke branch derived from the
  favicon mark) as a custom bullet — one inline SVG symbol, reused.
- `honest-status` band goes full-bleed edge-to-gutter on its tonal panel and
  gains the tree watermark at low opacity behind the copy. Text stays fully
  opaque so contrast never drops.
- Principles/paths/notes sections keep two-column intros and reading rows;
  only spacing rhythm is tuned so hero wash, status band, and footer band
  read as a deliberate progression, not striping.

### Article template

- Margin TOC becomes `position: sticky` inside the existing grid and sits in
  a subtle framed block: fine border, `--color-surface-subtle` background,
  labels at reading scale (no tiny uppercase labels, per the October 2026
  design-system addition).
- Callouts keep the rust left rule; the `strong` label gets semibold sans
  treatment consistent with eyebrows. No new callout colors.
- Prose `pre` blocks gain the same border treatment as `CodeExample` so
  component code and article code read as one system.

## Explicit non-goals

- No changes to `public/tokens.css` (shared, synced contract).
- No changes to `public/theme.css` or the font set (Source Sans 3 +
  TeX Gyre Pagella).
- No new dependencies, animation libraries, or markup restructuring beyond
  class hooks and the footer band wrapper.
- No copy changes; existing prose stands.
- Spec, docs, study, search, and 404 pages are out of scope.

## Files expected to change

- `src/styles/global.css` — bulk of the revision.
- `src/layouts/Site.astro` — footer band, inline SVG symbol.
- `src/pages/index.astro` — class hooks only.
- `src/layouts/Article.astro` — aside structure hooks.
- `docs/design-system.md` — presentation section updated for new treatments.
- `tests/browser/` — new visual snapshot tests (`/`, `/learn/tour/`,
  `/status/` at 1440px and 760px) with axe assertions.

## Verification

- `npm run build` and `npm test` stay green (42 tests at baseline).
- `npm run test:browser` passes, including new snapshots.
- Screenshot pass compared against before-images in
  `.cache/frontend-design-review-e273371/`.
- `npm run tokens:check` confirms the token contract is untouched.
