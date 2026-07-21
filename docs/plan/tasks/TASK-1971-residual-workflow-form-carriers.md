# TASK-1971: Residual Workflow-Form Carrier Removal

**Status:** In progress
**Phase:** [PLAN-201 Semantic Cleanup Follow-up](../PLAN-201-SEMANTIC-CLEANUP-FOLLOWUP.md)
**Source audit:** [AUDIT-201 Semantic Removal Vs Rename](../audits/AUDIT-201-semantic-removal-vs-rename.md)

## Description

Remove residual workflow-form parser and lowering carriers that are not needed for current target
Ash contracts. Target function contracts should lower directly through contract/evidence helpers
rather than old declaration adapters.

## Requirements

- Identify residual workflow-form parser/lowering carriers still reachable from target contract
  paths.
- Prove current `requires` and `ensures` contract paths do not need removed workflow declaration
  adapters.
- Delete or rewrite workflow-form-only tests.
- Add absence tests or gates so removed declaration adapters cannot re-enter active lowering paths.
- Preserve current target function contract parsing, lowering, checking, and engine metadata.

## TDD Steps

1. Add or tighten tests that expose residual workflow-form carrier use in parser/lowering paths.
2. Prove target function contract events lower directly through contract/evidence helpers.
3. Remove or confine stale workflow-form carriers and rewrite affected tests.
4. Run parser, typechecker, engine metadata, Phase 201 gate, and docs/index checks.

## Completion Checklist

- [x] Workflow-form-only parser/lowering carriers are removed.
- [x] Target function `requires`/`ensures` paths lower without the removed workflow-header
      declaration adapter.
- [x] Workflow-form-only parser/lowering tests and fixtures no longer construct
      `WorkflowDef.header_events` or `WorkflowDef.contract`.
- [x] Phase 201 removal gates cover the removed workflow-header and lowerer whole-definition
      carriers.
- [x] `Program` entry storage uses target function metadata instead of workflow-definition
      carriers.
- [x] Active parser module files no longer carry `ModuleFile.workflow`, and the active parser no
      longer exposes `parse_workflow::workflow_def`.
- [x] Parser surface, typechecker, and stale core AST/contract copies no longer expose
      `WorkflowDef` carriers.
- [x] Active parser grammar no longer accepts expression-level `act { ... }` or explicit
      `do:Act` / `do:Proc` / `do:Workflow` tower targets.
- [x] Surface `DoStmt` no longer carries workflow-specific `requires:` / `ensures:` contract
      variants.
- [x] Active parser grammar no longer exposes workflow/proxy/yield/receive parser modules.
- [x] Surface parser AST no longer exposes `Workflow`, proxy, action, check-target, receive/yield,
      or action-guard carriers.
- [x] Typechecker no longer exports workflow typecheck, workflow effect inference, workflow name
      resolution, or workflow runtime-verification modules.
- [x] Core contract helpers no longer expose the stale `workflow_contract::Workflow` enum.
- [x] Core AST no longer exposes `ash_core::Workflow`, proxy workflow carriers, workflow receive
      arm carriers, or workflow definition carriers.
- [x] Core no longer exposes the first-class `workflow_carrier` module or TCIR/AMIR workflow
      entry-artifact carriers.
- [x] Parser no longer exposes core workflow lifting passes (`lift_workflow*`).
- [x] Core/interpreter no longer expose workflow-only small-step/visualization helper modules or
      the CLI workflow DOT visualization command.
- [x] Engine target entry parsing/admission/execution stores lowered function bodies as direct core
      expressions instead of `ash_core::Workflow::Ret` shells.
- [x] Engine public parsed-entry handle is named `Entry` instead of `Workflow`, and CLI/REPL
      call sites no longer reference `ash_engine::Workflow`.
- [x] Interpreter no longer exposes workflow executor, stream executor, yield-state/yield-router
      modules, or runtime-state workflow body caches.
- [x] TypeEnv no longer exposes public `Workflow<T>` or `workflow::*` computation intrinsics.
- [x] Contract helpers use neutral `contract` module/classifier paths instead of
      `workflow_contract` module paths.
- [x] `CHANGELOG.md` records the in-progress removal slice.

## Evidence

2026-07-21 tooling and stale-test cleanup:

- Removed obsolete `Definition::Proxy` branches from LSP completion, hover, navigation, document
  symbols, and symbol indexing, including the stale proxy keyword completion and hover metadata.
- Removed the stale `ModuleFile.workflow` LSP test fixture initializer, the unused typechecker
  `DefinitionKind::Proxy` variant, the obsolete `#[workflow]` macro integration test, and the
  stale MCP cached-AST workflow assertion.
