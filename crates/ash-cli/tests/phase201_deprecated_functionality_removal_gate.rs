//! TASK-1961/TASK-1967: Phase 201 deprecated functionality removal gate.

use std::path::{Path, PathBuf};

const SCAN_ROOTS: &[&str] = &[
    "README.md",
    "crates",
    "examples",
    "std",
    "templates",
    "tests",
    "docs/API.md",
    "docs/README.md",
    "docs/SHARO_CORE_LANGUAGE.md",
    "docs/TUTORIAL.md",
    "docs/book",
    "docs/tutorials",
    "reference/language/functions.md",
    "reference/language/functions",
    "reference/agents/cards/functions.md",
    "reference/status/removed-forms.md",
];

const EXCLUDED_PREFIXES: &[&str] = &[
    ".git/",
    "target/",
    "docs/plan/PLAN-INDEX-HISTORY.md",
    "docs/plan/audits/AUDIT-200-legacy-deprecated-form-inventory.md",
    "docs/plan/audits/AUDIT-201-deprecated-functionality-removal.md",
    "docs/plan/tasks/",
    "CHANGELOG.md",
];

const EXCLUDED_FILES: &[&str] = &[
    "docs/plan/PLAN-201-DEPRECATED-FUNCTIONALITY-REMOVAL.md",
    "crates/ash-cli/tests/phase201_deprecated_functionality_removal_gate.rs",
];

const REMOVED_TYPE_NAMES: &[&str] = &["Act", "Proc", "Workflow"];

