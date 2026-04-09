# TASK-457: Migrate `McpProvider` to Unified Trait

## Status: Planned

## Description

Update `McpProvider` to implement `ash_core::CapabilityProvider`, consume unified `Action`
values, and return `CapabilityError` while preserving MCP request/response behavior.

## Specification Reference

- [DESIGN-015: Unified Action System](../../design/DESIGN-015-UNIFIED-ACTION-SYSTEM.md)
- [SPEC-017: Capability Integration](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)

## Dependencies

- ✅ [TASK-450](TASK-450-unified-provider-trait.md)
- ✅ [TASK-451](TASK-451-capability-context-unified-trait.md)

## Requirements

1. Implement the shared provider trait for `McpProvider`.
2. Map HTTP/JSON/tool-call failures into `CapabilityError`.
3. Preserve existing MCP operation names and argument expectations.
4. Update MCP provider tests to cover unified `Action` dispatch.

## TDD Steps

### Red

- `McpProvider` still implements the engine-local trait and returns `ProviderError`.

### Green

- `McpProvider` implements the shared trait and preserves observable MCP behavior.
- Existing MCP-focused tests pass after migrating to unified `Action`.

## Completion Checklist

- [ ] `McpProvider` trait impl migrated
- [ ] error mapping updated to `CapabilityError`
- [ ] MCP provider tests updated and passing
- [ ] focused `ash-engine` tests pass
- [ ] `cargo clippy --all-targets --all-features` clean for affected crates
- [ ] `cargo fmt --check` clean
- [ ] `CHANGELOG.md` updated
