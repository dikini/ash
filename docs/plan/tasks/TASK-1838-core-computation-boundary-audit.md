# TASK-1838: Audit Core computation boundaries

## Description

Audit current parser, typechecker, engine, Core, and docs boundaries for the first target Core computation slice.

## Requirements

- Identify current support for row-bearing `fn`.
- Identify current support and gaps for target `do { ... }`.
- Identify where `Act`, `Proc`, or `Workflow` still imply a target semantic foundation.
- Record implementation decisions for the bounded slice.

## Completion criteria

- [x] Audit evidence names the affected source/test/spec files.
- [x] Audit distinguishes implemented substrate from remaining gaps.
- [x] Audit records why target `do { ... }` is direct-style sequencing in this phase.

## Evidence

- Audited parser `do` entrypoints in `crates/ash-parser/src/parse_expr.rs` and raw lowering in `crates/ash-parser/src/lower.rs`.
- Audited typechecker `DoBlock` handling in `crates/ash-typeck/src/check_expr/mod.rs`.
- Audited engine callable-row summary and Core metadata paths in `crates/ash-engine/src/lib.rs`, `crates/ash-engine/src/module_loader.rs`, and existing `crates/ash-engine/tests/task_1823_parser_engine_typecheck_core_row_preservation.rs`.
- Recorded the bounded decision in PLAN-182 and reconciled `SPEC-095b`, `SPEC-098c`, and `NOTE-019`: target `do { ... }` is direct-style sequencing sugar; explicit `do:K` remains compatibility/profile behavior.

## Depends on

- PLAN-182.
