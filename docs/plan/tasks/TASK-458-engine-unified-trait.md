# TASK-458: Update `Engine` to Use Unified Provider Trait

## Status: Planned

## Description

Update `ash-engine` builder and runtime wiring to use `ash_core::CapabilityProvider` directly.
This task removes engine/runtime wiring that assumes separate engine and interpreter provider
traits while preserving the engine embedding API shape as much as possible.

## Specification Reference

- [DESIGN-015: Unified Action System](../../design/DESIGN-015-UNIFIED-ACTION-SYSTEM.md)
- [SPEC-010: Embedding](../../spec/SPEC-010-EMBEDDING.md)

## Dependencies

- ✅ [TASK-451](TASK-451-capability-context-unified-trait.md)
- ✅ [TASK-455](TASK-455-fs-provider-unified-trait.md)
- ✅ [TASK-456](TASK-456-stdio-provider-unified-trait.md)
- ✅ [TASK-457](TASK-457-mcp-provider-unified-trait.md)

## Requirements

1. Update `EngineBuilder` custom-provider storage to use the unified trait object type.
2. Remove engine-builder wiring that converts engine providers into interpreter providers through
   `InterpProviderAdapter`.
3. Keep built-in provider registration and engine construction behavior stable.
4. Update engine docs/tests/examples that show custom provider implementations.

## TDD Steps

### Red

- `EngineBuilder` still stores engine-local provider traits and builds interpreter adapters.

### Green

- `EngineBuilder` stores the shared trait directly.
- Engine construction no longer needs `InterpProviderAdapter`.
- Existing custom-provider tests compile against the shared trait.

## Completion Checklist

- [ ] `EngineBuilder` provider storage/wiring migrated
- [ ] engine tests/examples updated to implement the shared trait
- [ ] no engine build path requires `InterpProviderAdapter`
- [ ] focused engine tests pass
- [ ] `cargo clippy --all-targets --all-features` clean for affected crates
- [ ] `cargo fmt --check` clean
- [ ] `CHANGELOG.md` updated
