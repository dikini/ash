# TASK-1865: `fn main` Entry Adapter

**Status:** Complete
**Plan:** [PLAN-185](../PLAN-185-SURFACE-FUNCTION-LANGUAGE.md)

## Description

Allow ordinary target source that defines `fn main(...) -> T { ... }` and no `workflow` block to parse and check through the engine. The implementation may synthesize an internal runtime adapter, but the user-facing source path must remain ordinary `fn`.

## Requirements

- Add RED engine coverage for `fn main` without a workflow block.
- Preserve callable row metadata for `fn main`, including inline rows and `where row`.
- Lower the entry body through the same direct-style expression path as other functions.
- Do not introduce a new Core semantic path or a new tower runtime mode.
- Keep legacy `workflow` parsing compatible.

## TDD Steps

1. Add an engine test that parses/checks a source containing only top-level functions, including `fn main`.
2. Verify the test fails because the engine currently requires at least one workflow definition.
3. Implement the minimal adapter from function-only program source to existing engine execution/checking structures.
4. Verify the new test passes.

## Completion Checklist

- [x] RED failure recorded.
- [x] Implementation added.
- [x] Focused test passes.
- [x] Existing workflow compatibility tests remain passing.

## Verification Evidence

- RED: `cargo test -p ash-engine --test task_1865_surface_fn_main_entry` failed on `fn main` source with `Parse("Parsing Error: ContextError { context: [], cause: None }")`.
- GREEN: `cargo test -p ash-engine --test task_1865_surface_fn_main_entry` passed after the adapter.
