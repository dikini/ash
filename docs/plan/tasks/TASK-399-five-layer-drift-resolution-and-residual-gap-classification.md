# TASK-399: Five-Layer Drift Resolution and Residual Gap Classification (MCE-007)

## Status: ✅ Complete

## Description

Build on TASK-398’s ingested MCE-007 matrix by classifying what remaining five-layer misalignment is true residual drift versus merely documented, accepted partiality with an explicit owner. This task is documentation/planning/full-stack alignment work. It must take the current row-level Small-step → Interpreter classifications, assign owners to every still-open gap, and distinguish acceptable boundary ownership from issues that genuinely need new follow-on work.

## Specification Reference

- [SPEC-001: Intermediate Representation](../../spec/SPEC-001-IR.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [MCE-005: Small-Step Semantics](../../ideas/minimal-core/MCE-005-SMALL-STEP.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)
- [MCE-007: Full Layer Alignment](../../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md)
- [TASK-398: Runtime/Interpreter Correspondence Ingestion for MCE-007](TASK-398-runtime-interpreter-correspondence-ingestion-for-mce-007.md)

## Dependencies

- ✅ [TASK-398: Runtime/Interpreter Correspondence Ingestion for MCE-007](TASK-398-runtime-interpreter-correspondence-ingestion-for-mce-007.md)
- ✅ Frozen Phase 63 / MCE-006 evidence packet as consumed by TASK-398
- ✅ Accepted upstream MCE-004 / MCE-005 inputs

## Requirements

### Functional Requirements

1. Revisit every still-open MCE-007 matrix row and classify remaining issues into explicit residual-gap categories.
2. At minimum, distinguish:
   - accepted and closed,
   - closed but packaging/documentation can improve,
   - residual partiality with an accepted owner,
   - true residual drift requiring follow-on work.
3. Every non-closed issue must name an owner, such as:
   - later runtime cleanup,
   - future semantics/spec update,
   - MCE-007 closeout packaging,
   - explicit future follow-on task.
4. Make explicit which current gaps are simply accepted consequences of conservative evidence ownership and which are actual cross-layer tensions needing resolution.
5. Update the MCE-007 matrix/supporting notes so TASK-400 can consume one explicit residual-gap classification rather than re-deriving it.
6. Preserve the owner boundary:
   - do not reopen MCE-006,
   - do not invent new runtime theory,
   - do not reopen MCE-004 / MCE-005 accepted decisions.

### Non-Functional Requirements

1. Keep scope limited to docs/planning/full-stack alignment; no runtime redesign.
2. Use repo-relative links throughout.
3. Be conservative: partial runtime evidence alone is not automatically “drift” if the corpus already accepts it as follow-up-owned.
4. Make the owner table legible enough that TASK-400 can turn it into a stable closeout checklist.

## TDD Evidence

### Red

Before this task:

- TASK-398 ingests the frozen MCE-006 packet into row-level matrix classifications;
- but MCE-007 still does not separate accepted partiality from true residual drift with named owners;
- later closeout work would still need to infer which partial rows are merely accepted boundary ownership versus unresolved cross-layer issues.

### Green

This task is complete when:

- MCE-007 contains one explicit residual-gap classification layer over the current matrix;
- every open issue has an owner;
- accepted partiality is distinguished from true residual drift;
- TASK-400 can build on the resulting classification without revisiting MCE-006 evidence from scratch.

## Files

- Modify: `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Optional Modify: `docs/plans/2026-04-05-mce-007-full-layer-alignment-plan.md`
- Reference: `docs/plan/tasks/TASK-398-runtime-interpreter-correspondence-ingestion-for-mce-007.md`
- Reference: `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md`

## TDD Steps

### Step 1: Re-read the ingested matrix (Red)

Confirm that TASK-398 left MCE-007 with explicit row-level Small-step → Interpreter classifications but not yet one owner/classification layer for residual gaps.

### Step 2: Freeze the residual-gap categories

Define the categories MCE-007 will use to distinguish accepted partiality from real drift.

### Step 3: Assign owners

For every still-open issue, name the owner and say whether the issue is accepted or still needs follow-on work.

### Step 4: Verify GREEN

Expected pass condition:

- a reader can inspect MCE-007 and know which remaining partialities are acceptable/documented, which are packaging-only, and which are true residual gaps requiring follow-up.

## Completion Checklist

- [x] TASK-399 task file created
- [x] residual-gap categories frozen in MCE-007
- [x] non-closed issues assigned owners
- [x] accepted partiality distinguished from true residual drift
- [x] PLAN-INDEX updated
- [x] nearby reporting docs updated if needed

## Completion Notes

Completed 2026-04-05.

- Added a dedicated residual-gap classification layer to `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md` with four categories: `closed`, `packaging-only`, `accepted partiality`, and `true residual drift`.
- Classified all remaining MCE-007 row families using that layer and assigned explicit owners to every non-closed issue.
- Marked packaged big-step ↔ small-step correspondence and rejected-vs-runtime-failure subtype separation as owner-bound partiality rather than fresh five-layer drift claims.
- Marked the blocked/terminal/invalid runtime class boundary, authoritative cumulative `Ω` / `π` / `T` / `ε̂` packaging, retained completion-payload observation, and full helper-backed `Par` aggregation as the true residual drift set for downstream work.
- Made the sequencing / binding / branching row explicit as a mixed case: accepted local execution alignment plus one true residual drift dependency through missing authoritative cumulative-carrier packaging.
- Updated phase/index/reporting surfaces so TASK-400 can consume the frozen residual register directly.

## Notes

Important constraints:

- Do not relitigate Phase 63 evidence.
- Do not convert every partial row into a “drift” claim.
- Keep owner assignments explicit and stable.

Likely owner categories to keep visible:

- MCE-007 packaging/closeout work,
- later runtime cleanup work,
- future interpreter/runtime follow-on implementation,
- future spec/semantics clarification if a real cross-layer conflict is discovered.
