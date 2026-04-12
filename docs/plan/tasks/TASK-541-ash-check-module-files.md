# TASK-541: `ash check` module-file support

## Status: Draft

## Description

Add a fallback path in `ash check` that detects non-workflow module files and validates their `pub type`, `pub fn`, and `pub use` declarations.

## Spec Reference

- [SPEC-030: Module Type Resolution](../../spec/SPEC-030-MODULE-TYPE-RESOLUTION.md) §5
- [DESIGN-026](../../design/DESIGN-026-MODULE-TYPE-RESOLUTION-REMEDIATION.md) D2

## Dependencies

- [TASK-539](TASK-539-two-pass-type-collection.md)

## Requirements

1. `ash check <path>` falls back to module-file validation when workflow parse fails.
2. Module-file validation checks type/fn/use parse correctness.
3. Output format matches SPEC-030 §5.3.
4. `ash check std/src/llm/types.ash` succeeds.

## Completion Checklist

- [ ] Module-file detection implemented
- [ ] Validation for types, fns, uses
- [ ] Output format correct
- [ ] `ash check std/src/llm/types.ash` passes

