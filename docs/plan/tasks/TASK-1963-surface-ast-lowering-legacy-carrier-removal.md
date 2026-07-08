# TASK-1963: Surface AST/Lowering Legacy Carrier Removal

**Status:** Complete
**Phase:** [PLAN-201: Deprecated Functionality Removal](../PLAN-201-DEPRECATED-FUNCTIONALITY-REMOVAL.md)

## Description

Remove or quarantine legacy surface AST and lowering carriers that are no longer reachable from
current target Ash after parser/checker removal.

## Requirements

- Use AUDIT-201 to identify legacy AST and lowering carriers.
- Remove carriers that are not needed by current target Ash.
- Rename or replace surviving target behavior with row/profile/provider/evidence vocabulary.
- Preserve source-to-Core and Core-to-CPS behavior for current target programs.

## TDD Steps

1. Add focused tests proving target Ash lowering no longer emits deprecated carriers.
2. Add compile-fail or diagnostic tests without retaining deprecated Ash source snippets.
3. Remove or quarantine legacy AST/lowering carriers.
4. Run parser, engine, Core, CPS, and current-template gates.

## Completion Checklist

- [x] Current target Ash lowers without deprecated surface carriers.
- [x] Deprecated syntax cannot reach Core/CPS through compatibility lowering.
- [x] Deprecated Ash fixtures are removed rather than relabeled.
- [x] Engine summaries use target vocabulary.
- [x] Focused lowering and current-template gates pass.

## Evidence

- Removed current module-definition export metadata for removed capability interface and
  capability implementation definitions. `ModuleDefinitionExportKind` now carries current resource
  type metadata only, and import bindings no longer classify removed capability definition forms as
  current importable module items.
- Focused compile verification after this slice:
  `cargo check -p ash-parser -p ash-lsp-core --all-targets`.
- Removed the unreachable parser surface AST structs for removed capability interface and
  capability implementation declarations, removed typechecker registration/conformance APIs that
  accepted those structs, deleted legacy typechecker tests that constructed removed definitions
  directly, and removed the dead capability-implementation-body expression escape check.
- Focused compile verification after the carrier cleanup:
  `cargo check -p ash-parser -p ash-typeck -p ash-lsp-core --all-targets`.
- Removed the `ash-engine` ordinary type-snippet compatibility parser path
  (`parse_type_def_snippet`, simple alias snippet parsing, and parsed-snippet lowering helpers).
  LLM stdlib structural type tests now parse the target `ModuleFile` surface and lower type
  metadata through `lower_module_type_metadata`.
- Removed source/parser and typechecker compatibility for old-form act statements. Target
  `act { ... <- ...; return ... }` do-sugar parses as `DoBlock`.
- Removed the internal parser surface `ActBlock`/`ActStmt` carriers and their compatibility
  lowering path. Parser, typechecker, lint, engine, interp, and REPL code now use target Act
  do-sugar or core Act closures rather than constructing removed surface carriers.
- Focused engine verification after the type-snippet compatibility removal:
  `cargo test -p ash-engine --test llm_stdlib_e2e_tests -- --nocapture`;
  `cargo test -p ash-engine module_loader::tests::type_identity_collector_includes_builtin_type_forms -- --nocapture`;
  `cargo check -p ash-engine --all-targets`;
  `cargo clippy -p ash-engine --all-targets -- -D warnings`.
- Focused verification after removing the `ActBlock`/`ActStmt` carriers:
  `cargo check -p ash-parser -p ash-typeck -p ash-engine -p ash-interp -p ash-lint -p ash-repl --all-targets`;
  `cargo test -p ash-typeck --test task_750_target_act_do_sugar -- --nocapture`;
  `cargo test -p ash-engine --test task_719_proc_from_act_boundary -- --nocapture`;
  `cargo test -p ash-interp --test task_719_proc_from_act_runtime -- --nocapture`;
  `cargo clippy -p ash-parser -p ash-typeck -p ash-engine -p ash-interp -p ash-lint -p ash-repl --all-targets -- -D warnings`.
- Removed deleted workflow authority/resource variants from `WorkflowHeaderEvent`; the carrier now
  preserves only current `requires:`/`ensures:` contract order. Macro expansion, operator-section
  elaboration, and expression visitors no longer traverse stale header-event binding branches.
- Removed the dead `LoweredWorkflow` compatibility wrapper and `lower_workflow_def` helper after
  implicit-role synthesis was removed. Current callers use `lower_workflow` directly.
- Removed the `WorkflowDef` owned-resource and used-binding header carriers from the parser
  surface AST. Target workflows no longer have a direct Rust construction path for removed
  `owns`/`uses` workflow headers.
- Focused parser verification after the `WorkflowHeaderEvent` cleanup:
  `cargo check -p ash-parser --all-targets`.
- Retargeted parser contract-lowering vocabulary away from legacy Stage-1 wording. Lowered fn
  contract sidecars and deferred discharge reasons now use classified-contract terminology for
  current contract/runtime hooks.
- Focused parser verification after the contract-lowering vocabulary cleanup:
  `cargo test -p ash-parser lower::tests --lib -- --nocapture`;
  `cargo test -p ash-parser --test task_770_workflow_contract_surface -- --nocapture`.
- Retargeted parser capability-import metadata away from legacy capability classification.
  Imported provider/action targets now use current provider-operation binding vocabulary, and
  module definition export comments no longer describe current provider-operation targets as
  legacy direct capability metadata.
- Focused import-resolver verification after the provider-operation binding vocabulary cleanup:
  `cargo test -p ash-parser import_resolver::tests::test_capability_import_preserves_target_metadata --lib -- --nocapture`.
- Retargeted the parser `Decide` else-branch lowering rejection away from legacy wording. Removed
  else-branch carriers now fail canonical lowering with removed-form vocabulary, covered by a
  focused regression test that rejects reintroducing the legacy label.
- Focused parser verification after the decide-else diagnostic cleanup:
  `cargo test -p ash-parser decide_else_branch_lowering_error_uses_removed_form_vocabulary --lib -- --nocapture`;
  `cargo test -p ash-parser lower::tests --lib -- --nocapture`.
- Final closeout verification superseded the earlier remaining-scope note: admitted provider
  metadata, implementation-selection vocabulary, and Workflow/Proc carrier rows were either
  removed, retargeted, or classified for semantic follow-up in TASK-1969/TASK-1970.
