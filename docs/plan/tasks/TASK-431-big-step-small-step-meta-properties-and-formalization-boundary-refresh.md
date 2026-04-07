# TASK-431: Big-Step / Small-Step Meta-Properties and Formalization Boundary Refresh

## Status: ✅ Complete

## Description

Record the explicit theorem targets, correspondence obligations, and proof-facing boundary updates needed to make the current Ash semantic corpus mechanically usable for Lean and for future multi-implementation verification work. This task should refresh `docs/reference/formalization-boundary.md` so it matches the current accepted semantic authorities, and it should package the proof-facing meta-properties that connect `SPEC-004` big-step semantics, `SPEC-025` small-step semantics, and the future conformance/differential-testing work of Phase 67.

This remains docs/reference/spec work only.

## Specification Reference

- [SPEC-001: Intermediate Representation](../../spec/SPEC-001-IR.md)
- [SPEC-003: Type System](../../spec/SPEC-003-TYPE-SYSTEM.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [SPEC-021: Runtime Observable Behavior](../../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md)
- [SPEC-025: Small-Step Operational Semantics](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md)
- [Formalization Boundary and Proof Targets](../../reference/formalization-boundary.md)
- [TASK-428: Implementation Conformance Contract](TASK-428-implementation-conformance-contract.md)
- [TASK-429: SPEC-025 Full Rule Definitions](TASK-429-spec-025-full-rule-definitions.md)
- [TASK-430: Small-Step Helper Contracts and State Taxonomy](TASK-430-small-step-helper-contracts-and-state-taxonomy.md)

## Dependencies

- ✅ [TASK-428: Implementation Conformance Contract](TASK-428-implementation-conformance-contract.md)
- ✅ [TASK-429: SPEC-025 Full Rule Definitions](TASK-429-spec-025-full-rule-definitions.md)
- ✅ [TASK-430: Small-Step Helper Contracts and State Taxonomy](TASK-430-small-step-helper-contracts-and-state-taxonomy.md)

## Requirements

### Functional Requirements

1. Refresh `docs/reference/formalization-boundary.md` so it reflects the current canonical semantic corpus used for future Lean/reference work, including the small-step semantic authority now embodied in `SPEC-025`.
2. Record the first explicit theorem and correspondence targets for the current corpus, including at minimum:
   - terminal projection from small-step terminal configurations to `SPEC-004` outcomes,
   - progress-or-blocked classification goals for the admitted fragment,
   - deterministic-fragment determinism targets,
   - helper-bounded nondeterminism obligations,
   - preservation of cumulative semantic dimensions (`Ω`, `π`, `T`, `ε̂`) across the intended correspondence story.
3. Distinguish clearly between:
   - theorem targets that should hold over the semantic corpus itself,
   - conformance obligations for concrete implementations,
   - future proof/development work that remains out of scope.
4. Make explicit how future Lean modeling should treat:
   - canonical semantic specs,
   - source/handoff contracts,
   - historical planning artifacts.
5. Keep the updated proof targets compatible with the implementation-conformance contract from TASK-428 and the helper/taxonomy clarifications from TASK-430.
6. Update planning/reporting surfaces and `CHANGELOG.md`.

### Non-Functional Requirements

1. Do not start Lean implementation work here.
2. Do not claim that all listed theorem targets are already mechanized or fully proven.
3. Preserve the authority hierarchy: canonical specs first, reference/handoff docs second, plans/tasks as historical or implementation guidance only.
4. Use repo-relative links throughout.
5. Mark complete only if future Lean and differential-testing tasks can cite one refreshed formalization boundary without needing to infer current semantic authorities from old phase notes.

## TDD Evidence

### Red

Before this task:
- the formalization boundary note does not fully reflect the newer small-step semantic authority now captured in `SPEC-025`;
- theorem targets are present in partially older big-step-centric terms and are not yet repackaged around the current big-step/small-step/conformance story;
- future Lean/reference work risks relying on stale authority boundaries.

### Green

This task is complete when:
- `formalization-boundary.md` explicitly names the current semantic authorities and their roles;
- proof-facing meta-properties and correspondence targets are listed in terms compatible with the current corpus;
- later Lean/reference and conformance tasks can cite this refreshed boundary directly.

## Files

- Modify: `docs/reference/formalization-boundary.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `CHANGELOG.md`

## Completion Checklist

- [x] formalization-boundary refreshed to include `SPEC-025`
- [x] theorem targets updated for current big-step / small-step corpus
- [x] conformance obligations vs proof targets distinguished clearly
- [x] historical/planning artifact boundary preserved
- [x] planning/reporting surfaces updated as needed
- [x] `CHANGELOG.md` updated

## Dependencies for Next Task

This task outputs:
- a refreshed formalization boundary and proof-target note aligned with the current semantic corpus.

Required by:
- TASK-438: Canonical IR Semantics Corpus and Result Format
- TASK-440: Lean Reference Refresh Plan Against Current Semantic Corpus

## Notes

Important constraints:
- Do not silently demote existing canonical specs by over-centralizing authority in the reference note.
- Do not present theorem targets as already discharged proofs.
- Keep the note useful to both Lean and implementation-conformance work.
