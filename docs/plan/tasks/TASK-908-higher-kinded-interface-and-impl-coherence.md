# TASK-908: Register and resolve higher-kinded interface/impl evidence without overlap or output-directed selection

## Status: 📝 Planned

## Description

Register and resolve higher-kinded interface/impl evidence without overlap or output-directed selection

## Specification Reference

- [SPEC-067](../../spec/SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md)
- [PLAN-116](../PLAN-116-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md)

## Dependencies

- ✅ SPEC-067: spec packet exists
- ✅ PLAN-116: implementation plan exists
- Depends on TASK-904 audit gate replacing this task's fail-closed verification guard
- Depends on TASK-907 completion

## Requirements

1. Register and resolve higher-kinded interface/impl evidence without overlap or output-directed selection.
2. Preserve all non-goals and decision gates from the owning SPEC.
3. Add focused non-zero tests or an explicit docs/audit artifact matching the task type.
4. Avoid broadening later-task semantics by convenience or parser-only lowering.

## File Targets

- TypeEnv interface/impl coherence: `crates/ash-typeck/src/type_env.rs`, `crates/ash-typeck/src/types.rs`
- Semantic summary carriers/import: `crates/ash-core/src/semantic_summary.rs`, `crates/ash-engine/src/module_loader.rs`
- Focused tests:
  - `crates/ash-typeck/tests/task_908_hkt_interface_impl_coherence.rs`
  - `crates/ash-typeck/tests/task_908_hkt_evidence_lookup.rs`
  - `crates/ash-engine/tests/task_908_hkt_summary_non_interference.rs`
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
  - cargo test -p ash-typeck --test task_908_hkt_interface_impl_coherence
  - cargo test -p ash-typeck --test task_908_hkt_evidence_lookup
  - cargo test -p ash-engine --test task_908_hkt_summary_non_interference
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
