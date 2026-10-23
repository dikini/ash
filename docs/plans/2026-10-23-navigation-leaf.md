# Navigation leaf implementation plan

**Goal:** Make current location and its context readable in vertical navigation, retaining both leaf variants as shared assets.

**Architecture:** Shared public navigation CSS and a small progressive-enhancement script serve Astro contents menus and imported Quarto chapter/contents navigation. The upstream book remains byte-preserved except for site integration additions. Existing typography, palette, focus, spacing, and motion rules remain authoritative.

**Tech stack:** Astro, plain browser JavaScript, CSS masks, SVG, Node tests, Playwright.

1. Add behavioral tests for heading selection, fragment handling, current-section updates, and nested ancestor distinction. Run before implementation to confirm the missing behavior.
2. Keep outline and solid SVGs under public assets. Add shared styles with reserved marker gutters, semibold neutral ancestors, stronger neutral current labels, token-colored outline markers, and forced-colors fallback. Keep marker decorative and motionless.
3. Add section tracking with one current item per contents navigation; initialize from the URL, follow scrolling and history changes, and update ancestor emphasis. Use existing Quarto active chapter markup for static chapter selection and ancestor styles. Integrate through source layouts and study preparation, never generated outputs.
4. Update docs/design-system.md and repository guidance with the decision and rationale: users need both exact location and overall context; outline is quiet enough for repeated reading UI; solid is retained for intentional stronger emphasis, with no automatic future placement.
5. Run npm run check, npm test, npm run build, focused browser checks on article and hierarchical study menus, and git diff --check. Review desktop/mobile screenshots and report any limitations.
