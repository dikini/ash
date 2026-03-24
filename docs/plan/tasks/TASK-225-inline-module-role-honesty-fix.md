# TASK-225: Inline Module Role Honesty Fix

## Status: ✅ Complete

## Description

Make inline-module item parsing honest and spec-aligned by rejecting unsupported canonical module
items explicitly instead of skipping them silently, while preserving the existing same-module role
lowering path.

## Specification Reference

- SPEC-009: Module System
- SPEC-002: Surface Language

## Reference / Design Inputs

- `docs/plans/2026-03-24-inline-module-role-honesty-fix-plan.md`

## Requirements

### Functional Requirements

1. `parse_definitions()` must not silently skip canonical inline-module items such as workflows
2. Unsupported canonical inline-module items must fail explicitly instead of falling through
   recovery/skip logic
3. Existing parser entry points should be reused where practical rather than duplicating grammar
4. Same-module role lowering must continue to work

## Files

- Modify: `crates/ash-parser/src/parse_module.rs`
- Modify: `crates/ash-parser/src/lib.rs` (if test coverage needs it)

## TDD Steps

1. Add failing tests that expose silent skipping for canonical inline-module items
2. Run focused RED verification for the new/updated parser tests
3. Implement the minimal parser fix
4. Run focused GREEN verification for the touched `ash-parser` tests
5. Self-review for spec compliance and scope control

## Completion Checklist

- [x] inline modules no longer silently skip canonical workflow items
- [x] unsupported canonical inline items fail explicitly
- [x] same-module role lowering still passes focused regression coverage
- [x] scope remains limited to Task 1 parser honesty work

## Non-goals

- No expansion of the surface AST to support new inline-module definition variants
- No role-lowering API cleanup beyond what Task 1 requires
- No git commit in this task
