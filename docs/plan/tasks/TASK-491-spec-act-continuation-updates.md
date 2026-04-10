# TASK-491: Update Specs for Act Continuation Semantics

## Status: Done

## Description

Update SPEC-001, SPEC-002, SPEC-004, and SPEC-025 to document the new `result_name` and
`continuation` fields on `Workflow::Act` and the associated execution semantics.

## Specification Reference

- [DESIGN-019](../../design/DESIGN-019-ACTION-RESULT-BINDING.md)
- [PLAN-019](../PLAN-019-ACTION-RESULT-BINDING.md)

## Dependencies

- [TASK-489](TASK-489-interpreter-act-continuation.md) — semantics must be implemented and verified

## Requirements

1. **SPEC-001 (IR)**: Update the `Workflow::Act` contract to document `result_name` and `continuation`.
   Define the semantics: `result_name` binds the action result; `continuation` executes after binding.
2. **SPEC-002 (Surface)**: Document the three surface forms (`then`, `as`, `let = cap-call`) and their
   lowering rules. Document `as` as a contextual keyword after `act`.
3. **SPEC-004 (Semantics)**: Update the big-step ACT rule to include continuation execution and
   result binding steps.
4. **SPEC-025 (Small-step)**: Update the small-step ACT helper to model the continuation step
   and result environment extension.

## TDD Steps

### Red

- Identify all spec sections that reference `Workflow::Act` and list required updates.

### Green

- Update all four specs with accurate descriptions of the new fields and semantics.
- Cross-reference between specs for consistency.

## Completion Checklist

- [ ] SPEC-001 updated (Act contract)
- [ ] SPEC-002 updated (surface forms, lowering, keywords)
- [ ] SPEC-004 updated (big-step ACT rule)
- [ ] SPEC-025 updated (small-step ACT helper)
- [ ] Cross-references consistent
- [ ] CHANGELOG.md entry added
