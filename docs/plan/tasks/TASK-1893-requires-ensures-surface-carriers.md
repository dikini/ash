# TASK-1893: Requires/Ensures Surface Carriers

**Status:** ✅ Complete
**Plan:** [PLAN-194](../PLAN-194-CONTRACT-AND-EVIDENCE-SYSTEM.md)

## Description

Parse and preserve target-surface `requires` and `ensures` clauses on ordinary `fn` declarations so they become first-class AST and summary metadata before lowering.

## Requirements

1. Add parser grammar and AST nodes for `requires` and `ensures` clauses attached to `fn` declarations.
2. Support multiple clauses preserving source order and stable identities.
3. Bind `result` only in `ensures` positions; reject `result` in `requires`.
4. Preserve clauses through callable summaries so imported public callables carry contract metadata.
5. Keep contract-position syntax expression-like but routed into a dedicated predicate-expression AST boundary.

## TDD Steps

1. Add parser unit tests for `fn f(x: Int) requires { x > 0 } -> Int { ... }`.
2. Add parser unit tests for `fn f(x: Int) ensures { result >= x } -> Int { ... }`.
3. Add parser tests proving multiple `requires`/`ensures` preserve order and identities.
4. Add summary tests proving imported public callable summaries carry contract metadata without exposing private predicate helpers as ordinary exports.

## Completion Checklist

- [x] `requires`/`ensures` AST nodes and parser grammar added (`crates/ash-parser/src/surface.rs` surface contract structures).
- [x] Multiple-clause ordering and identity preserved (Stage 1 contract lowering context processes ordered clauses).
- [x] `result` scoping restricted to postconditions (lowering rejects `result` in `requires`).
- [x] Callable summaries carry contract metadata for import/export (`FnContractLoweringContext` and Core callable metadata).
- [x] Focused parser/summary tests pass (`pure_function_contracts_task_505.rs`).
