# TASK-460: Normalize Unified Provider Error Handling

## Status: Planned

## Description

Clean up workspace error handling after the provider-trait migration so `CapabilityError`
is used consistently on provider boundaries and converted deliberately into interpreter and CLI
error surfaces where required.

## Specification Reference

- [DESIGN-015: Unified Action System](../../design/DESIGN-015-UNIFIED-ACTION-SYSTEM.md)

## Dependencies

- ✅ [TASK-451](TASK-451-capability-context-unified-trait.md)
- ✅ [TASK-459](TASK-459-remove-old-provider-trait.md)

## Requirements

1. Audit provider-boundary error conversions across `ash-interp`, `ash-engine`, and `ash-cli`.
2. Keep `ExecError` and CLI-facing error classifications explicit where they remain distinct.
3. Remove stale references that still claim provider-local `ProviderError` surfaces exist.
4. Add regression coverage for key conversion paths.

## TDD Steps

### Red

- Error conversion and reporting still contain stale split-trait/provider-error assumptions.

### Green

- Provider errors originate as `CapabilityError`.
- Interpreter and CLI conversions remain explicit and tested.

## Completion Checklist

- [ ] conversion sites audited and updated
- [ ] regression tests cover representative conversion paths
- [ ] stale `ProviderError` references removed from active code/docs
- [ ] focused workspace tests pass
- [ ] `cargo clippy --all-targets --all-features` clean for affected crates
- [ ] `cargo fmt --check` clean
- [ ] `CHANGELOG.md` updated
