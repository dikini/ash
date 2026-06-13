# TASK-1444: Stdlib Algebra Agent Card

## Description

Create the derivative agent card `reference/agents/cards/stdlib-algebra.md` following the pattern of existing agent cards (e.g., `stdlib-result.md`). This is a derivative agent card for the `std::algebra` reference page, providing AI-agent-specific context, retrieval tags, stale-claim warnings, and edit preflight guidance.

## Requirements

1. Create `reference/agents/cards/stdlib-algebra.md` with SPEC-071 frontmatter:
   - `id: ref.agents.card.stdlib_algebra`
   - `kind: agent-card`
   - `authority: derivative`
   - `audience: [agent]`

2. Follow the exact pattern of existing agent cards:
   - `canonical_page` and `canonical_page_path` linking back to `ref.stdlib.algebra`
   - `dependency_order` field
   - Warning about reading canonical page first
   - `## Use` section explaining retrieval purpose
   - `## Retrieval tags` section with relevant search terms
   - `## Stale-claim warnings` section with common confusion warnings
   - `## Edit preflight` section with pre-editing checks

3. Include retrieval tags for:
   - "algebra interfaces"
   - "semigroup monoid functor applicative monad"
   - "comonad kleisli"
   - "law declarations"
   - "proof declarations"
   - "by test delegation"
   - "source-visible laws"
   - "Eq evidence"
   - "option result instances"
   - "do notation monad evidence"
   - "interface evidence constraints"

4. Include stale-claim warnings for:
   - `by_definition` proofs are not semantically validated
   - Generated law tests do not execute automatically
   - `Comonad` has no stdlib carrier instances
   - Kleisli helpers are deferred
   - Coapplicative is explicitly deferred

5. Include edit preflight steps:
   - Re-read `std/src/algebra/*.ash`
   - Run parser law/proof tests
   - Run stdlib parsing tests
   - Check proof policy audit

## Completion Checklist

- [ ] `reference/agents/cards/stdlib-algebra.md` created with SPEC-071 frontmatter
- [ ] `canonical_page` links to `ref.stdlib.algebra`
- [ ] `canonical_page_path` resolves to `../../stdlib/algebra.md`
- [ ] Retrieval tags included
- [ ] Stale-claim warnings included
- [ ] Edit preflight steps included
- [ ] `python3 tools/reference/check_frontmatter.py --root .` passes for the new card
- [ ] All markdown links resolve
- [ ] `reference/agents/common-confusions.md` or `reference/INDEX.md` links updated if needed
- [ ] CHANGELOG.md updated
