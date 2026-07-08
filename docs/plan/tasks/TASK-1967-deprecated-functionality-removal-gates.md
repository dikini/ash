# TASK-1967: Deprecated Functionality Removal Gates

**Status:** Complete
**Phase:** [PLAN-201: Deprecated Functionality Removal](../PLAN-201-DEPRECATED-FUNCTIONALITY-REMOVAL.md)

## Description

Add fail-closed gates that prevent removed deprecated functionality from re-entering repository
code, executable, productive, tooling, template, and documentation paths.

## Requirements

- Gate parser/checker/runtime/tooling/docs paths against AUDIT-201 removal classifications.
- Allow historical prose mentions only when explicitly labeled and owned.
- Include stale-claim sweeps for old support claims.
- Integrate gates with existing docs and local verification workflow where appropriate.

## TDD Steps

1. Add failing gate fixtures for reintroduced deprecated Ash snippets and unclassified deprecated
   functionality.
2. Implement the removal gates.
3. Verify gates allow historical-prose-only rows from AUDIT-201.
4. Run focused gates, docs gates, and workspace checks needed for touched code.

## Completion Checklist

- [x] Removed functionality is blocked from active Ash artifact paths covered by the Phase 201
      gate.
- [x] Productive examples/templates reject deprecated functionality.
- [x] Deprecated Ash source snippets are blocked from code, fixtures, templates, examples,
      snapshots, and Rust source string literals.
- [x] Historical prose remains allowed only when labeled.
- [x] Focused Ash-artifact removal gates pass.

## Evidence

- Added `crates/ash-cli/tests/phase201_deprecated_functionality_removal_gate.rs`, a fail-closed
  gate for removed `workflow`, `pub capability`, `observe ... with`, `act ... with`, and
  Act/Proc/Workflow carrier syntax in active Ash artifacts under `std`, `examples`, `templates`,
  and remaining Ash fixtures.
- Verified:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`.
- Expanded the gate to scan active Rust source roots (`crates`, `examples`, `std`, `templates`,
  `tests`) for source-shaped removed `workflow` declarations and removed carrier/capability forms
  in Rust string literals.
- Converted active CLI, parser, engine, LSP, MCP, lint, and runtime fixtures to target `fn` forms
  or split-token removed-form construction; deleted obsolete parser/engine/typechecker compatibility
  suites whose only purpose was old workflow/carrier syntax.
- Fresh verification:
  `cargo fmt --all --check`;
  `cargo check -p ash-cli -p ash-engine -p ash-parser -p ash-typeck -p ash-interp --all-targets`;
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`;
  `cargo test -p ash-parser --lib`;
  `cargo test -p ash-cli --test check_json_output_test --test cli --test cli_spec_compliance_test --test json_output_schema_test --test run_output --test test_command`
  passes after the synthesized-test/std-test-library repair.
- Additional focused verification for target-only synthesized metadata and removed syntax:
  `cargo test -p ash-cli test_runner::synthesized::tests -- --nocapture`;
  `cargo test -p ash-parser --test phase201_removed_syntax -- --nocapture`;
  `cargo test -p ash-engine --test phase201_removed_syntax -- --nocapture`.
- Tightened Rust scanning so multi-line raw string fixture bodies are checked for source-shaped
  removed Ash declarations, not only the line that opens the string literal.
- Tightened Rust scanning further so source-shaped continuation lines also reject removed
  type-carrier spellings in signatures, while avoiding ordinary Rust implementation identifiers.
- Converted or deleted the newly exposed stale fixtures: parser benchmarks, multi-crate resolver
  modules, LLM stdlib/router references, workflow-dependent typechecker integration tests, old
  capability-configuration tests, and residual module-loader workflow summary snippets.
- Deleted the newly exposed std carrier module-resolution fixtures and carrier-only law-purity
  snippets from active Rust tests.