const REMOVED_ACTIVE_REFERENCES: &[(&str, &str, &str)] = &[
    (
        "crates/ash-runtime/src/lib.rs",
        "pub mod eval;",
        "runtime-direct-ast-evaluation-export",
    ),
    (
        "crates/ash-runtime/src/lib.rs",
        "pub mod guard;",
        "runtime-direct-ast-guard-export",
    ),
    (
        "crates/ash-runtime/src/lib.rs",
        "pub mod policy;",
        "runtime-direct-ast-policy-export",
    ),
    ("std/src/ooda.ash", "", "ooda-stdlib-module"),
    ("std/src/lib.ash", "pub mod ooda", "ooda-stdlib-export"),
    ("std/src/lib.ash", "pub use ooda", "ooda-stdlib-export"),
    ("crates/ash-lint/", "ooda-missing", "ooda-lint-alias"),
    ("crates/ash-lint/", "OODA Compatibility", "ooda-lint-doc"),
    (
        "crates/ash-lint/",
        "LintCategory::Ooda",
        "ooda-lint-category",
    ),
    (
        "crates/ash-interp/src/eval.rs",
        "legacy eval_function_call",
        "legacy-builtin-fallback-label",
    ),
    (
        "crates/ash-cli/src/commands/check.rs",
        "real `workflow` keyword",
        "workflow-keyword-current-label",
    ),
    (
        "crates/ash-engine/tests/task_786_import_visibility_summary_rules/",
        "legacy TypeDef fallback",
        "legacy-typedef-fallback-label",
    ),
    (
        "crates/ash-engine/tests/task_786_import_visibility_summary_rules/",
        "legacy fallback representation",
        "legacy-typedef-fallback-label",
    ),
    (
        "reference/language/functions.md",
        "runtime-managed effect tower",
        "current-reference-effect-tower-guidance",
    ),
    (
        "reference/language/functions/boundaries.md",
        "live above pure code in the tower",
        "current-reference-effect-tower-guidance",
    ),
    (
        "reference/language/functions/boundaries.md",
        "explicit tower API",
        "current-reference-explicit-tower-api-guidance",
    ),
    (
        "reference/language/functions/local-and-anonymous.md",
        "higher tower contexts",
        "current-reference-higher-tower-context-guidance",
    ),
    (
        "reference/language/functions/local-and-anonymous.md",
        "Act/Proc/Workflow closures",
        "current-reference-tower-closure-guidance",
    ),
    (
        "reference/agents/cards/functions.md",
        "implicitly lift into Act/Proc/Workflow",
        "current-agent-card-tower-guidance",
    ),
    (
        "reference/agents/cards/functions.md",
        "reserved tower callable arrows",
        "current-agent-card-tower-guidance",
    ),
    (
        "crates/ash-typeck/tests/task_803_phase110_non_interference.rs",
        "behavior_still",
        "typeck-test-stale-workflow-compatibility-label",
    ),
    (
        "crates/ash-typeck/tests/task_909_act_proc_workflow_bridge_non_interference.rs",
        "do_workflow_still",
        "typeck-test-stale-workflow-compatibility-label",
    ),
    (
        "crates/ash-typeck/tests/task_959_pure_closure_arrow.rs",
        "workflow-context",
        "typeck-test-stale-workflow-context-label",
    ),
    (
        "crates/ash-typeck/tests/task_959_pure_closure_arrow.rs",
        "workflow contexts",
        "typeck-test-stale-workflow-context-label",
    ),
    (
        "crates/ash-typeck/src/do_target.rs",
        "legacy fallback dictionaries",
        "legacy-do-target-fallback-label",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "legacy `pub fn` snippet diagnostics",
        "legacy-pub-fn-snippet-label",
    ),
    (
        "crates/ash-cli/tests/cli_input_entry_source_test.rs",
        "entry workflow",
        "entry-workflow-stale-label",
    ),
    (
        "crates/ash-cli/tests/cli_input_entry_source_test.rs",
        "test_workflow",
        "entry-test-workflow-name",
    ),
    (
        "crates/ash-cli/tests/cli_input_entry_source_test.rs",
        "let workflow",
        "entry-test-workflow-variable",
    ),
    (
        "crates/ash-cli/tests/cli_input_entry_source_test.rs",
        "workflow_path",
        "entry-test-workflow-variable",
    ),
    (
        "crates/ash-cli/tests/input_functional_test.rs",
        "entry workflow",
        "entry-workflow-stale-label",
    ),
    (
        "crates/ash-cli/tests/input_functional_test.rs",
        "test_run_simple_workflow",
        "entry-test-workflow-name",
    ),
    (
        "crates/ash-cli/tests/input_functional_test.rs",
        "let workflow",
        "entry-test-workflow-variable",
    ),
    (
        "crates/ash-cli/tests/input_functional_test.rs",
        "workflow_path",
        "entry-test-workflow-variable",
    ),
    (
        "crates/ash-cli/tests/input_functional_test.rs",
        "Run the workflow",
        "entry-test-workflow-comment",
    ),
    (
        "crates/ash-cli/tests/input_functional_test.rs",
        "Workflow should execute",
        "entry-test-workflow-assertion",
    ),
    (
        "crates/ash-engine/tests/runtime_boundary_visibility.rs",
        "entry workflow",
        "entry-workflow-stale-label",
    ),
    (
        "crates/ash-parser/src/surface.rs",
        "main entry workflow",
        "entry-workflow-stale-label",
    ),
    (
        "crates/ash-parser/src/surface.rs",
        "WorkflowHeaderEvent",
        "parser-workflow-contract-header-carrier",
    ),
    (
        "crates/ash-parser/src/surface.rs",
        "header_events",
        "parser-workflow-contract-header-carrier",
    ),
    (
        "crates/ash-parser/src/parse_workflow.rs",
        "WorkflowHeaderEvent",
        "parser-workflow-contract-header-carrier",
    ),
    (
        "crates/ash-parser/src/parse_workflow.rs",
        "parse_workflow_header_events",
        "parser-workflow-contract-header-carrier",
    ),
    (
        "crates/ash-parser/src/lower.rs",
        "WorkflowDef",
        "parser-workflowdef-lowering-carrier",
    ),
    (
        "crates/ash-parser/src/surface.rs",
        "helper_workflows",
        "parser-program-workflowdef-entry-carrier",
    ),
    (
        "crates/ash-parser/src/surface.rs",
        "workflow: WorkflowDef",
        "parser-program-workflowdef-entry-carrier",
    ),
    (
        "crates/ash-parser/src/surface.rs",
        "workflow: Option<WorkflowDef>",
        "parser-modulefile-workflowdef-carrier",
    ),
    (
        "crates/ash-parser/src/parse_module.rs",
        "workflow_def",
        "parser-modulefile-workflowdef-carrier",
    ),
    (
        "crates/ash-parser/src/parse_workflow.rs",
        "pub fn workflow_def",
        "parser-workflowdef-definition-parser",
    ),
    (
        "crates/ash-parser/src/lib.rs",
        "pub mod parse_workflow",
        "parser-workflow-grammar-module",
    ),
    (
        "crates/ash-parser/src/lib.rs",
        "pub use parse_workflow",
        "parser-workflow-grammar-module",
    ),
    (
        "crates/ash-parser/src/lib.rs",
        "pub mod parse_receive",
        "parser-workflow-grammar-module",
    ),
    (
        "crates/ash-parser/src/lib.rs",
        "pub mod parse_observe",
        "parser-workflow-grammar-module",
    ),
    (
        "crates/ash-parser/src/parse_module.rs",
        "proxy_def",
        "parser-proxy-workflow-body-entry",
    ),
    (
        "crates/ash-parser/src/parse_module.rs",
        "parse_proxy_definition",
        "parser-proxy-workflow-body-entry",
    ),
    (
        "crates/ash-parser/src/parse_module.rs",
        "parse_yield",
        "parser-workflow-yield-entry",
    ),
    (
        "crates/ash-parser/src/parse_module.rs",
        "parse_resume",
        "parser-workflow-resume-entry",
    ),
    (
        "crates/ash-parser/src/error_recovery.rs",
        "parse_workflow::workflow",
        "parser-workflow-recovery-entry",
    ),
    (
        "crates/ash-parser/src/error_recovery.rs",
        "parse_workflow::workflow_def",
        "parser-workflowdef-definition-parser",
    ),
    (
        "crates/ash-parser/src/surface.rs",
        "pub struct WorkflowDef",
        "parser-surface-workflowdef-carrier",
    ),
    (
        "crates/ash-parser/src/surface.rs",
        "pub enum Workflow",
        "parser-surface-workflow-ast-carrier",
    ),
    (
        "crates/ash-parser/src/surface.rs",
        "pub struct ActionRef",
        "parser-surface-workflow-action-carrier",
    ),
    (
        "crates/ash-parser/src/surface.rs",
        "pub enum CheckTarget",
        "parser-surface-workflow-check-carrier",
    ),
    (
        "crates/ash-parser/src/surface.rs",
        "pub enum Guard",
        "parser-surface-workflow-guard-carrier",
    ),
    (
        "crates/ash-parser/src/surface.rs",
        "pub struct ProxyDef",
        "parser-surface-proxy-workflow-carrier",
    ),
    (
        "crates/ash-parser/src/desugar.rs",
        "desugar_workflow",
        "parser-workflow-desugar-module",
    ),
    (
        "crates/ash-parser/src/lower.rs",
        "lower_workflow_body",
        "parser-workflow-body-lowering-carrier",
    ),
    (
        "crates/ash-parser/src/lib.rs",
        "pub mod lift",
        "parser-core-workflow-lift-module-export",
    ),
    (
        "crates/ash-parser/src/lift.rs",
        "lift_workflow",
        "parser-core-workflow-lift-pass",
    ),
    (
        "crates/ash-parser/src/lift.rs",
        "lift_workflow_with_names",
        "parser-core-workflow-lift-pass",
    ),
    (
        "crates/ash-core/src/lib.rs",
        "pub mod small_step",
        "core-workflow-small-step-module-export",
    ),
    (
        "crates/ash-core/src/lib.rs",
        "pub mod visualize",
        "core-workflow-visualizer-module-export",
    ),
    (
        "crates/ash-core/src/lib.rs",
        "pub use visualize",
        "core-workflow-visualizer-reexport",
    ),
    (
        "crates/ash-core/src/small_step.rs",
        "pub fn lower_workflow",
        "core-workflow-small-step-lowerer",
    ),
    (
        "crates/ash-interp/src/lib.rs",
        "pub mod small_step",
        "interp-workflow-small-step-module-export",
    ),
    (
        "crates/ash-interp/src/small_step.rs",
        "pub enum StepOutcome",
        "interp-workflow-small-step-runner",
    ),
    (
        "crates/ash-core/src/visualize.rs",
        "pub trait ToDot",
        "core-workflow-dot-visualizer",
    ),
    (
        "crates/ash-cli/src/main.rs",
        "DotArgs",
        "cli-workflow-dot-command",
    ),
    (
        "crates/ash-cli/src/commands/mod.rs",
        "pub mod dot",
        "cli-workflow-dot-command-module",
    ),
    (
        "crates/ash-cli/src/commands/dot.rs",
        "pub fn dot",
        "cli-workflow-dot-command-entry",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "pub struct Workflow",
        "engine-public-workflow-entry-handle",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "pub core: ash_core::Workflow",
        "engine-entry-workflow-core-carrier",
    ),
    (
        "crates/ash-cli/src/commands/run.rs",
        "ash_engine::Workflow",
        "cli-engine-workflow-entry-handle",
    ),
    (
        "crates/ash-cli/src/commands/trace.rs",
        "ash_engine::Workflow",
        "cli-engine-workflow-entry-handle",
    ),
    (
        "crates/ash-repl/src/session.rs",
        "Workflow as EngineEntry",
        "repl-engine-workflow-entry-handle",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "ash_core::Workflow::Ret",
        "engine-entry-workflow-ret-shell",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "execute_core_workflow",
        "engine-direct-core-workflow-executor",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "register_function_body_with_params",
        "engine-workflow-function-body-registry-api",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "register_spawned_process_body",
        "engine-workflow-spawned-body-registry-api",
    ),
    (
        "crates/ash-engine/src/monomorphize.rs",
        "monomorphize_workflow",
        "engine-workflow-monomorphizer",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "workflow: ash_core::Workflow",
        "engine-workflow-admission-workflow-body",
    ),
    (
        "crates/ash-interp/src/lib.rs",
        "pub mod execute;",
        "interp-workflow-executor-module-export",
    ),
    (
        "crates/ash-interp/src/lib.rs",
        "pub mod execute_stream;",
        "interp-workflow-stream-executor-module-export",
    ),
    (
        "crates/ash-interp/src/lib.rs",
        "pub mod yield_routing;",
        "interp-workflow-yield-routing-module-export",
    ),
    (
        "crates/ash-interp/src/lib.rs",
        "pub mod yield_state;",
        "interp-workflow-yield-state-module-export",
    ),
    (
        "crates/ash-interp/src/lib.rs",
        "execute_workflow_with_behaviour",
        "interp-workflow-executor-api-export",
    ),
    (
        "crates/ash-interp/src/lib.rs",
        "execute_with_bindings_in_state",
        "interp-workflow-executor-api-export",
    ),
    (
        "crates/ash-interp/src/lib.rs",
        "interpret_in_state",
        "interp-workflow-executor-api-export",
    ),
    (
        "crates/ash-interp/src/execute.rs",
        "pub fn execute_workflow_with_behaviour",
        "interp-workflow-executor-api",
    ),
    (
        "crates/ash-interp/src/execute.rs",
        "pub async fn execute_simple",
        "interp-workflow-executor-api",
    ),
    (
        "crates/ash-interp/src/execute_stream.rs",
        "execute_core_receive",
        "interp-workflow-stream-executor-api",
    ),
    (
        "crates/ash-interp/src/yield_state.rs",
        "pub struct YieldState",
        "interp-workflow-yield-state-carrier",
    ),
    (
        "crates/ash-interp/src/yield_routing.rs",
        "pub struct YieldRouter",
        "interp-workflow-yield-router-carrier",
    ),
    (
        "crates/ash-interp/src/runtime_state.rs",
        "RegisteredFunctionBody",
        "interp-workflow-function-body-cache",
    ),
    (
        "crates/ash-interp/src/runtime_state.rs",
        "register_spawned_process_body",
        "interp-workflow-spawned-body-cache",
    ),
    (
        "crates/ash-interp/src/runtime_state.rs",
        "register_function_body",
        "interp-workflow-function-body-cache",
    ),
    (
        "crates/ash-interp/src/runtime_state.rs",
        "spawned_process_bodies",
        "interp-workflow-spawned-body-cache",
    ),
    (
        "crates/ash-typeck/src/lib.rs",
        "type_check_workflow",
        "typeck-workflow-entry-carrier",
    ),
    (
        "crates/ash-typeck/src/lib.rs",
        "type_check_workflow_def_in_env",
        "typeck-workflowdef-entry-carrier",
    ),
    (
        "crates/ash-typeck/src/lib.rs",
        "type_check_workflow_def",
        "typeck-workflowdef-entry-carrier",
    ),
    (
        "crates/ash-typeck/src/effect.rs",
        "infer_effect",
        "typeck-workflow-effect-module",
    ),
    (
        "crates/ash-typeck/src/names.rs",
        "resolve_workflow",
        "typeck-workflow-name-resolution-module",
    ),
    (
        "crates/ash-typeck/src/runtime_verification.rs",
        "EffectChecker",
        "typeck-workflow-runtime-verification-module",
    ),
    (
        "crates/ash-core/src/ast.rs",
        "pub struct WorkflowDef",
        "core-workflowdef-carrier",
    ),
    (
        "crates/ash-core/src/ast.rs",
        "pub enum Workflow",
        "core-workflow-ast-carrier",
    ),
    (
        "crates/ash-core/src/ast.rs",
        "pub struct ReceiveArm",
        "core-workflow-receive-arm-carrier",
    ),
    (
        "crates/ash-core/src/ast.rs",
        "pub enum ReceivePattern",
        "core-workflow-receive-pattern-carrier",
    ),
    (
        "crates/ash-core/src/ast.rs",
        "pub struct ProxyDef",
        "core-proxy-workflow-carrier",
    ),
    (
        "crates/ash-core/src/ast.rs",
        "pub enum InputCapability",
        "core-proxy-workflow-carrier",
    ),
    (
        "crates/ash-core/src/ast.rs",
        "Proxy(ProxyDef)",
        "core-proxy-workflow-module-item",
    ),
    (
        "crates/ash-core/src/ast.rs",
        "Workflow(Box<Workflow>)",
        "core-workflow-definition-carrier",
    ),
    (
        "crates/ash-core/src/stream.rs",
        "body: Workflow",
        "core-stream-workflow-receive-body-carrier",
    ),
    (
        "crates/ash-core/src/workflow_contract.rs",
        "pub struct WorkflowDef",
        "core-workflow-contract-workflowdef-carrier",
    ),
    (
        "crates/ash-core/src/workflow_contract.rs",
        "pub enum Workflow",
        "core-workflow-contract-workflow-carrier",
    ),
    (
        "crates/ash-core/src/lib.rs",
        "pub mod workflow_contract",
        "core-workflow-contract-module",
    ),
    (
        "crates/ash-parser/src/lib.rs",
        "pub mod workflow_contract_classifier",
        "parser-workflow-contract-classifier-module",
    ),
    (
        "crates/ash-parser/src/",
        "workflow_contract_classifier",
        "parser-workflow-contract-classifier-module",
    ),
    (
        "crates/ash-core/src/",
        "workflow_contract::",
        "core-workflow-contract-module-path",
    ),
    (
        "crates/ash-parser/src/",
        "workflow_contract::",
        "core-workflow-contract-module-path",
    ),
    (
        "crates/ash-typeck/src/",
        "workflow_contract::",
        "core-workflow-contract-module-path",
    ),
    (
        "crates/ash-engine/tests/",
        "workflow_contracts",
        "engine-workflow-contract-test-name",
    ),
    (
        "crates/ash-core/src/lib.rs",
        "pub mod workflow_carrier",
        "core-first-class-workflow-carrier-module",
    ),
    (
        "crates/ash-core/src/workflow_carrier.rs",
        "pub enum WorkflowForm",
        "core-first-class-workflow-form-carrier",
    ),
    (
        "crates/ash-core/src/type_ir.rs",
        "TcirEntryArtifactProvenance",
        "core-tcir-workflow-entry-artifact-carrier",
    ),
    (
        "crates/ash-core/src/amir.rs",
        "EntryArtifact",
        "core-amir-workflow-entry-artifact-carrier",
    ),
    (
        "crates/ash-parser/src/token.rs",
        "Workflow,",
        "parser-workflow-keyword-token",
    ),
    (
        "crates/ash-parser/src/token.rs",
        "Act,",
        "parser-act-keyword-token",
    ),
    (
        "crates/ash-parser/src/lexer.rs",
        "\"workflow\" =>",
        "parser-workflow-keyword-token",
    ),
    (
        "crates/ash-parser/src/lexer.rs",
        "\"act\" =>",
        "parser-act-keyword-token",
    ),
    (
        "crates/ash-parser/src/parse_module.rs",
        "keyword(\"proc\")",
        "parser-proc-row-alias",
    ),
    (
        "crates/ash-parser/src/parse_expr.rs",
        "parse_target_act_do_sugar_expr",
        "parser-act-do-sugar-entry",
    ),
    (
        "crates/ash-parser/src/surface.rs",
        "WorkflowRequires",
        "parser-do-workflow-contract-statement",
    ),
    (
        "crates/ash-parser/src/surface.rs",
        "WorkflowEnsures",
        "parser-do-workflow-contract-statement",
    ),
    (
        "crates/ash-parser/src/parse_expr.rs",
        "name: \"Act\".into()",
        "parser-explicit-tower-do-target",
    ),
    (
        "crates/ash-engine/src/module_loader/callable_exports.rs",
        "first-class do:Workflow public summary adapter",
        "engine-do-workflow-public-summary-adapter",
    ),
    (
        "crates/ash-cli/src/test_runner/synthesized.rs",
        "module.workflow",
        "synthesized-contract-workflow-fallback",
    ),
    (
        "crates/ash-engine/src/module_loader.rs",
        "synthesize_fn_main_entry_workflow",
        "engine-synthesized-workflowdef-entry-carrier",
    ),
    (
        "crates/ash-engine/src/module_loader.rs",
        "existing runtime entry carrier",
        "engine-synthesized-workflowdef-entry-carrier",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "surface_workflow_defs",
        "engine-surface-workflowdef-entry-storage",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "store_surface_workflow_def",
        "engine-surface-workflowdef-entry-storage",
    ),
    (
        "crates/ash-interp/src/role_runtime.rs",
        "WorkflowDef",
        "role-runtime-workflowdef-carrier",
    ),
    (
        "crates/ash-interp/tests/role_runtime_tests.rs",
        "WorkflowDef",
        "role-runtime-workflowdef-carrier",
    ),
    (
        "crates/ash-engine/tests/role_runtime_integration_tests.rs",
        "WorkflowDef",
        "role-runtime-workflowdef-carrier",
    ),
    (
        "crates/ash-core/src/runtime_kernel.rs",
        "WorkflowDefinitionId",
        "runtime-kernel-workflow-identity-carrier",
    ),
    (
        "crates/ash-core/src/runtime_kernel.rs",
        "WorkflowDefinitionIdentity",
        "runtime-kernel-workflow-identity-carrier",
    ),
    (
        "crates/ash-core/src/runtime_kernel.rs",
        "WorkflowArtifactId",
        "runtime-kernel-workflow-identity-carrier",
    ),
    (
        "crates/ash-core/src/runtime_kernel.rs",
        "WorkflowArtifactIdentity",
        "runtime-kernel-workflow-identity-carrier",
    ),
    (
        "crates/ash-core/src/runtime_kernel.rs",
        "WorkflowInstanceId",
        "runtime-kernel-workflow-identity-carrier",
    ),
    (
        "crates/ash-core/src/runtime_kernel.rs",
        "WorkflowInstanceIdentity",
        "runtime-kernel-workflow-identity-carrier",
    ),
    (
        "crates/ash-core/tests/alpha_runtime_kernel_carriers.rs",
        "WorkflowDefinitionIdentity",
        "runtime-kernel-workflow-identity-carrier",
    ),
    (
        "crates/ash-core/tests/alpha_runtime_kernel_carriers.rs",
        "WorkflowArtifactIdentity",
        "runtime-kernel-workflow-identity-carrier",
    ),
    (
        "crates/ash-core/tests/alpha_runtime_kernel_carriers.rs",
        "WorkflowInstanceIdentity",
        "runtime-kernel-workflow-identity-carrier",
    ),
    (
        "crates/ash-cli/src/commands/run.rs",
        "WorkflowDefinitionIdentity",
        "runtime-kernel-workflow-identity-carrier",
    ),
    (
        "crates/ash-cli/src/commands/run.rs",
        "WorkflowArtifactIdentity",
        "runtime-kernel-workflow-identity-carrier",
    ),
    (
        "crates/ash-cli/src/commands/run.rs",
        "WorkflowInstanceIdentity",
        "runtime-kernel-workflow-identity-carrier",
    ),
    (
        "crates/ash-cli/src/commands/daemon.rs",
        "WorkflowDefinitionIdentity",
        "runtime-kernel-workflow-identity-carrier",
    ),
    (
        "crates/ash-cli/src/commands/daemon.rs",
        "WorkflowArtifactIdentity",
        "runtime-kernel-workflow-identity-carrier",
    ),
    (
        "crates/ash-cli/src/commands/daemon.rs",
        "WorkflowInstanceIdentity",
        "runtime-kernel-workflow-identity-carrier",
    ),
    (
        "crates/ash-core/src/runtime.rs",
        "WorkflowAdmissionContext",
        "runtime-workflow-boundary-carrier",
    ),
    (
        "crates/ash-core/src/runtime.rs",
        "WorkflowFailure",
        "runtime-workflow-boundary-carrier",
    ),
    (
        "crates/ash-core/src/runtime.rs",
        "WorkflowFailureKind",
        "runtime-workflow-boundary-carrier",
    ),
    (
        "crates/ash-core/src/runtime.rs",
        "WorkflowFailureEvidence",
        "runtime-workflow-boundary-carrier",
    ),
    (
        "crates/ash-core/src/runtime.rs",
        "WorkflowReport",
        "runtime-workflow-boundary-carrier",
    ),
    (
        "crates/ash-core/src/runtime.rs",
        "WorkflowReportStatus",
        "runtime-workflow-boundary-carrier",
    ),
    (
        "crates/ash-core/src/runtime.rs",
        "WorkflowContractCheckEvidence",
        "runtime-workflow-boundary-carrier",
    ),
    (
        "crates/ash-core/src/runtime.rs",
        "WorkflowEvidenceStatus",
        "runtime-workflow-boundary-carrier",
    ),
    (
        "crates/ash-core/src/runtime.rs",
        "WorkflowBoundaryOutcome",
        "runtime-workflow-boundary-carrier",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "WorkflowAdmissionContext",
        "runtime-workflow-boundary-carrier",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "WorkflowAdmissionRequest",
        "runtime-workflow-boundary-carrier",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "WorkflowAdmissionOutcome",
        "runtime-workflow-boundary-carrier",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "WorkflowContractRequirement",
        "runtime-workflow-boundary-carrier",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "WorkflowContractCheckEvidence",
        "runtime-workflow-boundary-carrier",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "WorkflowBoundaryOutcome",
        "runtime-workflow-boundary-carrier",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "AdmittedWorkflowBoundary",
        "runtime-workflow-boundary-carrier",
    ),
    (
        "crates/ash-engine/src/row_admission.rs",
        "WorkflowAdmissionContext",
        "runtime-workflow-boundary-carrier",
    ),
    (
        "crates/ash-engine/src/row_admission.rs",
        "WorkflowAdmissionRequest",
        "runtime-workflow-boundary-carrier",
    ),
    (
        "crates/ash-engine/src/row_admission.rs",
        "WorkflowAdmissionOutcome",
        "runtime-workflow-boundary-carrier",
    ),
    (
        "crates/ash-core/src/runtime.rs",
        "pub workflow_id:",
        "workflow-report-workflow-id-field",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "pub workflow_id:",
        "workflow-admission-workflow-id-field",
    ),
    (
        "crates/ash-engine/src/row_admission.rs",
        "request.workflow_id",
        "workflow-row-admission-workflow-id-field",
    ),
    (
        "crates/ash-engine/tests/task_715_workflow_admission_red.rs",
        "workflow_id:",
        "workflow-admission-workflow-id-test-field",
    ),
    (
        "crates/ash-engine/tests/task_716_workflow_completion_red.rs",
        "workflow_id:",
        "workflow-completion-workflow-id-test-field",
    ),
    (
        "crates/ash-interp/src/lib.rs",
        "WorkflowBoundaryOutcome",
        "runtime-workflow-boundary-carrier",
    ),
    (
        "crates/ash-core/src/provenance.rs",
        "WorkflowId",
        "runtime-workflow-identity-carrier",
    ),
    (
        "crates/ash-core/src/provenance.rs",
        "workflow_id",
        "runtime-workflow-identity-field",
    ),
    (
        "crates/ash-core/src/runtime.rs",
        "ResourceOwner::Workflow",
        "runtime-workflow-resource-owner",
    ),
    (
        "crates/ash-core/src/runtime.rs",
        "TraceFactKind::Workflow",
        "runtime-workflow-trace-fact",
    ),
    (
        "crates/ash-core/src/core_ash_contract.rs",
        "TraceFactKind::Workflow",
        "core-contract-workflow-trace-fact",
    ),
    (
        "crates/ash-interp/src/execution_record.rs",
        "SemanticWorkflowOutcome",
        "interp-semantic-workflow-outcome-carrier",
    ),
    (
        "crates/ash-provenance/src/trace.rs",
        "WorkflowStarted",
        "provenance-workflow-trace-event",
    ),
    (
        "crates/ash-provenance/src/trace.rs",
        "events_for_workflow",
        "provenance-workflow-trace-query",
    ),
    (
        "crates/ash-typeck/src/do_target.rs",
        "DoTowerLevel",
        "do-target-tower-carrier",
    ),
    (
        "crates/ash-typeck/src/",
        "workflow::requires",
        "workflow-scoped-contract-intrinsic",
    ),
    (
        "crates/ash-typeck/src/",
        "workflow::ensures",
        "workflow-scoped-contract-intrinsic",
    ),
    (
        "crates/ash-typeck/tests/",
        "workflow::requires",
        "workflow-scoped-contract-intrinsic-test",
    ),
    (
        "crates/ash-typeck/tests/",
        "workflow::ensures",
        "workflow-scoped-contract-intrinsic-test",
    ),
    (
        "crates/ash-typeck/src/do_target.rs",
        "HiddenActReturn",
        "do-target-hidden-act-carrier",
    ),
    (
        "crates/ash-typeck/src/do_target.rs",
        "HiddenActBind",
        "do-target-hidden-act-carrier",
    ),
    (
        "crates/ash-typeck/src/do_target.rs",
        "resolve_tower_monad_evidence_dictionary",
        "do-target-tower-carrier",
    ),
    (
        "crates/ash-typeck/src/do_target.rs",
        "Act, Proc, or Workflow",
        "do-target-tower-diagnostic",
    ),
    (
        "crates/ash-typeck/src/do_target.rs",
        "\"workflow\".to_string()",
        "do-target-workflow-intrinsic",
    ),
    (
        "crates/ash-typeck/src/do_target.rs",
        "\"proc\".to_string()",
        "do-target-proc-intrinsic",
    ),
    (
        "crates/ash-typeck/src/do_target.rs",
        "\"act\".to_string()",
        "do-target-act-intrinsic",
    ),
    (
        "crates/ash-core/tests/task_714_workflow_boundary_carriers.rs",
        "WorkflowAdmissionContext",
        "runtime-workflow-boundary-carrier",
    ),
    (
        "crates/ash-core/tests/task_714_workflow_boundary_carriers.rs",
        "WorkflowContractCheckEvidence",
        "runtime-workflow-boundary-carrier",
    ),
    (
        "crates/ash-core/tests/task_714_workflow_boundary_carriers.rs",
        "WorkflowEvidenceStatus",
        "runtime-workflow-boundary-carrier",
    ),
    (
        "crates/ash-core/tests/task_714_workflow_boundary_carriers.rs",
        "WorkflowBoundaryOutcome",
        "runtime-workflow-boundary-carrier",
    ),
    (
        "crates/ash-core/src/runtime.rs",
        "pub enum TowerLevel",
        "runtime-tower-level-carrier",
    ),
    (
        "crates/ash-core/src/runtime.rs",
        "pub tower: TowerLevel",
        "runtime-tower-level-carrier",
    ),
    (
        "crates/ash-core/src/runtime.rs",
        "TowerLevel::Proc",
        "runtime-proc-tower-level",
    ),
    (
        "crates/ash-core/src/runtime.rs",
        "TowerLevel::Workflow",
        "runtime-workflow-tower-level",
    ),
    (
        "crates/ash-core/src/runtime.rs",
        "FailureEntity::Workflow",
        "runtime-workflow-failure-entity",
    ),
    (
        "crates/ash-core/src/type_ir.rs",
        "tower_level",
        "tcir-tower-level-carrier",
    ),
    (
        "crates/ash-core/src/type_ir.rs",
        "from_tower",
        "tcir-tower-level-carrier",
    ),
    (
        "crates/ash-core/src/type_ir.rs",
        "to_tower",
        "tcir-tower-level-carrier",
    ),
    (
        "crates/ash-core/src/amir.rs",
        "tower_level",
        "amir-tower-level-carrier",
    ),
    (
        "crates/ash-core/src/runtime_kernel.rs",
        "tower_level",
        "runtime-kernel-tower-level-carrier",
    ),
    (
        "crates/ash-cli/src/commands/daemon.rs",
        "workflow_succeeded",
        "daemon-workflow-status-label",
    ),
    (
        "crates/ash-cli/src/commands/daemon.rs",
        "workflow_request",
        "daemon-workflow-request-helper",
    ),
    (
        "crates/ash-cli/tests/alpha_run_daemon_artifact_equivalence.rs",
        "workflow_execution_failure",
        "daemon-workflow-failure-kind",
    ),
    (
        "crates/ash-cli/tests/alpha_run_daemon_artifact_equivalence.rs",
        "workflow-boundary execution failure",
        "daemon-workflow-boundary-label",
    ),
    (
        "crates/ash-engine/src/runtime_artifact.rs",
        "workflow_name",
        "runtime-artifact-workflow-name-carrier",
    ),
    (
        "crates/ash-engine/src/runtime_artifact.rs",
        "synthetic_tcir",
        "runtime-artifact-synthetic-entry-carrier",
    ),
    (
        "crates/ash-engine/src/runtime_artifact.rs",
        "RuntimeKernel<ApplicationEntry>",
        "runtime-artifact-synthetic-entry-carrier",
    ),
    (
        "crates/ash-cli/src/commands/run.rs",
        "workflow_name",
        "ash-run-workflow-name-carrier",
    ),
    (
        "crates/ash-cli/tests/alpha_ashd_local_daemon_control_plane.rs",
        "workflow_name",
        "daemon-test-workflow-name-helper",
    ),
    (
        "crates/ash-typeck/src/types.rs",
        "workflow_type",
        "type-instance-workflow-type-carrier",
    ),
    (
        "crates/ash-core/src/value.rs",
        "workflow_type",
        "runtime-instance-workflow-type-carrier",
    ),
    (
        "crates/ash-core/src/ast.rs",
        "workflow_type",
        "runtime-spawn-workflow-type-carrier",
    ),
    (
        "crates/ash-core/src/small_step.rs",
        "workflow_type",
        "runtime-spawn-workflow-type-carrier",
    ),
    (
        "crates/ash-core/src/visualize.rs",
        "workflow_type",
        "runtime-spawn-workflow-type-carrier",
    ),
    (
        "crates/ash-interp/src/",
        "workflow_type",
        "runtime-spawn-workflow-type-carrier",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "workflow_type",
        "runtime-spawn-workflow-type-carrier",
    ),
    (
        "crates/ash-interp/src/runtime_state.rs",
        "workflow_name",
        "runtime-callable-workflow-name-carrier",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "workflow_name",
        "engine-callable-workflow-name-carrier",
    ),
    (
        "crates/ash-interp/src/runtime_state.rs",
        "RegisteredCallableWorkflow",
        "runtime-callable-workflow-registry-carrier",
    ),
    (
        "crates/ash-interp/src/runtime_state.rs",
        "callable_workflows",
        "runtime-callable-workflow-registry-carrier",
    ),
    (
        "crates/ash-interp/src/runtime_state.rs",
        "register_callable_workflow",
        "runtime-callable-workflow-registry-carrier",
    ),
    (
        "crates/ash-interp/src/runtime_state.rs",
        "blocking_register_callable_workflow",
        "runtime-callable-workflow-registry-carrier",
    ),
    (
        "crates/ash-interp/src/runtime_state.rs",
        "callable_workflow",
        "runtime-callable-workflow-registry-carrier",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "register_callable_workflow",
        "engine-callable-workflow-registry-carrier",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "blocking_register_callable_workflow",
        "engine-callable-workflow-registry-carrier",
    ),
    (
        "crates/ash-interp/src/runtime_state.rs",
        "RegisteredCallableEntry",
        "runtime-callable-entry-registry-carrier",
    ),
    (
        "crates/ash-interp/src/runtime_state.rs",
        "callable_entries",
        "runtime-callable-entry-registry-carrier",
    ),
    (
        "crates/ash-interp/src/runtime_state.rs",
        "register_callable_entry",
        "runtime-callable-entry-registry-carrier",
    ),
    (
        "crates/ash-interp/src/runtime_state.rs",
        "blocking_register_callable_entry",
        "runtime-callable-entry-registry-carrier",
    ),
    (
        "crates/ash-interp/src/runtime_state.rs",
        "callable_entry",
        "runtime-callable-entry-registry-carrier",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "register_callable_entry",
        "engine-callable-entry-registry-carrier",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "register_callable_entry_with_params",
        "engine-callable-entry-registry-carrier",
    ),
    (
        "crates/ash-interp/src/runtime_state.rs",
        "child_workflows",
        "runtime-child-workflow-registry-carrier",
    ),
    (
        "crates/ash-interp/src/runtime_state.rs",
        "register_child_workflow",
        "runtime-child-workflow-registry-carrier",
    ),
    (
        "crates/ash-interp/src/runtime_state.rs",
        "child_workflow",
        "runtime-child-workflow-registry-carrier",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "register_child_workflow",
        "engine-child-workflow-registry-carrier",
    ),
    (
        "crates/ash-interp/src/runtime_state.rs",
        "child_entries",
        "runtime-child-entry-registry-carrier",
    ),
    (
        "crates/ash-interp/src/runtime_state.rs",
        "register_child_entry",
        "runtime-child-entry-registry-carrier",
    ),
    (
        "crates/ash-interp/src/runtime_state.rs",
        "child_entry",
        "runtime-child-entry-registry-carrier",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "register_child_entry",
        "engine-child-entry-registry-carrier",
    ),
    (
        "crates/ash-interp/src/lib.rs",
        "workflow_projection",
        "runtime-workflow-projection-wrapper",
    ),
    (
        "crates/ash-interp/src/workflow_projection.rs",
        "execute_workflow_proc_projection",
        "runtime-workflow-projection-wrapper",
    ),
    (
        "crates/ash-interp/src/workflow_projection.rs",
        "unsupported_workflow_proc_projection_message",
        "runtime-workflow-projection-wrapper",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "execute_workflow_proc_projection",
        "engine-workflow-projection-wrapper",
    ),
    (
        "crates/ash-interp/tests/",
        "execute_workflow_proc_projection",
        "runtime-workflow-projection-wrapper-test",
    ),
    (
        "crates/ash-interp/tests/",
        "unsupported_workflow_proc_projection_message",
        "runtime-workflow-projection-wrapper-test",
    ),
    (
        "crates/ash-engine/tests/",
        "execute_workflow_proc_projection",
        "engine-workflow-projection-wrapper-test",
    ),
    (
        "crates/ash-interp/tests/",
        "FirstClassWorkflowProjectionExecutionUnsupported",
        "runtime-workflow-projection-boundary-label",
    ),
    (
        "crates/ash-engine/tests/",
        "FirstClassWorkflowProjectionExecutionUnsupported",
        "engine-workflow-projection-boundary-label",
    ),
    (
        "crates/ash-interp/src/lib.rs",
        "entry_projection",
        "runtime-entry-proc-projection-wrapper",
    ),
    (
        "crates/ash-interp/src/entry_projection.rs",
        "execute_entry_proc_projection",
        "runtime-entry-proc-projection-wrapper",
    ),
    (
        "crates/ash-interp/src/entry_projection.rs",
        "unsupported_entry_proc_projection_message",
        "runtime-entry-proc-projection-wrapper",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "execute_entry_proc_projection",
        "engine-entry-proc-projection-wrapper",
    ),
    (
        "crates/ash-interp/tests/",
        "execute_entry_proc_projection",
        "runtime-entry-proc-projection-wrapper-test",
    ),
    (
        "crates/ash-engine/tests/",
        "execute_entry_proc_projection",
        "engine-entry-proc-projection-wrapper-test",
    ),
    (
        "crates/ash-interp/tests/",
        "FirstClassEntryProjectionExecutionUnsupported",
        "runtime-entry-proc-projection-boundary-label",
    ),
    (
        "crates/ash-engine/tests/",
        "FirstClassEntryProjectionExecutionUnsupported",
        "engine-entry-proc-projection-boundary-label",
    ),
    (
        "crates/ash-core/src/type_ir.rs",
        "TcirWorkflowArtifactProvenance",
        "tcir-workflow-artifact-carrier",
    ),
    (
        "crates/ash-core/src/type_ir.rs",
        "workflow_artifact",
        "tcir-workflow-artifact-carrier",
    ),
    (
        "crates/ash-core/src/type_ir.rs",
        "WorkflowArtifact",
        "tcir-workflow-artifact-carrier",
    ),
    (
        "crates/ash-core/src/amir.rs",
        "WorkflowArtifact",
        "amir-workflow-artifact-carrier",
    ),
    (
        "crates/ash-typeck/src/check_expr/",
        "workflow_artifact",
        "typeck-workflow-artifact-carrier",
    ),
    (
        "crates/ash-typeck/src/check_expr/",
        "WorkflowArtifactBuilder",
        "typeck-workflow-artifact-carrier",
    ),
    (
        "crates/ash-core/tests/alpha_tcir_computation_expression.rs",
        "TcirWorkflowArtifactProvenance",
        "tcir-workflow-artifact-carrier-test",
    ),
    (
        "crates/ash-core/tests/alpha_tcir_computation_expression.rs",
        "WorkflowArtifact",
        "tcir-workflow-artifact-carrier-test",
    ),
    (
        "crates/ash-core/tests/",
        "workflow_artifact",
        "tcir-workflow-artifact-carrier-test",
    ),
    (
        "crates/ash-typeck/tests/",
        "workflow_artifact",
        "typeck-workflow-artifact-carrier-test",
    ),
    (
        "crates/ash-engine/src/module_loader.rs",
        "workflow_source",
        "engine-module-loader-workflow-source-carrier",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "workflow_source",
        "engine-module-loader-workflow-source-carrier",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "parse_workflow_source_with_imports",
        "engine-workflow-source-parser-helper",
    ),
    (
        "crates/ash-typeck/src/type_env/",
        "workflow_effect",
        "type-env-workflow-effect-carrier",
    ),
    (
        "crates/ash-typeck/src/check_expr/mod.rs",
        "workflow_effect",
        "type-env-workflow-effect-carrier",
    ),
    (
        "crates/ash-typeck/src/lib.rs",
        "set_workflow_effect",
        "type-env-workflow-effect-carrier",
    ),
    (
        "crates/ash-typeck/src/type_env/",
        "add_workflow_type",
        "type-env-workflow-type-carrier",
    ),
    (
        "crates/ash-typeck/src/type_env/",
        "add_workflow_builtin_values",
        "type-env-workflow-intrinsic-carrier",
    ),
    (
        "crates/ash-typeck/src/type_env/",
        "name: \"Workflow\"",
        "type-env-workflow-type-carrier",
    ),
    (
        "crates/ash-typeck/src/type_env/",
        "workflow::unit",
        "type-env-workflow-intrinsic-carrier",
    ),
    (
        "crates/ash-typeck/src/type_env/",
        "workflow::bind",
        "type-env-workflow-intrinsic-carrier",
    ),
    (
        "crates/ash-typeck/src/type_env/",
        "workflow::from_proc",
        "type-env-workflow-intrinsic-carrier",
    ),
    (
        "crates/ash-typeck/src/type_env/",
        "workflow::from_act",
        "type-env-workflow-intrinsic-carrier",
    ),
    (
        "crates/ash-typeck/src/type_env/",
        "WorkflowContract",
        "type-env-workflow-contract-role",
    ),
    (
        "crates/ash-interp/src/eval",
        "workflow::unit",
        "interp-workflow-intrinsic-carrier",
    ),
    (
        "crates/ash-interp/src/eval",
        "(Some(\"workflow\"),",
        "interp-workflow-intrinsic-dispatch",
    ),
    (
        "crates/ash-core/src/value.rs",
        "\"Workflow\" =>",
        "core-workflow-effect-level-rank",
    ),
    (
        "crates/ash-typeck/src/runtime_verification.rs",
        "workflow_effect",
        "runtime-verification-workflow-effect-carrier",
    ),
    (
        "crates/ash-typeck/src/obligation_checker.rs",
        "workflow_effect",
        "obligation-checker-workflow-effect-carrier",
    ),
    (
        "crates/ash-typeck/src/check_expr/mod.rs",
        "Workflow effect context",
        "typeck-workflow-effect-context-wording",
    ),
    (
        "crates/ash-typeck/src/check_expr/mod.rs",
        "module.as_ref() == \"workflow\"",
        "typeck-workflow-module-elaboration-branch",
    ),
    (
        "crates/ash-typeck/src/check_expr/mod.rs",
        "unsupported workflow requires contract expression",
        "typeck-workflow-contract-requires-diagnostic",
    ),
    (
        "crates/ash-typeck/src/check_expr/mod.rs",
        "unsupported workflow ensures contract expression",
        "typeck-workflow-contract-ensures-diagnostic",
    ),
    (
        "crates/ash-typeck/src/check_expr/mod.rs",
        "workflow contract statement",
        "typeck-workflow-contract-statement-diagnostic",
    ),
    (
        "crates/ash-typeck/tests/",
        "workflow contract statement",
        "typeck-workflow-contract-statement-test-wording",
    ),
    (
        "crates/ash-typeck/tests/",
        "workflow_effect",
        "type-env-workflow-effect-carrier-test",
    ),
    (
        "crates/ash-typeck/src/type_env/",
        "WorkflowIntrinsic",
        "type-env-workflow-intrinsic-carrier",
    ),
    (
        "crates/ash-typeck/src/type_env/",
        "workflow_intrinsics",
        "type-env-workflow-intrinsic-carrier",
    ),
    (
        "crates/ash-typeck/src/type_env/",
        "lookup_workflow_intrinsic",
        "type-env-workflow-intrinsic-carrier",
    ),
    (
        "crates/ash-typeck/src/check_expr/mod.rs",
        "WorkflowIntrinsic",
        "typecheck-workflow-intrinsic-carrier",
    ),
    (
        "crates/ash-typeck/src/check_expr/mod.rs",
        "workflow_intrinsic",
        "typecheck-workflow-intrinsic-carrier",
    ),
    (
        "crates/ash-typeck/src/check_expr/mod.rs",
        "__workflow_intrinsic_context",
        "typecheck-workflow-intrinsic-context",
    ),
    (
        "crates/ash-typeck/src/lib.rs",
        "WorkflowIntrinsic",
        "type-env-workflow-intrinsic-carrier",
    ),
    (
        "crates/ash-typeck/tests/",
        "lookup_workflow_intrinsic",
        "type-env-workflow-intrinsic-carrier-test",
    ),
    (
        "crates/ash-typeck/src/lib.rs",
        "capability_check",
        "typeck-capability-check-workflow-surface-carrier",
    ),
    (
        "crates/ash-typeck/src/capability_check.rs",
        "",
        "typeck-capability-check-workflow-surface-carrier",
    ),
    (
        "crates/ash-typeck/tests/policy_contracts.rs",
        "",
        "typeck-capability-check-workflow-surface-test",
    ),
    (
        "crates/ash-typeck/tests/receive_contracts.rs",
        "",
        "typeck-capability-check-workflow-surface-test",
    ),
    (
        "crates/ash-typeck/tests/",
        "CapabilityChecker",
        "typeck-capability-check-workflow-surface-test",
    ),
    (
        "crates/ash-interp/tests/",
        "CapabilityChecker",
        "typeck-capability-check-workflow-surface-test",
    ),
    (
        "crates/ash-interp/tests/",
        "parse_workflow::workflow_def",
        "interp-workflow-parser-test-path",
    ),
    (
        "crates/ash-interp/tests/",
        "lower_workflow",
        "interp-workflow-lowerer-test-path",
    ),
    (
        "crates/ash-interp/tests/",
        "SurfaceWorkflow",
        "interp-workflow-surface-test-carrier",
    ),
    (
        "crates/ash-interp/tests/receive_execution.rs",
        "",
        "interp-workflow-parser-test-path",
    ),
    (
        "crates/ash-interp/tests/pipe_operator_e2e.rs",
        "",
        "interp-workflow-parser-test-path",
    ),
    (
        "crates/ash-interp/tests/task_1008_runtime_defensive_pattern_errors.rs",
        "",
        "interp-workflow-surface-test-carrier",
    ),
    (
        "crates/ash-typeck/tests/par_removal_tests.rs",
        "SurfaceWorkflow::Par",
        "typeck-stale-surface-workflow-par-label",
    ),
    (
        "crates/ash-typeck/tests/par_removal_tests.rs",
        "capability_checking",
        "typeck-stale-capability-checker-label",
    ),
    (
        "crates/ash-typeck/tests/par_removal_tests.rs",
        "Capability checking",
        "typeck-stale-capability-checker-label",
    ),
    (
        "crates/ash-typeck/src/obligation_checker.rs",
        "WorkflowCapabilities",
        "typeck-workflow-capabilities-carrier",
    ),
    (
        "crates/ash-typeck/src/runtime_verification.rs",
        "WorkflowCapabilities",
        "runtime-verification-workflow-capabilities-carrier",
    ),
    (
        "crates/ash-typeck/src/runtime_verification.rs",
        "workflow_capabilities",
        "runtime-verification-workflow-capabilities-carrier",
    ),
    (
        "crates/ash-typeck/tests/",
        "WorkflowCapabilities",
        "typeck-workflow-capabilities-test-carrier",
    ),
    (
        "crates/ash-typeck/tests/",
        "workflow_capabilities",
        "typeck-workflow-capabilities-test-carrier",
    ),
    (
        "crates/ash-interp/tests/",
        "WorkflowCapabilities",
        "runtime-verification-workflow-capabilities-test-carrier",
    ),
    (
        "crates/ash-interp/tests/proxy_execution_tests.rs",
        "suspended workflow",
        "interp-stale-suspended-workflow-label",
    ),
    (
        "crates/ash-interp/src/lib.rs",
        "proxy_registry",
        "interp-proxy-registry-module",
    ),
    (
        "crates/ash-interp/src/runtime_state.rs",
        "proxy_registry",
        "runtime-proxy-registry-state",
    ),
    (
        "crates/ash-interp/src/error.rs",
        "YieldSuspended",
        "interp-yield-suspended-error",
    ),
    (
        "crates/ash-interp/src/execution_record.rs",
        "YieldSuspended",
        "interp-yield-suspended-execution-record",
    ),
    (
        "crates/ash-cli/src/commands/daemon.rs",
        "DaemonStartArgs",
        "daemon-workflow-start-arguments",
    ),
    (
        "crates/ash-cli/src/commands/daemon.rs",
        "workflow: String",
        "daemon-workflow-request-field",
    ),
    (
        "crates/ash-cli/tests/",
        "workflow_path",
        "cli-test-stale-workflow-path-variable",
    ),
    (
        "crates/ash-cli/tests/",
        "workflow_file",
        "cli-test-stale-workflow-file-variable",
    ),
    (
        "crates/ash-cli/tests/",
        "write workflow",
        "cli-test-stale-write-workflow-label",
    ),
    (
        "crates/ash-cli/tests/run_output.rs",
        "entry_workflow",
        "cli-run-output-stale-entry-workflow-label",
    ),
    (
        "crates/ash-cli/tests/run_output.rs",
        "ordinary_non_entry_workflow",
        "cli-run-output-stale-ordinary-workflow-label",
    ),
    (
        "crates/ash-cli/tests/lexical_scope_conformance_test.rs",
        "let workflow =",
        "cli-lexical-stale-workflow-variable",
    ),
    (
        "crates/ash-engine/src/module_loader.rs",
        "workflow path",
        "engine-module-loader-stale-workflow-path-label",
    ),
    (
        "crates/ash-engine/src/module_loader.rs",
        "ordinary workflow",
        "engine-module-loader-stale-ordinary-workflow-label",
    ),
    (
        "crates/ash-engine/src/module_loader.rs",
        "workflow file",
        "engine-module-loader-stale-workflow-file-label",
    ),
    (
        "crates/ash-engine/src/module_loader.rs",
        "workflow source snapshot",
        "engine-module-loader-stale-workflow-source-snapshot-label",
    ),
    (
        "crates/ash-engine/src/module_loader/tests.rs",
        "A -> B",
        "engine-module-loader-test-stale-bare-callable-type",
    ),
    (
        "crates/ash-engine/src/module_loader/tests.rs",
        "Fn(A)",
        "engine-module-loader-test-stale-fn-callable-type",
    ),
    (
        "crates/ash-engine/src/module_loader/tests.rs",
        "act.ash",
        "engine-module-loader-test-stale-act-module-fixture",
    ),
    (
        "crates/ash-engine/src/module_loader/tests.rs",
        "std::act",
        "engine-module-loader-test-stale-act-module-label",
    ),
    (
        "crates/ash-engine/src/module_loader/tests.rs",
        "ActEnv",
        "engine-module-loader-test-stale-actenv-fixture",
    ),
    (
        "crates/ash-engine/src/module_loader.rs",
        "is_existing_opaque_compatibility_exception",
        "engine-module-loader-stale-opaque-compatibility-exception",
    ),
    (
        "crates/ash-engine/src/module_loader.rs",
        "std::act",
        "engine-module-loader-stale-std-act-exception",
    ),
    (
        "crates/ash-engine/src/module_loader.rs",
        "type_def.name == \"Act\"",
        "engine-module-loader-stale-act-type-special-case",
    ),
    (
        "crates/ash-engine/tests/inline_callable_signature_test.rs",
        "Fn(a)",
        "engine-test-stale-fn-callable-type-fixture",
    ),
    (
        "crates/ash-engine/tests/task_923_do_selected_evidence_monomorphize.rs",
        "Fn(Int)",
        "engine-test-stale-fn-callable-type-fixture",
    ),
    (
        "crates/ash-engine/src/tests.rs",
        "Fn(Int)",
        "engine-src-test-stale-fn-callable-type-fixture",
    ),
    (
        "crates/ash-engine/tests/task_1025_algebra_combinators.rs",
        "A -> B",
        "engine-test-stale-bare-callable-signature",
    ),
    (
        "crates/ash-engine/tests/task_1025_algebra_combinators.rs",
        "A -> M<B>",
        "engine-test-stale-bare-callable-signature",
    ),
    (
        "crates/ash-engine/tests/task_1025_algebra_combinators.rs",
        "F<A -> B>",
        "engine-test-stale-nested-bare-callable-signature",
    ),
    (
        "crates/ash-engine/src/lib.rs",
        "workflow file",
        "engine-lib-stale-workflow-file-label",
    ),
    (
        "crates/ash-engine/tests/runtime_boundary_visibility.rs",
        "workflow path",
        "engine-runtime-boundary-stale-workflow-path-label",
    ),
    (
        "crates/ash-cli/tests/json_output_schema_test.rs",
        "workflow filename",
        "cli-json-schema-stale-workflow-filename-label",
    ),
    (
        "std/README.md",
        "Fun(",
        "stdlib-readme-stale-fun-callable-syntax",
    ),
    (
        "std/README.md",
        "Option<T> ->",
        "stdlib-readme-stale-bare-option-callable-signature",
    ),
    (
        "std/README.md",
        "Result<T, E> ->",
        "stdlib-readme-stale-bare-result-callable-signature",
    ),
    (
        "crates/ash-parser/src/surface.rs",
        "obligation reference (legacy)",
        "parser-surface-stale-legacy-check-target-label",
    ),
    (
        "crates/ash-parser/tests/task_882_spec_h_surface_non_interference.rs",
        "legacy_impl_where",
        "parser-test-stale-legacy-impl-where-label",
    ),
    (
        "crates/ash-engine/tests/module_file_check_tests.rs",
        "legacy semicolon snippets",
        "engine-module-file-test-stale-legacy-snippet-label",
    ),
    (
        "crates/ash-interp/src/list_helpers.rs",
        "legacy list runtime variant",
        "interp-list-helper-stale-legacy-runtime-variant-label",
    ),
    (
        "crates/ash-core/src/workflow_carrier.rs",
        "legacy_contract",
        "core-workflow-carrier-stale-legacy-contract-field",
    ),
    (
        "crates/ash-core/tests/task_845_public_computation_summary_schema.rs",
        "legacy payload decodes",
        "core-summary-schema-test-stale-legacy-payload-label",
    ),
    (
        "crates/ash-core/tests/task_873_proposition_carriers.rs",
        "source_anchor: anchor(\"legacy proposition fact\")",
        "core-proposition-carrier-test-stale-legacy-fact-label",
    ),
    (
        "crates/ash-core/tests/task_879_proposition_summary_schema.rs",
        "module_identity(version.0 as usize, \"legacy\")",
        "core-proposition-summary-test-stale-legacy-module-label",
    ),
    (
        "crates/ash-core/tests/task_882_spec_h_summary_non_interference.rs",
        "legacy_payloads",
        "core-spec-h-summary-test-stale-legacy-payload-label",
    ),
    (
        "crates/ash-core/tests/task_882_spec_h_summary_non_interference.rs",
        "before_legacy_registration",
        "core-spec-h-summary-test-stale-legacy-registration-label",
    ),
    (
        "crates/ash-core/tests/task_882_spec_h_summary_non_interference.rs",
        "legacy_reject",
        "core-spec-h-summary-test-stale-legacy-reject-label",
    ),
    (
        "crates/ash-core/tests/task_882_spec_h_summary_non_interference.rs",
        "legacy summary version",
        "core-spec-h-summary-test-stale-legacy-version-label",
    ),
    (
        "crates/ash-core/tests/task_850_summary_versioning_cache.rs",
        "legacy-with-facts",
        "core-summary-versioning-test-stale-legacy-facts-label",
    ),
    (
        "crates/ash-core/tests/task_860_associated_family_carriers.rs",
        "legacy-with-family",
        "core-associated-family-test-stale-legacy-family-label",
    ),
    (
        "crates/ash-core/tests/alpha_runtime_kernel_carriers.rs",
        "actor:legacy",
        "core-runtime-kernel-test-stale-legacy-actor-label",
    ),
    (
        "crates/ash-core/tests/alpha_runtime_kernel_carriers.rs",
        "capability:legacy.call",
        "core-runtime-kernel-test-stale-legacy-capability-label",
    ),
    (
        "crates/ash-interp/tests/task_1922_external_actor_integration.rs",
        "actor:legacy",
        "interp-external-actor-test-stale-legacy-actor-label",
    ),
    (
        "crates/ash-interp/tests/task_1922_external_actor_integration.rs",
        "capability:legacy.call",
        "interp-external-actor-test-stale-legacy-capability-label",
    ),
    (
        "crates/ash-parser/src/lower/tests.rs",
        "preserve legacy vocabulary",
        "parser-lower-test-stale-legacy-vocabulary-label",
    ),
    (
        "crates/ash-interp/tests/task_1683_cps_multishot_validation.rs",
        "!s.contains(\"legacy\")",
        "interp-cps-validation-test-stale-legacy-negative-assertion",
    ),
    (
        "crates/ash-engine/tests/task_786_import_visibility_summary_rules/public_signatures.rs",
        "legacy_type_leaks",
        "engine-import-summary-test-stale-legacy-type-label",
    ),
    (
        "crates/ash-engine/src/module_loader/import_resolution.rs",
        "legacy git URL",
        "engine-import-resolution-stale-legacy-git-label",
    ),
    (
        "crates/ash-engine/tests/task_981_registry_metadata_lock_consumers.rs",
        "legacy_git",
        "engine-registry-lock-test-stale-legacy-git-variable",
    ),
    (
        "crates/ash-engine/tests/task_981_registry_metadata_lock_consumers.rs",
        "LegacyGit",
        "engine-registry-lock-test-stale-legacy-git-type",
    ),
    (
        "crates/ash-engine/tests/task_981_registry_metadata_lock_consumers.rs",
        "legacy git",
        "engine-registry-lock-test-stale-legacy-git-label",
    ),
    (
        "crates/ash-engine/src/providers/llm/chat.rs",
        "#[allow(deprecated)]",
        "engine-llm-chat-stale-deprecated-field-suppression",
    ),
    (
        "crates/ash-engine/src/providers/llm/stream_adapter.rs",
        "#[allow(deprecated)]",
        "engine-llm-stream-stale-deprecated-field-suppression",
    ),
    (
        "crates/ashgrove/",
        "legacy .ash.toml",
        "ashgrove-stale-legacy-manifest-label",
    ),
    (
        "crates/ashgrove/",
        "ash-legacy",
        "ashgrove-stale-legacy-manifest-fixture",
    ),
    (
        "crates/ashgrove/",
        "legacy git",
        "ashgrove-stale-legacy-git-label",
    ),
    (
        "crates/ashgrove/",
        "legacy source",
        "ashgrove-stale-legacy-source-label",
    ),
    (
        "crates/ashgrove/",
        "legacy_rev",
        "ashgrove-stale-legacy-source-variable",
    ),
    (
        "crates/ashgrove/",
        "legacy sentinel",
        "ashgrove-stale-legacy-sentinel-label",
    ),
    (
        "crates/ashgrove/",
        "reject_legacy_conflict",
        "ashgrove-stale-legacy-conflict-function",
    ),
    (
        "std/src/lib.ash",
        "workflow language",
        "stdlib-root-stale-workflow-language-label",
    ),
    (
        "std/src/runtime/error.ash",
        "entry-point workflows",
        "stdlib-runtime-error-stale-workflow-label",
    ),
    (
        "std/src/llm/",
        "workflow",
        "stdlib-llm-stale-workflow-comment-label",
    ),
    (
        "crates/ash-cli/tests/phase199_template_manifest.rs",
        "deprecated_template",
        "phase199-template-test-stale-deprecated-label",
    ),
    (
        "crates/ash-cli/tests/phase200_examples_current_syntax.rs",
        "legacy-workflow",
        "phase200-examples-test-stale-legacy-workflow-label",
    ),
    (
        "crates/ash-cli/tests/phase200_examples_current_syntax.rs",
        "has_legacy_marker",
        "phase200-examples-test-stale-legacy-marker-name",
    ),
    (
        "crates/ash-cli/tests/phase200_examples_current_syntax.rs",
        "retained_legacy_example_hits",
        "phase200-examples-test-stale-legacy-test-name",
    ),
    (
        "crates/ash-cli/tests/phase200_docs_current_syntax.rs",
        "legacy-workflow",
        "phase200-docs-test-stale-legacy-workflow-label",
    ),
    (
        "crates/ash-lsp-core/src/symbols.rs",
        "#![allow(deprecated",
        "lsp-symbols-stale-deprecated-protocol-suppression",
    ),
    (
        "crates/ash-lsp-core/src/symbols.rs",
        "deprecated: None",
        "lsp-symbols-stale-deprecated-protocol-field",
    ),
    (
        "crates/ash-parser/tests/task_1746_generated_identifier_hygiene.rs",
        "legacy_generated_helpers",
        "parser-generated-identifier-test-stale-legacy-helper-label",
    ),
    (
        "crates/ash-core/src/type_ir.rs",
        "legacy/imported projection",
        "core-type-ir-stale-legacy-imported-projection-label",
    ),
    (
        "crates/ash-core/src/type_ir.rs",
        "imported or legacy carriers",
        "core-type-ir-stale-imported-legacy-carrier-label",
    ),
    (
        "crates/ash-parser/tests/task_1911_process_concurrency_rows.rs",
        "legacy_proc_surface",
        "parser-process-row-test-stale-legacy-proc-label",
    ),
    (
        "crates/ash-parser/tests/task_874_proposition_surface.rs",
        "preserves_legacy_impl_where",
        "parser-test-stale-legacy-impl-where-label",
    ),
    (
        "crates/ash-parser/tests/task_881_proposition_parse_diagnostics.rs",
        "mask_legacy_impl_where",
        "parser-test-stale-legacy-impl-where-label",
    ),
    (
        "crates/ash-parser/tests/task_881_proposition_parse_diagnostics.rs",
        "legacy where bound",
        "parser-test-stale-legacy-where-bound-label",
    ),
    (
        "crates/ash-parser/src/parse_module/tests.rs",
        "legacy_capability",
        "parser-module-test-stale-legacy-capability-label",
    ),
    (
        "crates/ash-parser/src/parse_module/tests.rs",
        "legacy_capabilities",
        "parser-module-test-stale-legacy-capability-label",
    ),
    (
        "crates/ash-parser/src/lib.rs",
        "legacy_capability",
        "parser-lib-test-stale-legacy-capability-label",
    ),
    (
        "crates/ash-typeck/tests/task_825_non_inverting_unification_boundary.rs",
        "legacy_nominal",
        "typeck-test-stale-legacy-nominal-label",
    ),
    (
        "crates/ash-typeck/tests/task_827_normalizer_diagnostics.rs",
        "legacy meta solving",
        "typeck-test-stale-legacy-meta-label",
    ),
    (
        "crates/ash-typeck/tests/task_827_normalizer_diagnostics.rs",
        "legacy TypeEnv shape",
        "typeck-test-stale-legacy-typeenv-label",
    ),
    (
        "crates/ash-typeck/src/type_env/surface_types_laws_and_prelude.rs",
        "Unsupported legacy shapes",
        "typeck-typeenv-stale-legacy-shapes-label",
    ),
    (
        "crates/ash-typeck/src/normalizer.rs",
        "owned by the legacy",
        "typeck-normalizer-stale-legacy-unifier-label",
    ),
    (
        "crates/ash-typeck/tests/task_826_typeenv_forcing_point_rollout.rs",
        "legacy_meta_solving",
        "typeck-test-stale-legacy-meta-boundary-label",
    ),
    (
        "crates/ash-typeck/tests/task_826_typeenv_forcing_point_rollout.rs",
        "legacy Type::Var",
        "typeck-test-stale-legacy-typevar-label",
    ),
    (
        "crates/ash-typeck/tests/task_826_typeenv_forcing_point_rollout.rs",
        "legacy_fallback",
        "typeck-test-stale-legacy-fallback-label",
    ),
    (
        "crates/ash-typeck/tests/task_826_typeenv_forcing_point_rollout.rs",
        "Unsupported legacy shapes",
        "typeck-test-stale-legacy-shapes-label",
    ),
    (
        "crates/ash-typeck/tests/task_851_imported_type_function_normalizer.rs",
        "malformed_legacy_or_future_summary",
        "typeck-test-stale-legacy-summary-label",
    ),
    (
        "crates/ash-typeck/tests/task_852_type_computation_summary_diagnostics.rs",
        "legacy summary carrying computation fields",
        "typeck-test-stale-legacy-summary-label",
    ),
    (
        "crates/ash-typeck/tests/task_876_proposition_solver.rs",
        "legacy unification/substitution/meta evidence facts",
        "typeck-test-stale-legacy-proposition-evidence-label",
    ),
    (
        "crates/ash-typeck/tests/alpha_visible_computation_acceptance_matrix.rs",
        "legacy_surfaces",
        "typeck-alpha-test-stale-legacy-surface-label",
    ),
];

