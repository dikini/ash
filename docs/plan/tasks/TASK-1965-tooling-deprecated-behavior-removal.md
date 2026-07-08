# TASK-1965: Tooling Deprecated Behavior Removal

**Status:** Complete
**Phase:** [PLAN-201: Deprecated Functionality Removal](../PLAN-201-DEPRECATED-FUNCTIONALITY-REMOVAL.md)

## Description

Remove deprecated behavior from formatter, LSP, template, CLI, and example tooling without
retaining deprecated Ash snippets in repository code.

## Requirements

- Formatter must reject deprecated functionality instead of formatting it as valid Ash.
- LSP must not expose removed forms as current symbols or navigable current definitions.
- Template CLI and validators must reject deprecated functionality.
- `ash check` and example gates must keep current syntax as the only productive path.

## TDD Steps

1. Add failing formatter/LSP/template/CLI tests from AUDIT-201 rows.
2. Remove deprecated tooling behavior and stale diagnostic fixtures.
3. Update examples and fixtures to current syntax or delete them.
4. Run focused formatter, LSP, template, CLI, example, and docs gates.

## Completion Checklist

- [x] Formatter rejects deprecated functionality.
- [x] LSP has no current-symbol behavior for removed forms.
- [x] Templates cannot instantiate deprecated functionality.
- [x] Deprecated Ash snippets are absent from tooling fixtures, snapshots, and Rust source string
      literals.
- [x] `ash check` current paths remain target-only.
- [x] Focused tooling gates pass.

## Evidence

- Productive examples and app templates were rewritten to target `fn main() -> ...` entry syntax.
- Historical example and compatibility fixture `.ash` files were deleted instead of retained as
  stale source material.
- Focused gates passed after the cleanup slice:
  `cargo test -p ash-cli --test phase199_template_manifest --test phase199_template_instantiation_cli --test phase199_testing_helpers --test phase199_process_channel_helpers -- --nocapture`;
  `cargo test -p ash-cli --test example_corpus_check -- --nocapture`;
  `cargo test -p ash-cli --test phase200_examples_current_syntax --test phase200_old_syntax_demoted --test phase200_docs_current_syntax -- --nocapture`.
- Remaining tooling scope includes formatter/LSP/parser diagnostic behavior and Rust embedded
  stale fixture strings.
- LSP no longer exposes removed capability interface/implementation definitions as current
  completions, document/workspace symbols, goto targets, hover payloads, or db symbol-index entries;
  unreachable variants are skipped instead of advertised as current language items.
- LSP no longer exposes removed workflow entries as current completions, document/workspace
  symbols, goto targets, hover payloads, parse-summary flags, or db symbol-index entries. The
  matching-diagnostics LSP test now uses current `fn main` source diagnostics instead of
  constructing removed workflow carriers.
- `ash-lint` no longer traverses removed `module.workflow` declarations or retains the
  workflow-specific L004 policy-check rule. The active lint shell now parses target modules,
  visits current definitions only, has no active removed-form rule IDs by default, and its README
  and CLI help use Ash source/file wording instead of workflow-shaped examples.
- The `ash-lint` crate-local stale carrier sweep is silent for:
  `lint_workflow|WorkflowDef|Workflow::|module\.workflow|workflow files|workflow\.ash|src/workflows|empty-workflow|L004|DECIDE|decide/policy|Workflow with no operations`.
- Focused compile verification after the LSP cleanup:
  `cargo check -p ash-parser -p ash-lsp-core --all-targets`.
- After parser surface removal, the LSP no longer has deleted definition variants to skip or expose;
  the focused parser/typechecker/LSP compile gate remains green with those carriers absent:
  `cargo check -p ash-parser -p ash-typeck -p ash-lsp-core --all-targets`.
- Removed stale REPL completions for removed workflow/capability/action words and retargeted
  stored REPL computation names/errors from workflow vocabulary to entry vocabulary.
- Focused REPL verification after this slice:
  `cargo check -p ash-repl --all-targets`;
  `cargo test -p ash-repl --all-targets`.
