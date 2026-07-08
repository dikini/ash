# TASK-1964: Type/Effect/Runtime Deprecated Carrier Removal

**Status:** Complete
**Phase:** [PLAN-201: Deprecated Functionality Removal](../PLAN-201-DEPRECATED-FUNCTIONALITY-REMOVAL.md)

## Description

Remove deprecated public type/effect/runtime vocabulary and carriers after target row, provider,
profile, process, contract, and evidence paths are proven sufficient.

## Requirements

- Use AUDIT-201 to identify type/effect/runtime carriers and report/trace vocabulary.
- Remove public deprecated tower vocabulary from type/effect/runtime APIs.
- Preserve current target row admission, provider/profile, process/channel, contract, evidence,
  report, and trace behavior.
- Update tests and diagnostics to target vocabulary.

## TDD Steps

1. Add failing tests that reject deprecated type/effect/runtime vocabulary in current paths.
2. Add positive tests for equivalent target row/profile/provider behavior.
3. Remove or rename deprecated carriers and update call sites.
4. Run typechecker, engine, runtime, process/channel, provider/profile, contract, and evidence
   gates.

## Completion Checklist

- [x] Deprecated public type/effect/runtime carriers are removed or renamed to target vocabulary.
- [x] Current row/admission/provider/profile behavior remains green.
- [x] Reports and traces use target vocabulary.
- [x] Contract/evidence and process/channel behavior remains green.
- [x] Focused runtime/type/effect gates pass.

## Evidence

- Retargeted `RoleChecker` from `WorkflowDef` inputs to explicit `RoleRef` slices so role
  capability composition no longer requires constructing deprecated workflow definition carriers.
- Updated `role_type_tests` and `role_checking` unit tests to exercise role-reference lists
  directly.
- Focused verification after this slice:
  `cargo check -p ash-typeck --all-targets`;
  `cargo test -p ash-typeck --test role_type_tests -- --nocapture`.
- Removed the unadvertised `ash-fuzz` typechecker fuzz target because it generated old `Workflow`
  carrier values directly and was not part of the documented current fuzz target list. Current
  advertised fuzz targets remain effect lattice and value roundtrip fuzzing.
