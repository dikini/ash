# TASK-572: Type Checker — Add Spans to All Error Types

**Phase:** 85
**Spec:** SPEC-040 §4
**Related:** TASK-573
**Estimate:** 12 hours
**Status:** 📝 Planned

## Description

Add spans to all type-checker error variants that lack them.

## Prerequisites

- SPEC-039 AST span gaps must be resolved or explicitly worked around: `TypeDef`/`InterfaceDef`/`ImplDef` lack `Span` fields, `Expr::Variable`/`Literal` lack spans, `Pattern` variants lack spans, and `NameBinder` APIs take `&str` without `Span`. Until resolved, some construction sites may pass `Span::default()` or approximate parent spans.

## Requirements

1. Every `TypeEnvError` variant includes a `span` field.
2. **All** spanless `ConstructorError` variants (`UnknownConstructor`, `MissingField`, `UnknownField`, `FieldTypeMismatch`, `TupleFieldTypeMismatch`, `TupleArityMismatch`, `NonExhaustiveMatch`) and `TypeError::NotAConstructor` include a `span` field.
3. Every `NameError` variant includes a `span` field.
4. Every `ResolutionError` variant includes a `span` field.
5. Every `TypeError` variant includes a `span` field (verify existing `UnknownObligation`, `ObligationAlreadySatisfied`, `UnknownCapability`, and `InvalidConstraintField` already have spans at all call sites).
6. Ensure all updated error types are ready for `AshLspError` trait implementations in `crates/ash-diagnostic` (see SPEC-040 §5.4).

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