- Retargeted CLI user-facing command descriptions from workflow wording to target Ash source/entry
  wording, renamed the `run` source classifier away from workflow vocabulary, and moved `ash dot`
  off the removed workflow-definition parser onto the engine's current target-source parser.
- Retargeted daemon command help and non-schema diagnostics from workflow instance/definition
  wording to entry instance/definition wording. Runtime-kernel JSON fields such as `workflow` and
  failure class strings such as `workflow_failure` remain intentionally unrenamed in this slice as
  serialized compatibility surfaces rather than source-language forms.
- Retargeted daemon control-plane and artifact-equivalence fixtures to target entry syntax, and
  removed the stale process-carrier child-failure daemon fixture because it depended on removed
  active source carriers.
- Updated daemon indexing to discover target `fn main` entries from module files before validating
  them through the current engine/artifact path, rather than depending only on removed workflow
  declaration slots. Daemon runtime artifact requests now use the same application-entry identity
  shape as `ash run`.
- Updated the active runtime supervisor stdlib module to target import paths and
  `capability Args` parameter syntax after daemon entry indexing exposed the stale source form.
- Removed stale std carrier module-resolution fixtures and carrier-only law-purity snippets from
  active Rust tests after the Phase 201 gate began checking source-shaped Rust fixture continuation
  lines.
- Retargeted `ash-engine` module-resolution fixtures to target expression-body syntax and
  parse/check assertions for non-application entry return types, preserving module lookup coverage
  without relying on old `return` bodies or arbitrary runtime entry execution.
- Parser/CLI/LSP removed callable-arrow diagnostics now use neutral removed-arrow wording instead
  of Act/Proc/Workflow callable terminology, preserving fail-closed diagnostics without presenting
  removed tower forms as current tooling concepts.
- Synthesized algebra-law test generation no longer models removed tower forms as deferred
  `CarrierType` variants. Generated law profiles enumerate only target carriers, and removed
  carrier spellings are not retained as active source string literals in the law-profile tooling.
- Focused synthesized-law verification after removing the stale carrier variants:
  `cargo test -p ash-cli algebra_law_profile --lib -- --nocapture`;
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`.
- Synthesized policy, obligation, and small-world fallback rows no longer describe current
  deferred rows as legacy or compatibility behavior. The active runner wording now distinguishes
  structured target metadata from deferred raw-source fallback rows without carrying migration
  vocabulary.
- Focused synthesized fallback wording verification:
  `cargo test -p ash-cli test_runner::synthesized::tests::smallworld_metadata_only_oracle_with_executable_target_defers --lib -- --nocapture`;
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`.
- Synthesized contract-test unsupported target metadata now uses runtime-callable wording instead
  of workflow-callable wording.
- Deleted the stale Phase 98 cross-layer conformance test because it depended on removed workflow
  example files and asserted legacy workflow execution success.
- Retargeted LLM stdlib structural tests from ordinary type-snippet compatibility collection to
  target `ModuleFile` metadata lowering, and updated the dispatch stdlib helper-count assertion to
  the current target `pub fn` helper surface.
- Retargeted formatter and docs-current-syntax diagnostics from deprecated-syntax wording to
  removed-syntax wording so active tooling no longer presents removed Ash forms as deprecated
  functionality.
- Retargeted formatter removed-form detection internals from deprecated-pattern vocabulary to
  removed-form vocabulary.
- Retargeted the callable-syntax stdlib/reference gate and agent-facing reference prose from
  legacy/compatibility callable wording to historical removed-syntax wording.
- Removed active OODA compatibility tooling behavior: `ash-lint` no longer enables OODA-specific
  L001/L002 rules, no longer accepts OODA legacy CLI aliases, and its README no longer documents
  OODA compatibility rules. The active stdlib OODA helper module/export and compatibility-demotion
  test were deleted rather than retained as target code.
- The Phase 201 removal gate now blocks reintroducing `std/src/ooda.ash`, OODA root exports, or
  ash-lint OODA alias/category documentation in active tooling paths.
