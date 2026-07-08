# TASK-1975: Function Body Runtime Registry

**Status:** Complete
**Phase:** [PLAN-201 Semantic Cleanup Follow-up](../PLAN-201-SEMANTIC-CLEANUP-FOLLOWUP.md)
**Source audit:** [AUDIT-201 Semantic Removal Vs Rename](../audits/AUDIT-201-semantic-removal-vs-rename.md)

## Description

Refactor the runtime callable-entry registry so it is clearly an implementation cache for target
function bodies, not a separate workflow/callable-entry semantic category.

## Requirements

- Rename active runtime and engine registry APIs away from callable-entry terminology.
- Store registered target function bodies with parameter metadata.
- Preserve target function call behavior for big-step and small-step execution.
- Keep direct Core test construction possible without exposing stale public vocabulary.
- Update Phase 201 evidence and changelog.

## TDD Steps

1. Add or retarget failing tests/gates for stale callable-entry registry terminology.
2. Refactor runtime state, engine helpers, and focused tests to function-body registry vocabulary.
3. Verify big-step, small-step, and engine runtime-boundary function-call tests still pass.
4. Run the Phase 201 removal gate and affected crate checks.

## Completion Checklist

- [x] Runtime registry APIs use function-body vocabulary.
- [x] Focused big-step and small-step tests prove registered function bodies execute through the
      same function-call path.
- [x] Engine integration tests no longer expose callable-entry registry vocabulary.
- [x] Phase 201 removal gate blocks reintroducing the stale registry vocabulary.
- [x] `CHANGELOG.md` and Phase 201 audit/task evidence are updated.

## Evidence

- Added Phase 201 gate rows for stale callable-entry registry terms:
  `RegisteredCallableEntry`, `callable_entries`, `register_callable_entry`,
  `blocking_register_callable_entry`, `callable_entry`, and engine registration helpers.
- RED verification:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  failed on the active runtime/engine callable-entry registry names before the refactor.
- Retargeted `RuntimeState` from `RegisteredCallableEntry` / `callable_entries` to
  `RegisteredFunctionBody` / `function_bodies`, with async and blocking
  `register_function_body` helpers.
- Retargeted engine test helpers and focused tests to function-body vocabulary.
- Added provider operation metadata to the touched task-local fixture so focused runtime-boundary
  verification still goes through explicit provider authoring metadata.
- Verification:
  `cargo test -p ash-interp function_body -- --nocapture`;
  `cargo test -p ash-interp core_call --lib -- --nocapture`;
  `cargo test -p ash-engine --test runtime_boundary_visibility engine_execute_core_workflow_calls_registered_function_body -- --nocapture`;
  `cargo test -p ash-engine --test runtime_boundary_visibility engine_execute_core_workflow_rejects_callable_arity_mismatch -- --nocapture`;
  `cargo test -p ash-engine --test task_1898_dynamic_contract_runtime_checks -- --nocapture`;
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`.
