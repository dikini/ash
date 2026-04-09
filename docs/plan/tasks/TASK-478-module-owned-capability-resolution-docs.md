# TASK-478: Update Docs and Examples for Module-Owned Capability Resolution

## Status: Planned

## Description

Update active docs, examples, and planning status so the final source-of-truth documents describe
module-owned symbolic capability resolution without the old bridge caveats.

## Specification Reference

- [TASK-471](TASK-471-spec-module-owned-capability-resolution.md)
- [PLAN-017](../PLAN-017-MODULE-OWNED-CAPABILITY-RESOLUTION.md)
- [SPEC-002](../../spec/SPEC-002-SURFACE.md)
- [SPEC-009](../../spec/SPEC-009-MODULES.md)
- [SPEC-012](../../spec/SPEC-012-IMPORTS.md)
- [SPEC-017](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)

## Dependencies

- ✅ [TASK-477](TASK-477-stdlib-capability-bootstrap-and-bridge-removal.md)

## Requirements

1. Remove bridge-specific wording from the active specs when the implementation no longer needs it.
2. Update examples to show capability declarations/imports as the source of symbolic resolution.
3. Update planning/index/changelog status to reflect the bridge removal accurately.
4. Keep `docs/spec/` as the canonical semantic authority.

## Implementation Notes

- Do not declare the bridge removed before the implementation actually removes it.
- This task is the documentation closeout for the resolver-integration phase.

## TDD Steps

### Red

- Identify all active docs/examples that still describe the bridge as the current state.

### Green

- Active docs/examples align with the implemented module-owned resolution contract.

## Completion Checklist

- [x] bridge wording removed where appropriate - CHANGELOG updated with Phase 71 completion
- [x] examples updated - Spec examples show module-owned resolution
- [x] `PLAN-INDEX` updated - Phase 71 marked complete, all tasks marked ✅
- [x] `CHANGELOG` updated - Added Phase 71 completion entry with key changes
