# Ash design system

Ash uses warm paper surfaces, green-black text, seal-red accents, serif
display headings, and quiet rules. Code appears on a dark green surface. Use
contrast, typography, and spacing to establish hierarchy before introducing
decoration.

## Token contract

`public/tokens.css` is the website copy of the shared portable CSS contract
(version 1).
`public/theme.css` imports it and registers the website's bundled Source Sans
fonts. Both the website and hosted study chapters load this theme.

| Family | Tokens and purpose |
| --- | --- |
| Surfaces | `--color-surface-page`, `subtle`, `hover`, `callout` |
| Text | `--color-text-primary`, `secondary`, `navigation`, `status` |
| Accent | `--color-accent` for text and focus; `--color-accent-bright` for decoration on dark surfaces |
| Code | `--color-code-surface`, `text`, `muted`, `border`, `inline` |
| Type | `--font-sans`, `serif`, `mono`; `--text-xs` through `xl`, `reading`, `display` |
| Rhythm | `--space-1` through `6`, then `8`, `10`, `12`, `16`, `20` (4–80px) |
| Layout | `--layout-max` (1240px), `--layout-article` (760px), `--measure-reading` (65ch) |
| Responsive | `--layout-gutter`: 56px, 32px below 1100px, 20px below 760px |
| Shape | `--radius-control` (3px), `--radius-code` (5px), `--radius-brand` (10px) |
| Interaction | `--focus-width`, `--focus-offset`, `--motion-fast`, `--motion-easing` |

Names such as `--paper`, `--ink`, `--accent`, and `--mono` remain public aliases.
Use semantic tokens for new roles. Add a primitive only when multiple consumers
need it; tuned one-off illustration or layout dimensions can remain local.
Breakpoints stay literal in media queries because CSS custom properties cannot
be used in media-query conditions.

```css
.notice {
  background: var(--color-surface-subtle);
  color: var(--color-text-primary);
  padding: var(--space-5);
  border-radius: var(--radius-control);
}
```

## Components

- Buttons use ink on paper in reverse, semibold sans text, and the control
  radius. Their hover surface uses the accent with white text.
- Links in prose use the accent and an underline. Navigation identifies the
  current page with the accent. Preserve visible keyboard focus.
- Cards use a fine border and a subtle hover surface; headings use serif type.
- Callouts use a subdued surface and an accent left rule. Include a text label
  so meaning does not depend on color.
- Code uses monospace text, the dark code palette, horizontal scrolling,
  and keyboard focus for scrollable blocks. Inline code uses the light palette.
- Body copy uses sans; diary prose and display headings use serif. The study
  book retains its own bundled serif and monospace fonts and margin layout.

The bright benihi token is decorative; use the darker enji accent for links on
paper. Keep reduced-motion and forced-colors rules. Coarse-pointer navigation
has 44px targets. Avoid shrinking body text to fit a layout.

## Local bidirectional synchronization

Either repository can originate token edits. Each carries a committed
`.design-tokens-base.css` containing the last shared content. The same Python
engine lives in `scripts/token_sync.py` and ash-doc's `rendering/token_sync.py`.
It uses the Python standard library and POSIX file locks (Linux/macOS), with
no network, credentials, hooks, or hosted runners.

Before site development and builds, `predev` and `prebuild` synchronize tokens.
Ash-doc's HTML build entry point does the same before preparation. A one-sided
edit propagates to the other checkout and both baselines advance. Identical
edits are accepted; different edits on both sides fail with a diff and leave
all files unchanged. Synchronization edits working files in both repositories;
review and commit each repository's token and baseline changes together.

```sh
npm run tokens:sync
npm run tokens:check
# Optional explicit peer checkout or task worktree:
node scripts/sync-design-tokens.mjs /path/to/ash-doc
node scripts/sync-design-tokens.mjs --check /path/to/ash-doc
```

The default peer is `ash-doc` beside the primary ash-site checkout, including
when invoked from a Git worktree. Set `ASH_DOC_CHECKOUT=/path/to/ash-doc-worktree`
to select another checkout. From ash-doc, the corresponding variable is
`ASH_SITE_CHECKOUT`. Explicit missing or invalid peers fail; an absent automatic
peer allows standalone use without advancing the baseline. A present but
incomplete peer also fails, so a broken checkout cannot silently skip checks.

Set `ASH_TOKENS_STANDALONE=1` for an isolated build using only local tokens,
even when a sibling exists. This bypasses cross-repository drift checks too.
Synchronization never fetches or commits changes. The existing study importer
still fetches documentation as described in the integration guide. Its child
render/verify processes force standalone token mode to preserve the exact
fetched source, even when the parent shell configures a local peer.

