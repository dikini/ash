# Frontend design revision

Date: 2026-10-20

## Scope and constraints

Revise the site presentation using existing Astro layouts, content collections,
ordinary CSS, and Pagefind. Preserve all current source and discovery work.
Retain the documented warm paper, green-black text, rust links, quiet rules,
and dark green code treatment. Keep imported study reading layouts and shared
portable tokens intact. No new dependencies or product functionality.

## Review

The current homepage repeats small uppercase labels, generic promotional
headings, arbitrary numbered features, and equal card treatments. Its long
introduction competes with the first program. Navigation and code metadata are
too small. Article side navigation loses hierarchy and has excessive gaps.

## Direction

A technical publication with a readable first program and direct reading paths.
Use existing palette roles: paper #f5f3ec, ink #171c18, rust #973b18,
subtle #e9ecdf, code #18231c, code text #f1f3e6. Preserve tokens.css bytes.
Source Sans 3 remains body and interface type. Bundle TeX Gyre Pagella from the
existing study assets for website headings; preserve its license. Do not change
the book's own font choices or claim a new multilingual typography system.

All text is left aligned. The program is the memorable visual, with a compact
caption, filename, output, and plain explanation. Avoid decorative diagrams,
large background motifs, gradient washes, badges, and faux editor controls.
Atmosphere comes from restrained paper-to-subtle surface variation behind code.

Desktop:

    brand       primary navigation                search / source
    language identity                             development status
    specific heading             first complete program
    explanation + tour           output + short explanation
    language properties          three concise definitions
    learning routes              tour / study / reference reading rows
    implementation status        capabilities and research boundary
    developer diaries            latest dated update
    brand                        secondary navigation

Mobile follows the same reading order, stacking at the existing breakpoint.
Article layouts keep the reading column and margin navigation; strengthen the
heading, section label, contents rail, callouts, and metadata. Use one short
CSS entrance on the homepage with reduced motion support. No hidden content
or JavaScript dependency for navigation.

## Implementation plan

1. Copy the licensed existing Pagella regular asset into public/fonts and
   register it in theme.css. Use the new serif only for website presentation.
2. Revise index.astro copy and composition. Keep the canonical example source,
   tour, learning, status, diary, and update routes and target information.
3. Revise Site.astro footer copy and CodeExample.astro metadata; preserve roles,
   accessible navigation, and native details-based mobile controls.
4. Refactor global.css into coherent shared, home, article, responsive, and
   accessibility sections. Retain selector hooks needed by current layouts.
5. Record presentation choices in docs/design-system.md. Run Astro checks,
   static build, existing unit and browser checks, and visual review at desktop,
   tablet, 390px, and 320px. Review reduced motion and text-spacing reflow.

## Verification

Use existing checks instead of implementation-mirroring tests for this reversible
presentation revision. Check all main visitor journeys and imported reference
layouts with the current browser suite. Inspect screenshots of home and article
pages, confirm no horizontal viewport overflow, keyboard focus, and no serious
accessibility findings. Report pre-existing warnings separately.

## Completed verification

- Full `npm run build` passed, including imported references, study snapshot,
  Pagefind, local links and anchors, and build provenance.
- `npm run check` completed with zero errors or warnings and one pre-existing
  unused-import hint in `scripts/update-source-pin.mjs`.
- `npm test` passed: 42 JavaScript tests and 12 token synchronization tests.
- Existing browser suite passed in desktop and mobile Chromium: 75 passed and
  one intentionally skipped mobile-only footer check in the desktop project.
- Reviewed screenshots at 1440px, 820px, 390px, and 320px; verified reduced
  motion, code scrolling, text-spacing reflow, and heading font loading.
- Replaced the initial opacity entrance after browser checks caught transient
  text contrast loss. The final movement keeps text opacity at 1 throughout.
- Independent scoped code review found no actionable issues.
- All 14 pre-existing tracked modifications remain byte-identical to the
  original checkout. Shared token and baseline files are unchanged.

The frontend revision was prepared on `codex/frontend-design`. Review
screenshots and final build/browser logs are in `.cache/design-review/`;
the local preview uses port 4330.
