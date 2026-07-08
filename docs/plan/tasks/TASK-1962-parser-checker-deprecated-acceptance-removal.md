# TASK-1962: Parser/Checker Deprecated Acceptance Removal

**Status:** Complete
**Phase:** [PLAN-201: Deprecated Functionality Removal](../PLAN-201-DEPRECATED-FUNCTIONALITY-REMOVAL.md)

## Description

Remove parser and checker acceptance of deprecated syntax as valid Ash without retaining
deprecated Ash snippets in repository code.

## Requirements

- Deprecated forms must not produce valid checked modules, surface AST accepted for lowering, or
  runnable engine summaries.
- Parser/checker tests must not retain deprecated Ash source snippets in fixtures, snapshots, or
  Rust string literals.
- Parser/checker fixtures must distinguish target current syntax from removed functionality.

## TDD Steps

1. Add failing parser/checker tests or denylist gates for each AUDIT-201 parser/checker row
   without embedding deprecated Ash snippets.
2. Verify current target syntax remains accepted and old-form acceptors are reachable by grammar
   structure or token-level tests that do not embed stale Ash snippets.
3. Remove acceptance paths and stale diagnostics that depend on old-form Ash snippets.
4. Re-run focused parser, checker, CLI diagnostic, and docs gates.

## Completion Checklist

- [x] Legacy `workflow` entry syntax is not valid current Ash.
- [x] Old `observe ... with` and `act ... with` acceptors are removed or fail before creating
      valid Ash artifacts.
- [x] Public `Act`/`Proc`/`Workflow` tower carriers reject as current syntax.
- [x] Legacy capability/direct-provider authority forms reject as current syntax.
- [x] Deprecated Ash snippets are absent from parser/checker fixtures, snapshots, and Rust source
      string literals.
- [x] Current target syntax remains accepted.
- [x] Focused parser/checker/CLI diagnostic gates pass.

## Evidence

- Removed parser acceptance for old-form act block statements: `act { ... ret ...; }` no longer
  falls back to an `ActBlock` source carrier, while target `act { ... <- ...; return ... }`
  do-sugar remains accepted.
- Removed the typechecker compatibility path for direct `ActBlock` carriers; direct carriers now
  fail closed with `removed act block syntax`.
- Focused verification after old-form act block removal:
  `cargo test -p ash-parser parse_expr::tests::test_removed_act_block_statements_do_not_parse -- --nocapture`;
  `cargo test -p ash-parser parse_module::tests::test_parse_act_block -- --nocapture`;
  `cargo test -p ash-typeck --test task_750_act_block_compat -- --nocapture`.

- Removed obsolete direct `workflow_def` parser unit expectations for old workflow headers and
  rewrote active parser/checker fixtures to target `fn` syntax or removed-form token construction.
- Verified parser and active stale-snippet coverage with:
  `cargo test -p ash-parser --lib`;
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`.
- Verified target CLI checker/run/json paths touched by the conversion with:
  `cargo test -p ash-cli --test check_json_output_test --test cli --test cli_spec_compliance_test --test json_output_schema_test --test run_output --test test_command`.
- Repaired residual synthesized-test/std-test-library failures by making module metadata parsing
  strip current import/module declarations consistently, making synthesized law extraction use that
  target metadata view, converting std algebra imports to semicolon-terminated target syntax, and
  removing raw-source workflow fallback detection from contract synthesis.
- Verified the repaired synthesized paths with:
  `cargo test -p ash-cli --test test_command -- --nocapture`;
  `cargo test -p ash-cli test_runner::synthesized::tests -- --nocapture`;
  `cargo test -p ash-parser --test phase201_removed_syntax -- --nocapture`;
  `cargo test -p ash-engine --test phase201_removed_syntax -- --nocapture`.
- Removed parser acceptors for `capability interface` and `capability impl` definitions, deleted
  stale capability parser/conformance fixtures, and added split-token parser/engine regression
  tests proving those removed forms no longer parse as current Ash.
- Fresh focused verification after the capability acceptor removal:
  `cargo test -p ash-parser --test phase201_removed_syntax -- --nocapture`;
  `cargo test -p ash-engine --test phase201_removed_syntax -- --nocapture`;
  `cargo check -p ash-parser -p ash-engine -p ash-typeck -p ash-cli --all-targets`.
- Removed workflow-header acceptors for the old authority/resource forms: `plays role`,
  `capabilities:`, `owns`, and `uses` no longer parse as workflow header events, stale implicit
  role lowering from direct workflow capabilities was disabled, and stale parser fixtures for
  implicit roles/resource bindings were deleted or retargeted. Target `requires:`/`ensures:`
  contract syntax remains accepted.
- Focused verification after the workflow-header removal:
  `cargo test -p ash-parser --test phase_101_resource_binding_parser -- --nocapture`;
  `cargo test -p ash-parser parse_module::tests::test_parse_inline_module -- --nocapture`;
  `cargo test -p ash-parser --test task_882_spec_h_surface_non_interference -- --nocapture`;
  `cargo check -p ash-parser -p ash-typeck --all-targets`.
- Removed parser acceptance for historical callable type spellings: `Fn(<params>) -> <return>`
  and bare unary `<type> -> <return>` no longer parse as current Ash callable types. Current
  callable type syntax remains `(<params>) -> <return>`, and parser/engine fixtures touched by the
  slice were retargeted to that form.
- Focused verification after callable type acceptor removal:
  `cargo test -p ash-parser --test phase201_removed_syntax -- --nocapture`;
  `cargo test -p ash-parser --test task_957_callable_type_parser -- --nocapture`;
  `cargo test -p ash-parser --test fn_parser_tests parse_parenthesized_callable_type -- --nocapture`;
  `cargo check -p ash-parser --all-targets`.
- Retargeted parser surface function-type display so `Type::Fn` renders target parenthesized
  callable syntax instead of emitting the removed `Fn(...)` spelling.
- Focused verification after callable type display retargeting:
  `cargo test -p ash-parser surface::tests::function_type_display_uses_target_callable_syntax --lib -- --nocapture`;
  `cargo test -p ash-parser surface::tests --lib -- --nocapture`;
  `cargo test -p ash-parser --test task_957_callable_type_parser -- --nocapture`;
  `cargo test -p ash-parser --test phase201_removed_syntax -- --nocapture`.
