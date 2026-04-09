# TASK-456: Migrate `StdioProvider` to Unified Trait

## Status: Planned

## Description

Update `StdioProvider` to implement `ash_core::CapabilityProvider`, accept unified `Action`
dispatch, and return `CapabilityError` while preserving current buffered/non-buffered I/O
behavior.

## Specification Reference

- [DESIGN-015: Unified Action System](../../design/DESIGN-015-UNIFIED-ACTION-SYSTEM.md)
- [SPEC-017: Capability Integration](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)

## Dependencies

- ✅ [TASK-450](TASK-450-unified-provider-trait.md)
- ✅ [TASK-451](TASK-451-capability-context-unified-trait.md)

## Requirements

1. Implement the shared provider trait for `StdioProvider`.
2. Translate lock, read, and write failures into `CapabilityError`.
3. Keep existing `print` / `println` / `read_line` behavior and test helpers.
4. Update provider tests and engine-facing docs/examples that show custom stdio providers.

## TDD Steps

### Red

- `StdioProvider` still implements the engine-local provider trait and uses split action-name/arg APIs.

### Green

- `StdioProvider` accepts unified `Action` dispatch.
- Existing behavior and tests stay valid under the shared trait.

## Completion Checklist

- [ ] `StdioProvider` trait impl migrated
- [ ] error mapping updated to `CapabilityError`
- [ ] provider tests updated for unified `Action`
- [ ] focused `ash-engine` tests pass
- [ ] `cargo clippy --all-targets --all-features` clean for affected crates
- [ ] `cargo fmt --check` clean
- [ ] `CHANGELOG.md` updated