- Fresh focused verification after tightening the gate:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`;
  `cargo test -p ash-parser --test multi_crate_resolver --test task_910_hkt_diagnostics_surface -- --nocapture`;
  `cargo test -p ash-engine --test llm_e2e_usability_tests -- --nocapture`;
  `cargo test -p ash-typeck --test task_906_hkt_fail_closed -- --nocapture`.
- Extended the gate to cover productive documentation roots (`docs/API.md`, `docs/README.md`,
  `docs/TUTORIAL.md`, `docs/book`, and `docs/tutorials`) so source-shaped removed Ash snippets in
  docs fail alongside code, fixtures, templates, examples, snapshots, and Rust literals.
- Focused verification after the productive-doc scan expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`.
- Removed the stale `ash-fuzz` typechecker fuzz target that constructed deprecated carrier values
  directly; the target was not advertised by the current fuzz README and was outside the main
  workspace.
- Retargeted parser/CLI/LSP removed callable-arrow diagnostics to neutral removed-arrow wording so
  diagnostic gates remain fail-closed without retaining Act/Proc/Workflow callable messages.
- Deleted the stale Phase 98 cross-layer conformance test that still depended on removed workflow
  examples and asserted legacy workflow execution success.
- Removed the engine ordinary type-snippet compatibility parser path and retargeted the LLM stdlib
  structural tests to target `ModuleFile` metadata lowering.
- Retargeted active formatter/docs gate vocabulary from deprecated-syntax wording to
  removed-syntax wording.
- Retargeted the callable-syntax stdlib/reference gate from legacy/compatibility wording to
  historical removed-syntax wording.
- Extended the Phase 201 removal gate to reject source-shaped old-form act block `ret` statements,
  and retargeted exposed parser/lowering/typechecker fixtures away from source-shaped deprecated
  act snippets.
- Removed the remaining internal `ActBlock`/`ActStmt` carrier references from production Rust
  code and active tests, renamed target Act do-sugar parser/test identifiers away from stale
  carrier vocabulary, and kept the Phase 201 gate fail-closed for removed `ret` statement forms.
- Tightened the gate against reintroducing the removed stdlib OODA compatibility module/export or
  ash-lint OODA compatibility aliases/categories in active tooling code.
- Reconciled the stdlib corpus gate to fail on stale expected-fail classifications: all active
  stdlib Ash files now pass `ash check`, with `EXPECTED_STD_FAILING = 0`.
- Verification after the latest gate/tooling cleanup:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`;
  `cargo check -p ash-parser -p ash-typeck -p ash-engine -p ash-interp -p ash-lint -p ash-repl --all-targets`;
  `cargo clippy -p ash-parser -p ash-typeck -p ash-engine -p ash-interp -p ash-lint -p ash-repl --all-targets -- -D warnings`;
  `cargo test -p ash-typeck --test task_750_target_act_do_sugar -- --nocapture`;
  `cargo test -p ash-engine --test task_719_proc_from_act_boundary -- --nocapture`;
  `cargo test -p ash-interp --test task_719_proc_from_act_runtime -- --nocapture`;
  `cargo fmt --all --check`;
  `python3 tools/docs/validate_orientation_indexes.py --self-test`;
  `bash scripts/check-docs-gate.sh`;
  `git diff --check`.
- Extended the Phase 201 removal gate to scan `.core` fixtures for removed Core text row/effect
  aliases (`cap`, `op`, and `proc`) so active Core artifacts cannot retain stale syntax.
- Verification after the `.core` gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`;
  `cargo clippy -p ash-core -p ash-cli --all-targets -- -D warnings`;
  `cargo fmt --all --check`.
- Extended the Phase 201 removal gate to reject source-shaped old workflow authority/resource
  header clauses (`capabilities:`, `plays role`, `owns`, and `uses`) in active Ash artifacts and
  Rust string literals, without classifying target function/workflow `requires:`/`ensures:`
  contracts or current role-definition capability metadata as removed.
