# Home page colour and contrast design

Date: 2026-10-30

Status: approved visual direction

## Context

The home page's geometry, typography, copy, and content order are working, but
its warm paper surface dominates too much of the experience. The design should
add colour and contrast while preserving the white-paper, black-ink, red-stamp
identity and the existing ash detail lower on the page.

OCaml provides the main reference: a continuous dark opening gives the
navigation, headline, and program example a clear visual field. Rust and Go
also demonstrate the value of decisive colour blocking, but Ash should retain
a quieter editorial character.

## Design

Use a continuous green-black field behind the site navigation, introductory
status line, and home hero. Return to warm paper at Discover. Keep the existing
sage definition and status panels and the ash-tree detail. Red remains scarce:
it identifies actions, current navigation, and small brand details.

On the dark opening:

- Set primary text in warm paper and secondary text in muted sage.
- Use the bare ash-tree brand mark rather than the black-square icon.
- Use paper-toned navigation, with the existing bright beni accent for hover,
  current-page, and keyboard-focus states where contrast requires it.
- Make the primary tour button seal red with white text.
- Preserve the existing code panel, using its border and shadow to distinguish
  it from the dark hero.

Do not add gradients, cards, ornaments, animations, or new typography. Do not
rewrite copy, reorder sections, or change the home page geometry.

## Implementation boundaries

Limit changes to the shared header treatment, home-page colour rules, and the
browser theme colour. Prefer existing semantic tokens and dark-surface accent
tokens. Keep mobile navigation legible and preserve native disclosure,
keyboard focus, reduced-motion, and forced-colours behavior.

## Verification

Review the home page at desktop and mobile widths, including navigation and
the transition from the dark opening to paper. Run `npm run check`. Avoid a
broad testing pass for this restrained visual revision.