#[derive(Debug, Clone, Copy)]
struct Finding {
    line: usize,
    pattern: &'static str,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn should_scan(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("ash" | "core" | "md" | "rs" | "ron" | "snap" | "json" | "toml")
    )
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .expect("scanned path should be under repository root")
        .to_string_lossy()
        .replace('\\', "/")
}

fn is_excluded(relative: &str) -> bool {
    EXCLUDED_FILES.contains(&relative)
        || EXCLUDED_PREFIXES
            .iter()
            .any(|prefix| relative.starts_with(prefix))
}

fn collect_files(root: &Path, relative_root: &str, files: &mut Vec<PathBuf>) {
    let path = root.join(relative_root);
    if !path.exists() {
        return;
    }

    if path.is_file() {
        if should_scan(&path) {
            files.push(path);
        }
        return;
    }

    let entries = std::fs::read_dir(&path)
        .unwrap_or_else(|error| panic!("read scan root {}: {error}", path.display()));
    for entry in entries {
        let entry = entry.expect("read scan entry");
        let path = entry.path();
        if path.is_dir() {
            let relative = relative_path(root, &path);
            collect_files(root, &relative, files);
        } else if should_scan(&path) {
            files.push(path);
        }
    }
}

fn find_removed_form(line: &str) -> Option<&'static str> {
    let line = strip_ash_line_comment(line);
    if contains_token_followed_by_with(line, "observe") {
        return Some("observe-with");
    }
    if contains_token_followed_by_with(line, "act") {
        return Some("act-with");
    }
    if contains_removed_act_statement_form(line) {
        return Some("act-ret-statement");
    }
    if starts_with_removed_workflow_declaration(line) {
        return Some("workflow-keyword");
    }
    if contains_removed_workflow_header_clause(line) {
        return Some("workflow-header-clause");
    }
    if contains_removed_standalone_workflow_header_clause(line) {
        return Some("workflow-header-clause");
    }
    if contains_removed_capability_declaration(line) {
        return Some("capability-keyword");
    }
    if line_looks_like_ash_source_declaration(line) {
        if contains_removed_fn_constructor_callable_type(line) {
            return Some("fn-constructor-callable-type");
        }
        for name in REMOVED_TYPE_NAMES {
            if contains_type_constructor(line, name) {
                return Some(match *name {
                    "Act" => "act-carrier",
                    "Proc" => "proc-carrier",
                    "Workflow" => "workflow-carrier",
                    _ => unreachable!("removed type name table is fixed"),
                });
            }
        }
    }
    None
}

