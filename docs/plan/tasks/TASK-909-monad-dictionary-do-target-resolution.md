# TASK-909: Route generalized do target resolution through `Monad<K>` evidence while preserving Act/Proc/Workflow bridge semantics

## Status: ✅ Complete

## Description

Route generalized do target resolution through `Monad<K>` evidence while preserving Act/Proc/Workflow bridge semantics

## Specification Reference

- [SPEC-067](../../spec/SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md)
- [PLAN-116](../PLAN-116-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md)

## Dependencies

- ✅ SPEC-067: spec packet exists
- ✅ PLAN-116: implementation plan exists
- Depends on TASK-904 audit gate replacing this task's fail-closed verification guard
- Depends on TASK-908 completion

## Requirements

1. Route generalized do target resolution through `Monad<K>` evidence while preserving Act/Proc/Workflow bridge semantics.
2. Preserve all non-goals and decision gates from the owning SPEC.
3. Add focused non-zero tests or an explicit docs/audit artifact matching the task type.
4. Avoid broadening later-task semantics by convenience or parser-only lowering.

## File Targets

- Do-target resolution: `crates/ash-typeck/src/do_target.rs`, `crates/ash-typeck/src/check_expr.rs`, `crates/ash-typeck/src/lib.rs`
- TypeEnv evidence lookup: `crates/ash-typeck/src/type_env.rs`
- Parser do target surface if diagnostics need span adjustment: `crates/ash-parser/src/parse_expr.rs`, `crates/ash-parser/src/surface.rs`
- Focused tests:
  - `crates/ash-typeck/tests/task_909_monad_do_target_resolution.rs`
  - `crates/ash-typeck/tests/task_909_act_proc_workflow_bridge_non_interference.rs`
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
  - cargo test -p ash-typeck --test task_909_monad_do_target_resolution
  - cargo test -p ash-typeck --test task_909_act_proc_workflow_bridge_non_interference
  - cargo test -p ash-typeck do_target
  - cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
checklist:
  - [x] Focused tests are non-zero and pass
  - [x] cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings passes
  - [x] cargo fmt --check passes
  - [x] git diff --check passes
  - [x] cargo check --workspace passes
  - [x] Status surfaces and CHANGELOG are reconciled if this task completes
```

## Evidence

- 2026-05-16 focused verification: `cargo test -p ash-typeck --test task_909_monad_do_target_resolution` passed 4 tests.
- 2026-05-16 focused verification: `cargo test -p ash-typeck --test task_909_act_proc_workflow_bridge_non_interference` passed 4 tests.
- 2026-05-16 focused regression: `cargo test -p ash-typeck do_target` passed 9 in-module do-target tests plus 2 matching integration tests, with no failures.
- 2026-05-16 required gates: `cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings`, `cargo fmt --check`, `git diff --check`, and `cargo check --workspace` passed.

## Dependencies for Next Task

This task produces its verified slice for later tasks in [PLAN-116](../PLAN-116-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md). Later tasks may cite this task's evidence but must not silently expand its scope.

## Notes

HKT is a cross-cutting type-system feature; do not implement as do-only magic.
