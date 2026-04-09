# TASK-462: Final Integration Testing for Unified Action System

## Status: Planned

## Description

Run and close out the full verification slice for PLAN-015 after implementation lands across
`ash-core`, `ash-interp`, `ash-engine`, and `ash-cli`.

## Specification Reference

- [DESIGN-015: Unified Action System](../../design/DESIGN-015-UNIFIED-ACTION-SYSTEM.md)
- [SPEC-010: Embedding](../../spec/SPEC-010-EMBEDDING.md)
- [SPEC-017: Capability Integration](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)

## Dependencies

- ✅ [TASK-460](TASK-460-error-handling-unified.md)
- ✅ [TASK-461](TASK-461-documentation-updates.md)

## Requirements

1. Run focused and cross-crate tests covering parser/lowering, interpreter execution, engine
   providers, and CLI provider integration.
2. Run formatting, clippy, docs, and any required local gate commands for affected crates.
3. Verify no remaining code/doc references depend on the removed split-trait surface.
4. Update plan/index/changelog status to reflect completion.

## TDD Steps

### Red

- Workspace may still contain regressions or stale references after the migration.

### Green

- Verification gates pass for the affected workspace slice.
- PLAN-015 closeout surfaces accurately reflect the landed migration.

## Completion Checklist

- [ ] focused and cross-crate test commands pass
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --all-targets --all-features` passes
- [ ] `cargo doc --no-deps` for affected crates passes
- [ ] active docs/examples checked for stale split-trait references
- [ ] `PLAN-INDEX.md` and `CHANGELOG.md` updated for closeout
