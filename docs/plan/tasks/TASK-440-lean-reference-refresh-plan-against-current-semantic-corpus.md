# TASK-440: Lean Reference Refresh Plan Against Current Semantic Corpus

## Status: 📝 Planned

## Description

Refresh the Lean/reference implementation plan against the current canonical semantic corpus so future Lean work no longer depends on stale phase assumptions or older semantic authority boundaries. This task should update the Lean/reference planning story to target the accepted big-step and small-step corpus, the Phase 67 implementation-conformance contract, and the canonical corpus/result-format work instead of older ADT-only or pre-`SPEC-025` assumptions.

This remains planning/reference work only.

## Specification Reference

- [SPEC-001: Intermediate Representation](../../spec/SPEC-001-IR.md)
- [SPEC-003: Type System](../../spec/SPEC-003-TYPE-SYSTEM.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [SPEC-021: Runtime Observable Behavior](../../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md)
- [SPEC-025: Small-Step Operational Semantics](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md)
- [Formalization Boundary and Proof Targets](../../reference/formalization-boundary.md)
- [TASK-428: Implementation Conformance Contract](TASK-428-implementation-conformance-contract.md)
- [TASK-431: Big-Step / Small-Step Meta-Properties and Formalization Boundary Refresh](TASK-431-big-step-small-step-meta-properties-and-formalization-boundary-refresh.md)
- [TASK-438: Canonical IR Semantics Corpus and Result Format](TASK-438-canonical-ir-semantics-corpus-and-result-format.md)
- [PLAN-021: Lean Reference Interpreter](../PLAN-021-LEAN-REFERENCE.md)

## Dependencies

- 📝 [TASK-428: Implementation Conformance Contract](TASK-428-implementation-conformance-contract.md)
- ✅ [TASK-431: Big-Step / Small-Step Meta-Properties and Formalization Boundary Refresh](TASK-431-big-step-small-step-meta-properties-and-formalization-boundary-refresh.md)
- 📝 [TASK-438: Canonical IR Semantics Corpus and Result Format](TASK-438-canonical-ir-semantics-corpus-and-result-format.md)

## Requirements

### Functional Requirements

1. Refresh the Lean/reference implementation plan so it targets the current semantic corpus rather than the older ADT-only or pre-`SPEC-025` planning assumptions.
2. Define the updated Lean/reference scope in terms of:
   - canonical IR support,
   - big-step semantics,
   - small-step semantics,
   - conformance/differential-testing integration,
   - staged proof targets.
3. State explicitly which documents are authoritative for future Lean work and how they map to implementation versus proof tasks.
4. Repackage older Lean/reference phases and tasks so they are either:
   - still valid under the current corpus,
   - superseded and needing rewrite,
   - or retained only as historical context.
5. Define how future Lean/reference work should consume the canonical corpus/result format from TASK-438 and the conformance contract from TASK-428.
6. Update planning/reporting/reference surfaces and `CHANGELOG.md`.

### Non-Functional Requirements

1. Do not implement Lean code here.
2. Do not silently treat older Lean plans as current without auditing them against `SPEC-025` and Phase 67.
3. Preserve useful historical context, but make the new plan authoritative for future Lean work.
4. Use repo-relative links throughout.
5. Mark complete only if future Lean/reference execution work can follow one refreshed plan instead of reconciling multiple stale plan layers.

## TDD Evidence

### Red

Before this task:
- the existing Lean reference plan is centered on an older, narrower interpreter/differential-testing story;
- the planning corpus does not yet fully reflect the accepted `SPEC-025` small-step authority or the new Phase 67 conformance/result-format work;
- future Lean/reference implementation would otherwise start from stale assumptions.

### Green

This task is complete when:
- one refreshed Lean/reference plan exists for the current semantic corpus;
- old plan artifacts are explicitly reconciled as current, superseded, or historical;
- the refreshed plan is aligned with TASK-428, TASK-431, and TASK-438.

## Files

- Modify: `docs/plan/PLAN-021-LEAN-REFERENCE.md`
- Modify: `docs/plan/LEAN_REFERENCE_SUMMARY.md`
- Modify: `docs/reference/formalization-boundary.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `CHANGELOG.md`

## Completion Checklist

- [ ] Lean/reference plan refreshed against current corpus
- [ ] old Lean/reference assumptions audited and classified
- [ ] integration with TASK-428 / TASK-431 / TASK-438 made explicit
- [ ] planning/reference surfaces updated
- [ ] `CHANGELOG.md` updated

## Dependencies for Next Task

This task outputs:
- the refreshed authoritative plan for future Lean/reference implementation work.

Required by:
- future Lean/reference implementation tasks after Phase 67

## Notes

Important constraints:
- Keep the refreshed plan centered on canonical semantics, not Rust implementation quirks.
- Preserve differential-testing value, but align it with the new conformance/result-format story.
- Be explicit about what remains aspirational versus implementation-ready.