- Removed the orphaned interpreter `ExecutionRecorder` and test-only `RuntimeState::control_registry`
  accessor exposed by removed execution paths; control-link coverage now uses the public runtime API.
- Removed obsolete CLI `dot` command coverage and aligned undefined-function and capability-call
  assertions with the current diagnostic contract.
- Reclassified the proxy-workflow specification index/readme entry as historical and removed
  Proxy-only current lint/formatter specification examples. The current grammar, lint, and
  formatter specifications no longer describe removed proxy/workflow module carriers.
- Verification: `cargo test -p ash-lsp-core --test phase200_lsp_migration_polish`;
  `cargo check -p ash-lsp-core --all-targets`; `cargo test -p ash-macros`;
  `cargo check -p ash-macros --all-targets`; `cargo check -p ash-typeck --all-targets`;
  `cargo test -p ash-cli --test cli -- --test-threads=1`;
  `cargo fmt --check`; `python3 tools/docs/validate_orientation_indexes.py --self-test`; and
  `bash scripts/check-docs-gate.sh`.

2026-07-21 runtime-kernel failure-report follow-up:

- Replaced the removed `panic` surface fixture in the CLI runtime-kernel failure regression with
  a valid target `Err(RuntimeError(...))` return, so it reaches the admitted execution boundary.
- Runtime-kernel reporting now classifies nonzero target-entry exits as failed while preserving the
  CLI's existing exit code and output behavior.
- Verification: `cargo test -p ash-cli --test alpha_admission_profile
  execution_failure_still_emits_runtime_kernel_report -- --exact`; `cargo test -p ash-cli --test
  alpha_admission_profile`; and `cargo check -p ash-cli --all-targets`.

2026-07-21 daemon protocol cleanup:

- Retargeted daemon protocol serialization from stale `workflow` keys to `application` keys for
  start requests and definition/instance/status records, while leaving internal bookkeeping
  unchanged.
- Verification: `cargo test -p ash-cli --test alpha_ashd_local_daemon_control_plane --
  --test-threads=1`; `cargo check -p ash-cli --all-targets`; and `cargo fmt --check`.

2026-07-11 slice:

- Removed `WorkflowHeaderEvent`, `WorkflowDef.header_events`, and `WorkflowDef.contract` from the
  surface AST and parser.
- Removed workflow-header `requires:` / `ensures:` adaptation from the old workflow-definition
  parser.
  Target function contracts continue to parse through `FnDef.contract` and lower through
  `lower_fn_contract`.
- Removed parser lowering's whole-definition workflow entry point from active program-entry paths.
- Retargeted synthesized contract discovery to function contracts only.
- Added Phase 201 gate rows for `WorkflowHeaderEvent`, `header_events`,
  `parse_workflow_header_events`, and `WorkflowDef` in active parser lowering.
- Verification run for this slice:
  `cargo check -p ash-parser -p ash-engine -p ash-typeck --all-targets`;
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate`;
  `cargo test -p ash-parser`;
  `cargo test -p ash-typeck --test pure_function_contracts_task_505`;
  `cargo test -p ash-cli --test test_command only_synthesized_function_contract_module_uses_live_checked_snapshot`.

2026-07-11 follow-up slice:

- Replaced `Program.workflow` and helper workflow entry storage with `ProgramEntry` function
  metadata.
- Removed engine storage for surface workflow definitions and removed synthesized `fn main`
  surface workflow bodies.
- Lowered `fn main` bodies directly from expressions at the current interpreter boundary instead
  of fabricating a surface workflow call wrapper.
- Removed the stale public `lower_entry_body` / `lower_entry_body_with_context` wrappers after the
  engine stopped using surface workflow entry bodies.
- Removed `ModuleFile.workflow`, the public `parse_workflow::workflow_def` parser, error-recovery
  routing to workflow-definition parsing, and the remaining parser tests that parsed workflow
  definitions directly.
- Typechecked selected program entries through `FnDef` bodies and ran fn contract precondition
  validation directly over expression bodies.
- Retargeted affected program-entry tests to `FnDef` fixtures; legacy workflow API tests remain
  isolated on legacy workflow typechecker entry points rather than being adapted into `Program`.

2026-07-11 carrier deletion slice:

- Removed `ash_parser::surface::WorkflowDef`.
- Removed `ash_typeck::type_check_workflow_def` and
  `ash_typeck::type_check_workflow_def_in_env`.
- Deleted or retargeted tests that constructed workflow definitions to preserve feature behavior.
- Removed stale core `WorkflowDef` carriers from `ash_core::ast` and
  `ash_core::workflow_contract`.
- Removed lexer keyword tokens/mapping for `workflow` and `act`; they now lex as identifiers.
- Removed the parser `proc` computation-row alias, leaving `process` as the current spelling.
- Added Phase 201 gate rows for the removed parser, typechecker, core, lexer, and `proc` alias
  carriers.
- Verification run for this slice:
  `cargo fmt --check`;
  `cargo check -p ash-core -p ash-parser -p ash-typeck -p ash-engine -p ash-cli --all-targets`;
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate`.