fn find_removed_form_in_rust_literal(line: &str) -> Option<&'static str> {
    if contains_token_followed_by_with(line, "observe") {
        return Some("observe-with");
    }
    if contains_token_followed_by_with(line, "act") {
        return Some("act-with");
    }
    if contains_removed_act_statement_form(line) {
        return Some("act-ret-statement");
    }
    if contains_removed_workflow_declaration_in_rust_literal(line) {
        return Some("workflow-keyword");
    }
    if contains_removed_workflow_header_clause_in_rust_literal(line) {
        return Some("workflow-header-clause");
    }
    if contains_removed_workflow_header_clause(line) {
        return Some("workflow-header-clause");
    }
    if contains_removed_standalone_workflow_header_clause(line) {
        return Some("workflow-header-clause");
    }
    if line_looks_like_ash_source_declaration(line) {
        if contains_removed_fn_constructor_callable_type(line) {
            return Some("fn-constructor-callable-type");
        }
        for pattern in ["Act<", "Proc<", "Workflow<"] {
            if !line.contains(pattern) {
                continue;
            }
            return Some(match pattern {
                "Act<" => "act-carrier",
                "Proc<" => "proc-carrier",
                "Workflow<" => "workflow-carrier",
                _ => unreachable!("removed Rust literal pattern table is fixed"),
            });
        }
    }
    if contains_removed_capability_declaration_in_rust_literal(line) {
        return Some("capability-keyword");
    }
    None
}

