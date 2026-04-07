# TASK-425: SPEC-025 Rule-Schema and Helper-Boundary Consolidation

## Status: ✅ Complete

## Description

Rewrite and tighten `SPEC-025` so its judgment backbone, rule-family presentation, helper-boundary wording, and blocked-state story read as a faithful small-step specification rather than only an initial distilled note. This task should preserve accepted `MCE-005` semantics while making the normative/informative split explicit.

This is docs/spec work only. It does not introduce new runtime behavior or a concrete abstract machine.

## Specification Reference

- [SPEC-025: Small-Step Operational Semantics](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [MCE-005: Small-Step Semantics](../../ideas/minimal-core/MCE-005-SMALL-STEP.md)

## Dependencies

- ✅ [TASK-424: SPEC-025 Faithfulness and Compatibility Contract](TASK-424-spec-025-faithfulness-and-compatibility-contract.md)
- ✅ [TASK-395: Canonical Workflow Small-Step Rule Set and Concurrency Semantics](TASK-395-canonical-workflow-small-step-rule-set-and-concurrency-semantics.md)
- ✅ [TASK-396: Small-Step / Big-Step Correspondence and MCE-006 Handoff](TASK-396-small-step-big-step-correspondence-and-mce-006-handoff.md)

## Requirements

### Functional Requirements

1. Tighten the rule-family presentation in `SPEC-025` while preserving the accepted inventory from `MCE-005`.
2. Make the normative/informative split explicit, distinguishing:
   - normative semantic judgments/configurations/contracts,
   - informative implementation-evidence notes.
3. Make helper-owned semantic boundaries explicit and faithful to accepted `SPEC-004` / `MCE-005` ownership boundaries.
4. Preserve the accepted blocked/suspended contract and its distinction from stuckness.
5. Preserve the accepted concurrency stance for `Par`:
   - interleaving progress,
   - helper-backed terminal aggregation,
   - no accidental left-to-right collapse.
6. State clearly that helper names in `SPEC-025` are schematic ownership markers rather than mandatory Rust API names.
7. Keep pure expressions and pattern matching atomic in v1.

### Non-Functional Requirements

1. Keep the document workflow-first.
2. Do not add full formal inference schemata beyond what the accepted corpus supports.
3. Use repo-relative links throughout.
4. Avoid duplicating MCE-006 runtime evidence inside the normative sections.

## TDD Evidence

### Red

Before this task:
- `SPEC-025` contains the right substance, but its rule presentation and helper-boundary wording are still a first-pass distillation.
- The normative/informative split can be made more explicit.

### Green

This task is complete when:
- `SPEC-025` reads as a tighter faithful spec,
- helper and rule-family boundaries are explicit,
- runtime notes are clearly segregated from normative semantics.

## Files

- Create: `docs/plan/tasks/TASK-425-spec-025-rule-schema-and-helper-boundary-consolidation.md`
- Modify: `docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md`
- Reference: `docs/ideas/minimal-core/MCE-005-SMALL-STEP.md`
- Reference: `docs/spec/SPEC-004-SEMANTICS.md`

## Completion Checklist

- [x] TASK-425 task file created
- [x] rule-family presentation tightened in SPEC-025
- [x] helper-boundary wording clarified
- [x] normative/informative split made explicit
- [x] blocked/suspended and `Par` wording preserved faithfully

## Completion Notes

Completed 2026-04-07.

- Tightened [SPEC-025](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md) so the normative/informative split is explicit in both `§1.3.5` and the section labeling for `§9`.
- Reworked the judgment/helper section so helper names are stated as schematic ownership markers rather than required Rust API names, while keeping helper-owned boundaries faithful to [SPEC-004](../../spec/SPEC-004-SEMANTICS.md) and accepted [MCE-005](../../ideas/minimal-core/MCE-005-SMALL-STEP.md).
- Added a rule-family presentation contract and tightened `§7 Canonical Rule Inventory` so the uppercase entries are clearly normative family markers and inventory/intent contracts, not full formal inference schemata.
- Preserved the accepted MCE-005 inventory, the blocked/suspended vs stuck distinction, pure-expression/pattern atomicity in v1, and the `Par` stance of interleaving progress plus helper-backed terminal aggregation without left-to-right collapse.
- Performed a nearby corpus-consistency sweep and updated [PLAN-INDEX](../PLAN-INDEX.md), dependent task references, and [CHANGELOG.md](../../../CHANGELOG.md) to reflect TASK-425 completion.

## Notes

Important constraints:
- Do not redesign semantics.
- Do not over-formalize beyond what MCE-005 actually freezes.
- Do not let runtime examples leak into normative semantics.
