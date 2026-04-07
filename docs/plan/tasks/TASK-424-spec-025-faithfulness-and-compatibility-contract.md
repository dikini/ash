# TASK-424: SPEC-025 Faithfulness and Compatibility Contract

## Status: 📝 Planned

## Description

Define the exact contract that a faithful `SPEC-025` must satisfy. This task creates the planning/spec baseline for promoting `SPEC-025` from an initial distilled small-step writeup into a durable normative document that is faithful to accepted `MCE-005`, compatible with `SPEC-004`, and honest about the runtime/interpreter evidence frozen in `MCE-006`.

This is docs/spec-planning work only. It does not implement runtime changes or reopen accepted small-step semantic decisions.

## Specification Reference

- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [SPEC-025: Small-Step Operational Semantics](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md)
- [MCE-005: Small-Step Semantics](../../ideas/minimal-core/MCE-005-SMALL-STEP.md)
- [MCE-006: Align Small-Step Semantics with IR Execution](../../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md)

## Dependencies

- ✅ [TASK-395: Canonical Workflow Small-Step Rule Set and Concurrency Semantics](TASK-395-canonical-workflow-small-step-rule-set-and-concurrency-semantics.md)
- ✅ [TASK-396: Small-Step / Big-Step Correspondence and MCE-006 Handoff](TASK-396-small-step-big-step-correspondence-and-mce-006-handoff.md)
- ✅ [TASK-401: Runtime Carrier Inventory and Semantic Mapping Table](TASK-401-runtime-carrier-inventory-and-semantic-mapping-table.md)
- ✅ [TASK-404: Observable Preservation, Gap Classification, and MCE-007 Handoff](TASK-404-observable-preservation-gap-classification-and-mce-007-handoff.md)

## Requirements

### Functional Requirements

1. Define one explicit faithfulness contract for `SPEC-025` relative to accepted `MCE-005`.
2. Enumerate the specific semantic decisions that `SPEC-025` must preserve from `MCE-005`, including:
   - workflow-first semantic subject,
   - canonical configuration vocabulary,
   - state/label observability split,
   - blocked vs stuck distinction,
   - v1 atomic boundaries,
   - helper-owned concurrency and aggregation boundaries.
3. Enumerate the specific compatibility constraints that `SPEC-025` must preserve relative to `SPEC-004`, including:
   - terminal outcome reconstruction,
   - helper-boundary ownership,
   - receive blocking/fallthrough semantics,
   - `Par` aggregation and determinism boundaries,
   - spawned-child completion/control ownership boundaries.
4. Enumerate the specific runtime-correspondence honesty constraints that `SPEC-025` must preserve relative to `MCE-006`, especially where implementation support is partial or weak.
5. Freeze explicit non-goals preventing:
   - runtime redesign,
   - new workflow syntax,
   - speculative fairness claims,
   - overclaiming current runtime support for `π`, `T`, `ε̂`, or retained completion packaging.
6. State what kinds of claims belong in `SPEC-025` normatively versus only informatively.

### Non-Functional Requirements

1. Keep scope limited to docs/spec planning.
2. Preserve accepted terminology from `MCE-005`, `MCE-006`, and `SPEC-004`.
3. Use repo-relative links throughout.
4. Be conservative where current runtime evidence is partial.

## TDD Evidence

### Red

Before this task:
- `SPEC-025` exists, but there is no dedicated task that freezes what “faithful” means for it.
- The relationship among `SPEC-025`, `MCE-005`, `SPEC-004`, and `MCE-006` is inferable but not packaged as one explicit contract.

### Green

This task is complete when:
- a reader can tell exactly what `SPEC-025` may and may not claim,
- the preservation obligations to `MCE-005`, `SPEC-004`, and `MCE-006` are explicit,
- later rewrite/audit tasks can use this contract mechanically.

## Files

- Create: `docs/plan/tasks/TASK-424-spec-025-faithfulness-and-compatibility-contract.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Reference: `docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md`
- Reference: `docs/ideas/minimal-core/MCE-005-SMALL-STEP.md`
- Reference: `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md`
- Reference: `docs/spec/SPEC-004-SEMANTICS.md`

## Completion Checklist

- [ ] TASK-424 task file created
- [ ] explicit SPEC-025 faithfulness contract defined
- [ ] required preserved decisions from MCE-005 listed
- [ ] required compatibility constraints from SPEC-004 listed
- [ ] required honesty constraints from MCE-006 listed
- [ ] non-goals frozen
- [ ] PLAN-INDEX updated

## Notes

Important constraints:
- Do not reopen MCE-005 semantic design decisions.
- Do not redesign the runtime via this task.
- Do not claim full implementation correspondence where MCE-006 records only partial support.
