# TASK-2047: Language Reference for Declarations, Functions, Forms, and Patterns

**Status:** Planned
**Phase:** [PLAN-206](../PLAN-206-IMPLEMENTATION-BACKED-LANGUAGE-REFERENCE.md)
**Depends on:** TASK-2045
**Owned feature IDs:** LANG-004, LANG-005, LANG-006, LANG-007, LANG-015, LANG-019, LANG-023.

## Description

Document active declaration and expression forms: functions and builtins, values/bindings/blocks,
calls/closures, control flow, and patterns. This task documents `law`/`proof` only at their
implemented authoring/evidence boundary.

## Requirements

- Create `docs/reference/language/forms/index.md`, `declarations-and-functions.md`,
  `values-bindings-blocks-and-calls.md`, and `control-flow-and-patterns.md`.
- Establish live spelling from parser branches; distinguish static acceptance, selected lowering,
  bounded `fn main` execution, and no general execution route.
- Re-audit named local-function claims rather than copying
  `reference/language/functions/local-and-anonymous.md`.
- Document current `fn`/`handler` `requires:`/`ensures:` clauses as contracts, separately from
  removed workflow headers, with parser/static/lowering/Engine status.
- Document the source `check obligation_name` expression with its exact current static/runtime
  classification; do not inherit workflow-obligation semantics without live route evidence.
- Include EBNF and precise typing/evaluation sequents only where checker/lowering evidence exists;
  document diagnostics and pattern-exhaustiveness limits.

## Handoffs and dependencies

- **Consumes:** `parse_module/fn_defs.rs`, `parse_expr.rs`, `surface.rs`, `lower.rs`, function and
  pattern checker paths.
- **Evidence:** `cargo test -p ash-parser --test task_959_pure_closure_arrow`,
  `--test task_1007_if_let_parser_entrypoints`; `cargo test -p ash-typeck --test
  task_916_pattern_canonicalization_diagnostics`; `cargo test -p ash-engine --test
  task_1865_surface_fn_main_entry`, `--test function_contracts_integration`; `cargo test -p
  ash-parser --test fn_parser_tests contracts_and_types`.
- **Produces:** cross-links for type, execution, and diagnostics pages.
- **Non-goals:** workflow declarations, tower closures/arrows, implicit effects, or broad runtime
  semantics for every parsed expression.

## TDD and verification steps

1. Create an example matrix that initially marks each example parse-only until its checker/lowering
   test is named.
2. Verify positive and negative examples with the selected parser/typeck tests.
3. Render representative EBNF/sequent fences and run link/diff checks.

## Completion checklist

- [ ] Active forms, diagnostics, and status axes are evidence-backed.
- [ ] Local/anonymous function claims have fresh tests or are marked partial.
- [ ] Removed forms never appear as current examples.
- [ ] Indexes, changelog, and PLAN-INDEX are updated.
