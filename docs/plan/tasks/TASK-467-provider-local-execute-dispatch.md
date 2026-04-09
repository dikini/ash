# TASK-467: Refactor Runtime Dispatch to Provider-local `execute(action_name, args)`

## Status: Planned

## Description

Remove the remaining runtime overload where one name is used for both provider lookup and action
dispatch. Runtime execution should resolve provider first, then call a provider-local execute
method with the action name and evaluated arguments.

## Specification Reference

- [TASK-463](TASK-463-spec-capability-call-dispatch-contract.md)
- [TASK-465](TASK-465-core-act-provider-action-shape.md)
- [TASK-466](TASK-466-resolver-capability-target-pairs.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [SPEC-010: Embedding](../../spec/SPEC-010-EMBEDDING.md)

## Dependencies

- ✅ [TASK-465](TASK-465-core-act-provider-action-shape.md)
- ✅ [TASK-466](TASK-466-resolver-capability-target-pairs.md)

## Requirements

1. Change the provider trait boundary to provider-local action dispatch, such as
   `execute(&self, action_name: &str, args: &[Value])`.
2. Update interpreter/runtime ACT execution to:
   - evaluate guard
   - evaluate arguments
   - look up provider by `provider_name`
   - dispatch action within that provider
3. Remove the remaining dependence on overloaded `Action.name` for provider lookup.
4. Add focused interpreter/runtime tests for split provider/action dispatch.
5. Keep provider lookup outside the provider trait; do not add a redundant provider argument to
   `CapabilityProvider::execute(...)`.
6. Make the runtime path use the same split dispatch mechanism regardless of whether the source form
   was symbolic or explicit.

## TDD Steps

### Red

- Add tests that fail under the current overloaded one-name dispatch path.

### Green

- Runtime dispatch uses explicit provider lookup followed by provider-local action dispatch.

## Implementation Notes

- This task is not complete if the runtime still constructs a flat name and then uses that same
  string for registry lookup and provider-local dispatch.
- If a temporary compatibility shim is required, it must be obviously transitional and removed
  before `TASK-470`.

## Completion Checklist

- [ ] provider trait updated to provider-local action dispatch
- [ ] interpreter/runtime ACT path uses split provider/action dispatch
- [ ] overloaded provider lookup via `Action.name` removed
- [ ] focused runtime tests pass
- [ ] provider trait does not redundantly accept provider name