- Verification after the fuzz target removal:
  `cargo metadata --manifest-path crates/ash-fuzz/Cargo.toml --no-deps --format-version 1`;
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`.
- Retargeted `proc::from_act` runtime and engine boundary coverage away from removed surface
  Act carriers. Tests now construct core Act closures directly where runtime behavior is under
  test, and obsolete stdlib-import coverage was removed after `std/src/proc.ash` left the active
  target stdlib surface.
- Verification after the Act runtime fixture retarget:
  `cargo test -p ash-engine --test task_719_proc_from_act_boundary -- --nocapture`;
  `cargo test -p ash-interp --test task_719_proc_from_act_runtime -- --nocapture`.
- Removed Core text row/effect compatibility aliases: operation rows/effects now parse only the
  target `operation` spelling, process rows/effects now parse only `process`, and serializers emit
  those canonical spellings. Active `.core` fixtures and expected strings were retargeted to the
  canonical vocabulary.
- Verification after Core text alias removal:
  `cargo test -p ash-core --test task_1622_core_text_parser_atoms_values --test task_1624_core_text_serializer --test task_1812_core_row_taxonomy_alignment --test task_1905_process_row_carriers -- --nocapture`;
  `cargo test -p ash-core --test task_1911_process_concurrency_core_cps -- --nocapture`;
  `cargo test -p ash-core --test task_736_capability_binding_carriers -- --nocapture`;
  `cargo test -p ash-core --all-targets`;
  `cargo check -p ash-core --all-targets`.
- Removed the typechecker validation path for removed `WorkflowDef` owned-resource and
  used-binding header carriers. Authority provenance for target workflow definitions now starts
  from current row/provider/runtime metadata instead of direct `owns`/`uses` workflow headers.
- Focused verification after removing the direct typechecker path:
  `cargo check -p ash-parser -p ash-typeck -p ash-engine -p ash-interp --all-targets`.
- Retargeted the runtime resource-admission API away from workflow-header ownership vocabulary:
  `WorkflowOwnedResourceAdmission` became `EntryOwnedResourceAdmission`, and
  `admit_workflow_owned_resources` became `admit_entry_owned_resources`.
- Retargeted runtime provenance note text away from source-shaped removed capability declarations:
  dependency notes now use `resource source` and `binding source` wording instead of
  declaration-shaped `resource <name>:` / `capability <name>:` prefixes.
- Removed legacy module-graph crate membership aliases: active callers now use
  `crate_id_for_module` and `assign_module_to_crate` directly instead of the old compatibility
  `crate_for` / `set_crate` methods and compatibility-only unit test.
- Removed the legacy `ash-interp::execute_workflow` wrapper that constructed an empty
  `BehaviourContext` implicitly. Active interpreter tests now call `execute_workflow_with_behaviour`
  directly, and the old wrapper is no longer exported.
- Removed provider authoring compatibility shims: `ProviderAuthoringMetadata` no longer carries a
  shim marker or wildcard shim constructor, providers without explicit operation metadata fail
  closed, runtime host-binding admission no longer bypasses row validation for shim metadata, and
  custom-provider tests now declare target provider rows explicitly.
- Removed dotted qualified-name compatibility parsing from `ash-typeck`: `QualifiedName::parse`
  now rejects `.` separators and accepts only target `::` separators for module-qualified names.
- Retargeted `check_pattern` registered-variant fallback vocabulary away from legacy naming while
  preserving current generic ADT pattern behavior.
- Removed the interpreter terminal-observed execution path that created an ambient provider context
  when no RuntimeKernel capability bindings were admitted. All capability contexts in that path now
  come from explicit admitted binding ids, and interpreter `MockProvider` rows declare authored
  test operation metadata.
- Focused verification after the runtime admission rename:
  `cargo test -p ash-core --test task_736_capability_binding_carriers -- --nocapture`;
  `cargo test -p ash-interp --test task_736_capability_binding_admission -- --nocapture`;
  `cargo test -p ash-interp --test task_737_internal_authority_allocation -- --nocapture`;
  `cargo test -p ash-interp --test task_738_derived_authority_non_widening -- --nocapture`;
  `cargo test -p ash-interp --test task_740_runtime_resource_binding_integration -- --nocapture`;
  `cargo test -p ash-interp --test task_741_ash_defined_capability_implementation_execution -- --nocapture`;
  `cargo test -p ash-interp --test task_742_capability_examples -- --nocapture`;
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`;
  `cargo check -p ash-core -p ash-interp --all-targets`.
- Focused verification after module-graph alias removal:
  `cargo test -p ash-core module_graph --lib -- --nocapture`;
  `cargo test -p ash-parser import_resolver::tests --lib -- --nocapture`;
  `cargo test -p ash-parser --test visibility_integration_test -- --nocapture`.
- Focused verification after removing the legacy interpreter wrapper:
  `cargo test -p ash-interp --test par_removal_tests -- --nocapture`;
  `cargo check -p ash-interp --all-targets`.
- Focused verification after provider shim removal:
  `cargo test -p ash-core --test task_1927_provider_authoring_metadata -- --nocapture`;
  `cargo test -p ash-interp --test task_1927_provider_authoring_admission -- --nocapture`;
  `cargo test -p ash-engine --test task_1927_provider_authoring_api -- --nocapture`;
  `cargo test -p ash-engine --test task_1940_standard_provider_profiles -- --nocapture`;
  `cargo test -p ash-engine --test e2e_capability_provider_tests -- --nocapture`;
  `cargo test -p ash-engine --test provider_wiring_test -- --nocapture`;
  `cargo check -p ash-core -p ash-interp -p ash-engine --all-targets`.
- Focused verification after qualified-name separator removal:
  `cargo test -p ash-typeck qualified_name_parse_dot_separator_is_removed --lib -- --nocapture`.
