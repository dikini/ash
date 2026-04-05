# TASK-398: Runtime/Interpreter Correspondence Ingestion for MCE-007

## Status: ✅ Complete

## Description

Consume the frozen Phase 63 / MCE-006 runtime-evidence packet into the MCE-007 five-layer alignment matrix. This task is documentation/planning/full-stack alignment work. It must import, classify, and summarize the accepted MCE-006 small-step → interpreter evidence by construct family without inventing new runtime theory or reopening MCE-006 ownership boundaries.

## Specification Reference

- [SPEC-001: Intermediate Representation](../../spec/SPEC-001-IR.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [MCE-005: Small-Step Semantics](../../ideas/minimal-core/MCE-005-SMALL-STEP.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)
- [MCE-007: Full Layer Alignment](../../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md)
- [TASK-397: Five-Layer Alignment Matrix and Closure Contract](TASK-397-five-layer-alignment-matrix-and-closure-contract.md)
- [TASK-404: Observable Preservation, Gap Classification, and MCE-007 Handoff](TASK-404-observable-preservation-gap-classification-and-mce-007-handoff.md)

## Dependencies

- The MCE-007 matrix/closure scaffold from [TASK-397: Five-Layer Alignment Matrix and Closure Contract](TASK-397-five-layer-alignment-matrix-and-closure-contract.md) is used here as design guidance, but TASK-397 itself remains separately tracked and is not required to be completed before this evidence-ingestion step.
- ✅ Phase 63 / MCE-006 runtime correspondence corpus:
  - [TASK-401](TASK-401-runtime-carrier-inventory-and-semantic-mapping-table.md)
  - [TASK-402](TASK-402-residual-control-blocked-state-and-completion-realization.md)
  - [TASK-403](TASK-403-par-interleaving-branch-state-and-aggregation-correspondence.md)
  - [TASK-404](TASK-404-observable-preservation-gap-classification-and-mce-007-handoff.md)
- ✅ Accepted upstream MCE-004 / MCE-005 inputs

## Requirements

### Functional Requirements

1. Update the MCE-007 verification matrix so the Small-step → Interpreter column is populated per construct family using the frozen Phase 63 evidence packet.
2. For each construct-family row, classify the Small-step → Interpreter state as one of:
   - closed,
   - closed with residual note,
   - partial / follow-up required.
3. Import at minimum the following MCE-006 evidence areas into the matrix and supporting notes:
   - workflow-configuration to runtime-structure mapping,
   - blocked/suspended-state realization,
   - `Par` interleaving / branch-state / aggregation realization,
   - observable-preservation verdict,
   - spawn/completion control-authority and retained-completion limits.
4. Add concise per-row runtime notes explaining why a row is not fully closed when residual issues remain.
5. Preserve the owner boundary:
   - TASK-398 may summarize and classify MCE-006 evidence,
   - but must not invent replacement runtime theory or reopen MCE-006 closeout conclusions.
6. Update nearby planning/reporting surfaces so they reflect that MCE-007 now has ingested Phase 63 evidence rather than merely waiting on it.

### Non-Functional Requirements

1. Keep scope limited to docs/planning/full-stack alignment; no runtime redesign.
2. Use repo-relative links throughout.
3. Be conservative: do not mark a row closed if the runtime evidence remains partial for cumulative carriers, retained completion payloads, or blocked/invalid distinction.
4. Keep the ingestion readable enough that TASK-399 can build on the resulting matrix without re-reading all of MCE-006.

## TDD Evidence

### Red

Before this task:

- MCE-007 acknowledges that Phase 63 evidence exists;
- but its matrix still does not ingest that evidence explicitly by construct family;
- downstream closeout work would still need to reconstruct runtime status from MCE-006 manually.

### Green

This task is complete when:

- the MCE-007 matrix explicitly reflects Phase 63 evidence in the Small-step → Interpreter column;
- each row is classified conservatively as closed, closed with residual note, or partial / follow-up required;
- MCE-007 contains concise runtime notes derived from the MCE-006 handoff packet;
- the corpus no longer treats Phase 63 evidence as merely awaiting ingestion.

## Files

- Modify: `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Optional Modify: `docs/plans/2026-04-05-mce-007-full-layer-alignment-plan.md`
- Reference: `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md`
- Reference: `docs/plan/tasks/TASK-404-observable-preservation-gap-classification-and-mce-007-handoff.md`

## TDD Steps

### Step 1: Re-read the frozen handoff packet (Red)

Confirm that MCE-006 now provides:

- carrier mapping,
- control/blocking correspondence,
- `Par` correspondence,
- observable-preservation verdict,
- concise TASK-398 ingestion guidance.

### Step 2: Populate the matrix

Replace “evidence available / partial ingestion still needed” placeholders with actual Small-step → Interpreter row classifications and short rationale.

### Step 3: Freeze residual notes

For each non-closed row, record what remains partial and why.

### Step 4: Verify GREEN

Expected pass condition:

- a reader can inspect MCE-007 and understand the current Small-step → Interpreter state without re-deriving it from Phase 63.

## Completion Checklist

- [x] TASK-398 task file created
- [x] MCE-007 matrix Small-step → Interpreter column populated per construct family
- [x] per-row runtime notes added
- [x] row statuses classified conservatively
- [x] PLAN-INDEX updated
- [x] nearby reporting docs updated if needed

## Completion Notes

Completed 2026-04-05.

- Replaced the old MCE-007 Small-step → Interpreter placeholders with row-level ingestion derived from the frozen MCE-006 Phase 63 packet.
- Classified each construct-family row conservatively using the TASK-398 vocabulary. In this execution, every current row landed in the `partial / follow-up required` bucket, with stronger direct evidence for ordinary residual execution and pattern-driven control than for receive/blocking, `Par`, capability/obligation observables, and spawn/completion retention.
- Imported the required evidence areas into MCE-007: workflow-configuration mapping, blocked/suspended realization, `Par` interleaving and aggregation realization, observable-preservation verdict, and spawn/completion control-authority versus retained-completion limits.
- Updated planning/reporting surfaces so MCE-007 no longer reads as merely waiting for Phase 63 ingestion.

## Notes

Important constraints:

- Do not reopen MCE-006 conclusions.
- Do not infer hidden runtime carriers from nearby utility types.
- Do not mark rows closed where MCE-006 explicitly says partial/follow-up required.

Likely partial rows to keep visible unless evidence improves:

- blocked vs terminal vs invalid as one authoritative runtime class,
- retained completion payload/state observation,
- authoritative cumulative `Ω`, `π`, `T`, and `ε̂` packaging,
- full helper-backed concurrent cumulative-state aggregation for `Par`.
