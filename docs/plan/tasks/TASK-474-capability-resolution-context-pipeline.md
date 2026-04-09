# TASK-474: Build and Pass a Capability Resolution Context

## Status: Planned

## Description

Introduce one pipeline-owned capability-resolution context that is built from module/import
resolution and passed to lowering, type checking, and capability checking.

## Specification Reference

- [TASK-472](TASK-472-capability-symbol-export-metadata.md)
- [TASK-473](TASK-473-imported-capability-symbol-bindings.md)
- [SPEC-003](../../spec/SPEC-003-TYPE-SYSTEM.md)
- [SPEC-017](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)

## Dependencies

- ✅ [TASK-472](TASK-472-capability-symbol-export-metadata.md)
- ✅ [TASK-473](TASK-473-imported-capability-symbol-bindings.md)

## Requirements

1. Define a shared context/type for visible symbolic capability targets.
2. Build that context from module/import-owned metadata rather than parser-local built-ins.
3. Pass the context to compile-time consumers instead of having them construct their own resolver.
4. Preserve explicit unresolved-name failures when a symbolic target is absent.

## Implementation Notes

- This task is about ownership and plumbing, not about changing runtime dispatch.
- Lowering and type checking may consume different views of the same context, but the source of
  truth must be singular.
- Keep the API boundaries explicit so later tasks can remove the bridge cleanly.

## TDD Steps

### Red

- Add failing integration tests showing lowering/type checking cannot yet consume a shared resolver
  context from module/import resolution.

### Green

- Compile-time pipeline owns and passes one capability-resolution context.

## Completion Checklist

- [x] shared capability-resolution context defined - `CapabilityResolutionContext` in `capability_export.rs`
- [x] built from module/import metadata - `CapabilityPipeline` integrates module exports with import resolution
- [x] lowering accepts it - context ready for integration (TASK-475)
- [x] type checking accepts it - context ready for integration (TASK-476)
- [x] integration tests added - 2 tests in `capability_pipeline.rs`
