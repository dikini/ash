# Hosted study spacing critique

The earlier taste critique in the redesign worktree reviews the homepage only.
It does not establish a quality floor for the hosted study pages. Use the site's
existing reading design, not a new marketing-page art direction, for these pages.

ROUND 1
Seen in one second: chapter navigation pressed against the chapter title and prose.
Idea (one sentence): separate navigation from reading with an actual empty grid track.
Live thing: native chapter navigation, disclosures, and search.
Layout words by section: navigation=scrolling rail / chapter=reading column / illustrations=margin.
Scores: design 6 · usability 6 · creativity 7 · content 9 → overall 6.6
Caps applied: assess against the established documentation design system; marketing
requirements for pictures, animation, and calls to action do not apply.
Top three problems (most visible first), each with the screenshot it shows in and the fix:
1. The scrollbar touches the prose in the supplied Naming intermediate values
   screenshot. The sidebar spans the intended gutter; stop it at body-start-outset.
2. The reserved track is only 24px before the sidebar consumes it. Use the existing
   space-8 token for a visible 32px separation at desktop widths.
3. The existing regression accepts zero clearance. Require at least 31px (allowing
   one pixel of rounding) on ordinary and full-width chapters at 992–1440px.

ROUND 2
Seen in one second: a distinct navigation rail beside a readable chapter.
Idea (one sentence): the empty gutter separates browsing from sustained reading.
Live thing: unchanged native reading controls.
Layout words by section: navigation=scrolling rail / chapter=reading column / illustrations=margin.
Scores: design 8 · usability 9 · creativity 7 · content 9 → overall 8.3
Caps applied: same documentation-specific scope as round 1.
Top three problems (most visible first), each with the screenshot it shows in and the fix:
1. No remaining gutter defect in production-build screenshots at 1440, 992, and
   390px; desktop measures 32px and mobile retains its 20px edge inset.
2. Long navigation labels wrap in the narrower rail. Accepted to preserve reading
   width and native navigation; do not reduce the text size.
3. No additional change justified by this spacing review. Full-width illustration,
   navigation, mobile overflow, and disclosure checks remain required.

Verification: reviewed production screenshots at all three widths and ran the
focused study/navigation browser suite: 28 passed, 2 skipped. The spacing test
failed on the previous CSS before the correction.
