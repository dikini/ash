# TASK-572: Type Checker — Add Spans to All Error Types

**Phase:** 85
**Spec:** SPEC-040 §4
**Related:** TASK-573
**Estimate:** 12 hours
**Status:** 📝 Planned

## Description

Add spans to all type-checker error variants that lack them.

## Requirements

1. Every `TypeEnvError` variant includes a `span` field.
2. **All** spanless `ConstructorError` variants (`UnknownConstructor`, `MissingField`, `UnknownField`, `FieldTypeMismatch`, `TupleFieldTypeMismatch`, `TupleArityMismatch`, `NonExhaustiveMatch`) and `TypeError::NotAConstructor` include a `span` field.
3. Every `NameError` variant includes a `span` field.
4. Every `ResolutionError` variant includes a `span` field.
5. Every `TypeError` variant includes a `span` field (verify existing `UnknownObligation`, `ObligationAlreadySatisfied`, `UnknownCapability`, and `InvalidConstraintField` already have spans at all call sites).

## Affected Files

- `crates/ash-typeck/src/error.rs` — enum definitions
- `crates/ash-typeck/src/type_env.rs` — all `TypeEnvError` construction
- `crates/ash-typeck/src/check_expr.rs` — all `ConstructorError` construction
- `crates/ash-typeck/src/name_binding.rs` — all `NameError` construction
- `crates/ash-typeck/src/names.rs` — all `ResolutionError` construction
- `crates/ash-typeck/src/solver.rs` — all `TypeError` construction (including `NotAConstructor`)
- All tests constructing these errors

## Completion Checklist

- [ ] All error variants carry `span`
- [ ] All construction sites updated
- [ ] All tests updated
- [ ] `cargo test --all` passing
- [ ] Clippy and fmt clean
