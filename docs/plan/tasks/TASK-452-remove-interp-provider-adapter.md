# TASK-452: Remove Interpreter Provider Adapter Scaffolding

## Status: Planned

## Description

Remove interpreter-side wrapper and adapter code that only exists to bridge the old split
provider-trait design. After TASK-451, `RuntimeState` and related helper paths should no longer
need compatibility wrappers around interpreter-local provider trait objects.

## Specification Reference

- [DESIGN-015: Unified Action System](../../design/DESIGN-015-UNIFIED-ACTION-SYSTEM.md)

## Dependencies

- ✅ [TASK-451](TASK-451-capability-context-unified-trait.md)

## Requirements

1. Remove obsolete wrapper types used only to adapt `Arc<dyn CapabilityProvider>` into the old
   interpreter registry shape.
2. Simplify `RuntimeState::create_capability_context()` and related provider-registration helpers.
3. Preserve runtime-owned provider registration semantics and existing call sites.
4. Update docs/tests/comments that still describe adapter behavior as necessary runtime plumbing.

## TDD Steps

### Red

- Current runtime-state/provider plumbing still contains wrapper code introduced for the split
  trait model.

### Green

- `RuntimeState` creates `CapabilityContext` without compatibility adapter scaffolding.
- Provider registration and lookup behavior remain unchanged for callers.
- No remaining interpreter comments or types claim the old adapter is still required.

## Completion Checklist

- [ ] obsolete wrapper/adapter types removed from `crates/ash-interp/src/runtime_state.rs`
- [ ] runtime-state tests updated for direct unified-trait usage
- [ ] no stale adapter references remain in `ash-interp`
- [ ] focused interpreter tests pass
- [ ] `cargo clippy --all-targets --all-features` clean for affected crates
- [ ] `cargo fmt --check` clean
- [ ] `CHANGELOG.md` updated

## Implementation Notes

- This task is interpreter-only cleanup.
- Do not remove the engine-side adapter until engine/provider migration tasks are complete.