- The stdlib corpus baseline was reconciled after removing OODA and stale expected-fail rows:
  all 59 active `std/src` Ash files now pass `ash check` with no reference-only or expected-fail
  exceptions.
- Productive book labels that described current Ash through OODA were retargeted to target effects
  and policies.
- Focused verification after the diagnostic/metadata retarget:
  `cargo test -p ash-cli --test check_parse_diagnostics -- --nocapture`;
  `cargo test -p ash-parser --test task_960_reserved_callable_arrows -- --nocapture`;
  `cargo test -p ash-lsp-core --test phase200_lsp_migration_polish -- --nocapture`;
  `cargo test -p ash-cli test_runner::synthesized::tests -- --nocapture`;
  `cargo check -p ash-cli -p ash-parser -p ash-lsp-core --all-targets`;
  `cargo clippy -p ash-cli -p ash-parser -p ash-lsp-core --all-targets -- -D warnings`.
- Focused CLI verification after this slice:
  `cargo check -p ash-cli --all-targets`;
  `cargo test -p ash-cli commands::dot::tests::test_dot_source_parser_accepts_target_entry -- --nocapture`;
  `cargo test -p ash-cli --test cli -- --nocapture`.
- Focused formatter/docs verification after removed-syntax wording cleanup:
  `cargo test -p ash-cli --test phase200_formatter_current_syntax -- --nocapture`;
  `cargo test -p ash-cli --test phase200_docs_current_syntax -- --nocapture`.
- Focused formatter verification after removed-form detector retarget:
  `cargo test -p ash-cli --test phase200_formatter_current_syntax -- --nocapture`.
- Focused callable reference verification after removed-syntax wording cleanup:
  `cargo test -p ash-parser --test task_963_stdlib_reference_callable_syntax -- --nocapture`.
- Focused OODA/tooling removal verification:
  `cargo test -p ash-lint --lib -- --nocapture`;
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`;
  `cargo test -p ash-cli --test stdlib_corpus_check -- --nocapture`;
  `cargo clippy -p ash-lint -p ash-cli -p ash-parser -p ash-typeck --all-targets -- -D warnings`.
- Focused LSP verification after removing workflow current-symbol exposure:
  `cargo test -p ash-lsp-core --lib -- --nocapture`;
  `cargo test -p ash-lsp-core --test phase200_lsp_migration_polish --test task_1008_matching_diagnostics_lsp -- --nocapture`;
  `cargo check -p ash-lsp-core --all-targets`;
  `cargo clippy -p ash-lsp-core --all-targets -- -D warnings`;
  `cargo test -p ash-lsp -- --nocapture`;
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`;
  `cargo fmt --all --check`.
- Focused `ash-lint` verification after removing workflow-carrier lint behavior:
  `cargo test -p ash-lint --lib -- --nocapture`;
  `cargo check -p ash-lint --all-targets`.
- Retargeted `ash check` parser-fallback comments and helper naming away from current-workflow
  wording. The fallback now explicitly treats old `workflow` declarations as removed syntax while
  describing successful current paths as entry-source/module-file checks.
- Focused CLI verification after the `ash check` fallback vocabulary cleanup:
  `cargo test -p ash-cli --test check_parse_diagnostics -- --nocapture`;
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`.
- Retargeted import-visibility summary-transport tests away from legacy TypeDef fallback wording
  and old `return` source snippets. Active temp Ash fixtures now use target expression-tail
  entries while keeping callable/type re-export visibility coverage.
- Focused import-visibility verification after this slice:
  `cargo test -p ash-engine --test task_786_import_visibility_summary_rules -- --nocapture`;
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`.
- Retargeted LSP macro-summary function-type rendering away from removed callable syntax. Compact
  LSP identity summaries now render surface function types as target `(<params>) -> <return>`
  strings instead of the historical `Fn(...)` spelling.
- RED/GREEN verification for the LSP summary rendering regression:
  `cargo test -p ash-lsp-core db::tests::macro_summary_renders_function_types_with_target_callable_syntax --lib -- --nocapture`
  first failed with `Fn(Int) -> Bool`, then passed after the formatter retarget;
  `cargo test -p ash-lsp-core --lib db::tests -- --nocapture`;
  `cargo check -p ash-lsp-core --all-targets`.