fn find_removed_ash_source_line_in_rust(line: &str) -> Option<&'static str> {
    let line = strip_ash_line_comment(line);
    if contains_token_followed_by_with(line, "observe") {
        return Some("observe-with");
    }
    if contains_token_followed_by_with(line, "act") {
        return Some("act-with");
    }
    if contains_removed_act_statement_form(line) {
        return Some("act-ret-statement");
    }
    if starts_with_removed_workflow_declaration(line) {
        return Some("workflow-keyword");
    }
    if contains_removed_workflow_header_clause(line) {
        return Some("workflow-header-clause");
    }
    if contains_removed_standalone_workflow_header_clause(line) {
        return Some("workflow-header-clause");
    }
    if contains_removed_capability_declaration(line) {
        return Some("capability-keyword");
    }
    if line_looks_like_ash_source_declaration(line) {
        if contains_removed_fn_constructor_callable_type(line) {
            return Some("fn-constructor-callable-type");
        }
        for name in REMOVED_TYPE_NAMES {
            if contains_type_constructor(line, name) {
                return Some(match *name {
                    "Act" => "act-carrier",
                    "Proc" => "proc-carrier",
                    "Workflow" => "workflow-carrier",
                    _ => unreachable!("removed type name table is fixed"),
                });
            }
        }
    }
    None
}

