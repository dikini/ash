# TASK-1536: Runtime Remove List Primitive

## Status: 📝 Planned

## Description

Remove `Value::List(Box<Vec<Value>>)` from the runtime. Update `eval.rs`, `small_step.rs`, and all list handling to use `Value::Variant` (Cons/Nil) instead. This is the highest-risk task in Phase 153.

## Specification Reference

- [SPEC-089: List Builtin to Stdlib](../../spec/SPEC-089-LIST-BUILTIN-TO-STDLIB.md)
- [PLAN-153: List Builtin to Stdlib](../PLAN-153-LIST-BUILTIN-TO-STDLIB.md)
- [TASK-1535](TASK-1535-typechecker-list-constructor.md) — Type checker dependency

## Acceptance Criteria

- [ ] `Value::List` removed from `crates/ash-core/src/value.rs`
- [ ] List builtins removed from `builtin_dispatch_table()`
- [ ] `eval.rs` updated: list literal evaluation, pattern matching, foreach
- [ ] `small_step.rs` updated: list iteration, foreach
- [ ] `execute.rs` updated: list type conversion
- [ ] All pattern matching on lists uses `Cons`/`Nil`
- [ ] Serialization updated for lists

## Risk Mitigation

- [ ] Create backup branch before changes
- [ ] Change one file at a time, test after each
- [ ] Use `grep` to find all `Value::List` references
- [ ] Run full test suite after each file change

## Verification

- `cargo test --workspace` passes
- `cargo test -p ash-interp` passes
- `cargo test -p ash-engine` passes
- No `Value::List` references remain: `grep -r "Value::List" crates/`
