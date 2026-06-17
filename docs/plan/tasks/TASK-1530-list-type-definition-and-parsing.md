# TASK-1530: List Type Definition and Parsing

## Status: ✅ Complete

## Description

Add `List<T>` type definition to `std/src/list.ash` as an ordinary algebraic data type. Verify that the parser and typechecker handle the definition correctly.

## Specification Reference

- [SPEC-089: List Builtin to Stdlib](../../spec/SPEC-089-LIST-BUILTIN-TO-STDLIB.md)
- [PLAN-153: List Builtin to Stdlib](../PLAN-153-LIST-BUILTIN-TO-STDLIB.md)

## Acceptance Criteria

- [ ] `List<T>` type defined as `Nil | Cons { head: T, tail: List<T> }`
- [ ] Parser accepts the type definition
- [ ] Typechecker resolves `List<T>` correctly
- [ ] Existing code using `List<T>` still compiles
- [ ] No regressions in stdlib corpus check

## Verification

- `cargo test -p ash-parser` passes
- `cargo test -p ash-typeck` passes
- `cargo test -p ash-cli --test stdlib_corpus_check` passes