- Retargeted active parser, engine, and CLI comments away from stale `entry workflow` labels to
  entry-source/entry-definition wording, and extended the Phase 201 removal gate to block those
  labels in active paths.
- RED/GREEN verification after entry-label gate extension:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on active `entry workflow` labels, then passed after the wording cleanup.
- Renamed the CLI input entry-source integration test artifact away from workflow wording, retargeted
  active entry-source test names/variables/assertions, and repaired the checked target fixture to
  return `Ok { value: {} }` instead of a null body for `Result<(), RuntimeError>`.
- RED/GREEN verification after CLI entry-source test-name gate extension:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on stale workflow-named entry-source test identifiers, variables, comments, and
  assertions, then passed after retargeting; `cargo test -p ash-cli --test
  cli_input_entry_source_test -- --nocapture`; `cargo test -p ash-cli --test
  input_functional_test test_run_simple_entry_source -- --nocapture`.
- Retargeted broader active CLI test vocabulary away from workflow-file/source labels.
  `alpha_admission_profile`, `alpha_ash_run_runtime_kernel_mode`, `cli`, `lexical_scope`,
  `run_output`, and `trace_output` tests now use entry/source path names and entry-source labels
  instead of `workflow_path`, `workflow_file`, `entry_workflow`, or
  `ordinary_non_entry_workflow`.
- RED/GREEN verification after broader CLI test-label gate extension:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on those stale CLI test labels, then passed after retargeting; focused
  verification:
  `cargo test -p ash-cli --test run_output --test trace_output --test cli --test
  alpha_admission_profile --test alpha_ash_run_runtime_kernel_mode -- --nocapture`;
  `cargo test -p ash-cli --test lexical_scope_conformance_test
  variables_scope_check_run_trace_agree_on_unbound_failure -- --nocapture`.
- Closeout update: the lexical-scope conformance target was retargeted during Phase 201, and the
  final workspace test gate passed after stale statement-body fixtures were removed or converted.
- Retargeted `ash-engine` module-file warning documentation away from legacy `pub fn` snippet
  diagnostics. Warnings are now described as non-fatal public function export diagnostics.
- RED/GREEN verification after engine warning-label gate extension:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on the stale engine warning label, then passed after retargeting; focused
  `module_file_check_tests` warning-path tests passed for `test_count_pub_fn_snippets_with_diagnostics`,
  `test_pub_fn_parse_failure_produces_warning`, and `test_valid_pub_fn_no_warning`;
  `cargo check -p ash-engine --all-targets`. The full `module_file_check_tests` target still has
  unrelated existing failures around deleted stdlib act fixtures and interface visibility assertions.
- Repaired the remaining `module_file_check_tests` failures. The stale deleted `std/src/act.ash`
  fixture now targets current `std/src/process.ash`; inline module declarations now assert the
  current authoritative ModuleFile parse failure; and the module metadata import stripper no longer
  treats balanced one-line imports without semicolons as unterminated multi-line imports, so public
  interface constraint visibility validation sees following target definitions.
- Focused verification after module-file cleanup:
  `cargo test -p ash-engine --test module_file_check_tests constrained_public_interface_cannot_use_private_imported_interface -- --nocapture`;
  `cargo test -p ash-engine --test module_file_check_tests constrained_public_interface_cannot_use_transitively_imported_interface -- --nocapture`;
  `cargo test -p ash-engine --test module_file_check_tests constrained_public_interface_can_use_direct_import_alias -- --nocapture`;
  `cargo test -p ash-engine --test module_file_check_tests constrained_public_interface_with_associated_family_can_use_direct_import -- --nocapture`;
  `cargo test -p ash-engine --test module_file_check_tests inline_module_declarations_fail_authoritative_module_file_parse -- --nocapture`;
  `cargo test -p ash-engine --test module_file_check_tests test_check_module_file_stdlib_process_module -- --nocapture`;
  `cargo test -p ash-engine --test module_file_check_tests -- --nocapture`.
