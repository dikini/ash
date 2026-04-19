# TASK-595: std::regex Interface and Rust Backend

## Status: ✅ Complete

## Completion Note

`std::regex` is now proven end-to-end from Ash source: `std/src/regex.ash` exposes
`pub builtin fn` declarations, `use regex::{find,matches,replace}` resolves through
the module loader, typechecking succeeds, and runtime execution dispatches through
the evaluator builtin table. The legacy capability carrier has now been removed,
and the user-visible Ash import/runtime path required for TASK-595 is working on
the builtin path alone.

## Description

Add `std::regex` with `find`, `matches`, and `replace` functions, backed by the `regex` Rust crate.

## Specification Reference

- PLAN-090-SPEC-PROCESSOR.md — Track B
- DESIGN-NOTE-PROCESS-EFFECT.md §6 (auto-registered built-in pattern)

## Dependencies

- None

## Requirements

1. Define Ash interface: `find`, `matches`, `replace`.
2. Implement the Rust-backed regex behavior used by the runtime.
3. Surface clear invalid-pattern errors for imported regex calls.

## TDD Steps

### Step 1: Write failing test

Rust integration test calling `regex::find` through the engine.

### Step 2: Implement

- `std/src/regex.ash` builtin declarations (landed)
- runtime builtin dispatch implementation in `ash-interp`
- evaluator builtin dispatch for imported regex calls (landed)

### Step 3: Verify

Ash-language builtin-import tests and runtime dispatch tests pass.

## Verification Steps

- [x] Ash-language `use regex::{find}` import succeeds and function is callable
- [x] Positive builtin-import/runtime coverage exists (`crates/ash-engine/tests/builtin_fn_e2e_import.rs`, `crates/ash-engine/tests/regex_import_limitation.rs`)
