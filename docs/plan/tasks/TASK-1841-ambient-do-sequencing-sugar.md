# TASK-1841: Implement target `do { ... }` sequencing sugar

## Description

Implement target ambient `do { ... }` as direct-style sequencing sugar. This form must not target `Act`, `Proc`, or `Workflow`.

## Requirements

- Add parser tests first for `do { let x = ...; return x }` and `do { x <- expr; return x }`.
- Add typechecker tests first proving direct-style typechecking.
- Implement the minimal parser and typechecker support.
- Keep explicit `do:K` behavior unchanged.

## Completion criteria

- [x] Parser test fails before implementation and passes after.
- [x] Typechecker test fails before implementation and passes after.
- [x] `do { ... }` exposes no tower target in the surface AST.
- [x] `do { ... }` result type is the final returned expression type.

## Evidence

- Added parser regression `target_ambient_do_block_parses_without_tower_target` in `crates/ash-parser/src/parse_expr/tests.rs`.
- Implemented target ambient parsing in `crates/ash-parser/src/parse_expr.rs`.
- Added typechecker regressions in `crates/ash-typeck/tests/task_1841_ambient_do.rs`.
- Implemented direct-style ambient `do` checking in `crates/ash-typeck/src/check_expr/mod.rs`.
- Verification: parser and typechecker RED runs failed before implementation with missing ambient support; `cargo test -p ash-parser target_ambient_do` and `cargo test -p ash-typeck target_ambient` passed after implementation.

## Depends on

- TASK-1838.
