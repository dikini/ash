# TASK-471: Freeze the Spec Contract for Module-Owned Capability Resolution

## Status: Planned

## Description

Update the normative specs so the follow-on phase after the Phase 70 bridge is explicit: symbolic
operational capability calls resolve from module/import-owned metadata, while explicit
`provider:action(...)` remains a direct surface form.

## Specification Reference

- [PLAN-017](../PLAN-017-MODULE-OWNED-CAPABILITY-RESOLUTION.md)
- [DESIGN-017](../../design/DESIGN-017-MODULE-OWNED-CAPABILITY-RESOLUTION.md)
- [SPEC-002](../../spec/SPEC-002-SURFACE.md)
- [SPEC-003](../../spec/SPEC-003-TYPE-SYSTEM.md)
- [SPEC-009](../../spec/SPEC-009-MODULES.md)
- [SPEC-012](../../spec/SPEC-012-IMPORTS.md)
- [SPEC-017](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)

## Dependencies

- None

## Requirements

1. Describe the current Phase 70 bridge as transitional, not canonical.
2. State that capability declarations, imports, and re-exports own symbolic operational
   resolution.
3. State that module-qualified symbolic names such as `io::fs_read(...)` resolve through module
   paths, not through provider syntax.
4. Preserve explicit `provider:action(...)` as a direct operational form.
5. Define unresolved symbolic capability names as explicit compile-time failures.

## Implementation Notes

- This task is spec-first and must land before code changes that remove the bridge.
- Canonical end-state semantics must live in `docs/spec/`, not only in design docs.
- Do not overclaim implementation completion; bridge notes should remain until later tasks land.

## TDD Steps

### Red

- Identify every spec section that still describes bridge mappings as if they were final.

### Green

- Amend the spec set so the final authority describes module-owned symbolic resolution and the
  bridge as transitional.

## Completion Checklist

- [x] `SPEC-002` updated - Surface syntax symbolic vs explicit resolution
- [x] `SPEC-003` updated - N/A (type-level resolution not affected at surface)
- [x] `SPEC-009` updated - Module-owned capability symbol resolution section
- [x] `SPEC-012` updated - Capability symbol imports section
- [x] `SPEC-017` updated - Resolution sources and compile-time contract
- [x] Bridge status described honestly as transitional until Phase 71 complete