`npm run check` performs the read-only token check before Astro diagnostics.
`npm test` includes hermetic synchronization tests. In ash-doc, run
`python3 rendering/token_sync.py --check`; its artifact verifier also runs this
read-only check. PDF/EPUB-only builds do not synchronize HTML tokens.

### Resolve conflicts

Review the reported diff, reconcile the token content, and put the intended
identical bytes in both token files. Run synchronization again to establish
the new shared baseline. Do the same when baselines differ after branch changes.
Do not delete or manually reset baselines to choose a winner. The tool never
uses file modification times to decide ownership.

Locks serialize concurrent synchronization; individual replacements are atomic.
A process or filesystem failure between replacements can leave partial progress.
If rerunning reports a baseline conflict, reconcile the two token files as above.
No automatic cross-repository commit transaction is claimed.

### Build identity and ownership

The site writes `dist/design-tokens.json` with the SHA-256 of the copied CSS.
Ash-doc records `design_tokens_sha256` in its HTML `build.json`; artifact
verification checks the staged and published token files against that hash.
Keep the identical engine and regression tests updated together when changing
the synchronization protocol. No GitHub workflow is added for this feature.

Hosted chapters receive the website theme, including snapshots from older
ash-doc revisions. The book retains its bundled reading fonts and margin
layout; EPUB and PDF retain their format styles. Token versions describe
presentation, not an Ash language edition.

## Website presentation (October 2026)

The website pairs bundled Source Sans 3 body and navigation text with TeX Gyre
Pagella display headings. Pagella's regular font and GUST license are copied into
`public/fonts/` from the existing study publication assets so site headings do
not depend on a device-installed serif or a generated study snapshot. This is a
website font registration, not a change to the portable token contract or the
book's reading fonts. Multilingual font evaluation remains separate work.

Use sentence-case section labels at a readable size. Homepage language properties
are definitions, and learning routes are reading rows rather than an equal card
grid. Prefer direct descriptions of language behavior to promotional headings.
Keep the complete canonical program and its output as the homepage's main visual.
Code captions identify the source filename and result without imitation editor
controls. Status copy distinguishes the implementation from research ambitions.
The latest homepage update is derived from published content, including its date.

Article layouts retain the reading column and margin contents navigation. Match
metadata and navigation to the reading scale; do not use tiny uppercase labels.
A brief homepage movement is the only entrance animation and respects reduced
motion. Text remains fully opaque throughout so its contrast never drops.

