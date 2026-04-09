# TASK-476: Make Type Checking Use Module-Owned Capability Resolution

## Status: Planned

## Description

Refactor type checking and capability checking so symbolic operational name validation uses the same
pipeline-owned capability-resolution context as lowering.

## Specification Reference

- [TASK-474](TASK-474-capability-resolution-context-pipeline.md)
- [SPEC-003](../../spec/SPEC-003-TYPE-SYSTEM.md)
- [SPEC-017](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md)

## Dependencies

- ✅ [TASK-474](TASK-474-capability-resolution-context-pipeline.md)

## Requirements

1. Type checking/capability checking must stop constructing built-in resolver tables.
2. Compile-time validation of symbolic operational calls must agree with lowering on visible target
   pairs.
3. Errors for unresolved symbolic capability names must remain explicit.
4. ACT declaration checks must cover simple symbolic, module-qualified, and imported aliases.

## Implementation Notes

- This task closes the typechecker-side bridge introduced in Phase 70.
- Use the same capability-resolution context that lowering consumes; do not keep a “matching but
  separate” resolver instance.
- Keep explicit `provider:action(...)` outside symbolic lookup.

## TDD Steps

### Red

- Add failing tests for symbolic/imported/qualified ACT capability checks using the shared context.

### Green

- Type checking and capability checking consume the shared context and agree with lowering.

## Completion Checklist

- [x] typechecker-local built-in resolver removed - `with_builtin_mappings()` deleted
- [~] capability checker uses shared context - `resolve_capability()` calls `CapabilityResolutionContext`, BUT underlying `resolve_for_lowering()` has module scoping TODO
- [x] symbolic ACT checks covered - Shared context checked first for symbolic names
- [x] qualified/imported alias checks covered - Same helper used for qualified names
- [ ] tests added - **GAP:** No dedicated tests for shared context resolution path

## Implementation Status

### Completed
- `CapabilityChecker` has `with_resolution_context(context)` constructor
- `resolve_capability()` helper uses shared context (but see caveat below)
- `verify_with_context()` preserves the resolution context

### Remaining Gaps
- **Module scoping TODO**: Both `resolve_for_lowering()` in parser and its use in type checker search across all modules rather than scoped to current module
- **Parser → type checker integration**: The pipeline exists but is not wired together
- **Tests**: No dedicated tests exercising the shared context resolution path

### Blocked On
- `crates/ash-parser/src/capability_export.rs:resolve_for_lowering()` needs `module_id` parameter for proper scoping
- Once parser is fixed, type checker needs to pass module ID when calling resolve
