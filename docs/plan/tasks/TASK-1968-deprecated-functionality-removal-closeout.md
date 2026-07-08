# TASK-1968: Deprecated Functionality Removal Closeout

**Status:** Complete
**Phase:** [PLAN-201: Deprecated Functionality Removal](../PLAN-201-DEPRECATED-FUNCTIONALITY-REMOVAL.md)

## Description

Close out Phase 201 with full verification, stale-claim sweeps, docs/changelog reconciliation, and
review remediation.

## Requirements

- Run all Phase 201 focused gates.
- Run broad workspace and docs gates.
- Reconcile PLAN-201, task files, PLAN-INDEX, CHANGELOG, SPEC-INDEX, NOTE-INDEX, and AUDIT-201.
- Run stale-claim sweeps for removed functionality and old support claims.
- Address review findings before marking complete.

## TDD Steps

1. Run focused Phase 201 gates and fix failures.
2. Run broad workspace and docs gates.
3. Update plan/task/changelog/index evidence.
4. Complete review remediation.

## Completion Checklist

- [x] Phase 201 focused gates pass.
- [x] Workspace and docs gates pass.
- [x] PLAN-201, PLAN-INDEX, task files, AUDIT-201, and CHANGELOG are reconciled.
- [x] Stale-claim sweep is recorded.
- [x] Review remediation is complete.

## Evidence

- `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  passed after the closeout fixture sweep.
- Focused stale-fixture retargeting checks passed for:
  `phase127_vendored_dependency_resolution`, `phase128_release_deployment_acceptance`,
  `phase199_canonical_templates`, `phase199_current_syntax_audit`,
  `phase199_template_instantiation_cli`, `task_1008_matching_diagnostics_surface`,
  `builtin_fn_e2e_import`, `builtin_signature_typeck`, `expr_let_integration`,
  `fn_expr_parsing`, `io_stdlib_wiring_test`, `json_stdlib_e2e`, `lexical_scope`, and
  `list_algebraic_laws`, `llm_stdlib_tests`, `multi_file_e2e`, `performance_baseline`,
  `phase151_quickcheck_stdlib`, `record_stdlib_e2e`, `regex_import_limitation`,
  `runtime_boundary_visibility`, `string_stdlib_e2e`,
  `task_1021_std_algebra_namespace_and_interfaces`, and
  `task_1022_pure_algebra_instances`, `task_1024_stdlib_do_evidence`,
  `task_1025_algebra_combinators`, `task_1044_stdlib_monad_constraint`, and
  `task_1045_stdlib_applicative_constraint`, `task_1046_stdlib_monoid_constraint`, and
  `task_1540_type_annotation_quirks`, `task_1773_phase_173_boundaries`,
  `task_1814_row_cross_boundary_engine`, `task_1820_row_summary_transport`,
  `task_1821_core_callable_row_lowering`,
  `task_1822_row_authority_neutrality`,
  `task_1823_parser_engine_typecheck_core_row_preservation`,
  `task_1829_1830_1831_1832_1833_row_admission`,
  `task_1865_surface_fn_main_entry`,
  `task_1911_process_concurrency_cross_boundary`,
  `task_1936_filesystem_provider_wrappers`,
  `task_1937_http_provider_wrappers`,
  `task_1938_clock_time_provider_wrappers`, and
  `task_1939_logging_provider_wrappers`, plus interpreter
  `act_env_runtime_boundary` and
  `task_741_ash_defined_capability_implementation_execution`,
  `task_742_capability_examples`, and LSP
  `phase200_lsp_migration_polish`.
- Additional parser/typechecker closeout checks passed after removing or retargeting active stale
  fixtures: `cargo test -p ash-parser --quiet`,
  `cargo test -p ash-typeck --test alpha_generalized_do_full_bind_lowering --quiet`,
  `cargo test -p ash-typeck --test alpha_monad_evidence_method_body_lowering --quiet`,
  `cargo test -p ash-typeck --test alpha_tcir_typeck_attachment --quiet`,
  `cargo test -p ash-typeck --test task_1021_algebra_interface_registration --quiet`,
  `cargo test -p ash-typeck --test task_1022_pure_algebra_instances --quiet`,
  `cargo test -p ash-typeck --test task_1024_do_and_comprehension_stdlib_evidence --quiet`, and
  `cargo test -p ash-typeck --test task_1814_row_cross_boundary_non_authority --quiet`.
- Final closeout retargeting checks passed for:
  `cargo test -p ash-typeck --test task_747_do_block_substrate --test task_748_do_target_resolution --test task_909_monad_do_target_resolution --quiet`,
  `cargo test -p ash-typeck --test task_880_proposition_checking_points --quiet`,
  `cargo test -p ash-typeck --test task_902_do_target_partial_application --quiet`,
  `cargo test -p ash-typeck --test task_910_hkt_acceptance_matrix --quiet`, and
  `cargo test -p ash-engine --doc --quiet`.
- Deleted obsolete active parser suites for removed source forms:
  `lexical_block_scope`, `policy_lowering`, `proxy_parser_tests`, `receive_lowering`,
  `receive_parser`, and `yield_lowering_tests`.
- Deleted obsolete active typechecker suites for removed source/carrier forms:
  `task_750_target_act_do_sugar`, `task_772_workflow_do`,
  `task_773_workflow_algebra_calls`, and
  `task_778_workflow_contract_classifier_diagnostics`.
- Deleted obsolete engine stdlib import coverage for the removed `proc` module:
  `crates/ash-engine/tests/task_718_proc_stdlib.rs`.
- Workspace and docs gates passed:
  `cargo test --all --quiet`,
  `cargo fmt --check`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`,
  `python3 tools/docs/validate_orientation_indexes.py --self-test`,
  `bash scripts/check-docs-gate.sh`, and
  `git diff --check`.
