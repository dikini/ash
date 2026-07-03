# TASK-1840: Prove `fn` as primary row-bearing computation unit

## Description

Add or confirm tests showing ordinary `fn` declarations are the primary target computation unit and explicit computation rows are preserved.

## Requirements

- Cover inline row and `where row` forms where relevant.
- Ensure rows preserve operation identities.
- Do not add workflow-specific semantics.

## Completion criteria

- [x] Tests cover a target row-bearing `fn` without requiring workflow syntax.
- [x] Tests pass with existing or updated implementation.

## Evidence

- Existing Phase 181 row fixtures cover inline and `where row` function metadata in `crates/ash-engine/tests/task_1823_parser_engine_typecheck_core_row_preservation.rs`.
- Added Phase 182 fixture `crates/ash-engine/tests/task_1844_core_computation_conformance.rs` with a row-bearing `fn` using target `do { ... }`.
- Verification: `cargo test -p ash-engine --test task_1844_core_computation_conformance` passed.

## Depends on

- TASK-1838.
