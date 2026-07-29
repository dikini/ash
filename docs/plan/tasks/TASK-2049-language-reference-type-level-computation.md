# TASK-2049: Language Reference for Type-Level Computation and Propositions

**Status:** Complete
**Phase:** [PLAN-206](../PLAN-206-IMPLEMENTATION-BACKED-LANGUAGE-REFERENCE.md)
**Depends on:** TASK-2045
**Owned feature IDs:** LANG-010.
**Semantic task classification:** non-semantic-workflow-enforcement

## Description

Document the implemented/partial source surface for sealed domains, type functions, associated
families, promoted data kinds, holes, normalization-facing diagnostics, and propositions.

## Requirements

- Create `docs/reference/language/types/type-level-domains-functions-families-and-propositions.md`.
- Keep source grammar, TypeEnv/summary transport, normalization, and runtime execution status
  separate. Most material is static semantics, not executable program semantics.
- Use sequents for actual checker/normalizer judgments only when an exact implemented rule is
  evidenced; record deferred or non-inverting behaviour as a limitation.
- Explicitly exclude `dtype`; explain only `type`, `newtype`, and `data kind` where accepted.

## Handoffs and dependencies

- **Consumes:** parser type-level branches, `ash-typeck` normalizer/diagnostics, and summary
  transport code.
- **Evidence:** `cargo test -p ash-parser --test task_813_sealed_domain_diagnostics`, `--test
  task_846_public_type_fn_visibility`, `--test task_881_proposition_parse_diagnostics`; `cargo
  test -p ash-typeck --test task_827_normalizer_diagnostics`, `--test
  task_868_associated_family_diagnostics`.
- **Produces:** a type-level terminology boundary for TASK-2053 stdlib documentation.
- **Non-goals:** `dtype`, unrestricted proof search/SMT claims, a runtime evaluator for type
  functions, or semantic rules inferred solely from target specs.

## TDD and verification steps

1. Enumerate each public spelling and required negative diagnostic before prose.
2. Verify the listed parser/typeck tests and mark any untested target clause planned or partial.
3. Render all EBNF and checked-sequent fences with the external tools.

## Verification evidence

- Re-audited the live parser branches for sealed domains, type functions, proposition predicates
  and tails, data-kind declarations, and sealed associated-family members. Re-audited
  `lower_module_type_metadata`, `TypeEnv` registration, normalizer diagnostics, and selected
  Engine summary-transport tests. The page separates these static metadata routes from Engine
  admission/runtime, which is not applicable for this type-level material.
- The page records `data kind Name from type Source;` as parser-only: no current source summary or
  lowering route is claimed. It excludes `dtype` rather than inventing a grammar or example.
- Passed `cargo test -p ash-parser --test task_813_sealed_domain_diagnostics --test
  task_846_public_type_fn_visibility --test task_874_proposition_surface --test
  task_881_proposition_parse_diagnostics --test task_893_promoted_constructor_parser_surface`
  (36 passed).
- Passed `cargo test -p ash-typeck --test task_827_normalizer_diagnostics --test
  task_837_type_function_recursion --test task_838_type_function_normalizer --test
  task_866_associated_family_normalizer --test task_868_associated_family_diagnostics`
  (44 passed).
- Passed `cargo test -p ash-engine --test task_811_domain_summary_transport --test
  task_849_type_computation_summary_transport --test task_867_associated_family_summary_transport`
  (27 passed).
- Passed `cargo test -p ash-typeck --test task_875_proposition_environment --test
  task_876_proposition_solver --test task_878_named_predicate_registration --test
  task_879_proposition_summary_import --test task_880_proposition_checking_points --test
  task_881_proposition_diagnostics --test task_882_spec_h_acceptance_matrix` (59 passed), and
  `cargo test -p ash-engine --test task_879_proposition_summary_transport --test
  task_880_proposition_public_integration --test task_882_spec_h_transport_non_interference`
  (8 passed).
- After independent review, passed `cargo test -p ash-parser --test
  task_813_sealed_domain_diagnostics --test task_846_public_type_fn_visibility --test
  task_874_proposition_surface --test task_881_proposition_parse_diagnostics --test
  task_893_promoted_constructor_parser_surface --test task_900_type_hole_surface --test
  task_906_hkt_kinded_binder_surface` (43 passed) and `cargo test -p ash-typeck --test
  task_838_type_function_normalizer --test task_906_hkt_fail_closed` (12 passed). These reruns
  cover the corrected normalizer sequent, empty domain-constructor syntax, single-row proposition
  tails, wildcard/type-hole distinction, and constructor-kinded parser/checker boundary.
- Rendered the page's EBNF fence with `/home/dikini/Projects/railroad/src/ebnf.js::compileEbnf`
  and its sequent fence with `/home/dikini/Projects/sequent-md/packages/core/src/index.js::render`;
  both produced no diagnostics.
- Passed `python3 tools/docs/validate_orientation_indexes.py --self-test`, `bash
  scripts/check-docs-gate.sh`, and `git diff --check` after this task's edits.

## Completion checklist

- [x] Every form has parser/static/summary/runtime status and exact evidence.
- [x] `dtype` and other absent forms are excluded, not invented.
- [x] Normalization/proposition limits are explicit.
- [x] Removed forms never appear as current examples; indexes/changelog/PLAN-INDEX are updated.
