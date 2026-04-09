# TASK-455: Migrate `FsProvider` to Unified Trait

## Status: Planned

## Description

Update `FsProvider` to implement `ash_core::CapabilityProvider`, accept unified `Action`
dispatch, and return `CapabilityError` instead of engine-local `ProviderError`.

## Specification Reference

- [DESIGN-015: Unified Action System](../../design/DESIGN-015-UNIFIED-ACTION-SYSTEM.md)
- [SPEC-017: Capability Integration](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)

## Dependencies

- ✅ [TASK-450](TASK-450-unified-provider-trait.md)
- ✅ [TASK-451](TASK-451-capability-context-unified-trait.md)

## Requirements

1. Implement the shared provider trait for `FsProvider`.
2. Translate file-system validation and IO failures into `CapabilityError`.
3. Dispatch operations from the unified `Action` shape without reintroducing string/args split APIs.
4. Preserve existing path-allowlist and read-only behavior.
5. Update engine tests covering filesystem provider behavior.

## TDD Steps

### Red

- `FsProvider` still implements the engine-local provider trait and returns `ProviderError`.

### Green

- `FsProvider` implements `ash_core::CapabilityProvider`.
- Existing fs behavior is preserved through unified `Action` dispatch.
- Targeted provider and engine wiring tests pass.

## Completion Checklist

- [ ] `FsProvider` trait impl migrated
- [ ] errors converted to `CapabilityError`
- [ ] tests updated for unified `Action`
- [ ] focused `ash-engine` tests pass
- [ ] `cargo clippy --all-targets --all-features` clean for affected crates
- [ ] `cargo fmt --check` clean
- [ ] `CHANGELOG.md` updated