fn strip_ash_line_comment(line: &str) -> &str {
    line.split_once("--")
        .map_or(line, |(code, _comment)| code)
        .trim()
}

fn line_looks_like_rust_string_literal(line: &str) -> bool {
    line.contains('"') || line.contains("r#") || line.contains("r\"")
}

fn line_looks_like_ash_source_declaration(line: &str) -> bool {
    let trimmed = line.trim_start();
    [
        "fn ",
        "pub fn ",
        "builtin fn ",
        "pub builtin fn ",
        "type ",
        "pub type ",
        "builtin type ",
        "pub builtin type ",
        "interface ",
        "pub interface ",
        "impl ",
        "pub impl ",
    ]
    .iter()
    .any(|prefix| trimmed.starts_with(prefix))
}

fn contains_token_followed_by_with(line: &str, token: &str) -> bool {
    let mut rest = line;
    while let Some(index) = rest.find(token) {
        let before = rest[..index].chars().next_back();
        let after = rest[index + token.len()..].chars().next();
        let boundary_before = before.is_none_or(|ch| !is_ident_char(ch));
        let boundary_after = after.is_some_and(char::is_whitespace);
        if boundary_before && boundary_after {
            let after_token = &rest[index + token.len()..];
            if after_token
                .split_whitespace()
                .any(|part| part.trim_matches(|ch: char| !is_ident_char(ch)) == "with")
            {
                return true;
            }
        }
        rest = &rest[index + token.len()..];
    }
    false
}