- Retargeted active daemon/runtime-kernel CLI fixtures to current target entry bodies returning
  `Result<(), RuntimeError>` with explicit `Ok { value: {} }`, and changed runtime-kernel report
  assertions to compare application-entry artifact summaries rather than workflow-boundary
  summaries.
- Repaired the module metadata import stripper for balanced multi-line imports without semicolons
  and retargeted active stdlib algebra interfaces from stale bare callable type forms such as
  `A -> B` inside method signatures to target callable syntax such as `(A) -> B`. This keeps
  daemon entry indexing and one-shot artifact reporting on target-only stdlib code.
- RED/GREEN verification after the daemon/std metadata retarget:
  `cargo test -p ash-engine module_loader::tests::metadata_stripper_preserves_definitions_after_imports_without_semicolons --lib -- --nocapture`
  first failed because a following public interface was stripped after a multi-line import, then
  passed after brace-depth tracking; `cargo test -p ash-cli --test
  alpha_run_daemon_artifact_equivalence -- --nocapture` first failed because daemon startup hit
  stale stdlib callable syntax, then passed after the stdlib algebra retarget.
- Retargeted daemon start-execute status/failure report vocabulary away from workflow labels.
  Successful execution now reports `application_succeeded`, request-shaped failures use an
  `application_request` helper, and the daemon artifact equivalence test expects
  `application_execution_failure` / application-boundary wording for admitted-source drift.
- RED/GREEN verification after daemon report vocabulary cleanup:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `workflow_succeeded`, `workflow_request`, `workflow_execution_failure`, and
  workflow-boundary failure wording, then passed after retargeting; `cargo test -p ash-cli --test
  alpha_run_daemon_artifact_equivalence -- --nocapture`; `cargo check -p ash-cli --all-targets`.
- Retargeted runtime artifact build request naming in engine/CLI tooling paths. The shared
  `RuntimeArtifactBuildRequest` carrier and `ash run` artifact construction now use `entry_name`
  for checked application entries instead of `workflow_name`, and the daemon control-plane helper
  uses entry-name wording for its expected check summary.
- RED/GREEN verification after runtime artifact request-name cleanup:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on the old `workflow_name` carrier in `ash-engine::runtime_artifact`, `ash run`,
  and the daemon control-plane test helper, then passed after retargeting; `cargo check -p
  ash-engine -p ash-cli --all-targets`; `cargo test -p ash-engine --test
  alpha_runtime_kernel_artifact_builder -- --nocapture`; `cargo test -p ash-cli --test
  alpha_ash_run_runtime_kernel_mode --test alpha_run_daemon_artifact_equivalence -- --nocapture`.
- Retargeted engine ordinary-source loader vocabulary away from workflow-source naming.
  `LoadedOrdinaryFile` now exposes `ordinary_source`, and the import-aware parser helper is
  `parse_entry_source_with_imports`, matching the current entry/module source path.
- RED/GREEN verification after engine source-loader cleanup:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `workflow_source` and `parse_workflow_source_with_imports` in active engine
  paths, then passed after retargeting; `cargo check -p ash-engine -p ash-cli --all-targets`.
- Retargeted the remaining module-loader path/file comments and parent-path diagnostic away from
  workflow wording. Ordinary import-backed loading now describes source files, the source tree,
  local source modules, source snapshots, and source parent-path errors.
- RED/GREEN verification after module-loader path/file vocabulary cleanup:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `workflow path`, `ordinary workflow`, `workflow file`, and
  `workflow source snapshot` in `ash-engine::module_loader`, then passed after retargeting them to
  source/module wording.
- Removed the module-loader special case that treated a private ordinary `Act` alias as an opaque
  importable type. Private ordinary aliases now stay non-exportable under the same rule as other
  private ordinary types, while private builtin substrate types remain importable opaquely.
- Retargeted module-loader Rust fixtures away from removed callable/tower vocabulary:
  higher-order builtin fixture source now uses `(A) -> P<B>`, the metadata-stripper fixture uses
  `(A) -> B`, and builtin type identity tests use a neutral `RuntimeHandle` fixture.