2026-07-11 AST/grammar removal slice:

- Removed expression-level `act { ... }` do-sugar parsing.
- Rejected explicit `do:Act`, `do:Proc`, and `do:Workflow` targets at parse time while preserving
  ambient `do { ... }`.
- Removed `DoStmt::WorkflowRequires` and `DoStmt::WorkflowEnsures` from the surface AST.
- Deleted positive parser coverage that preserved `do:Workflow` contract statements and
  `do:Act` / `do:Proc` parsing.
- Removed the engine parser-only public-summary adapter that synthesized workflow summaries from
  `do:Workflow`.
- Added Phase 201 gate rows for the removed act do-sugar parser, workflow-specific do-statement
  variants, explicit tower target construction, and engine `do:Workflow` summary adapter.
- Verification run for this slice:
  `cargo fmt --check`;
  `cargo check -p ash-parser -p ash-typeck -p ash-engine -p ash-repl --all-targets`;
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate`;
  `cargo test -p ash-parser removed_explicit_tower_do_targets_do_not_parse`;
  `cargo test -p ash-parser removed_act_do_sugar_does_not_parse_in_function_body`;
  `cargo test -p ash-typeck --test task_748_do_target_resolution removed_act_do_target_fails_before_typechecking`;
  `cargo test -p ash-typeck --test task_748_do_target_resolution removed_proc_do_target_fails_before_typechecking`;
  `python3 tools/docs/validate_orientation_indexes.py --self-test`;
 `bash scripts/check-docs-gate.sh`.

2026-07-11 parser/typechecker AST removal slice:

- Removed parser workflow grammar modules and exports: `parse_workflow`, `parse_receive`,
  `parse_send`, `parse_set`, and `parse_observe`.
- Removed proxy/yield/resume parser entry points from `parse_module`, and moved the role
  capability-clause parser to module parsing without depending on workflow grammar.
- Removed workflow-specific recovery entry points from `error_recovery`.
- Removed parser surface workflow/proxy/action/check/guard/receive/yield AST carriers:
  `Workflow`, `ProxyDef`, `CapabilityRef`, `ActionRef`, `ObligationRef`, `CheckTarget`, `Guard`,
  `ReceiveMode`, `ReceiveArm`, `StreamPattern`, and `YieldArm`.
- Removed parser workflow desugaring and deleted positive workflow AST/effect tests instead of
  preserving the removed behavior.
- Removed lexer token variants and keyword mappings for legacy workflow action/check words:
  `observe`, `orient`, `propose`, `decide`, `oblige`, and `check`.
- Removed typechecker workflow entry/effect/name-resolution/runtime-verification modules and
  positive workflow obligation/effect/runtime verification tests.
- Restored program typechecking around target `ProgramEntry` function metadata instead of
  workflow definitions.
- Added Phase 201 gate rows for parser surface workflow AST carriers, workflow desugaring,
  workflow body lowering, and typechecker workflow modules.
- Verification run for this slice:
  `cargo fmt --check`;
  `cargo test -p ash-parser --test par_removal_tests`;
  `cargo test -p ash-typeck task959_fn_return_pure_closure_is_accepted`;
  `cargo check -p ash-parser --all-targets`;
  `cargo check -p ash-typeck --all-targets`;
  `cargo check -p ash-parser -p ash-typeck -p ash-engine -p ash-repl -p ash-cli --all-targets`.

2026-07-11 core workflow-carrier removal slice:

- Removed the stale `ash_core::workflow_contract::Workflow` enum and deleted positive tests that
  preserved its `Oblige`, `CheckObligation`, and `Done` variants.
- Removed typechecker `EntryTypedArtifact` / `WorkflowForm` exposure and the `do:Workflow`
  artifact builder path.
- Removed public workflow-summary import plumbing from `TypeEnv`, engine `Workflow` metadata,
  and module-loader callable exports.
- Deleted `ash_core::workflow_carrier` and positive workflow-form projection/coverage tests.
- Removed TCIR `TcirEntryArtifactProvenance`, `TcirStatementKind::EntryArtifact`, and AMIR
  `EntryArtifact` opcode carriers.
- Added Phase 201 gate rows for the removed core workflow contract enum, first-class workflow
  carrier module, TCIR entry artifact carrier, and AMIR entry artifact carrier.
- Verification run for this slice:
  `cargo check -p ash-core --all-targets`;
  `cargo test -p ash-core --test par_removal_tests`;
  `cargo check -p ash-core -p ash-typeck -p ash-engine --all-targets`.

2026-07-11 parser core-lift removal slice:

- Removed `ash_parser::lift` and its public `lift_workflow` /
  `lift_workflow_with_names` exports.
- Deleted positive core workflow lift tests that exercised legacy `ash_core::Workflow` variants.
- Updated engine program parsing to keep lowered target function bodies in the narrow
  `ash_core::Workflow::Ret` bridge while the remaining runtime workflow API is removed separately.
- Added Phase 201 gate rows for the removed parser lift module and functions.
- Verification run for this slice:
  `cargo check -p ash-parser -p ash-engine --all-targets`.

2026-07-11 core workflow tooling removal slice:

- Removed `ash_core::small_step` and its `lower_workflow` helper.
- Removed `ash_interp::small_step` and its public small-step workflow runner facade.
- Removed `ash_core::visualize`, its `ToDot` trait, and DOT emission for legacy core workflows.
- Removed the CLI `dot` command module and command wiring that depended on workflow DOT
  visualization.
- Removed stale visualizer-specific positive test coverage.
- Added Phase 201 gate rows for the removed core/interpreter workflow tooling modules, visualizer
  trait, small-step lowerers/runners, and CLI DOT command wiring.
- Verification run for this slice:
  `cargo fmt --check`;
  `cargo check -p ash-core -p ash-interp -p ash-cli --all-targets`;
  `cargo check -p ash-core -p ash-parser -p ash-typeck -p ash-engine -p ash-repl -p ash-cli --all-targets`;
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate`;
  `python3 tools/docs/validate_orientation_indexes.py --self-test`;
  `bash scripts/check-docs-gate.sh`;
  active-source scan for removed small-step/visualizer/DOT command identifiers.

