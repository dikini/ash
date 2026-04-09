# TASK-466: Resolve Symbolic Capability Calls to `(provider, action)` Pairs

## Status: Planned

## Description

Teach resolution and type-system surfaces to represent symbolic operational capability names as
resolved `(provider, action)` targets, including future module-qualified names such as
`io::fs_read(...)`.

## Specification Reference

- [TASK-463](TASK-463-spec-capability-call-dispatch-contract.md)
- [TASK-465](TASK-465-core-act-provider-action-shape.md)
- [SPEC-003: Type System](../../spec/SPEC-003-TYPE-SYSTEM.md)
- [SPEC-017: Capability Integration](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)

## Dependencies

- ✅ [TASK-463](TASK-463-spec-capability-call-dispatch-contract.md)
- ✅ [TASK-465](TASK-465-core-act-provider-action-shape.md)

## Requirements

1. Represent resolved symbolic operational calls as `(provider, action)` targets.
2. Keep explicit `provider:action(...)` as a direct target shape rather than forcing a second
   resolution path.
3. Ensure module-qualified capability names can participate in the same target contract.
4. Add resolver/typechecker coverage showing symbolic capability calls and explicit qualified calls
   converge on the same canonical target pair.
5. Make the failure mode explicit when a symbolic capability name does not resolve to an operational
   `(provider, action)` target.

## Implementation Notes

- This task owns the semantic boundary between module/name resolution and runtime dispatch.
- Do not leak runtime provider registry assumptions into name resolution.
- Do not treat `provider:action(...)` as if it had to pass through the same symbol lookup as
  `io::fs_read(...)`; those are different front-door forms that must converge on the same canonical
  resolved target.

## TDD Steps

### Red

- Add failing tests for symbolic capability target resolution and module-qualified capability calls.

### Green

- Resolver/typechecker surfaces expose one explicit target-pair contract for operational calls.

## Completion Checklist

- [x] resolved symbolic capability targets carry provider/action separately
- [x] explicit `provider:action(...)` forms remain supported
- [x] module-qualified capability symbols covered (parsed and resolved via bridge)
- [x] resolution/typechecking tests added
- [x] unresolved or misclassified symbolic capability targets fail explicitly

## Implementation Notes

**Status:** Bridge implementation complete. Symbolic and module-qualified capability names
are parsed and resolved through a `CapabilityResolver` with explicit built-in mappings.
Both lowering and capability checking use the same resolver for consistency.

**Future Work:** Full module-system integration where capability declarations in source
automatically register with the resolver, and imports bring in capability mappings from
other modules. This requires deeper integration with the module resolution pipeline.
