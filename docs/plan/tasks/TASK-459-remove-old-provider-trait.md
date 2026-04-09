# TASK-459: Remove Old Engine Provider Trait

## Status: Planned

## Description

Remove the old `ash-engine::providers::CapabilityProvider` trait and any remaining split-trait
public API once engine wiring and built-in providers have migrated to the shared
`ash_core::CapabilityProvider`.

## Specification Reference

- [DESIGN-015: Unified Action System](../../design/DESIGN-015-UNIFIED-ACTION-SYSTEM.md)

## Dependencies

- ✅ [TASK-458](TASK-458-engine-unified-trait.md)

## Requirements

1. Delete the old engine-local trait and obsolete `ProviderError` type where no longer needed.
2. Update public re-exports to point at the shared trait/error surface.
3. Remove stale doc examples, test helpers, and comments that still describe the old trait.
4. Keep downstream public API breakage explicit and documented.

## TDD Steps

### Red

- Engine crate still exposes a legacy provider trait/error surface after migration.

### Green

- Only the shared trait remains as the provider abstraction.
- Public docs/tests compile against the new API only.

## Completion Checklist

- [ ] old engine-local provider trait removed
- [ ] old engine-local provider error removed or reduced to intentional compatibility glue
- [ ] public re-exports/docs updated
- [ ] engine test suite compiles against the unified API only
- [ ] `cargo clippy --all-targets --all-features` clean for affected crates
- [ ] `cargo fmt --check` clean
- [ ] `CHANGELOG.md` updated