2026-07-11 engine expression-entry removal slice:

- Changed the engine entry handle to store a lowered target `Expr` body instead of an
  `ash_core::Workflow` body.
- Changed application admission requests to carry an `Expr` body and evaluate it directly through
  the runtime expression evaluator.
- Removed the engine's direct core-workflow executor and workflow body registration APIs.
- Replaced the engine workflow monomorphizer with the expression monomorphizer used by target
  entries.
- Deleted positive engine tests that preserved hand-constructed core workflow execution,
  workflow-call registration, workflow admission/completion, and workflow action dispatch.
- Added Phase 201 gate rows for the removed engine workflow body shell, direct executor,
  registration APIs, and workflow monomorphizer.
- Verification run for this slice:
  `cargo check -p ash-engine --all-targets`;
  `cargo check -p ash-core -p ash-parser -p ash-typeck -p ash-engine -p ash-repl -p ash-cli --all-targets`;
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate`;
  `cargo fmt --check`;
  `python3 tools/docs/validate_orientation_indexes.py --self-test`;
  `bash scripts/check-docs-gate.sh`;
  active-source scan for removed engine workflow entry shell, direct executor, registration APIs,
  and workflow monomorphizer identifiers.

2026-07-11 engine public entry-handle removal slice:

- Renamed the public parsed-entry handle from `ash_engine::Workflow` to `ash_engine::Entry`.
- Updated engine row-admission APIs, engine tests, CLI run/trace commands, and REPL session code to
  use the `Entry` handle.
- Added Phase 201 gate rows for the removed public engine workflow handle and CLI/REPL references.
- Verification run for this slice:
  `cargo check -p ash-engine --all-targets`;
  `cargo check -p ash-core -p ash-parser -p ash-typeck -p ash-engine -p ash-repl -p ash-cli --all-targets`;
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate`;
  `cargo fmt --check`;
 no-match scan for `ash_engine::Workflow`, `pub struct Workflow`, `Result<Workflow, EngineError>`,
  `&Workflow`, `Workflow as EngineEntry`, `parse_runnable_workflow`, and `Ok(Workflow {` in
  engine/CLI/REPL active paths.

2026-07-11 interpreter workflow executor removal slice:

- Removed the `ash-interp` workflow executor, stream executor, yield-state, and yield-routing
  modules from the crate.
- Removed `RuntimeState` workflow body caches and public registration/lookup helpers for spawned
  process bodies and function bodies.
- Deleted positive interpreter integration tests that preserved hand-constructed
  `ash_core::Workflow` execution, workflow yields/proxy resumes, direct workflow action dispatch,
  and workflow receive/obligation execution.