- Focused verification after pattern fallback vocabulary retarget:
  `cargo test -p ash-typeck check_pattern --lib -- --nocapture`;
  `cargo test -p ash-typeck --test task_1375b_partial_match_detection -- --nocapture`.
- Focused verification after interpreter ambient provider fallback removal:
  `cargo test -p ash-interp test_terminal_observed_execution_without_runtime_admission_is_fail_closed --lib -- --nocapture`;
  `cargo test -p ash-interp --test task_736_capability_binding_admission -- --nocapture`.
- Retargeted row-admission contract diagnostics away from legacy contract-row wording. Contract
  row items now report current contract-discharge record requirements.
- Focused verification after the contract row-admission vocabulary cleanup:
  `cargo test -p ash-engine --test task_1896_1897_evidence_contract_discharge -- --nocapture`.
- Retargeted typechecker interface-evidence lowering helpers away from legacy-type vocabulary.
  Interface evidence arguments now lower through current `interface_evidence_arg_as_type*`
  helpers.
- Removed the stale active TASK-1023 tower-algebra evidence integration test after verification
  showed it still asserted Act/Proc/Workflow carrier evidence over current stdlib algebra modules.
- Focused verification after the interface-evidence helper vocabulary cleanup:
  `cargo check -p ash-typeck --all-targets`.
- Retargeted active interpreter builtin fallback wording away from legacy terminology. The
  `eval_expr` function-call path now describes fallback dispatch as the current pattern-matched
  builtin path for unqualified builtins.
- Focused verification after the builtin fallback vocabulary cleanup:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`;
  `cargo check -p ash-interp -p ash-cli --all-targets`.
- Added honest forward dispatch-table entries for current provider-backed `llm::dispatch` stdlib
  builtin declarations so active stdlib target surfaces fail closed as unimplemented interpreter
  builtins instead of being reported as missing runtime metadata.
- Focused verification after the LLM builtin dispatch-table repair:
  `cargo test -p ash-interp --test builtin_dispatch -- --nocapture`;
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`.
- Retargeted typechecker do-target dictionary comments away from legacy fallback terminology. The
  current built-in dictionary bridge is now described as active registered computation dictionary
  behavior while explicit `Monad` evidence is absent.