- Verification after the workflow-header gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`;
  `cargo test -p ash-parser --test phase_101_resource_binding_parser -- --nocapture`;
  `cargo test -p ash-parser parse_module::tests::test_parse_inline_module -- --nocapture`;
  `cargo check -p ash-parser -p ash-typeck --all-targets`.
- Extended the Phase 201 removal gate to reject source-shaped historical `Fn(...)` callable type
  snippets in active Ash artifacts and Rust string literals, while ignoring ordinary Rust `dyn Fn`
  trait types.
- Added parser surface display regression coverage so active formatter/display paths cannot emit
  the removed `Fn(...)` callable spelling from `Type::Fn`.
- Extended the Phase 201 removal gate to block the active interpreter from labeling current
  pattern-matched builtin fallback dispatch as legacy.
- Extended the Phase 201 removal gate to block stale `ash check` wording that described removed
  workflow declarations as a current workflow keyword.
- Extended the Phase 201 removal gate to block stale import-visibility test labels that describe
  current imported type-definition and semantic-summary transport as legacy TypeDef fallback
  behavior.
- Extended the Phase 201 removal gate to scan `README.md` and `docs/SHARO_CORE_LANGUAGE.md`, so
  source-shaped removed Ash snippets in root and historical language docs fail alongside other
  productive docs, code, fixtures, templates, examples, snapshots, and Rust literals.
- Verification after the root/historical-doc gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`;
  `bash scripts/check-docs-gate.sh`;
  `git diff --check`.
- Extended the Phase 201 removal gate to block stale typechecker do-target comments that describe
  the current built-in computation dictionary bridge as legacy fallback dictionaries.
