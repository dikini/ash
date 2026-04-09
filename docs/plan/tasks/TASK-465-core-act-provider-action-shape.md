# TASK-465: Split Core `Workflow::Act` into Provider and Action Fields

## Status: Planned

## Description

Change the canonical lowered/core ACT representation so provider lookup and provider-local action
dispatch are separate fields, and lower all operational call surface forms into that canonical
shape.

## Specification Reference

- [TASK-463](TASK-463-spec-capability-call-dispatch-contract.md)
- [TASK-464](TASK-464-surface-operational-call-sugar.md)
- [SPEC-001: IR](../../spec/SPEC-001-IR.md)

## Dependencies

- ✅ [TASK-463](TASK-463-spec-capability-call-dispatch-contract.md)
- ✅ [TASK-464](TASK-464-surface-operational-call-sugar.md)

## Requirements

1. Change core `Workflow::Act` to carry separate `provider_name` and `action_name` fields.
2. Update lowering so both symbolic and explicit operational call forms produce the same canonical
   ACT shape.
3. Preserve guard/default-guard behavior during lowering.
4. Update affected AST visualization or debug helpers.
5. Define and implement one explicit lowering rule for legacy explicit `act ...` forms so they do
   not remain a second overloaded core representation.

## Implementation Notes

- The core AST is the canonical implementation substrate. No downstream crate should need to guess
  whether an ACT target came from a symbolic name or an explicit provider/action source form.
- If symbolic capability targets cannot yet be fully resolved at lowering time, the plan executor
  must make the unresolved-vs-resolved boundary explicit rather than stuffing the unresolved form
  back into one overloaded field.

## TDD Steps

### Red

- Add lowering tests proving the new surface forms lower to split provider/action ACT nodes.

### Green

- Core AST and lowering consistently use explicit provider/action fields.

## Completion Checklist

- [ ] core `Workflow::Act` updated
- [ ] lowering uses canonical provider/action ACT shape
- [ ] lowering tests cover symbolic and explicit forms
- [ ] helper/visualization code updated as needed
- [ ] no alternate overloaded ACT core form remains
