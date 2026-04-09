# TASK-463: Freeze Capability Call Dispatch Contract in the Active Specs

## Status: Planned

## Description

Update the normative spec set so operational capability execution is defined in terms of an
explicit `(provider, action)` target pair, with workflow-position sugar for symbolic capability
calls and explicit `provider:action(...)` calls.

This is the canonical-authority task for the phase. The phase is not complete unless the final
semantic contract lives in `docs/spec/`, with design/plan docs reduced to rationale and execution
history only.

## Specification Reference

- [DESIGN-016: Capability Call Dispatch Split and Operational Call Sugar](../../design/DESIGN-016-CAPABILITY-CALL-DISPATCH.md)
- [SPEC-001: IR](../../spec/SPEC-001-IR.md)
- [SPEC-002: Surface Syntax](../../spec/SPEC-002-SURFACE.md)
- [SPEC-003: Type System](../../spec/SPEC-003-TYPE-SYSTEM.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [SPEC-010: Embedding](../../spec/SPEC-010-EMBEDDING.md)
- [SPEC-017: Capability Integration](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)
- [SPEC-025: Small-Step Operational Semantics](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md)

## Dependencies

- None

## Requirements

1. Define act-less workflow sugar for:
   - `capability(args)`
   - `capability(args) when guard`
   - `provider:action(args)`
   - `provider:action(args) when guard`
2. State that symbolic capability names resolve to `(provider, action)` targets.
3. Update the core ACT contract to carry separate provider and action fields.
4. Update both big-step and small-step ACT semantics to use explicit provider lookup plus
   provider-local action dispatch.
5. Update embedding/capability specs to reflect provider-local `execute(action_name, args)` style
   dispatch.
6. State the compatibility rule for legacy explicit `act ...` forms:
   - they remain parseable during migration,
   - but they must lower to the same split `(provider, action)` contract,
   - and the specs must not leave a second overloaded-name semantic path open.
7. Make explicit that `docs/spec/` is the final semantic authority after implementation; do not
   leave any normative behavior defined only in `DESIGN-016` or `PLAN-016`.

## Implementation Notes

- Be explicit about which surface forms are sugar and which internal form is canonical.
- Be explicit that symbolic capability names are resolver-owned symbols, not runtime provider keys.
- Be explicit that `SPEC-025` changes narrowly: helper boundaries must accept split ACT targets
  without inventing a new semantic family.

## TDD Steps

### Red

- The current spec set still reflects overloaded one-name ACT execution and lacks the new sugar
  forms.

### Green

- The active specs state one coherent provider/action operational call contract.

## Completion Checklist

- [ ] `SPEC-001` updated for split ACT target
- [ ] `SPEC-002` updated for operational call sugar
- [ ] `SPEC-003` updated for symbolic target resolution contract
- [ ] `SPEC-004` updated for split ACT big-step semantics
- [ ] `SPEC-010` and `SPEC-017` updated for provider-local dispatch
- [ ] `SPEC-025` updated for split ACT helper boundary
- [ ] legacy explicit `act ...` compatibility story is frozen without preserving a second semantic
      path
- [ ] `docs/spec/` is sufficient to recover the final normative contract without consulting
      `DESIGN-016` or `PLAN-016`
- [ ] docs remain internally consistent across the updated spec set