- Verification after callable-type gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`;
  `cargo test -p ash-engine --test list_ops_e2e -- --nocapture`;
  `cargo test -p ash-engine --test task_1769_binder_macro_boundaries -- --nocapture`;
  `cargo test -p ash-engine --test task_1798_closure_module_function_visibility -- --nocapture`.
- Verification after import-visibility stale-label gate expansion:
  `cargo test -p ash-engine --test task_786_import_visibility_summary_rules -- --nocapture`;
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`.
- Verification after do-target stale-label gate expansion:
  `cargo test -p ash-typeck do_target --lib -- --nocapture`;
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`.
- Extended the Phase 201 removal gate to block stale `entry workflow` labels in active parser,
  engine, and CLI paths.
- RED/GREEN verification after entry-label gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on active `entry workflow` labels, then passed after parser/engine/CLI wording was
  retargeted to entry-source/entry-definition terminology.
- Extended the Phase 201 removal gate to block stale workflow-named entry-source test identifiers,
  variables, comments, and assertions in active CLI entry-source tests.
- RED/GREEN verification after CLI entry-source gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on stale workflow-named entry-source test artifacts, then passed after the CLI test
  rename and fixture retarget; `cargo test -p ash-cli --test cli_input_entry_source_test --
  --nocapture`; `cargo test -p ash-cli --test input_functional_test test_run_simple_entry_source --
  --nocapture`.
- Extended the Phase 201 removal gate to block stale `ash-engine` module-file warning docs that
  describe current non-fatal public function export diagnostics as legacy `pub fn` snippet
  diagnostics.
- RED/GREEN verification after engine warning-label gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on the stale engine label, then passed after the documentation retarget.
- Extended the Phase 201 removal gate to block role-runtime code and tests from constructing
  `WorkflowDef` solely to resolve role references.
- RED/GREEN verification after role-runtime carrier gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `WorkflowDef` in role-runtime paths, then passed after `RoleRegistry` was
  retargeted to explicit role refs and capability declarations.
- Extended the Phase 201 removal gate to block old RuntimeKernel workflow identity carrier names
  in active runtime-kernel implementation, CLI call sites, and core carrier tests:
  `WorkflowDefinitionId`, `WorkflowDefinitionIdentity`, `WorkflowArtifactId`,
  `WorkflowArtifactIdentity`, `WorkflowInstanceId`, and `WorkflowInstanceIdentity`.
- RED/GREEN verification after RuntimeKernel identity-carrier gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on those old carrier names, then passed after the runtime-kernel carrier API was
  retargeted to application/entry vocabulary.
- Extended the Phase 201 removal gate to block lower runtime workflow-named admission and boundary
  carriers in active runtime/engine/interpreter paths, including old admission request/outcome,
  contract evidence, boundary outcome, report, failure, and admitted-boundary wrapper names.
- RED/GREEN verification after lower boundary-carrier gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on the old lower runtime boundary carrier names, then passed after the
  application-boundary carrier retarget.
- Extended the Phase 201 removal gate to block hard-coded do-target tower fallback support in
  `ash-typeck::do_target`: old `DoTowerLevel`, hidden Act bind/return carriers, the tower
  dictionary resolver, Act/Proc/Workflow target diagnostics, and hard-coded Act/Proc/Workflow
  intrinsic shim strings.
- RED/GREEN verification after do-target fallback gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on the old do-target fallback surface, then passed after do-target resolution was
  retargeted to explicit `Monad` evidence only.
- Extended the Phase 201 removal gate to block runtime/Core tower-attribution carriers in active
  source: old `TowerLevel`, `OperationalFailure::tower`, Proc/Workflow tower variants, workflow
  failure entities, and TCIR/AMIR/runtime-kernel `tower_level` / `from_tower` / `to_tower` fields.
- RED/GREEN verification after runtime/Core tower-carrier gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on the old runtime/Core tower attribution surface, then passed after the API was
  retargeted to boundary/application terminology.
- Extended the Phase 201 removal gate to block workflow-intrinsic carrier names in active
  typechecker paths: `WorkflowIntrinsic`, `workflow_intrinsics`,
  `lookup_workflow_intrinsic`, `workflow_intrinsic`, and `__workflow_intrinsic_context`.
- RED/GREEN verification after contract-intrinsic carrier gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on those active typechecker carrier names, then passed after the typechecker
  carrier API and hidden context sentinel were retargeted to contract-intrinsic vocabulary.
- Extended the Phase 201 removal gate to block the obsolete typechecker workflow-surface
  capability checker and interpreter workflow-parser test path, including `capability_check`,
  `CapabilityChecker`, `parse_workflow::workflow_def`, `lower_workflow`, and `SurfaceWorkflow`
  in the selected active typechecker/interpreter paths.
- Extended the Phase 201 removal gate to block stale Par-removal test labels that refer to the
  deleted `SurfaceWorkflow::Par` carrier or removed capability-checker path.
- RED/GREEN verification after the capability-checker/interpreter-test gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on the stale checker module/tests and old interpreter parser/lowerer tests, then
  passed after those active stale-code surfaces were removed; a later targeted scan found and
  retargeted the stale Par-removal labels, and the same gate remained green.
- Extended the Phase 201 removal gate to block workflow-capability carrier names in active
  obligation/runtime verification paths and focused tests, including `WorkflowCapabilities` and
  `workflow_capabilities`.
- RED/GREEN verification after workflow-capability carrier gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on the old capability carrier names, then passed after the active API and test
  surface was retargeted to entry-capability vocabulary.
- Extended the Phase 201 removal gate to block stale workflow-file/source labels in active CLI
  tests, including `workflow_path`, `workflow_file`, `entry_workflow`,
  `ordinary_non_entry_workflow`, and local `let workflow =` source fixtures in the selected
  CLI test paths.
- RED/GREEN verification after broader CLI test-label gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on the stale CLI test labels, then passed after retargeting those tests to
  entry/source vocabulary.
- Extended the Phase 201 removal gate to block stale daemon workflow report labels in active CLI
  paths and daemon artifact tests, including `workflow_succeeded`, the request-failure helper
  name, `workflow_execution_failure`, and workflow-boundary failure wording.
- RED/GREEN verification after daemon report-label gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on the stale daemon report labels, then passed after the daemon report vocabulary
  was retargeted to application/entry wording.
- Extended the Phase 201 removal gate to block the old `workflow_name` runtime artifact request
  carrier in selected active engine/CLI paths: `ash-engine::runtime_artifact`, `ash run` artifact
  construction, and the daemon control-plane expected-check-summary helper.
- RED/GREEN verification after runtime artifact request-name gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on those workflow-name carrier paths, then passed after retargeting them to
  `entry_name`.
- Extended the Phase 201 removal gate to block the old `workflow_type` typechecker
  instance/control-link field carrier in `crates/ash-typeck/src/types.rs`.
- RED/GREEN verification after type instance-carrier gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `workflow_type` in `crates/ash-typeck/src/types.rs`, then passed after
  retargeting the carrier to `entry_type`; `cargo check -p ash-typeck -p ash-engine
  --all-targets`.
- Extended the Phase 201 removal gate to block the old `workflow_type` runtime spawn/instance
  carrier token in selected active core, interpreter, and engine paths.
- RED/GREEN verification after runtime spawn/instance-carrier gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `workflow_type` in the gated runtime spawn/instance paths, then passed after
  retargeting those carriers to `entry_type`; `cargo check -p ash-core -p ash-parser -p
  ash-interp -p ash-engine -p ash-cli -p ash-typeck --all-targets`.
- Extended the Phase 201 removal gate to block the old `workflow_name` callable/admission carrier
  token in active engine/interpreter source paths.
- RED/GREEN verification after callable/admission name-carrier gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `workflow_name` in `crates/ash-interp/src/runtime_state.rs` and
  `crates/ash-engine/src/lib.rs`, then passed after retargeting the carrier to `entry_name`;
  `cargo check -p ash-interp -p ash-engine -p ash-cli --all-targets`.
- Extended the Phase 201 removal gate to block stale callable-workflow registry API identifiers in
  active engine/interpreter source paths, including `RegisteredCallableWorkflow`,
  `callable_workflows`, `register_callable_workflow`, `blocking_register_callable_workflow`, and
  `callable_workflow`.
- RED/GREEN verification after callable registry API gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on the stale callable-workflow registry identifiers, then passed after retargeting
  them to callable-entry identifiers; `cargo check -p ash-interp -p ash-engine -p ash-cli
  --all-targets`.
- Extended the Phase 201 removal gate to block stale child-workflow registry API identifiers in
  active engine/interpreter source paths, including `child_workflows`, `register_child_workflow`,
  and `child_workflow`.
- RED/GREEN verification after child registry API gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on the stale child-workflow registry identifiers, then passed after retargeting them
  to child-entry identifiers; `cargo check -p ash-interp -p ash-engine -p ash-cli --all-targets`.
- Extended the Phase 201 removal gate to block stale runtime workflow-projection wrapper names in
  active interpreter/engine source and focused tests, including `workflow_projection`,
  `execute_workflow_proc_projection`, `unsupported_workflow_proc_projection_message`, and the old
  `FirstClassWorkflowProjectionExecutionUnsupported` diagnostic label.
- RED/GREEN verification after entry projection wrapper gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on the stale workflow-projection wrapper names, then passed after retargeting them
  to entry-projection names; `cargo check -p ash-interp -p ash-engine -p ash-cli --all-targets`.
- Extended the Phase 201 removal gate to block stale TCIR/AMIR workflow-artifact carrier names in
  active core/typechecker/test paths, including `TcirWorkflowArtifactProvenance`,
  `workflow_artifact`, `WorkflowArtifact`, and `WorkflowArtifactBuilder`.
- RED/GREEN verification after TCIR/AMIR artifact-carrier gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on the stale workflow-artifact carrier names, then passed after retargeting them to
  entry-artifact names; `cargo check -p ash-core -p ash-typeck -p ash-engine -p ash-cli
  --all-targets`.
- Extended the Phase 201 removal gate to block stale engine workflow-source loader names in active
  engine paths: `workflow_source` and `parse_workflow_source_with_imports`.
- RED/GREEN verification after engine source-loader gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on the stale engine source-loader names, then passed after retargeting them to
  ordinary/entry source names; `cargo check -p ash-engine -p ash-cli --all-targets`.
- Extended the Phase 201 removal gate to block stale engine module-loader path/file labels in
  active engine code: `workflow path`, `ordinary workflow`, `workflow file`, and
  `workflow source snapshot`.
- RED/GREEN verification after module-loader path/file label gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on those stale module-loader labels, then passed after retargeting comments and the
  parent-path diagnostic to source/module wording.
- Extended the Phase 201 removal gate to block removed callable/tower fixtures and the active
  module-loader `Act` opaque-type compatibility exception, including `A -> B`, `Fn(A)`, `act.ash`,
  `std::act`, `ActEnv`, `is_existing_opaque_compatibility_exception`, and the hard-coded `Act`
  type-name special case in selected engine module-loader paths.
- RED/GREEN verification after module-loader compatibility-exception gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on those stale fixtures and the active `Act` exception, then passed after removing
  the exception and retargeting fixtures to target callable syntax and neutral builtin handles;
  `cargo test -p ash-engine module_loader --lib -- --nocapture`.
- Extended the Phase 201 removal gate to block additional active engine and CLI fixtures from
  retaining removed callable syntax or workflow-file/path labels, including selected occurrences
  of `Fn(a)`, `Fn(Int)`, `A -> B`, `A -> M<B>`, `F<A -> B>`, `workflow file`, `workflow path`,
  and `workflow filename`.
- RED/GREEN verification after broader engine fixture gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on those selected active fixture labels, then passed after retargeting them to
  target callable syntax and source/entry wording.
- Extended the Phase 201 removal gate to block stale active `std/README.md` function-table
  callable syntax: `Fun(`, `Option<T> ->`, and `Result<T, E> ->`.
- RED/GREEN verification after stdlib README callable-table gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on those stale table forms, then passed after retargeting signatures to target
  parenthesized callable syntax.
- Extended the Phase 201 removal gate to block selected stale parser/engine legacy labels:
  `obligation reference (legacy)`, `legacy_impl_where`, and `legacy semicolon snippets`.
- RED/GREEN verification after parser/engine stale-label gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on those stale labels, then passed after retargeting them to current behavior
  wording.
- Extended the Phase 201 removal gate to block selected stale TypeEnv fallback labels:
  `legacy_nominal`, `legacy meta solving`, `legacy TypeEnv shape`, and
  `Unsupported legacy shapes`.
- RED/GREEN verification after TypeEnv fallback-label gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on those stale labels, then passed after retargeting them to current Type unifier
  fallback and noncanonical TypeEnv wording.
- Extended the Phase 201 removal gate to block selected TASK-826 TypeEnv forcing-point fallback
  labels: `legacy_meta_solving`, `legacy Type::Var`, `legacy_fallback`, and
  `Unsupported legacy shapes`.
- RED/GREEN verification after TASK-826 TypeEnv forcing-point gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on those stale labels, then passed after retargeting them to inference-meta,
  current `Type::Var`, fallback unifier, and noncanonical-shape wording.
- Extended the Phase 201 removal gate to block selected parser proposition where-bound labels:
  `preserves_legacy_impl_where`, `mask_legacy_impl_where`, and `legacy where bound`.
- RED/GREEN verification after parser proposition where-bound gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on those stale labels, then passed after retargeting them to current impl
  where-bound wording.
- Extended the Phase 201 removal gate to block selected parser removed-capability rejection labels:
  `legacy_capability` and `legacy_capabilities` in active parser module/lib tests.
- RED/GREEN verification after parser removed-capability rejection gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on those stale labels, then passed after retargeting them to removed-capability
  wording.
- Extended the Phase 201 removal gate to block the stale interpreter list-helper runtime label
  `legacy list runtime variant`.
- RED/GREEN verification after interpreter list-helper gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on that stale label, then passed after retargeting it to current Cons/Nil value
  wording.
- Extended the Phase 201 removal gate to block the stale `WorkflowContract` carrier field name
  `legacy_contract` in `crates/ash-core/src/workflow_carrier.rs`.
- RED/GREEN verification after `WorkflowContract` carrier-field gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on that stale field name, then passed after renaming it to `source_contract`.
- Extended the Phase 201 removal gate to block the stale core summary-schema test label
  `legacy payload decodes` in
  `crates/ash-core/tests/task_845_public_computation_summary_schema.rs`.
- RED/GREEN verification after core summary-schema older-payload label gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on that stale label, then passed after retargeting it to older-payload wording.
- Extended the Phase 201 removal gate to block the stale parser generated-identifier hygiene test
  label `legacy_generated_helpers` in
  `crates/ash-parser/tests/task_1746_generated_identifier_hygiene.rs`.
- RED/GREEN verification after parser generated-identifier hygiene gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on that stale label, then passed after retargeting it to generated-helper
  placeholder wording.
- Extended the Phase 201 removal gate to block stale core proposition summary schema labels:
  `legacy proposition fact`, `module_identity(version.0 as usize, "legacy")`, `legacy_payloads`,
  `before_legacy_registration`, `legacy_reject`, and `legacy summary version` in the active
  proposition summary carrier/schema/non-interference tests.
- RED/GREEN verification after core proposition summary schema gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on those stale labels, then passed after retargeting them to pre-V5 and
  older-payload wording.
- Extended the Phase 201 removal gate to block stale Type IR normal-form and parser process-row
  labels: `legacy/imported projection`, `imported or legacy carriers`, and
  `legacy_proc_surface`.
- RED/GREEN verification after Type IR/process-row label gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on those stale labels, then passed after retargeting them to imported
  pre-attribution carrier and removed-proc wording.
- Extended the Phase 201 removal gate to block stale runtime actor and older-summary fixture
  labels: `actor:legacy`, `capability:legacy.call`, `legacy-with-facts`, and
  `legacy-with-family` in the active runtime-kernel, external-actor, and summary-versioning tests.
- RED/GREEN verification after runtime actor/older-summary fixture gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on those stale fixture labels, then passed after retargeting them to unsupported
  actor/capability IDs and pre-version module IDs.
- Extended the Phase 201 removal gate to block stale parser/interpreter assertion labels and
  engine import-summary test naming: `preserve legacy vocabulary`, `!s.contains("legacy")`, and
  `legacy_type_leaks`.
- RED/GREEN verification after assertion/test-label gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on those stale labels, then passed after using positive current-wording assertions
  and public-representation transport naming.
- Extended the Phase 201 removal gate to block stale `ash.lock` redundant git-field labels:
  `legacy git URL`, `legacy_git`, `LegacyGit`, and `legacy git` in import resolution and registry
  metadata lock consumer tests.
- RED/GREEN verification after lockfile redundant-git vocabulary gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on those stale labels, then passed after retargeting them to redundant git-field
  wording.
- Extended the Phase 201 removal gate to block `#[allow(deprecated)]` suppressions in the active
  LLM provider chat and stream adapter tests.
- RED/GREEN verification after LLM deprecated-field suppression gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on those suppressions, then passed after rebuilding fixtures through JSON decoding
  and current defaulted stream-delta fields.
- Extended the Phase 201 removal gate to block stale ashgrove labels for `.ash.toml` metadata,
  `.source-rev`, redundant lockfile git metadata, and ignored-source sentinels.
- RED/GREEN verification after ashgrove label gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on those stale labels, then passed after retargeting them to superseded-manifest,
  direct `.source-rev`, redundant-git, and neutral sentinel wording.
- Extended the Phase 201 removal gate to block stale productive stdlib workflow-era comments in
  `std/src/lib.ash`, `std/src/runtime/error.ash`, and `std/src/llm/`.
- RED/GREEN verification after stdlib comment gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on those stale comments, then passed after retargeting them to target helper,
  entry execution, and orchestration wording.
- Extended the Phase 201 removal gate to block stale Phase 199/200 inventory-test labels and LSP
  `DocumentSymbol` deprecated protocol field literals.
- RED/GREEN verification after Phase 199/200 and LSP gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on those stale labels, then passed after retargeting tests to removed/historical
  vocabulary and constructing current LSP wire shapes through serde.
- Extended the Phase 201 removal gate to block the stale normalizer inference-meta boundary label
  `owned by the legacy` in `crates/ash-typeck/src/normalizer.rs`.
- RED/GREEN verification after normalizer inference-meta boundary gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on that stale label, then passed after retargeting it to existing `Type` unifier
  wording.
- Extended the Phase 201 removal gate to block selected typechecker semantic-summary rejection
  labels: `malformed_legacy_or_future_summary` and
  `legacy summary carrying computation fields`.
- RED/GREEN verification after typechecker semantic-summary rejection gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on those stale labels, then passed after retargeting them to malformed or
  unsupported summary wording.
- Extended the Phase 201 removal gate to block the stale TASK-876 proposition-solver assertion
  label `legacy unification/substitution/meta evidence facts`.
- RED/GREEN verification after TASK-876 proposition-solver assertion gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on that stale label, then passed after retargeting it to no-inversion/no-mutation
  evidence wording.
- Extended the Phase 201 removal gate to block the stale alpha visible-computation test label
  `legacy_surfaces`.
- RED/GREEN verification after alpha visible-computation test-label gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on that stale label, then passed after retargeting it to removed-surface wording.
- Extended the Phase 201 removal gate to block stale typechecker ambient-effect carrier names in
  active typechecker/runtime paths, including `workflow_effect` and `set_workflow_effect`.
- RED/GREEN verification after ambient-effect carrier gate expansion:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on those stale effect-carrier names, then passed after retargeting them to
  `ambient_effect` / `set_ambient_effect` and `entry_effect`; `cargo check -p ash-typeck -p
  ash-cli --all-targets`.
