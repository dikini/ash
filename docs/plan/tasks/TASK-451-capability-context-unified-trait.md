# TASK-451: Update `CapabilityContext` to Use Unified Trait

## Status: Planned

## Description

Migrate `ash-interp` capability registry and execution context types to use
`ash_core::capability::CapabilityProvider` and `CapabilityError` directly. This task
realigns the interpreter-side runtime/provider boundary after TASK-449 and TASK-450.

## Specification Reference

- [DESIGN-015: Unified Action System](../../design/DESIGN-015-UNIFIED-ACTION-SYSTEM.md)
- [SPEC-017: Capability Integration](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)

## Dependencies

- ✅ [TASK-449](TASK-449-action-vec-value.md)
- ✅ [TASK-450](TASK-450-unified-provider-trait.md)

## Requirements

1. Replace interpreter-owned provider trait references with `ash_core::CapabilityProvider`.
2. Update `CapabilityRegistry`, `CapabilityContext`, and exported interpreter helpers to store
   and expose the unified trait object type.
3. Convert provider results from `CapabilityError` into `ExecError` at the interpreter boundary.
4. Preserve existing observe/action capability-availability checks and effect checks.
5. Update interpreter mocks/tests that currently implement the old interpreter-local trait.

## TDD Steps

### Red

- Current `ash-interp` code defines its own provider trait and registry around `ExecResult`.
- `CapabilityContext` and related tests still depend on that interpreter-local trait surface.

### Green

- `CapabilityContext` and `CapabilityRegistry` compile against the shared `ash_core` trait.
- Boundary conversion from `CapabilityError` to `ExecError` is explicit and tested.
- Existing interpreter behavior for unavailable capabilities and effect gating is preserved.

## Completion Checklist

- [ ] `crates/ash-interp/src/capability.rs` uses the shared trait and error type
- [ ] interpreter exports in `crates/ash-interp/src/lib.rs` stay coherent
- [ ] boundary conversion tests cover observe/execute error mapping
- [ ] existing capability-context tests updated and passing
- [ ] `cargo test -p ash-interp capability` or focused equivalents pass
- [ ] `cargo clippy --all-targets --all-features` clean for affected crates
- [ ] `cargo fmt --check` clean
- [ ] `CHANGELOG.md` updated

## Implementation Notes

- Keep the interpreter-level policy/runtime capability types in
  `capability_policy.rs` and `role_runtime.rs` distinct from the provider runtime surface.
- This task should not remove wrapper code in `runtime_state.rs`; that cleanup belongs to TASK-452.
