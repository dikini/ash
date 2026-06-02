# TASK-1004: Workflow and operational binder irrefutability enforcement

## Status: 📝 Planned

## Description

Enforce irrefutable patterns for workflow-level binders such as workflow `let`, observe result binding, spawn/split binding, and loop element binding where the live audit confirms typed binders exist.

## Specification Reference

- [SPEC-076](../../spec/SPEC-076-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)
- [PLAN-126](../PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)

## Dependencies

- ✅ TASK-1000 packet exists
- ✅ TASK-1001 audit gate completed and patched this task with focused commands
- 📝 TASK-1002 shared irrefutability API must be implemented before this task wires workflow/operational binders

## Requirements

1. Preserve SPEC-076 non-goals and decision gates.
2. Add RED tests for each live source-level, lowered-only, and core-only binder using a refutable sum/list/literal pattern, including yield-arm lowering and any core spawn/split patterns identified by TASK-1001.
3. Wire checks at the semantic type-checking boundary, not only parser or lowering.
4. Keep runtime pattern-failure variants defensive but unreachable for checked binder cases, using TASK-1001's exact refreshed names.
5. Document any binder whose type is unavailable and add a blocked/deferred diagnostic instead of guessing.

## File Targets

- Modify candidates: `crates/ash-typeck/src/lib.rs`, `crates/ash-typeck/src/check_expr.rs`, `crates/ash-typeck/src/error.rs`, `crates/ash-parser/src/lower.rs`
- Test: `crates/ash-typeck/tests/task_1004_workflow_binder_irrefutability.rs`

## TDD / Execution Steps

1. Stop if this file still contains the fail-closed TASK-1001 verification guard.
2. Write RED tests named by TASK-1001.
3. Implement the smallest semantic change for this task only.
4. Run focused tests and required workspace checks.
5. Request independent review before marking complete.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 20
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - bash -lc "cargo test -p ash-typeck --test task_1004_workflow_binder_irrefutability -- --list | rg 'workflow_let_rejects_refutable_sum_literal_and_list_patterns|observe_binding_rejects_refutable_pattern|orient_binding_either_rejects_or_documents_lowering_defer|for_binder_rejects_refutable_item_pattern|yield_arms_reject_or_document_current_lowered_binder_semantics|core_spawn_pattern_rejects_refutable_instance_pattern|core_split_pattern_rejects_refutable_tuple_pattern|receive_stream_pattern_remains_selective_not_irrefutable' && cargo test -p ash-typeck --test task_1004_workflow_binder_irrefutability"
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
checklist:
  - [x] TASK-1001 replaced the fail-closed guard
  - [ ] RED tests fail before implementation and pass after implementation
  - [ ] Scope did not expand beyond SPEC-076
  - [ ] Diagnostics are asserted where required
```

## Dependencies for Next Task

This task produces its verified slice for later tasks in [PLAN-126](../PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md). Later tasks may cite this task's evidence but must not silently expand its scope.

## Notes

Workflow/typeck binder semantics
