# TASK-907: Track constructor variables, apply them by kind, and add non-inverting constructor unification

## Status: ✅ Complete

## Description

Track constructor variables, apply them by kind, and add non-inverting constructor unification

## Specification Reference

- [SPEC-067](../../spec/SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md)
- [PLAN-116](../PLAN-116-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md)

## Dependencies

- ✅ SPEC-067: spec packet exists
- ✅ PLAN-116: implementation plan exists
- Depends on TASK-904 audit gate replacing this task's fail-closed verification guard
- Depends on TASK-906 completion

## Requirements

1. Track constructor variables, apply them by kind, and add non-inverting constructor unification.
2. Preserve all non-goals and decision gates from the owning SPEC.
3. Add focused non-zero tests or an explicit docs/audit artifact matching the task type.
4. Avoid broadening later-task semantics by convenience or parser-only lowering.

## Implementation Summary

- Added TypeEnv source type-parameter kind tracking for proper type variables and constructor variables.
- Accepted fully applied constructor-variable applications such as `M<A>` in TASK-907-owned function, builtin-function, workflow type-signature, TypeEnv surface-lowering, and core type-expression seams.
- Rejected applying proper type variables as constructors, bare constructor variables in proper type positions, and wrong-arity constructor-variable applications with focused diagnostics.
- Preserved constructor-variable applications as constructor-variable carriers instead of nominal `Type::Constructor`/`CanonicalTypeExpr::NominalApp` lowering.
- Added structural, non-inverting unification for same-headed constructor-variable applications while rejecting incompatible closed spines and nominal-output inversion.
- Kept TASK-908+ interface/impl/proposition evidence and coherence surfaces fail-closed.

## File Targets

- TypeEnv kinding/lowering: `crates/ash-typeck/src/type_env.rs`, `crates/ash-typeck/src/types.rs`, `crates/ash-typeck/src/kind.rs`
- Core carrier consumers: `crates/ash-core/src/type_ir.rs`
- Focused tests:
  - `crates/ash-typeck/tests/task_907_constructor_variable_kinding.rs`
  - `crates/ash-typeck/tests/task_907_constructor_unification_non_inverting.rs`
- Audit source: `docs/plan/audits/TASK-904-hkt-audit-gate.md`

## TDD / Execution Steps

1. Re-read the referenced SPEC and this task file.
2. Add the smallest failing parser/typechecker/core/engine/doc test that proves this task's boundary.
3. Implement only this task's boundary; do not import later-task semantics.
4. Run the focused command set recorded in this task.
5. Update this task status, the owning PLAN row, PLAN-INDEX, and CHANGELOG only after verification evidence is fresh.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 16
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-typeck --test task_907_constructor_variable_kinding
  - cargo test -p ash-typeck --test task_907_constructor_unification_non_inverting
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
checklist:
  - [x] Focused tests are non-zero and pass
  - [x] cargo fmt --check passes
  - [x] git diff --check passes
  - [x] Status surfaces and CHANGELOG are reconciled if this task completes
```

### Evidence

- RED: `cargo test -p ash-typeck --test task_907_constructor_variable_kinding` failed before implementation because `TypeEnv::register_type_parameter_kind` did not exist.
- RED: `cargo test -p ash-typeck --test task_907_constructor_unification_non_inverting` failed before implementation because `Type::ConstructorVariableApp` did not exist.
- GREEN: `cargo test -p ash-typeck --test task_907_constructor_variable_kinding` passes 6 tests.
- GREEN: `cargo test -p ash-typeck --test task_907_constructor_unification_non_inverting` passes 5 tests.
- Regression: `cargo test -p ash-typeck --test task_906_hkt_fail_closed` passes 7 tests, with TASK-908-owned interface/impl/proposition guards still fail-closed.
- Required gates: `cargo fmt --check`, `git diff --check`, and `cargo check --workspace` pass.

## Dependencies for Next Task

This task produces its verified slice for later tasks in [PLAN-116](../PLAN-116-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md). Later tasks may cite this task's evidence but must not silently expand its scope.

## Notes

HKT is a cross-cutting type-system feature; do not implement as do-only magic.
