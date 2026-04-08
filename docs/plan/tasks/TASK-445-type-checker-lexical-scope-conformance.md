# TASK-445: Type Checker Lexical Scope Conformance

## Status: ✅ Complete

## Description

Align type checking with the canonical lexical-block lowering so compile-time name resolution follows the same continuation-owned scope model as the normative semantics. This task should make bound names visible for the remainder of the enclosing block and reject truly unbound names consistently.

## Specification Reference

- [SPEC-003: Type System](../../spec/SPEC-003-TYPE-SYSTEM.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [TASK-443: Surface Statement List Scoping Spec Amendment](TASK-443-surface-statement-list-scoping-spec-amendment.md)
- [TASK-444: Parser And Lowering Lexical Block Normalization](TASK-444-parser-and-lowering-lexical-block-normalization.md)

## Dependencies

- ✅ [TASK-443: Surface Statement List Scoping Spec Amendment](TASK-443-surface-statement-list-scoping-spec-amendment.md)
- ✅ [TASK-444: Parser And Lowering Lexical Block Normalization](TASK-444-parser-and-lowering-lexical-block-normalization.md)

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

- [x] Focused type-check tests cover lexical scope and unbound names
- [x] Type environment extension matches canonical `LET ... in cont`
- [x] `CHANGELOG.md` records the type-checker alignment

## Implementation Notes

The type checker has been aligned with the canonical lexical-block lowering:
- Name resolution now correctly extends the type environment for `let` bindings
- Later statements in the same block can reference earlier bindings
- Unbound names are rejected with appropriate type errors
- Test coverage confirms lexical scope behavior

This ensures that compile-time name resolution matches the runtime semantics established by the canonical lowering.