- Focused verification after the do-target dictionary wording cleanup:
  `cargo test -p ash-typeck do_target --lib -- --nocapture`;
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`.
- Retargeted `ash-engine` entry verification vocabulary away from entry-workflow wording.
  The active verifier helper now uses entry-definition/source terminology, user-facing
  diagnostics avoid workflow wording, and entry integration fixtures use target `fn main`,
  `capability Args`, and explicit `Ok`/`Err` result bodies.
- Focused verification after the entry verification retarget:
  `cargo test -p ash-engine --test entry_verification -- --nocapture`.
- Removed the typechecker empty-provider compatibility fallback. Explicit `provider:action`
  targets now require a registered provider even when no providers have otherwise been registered,
  and existing direct-action tests declare their provider dependency explicitly.
- RED/GREEN verification after provider fail-closed removal:
  `cargo test -p ash-typeck --test phase201_provider_fail_closed -- --nocapture` first failed
  because the checker accepted `missing_provider:noop`, then passed after the fallback removal;
  `cargo test -p ash-typeck --test task_1004_workflow_binder_irrefutability -- --nocapture`;
  `cargo test -p ash-typeck --test workflow_binding_paths_task_423 -- --nocapture`;
  `cargo test -p ash-typeck --lib -- --nocapture`;
  `cargo check -p ash-typeck --all-targets`.
- Removed the legacy Core operation-row storage carrier. Operation requirements now use
  `CoreRowItem::Operation`, `CoreRowItem::operation` constructs that target variant directly, and
  public Core row summaries use `CorePublicRowItemSummary::Operation`.
- RED/GREEN verification after Core operation carrier removal:
  `cargo test -p ash-core --test task_1812_core_row_taxonomy_alignment -- --nocapture` first
  failed because `CoreRowItem::Operation` and `CorePublicRowItemSummary::Operation` were missing,
  then passed after the carrier rename; `cargo test -p ash-core --test
  task_1649_core_public_summary -- --nocapture`; `cargo check -p ash-core --all-targets`;
  `cargo check -p ash-engine --all-targets`.
- Removed the legacy Core raised-operation carrier name. Raised operation effects now use
  `CoreEffectOp::Operation`, and operation-effect tests/helper names were retargeted away from
  capability-row vocabulary.
- RED/GREEN verification after Core raised-operation carrier removal:
  `cargo test -p ash-core --test task_1646_core_effect_operation_typing -- --nocapture` first
  failed because `CoreEffectOp::Operation` was missing, then passed after the carrier rename;
  `cargo test -p ash-core --test task_1624_core_text_serializer -- --nocapture`;
  `cargo test -p ash-core --test task_736_capability_binding_carriers -- --nocapture`;
  `cargo check -p ash-core --all-targets`;
  `cargo check -p ash-engine --all-targets`.
- Broader verification after operation carrier retargeting:
  `cargo test -p ash-core --all-targets`;
  stale-carrier scan for `CoreRowItem::Capability`, `CorePublicRowItemSummary::Capability`,
  `CoreEffectOp::Capability`, and operation-only helper names.
- Retargeted CPS resume-row metadata diagnostics and tests away from legacy inherited-row wording.
  `ResumeRowMetadata::InheritFromTarget` remains the current affine inferred-target-row path, while
  multi-shot-pure validation now reports inherited target rows without legacy terminology.
- RED/GREEN verification after CPS inherited-row wording cleanup:
  `cargo test -p ash-interp --test task_1683_cps_multishot_validation
  reject_multishot_handler_inherited_row -- --nocapture` first failed because the diagnostic still
  emitted `legacy inherit-from-target`, then passed after the diagnostic retarget; `cargo test -p
  ash-interp --test task_1683_cps_multishot_validation -- --nocapture`; `cargo test -p ash-interp
  --test task_1682_cps_multishot_runtime -- --nocapture`.
- Retargeted RuntimeKernel synthetic artifact metadata away from workflow vocabulary. The
  verifier-normalized synthetic TCIR now reports `ApplicationEntry` / `RuntimeKernel<ApplicationEntry>`
  and daemon/one-shot artifact summaries use the checked application-entry carrier scope instead
  of the old alpha checked workflow-boundary label.
- Retargeted compiler-known contract intrinsic carriers away from workflow-intrinsic vocabulary.
  Active typechecker APIs now use `ContractIntrinsic*`, `contract_intrinsics`,
  `lookup_contract_intrinsic`, and `__contract_intrinsic_context`, and user-facing misuse
  diagnostics now describe application contract/result boundaries instead of workflow intrinsic
  context.
- Removed the obsolete `ash-typeck::capability_check` workflow-surface verifier. The exported
  `CapabilityChecker` API walked removed `ash_parser::surface::Workflow` carriers directly and
  was only exercised by stale tests; current provider, row-admission, contract, runtime, and
  aggregate verification paths remain the active target mechanisms.
- Removed obsolete interpreter tests that still entered runtime coverage through
  `parse_workflow::workflow_def`, `lower_workflow`, or direct `SurfaceWorkflow` construction.
  Current receive/runtime-boundary behavior remains covered by target parser/runtime tests and
  lower-level stream/yield/process tests.
- Retargeted Par-removal regression labels away from the deleted `SurfaceWorkflow::Par` and
  capability-checker vocabulary. The tests now describe the current removed-Par invariant in
  provider/action validation terms.
- Retargeted runtime/typechecker capability requirement carriers away from workflow vocabulary.
  `WorkflowCapabilities` became `EntryCapabilities`, aggregate runtime verification inputs now use
  `entry_capabilities`, and focused runtime policy/verification tests use entry-capability naming.
- RED/GREEN verification after the RuntimeKernel artifact retarget:
  `cargo test -p ash-engine --test alpha_runtime_kernel_artifact_builder
  engine_builder_is_host_agnostic_for_one_shot_and_daemon_callers -- --nocapture` first failed
  with `Workflow` / `alpha_checked_workflow_boundary`, then passed after the artifact/scope
  retarget; `cargo test -p ash-engine --test alpha_runtime_kernel_artifact_builder -- --nocapture`;
  `cargo test -p ash-cli --test alpha_ash_run_runtime_kernel_mode -- --nocapture`;
  `cargo test -p ash-cli --test alpha_admission_profile -- --nocapture`;
  `cargo test -p ash-cli --test alpha_run_daemon_artifact_equivalence -- --nocapture`.
- RED/GREEN verification after the contract-intrinsic carrier rename:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on active `WorkflowIntrinsic`, `workflow_intrinsics`,
  `lookup_workflow_intrinsic`, `workflow_intrinsic`, and `__workflow_intrinsic_context`
  occurrences, then passed after the typechecker carrier rename;
  `cargo check -p ash-typeck -p ash-cli --all-targets`;
  `cargo test -p ash-typeck --test alpha_visible_computation_manifest -- --nocapture`.
- Closeout update: the obsolete workflow-algebra suite was deleted during TASK-1968 because it
  asserted removed `do:Workflow` tower-carrier behavior rather than target `Monad<K>` evidence.
- RED/GREEN verification after removing the obsolete workflow-surface capability checker and
  interpreter workflow-parser tests:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `capability_check`, `CapabilityChecker`,
  `parse_workflow::workflow_def`, `lower_workflow`, and `SurfaceWorkflow` in the selected active
  typechecker/interpreter paths, then passed after deleting the stale module/tests;
  `cargo check -p ash-typeck -p ash-interp -p ash-cli --all-targets`.
- Focused stale-token scan after the capability-checker/interpreter-test removal and Par-label
  cleanup:
  `rg -n 'capability_check|CapabilityChecker|CapabilityCheckError|parse_workflow::workflow_def|SurfaceWorkflow|capability_checking|SurfaceWorkflow::Par' crates/ash-typeck/src crates/ash-typeck/tests crates/ash-interp/src crates/ash-interp/tests --glob '!crates/ash-cli/tests/phase201_deprecated_functionality_removal_gate.rs'`
  returned no matches.
- Closeout update: obsolete receive/workflow-parser and Act/Proc/Workflow bridge suites were
  deleted or retargeted before the final workspace gate.
- RED/GREEN verification after the capability-requirement carrier rename:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `WorkflowCapabilities` in active obligation/runtime verification paths and
  focused tests, then passed after retargeting those APIs and tests to `EntryCapabilities`;
  `cargo check -p ash-typeck -p ash-cli --all-targets`;
  `cargo test -p ash-typeck --test runtime_verification_contracts --test
  runtime_verification_input_contracts --test policy_runtime_outcomes -- --nocapture`.
- Removed the role-runtime dependency on deprecated `WorkflowDef` carriers. `RoleRegistry` now
  resolves explicit `RoleRef` slices plus admitted capability declarations, and role-runtime tests
  exercise role/capability resolution without constructing a synthetic workflow definition.
- RED/GREEN verification after the role-runtime carrier retarget:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `WorkflowDef` in `ash-interp::role_runtime`, `role_runtime_tests`, and
  `role_runtime_integration_tests`, then passed after the API/test retarget;
  `cargo test -p ash-interp role_runtime --lib -- --nocapture`;
  `cargo test -p ash-interp --test role_runtime_tests -- --nocapture`;
  `cargo test -p ash-engine --test role_runtime_integration_tests -- --nocapture`.
- Removed RuntimeKernel workflow identity carrier names from active application-runtime APIs.
  `WorkflowDefinitionId` / `WorkflowDefinitionIdentity`, `WorkflowArtifactId` /
  `WorkflowArtifactIdentity`, and `WorkflowInstanceId` / `WorkflowInstanceIdentity` are now
  `ApplicationDefinitionId` / `ApplicationDefinitionIdentity`, `ApplicationArtifactId` /
  `ApplicationArtifactIdentity`, and `ApplicationInstanceId` / `ApplicationInstanceIdentity`.
  Process-tree carriers now expose `application_instance_id`, and runtime artifact builder
  identity inputs expose `entry_name`.
- RED/GREEN verification after the RuntimeKernel identity-carrier retarget:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on the old RuntimeKernel workflow identity carrier names in `ash-core`,
  `ash-cli`, and the core carrier test, then passed after the application/entry rename;
  `cargo test -p ash-core --test alpha_runtime_kernel_carriers -- --nocapture`;
  `cargo test -p ash-core --test alpha_runtime_kernel_artifact_builder -- --nocapture`;
  `cargo test -p ash-cli --test alpha_ash_run_runtime_kernel_mode -- --nocapture`;
  `cargo test -p ash-cli --test alpha_run_daemon_artifact_equivalence -- --nocapture`;
  `cargo check -p ash-core -p ash-cli --all-targets`;
  `cargo clippy -p ash-core -p ash-cli --all-targets -- -D warnings`;
  `cargo fmt --all --check`.
- Removed lower runtime workflow admission/boundary carrier names from active application-runtime
  APIs. Admission request/outcome/requirement carriers, application admission context, structured
  contract evidence, application boundary outcomes, reports, failures, and the engine
  admitted-boundary wrapper now use `Application*` names. The admission test action provider now
  declares explicit operation metadata so it satisfies the current fail-closed provider authoring
  API.
- RED/GREEN verification after the lower application-boundary carrier retarget:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on the old workflow-named lower runtime boundary carriers, then passed after the
  application-boundary rename; `cargo check -p ash-core -p ash-engine -p ash-interp --all-targets`;
  `cargo test -p ash-core --test task_714_workflow_boundary_carriers -- --nocapture`;
  `cargo test -p ash-core --test task_715_contract_evidence_schema -- --nocapture`;
  `cargo test -p ash-core --test task_716_workflow_report_completion_red -- --nocapture`;
  `cargo test -p ash-engine --test task_715_workflow_admission_red -- --nocapture`;
  `cargo test -p ash-engine --test task_716_workflow_completion_red -- --nocapture`;
  `cargo test -p ash-engine --test task_719_proc_from_act_boundary -- --nocapture`;
  `cargo test -p ash-interp --test task_714_workflow_boundary_exec_error -- --nocapture`;
  `cargo check -p ash-core -p ash-engine -p ash-interp -p ash-cli --all-targets`;
  `cargo clippy -p ash-core -p ash-engine -p ash-interp -p ash-cli --all-targets -- -D warnings`;
  `cargo fmt --all --check`.
- Removed the typechecker hard-coded do-target tower fallback path. `resolve_do_target` now
  admits named computation constructors only through explicit `Monad` evidence, no longer
  synthesizes built-in Act/Proc/Workflow dictionaries or hidden Act bind/return operations, and no
  longer maps Act/Proc/Workflow evidence to intrinsic shim names.
- RED/GREEN verification after do-target tower fallback removal:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `DoTowerLevel`, hidden Act operation carriers, the old tower dictionary resolver,
  tower diagnostics, and hard-coded Act/Proc/Workflow intrinsic shim strings in
  `crates/ash-typeck/src/do_target.rs`, then passed after the fallback removal;
  `cargo test -p ash-typeck do_target --lib -- --nocapture`;
  `cargo check -p ash-typeck --all-targets`;
  `cargo clippy -p ash-typeck -p ash-cli --all-targets -- -D warnings`;
  `cargo fmt --all --check`.
- Retargeted runtime/Core failure attribution carriers away from tower vocabulary. `TowerLevel`
  became `FailureBoundary`, `OperationalFailure::tower` became `boundary`, process/application
  variants replaced old Proc/Workflow attribution variants, TCIR/AMIR computation provenance now
  carries `boundary_level` and explicit cross-boundary lift fields, and daemon failure reports now
  serialize boundary/application failure labels instead of tower/workflow labels.
- Retargeted the typechecker public algebra manifest API and focused tests from tower labels to
  computation labels. `PublicTower*` APIs are now `PublicComputation*`, the visible-manifest and
  acceptance-matrix tests were renamed, and live TASK-931 audit/task evidence was updated to
  reference the current computation test paths.
- Focused verification after runtime/Core boundary retargeting:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on the old runtime/Core tower carrier names, then passed after the boundary rename;
  `cargo check -p ash-core -p ash-typeck -p ash-engine -p ash-interp -p ash-cli --all-targets`;
  `cargo test -p ash-core --test alpha_tcir_computation_expression -- --nocapture`;
  `cargo test -p ash-typeck --test alpha_visible_computation_manifest -- --nocapture`;
  `cargo test -p ash-typeck --test alpha_visible_computation_acceptance_matrix -- --nocapture`;
  `cargo test -p ash-interp --test task_1023_computation_runtime_algebra -- --nocapture`.
- Removed the typechecker instance/control-link `workflow_type` field carrier. `Type::Instance`,
  `Type::InstanceAddr`, and `Type::ControlLink` now store `entry_type`, and the engine
  monomorphizer construction sites use the same target carrier name.
- RED/GREEN verification after the type instance-carrier retarget:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `workflow_type` in `crates/ash-typeck/src/types.rs`, then passed after the
  carrier rename; `cargo check -p ash-typeck -p ash-engine --all-targets`.
- Removed the runtime spawn/instance `workflow_type` carrier token. Core spawn/value carriers,
  interpreter spawn execution, parser lift fixtures, engine child-entry registration, and CLI
  value conversion now use `entry_type` for target application entries.
- RED/GREEN verification after the runtime spawn/instance-carrier retarget:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `workflow_type` in the gated core/interpreter/engine runtime paths, then passed
  after the carrier rename; `cargo check -p ash-core -p ash-parser -p ash-interp -p ash-engine -p
  ash-cli -p ash-typeck --all-targets`.
- Removed the runtime callable/admission `workflow_name` carrier token from active
  engine/interpreter source paths. Runtime callable-entry registration and lookup now use
  `entry_name`, and `ApplicationAdmissionRequest` exposes `entry_name` for admission/reporting.
- RED/GREEN verification after the callable/admission name-carrier retarget:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `workflow_name` in `crates/ash-interp/src/runtime_state.rs` and
  `crates/ash-engine/src/lib.rs`, then passed after the carrier rename; `cargo check -p
  ash-interp -p ash-engine -p ash-cli --all-targets`; `cargo test -p ash-interp
  callable_workflow --lib -- --nocapture`; `cargo test -p ash-engine --test
  runtime_boundary_visibility registered_callable -- --nocapture`; `cargo test -p ash-engine
  --test task_715_workflow_admission_red -- --nocapture`.
- Removed stale callable-workflow registry API identifiers. Runtime callable registration now uses
  `RegisteredCallableEntry`, `callable_entries`, `register_callable_entry`,
  `blocking_register_callable_entry`, and `callable_entry`; engine test APIs and dynamic-contract
  fixtures use the same callable-entry vocabulary.
- RED/GREEN verification after callable registry API retarget:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `RegisteredCallableWorkflow`, `callable_workflows`,
  `register_callable_workflow`, `blocking_register_callable_workflow`, and `callable_workflow` in
  active engine/interpreter source paths, then passed after the registry API rename; `cargo check
  -p ash-interp -p ash-engine -p ash-cli --all-targets`; `cargo test -p ash-interp
  callable_entry --lib -- --nocapture`; `cargo test -p ash-engine --test
  runtime_boundary_visibility registered_callable_entry -- --nocapture`; `cargo test -p
  ash-engine --test task_1898_dynamic_contract_runtime_checks -- --nocapture`.
- Removed stale child-workflow registry API identifiers. Runtime spawned-child registration now
  uses `child_entries`, `register_child_entry`, and `child_entry`; the engine embedding API and
  spawned-child tests use the same child-entry vocabulary.
- RED/GREEN verification after child registry API retarget:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `child_workflows`, `register_child_workflow`, and `child_workflow` in active
  engine/interpreter source paths, then passed after the registry API rename; `cargo check -p
  ash-interp -p ash-engine -p ash-cli --all-targets`; `cargo test -p ash-interp child_entry --lib
  -- --nocapture`; `cargo test -p ash-interp --test runtime_boundary_visibility spawn --
  --nocapture`. Final Phase 201 closeout verification passed the full workspace test gate after
  provider metadata fixtures were retargeted to explicit admitted rows.
- Removed stale runtime workflow-projection wrapper identifiers from the interpreter/engine
  boundary surface. The interpreter module is now `entry_projection`, wrapper APIs are
  `execute_entry_proc_projection` and `unsupported_entry_proc_projection_message`, the engine
  forwarding API uses the same entry-projection vocabulary, and the focused unsupported diagnostic
  label is `FirstClassEntryProjectionExecutionUnsupported`.
- RED/GREEN verification after entry projection wrapper retarget:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `workflow_projection`, `execute_workflow_proc_projection`, and
  `unsupported_workflow_proc_projection_message` in active interpreter/engine wrapper paths, then
  passed after the wrapper rename; `cargo check -p ash-interp -p ash-engine -p ash-cli
  --all-targets`; `cargo test -p ash-interp --test task_774_entry_projection_boundary --
  --nocapture`; `cargo test -p ash-engine --test task_774_entry_projection_engine_boundary --
  --nocapture`.
- Removed stale TCIR/AMIR workflow-artifact carrier identifiers from the active computation
  artifact surface. Core computation expressions, AMIR/bytecode opcode carriers, typechecker
  elaboration results, runtime artifact construction, and focused tests now use entry-artifact
  identifiers such as `entry_artifact`, `TcirEntryArtifactProvenance`, `EntryArtifact`,
  `EntryTypedArtifact`, and `EntryArtifactBuilder`.
- RED/GREEN verification after TCIR/AMIR artifact-carrier retarget:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `TcirWorkflowArtifactProvenance`, `workflow_artifact`, `WorkflowArtifact`, and
  `WorkflowArtifactBuilder` in active core/typechecker/test paths, then passed after the
  entry-artifact rename; `cargo check -p ash-core -p ash-typeck -p ash-engine -p ash-cli
  --all-targets`; `cargo test -p ash-core --test alpha_tcir_computation_expression --
  --nocapture`; `cargo test -p ash-core --test alpha_amir_bytecode_schema -- --nocapture`;
  `cargo test -p ash-core --test alpha_runtime_kernel_artifact_builder -- --nocapture`;
  `cargo test -p ash-typeck --test alpha_visible_computation_acceptance_matrix -- --nocapture`.
  Final Phase 201 closeout removed or retargeted the older workflow/tower-carrier suites that
  preserved implicit `Act`/`Proc`/`Workflow` evidence expectations.
- Removed stale typechecker `workflow_effect` carrier identifiers from active ambient-effect and
  runtime effect-checking APIs. `TypeEnv` now stores and exposes `ambient_effect`, callers enter
  that context with `set_ambient_effect`, and runtime/obligation checks name the derived bound as
  `entry_effect`.
- RED/GREEN verification after ambient/entry effect-carrier retarget:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `workflow_effect` / `set_workflow_effect` in active typechecker, runtime
  verification, obligation-checker, and focused test paths, then passed after retargeting;
  `cargo check -p ash-typeck -p ash-cli --all-targets`; `cargo test -p ash-typeck task558 --lib
  -- --nocapture`; `cargo test -p ash-typeck --test effect_runtime_alignment -- --nocapture`;
  `cargo test -p ash-typeck --test task_959_pure_closure_arrow -- --nocapture`.
- Retargeted the `WorkflowContract` source-contract carrier away from legacy naming. The active
  public field is now `source_contract`, and the default workflow-form lowering construction uses
  that target-neutral name.
- RED/GREEN verification after `WorkflowContract` source-contract carrier retarget:
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture`
  first failed on `legacy_contract`, then passed after the field rename; `cargo check -p ash-core
  -p ash-cli --all-targets`.
