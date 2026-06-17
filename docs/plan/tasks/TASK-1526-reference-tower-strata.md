# TASK-1526: Reference Documentation — Tower Strata

## Status: 📝 Planned

## Description

Write `reference/language/tower.md` with comprehensive documentation on Ash's semantic tower: Pure, Act, Proc, and Workflow strata, with examples, callable arrows, and boundary rules.

## Specification Reference

- [SPEC-088: Closure Refinement and Effect-Safe Capture](../../spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md)
- [SPEC-072: Tower Callable Type and Closure Syntax](../../spec/SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md)
- [PLAN-152: Closure Refinement and Tower Documentation](../PLAN-152-CLOSURE-REFINEMENT-AND-TOWER-DOCUMENTATION.md)

## Acceptance Criteria

- [ ] Document the four strata: Pure, Act, Proc, Workflow
- [ ] Document callable arrows for each stratum (`->`, `-*`, `=>`, `=*>`)
- [ ] Document stratum boundaries and what crosses them
- [ ] Document closure rules at each stratum
- [ ] Document effect levels and ordering
- [ ] Include examples at each stratum
- [ ] Include YAML frontmatter with metadata, verified_against, cross-references
- [ ] Cross-reference to functions.md and types/records.md

## Verification

- All examples parse and typecheck correctly
- Cross-references are accurate
- Documentation reviewed for clarity and completeness
