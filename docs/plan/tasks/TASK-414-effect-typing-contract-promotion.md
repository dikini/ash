# TASK-414: Effect Typing Contract Promotion and Vocabulary Cleanup

## Status: ✅ Complete

## Description

Promote the most actionable parts of `TYPES-003` and `TYPES-004` into the formal docs/planning
corpus.

This task is intentionally narrow. It should not attempt to redesign Ash's entire type system or
implement a new rich effect calculus. Instead, it should do two things:

1. adopt a canonical prose vocabulary for capabilities, providers, witnesses, effects,
   obligations, and provenance;
2. freeze a narrow effect-typing contract around the current coarse effect grades and workflow-form
   classification, while explicitly recording open follow-up questions such as whether `Pure`
   becomes a surfaced bottom element later.

The immediate goal is documentation/spec convergence, not code churn.

## Specification Reference

- [TYPES-003: Capability and Effect Vocabulary](../../ideas/type-system/TYPES-003-capabilities-effects-vocabulary.md)
- [TYPES-004: Effect Typing Foundations](../../ideas/type-system/TYPES-004-effect-typing-foundations.md)
- [SPEC-001: Intermediate Representation](../../spec/SPEC-001-IR.md)
- [SPEC-003: Type System](../../spec/SPEC-003-TYPE-SYSTEM.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [SPEC-010: Embedding](../../spec/SPEC-010-EMBEDDING.md)
- [SPEC-017: Capability Integration](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)
- [Type-to-Runtime Contract](../../reference/type-to-runtime-contract.md)
- [Type-System Vocabulary Guidance](../../reference/type-system-vocabulary-guidance.md)

## Dependencies

- ✅ `TYPES-003` exists as the reasoning record
- ✅ `TYPES-004` exists as the effect-typing exploration
- ✅ Existing specs already expose the coarse effect lattice and capability/provider split in partially aligned form

## Requirements

### Functional Requirements

1. Promote the `TYPES-003` terminology into a reusable reference guidance document.
2. Update the main affected specs to use the clarified vocabulary more consistently:
   - capability declaration
   - capability identity
   - capability witness
   - provider
   - effect classification
   - policy context
   - obligation context
   - provenance context
3. Freeze one narrow effect-typing contract for the current coarse grade system:
   - effect classification is computed from Ash workflow forms and source-level contracts;
   - provider effect metadata is compatibility/validation metadata, not the primary source of effect typing;
   - composition remains join-based over the coarse grade lattice.
4. Add or update one docs/planning artifact that clearly maps major workflow forms onto their current coarse effect classifications.
5. Record the `Pure` bottom-element question as an explicit follow-up rather than silently mixing it into current normative text unless the task also updates every affected normative statement coherently.
6. Update `docs/ideas/README.md` and `docs/ideas/IMPLEMENTABILITY-REPORT.md` so `TYPES-003` and `TYPES-004` read as promoted candidate work rather than loose drafting-only notes.
7. Update `PLAN-INDEX.md`.
8. Update `CHANGELOG.md`.

### Non-Functional Requirements

1. Keep this task documentation/spec-first; no interpreter/typechecker implementation changes required.
2. Avoid overstating provider metadata as if it determines source-level effect typing.
3. Keep the task narrow enough that later implementation tasks can consume it without reopening the whole effect-system design space.
4. Preserve existing canonical effect vocabulary unless a fully coherent replacement is staged.

## Deliverables

1. Reference guidance for type-system vocabulary.
2. A narrowed effect-typing contract and workflow-form classification surface.
3. Planning/reporting updates that mark the work as promoted and ready for follow-on implementation tasks.

## TDD Evidence

### Red

Before this task:

- capability/effect/provider terminology remains inconsistently overloaded across the docs corpus;
- the effect-typing exploration is stronger than the normative handoff, but not yet promoted into a narrow contract-first task;
- the `Pure` proposal exists as exploration but is not yet clearly staged as a follow-up decision.

### Green

This task is complete when:

- docs use a sharper, reusable vocabulary;
- the current coarse effect-typing contract is written down in one narrow, implementation-consumable way;
- later implementation work can target effect inference/runtime-verification alignment without reopening basic terminology.

## Files

- Modify: `docs/ideas/type-system/TYPES-003-capabilities-effects-vocabulary.md`
- Modify: `docs/ideas/type-system/TYPES-004-effect-typing-foundations.md`
- Modify: `docs/reference/type-system-vocabulary-guidance.md`
- Modify: `docs/spec/SPEC-001-IR.md`
- Modify: `docs/spec/SPEC-003-TYPE-SYSTEM.md`
- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `docs/spec/SPEC-010-EMBEDDING.md`
- Modify: `docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md`
- Modify: `docs/reference/type-to-runtime-contract.md`
- Modify: `docs/ideas/README.md`
- Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

## Completion Checklist

- [x] vocabulary guidance file present and linked
- [x] affected specs/reference docs aligned sufficiently for the promoted vocabulary
- [x] narrow effect-typing contract documented
- [x] `Pure` tracked explicitly as follow-up or coherently integrated everywhere it must be
- [x] ideas/reporting corpus updated
- [x] `PLAN-INDEX.md` updated
- [x] `CHANGELOG.md` updated

## Notes

This task is deliberately a promotion/cleanup task, not a full effect-system redesign. If later work
adds `Pure`, effect variables, or associated effects, those should be separate follow-on tasks.

Implementation note: the completed promotion freezes the current coarse effect-classification
contract around workflow forms plus source-level contracts, explicitly demotes provider effect
metadata to compatibility/validation metadata, adds corpus-visible workflow-form classification
tables, and leaves `Pure` as an explicit follow-up rather than silently normative text.
