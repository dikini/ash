# TASK-1869: Surface Function Do Return and Execution

**Status:** Complete
**Plan:** [PLAN-185](../PLAN-185-SURFACE-FUNCTION-LANGUAGE.md)

## Description

Close two user-facing gaps in the Phase 185 surface function slice: target `do { ... }` should accept the documented statement form `return expr;`, and `fn main` entry sources should execute through the engine, not merely parse and check.

## Requirements

- Add RED coverage for `do { return expr; }`.
- Add coverage that `fn main() -> {row} T { ... }` is accepted as an inline-row entry source.
- Add coverage that a function-only `fn main` source executes through `Engine::run`.
- Preserve existing `return expr` without semicolon compatibility.
- Do not introduce a new workflow source requirement or tower runtime mode.

## TDD Steps

1. Write failing tests for semicolon return and executable `fn main`.
2. Verify RED failures.
3. Implement the minimal parser/engine changes.
4. Verify the focused tests pass.

## Completion Checklist

- [x] RED failures recorded.
- [x] Implementation added.
- [x] Focused tests pass.
- [x] Changelog/task evidence updated.

## Verification Evidence

- RED: `cargo test -p ash-engine --test task_1865_surface_fn_main_entry` failed with parse errors for function-only sources containing `do { return ...; }`.
- GREEN: `cargo test -p ash-engine --test task_1865_surface_fn_main_entry` passed with 4/4 tests after allowing semicolon returns and unwrapping declaration-level inline row return types in the typechecker.
