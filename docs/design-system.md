# Ash design system

Ash uses warm paper surfaces, green-black text, rust links, serif display
headings, and quiet rules. Code appears on a dark green surface. Use contrast,
typography, and spacing to establish hierarchy before introducing decoration.

## Token contract

`public/tokens.css` is the canonical, portable CSS contract (version 1).
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
  radius. Their hover surface uses the rust accent with white text.
- Links in prose use rust and an underline. Navigation identifies the current
  page with the accent. Preserve visible keyboard focus.
- Cards use a fine border and a subtle hover surface; headings use serif type.
- Callouts use a subdued surface and rust left rule. Include a text label so
  meaning does not depend on color.
- Code uses monospace text, the dark code palette, horizontal scrolling,
  and keyboard focus for scrollable blocks. Inline code uses the light palette.
- Body copy uses sans; diary prose and display headings use serif. The study
  book retains its own bundled serif and monospace fonts and margin layout.

The bright orange token is decorative; use the darker accent for links on
paper. Keep reduced-motion and forced-colors rules. Coarse-pointer navigation
has 44px targets. Avoid shrinking body text to fit a layout.

## Synchronize ash-doc

Ash-site owns the token values. Ash-doc vendors a byte-identical, asset-free
copy at `rendering/styles/ash-tokens.css`; its HTML configuration loads that
file alongside its reading stylesheet. EPUB and PDF retain their format styles.
Standalone documentation does not require the website or its font assets.

From ash-site, pass the intended ash-doc checkout or task worktree explicitly:

```sh
node scripts/sync-design-tokens.mjs /path/to/ash-doc
node scripts/sync-design-tokens.mjs --check /path/to/ash-doc
```

Review both repository diffs after changing tokens. Run `npm run check`,
`npm run build`, and `npm test`; render a representative ash-doc HTML lesson
and inspect desktop, narrow viewports, code, links, and focus. Run the sync
check before submitting changes. The sync command only updates the token copy;
renderer wiring is installed once in ash-doc.

Hosted chapters always receive the current website theme, including snapshots
from older ash-doc revisions. Upstream content and print styles remain owned
by ash-doc. Token version changes describe a presentation contract, not an Ash
language edition.