fn contains_removed_capability_declaration(line: &str) -> bool {
    let trimmed = line.trim_start();
    if !(trimmed.starts_with("capability ") || trimmed.starts_with("pub capability ")) {
        return false;
    }

    let mut words = line
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .filter(|part| !part.is_empty());

    while let Some(word) = words.next() {
        if word != "capability" {
            continue;
        }

        let Some(next) = words.next() else {
            return false;
        };
        let Some(after_keyword) = line
            .split_once("capability")
            .map(|(_, rest)| rest.trim_start())
        else {
            return false;
        };

        if next == "interface" {
            return after_keyword
                .strip_prefix("interface")
                .is_some_and(capability_interface_remainder_is_source_shaped);
        }

        if next == "implementation" || next == "impl" {
            return after_keyword
                .strip_prefix(next)
                .is_some_and(capability_implementation_remainder_is_source_shaped);
        }

        let after_name = after_keyword
            .strip_prefix(next)
            .map(str::trim_start)
            .unwrap_or_default();

        return after_name.starts_with(':');
    }

    false
}

fn contains_removed_workflow_header_clause(line: &str) -> bool {
    let trimmed = line.trim_start();
    if !(trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ")) {
        return false;
    }

    [" capabilities:", " plays role", " owns ", " uses "]
        .iter()
        .any(|pattern| {
            trimmed
                .find(pattern)
                .is_some_and(|index| trimmed[..index].contains(')'))
        })
}

fn contains_removed_standalone_workflow_header_clause(line: &str) -> bool {
    let trimmed = line.trim_start();
    let source_shaped = trimmed.starts_with("capabilities:") && trimmed.contains('[');
    source_shaped
        && !trimmed.ends_with(',')
        && !trimmed.contains("vec!")
        && !trimmed.contains("Vec::")
        && !trimmed.contains("&str")
}

fn contains_removed_capability_declaration_in_rust_literal(line: &str) -> bool {
    for marker in [
        "\"capability ",
        "\\ncapability ",
        "\"pub capability ",
        "\\npub capability ",
    ] {
        let mut rest = line;
        while let Some(index) = rest.find(marker) {
            let candidate = &rest[index + marker.find("capability").unwrap_or(0)..];
            if contains_removed_capability_declaration(candidate) {
                return true;
            }
            rest = &rest[index + marker.len()..];
        }
    }
    false
}

fn contains_removed_workflow_declaration_in_rust_literal(line: &str) -> bool {
    let mut rest = line;
    while let Some(index) = rest.find("workflow ") {
        let after = &rest[index + "workflow ".len()..];
        let Some(name_end) = after.find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        else {
            return true;
        };
        let after_name = after[name_end..].trim_start();
        if after_name.is_empty()
            || after_name.starts_with('{')
            || after_name.starts_with('(')
            || after_name.starts_with("->")
            || after_name.starts_with("plays ")
            || after_name.starts_with("capabilities:")
            || after_name.starts_with("owns ")
            || after_name.starts_with("uses ")
        {
            return true;
        }
        rest = after;
    }
    false
}

fn contains_removed_workflow_header_clause_in_rust_literal(line: &str) -> bool {
    let mut rest = line;
    while let Some(index) = rest.find("fn ") {
        let candidate = &rest[index..];
        if contains_removed_workflow_header_clause(candidate) {
            return true;
        }
        rest = &candidate["fn ".len()..];
    }
    false
}

fn starts_with_removed_workflow_declaration(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("workflow ") || trimmed.starts_with("pub workflow ")
}

fn contains_removed_act_statement_form(line: &str) -> bool {
    let Some(block_start) = line.find("act {") else {
        return false;
    };
    let block = &line[block_start + "act {".len()..];
    block
        .split([';', '}'])
        .any(|statement| statement.split_whitespace().next() == Some("ret"))
}

fn capability_interface_remainder_is_source_shaped(rest: &str) -> bool {
    let rest = rest.trim_start();
    let Some(name_end) = rest.find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_')) else {
        return false;
    };
    let name = &rest[..name_end];
    !name.is_empty() && rest[name_end..].trim_start().starts_with(':')
}

fn capability_implementation_remainder_is_source_shaped(rest: &str) -> bool {
    let rest = rest.trim_start();
    let Some(name_end) = rest.find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_')) else {
        return false;
    };
    let name = &rest[..name_end];
    !name.is_empty() && rest[name_end..].contains(" for ")
}

fn contains_type_constructor(line: &str, name: &str) -> bool {
    let Some(start) = line.find(name) else {
        return false;
    };

    let before = line[..start].chars().next_back();
    let after = line[start + name.len()..].chars().next();
    let boundary_before = before.is_none_or(|ch| !is_ident_char(ch));
    let boundary_after = after.is_some_and(|ch| ch == '<' || ch == ':');

    boundary_before && boundary_after
}

fn contains_removed_fn_constructor_callable_type(line: &str) -> bool {
    let mut rest = line;
    while let Some(index) = rest.find("Fn") {
        let before_text = &rest[..index];
        let before = before_text.chars().next_back();
        let after = rest[index + "Fn".len()..].chars().next();
        let boundary_before = before.is_none_or(|ch| !is_ident_char(ch));
        let previous_word = before_text
            .split(|ch: char| !(ch == '_' || ch.is_ascii_alphanumeric()))
            .rfind(|part| !part.is_empty());
        if boundary_before && after == Some('(') && !matches!(previous_word, Some("dyn" | "impl")) {
            return true;
        }
        rest = &rest[index + "Fn".len()..];
    }
    false
}

fn is_ident_char(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

fn scan_source(source: &str) -> Vec<Finding> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            find_removed_form(line).map(|pattern| Finding {
                line: index + 1,
                pattern,
            })
        })
        .collect()
}

fn scan_markdown_source(source: &str) -> Vec<Finding> {
    let mut in_ash_fence = false;
    let mut findings = Vec::new();

    for (index, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            let info = trimmed.trim_start_matches('`').trim();
            in_ash_fence = !in_ash_fence && info.starts_with("ash");
            if !trimmed.starts_with("```ash") && trimmed.starts_with("```") {
                in_ash_fence = false;
            }
            continue;
        }

        let has_inline_snippet = line.contains('`')
            && [
                "workflow ",
                "capability ",
                "observe ",
                "act ",
                "Act<",
                "Proc<",
                "Workflow<",
                "Fn(",
            ]
            .iter()
            .any(|needle| line.contains(needle));

        if (in_ash_fence || has_inline_snippet)
            && let Some(pattern) = find_removed_form(line)
        {
            findings.push(Finding {
                line: index + 1,
                pattern,
            });
        }
    }

    findings
}

fn scan_rust_source(source: &str) -> Vec<Finding> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                return None;
            }

            let pattern = find_removed_ash_source_line_in_rust(trimmed).or_else(|| {
                line_looks_like_rust_string_literal(line)
                    .then(|| find_removed_form_in_rust_literal(line))
                    .flatten()
            })?;

            Some(Finding {
                line: index + 1,
                pattern,
            })
        })
        .collect()
}

fn scan_file_source(path: &Path, source: &str) -> Vec<Finding> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("ash") => scan_source(source),
        Some("core") => scan_core_text(source),
        Some("md") => scan_markdown_source(source),
        Some("rs") => scan_rust_source(source),
        Some("json" | "ron" | "snap") => scan_source(source),
        Some("toml") => Vec::new(),
        _ => Vec::new(),
    }
}

fn scan_core_text(source: &str) -> Vec<Finding> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let pattern = if line.contains("(cap ")
                || line.contains("{cap ")
                || line.contains(", cap ")
            {
                Some("core-cap-row-or-effect")
            } else if line.contains("(proc ") || line.contains("{proc ") || line.contains(", proc ")
            {
                Some("core-proc-row-or-effect")
            } else if line.contains("{op ") || line.contains(", op ") {
                Some("core-op-row-alias")
            } else {
                None
            }?;
            Some(Finding {
                line: index + 1,
                pattern,
            })
        })
        .collect()
}

#[test]
fn repository_contains_no_deprecated_ash_forms() {
    let root = repo_root();
    assert!(
        root.join("reference/status/removed-forms.md").exists(),
        "Phase 201 removed-form authority page must exist"
    );
    let mut files = Vec::new();
    for scan_root in SCAN_ROOTS {
        collect_files(&root, scan_root, &mut files);
    }
    files.sort();

    let mut failures = Vec::new();
    for path in files {
        let relative = relative_path(&root, &path);
        if is_excluded(&relative) {
            continue;
        }

        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
        for (path_pattern, text_pattern, finding) in REMOVED_ACTIVE_REFERENCES {
            let path_matches = if path_pattern.ends_with('/') {
                relative.starts_with(path_pattern)
            } else {
                relative == *path_pattern
            };
            if path_matches && (text_pattern.is_empty() || source.contains(text_pattern)) {
                failures.push(format!("{}:1:{}", relative, finding));
            }
        }
        for finding in scan_file_source(&path, &source) {
            failures.push(format!("{}:{}:{}", relative, finding.line, finding.pattern));
        }
    }

    assert!(
        failures.is_empty(),
        "Phase 201 requires repository Ash code to use target Ash only; remove these deprecated form hits:\n{}",
        failures.join("\n")
    );
}

#[test]
fn active_runtime_paths_have_no_stale_workflow_names() {
    let root = repo_root();
    let forbidden_references = [
        (
            "crates/ash-cli/src/commands/daemon.rs",
            "checked_workflow",
            "daemon-checked-workflow-local",
        ),
        (
            "crates/ash-core/src/type_ir.rs",
            "run/process/workflow",
            "tcir-failure-boundary-workflow-comment",
        ),
        (
            "crates/ash-typeck/src/check_expr/mod.rs",
            "pure closure syntax should unify with Type::Fn in workflow contexts",
            "typeck-pure-closure-workflow-context-message",
        ),
    ];

    let failures = forbidden_references
        .into_iter()
        .filter_map(|(relative, forbidden, finding)| {
            let source = std::fs::read_to_string(root.join(relative))
                .unwrap_or_else(|error| panic!("read {relative}: {error}"));
            source
                .contains(forbidden)
                .then_some(format!("{relative}:1:{finding}"))
        })
        .collect::<Vec<_>>();

    assert!(
        failures.is_empty(),
        "Phase 201 active runtime paths must use application/function terminology:\n{}",
        failures.join("\n")
    );
}

