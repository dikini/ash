# TASK-468: Migrate Engine Providers and Wiring to Split Dispatch

## Status: Planned

## Description

Update engine providers, custom-provider wiring, and engine integration tests to the split
provider/action dispatch contract.

## Specification Reference

- [TASK-467](TASK-467-provider-local-execute-dispatch.md)
- [SPEC-010: Embedding](../../spec/SPEC-010-EMBEDDING.md)
- [SPEC-017: Capability Integration](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)

## Dependencies

- ✅ [TASK-467](TASK-467-provider-local-execute-dispatch.md)

## Requirements

1. Update built-in providers (`StdioProvider`, `FsProvider`, `McpProvider`) to implement the new
   provider-local execution contract.
2. Update engine provider registration and runtime-state wiring to use the split dispatch model.
3. Update custom-provider integration tests and end-to-end provider tests.
4. Preserve existing capability behavior while making provider/action dispatch explicit.
5. Ensure provider implementations dispatch on provider-local action names only after provider
   selection has already happened.

## Implementation Notes

- This task is not merely mechanical trait churn; it must prove that the engine no longer relies on
  global flat action-name dispatch.
- Preserve user-visible behavior for built-in capabilities while changing the internal execution
  contract.

## TDD Steps

### Red

- Add or update failing engine/provider tests that reflect the new provider-local dispatch
  contract.

### Green

- Engine providers and engine integration tests all use split provider/action dispatch.

## Completion Checklist

- [ ] built-in providers migrated
- [ ] engine builder/runtime wiring updated
- [ ] provider integration tests updated
- [ ] custom-provider path remains supported
- [ ] no engine provider path depends on global flat action-name dispatch
