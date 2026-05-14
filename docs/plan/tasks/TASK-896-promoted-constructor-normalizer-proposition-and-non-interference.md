# TASK-896: Integrate promoted apps with type functions/propositions and prove runtime ADT/sealed-domain non-interference

## Status: 📝 Planned

## Description

Integrate promoted apps with type functions/propositions and prove runtime ADT/sealed-domain non-interference

## Specification Reference

- [SPEC-065](../../spec/SPEC-065-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md)
- [PLAN-114](../PLAN-114-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md)

## Dependencies

- ✅ SPEC-065: spec packet exists
- ✅ PLAN-114: implementation plan exists
- Depends on TASK-892 audit gate replacing this task's fail-closed verification guard
- Depends on TASK-895 completion

## Requirements

1. Integrate promoted apps with type functions/propositions and prove runtime ADT/sealed-domain non-interference.
2. Preserve all non-goals and decision gates from the owning SPEC.
3. Add focused non-zero tests or an explicit docs/audit artifact matching the task type.
4. Avoid broadening later-task semantics by convenience or parser-only lowering.

## File Targets

- Modify: `crates/ash-typeck/src/normalizer.rs`
- Modify: `crates/ash-typeck/src/solver.rs` / `crates/ash-typeck/src/constraint_checking.rs` as needed for proposition operands
- Add: `crates/ash-typeck/tests/task_896_promoted_constructor_integration.rs`
- Add: `crates/ash-engine/tests/task_896_promoted_constructor_non_interference.rs`

## TDD / Execution Steps

1. Re-read the referenced SPEC and this task file.
2. Add the smallest failing parser/typechecker/core/engine/doc test that proves this task's boundary.
3. Implement only this task's boundary; do not import later-task semantics.
4. Run the focused command set recorded in this task after the audit gate replaces any failing placeholder.
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
  - cargo test -p ash-typeck --test task_896_promoted_constructor_integration
  - cargo test -p ash-engine --test task_896_promoted_constructor_non_interference
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

This task produces its verified slice for later tasks in [PLAN-114](../PLAN-114-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md). Later tasks may cite this task's evidence but must not silently expand its scope.

## Notes

Promoted constructors are opt-in and distinct from sealed-domain markers and runtime constructors.