- RED/GREEN verification after module-loader compatibility-exception cleanup:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `A -> B`, `Fn(A)`, `act.ash`, `std::act`,
  `is_existing_opaque_compatibility_exception`, the `Act` type special case, and `ActEnv`, then
  passed after removing the special case and retargeting the fixtures; `cargo test -p ash-engine
  module_loader --lib -- --nocapture`.
- Retargeted additional active engine/CLI test fixtures away from removed callable syntax and
  workflow-file/path labels. Inline callable, selected-evidence monomorphize, engine source tests,
  and stdlib algebra signature assertions now use target parenthesized callable type syntax, while
  JSON/path labels use entry/source wording.
- RED/GREEN verification after broader engine fixture cleanup:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `Fn(a)`, `Fn(Int)`, `A -> B`, `A -> M<B>`, `F<A -> B>`, and workflow
  file/path/filename labels in selected active engine/CLI test paths, then passed after
  retargeting; focused green checks included `cargo test -p ash-engine --test
  inline_callable_signature_test -- --nocapture`, `cargo test -p ash-engine --test
  task_1025_algebra_combinators algebra_interface_method_signatures_are_generic_not_int_placeholders
  -- --nocapture`, `cargo test -p ash-engine --test task_923_do_selected_evidence_monomorphize
  -- --nocapture`, `cargo test -p ash-engine
  test_bind_imported_callable_types_uses_imported_pub_fn_signature --lib -- --nocapture`,
  `cargo test -p ash-engine --test runtime_boundary_visibility
  engine_execute_core_workflow_rejects_callable_arity_mismatch -- --nocapture`, and `cargo test
  -p ash-cli --test json_output_schema_test test_json_file_path_present -- --nocapture`.
- Retargeted `std/README.md` active standard-library function signature tables from removed
  `Fun(...)` and bare unary arrow forms to target parenthesized callable signatures.
- RED/GREEN verification after stdlib README signature cleanup:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `Fun(`, `Option<T> ->`, and `Result<T, E> ->` in `std/README.md`, then passed
  after the table retarget; a targeted `rg -n "Fun\\(|Option<T> ->|Result<T, E> ->" std/README.md`
  scan returned no matches.
- Retargeted active parser/engine stale legacy labels. Check-target comments now describe
  obligation references and policy instances without migration wording, parser H12 where-bound
  tests use current behavior names, and module-file parse assertions describe snippet-only parsing
  rather than legacy semicolon snippets.
- RED/GREEN verification after parser/engine stale-label cleanup:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `obligation reference (legacy)`, `legacy_impl_where`, and
  `legacy semicolon snippets`, then passed after retargeting; focused green checks included
  `cargo test -p ash-parser --test task_882_spec_h_surface_non_interference -- --nocapture` and
  `cargo test -p ash-engine --test module_file_check_tests
  malformed_type_without_semicolon_fails_modulefile_parse_instead_of_snippet_skip -- --nocapture`.
- Retargeted TypeEnv fallback-boundary labels away from legacy wording. Nominal unification tests
  now identify the current Type unifier boundary, guarded normalizer rollout tests describe
  inference-meta solving and noncanonical TypeEnv shapes, and the TypeEnv helper comment uses the
  same current fallback vocabulary.
- RED/GREEN verification after TypeEnv stale-label cleanup:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `legacy_nominal`, `legacy meta solving`, `legacy TypeEnv shape`, and
  `Unsupported legacy shapes`, then passed after retargeting; focused green checks included
  `cargo test -p ash-typeck --test task_825_non_inverting_unification_boundary
  task_825_type_unification_nominal_boundary_remains_unchanged -- --nocapture` and `cargo test -p
  ash-typeck --test task_827_normalizer_diagnostics
  task_827_typeenv_rollout_remains_guarded_and_noncanonical_shapes_fallback -- --nocapture`.