- Added Phase 201 gate rows for the removed interpreter workflow executor modules/APIs, yield
  carriers, and runtime-state workflow body caches.
- Verification run for this slice:
  `cargo fmt --check`;
  `cargo check -p ash-interp --all-targets`;
  `cargo check -p ash-core -p ash-parser -p ash-typeck -p ash-engine -p ash-interp -p ash-repl -p ash-cli --all-targets`;
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate`;
  `python3 tools/docs/validate_orientation_indexes.py --self-test`;
 `bash scripts/check-docs-gate.sh`;
  active-source scan for removed interpreter workflow executor modules/APIs, yield carriers, and
  runtime-state workflow body caches.

2026-07-11 core workflow AST removal slice:

- Removed `ash_core::Workflow` from the core AST.
- Removed core proxy workflow AST carriers, core workflow receive-arm/pattern carriers, and the
  workflow variant from top-level core definitions.
- Changed stream receive arms to carry expression bodies instead of workflow bodies.
- Deleted positive core tests/helpers that preserved workflow construction, proxy workflow AST
  structures, and workflow serialization.
- Added Phase 201 gate rows for the removed core AST/proxy/receive carriers.
- Verification run for this slice:
  `cargo fmt --check`;
  `cargo check -p ash-core --all-targets`;
  `cargo check -p ash-core -p ash-parser -p ash-typeck -p ash-engine -p ash-interp -p ash-repl -p ash-cli --all-targets`;
 `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate`;
  `python3 tools/docs/validate_orientation_indexes.py --self-test`;
  `bash scripts/check-docs-gate.sh`.

2026-07-11 public workflow computation carrier removal slice:

- Removed public `Workflow<T>` registration from `TypeEnv`.
- Removed `workflow::unit`, `workflow::bind`, `workflow::then`, `workflow::from_proc`, and
  `workflow::from_act` from TypeEnv builtin values, public computation manifests, builtin tables,
  and evaluator dispatch.
- Changed `contract::requires` / `contract::ensures` to return a neutral contract helper result
  instead of a fake `Workflow<Null>` carrier.
- Removed workflow effect-tower ranking and retargeted instance/control-link runtime values to the
  current process-level effect carrier.
- Moved `ash_core::workflow_contract` to `ash_core::contract` and
  `ash_parser::workflow_contract_classifier` to `ash_parser::contract_classifier` without
  compatibility shim modules.
- Retargeted tests that used `Workflow` only as a computation target fixture to `Proc`, and added
  manifest assertions that `Workflow` / `workflow::*` are absent.
- Added Phase 201 gate rows for the removed public workflow type, workflow intrinsic values,
  workflow runtime dispatch, workflow effect rank, and old contract module paths.
- Verification run for this slice:
  `cargo fmt --check`;
  `cargo check -p ash-typeck -p ash-interp -p ash-engine --all-targets`;
  `cargo check -p ash-core -p ash-parser -p ash-typeck -p ash-engine -p ash-interp -p ash-repl -p ash-cli --all-targets`;
  `cargo test -p ash-typeck --test alpha_visible_computation_manifest`;
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate`;
  `python3 tools/docs/validate_orientation_indexes.py --self-test`;
  `bash scripts/check-docs-gate.sh`.

2026-07-11 runtime identity/provenance vocabulary removal slice:

- Renamed the exported runtime/provenance identity from `WorkflowId` to `ApplicationId` and changed
  `Provenance.workflow_id` to `Provenance.application_id`.
- Replaced runtime resource/failure/trace identity variants with application vocabulary:
  `ResourceOwner::Application`, `FailureEntity::Application`, and `TraceFactKind::Application`.
- Retargeted interpreter semantic terminal projection from `SemanticWorkflowOutcome` /
  `project_workflow_outcome` to `SemanticApplicationOutcome` / `project_application_outcome`.
- Retargeted `ash-provenance` trace events, recorder accessors, export helpers, and macro call
  sites from workflow start/completion/query terminology to application terminology.
- Deleted the stale unregistered benchmark source that still constructed the removed
  `ash_core::Workflow` enum.
- Added Phase 201 gate rows for `WorkflowId`, `workflow_id`, `ResourceOwner::Workflow`,
  `TraceFactKind::Workflow`, `SemanticWorkflowOutcome`, and old provenance trace event/query names.
- Verification run for this slice:
  `cargo fmt`;
  `cargo check -p ash-core -p ash-parser -p ash-typeck -p ash-engine -p ash-interp -p ash-repl -p ash-cli --all-targets`;
  `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate`.
