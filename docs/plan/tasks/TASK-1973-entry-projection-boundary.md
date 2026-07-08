# TASK-1973: Application Result Projection Boundary

**Status:** Complete
**Phase:** [PLAN-201 Semantic Cleanup Follow-up](../PLAN-201-SEMANTIC-CLEANUP-FOLLOWUP.md)
**Source audit:** [AUDIT-201 Semantic Removal Vs Rename](../audits/AUDIT-201-semantic-removal-vs-rename.md)

## Description

Remove the runtime/engine first-class entry Proc projection boundary that preserved workflow
projection semantics under entry vocabulary. Replace it with target-facing application result or
report projection, or delete it if no current target Ash path needs a separate projection executor.

## Requirements

- Remove public interpreter and engine APIs that execute `WorkflowProcProjection` values.
- Remove or rewrite tests whose only purpose is proving old workflow Proc projection execution.
- Keep current application/report/result projection behavior through target runtime report APIs.
- Tighten the Phase 201 removal gate so stale entry-proc projection APIs cannot re-enter active
  runtime/engine code.
- Update Phase 201 audit/task evidence and changelog.

## TDD Steps

1. Add or retarget failing gate rows for active entry-proc projection terminology.
2. Delete or replace the stale projection executor and engine forwarding API.
3. Rewrite focused tests to prove current application/result projection behavior instead of old
   workflow Proc projection execution.
4. Run the Phase 201 removal gate and affected interpreter/engine checks.

## Completion Checklist

- [x] `ash-interp` no longer exports a public `WorkflowProcProjection` execution boundary.
- [x] `ash-engine` no longer forwards a first-class entry Proc projection executor.
- [x] Focused tests assert target application/result/report projection behavior.
- [x] Phase 201 removal gate blocks reintroducing stale entry-proc projection APIs.
- [x] `CHANGELOG.md` and Phase 201 audit/task evidence are updated.

## Evidence

- RED verification:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  failed on the active `entry_projection`, `execute_entry_proc_projection`,
  `unsupported_entry_proc_projection_message`, and `FirstClassEntryProjectionExecutionUnsupported`
  runtime/engine paths before deletion.
- Removed `crates/ash-interp/src/entry_projection.rs` and the public `ash-interp` re-exports for
  the old `WorkflowProcProjection<Value>` executor.
- Removed `Engine::execute_entry_proc_projection` and the engine import of
  `WorkflowProcProjection`.
- Deleted the focused interpreter and engine tests that existed only to prove old entry Proc
  projection execution.
- Replacement target behavior is covered by application boundary tests:
  `cargo test -p ash-engine --test task_715_workflow_admission_red -- --nocapture` and
  `cargo test -p ash-engine --test task_716_workflow_completion_red -- --nocapture`.
- Removal gate:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`.
