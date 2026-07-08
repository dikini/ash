# TASK-1977: Application Report Identity

**Status:** Complete
**Phase:** [PLAN-201 Semantic Cleanup Follow-up](../PLAN-201-SEMANTIC-CLEANUP-FOLLOWUP.md)
**Source audit:** [AUDIT-201 Semantic Removal Vs Rename](../audits/AUDIT-201-semantic-removal-vs-rename.md)

## Description

Retarget application-boundary report and admission identity fields away from workflow identity
vocabulary. Application reports should expose application identity over checked target entries, not
teach a separate workflow report schema.

## Requirements

- Rename public application-boundary report/failure/request identity fields from `workflow_id` to
  `application_id` in active engine/core APIs.
- Preserve existing identity semantics and serialization behavior where the underlying ID type is
  still `WorkflowId`.
- Update focused admission/completion/report tests to assert `application_id`.
- Tighten the Phase 201 removal gate so active application report paths cannot reintroduce
  workflow-id field vocabulary.
- Update Phase 201 audit/task evidence and changelog.

## TDD Steps

1. Add failing Phase 201 gate rows for `workflow_id` in active application report/admission paths.
2. Rename core runtime report/failure fields and engine request/boundary methods to
   `application_id`.
3. Update focused tests and row-admission helpers.
4. Run report/admission tests, the Phase 201 removal gate, and affected crate checks.

## Completion Checklist

- [x] `ApplicationFailure` and `ApplicationReport` expose `application_id`.
- [x] `ApplicationAdmissionRequest` and `AdmittedApplicationBoundary` expose `application_id`.
- [x] Focused engine/core tests assert application identity rather than workflow identity.
- [x] Phase 201 removal gate blocks reintroducing active report `workflow_id` fields.
- [x] `CHANGELOG.md` and Phase 201 audit/task evidence are updated.

## Evidence

- RED verification:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  failed on public application-boundary `workflow_id` field patterns before the rename.
- Retargeted `ApplicationFailure`, `ApplicationReport`, `ApplicationAdmissionRequest`, and
  `AdmittedApplicationBoundary` to expose `application_id`.
- Retargeted application report provenance notes from `execution_workflow_id` /
  `execution_parent_workflow_id` to `execution_application_id` /
  `execution_parent_application_id`.
- Verification:
  `cargo check -p ash-core -p ash-engine -p ash-cli --all-targets`;
  `cargo test -p ash-core --test task_714_workflow_boundary_carriers -- --nocapture`;
  `cargo test -p ash-core --test task_715_contract_evidence_schema -- --nocapture`;
  `cargo test -p ash-engine --test task_715_workflow_admission_red -- --nocapture`;
  `cargo test -p ash-engine --test task_716_workflow_completion_red -- --nocapture`;
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`.

## Notes

- `cargo test -p ash-engine --test task_1829_1830_1831_1832_1833_row_admission -- --nocapture`
  still fails on pre-existing provider metadata and old parser-entry fixture assumptions; those
  failures are not caused by the application identity rename and remain outside this slice.
