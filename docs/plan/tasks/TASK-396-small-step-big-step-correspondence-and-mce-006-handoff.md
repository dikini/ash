# TASK-396: Small-Step / Big-Step Correspondence and MCE-006 Handoff

## Status: ✅ Complete

## Description

Package the correspondence-facing results of Phase 61 so the accepted MCE-005 backbone is explicitly tied back to `SPEC-004` and forward to MCE-006. This is documentation/spec-planning work only.

This task closes the remaining Phase 61 reporting/alignment work by:

- stating how small-step terminal configurations reconstruct the `SPEC-004` workflow outcomes;
- recording where traces, effects, obligations, provenance, and terminal success/rejection live in the stepwise model;
- updating MCE-006 so it consumes a fixed semantic target instead of an undefined upstream;
- updating MCE-007 and reporting/index docs so the remaining dependency weight is on interpreter/runtime alignment, not on backbone ambiguity.

## Specification Reference

- [SPEC-001: Intermediate Representation](../../spec/SPEC-001-IR.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [MCE-005: Small-Step Semantics](../../ideas/minimal-core/MCE-005-SMALL-STEP.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)
- [MCE-007: Full Layer Alignment](../../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md)
- [TASK-394: Small-Step Semantics Scope and Configuration Contract](TASK-394-small-step-semantics-scope-and-configuration-contract.md)
- [TASK-395: Canonical Workflow Small-Step Rule Set and Concurrency Semantics](TASK-395-canonical-workflow-small-step-rule-set-and-concurrency-semantics.md)

## Dependencies

- ✅ [TASK-394: Small-Step Semantics Scope and Configuration Contract](TASK-394-small-step-semantics-scope-and-configuration-contract.md)
- ✅ [TASK-395: Canonical Workflow Small-Step Rule Set and Concurrency Semantics](TASK-395-canonical-workflow-small-step-rule-set-and-concurrency-semantics.md)
- ✅ [TASK-393: Big-Step Semantics Alignment (MCE-004)](TASK-393-big-step-semantics-alignment.md)

## Requirements

### Functional Requirements

1. Revise `MCE-005` so it includes an explicit correspondence/handoff section for MCE-006.
2. Record how terminal `Returned(...)` / `Rejected(...)` configurations reconstruct the `SPEC-004` `Return(...)` / `Reject(...)` workflow outcomes.
3. Record explicitly how terminal semantic dimensions are preserved or reconstructed in the small-step model:
   - outcome/rejection,
   - obligations,
   - provenance,
   - effects,
   - traces.
4. Revise `MCE-006` so it consumes the frozen MCE-005 backbone and focuses on runtime/interpreter realization questions.
5. Revise `MCE-007` so it reflects that MCE-005 is materially defined and that the primary remaining dependency is MCE-006.
6. Update planning/reporting/index docs so Phase 61 is marked complete and MCE-005 is shown as accepted enough to unblock MCE-006.
7. Update `CHANGELOG.md` with a concise Phase 61 closeout entry.

### Non-Functional Requirements

1. Keep scope limited to documentation/spec alignment; no runtime implementation changes.
2. Preserve helper-contract and determinism-boundary language from `SPEC-004`.
3. Use repo-relative links throughout.
4. Mark complete only if the surrounding corpus is internally consistent.

## Completion Notes

- Added explicit terminal-outcome reconstruction and observability-carrier text to `docs/ideas/minimal-core/MCE-005-SMALL-STEP.md`.
- Reframed `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md` around the accepted MCE-005 backbone, with the remaining questions now stated as runtime/interpreter mapping questions.
- Updated `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md` so the remaining dependency weight is on MCE-006 rather than unresolved backbone ambiguity.
- Updated `docs/plan/PLAN-INDEX.md`, `docs/ideas/README.md`, `docs/ideas/IMPLEMENTABILITY-REPORT.md`, `docs/plans/2026-04-05-mce-005-small-step-semantics-plan.md`, and `CHANGELOG.md` so Phase 61 is complete in the planning/reporting corpus.

## Completion Checklist

- [x] MCE-005 records explicit big-step reconstruction and MCE-006 handoff
- [x] MCE-006 no longer reads as blocked on undefined small-step foundations
- [x] MCE-007 updated for the accepted Phase 61 backbone
- [x] Phase 61 planning/index/reporting docs mark TASK-394/395/396 complete
- [x] `CHANGELOG.md` updated

## Files

- Modify: `docs/ideas/minimal-core/MCE-005-SMALL-STEP.md`
- Modify: `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md`
- Modify: `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/ideas/README.md`
- Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `docs/plans/2026-04-05-mce-005-small-step-semantics-plan.md`
- Modify: `CHANGELOG.md`

## Non-goals

- No interpreter/runtime implementation changes
- No new proof artifact beyond documentation/planning correspondence packaging
- No reopening of MCE-004 or SPEC-004 decisions