- Retargeted TASK-826 TypeEnv forcing-point labels away from legacy wording. The tests now
  describe inference-meta solving, current `Type::Var` unification, fallback unification, and
  noncanonical shapes.
- RED/GREEN verification after TASK-826 TypeEnv forcing-point label cleanup:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `legacy_meta_solving`, `legacy Type::Var`, `legacy_fallback`, and
  `Unsupported legacy shapes`, then passed after retargeting; `cargo test -p ash-typeck --test
  task_826_typeenv_forcing_point_rollout -- --nocapture`.
- Retargeted parser proposition where-bound labels away from legacy wording. TASK-874 now
  describes preserving current impl where bounds, and TASK-881 now describes malformed impl bodies
  after where bounds without legacy terminology.
- RED/GREEN verification after parser proposition where-bound label cleanup:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `preserves_legacy_impl_where`, `mask_legacy_impl_where`, and
  `legacy where bound`, then passed after retargeting; focused green checks included `cargo test
  -p ash-parser --test task_874_proposition_surface
  task_874_preserves_impl_where_bounds_without_generalizing_them -- --nocapture` and `cargo test
  -p ash-parser --test task_881_proposition_parse_diagnostics
  task_881_parse_surface_file_does_not_mask_impl_where_errors -- --nocapture`.
- Retargeted parser removed-capability rejection labels away from legacy wording. Inline module
  and parser-lib tests now describe removed capability declaration syntax and removed
  role-authority capability metadata.
- RED/GREEN verification after parser removed-capability label cleanup:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `legacy_capability` and `legacy_capabilities`, then passed after retargeting;
  focused green checks included `cargo test -p ash-parser removed_capability -- --nocapture` and
  `cargo test -p ash-parser
  test_parse_inline_module_rejects_visibility_qualified_removed_capabilities -- --nocapture`.
- Retargeted interpreter list-helper runtime documentation away from legacy/transition wording.
  The active helper docs now describe current Cons/Nil values directly.
- RED/GREEN verification after interpreter list-helper label cleanup:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `legacy list runtime variant`, then passed after retargeting; `cargo check -p
  ash-interp -p ash-cli --all-targets`.
- Retargeted the normalizer definitional-equality inference-meta boundary away from legacy
  wording. The API documentation now describes the existing `Type` unifier boundary.
- RED/GREEN verification after normalizer inference-meta boundary label cleanup:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `owned by the legacy`, then passed after retargeting; `cargo check -p
  ash-typeck -p ash-cli --all-targets`.
- Retargeted typechecker semantic-summary rejection labels away from legacy wording. TASK-851 now
  describes malformed or unsupported summaries, and TASK-852 now describes unsupported summaries
  carrying computation fields as malformed.
- RED/GREEN verification after typechecker semantic-summary label cleanup:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `malformed_legacy_or_future_summary` and
  `legacy summary carrying computation fields`, then passed after retargeting; focused green
  checks included `cargo test -p ash-typeck --test task_851_imported_type_function_normalizer
  malformed_or_unsupported_summary_is_rejected_without_partial_computation_registration --
  --nocapture` and `cargo test -p ash-typeck --test
  task_852_type_computation_summary_diagnostics
  summary_version_and_malformed_imports_are_rejected_before_partial_registration -- --nocapture`.
- Retargeted TASK-876 proposition-solver no-inversion assertion wording away from legacy
  terminology. The active assertion now describes forbidden inversion, substitution, or
  meta-solving evidence facts.
- RED/GREEN verification after TASK-876 proposition-solver assertion cleanup:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `legacy unification/substitution/meta evidence facts`, then passed after
  retargeting; `cargo test -p ash-typeck --test task_876_proposition_solver -- --nocapture`.
- Retargeted alpha visible-computation non-interference test labels away from legacy wording. The
  active acceptance matrix now describes removed surfaces.
- RED/GREEN verification after alpha visible-computation label cleanup:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `legacy_surfaces`, then passed after retargeting; `cargo test -p ash-typeck
  --test alpha_visible_computation_acceptance_matrix
  alpha_non_interference_matrix_covers_removed_surfaces -- --nocapture`.
