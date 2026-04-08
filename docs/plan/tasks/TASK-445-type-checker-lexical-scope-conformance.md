# TASK-445: Type Checker Lexical Scope Conformance

## Status: ⏳ Planned

## Description

Align type checking with the canonical lexical-block lowering so compile-time name resolution follows the same continuation-owned scope model as the normative semantics. This task should make bound names visible for the remainder of the enclosing block and reject truly unbound names consistently.

## Specification Reference

- [SPEC-003: Type System](../../spec/SPEC-003-TYPE-SYSTEM.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [TASK-443: Surface Statement List Scoping Spec Amendment](TASK-443-surface-statement-list-scoping-spec-amendment.md)
- [TASK-444: Parser And Lowering Lexical Block Normalization](TASK-444-parser-and-lowering-lexical-block-normalization.md)

## Dependencies

- ⏳ [TASK-443: Surface Statement List Scoping Spec Amendment](TASK-443-surface-statement-list-scoping-spec-amendment.md)
- ⏳ [TASK-444: Parser And Lowering Lexical Block Normalization](TASK-444-parser-and-lowering-lexical-block-normalization.md)

## Requirements

1. Type checking must consume the canonical lowered lexical-block form consistently.
2. Later statements in the same block must be able to type-check against earlier bindings.
3. Truly unbound names must still be rejected with the appropriate type-checking error.
4. Add focused type-check tests for lexical visibility, shadowing, and unbound-name rejection.

## TDD Steps

### Red

- Current compile-time behavior can drift from the intended lexical-block scope because the surface-to-core normalization rule is not yet fixed end-to-end.

### Green

- Type-checking behavior follows the same lexical-scope rule that the spec and lowering now declare.

## Completion Checklist

- [ ] Focused type-check tests cover lexical scope and unbound names
- [ ] Type environment extension matches canonical `LET ... in cont`
- [ ] `CHANGELOG.md` records the type-checker alignment
