# TASK-404: Observable Preservation, Gap Classification, and MCE-007 Handoff (MCE-006)

## Status: ✅ Complete

## Description

Build on TASK-401 through TASK-403 by packaging the remaining MCE-006 runtime-correspondence evidence into one explicit observable-preservation and gap-classification closeout artifact for downstream MCE-007 ingestion. This task is documentation/planning/runtime-correspondence work. It must freeze what the current runtime preserves directly, what it only reconstructs or approximates, what remains missing, and what exact evidence packet TASK-398 / MCE-007 should consume without reopening Phase 63 scope.

## Specification Reference

- [SPEC-001: Intermediate Representation](../../spec/SPEC-001-IR.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [MCE-005: Small-Step Semantics](../../ideas/minimal-core/MCE-005-SMALL-STEP.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)
- [MCE-007: Full Layer Alignment](../../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md)
- [TASK-401: Runtime Carrier Inventory and Semantic Mapping Table](TASK-401-runtime-carrier-inventory-and-semantic-mapping-table.md)
- [TASK-402: Residual Control, Blocked-State, and Completion Realization](TASK-402-residual-control-blocked-state-and-completion-realization.md)
- [TASK-403: `Par` Interleaving, Branch-Local State, and Aggregation Correspondence](TASK-403-par-interleaving-branch-state-and-aggregation-correspondence.md)

## Dependencies

- ✅ [TASK-401: Runtime Carrier Inventory and Semantic Mapping Table](TASK-401-runtime-carrier-inventory-and-semantic-mapping-table.md)
- ✅ [TASK-402: Residual Control, Blocked-State, and Completion Realization](TASK-402-residual-control-blocked-state-and-completion-realization.md)
- ✅ [TASK-403: `Par` Interleaving, Branch-Local State, and Aggregation Correspondence](TASK-403-par-interleaving-branch-state-and-aggregation-correspondence.md)
- ✅ Accepted MCE-005 backbone in `docs/ideas/minimal-core/MCE-005-SMALL-STEP.md`
- 📝 Current interpreter/runtime evidence in `crates/ash-interp/`

## Requirements

### Functional Requirements

1. Freeze an explicit observable-preservation checklist covering at minimum:
   - return / reject outcome class,
   - blocked vs terminal vs invalid/runtime-failure status,
   - obligations (`Ω`),
   - provenance (`π`),
   - trace (`T`),
   - effect summary (`ε̂`).
2. For each observable, classify the current runtime story as one of:
   - directly preserved,
   - reconstructed / approximated,
   - weak / missing,
   - correspondence risk.
3. Freeze one explicit divergence taxonomy for MCE-006 closeout, distinguishing at minimum:
   - semantically safe indirection,
   - documentation gap,
   - correspondence risk,
   - contract-preserving follow-up required.
4. Produce one concise MCE-007 handoff packet that TASK-398 can ingest without reinterpreting all of MCE-006.
5. State clearly whether the current interpreter:
   - already realizes the accepted MCE-005 backbone for observable purposes,
   - realizes it only partially,
   - or still needs contract-preserving follow-up before MCE-007 can mark rows closed.
6. Record concrete evidence sources for each checklist row wherever current runtime support exists.

### Non-Functional Requirements

1. Keep scope limited to docs/planning/runtime-correspondence; no runtime redesign.
2. Do not reopen MCE-005 semantics or TASK-401 through TASK-403 conclusions.
3. Use repo-relative links throughout.
4. Be conservative: if `π`, `T`, `ε̂`, or completion-payload-style observables lack authoritative runtime carriers, say so directly rather than implying hidden support.

## TDD Evidence

### Red

Before this task:

- MCE-006 now has a frozen carrier baseline, control/blocking/completion correspondence story, and `Par` correspondence story;
- but Phase 63 still lacks one explicit observable-preservation checklist and one compact handoff packet for MCE-007;
- downstream MCE-007 work would still need to reinterpret multiple Phase 63 sections manually.

### Green

This task is complete when:

- MCE-006 contains one explicit observable-preservation and divergence-classification closeout section;
- every required observable is classified conservatively with evidence or an explicit gap label;
- the document includes one concise handoff packet for MCE-007 / TASK-398;
- Phase 63 can be treated as a frozen MCE-006 evidence bundle rather than an open exploratory note.

## Files

- Modify: `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Reference: `docs/plan/tasks/TASK-401-runtime-carrier-inventory-and-semantic-mapping-table.md`
- Reference: `docs/plan/tasks/TASK-402-residual-control-blocked-state-and-completion-realization.md`
- Reference: `docs/plan/tasks/TASK-403-par-interleaving-branch-state-and-aggregation-correspondence.md`
- Evidence source: current interpreter/runtime implementation files

## TDD Steps

### Step 1: Re-read the frozen Phase 63 baseline (Red)

Confirm what remains open after TASK-401 through TASK-403:

- observable preservation is still distributed across sections,
- divergence labels are not yet frozen in one final taxonomy,
- MCE-007 still lacks one concise ingestible handoff packet.

### Step 2: Freeze the observable checklist

For each required observable, document the strongest conservative claim the current runtime supports.

### Step 3: Freeze the divergence taxonomy

Distinguish safe indirection, documentation gaps, real correspondence risks, and contract-preserving follow-up work.

### Step 4: Freeze the MCE-007 handoff packet

Produce one compact section that TASK-398 can later consume directly.

### Step 5: Verify GREEN

Expected pass condition:

- a reader can tell, without rereading all earlier Phase 63 sections, what observables are preserved, what remains missing, and what MCE-007 should ingest from MCE-006.

## Completion Checklist

- [x] TASK-404 task file created
- [x] dedicated observable-preservation / gap-classification / handoff section added to `MCE-006-SMALL-STEP-IR.md`
- [x] required observable checklist completed
- [x] divergence taxonomy frozen
- [x] concise MCE-007 handoff packet added
- [x] PLAN-INDEX updated

## Completion Notes

Completed 2026-04-05.

- Added one explicit Phase 63 closeout section to [MCE-006](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md) covering observable preservation, divergence taxonomy, and a concise TASK-398 / MCE-007 handoff packet.
- Froze a conservative observable-preservation checklist for successful return class, rejected/non-success boundary, blocked vs terminal vs invalid/runtime-failure status, `Ω`, `π`, `T`, and `ε̂`, with concrete runtime evidence for supported rows and explicit weak/missing labels where no authoritative carrier exists.
- Recorded the final MCE-006 verdict for observable purposes: the current interpreter partially realizes the accepted MCE-005 backbone, but does not yet provide authoritative cumulative runtime carriers for `π`, `T`, `ε̂`, stronger terminal `Ω` packaging, or retained completion-payload-style observables.
- Updated planning/reporting surfaces so Phase 63 now reads as complete and MCE-007 can consume the frozen MCE-006 evidence packet without reopening TASK-401 through TASK-403 conclusions.

## Notes

Important constraints:

- Do not overstate preservation of `Ω`, `π`, `T`, or `ε̂` when the current runtime only partially or weakly carries them.
- Do not treat `Ok(Value)` / `Err(ExecError)` alone as automatically sufficient for all accepted semantic observables.
- Keep the handoff packet concise and consumable by MCE-007 rather than re-expanding Phase 63 theory.

Edge cases to keep visible:

- rejected vs runtime-failure outcome boundaries,
- blocked/suspended status versus terminal completion,
- `Par` value collation without helper-backed cumulative-state aggregation,
- missing authoritative cumulative carriers for `π`, `T`, and `ε̂`.
