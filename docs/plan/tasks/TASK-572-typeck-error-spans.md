# TASK-572: Type Checker — Add Spans to All Error Types

**Phase:** 85
**Spec:** SPEC-040 §4
**Related:** TASK-573
**Estimate:** 12 hours
**Status:** 📝 Planned

## Description

Add `span: ash_parser::token::Span` to every variant of `TypeEnvError`, `ExhaustivenessError`, `NameError`, and `ConstructorError::UnknownConstructor`.

## Requirements

1. Every `TypeEnvError` variant includes a `span` field.
2. `ExhaustivenessError::NonExhaustiveMatch` includes a `span` field.
3. Every `NameError` variant includes a `span` field.
4. `ConstructorError::UnknownConstructor` becomes a struct variant with `span`.
5. All construction sites updated to pass the relevant span.

## Affected Files

- `crates/ash-typeck/src/error.rs` — enum definitions
- `crates/ash-typeck/src/type_env.rs` — all `TypeEnvError` construction
- `crates/ash-typeck/src/check_expr.rs` — `ExhaustivenessError`, `UnknownConstructor`
- `crates/ash-typeck/src/check_pattern.rs` — `ExhaustivenessError`
- `crates/ash-typeck/src/name_binding.rs` — all `NameError` construction
- All tests constructing these errors

## Completion Checklist

- [ ] All error variants carry `span`
- [ ] All construction sites updated
- [ ] All tests updated
- [ ] `cargo test --all` passing
- [ ] Clippy and fmt clean
