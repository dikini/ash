# TASK-393: Big-Step Semantics Alignment (MCE-004)

## Status: ✅ Complete

## Description

Consolidate the now-resolved MCE-004 alignment decisions into the formal planning/docs corpus so the repository no longer treats big-step semantics alignment as an open exploratory gap. This is a documentation and planning task only: it records the settled relationship between surface syntax, canonical IR, and the big-step semantics already established in `SPEC-001` and `SPEC-004`, especially after TASK-350 completed the proof-shaped semantic backbone.

This task closes the loop between:

- [MCE-004: Big-Step Semantics Alignment](../../ideas/minimal-core/MCE-004-BIG-STEP-ALIGNMENT.md)
- [MCE-002 Audit Report](../../ideas/minimal-core/MCE-002-IR-AUDIT-REPORT.md)
- [TASK-350: Revise SPEC-004 to Complete Big-Step Core Semantics](TASK-350-revise-spec-004-to-complete-big-step-core-semantics.md)
- [SPEC-001: Intermediate Representation](../../spec/SPEC-001-IR.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [Parser-to-Core Lowering Contract](../../reference/parser-to-core-lowering-contract.md)

## Requirements

### Functional Requirements

1. Create a formal task record for MCE-004 under `docs/plan/tasks/` so the completed alignment work is tracked in the plan corpus.
2. Update the planning index to add a dedicated phase for MCE-004 after Phase 59.
3. Revise the MCE-004 exploration so it no longer frames the main alignment questions as unresolved.
4. Record the settled surface → IR → semantics contract explicitly:
   - surface syntax lowers to canonical IR first;
   - `SPEC-004` defines runtime meaning for canonical IR forms, not surface syntax directly.
5. Record the settled form/semantics decisions with repo-relative references:
   - `Workflow::Seq` remains primitive and is not rewritten to `Let`;
   - `Par` combines all-success branch effects by join and uses helper-backed concurrent aggregation for obligations/provenance;
   - spawned child completion seals the child's authoritative terminal obligation/provenance/effect state into its own `CompletionPayload`;
   - `Expr::Match` remains a primitive core expression and `if let` lowers to `Expr::Match` with a wildcard fallback arm.
6. Update ideas index/reporting docs so MCE-004 is shown as accepted/resolved rather than drafting.
7. Update `CHANGELOG.md` for the documentation/planning closeout.

### Non-Functional Requirements

1. Keep scope limited to documentation/spec alignment; no Rust runtime or interpreter changes.
2. Use repo-relative links and references throughout.
3. Keep the recorded decisions aligned with the current normative corpus rather than introducing new semantic design.
4. Mark the task complete only if all referenced docs are updated coherently.

## Completion Notes

- Added this missing task file so MCE-004 has a formal plan record.
- Added Phase 60 in `docs/plan/PLAN-INDEX.md` and marked it complete with `TASK-393`.
- Promoted `docs/ideas/minimal-core/MCE-004-BIG-STEP-ALIGNMENT.md` from drafting to accepted, with a concise resolved alignment summary.
- Updated idea-index/reporting docs to show that the former MCE-004 gap analysis has been superseded by settled spec and lowering contracts.
- Recorded that TASK-350 already supplied the explicit workflow/expression/pattern judgments and helper contracts that MCE-004 originally called for.

## Completion Checklist

- [x] `docs/plan/tasks/TASK-393-big-step-semantics-alignment.md` created
- [x] `docs/plan/PLAN-INDEX.md` updated with a dedicated complete phase and task entry
- [x] `docs/ideas/minimal-core/MCE-004-BIG-STEP-ALIGNMENT.md` updated to accepted/resolved status
- [x] `docs/ideas/README.md` updated to show MCE-004 as accepted and linked to TASK-393
- [x] `docs/ideas/IMPLEMENTABILITY-REPORT.md` updated to reflect completion/alignment
- [x] `CHANGELOG.md` updated

## Dependencies

- ✅ [TASK-350: Revise SPEC-004 to Complete Big-Step Core Semantics](TASK-350-revise-spec-004-to-complete-big-step-core-semantics.md)
- ✅ [TASK-370: IR Core Forms Audit](TASK-370-ir-core-forms-audit.md)

## Non-goals

- No edits to Rust crates or runtime behavior
- No new semantic forms beyond those already specified in `SPEC-001` and `SPEC-004`
- No small-step semantics work
