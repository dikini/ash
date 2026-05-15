# TASK-894: Add core promoted data-kind/constructor identities, type-level app carriers, and summary version contract

## Status: ✅ Complete

## Description

Add core promoted data-kind/constructor identities, type-level app carriers, and summary version contract

## Specification Reference

- [SPEC-065](../../spec/SPEC-065-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md)
- [PLAN-114](../PLAN-114-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md)

## Dependencies

- ✅ SPEC-065: spec packet exists
- ✅ PLAN-114: implementation plan exists
- Depends on TASK-892 audit gate replacing this task's fail-closed verification guard
- Depends on TASK-893 completion

## Requirements

1. Add core promoted data-kind/constructor identities, type-level app carriers, and summary version contract.
2. Preserve all non-goals and decision gates from the owning SPEC.
3. Add focused non-zero tests or an explicit docs/audit artifact matching the task type.
4. Avoid broadening later-task semantics by convenience or parser-only lowering.

## File Targets

- Modify: `crates/ash-core/src/type_ir.rs`
- Modify: `crates/ash-core/src/semantic_summary.rs`
- Modify: `crates/ash-core/src/lib.rs`
- Add: `crates/ash-core/tests/task_894_promoted_constructor_identities.rs`

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
  - cargo test -p ash-core --test task_894_promoted_constructor_identities
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

- Added core promoted data-kind and promoted constructor identity carriers distinct from runtime ADT constructor identities and sealed-domain marker identities.
- Added promoted-constructor application carriers in canonical and normal type IR, plus conservative downstream exhaustive-match handling that preserves structure or fails closed without TypeEnv registration/kinding semantics.
- Added V6 semantic-summary promoted data-kind/constructor/field carriers, summary cache-key participation, and version-contract rejection for pre-V6 summaries carrying promoted data-kind facts.
- Verification evidence: `cargo test -p ash-core --test task_894_promoted_constructor_identities` ran 4 focused tests; `cargo fmt --check`, `git diff --check`, and `cargo check --workspace` all passed after the final TASK-894 changes.