The October 2026 revision adds restrained depth without changing tokens or
fonts. The footer is a full-bleed ink band; the tree mark appears as a
low-opacity watermark in paper (`currentColor`, 7%) and the footer brand uses
the bare tree in place of the black-square icon; footer link hovers and focus
outlines use `--color-accent-bright` (5.0:1 on ink — the dark accent fails on
ink, so the bright token is required there). Eyebrows carry a two-stroke
branch tick in the accent, rendered with a `data:`-URI CSS mask — the revision
design doc's (`docs/plans/2026-10-20-site-design-revision-design.md`)
inline-SVG-symbol tick was superseded, because the mask avoids a third copy of
branch geometry in markup and works on every eyebrow from one rule. The
homepage code panel sits in a radial surface-subtle spotlight and carries the
`--shadow-code` token shadow. The homepage entrance staggers translateY-only
arrivals (about 700ms; text never fades, per this section's opacity rule). The
status panel carries the same tree watermark at 5.5% opacity. The article
margin column is sticky for the article's full height, capped to
`calc(100vh - 2 * var(--space-6))`; the "On this page" frame scrolls
internally when a TOC exceeds the cap — the whole column rides, note included,
and long TOCs like the 250-entry kernel page scroll inside the frame — and is
`position: static` below 760px. Callout labels are semibold sans. Prose code
blocks share the component border, with `border: 0` on nested
`.code-example pre` to avoid doubled separators. Watermarks are decorative
(`aria-hidden`) and hidden under forced colors. The single-entrance-animation
budget still holds: no scroll-linked or per-section motion.

### Beni accents and the home page revision (October 2026)

The accent palette moves from rust to a beni family: enji `#b7282e` (5.6:1 on
paper; 6.3:1 with white button text) becomes `--color-accent`, benihi `#f35336`
becomes `--color-accent-bright`, and text selection becomes a pale rose
`#f3d4d2`. The palette completes the paper-and-ink identity with the hanko
convention — the red seal that signs the page; the brand mark's corner dot
uses benihi. Edit `public/tokens.css` to change a token: the engine advances
the committed `.design-tokens-base.css` baseline during synchronization, so a
hand-edited baseline is reverted on the next sync.

The home page headline is a three-line haiku (5–7–5) set as block spans with
a 0.4em verse gap. The example panel holds a fixed 480px and aligns its top
with the heading on wide screens; the hero stacks at 900px so tablet portrait
gets the poem and panel at full width. Homepage sections carry eyebrows that
name their destination (Discover, Learn, Diaries, Updates, Project status) with
the shared branch tick, and right-hand columns offset by the eyebrow block so
their headings align with the section heading. The status band and the
homepage definitions list share a framed-panel treatment — surface-subtle
wash, hairline border, control radius — so definitions read as a reference
panel beside the open reading rows; the hairline keeps the band visible where
the wash alone falls below the perceptual threshold. The type distinction the
frames carry stays: definition terms are semibold sans labels, route titles
use serif.

The colour revision gives the shared header and home opening a continuous
green-black field. The home hero uses paper headings, code-muted secondary
text, a seal-red primary action, and benihi links and focus outlines on the
dark surface. The header brand reuses the bare tree symbol rather than the
black-square favicon; the browser theme colour matches the ink header. Paper
returns at Discover, while the home definition and status panels use a deeper
sage mixed from the existing surface and status tokens. The hero display is
capped at 56px so each haiku verse stays on one line on wide screens; narrow
screens retain readable wrapping. The Learn heading is “Tour, Study, Specs”;
the adjoining Discover and Learn padding is 48px on each side, while their
outer spacing and mobile rhythm remain unchanged. The ash-tree detail,
typefaces, column geometry, and section order remain unchanged. The mobile
menu retains its paper reading surface and native disclosure behavior.

### Location and context in vertical navigation

The primary job of reading navigation is to answer two questions at a glance:
where am I, and where does this page or section sit in the surrounding material?
Represent exact location and its context with different levels of emphasis.

- Current page or section: strongest neutral text and one small beni outline
  ash leaflet beside the label, pointing toward it. Use `--color-accent` from
  the existing beni palette, not a new red or the brighter dark-surface accent.
- Ancestors on the active branch: semibold primary neutral text. They are
  more prominent than unrelated items, without colored markers. Preserve
  indentation, disclosure controls, and expanded current-page ancestors.
- Other items: regular navigation text. Hover and visible keyboard focus
  retain their own treatments and never impersonate the current-location mark.

The outline leaf was selected over a bar, dot, and solid leaf. Its light visual
weight suits repeated consultation while reading, while its silhouette relates
to the Ash tree and eyebrow branches. It is a pointer, not a decorative branch,
a progress meter, a completion symbol, or an animation. The current label stays
strong enough to carry the state without relying on the accent color alone.
A fixed gutter prevents text from shifting as the marker changes, and the marker
aligns with the first line of a wrapped label. Ancestors supply the context;
repeating the same marker on them would weaken the exact-location signal.

Both variants are maintained as reusable assets:
`public/marks/ash-leaf-outline.svg` and `public/marks/ash-leaf-solid.svg`.
They use `currentColor`; shared CSS exposes `--ash-leaf-outline` and
`--ash-leaf-solid` as mask sources. Use the outline by default for navigation.
Retain the solid variant for future contexts that need a stronger visual
signal; its presence is not a recommendation to place it on every selected
control. Choose by the attention a state deserves relative to its surroundings,
and recheck legibility at the actual size and contrast on the actual surface.
Do not replace unrelated symbols or disclosure chevrons with a leaf.

`public/navigation.css` and `public/navigation.js` apply this treatment to Astro
article contents, the vertical mobile menu, and hosted Quarto study navigation. Contents track the section
being read with `aria-current="location"`; chapter links use
`aria-current="page"`. Quarto keeps its native collapse controls and nesting;
CSS emphasizes the current chapter's ancestors and preserves its expanded path.
The imported source is integrated through `scripts/study-html.mjs`, never by
editing generated chapters. Static chapter selection works without JavaScript;
scroll-following contents is progressive enhancement. The book's tablet/mobile
`details.on-this-page` disclosure is a separate list cloned after the DOM loads;
initialize its tracking after those handlers and apply the same marker and
ancestor treatment. Restore depth from its target headings because the clone
flattens the original nested lists. Cover this disclosure explicitly at the
820 × 1180 iPad viewport, rather than assuming desktop contents tests cover it.
Markers are decorative,
stationary, do not move focus, and have a forced-colors shape fallback.

Apply the same location/context distinction when designing other navigation:
flat menus need only a current item; nested menus also need ancestor emphasis.
Use hierarchy, contrast, and type before adding more color or ornament.
