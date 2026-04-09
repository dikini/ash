# TASK-464: Add Surface Operational Call Sugar and Explicit `provider:action(...)`

## Status: Planned

## Description

Extend the parser surface to recognize act-less operational call forms and explicit
`provider:action(...)` workflow syntax.

## Specification Reference

- [TASK-463](TASK-463-spec-capability-call-dispatch-contract.md)
- [SPEC-002: Surface Syntax](../../spec/SPEC-002-SURFACE.md)

## Dependencies

- ✅ [TASK-463](TASK-463-spec-capability-call-dispatch-contract.md)

## Requirements

1. Add parser support for workflow-position forms:
   - `capability(args)`
   - `capability(args) when guard`
   - `provider:action(args)`
   - `provider:action(args) when guard`
2. Extend the surface AST with an explicit operational call target shape that can represent either
   a symbolic capability name or an explicit provider/action pair.
3. Preserve existing explicit `act ...` parsing during the migration.
4. Add parser tests covering acceptance, rejection, and guard parsing.
5. Ensure the surface AST does not collapse `provider:action(...)` back into one flat name.
6. Ensure the act-less forms are only accepted in workflow operational-call position unless a later
   spec task explicitly broadens them into general expression calls.

## Implementation Notes

- Treat `when guard` as syntax sugar for the ACT guard carried by the canonical internal form.
- Do not make parser acceptance depend on provider registration or resolver state; parsing should
  preserve target shape, not perform capability resolution.
- Preserve enough target structure that lowering can distinguish:
  - symbolic capability target
  - explicit provider/action target

## TDD Steps

### Red

- Add failing parser tests for each new sugar form and for explicit `provider:action(...)`.

### Green

- Parser and surface AST accept the new forms and represent them without collapsing provider and
  action into one flat name.

## Completion Checklist

- [ ] new parser tests written first
- [ ] surface AST carries explicit operational call target information
- [ ] parser accepts `when guard` forms
- [ ] explicit `provider:action(...)` forms parse correctly
- [ ] legacy explicit `act ...` parsing remains intact
- [ ] surface AST preserves symbolic-vs-explicit target distinction
