# TASK-895: Register promoted identities in TypeEnv and validate kind/domain/source-ADT constraints

## Status: ✅ Complete

## Description

Register promoted identities in TypeEnv and validate kind/domain/source-ADT constraints

## Specification Reference

- [SPEC-065](../../spec/SPEC-065-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md)
- [PLAN-114](../PLAN-114-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md)

## Dependencies

- ✅ SPEC-065: spec packet exists
- ✅ PLAN-114: implementation plan exists
- Depends on TASK-892 audit gate replacing this task's fail-closed verification guard
- Depends on TASK-894 completion

## Requirements

1. Register promoted identities in TypeEnv and validate kind/domain/source-ADT constraints.
2. Preserve all non-goals and decision gates from the owning SPEC.
3. Add focused non-zero tests or an explicit docs/audit artifact matching the task type.
4. Avoid broadening later-task semantics by convenience or parser-only lowering.

## File Targets

- Modify: `crates/ash-typeck/src/type_env.rs`
- Modify: `crates/ash-typeck/src/lib.rs` only if public registration entrypoints need wiring
- Add: `crates/ash-typeck/tests/task_895_promoted_constructor_registration.rs`

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
  - cargo test -p ash-typeck --test task_895_promoted_constructor_registration
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
checklist:
  - [x] Focused tests are non-zero and pass
  - [x] cargo fmt --check passes
  - [x] git diff --check passes
  - [x] Status surfaces and CHANGELOG are reconciled if this task completes
```

## Dependencies for Next Task

This task produces its verified slice for later tasks in [PLAN-114](../PLAN-114-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md). Later tasks may cite this task's evidence but must not silently expand its scope.

## Notes

Promoted constructors are opt-in and distinct from sealed-domain markers and runtime constructors.

## Completion Notes

- Added TypeEnv registration tables and lookup APIs for promoted data-kind summaries, promoted constructor summaries, and checked promoted-constructor kind/domain metadata.
- Registration is transactional through staged semantic-summary registration and validates public V6 promoted data-kind summaries against exposed source ADTs.
- Validation now rejects wrong source-ADT parents, omitted or out-of-source-order constructors, payload-kind/field-count mismatches, non-Type fields, missing or unknown promoted field domains, source field type/domain mismatches, and record field-name drift.
- Verification evidence: `cargo test -p ash-typeck --test task_895_promoted_constructor_registration` ran 9 focused tests; `cargo fmt --check`, `git diff --check`, and `cargo check --workspace` all passed after the final TASK-895 changes.
