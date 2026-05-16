# TASK-910: Add Functor/Applicative/Monad diagnostics, acceptance, and non-interference matrix

## Status: ✅ Complete

## Description

Add Functor/Applicative/Monad diagnostics, acceptance, and non-interference matrix

## Specification Reference

- [SPEC-067](../../spec/SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md)
- [PLAN-116](../PLAN-116-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md)

## Dependencies

- ✅ SPEC-067: spec packet exists
- ✅ PLAN-116: implementation plan exists
- Depends on TASK-904 audit gate replacing this task's fail-closed verification guard
- Depends on TASK-909 completion

## Requirements

1. Add Functor/Applicative/Monad diagnostics, acceptance, and non-interference matrix.
2. Preserve all non-goals and decision gates from the owning SPEC.
3. Add focused non-zero tests or an explicit docs/audit artifact matching the task type.
4. Avoid broadening later-task semantics by convenience or parser-only lowering.

## File Targets

- Parser diagnostics/tests: `crates/ash-parser/src/error.rs`, `crates/ash-parser/src/parse_module.rs`, `crates/ash-parser/src/parse_expr.rs`
- TypeEnv diagnostics/tests: `crates/ash-typeck/src/type_env.rs`, `crates/ash-typeck/src/do_target.rs`, `crates/ash-typeck/src/check_expr.rs`, `crates/ash-typeck/src/diagnostic.rs`
- Engine non-interference/tests: `crates/ash-engine/src/module_loader.rs`
- Acceptance artifact: `docs/plan/audits/TASK-910-hkt-acceptance-matrix.md`
- Focused tests:
  - `crates/ash-parser/tests/task_910_hkt_diagnostics_surface.rs`
  - `crates/ash-typeck/tests/task_910_hkt_acceptance_matrix.rs`
  - `crates/ash-engine/tests/task_910_hkt_summary_non_interference.rs`
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
  - cargo test -p ash-parser --test task_910_hkt_diagnostics_surface
  - cargo test -p ash-typeck --test task_910_hkt_acceptance_matrix
  - cargo test -p ash-engine --test task_910_hkt_summary_non_interference
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
checklist:
  - [x] Focused tests are non-zero and pass
  - [x] cargo fmt --check passes
  - [x] git diff --check passes
  - [x] Status surfaces and CHANGELOG are reconciled if this task completes
```

## Evidence

- 2026-05-16 RED evidence: `cargo test -p ash-typeck --test task_910_hkt_acceptance_matrix` initially failed 1 of 10 tests because `impl <E : *> Monad<Result<_, E>> {}` was lowered as a proper `Result<_, E>` type instead of a SPEC-066 partial constructor shape.
- 2026-05-16 remediation RED evidence: `cargo test -p ash-parser --test task_910_hkt_diagnostics_surface` initially failed 5 of 9 tests because parser `_` holes were accepted in ordinary function, interface, proposition, resource/capability, and associated type binding positions.
- 2026-05-16 remediation RED evidence: `cargo test -p ash-parser --test task_910_hkt_diagnostics_surface underscore_prefixed_type_names_are_not_treated_as_holes` initially failed 1 of 1 test because the impl-head hole recognizer treated `_M` as a hole prefix instead of an underscore-prefixed type name.
- 2026-05-16 focused verification: `cargo test -p ash-parser --test task_910_hkt_diagnostics_surface` passed 10 tests.
- 2026-05-16 focused verification: `cargo test -p ash-typeck --test task_910_hkt_acceptance_matrix` passed 10 tests.
- 2026-05-16 focused verification: `cargo test -p ash-engine --test task_910_hkt_summary_non_interference` passed 1 test.
- 2026-05-16 relevant regression verification: `cargo test -p ash-parser --test task_900_type_hole_surface` passed 4 tests; `cargo test -p ash-parser --test task_906_hkt_kinded_binder_surface` passed 5 tests; `cargo test -p ash-parser --test task_906_hkt_non_interference` passed 3 tests; `cargo test -p ash-typeck --test task_902_do_target_partial_application` passed 6 tests; `cargo test -p ash-typeck --test task_908_hkt_interface_impl_coherence` passed 4 tests; `cargo test -p ash-typeck --test task_908_hkt_evidence_lookup` passed 3 tests; `cargo test -p ash-typeck --test task_909_monad_do_target_resolution` passed 4 tests; `cargo test -p ash-engine --test task_908_hkt_summary_non_interference` passed 1 test.
- 2026-05-16 required gates: `cargo fmt --check`, `git diff --check`, and `cargo check --workspace` passed.

## Acceptance Artifact

- [TASK-910 HKT acceptance and non-interference matrix](../audits/TASK-910-hkt-acceptance-matrix.md)

## Dependencies for Next Task

This task produces its verified slice for later tasks in [PLAN-116](../PLAN-116-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md). Later tasks may cite this task's evidence but must not silently expand its scope.

## Notes

HKT is a cross-cutting type-system feature; do not implement as do-only magic.
