# TASK-906: Parse kinded binders in interfaces, impls, functions, type functions, and propositions at audited sites

## Status: 📝 Planned

## Description

Parse kinded binders in interfaces, impls, functions, type functions, and propositions at audited sites

## Specification Reference

- [SPEC-067](../../spec/SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md)
- [PLAN-116](../PLAN-116-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md)

## Dependencies

- ✅ SPEC-067: spec packet exists
- ✅ PLAN-116: implementation plan exists
- Depends on TASK-904 audit gate replacing this task's fail-closed verification guard
- Depends on TASK-905 completion

## Requirements

1. Parse kinded binders in interfaces, impls, functions, type functions, and propositions at audited sites.
2. Preserve all non-goals and decision gates from the owning SPEC.
3. Add focused non-zero tests or an explicit docs/audit artifact matching the task type.
4. Avoid broadening later-task semantics by convenience or parser-only lowering.

## File Targets

- Parser surface carriers: `crates/ash-parser/src/surface.rs`
- Parser entrypoints: `crates/ash-parser/src/parse_module.rs`, `crates/ash-parser/src/parse_workflow.rs`, `crates/ash-parser/src/parse_expr.rs`
- Parser lowering adaptation if carrier shapes change: `crates/ash-parser/src/lower.rs`
- Focused tests:
  - `crates/ash-parser/tests/task_906_hkt_kinded_binder_surface.rs`
  - `crates/ash-parser/tests/task_906_hkt_non_interference.rs`
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
  - cargo test -p ash-parser --test task_906_hkt_kinded_binder_surface
  - cargo test -p ash-parser --test task_906_hkt_non_interference
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
checklist:
  - [ ] Focused tests are non-zero and pass
  - [ ] cargo fmt --check passes
  - [ ] git diff --check passes
  - [ ] Status surfaces and CHANGELOG are reconciled if this task completes
```

## Dependencies for Next Task

This task produces its verified slice for later tasks in [PLAN-116](../PLAN-116-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md). Later tasks may cite this task's evidence but must not silently expand its scope.

## Notes

HKT is a cross-cutting type-system feature; do not implement as do-only magic.
