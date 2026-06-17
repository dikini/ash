# TASK-1534: Parser List Literal Desugaring

## Status: 📝 Planned

## Description

Update the parser to desugar `[1, 2, 3]` syntax to `Cons`/`Nil` variant expressions instead of `Value::List` literals.

## Specification Reference

- [SPEC-089: List Builtin to Stdlib](../../spec/SPEC-089-LIST-BUILTIN-TO-STDLIB.md)
- [PLAN-153: List Builtin to Stdlib](../PLAN-153-LIST-BUILTIN-TO-STDLIB.md)
- [TASK-1530](TASK-1530-list-type-definition-and-parsing.md) — Type definition dependency

## Acceptance Criteria

- [ ] `[1, 2, 3]` parses to `Cons(1, Cons(2, Cons(3, Nil)))`
- [ ] `[]` parses to `Nil`
- [ ] Pattern matching `[head, ..tail]` works
- [ ] Empty list pattern `[]` works
- [ ] All existing list literal tests pass

## Verification

- Parser tests for list literals pass
- `cargo test -p ash-parser` passes
- No regressions in existing tests