#[test]
fn productive_reference_docs_do_not_teach_removed_workflow_tower_model() {
    let root = repo_root();
    let forbidden_references = [
        (
            "reference/README.md",
            "covers the current `Act`, `Proc`, `Workflow`, and `Result` public library surfaces",
            "reference-root-current-tower-api",
        ),
        (
            "reference/getting-started/README.md",
            "Effect with Act/Proc.",
            "getting-started-act-proc-reading-model",
        ),
        (
            "reference/getting-started/README.md",
            "Orchestrate with Workflow.",
            "getting-started-workflow-reading-model",
        ),
        (
            "reference/getting-started/what-is-ash.md",
            "| Act | Effect with Act |",
            "what-is-ash-act-reading-model",
        ),
        (
            "reference/getting-started/what-is-ash.md",
            "| Proc | Effect with Proc |",
            "what-is-ash-proc-reading-model",
        ),
        (
            "reference/getting-started/what-is-ash.md",
            "| Workflow | Orchestrate with Workflow |",
            "what-is-ash-workflow-reading-model",
        ),
        (
            "reference/getting-started/run-a-program.md",
            "FILE[:WORKFLOW]",
            "run-guide-workflow-selection-syntax",
        ),
        (
            "reference/runtime/README.md",
            "checked workflow definitions",
            "runtime-index-workflow-definitions",
        ),
        (
            "reference/runtime/artifacts.md",
            "alpha_checked_workflow_boundary",
            "runtime-artifact-workflow-boundary",
        ),
        (
            "reference/tools/cli.md",
            "Type check workflow files",
            "cli-workflow-check-claim",
        ),
        (
            "reference/tools/cli.md",
            "Execute a workflow",
            "cli-workflow-run-claim",
        ),
        (
            "reference/language/functions/boundaries.md",
            "Put the call in an `Act`/runtime context.",
            "function-boundary-act-runtime-context",
        ),
        (
            "reference/stdlib/algebra.md",
            "`std::act`, `std::proc`, `std::workflow`",
            "stdlib-current-tower-module-guidance",
        ),
    ];

    let failures = forbidden_references
        .into_iter()
        .filter_map(|(relative, forbidden, finding)| {
            let source = std::fs::read_to_string(root.join(relative))
                .unwrap_or_else(|error| panic!("read {relative}: {error}"));
            source
                .contains(forbidden)
                .then_some(format!("{relative}:1:{finding}"))
        })
        .collect::<Vec<_>>();

    assert!(
        failures.is_empty(),
        "Phase 201 productive reference docs must route readers to target functions, rows, process helpers, and application runtime terms:\n{}",
        failures.join("\n")
    );
}

#[test]
fn productive_reference_docs_do_not_retain_residual_tower_read_paths() {
    let root = repo_root();
    let forbidden_references = [
        (
            "README.md",
            "Sharo Core workflow language",
            "project-root-workflow-language-claim",
        ),
        (
            "reference/language/functions.md",
            "workflow obligations",
            "function-reference-workflow-obligations",
        ),
        (
            "reference/language/functions.md",
            "workflow contracts",
            "function-reference-workflow-contracts",
        ),
        (
            "reference/language/functions.md",
            "Pure vs `Act`",
            "function-reference-act-boundary",
        ),
        (
            "reference/language/functions.md",
            "process/workflow values",
            "function-reference-workflow-values",
        ),
        (
            "reference/language/functions/local-and-anonymous.md",
            "Act-produced values",
            "local-function-act-produced-values",
        ),
        (
            "reference/language/functions/local-and-anonymous.md",
            "effect level Act",
            "local-function-act-effect-level",
        ),
        (
            "reference/getting-started/next-steps.md",
            "- ref.language.act",
            "next-steps-historical-act-read-path",
        ),
        (
            "reference/getting-started/next-steps.md",
            "- ref.language.proc",
            "next-steps-historical-proc-read-path",
        ),
        (
            "reference/getting-started/next-steps.md",
            "- ref.language.workflow",
            "next-steps-historical-workflow-read-path",
        ),
        (
            "reference/agents/common-confusions.md",
            "- ref.language.act",
            "agent-common-confusions-historical-act-read-path",
        ),
        (
            "reference/agents/cards/stdlib-result.md",
            "- ref.stdlib.act",
            "result-card-historical-act-read-path",
        ),
        (
            "reference/agents/cards/stdlib-result.md",
            "- ../../stdlib/act.md",
            "result-card-historical-act-preflight",
        ),
        (
            "reference/agents/cards/stdlib-result.md",
            "Result is Act.",
            "result-card-act-stale-claim",
        ),
        (
            "reference/stdlib/result.md",
            "- ref.stdlib.act",
            "result-reference-historical-act-read-path",
        ),
        (
            "reference/stdlib/result.md",
            "standalone runnable workflow",
            "result-reference-workflow-execution-claim",
        ),
        (
            "reference/agents/cards/functions.md",
            "workflow obligations",
            "function-card-workflow-obligations",
        ),
        (
            "reference/agents/cards/functions.md",
            "process/workflow boundaries",
            "function-card-workflow-boundaries",
        ),
        (
            "reference/agents/cards/functions.md",
            "process/workflow payloads",
            "function-card-workflow-payloads",
        ),
        (
            "reference/agents/cards/cps-operational-semantics.md",
            "until Proc/process semantics is defined",
            "cps-card-proc-semantics-claim",
        ),
        (
            "reference/tools/cli.md",
            "for a workflow file.",
            "cli-dot-workflow-file-claim",
        ),
        (
            "reference/tools/test.md",
            "arbitrary capability/Act/workflow execution",
            "test-guide-act-workflow-execution-claim",
        ),
    ];

    let failures = forbidden_references
        .into_iter()
        .filter_map(|(relative, forbidden, finding)| {
            let source = std::fs::read_to_string(root.join(relative))
                .unwrap_or_else(|error| panic!("read {relative}: {error}"));
            source
                .contains(forbidden)
                .then_some(format!("{relative}:1:{finding}"))
        })
        .collect::<Vec<_>>();

    assert!(
        failures.is_empty(),
        "Phase 201 active reference read paths must not retain removed tower or workflow guidance:\n{}",
        failures.join("\n")
    );
}

#[test]
fn active_language_docs_do_not_retain_removed_workflow_or_tower_forms() {
    let root = repo_root();
    let forbidden_references = [
        (
            "reference/language/functions/bodies-and-expressions.md",
            "workflow obligations",
            "function-bodies-workflow-obligations",
        ),
        (
            "reference/language/functions/calls-and-values.md",
            "workflow/process payloads",
            "function-calls-workflow-process-payloads",
        ),
        (
            "reference/language/functions/calls-and-values.md",
            "## Reserved tower callable arrows",
            "function-calls-tower-callable-arrow-heading",
        ),
        (
            "reference/language/types/records.md",
            "Workflow `observe` blocks",
            "record-guide-workflow-observe-claim",
        ),
        (
            "reference/language/types/records.md",
            "`observe test { let { x: a } = p; ... }`",
            "record-guide-workflow-observe-source-snippet",
        ),
        (
            "reference/language/types/records.md",
            "`act` blocks",
            "record-guide-act-block-claim",
        ),
        (
            "reference/language/types/records.md",
            "Workflow block destructuring",
            "record-guide-workflow-block-limitation",
        ),
        (
            "reference/language/types/records.md",
            "in `observe` and `act` blocks",
            "record-guide-observe-act-limitation",
        ),
    ];

    let failures = forbidden_references
        .into_iter()
        .filter_map(|(relative, forbidden, finding)| {
            let source = std::fs::read_to_string(root.join(relative))
                .unwrap_or_else(|error| panic!("read {relative}: {error}"));
            source
                .contains(forbidden)
                .then_some(format!("{relative}:1:{finding}"))
        })
        .collect::<Vec<_>>();

    assert!(
        failures.is_empty(),
        "Phase 201 current language guides must not retain removed workflow/tower prose or source-shaped forms:\n{}",
        failures.join("\n")
    );
}

#[test]
fn current_reference_metadata_does_not_route_to_live_tower_specs() {
    let root = repo_root();
    let current_reference_pages = [
        "reference/getting-started/what-is-ash.md",
        "reference/language/functions.md",
        "reference/language/functions/local-and-anonymous.md",
        "reference/language/functions/calls-and-values.md",
        "reference/agents/cards/functions.md",
        "reference/agents/cards/stdlib-result.md",
        "reference/stdlib/result.md",
        "reference/runtime/README.md",
        "reference/runtime/artifacts.md",
        "reference/runtime/kernel.md",
    ];
    let stale_specs = [
        "SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md",
        "SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md",
    ];

    let failures = current_reference_pages
        .into_iter()
        .flat_map(|relative| {
            let source = std::fs::read_to_string(root.join(relative))
                .unwrap_or_else(|error| panic!("read {relative}: {error}"));
            assert!(
                source.contains("status: current"),
                "metadata invariant must list only current pages: {relative}"
            );
            stale_specs.into_iter().filter_map(move |spec| {
                source
                    .contains(spec)
                    .then_some(format!("{relative}:1:current-page-routes-to-{spec}"))
            })
        })
        .collect::<Vec<_>>();

    assert!(
        failures.is_empty(),
        "Current reference metadata must route to target authority or removed-form status, not live tower specs:\n{}",
        failures.join("\n")
    );
}

#[test]
fn current_reference_status_cards_and_maintenance_do_not_retain_tower_authority() {
    let root = repo_root();
    let forbidden_references = [
        (
            "reference/agents/cards/runtime-kernel.md",
            "SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md",
            "runtime-kernel-card-live-tower-spec",
        ),
        (
            "reference/agents/cards/stdlib-algebra.md",
            "function-valued/tower law families",
            "stdlib-algebra-card-live-tower-law-claim",
        ),
        (
            "reference/language/functions/declarations.md",
            "SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md",
            "function-declarations-live-tower-spec",
        ),
        (
            "reference/language/functions/implementation-notes.md",
            "SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md",
            "function-implementation-notes-live-tower-spec",
        ),
        (
            "reference/language/functions/implementation-notes.md",
            "process/workflow serialization boundaries",
            "function-implementation-notes-workflow-boundaries",
        ),
        (
            "reference/language/functions/implementation-notes.md",
            "tower layers",
            "function-implementation-notes-tower-layers",
        ),
        (
            "reference/methodology.md",
            "changes to SPEC-069",
            "reference-methodology-live-tower-spec-trigger",
        ),
        (
            "reference/methodology.md",
            "std/src/{act,proc,workflow,result}.ash",
            "reference-methodology-live-tower-stdlib-trigger",
        ),
        (
            "reference/status/runtime-kernel.md",
            "SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md",
            "runtime-kernel-status-live-tower-spec",
        ),
        (
            "reference/status/runtime-kernel.md",
            "alpha_checked_workflow_boundary",
            "runtime-kernel-status-workflow-artifact-boundary",
        ),
        (
            "reference/status/runtime-kernel.md",
            "before workflow and spawned-child execution",
            "runtime-kernel-status-workflow-grant-claim",
        ),
        (
            "reference/status/runtime-kernel.md",
            "FILE[:WORKFLOW]",
            "runtime-kernel-status-workflow-selection-syntax",
        ),
        (
            "reference/status/alpha-limitations.md",
            "SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md",
            "alpha-limitations-live-tower-spec",
        ),
        (
            "reference/status/drift-report.md",
            "pilot pages list SPEC-069/SPEC-070",
            "drift-report-live-tower-spec-promotion",
        ),
    ];

    let failures = forbidden_references
        .into_iter()
        .filter_map(|(relative, forbidden, finding)| {
            let source = std::fs::read_to_string(root.join(relative))
                .unwrap_or_else(|error| panic!("read {relative}: {error}"));
            source
                .contains(forbidden)
                .then_some(format!("{relative}:1:{finding}"))
        })
        .collect::<Vec<_>>();

    assert!(
        failures.is_empty(),
        "Phase 201 current status, card, and maintenance reference paths must use target authority and application terminology:\n{}",
        failures.join("\n")
    );
}

#[test]
fn cli_library_docs_do_not_advertise_removed_dot_command() {
    let root = repo_root();
    let source = std::fs::read_to_string(root.join("crates/ash-cli/src/lib.rs"))
        .expect("read CLI library documentation");
    let forbidden = [
        "//! - `dot` - Generate Graphviz DOT output",
        "ash dot main.ash --output graph.dot",
    ];
    let failures = forbidden
        .into_iter()
        .filter(|phrase| source.contains(phrase))
        .collect::<Vec<_>>();

    assert!(
        failures.is_empty(),
        "Phase 201 CLI library docs must not advertise removed dot command forms: {}",
        failures.join(", ")
    );
}
