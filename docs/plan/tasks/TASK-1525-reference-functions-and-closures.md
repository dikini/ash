# TASK-1525: Reference Documentation — Functions and Closures

## Status: ✅ Complete

## Description

Write `reference/language/functions.md` with comprehensive documentation on Ash functions, closures, syntax, capture rules, and examples.

## Specification Reference

- [SPEC-088: Closure Refinement and Effect-Safe Capture](../../spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md)
- [SPEC-031: First-Class Functions](../../spec/SPEC-031-FIRST-CLASS-FUNCTIONS.md)
- [SPEC-072: Tower Callable Type and Closure Syntax](../../spec/SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md)
- [PLAN-152: Closure Refinement and Tower Documentation](../PLAN-152-CLOSURE-REFINEMENT-AND-TOWER-DOCUMENTATION.md)

## Acceptance Criteria

- [ ] Document function definitions (`fn name(params) { body }`)
- [ ] Document closure literals (`fn(params) { body }` and `|params| -> body`)
- [ ] Document callable types (`(A, B) -> C`, `(A, B) -*> C`, etc.)
- [ ] Document capture rules with examples (allowed and blocked)
- [ ] Document effect levels and the capture rule
- [ ] Document recursion via late binding
- [ ] Include YAML frontmatter with metadata, verified_against, cross-references
- [ ] Include working examples for each concept

## Verification

- All examples parse and typecheck correctly
- Cross-references to specs are accurate
- Documentation reviewed for clarity and completeness
