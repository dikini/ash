# TASK-2047: Language Reference for Declarations, Functions, Forms, and Patterns

**Status:** Complete
**Phase:** [PLAN-206](../PLAN-206-IMPLEMENTATION-BACKED-LANGUAGE-REFERENCE.md)
**Depends on:** TASK-2045
**Owned feature IDs:** LANG-004, LANG-005, LANG-006, LANG-007, LANG-015, LANG-019, LANG-023.
**Semantic task classification:** non-semantic-workflow-enforcement

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
  task_1865_surface_fn_main_entry`, `--test builtin_fn_e2e_import`; `cargo test -p ash-parser
  --test fn_parser_tests contracts_and_types`. `function_contracts_integration` is a Rust-only
  carrier test and is not source-contract admission/runtime evidence.
- **Produces:** cross-links for type, execution, and diagnostics pages.
- **Non-goals:** workflow declarations, tower closures/arrows, implicit effects, or broad runtime
  semantics for every parsed expression.

## TDD and verification steps

1. Create an example matrix that initially marks each example parse-only until its checker/lowering
   test is named.
2. Verify positive and negative examples with the selected parser/typeck tests.
3. Render representative EBNF/sequent fences and run link/diff checks.

## Verification evidence

- Reviewed live parser branches in `parse_module.rs::module_file`,
  `parse_module/fn_defs.rs`, `parse_expr.rs`, and `parse_pattern.rs`; reviewed the matching
  checker/lowering branches in `check_expr/mod.rs` and `lower.rs`.
- Recorded the re-audit result for named local functions: parser desugars the block form to
  `BlockStmt::Let` plus `Expr::FnDef`; it has no general admitted Engine execution claim.
- Example matrix (the runtime column is never inferred from parse/static success):

  | Page example | Parser/static/lowering evidence | Runtime status |
  |---|---|---|
  | exact `fn main { do { return 42; } }` | `task_1865_surface_fn_main_entry` | fixture-bounded |
  | `requires:` / `ensures:` function | `fn_parser_tests`, `pure_function_contracts_task_505` | closed |
  | imported `builtin fn` call | `builtin_fn_e2e_import` parser/import/static/lowering evidence | closed |
  | `|x: Int| -> x + 1` | parser and typechecker TASK-959 tests | closed |
  | named local `fn` | `fn_parser_tests::closures::task556_named_fn_in_block_desugars_to_let`; parser/lowerer desugaring branch | closed |
  | pure Boolean `if` | `task_2003_pure_anf_normalizer` | fixture-bounded |
  | `if let` / `match` | TASK-1007 parser and TASK-916 pattern diagnostics | closed |
  | named-function `panic "message"` | `fn_parser_tests::control_flow_and_blocks::parse_fn_panic`; `check_expr` and `purity::panic_in_pure_fn_is_ok` | closed |
  | `check obligation_name` | `parse_expr` and `lower_expr`; `check_expr` rejection branch | closed |
  | matching module law / proof | TASK-1365 static registration test | not-applicable |

- Passed `cargo test -p ash-parser --test task_959_pure_closure_arrow --test
  task_1007_if_let_parser_entrypoints --test fn_parser_tests --test
  task_1361_law_keyword_module_scope --test task_1363_proof_keyword_module_scope` (52 passed,
  5 ignored).
- Passed `cargo test -p ash-typeck --test task_959_pure_closure_arrow --test
  task_916_pattern_canonicalization_diagnostics` (5 passed), then passed `cargo test -p
  ash-typeck --test pure_function_contracts_task_505 --test task_1365_proof_name_checking`
  (18 passed).
- Passed `cargo test -p ash-engine --test task_1865_surface_fn_main_entry --test
  task_2003_pure_anf_normalizer` (11 passed). The separately run Rust-only
  `function_contracts_integration` test is intentionally excluded from source-contract
  admission/runtime evidence.
- Passed `cargo test -p ash-engine --test builtin_fn_e2e_import
  builtin_fn_runtime_rejects_without_validated_core_cps_lowering` (1 passed; 8 filtered), proving
  that an imported builtin source call is rejected at the checked Core/CPS admission boundary.
- Passed `cargo test -p ash-cli --test task_1008_matching_diagnostics_surface` (1 passed).
- Passed `cargo test -p ash-parser --test fn_parser_tests
  control_flow_and_blocks::parse_fn_panic` and `cargo test -p ash-typeck
  panic_in_pure_fn_is_ok` for the static-only `panic` example.
- Rendered 3 EBNF and 6 `sequent` fences with external `compileEbnf` and `render` APIs; their
  diagnostics were empty.
- Passed `python3 tools/docs/validate_orientation_indexes.py --self-test`,
  `bash scripts/check-docs-gate.sh` (1,848 Markdown links checked; 0 missing), and
  `git diff --check` after the task's final documentation updates.

## Completion checklist

- [x] Active forms, diagnostics, and status axes are evidence-backed.
- [x] Local/anonymous function claims have fresh tests or are marked partial.
- [x] Removed forms never appear as current examples.
- [x] Indexes, changelog, and PLAN-INDEX are updated.
