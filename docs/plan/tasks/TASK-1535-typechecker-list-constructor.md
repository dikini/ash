# TASK-1535: Type Checker List Constructor

## Status: ✅ Complete

## Description

Update the type checker to handle `List<T>` as an ordinary type constructor rather than a primitive type. Remove or repurpose `Type::List(Box<Type>)`.

## Specification Reference

- [SPEC-089: List Builtin to Stdlib](../../spec/SPEC-089-LIST-BUILTIN-TO-STDLIB.md)
- [PLAN-153: List Builtin to Stdlib](../PLAN-153-LIST-BUILTIN-TO-STDLIB.md)
- [TASK-1534](TASK-1534-parser-list-literal-desugaring.md) — Parser dependency

## Acceptance Criteria

- [ ] `List<T>` typechecks as ordinary constructor
- [ ] Type unification works with `List<T>`
- [ ] Generic instantiation works with `List<T>`
- [ ] Pattern type inference works with `Nil` and `Cons`
- [ ] All existing list type tests pass

## Verification

- `cargo test -p ash-typeck` passes
- `cargo test -p ash-cli --test list_builtin_typeck` passes
- No regressions in existing tests
