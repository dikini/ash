# TASK-943: Phase 123 Follow-Up Child Admission and Status Drift

**Status:** ✅ Complete
**Phase:** 123 follow-up remediation
**Priority:** High
**Type:** Semantic/Docs

## Context

Phase 122/123 post-merge review found one remaining child-process authority drift after TASK-942: a spawned child with an explicitly empty inherited admitted binding list could be repopulated from `RuntimeState::admitted_capability_binding_ids()`. That made globally admitted host bindings available to children that did not inherit those binding IDs.

Phase 123 status surfaces must also cite the post-merge follow-up tasks honestly so SPEC-069/SPEC-070 Implemented MVP status is tied to TASK-941 closeout plus TASK-942/TASK-943 remediation evidence.

## Requirements

1. Add a RED regression in `crates/ash-interp/tests/runtime_action_control.rs` proving a spawned child without an inherited binding ID cannot execute a globally admitted host provider binding.
2. The regression must register a deploy provider, admit `workflow-deploy -> deploy` with grant `deploy.deploy` into `RuntimeState`, spawn the child without passing the binding ID, and assert the provider call count remains zero while the child fails.
3. Preserve positive admitted-child behavior when the binding ID is explicitly inherited.
4. Fix only the authority projection bug: process/spawned-child contexts must treat an empty admitted binding list as explicit empty authority, while non-process legacy execution paths may keep ambient provider behavior.
5. Update PLAN-119, PLAN-INDEX Phase 123, SPEC-069, SPEC-070, TASK-941 audit evidence, and CHANGELOG so source-of-truth status surfaces include TASK-943.

## TDD Steps

1. Create the TASK-943 task file and reopen Phase 123 status while remediation is in progress.
2. Add the focused child-admission regression and verify it fails before implementation.
3. Implement the minimal executor fix in `crates/ash-interp/src/execute.rs`.
4. Re-run the RED test, positive admitted-child tests, and requested focused suites.
5. Mark TASK-943 complete only after focused verification and formatting pass.

## Verification Checklist

- [x] RED regression fails before implementation.
- [x] `cargo test -p ash-interp --test runtime_action_control -- --nocapture` passes.
- [x] `cargo test -p ash-interp --test task_736_capability_binding_admission -- --nocapture` passes.
- [x] `cargo test -p ash-interp --test act_env_runtime_boundary -- --nocapture` passes.
- [x] `cargo test -p ash-engine --test task_715_workflow_admission_red -- --nocapture` passes.
- [x] `cargo test -p ash-cli --test alpha_ashd_child_failure_trace -- --nocapture` passes.
- [x] `cargo fmt --check` passes.
- [x] CHANGELOG and Phase 123 status surfaces cite TASK-943 honestly.

## Completion Checklist

- [x] Child process empty-admission authority does not fall back to globally admitted runtime bindings.
- [x] Explicitly inherited admitted child binding behavior remains green.
- [x] SPEC-069/SPEC-070 implementation task headers cite through TASK-943.
- [x] TASK-941 closeout audit has a post-merge TASK-942/TASK-943 addendum.
- [x] No commits are created by this task.

## Verification Evidence

- RED: reverting `crates/ash-interp/src/execute.rs` while keeping the TASK-943 regression failed with provider call count `left: 1`, `right: 0`.
- GREEN focused verification was rerun on the final diff by the main agent before commit.
