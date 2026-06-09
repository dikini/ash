//! Runner-facing synthesized-case schema and metadata types.

use std::collections::BTreeMap;

use ash_core::Expr as CoreExpr;
use serde::Serialize;
use serde_json::Value;

use crate::test_runner::types::TestSource;

/// Read-only runner-facing introspection snapshot for synthesized tests.
#[derive(Debug, Clone, Serialize, Default)]
pub struct RunnerIntrospectionSnapshot {
    /// Snapshot schema version.
    pub schema_version: String,
    /// Module or suite identity.
    pub module_identity: String,
    /// Source artifact identity used to produce the snapshot.
    pub source_artifact_id: String,
    /// Checked/lowered summary identity used to produce the snapshot.
    pub check_summary_id: String,
    /// Contract metadata rows.
    pub contracts: Vec<RunnerContractMetadata>,
    /// Policy metadata rows.
    pub policies: Vec<RunnerPolicyMetadata>,
    /// Obligation metadata rows.
    pub obligations: Vec<RunnerObligationMetadata>,
    /// Law metadata rows extracted from the parsed AST.
    pub laws: Vec<RunnerLawMetadata>,
    /// Available bounded generators.
    pub generators: Vec<TypeGeneratorDescriptor>,
    /// Available finite small-world domains.
    pub small_world_domains: Vec<SmallWorldDomain>,
    /// Unsupported metadata rows that may only produce deferred skip output.
    pub unsupported: Vec<IntrospectionUnsupportedReason>,
}

/// Runner-facing contract metadata.
#[derive(Debug, Clone, Serialize, Default)]
pub struct RunnerContractMetadata {
    /// Stable metadata id.
    pub id: String,
    /// Callable name.
    pub callable_name: String,
    /// Callable kind.
    pub callable_kind: String,
    /// Parameter names.
    pub param_names: Vec<String>,
    /// Parameter type names.
    pub param_types: Vec<String>,
    /// Return type name.
    pub return_type: Option<String>,
    /// Lowered `requires` predicates.
    pub lowered_requires: Vec<String>,
    /// Lowered `ensures` predicates.
    pub lowered_ensures: Vec<String>,
    /// Runtime postcondition identifiers.
    pub runtime_postconditions: Vec<String>,
    /// Checked/lowered executable postcondition oracles.
    pub executable_postconditions: Vec<ContractPostconditionOracle>,
    /// Explicit executable target metadata for contract target invocation.
    pub executable_target: Option<ContractExecutableTarget>,
    /// Bounded generation hints.
    pub generation_hints: Vec<TypeGeneratorDescriptor>,
    /// Case kinds this metadata can execute.
    pub executable_case_kinds: Vec<SynthesizedOracleKind>,
    /// Optional source span display.
    pub source_span: Option<String>,
}

/// Checked/lowered contract postcondition oracle metadata.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ContractPostconditionOracle {
    /// Human-readable postcondition text for reports.
    pub display: String,
    /// Checked/lowered oracle expression evaluated by the interpreter.
    pub expression: CoreExpr,
}

/// Narrow executable contract target metadata.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ContractExecutableTarget {
    /// Target kind.
    pub kind: ContractExecutableTargetKind,
    /// Stable target reference or callable name.
    pub target_ref: String,
    /// Explicit setup contract.
    pub setup: ContractExecutionSetup,
    /// Narrow executable target body model.
    pub body: ContractTargetBody,
}

/// Supported contract target kinds.
#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContractExecutableTargetKind {
    /// Pure function target.
    PureFunction,
    /// Act function target, deferred until executable capability setup is available.
    ActFunction,
    /// Workflow callable target, deferred until finite admission/setup is available.
    WorkflowCallable,
    /// Unsupported target kind.
    #[default]
    Unsupported,
}

/// Explicit contract execution setup metadata.
#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContractExecutionSetup {
    /// Pure target requires no external setup.
    PureNoSetup,
    /// Non-pure target has explicit finite setup metadata.
    ExplicitFinite,
    /// Setup metadata is missing.
    #[default]
    Missing,
    /// Setup metadata is present but unsupported.
    Unsupported,
}

/// Narrow executable contract target body model.
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ContractTargetBody {
    /// Return the value of a checked/lowered core expression evaluated by the interpreter.
    ReturnExpression {
        /// Checked/lowered core expression.
        expression: CoreExpr,
    },
    /// Return a literal JSON value.
    ReturnLiteral {
        /// Literal output.
        value: Value,
    },
    /// Unsupported target body.
    #[default]
    Unsupported,
}

/// Runner-facing policy metadata.
#[derive(Debug, Clone, Serialize, Default)]
pub struct RunnerPolicyMetadata {
    /// Stable metadata id.
    pub id: String,
    /// Policy name.
    pub policy_name: String,
    /// Bounded policy input domain descriptors.
    pub input_domain: Vec<TypeGeneratorDescriptor>,
    /// Lowered policy reference.
    pub lowered_policy_ref: Option<String>,
    /// Supported terminal outcomes.
    pub supported_terminal_outcomes: Vec<PolicyTerminalOutcome>,
    /// Oracle shape.
    pub oracle_shape: Option<PolicyOracleShape>,
    /// Explicit executable target/oracle metadata for supported policy execution.
    pub executable_target: Option<PolicyExecutableTarget>,
    /// Required authority summary.
    pub required_authority: Option<String>,
    /// Materialization limits summary.
    pub materialization_limits: Option<String>,
    /// Optional source span display.
    pub source_span: Option<String>,
}

/// Narrow executable policy target metadata.
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct PolicyExecutableTarget {
    /// Target kind.
    pub kind: PolicyExecutableTargetKind,
    /// Stable lowered policy target reference.
    pub target_ref: String,
    /// Explicit authority setup for the policy execution.
    pub authority_setup: PolicyAuthoritySetup,
    /// Stable terminal oracle evaluated against finite inputs.
    pub terminal_oracle: PolicyTerminalOracle,
}

/// Supported policy executable target kinds.
#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyExecutableTargetKind {
    /// Evaluate an explicit finite terminal oracle.
    TerminalOracle,
    /// Unsupported policy target kind.
    #[default]
    Unsupported,
}

/// Explicit authority setup for policy execution.
#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyAuthoritySetup {
    /// Policy metadata declares no required authority.
    NoAuthorityRequired,
    /// Required authority is explicitly present for this finite case.
    ExplicitAuthority {
        /// Authority granted for policy execution.
        authority: String,
    },
    /// Required authority setup is missing.
    #[default]
    Missing,
    /// Authority setup exists but is unsupported by this runner slice.
    Unsupported,
}

/// Stable terminal oracle evaluated by the policy runner.
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PolicyTerminalOracle {
    /// Exact field-match table over a finite input binding.
    ExactMatchTable {
        /// Binding name containing the policy input object.
        input_binding: String,
        /// Ordered rows; the first row whose fields match supplies the terminal.
        rows: Vec<PolicyTerminalOracleRow>,
    },
    /// Unsupported policy terminal oracle.
    #[default]
    Unsupported,
}

/// One exact terminal-oracle row.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PolicyTerminalOracleRow {
    /// Required input-object field values.
    pub when: BTreeMap<String, Value>,
    /// Terminal outcome produced when `when` matches.
    pub terminal: PolicyTerminalOutcome,
}

/// Runner-facing obligation metadata.
#[derive(Debug, Clone, Serialize, Default)]
pub struct RunnerObligationMetadata {
    /// Stable metadata id.
    pub id: String,
    /// Obligation name.
    pub obligation_name: String,
    /// Obligation scope.
    pub scope: String,
    /// Lifecycle model summary.
    pub lifecycle_model: Option<String>,
    /// Introduction sites.
    pub introduction_sites: Vec<String>,
    /// Discharge sites.
    pub discharge_sites: Vec<String>,
    /// Check sites.
    pub check_sites: Vec<String>,
    /// Required closeout behavior.
    pub required_closeout_behavior: Option<String>,
    /// Terminal expectations.
    pub terminal_expectations: Vec<ObligationTerminalExpectation>,
    /// Small-world derivation hints.
    pub small_world_derivation_hints: Vec<String>,
    /// Explicit typed lifecycle transition plan for the supported runner slice.
    pub lifecycle_transition_plan: Option<ObligationLifecycleTransitionPlan>,
    /// Explicit typed lifecycle transition traces ordered to `terminal_expectations`.
    pub lifecycle_transition_traces: Vec<ObligationLifecycleTransitionTrace>,
    /// Explicit finite lifecycle world states ordered to `terminal_expectations`.
    ///
    /// These states are preserved as reproducible world snapshots. TASK-1015
    /// requires pass/fail to come from typed transition execution rather than
    /// trusting this state's claimed `control_state`.
    pub lifecycle_worlds: Vec<SmallWorldState>,
    /// Optional source span display.
    pub source_span: Option<String>,
}

/// Scope where a law declaration was found.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LawScope {
    /// Law declared directly at module scope.
    Module,
    /// Law declared inside an interface definition.
    Interface,
}

/// Runner-facing law metadata extracted from the parsed surface AST.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RunnerLawMetadata {
    /// Stable metadata id.
    pub id: String,
    /// Law name.
    pub name: String,
    /// Declaration scope.
    pub scope: LawScope,
    /// Owning interface name for interface laws.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// Source-level parameter summaries in declaration order.
    pub params: Vec<String>,
    /// Source-level proposition summary.
    pub proposition: String,
    /// Explicit `by test "..."` delegation target, when this law is backed by a test proof.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegated_test: Option<String>,
}

/// Narrow explicit obligation lifecycle transition plan.
#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
pub struct ObligationLifecycleTransitionPlan {
    /// Supported lifecycle model.
    pub model: ObligationLifecycleModelKind,
    /// Introduction sites accepted by this plan.
    pub introduction_sites: Vec<String>,
    /// Discharge sites accepted by this plan.
    pub discharge_sites: Vec<String>,
    /// Closeout/check sites accepted by this plan.
    pub check_sites: Vec<String>,
    /// Required closeout behavior.
    pub required_closeout: ObligationCloseoutBehavior,
}

/// Supported typed obligation lifecycle models.
#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ObligationLifecycleModelKind {
    /// Introduce, discharge, and closeout-check finite state model.
    IntroduceDischargeCheck,
    /// Unsupported lifecycle model.
    #[default]
    Unsupported,
}

/// Supported obligation closeout behavior.
#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ObligationCloseoutBehavior {
    /// Reject an introduced obligation that remains open at closeout.
    RejectIfOpen,
    /// Unsupported closeout behavior.
    #[default]
    Unsupported,
}

/// Explicit typed lifecycle transition trace for one synthesized row.
#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
pub struct ObligationLifecycleTransitionTrace {
    /// Stable trace id.
    pub id: String,
    /// Ordered typed transitions to execute.
    pub transitions: Vec<ObligationLifecycleTransition>,
}

/// Typed lifecycle transition event.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ObligationLifecycleTransition {
    /// Introduce an obligation at a lowered introduction site.
    Introduce {
        /// Introduction site id.
        site: String,
    },
    /// Discharge an obligation at a lowered discharge site.
    Discharge {
        /// Discharge site id.
        site: String,
    },
    /// Check/close out an obligation at a lowered check site.
    Check {
        /// Check site id.
        site: String,
    },
    /// Explicit rejection observation. The executor validates that previous
    /// transitions already justify this rejection reason.
    Reject {
        /// Rejection reason.
        reason: ObligationLifecycleRejection,
    },
}

/// Supported lifecycle rejection reasons.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ObligationLifecycleRejection {
    /// Required discharge was missing at closeout.
    MissingDischarge,
    /// A discharge was attempted after the obligation was already discharged.
    DoubleDischarge,
    /// Unsupported rejection reason.
    Unsupported,
}

/// Exact bounded type generator descriptor for runner materialization.
#[derive(Debug, Clone, Serialize, Default)]
pub struct TypeGeneratorDescriptor {
    /// Stable generator id.
    pub id: String,
    /// Target type name.
    pub target_type: String,
    /// Generator source.
    pub source: TypeGeneratorSource,
    /// Exact bounded values when available.
    pub exact_values: Vec<Value>,
    /// Seed policy summary.
    pub seed_policy: Option<String>,
    /// Maximum generated cases.
    pub max_cases: Option<usize>,
    /// Reason this descriptor cannot materialize values.
    pub unsupported_reason: Option<String>,
}

/// Explicit finite small-world domain descriptor.
#[derive(Debug, Clone, Serialize)]
pub struct SmallWorldDomain {
    /// Stable domain id.
    pub id: String,
    /// Domain enumeration strategy.
    pub domain_kind: SmallWorldDomainKind,
    /// Value type summary for generated value worlds.
    pub value_type: Option<String>,
    /// Numeric bounds for bounded integer worlds.
    pub bounds: BTreeMap<String, i64>,
    /// Stable ordering policy summary.
    pub ordering_policy: Option<String>,
    /// Metadata source that produced this domain.
    pub source: TestSource,
    /// Reason this domain cannot be enumerated.
    pub unsupported_reason: Option<String>,
    /// Explicit values for value-domain worlds.
    pub explicit_values: Vec<Value>,
    /// Explicit canonical world states.
    pub explicit_states: Vec<SmallWorldState>,
    /// Explicit finite axes for bounded product worlds.
    pub product_axes: Vec<SmallWorldProductAxis>,
    /// Explicit finite element descriptor for bounded list worlds.
    pub list_descriptor: Option<SmallWorldListDescriptor>,
    /// Explicit finite role/capability inclusion-set descriptor.
    pub inclusion_descriptor: Option<SmallWorldInclusionSetDescriptor>,
    /// Explicit stable obligation lifecycle state-machine descriptor.
    pub lifecycle_descriptor: Option<SmallWorldLifecycleDescriptor>,
    /// Explicit stable policy-context descriptor.
    pub policy_context_descriptor: Option<SmallWorldPolicyContextDescriptor>,
    /// World oracle to evaluate for each enumerated state after target execution.
    pub oracle: Option<SmallWorldOracle>,
    /// Explicit executable target metadata for supported small-world execution.
    pub executable_target: Option<SmallWorldExecutableTarget>,
    /// Default world limit from metadata.
    pub max_worlds_default: Option<usize>,
}

impl Default for SmallWorldDomain {
    fn default() -> Self {
        Self {
            id: String::new(),
            domain_kind: SmallWorldDomainKind::Unsupported,
            value_type: None,
            bounds: BTreeMap::new(),
            ordering_policy: None,
            source: TestSource::Authored,
            unsupported_reason: None,
            explicit_values: Vec::new(),
            explicit_states: Vec::new(),
            product_axes: Vec::new(),
            list_descriptor: None,
            inclusion_descriptor: None,
            lifecycle_descriptor: None,
            policy_context_descriptor: None,
            oracle: None,
            executable_target: None,
            max_worlds_default: None,
        }
    }
}

/// One axis of an explicit bounded product world domain.
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct SmallWorldProductAxis {
    /// Binding name populated by this axis.
    pub binding: String,
    /// Exact finite values for this axis, in deterministic order.
    pub values: Vec<Value>,
}

/// Explicit bounded list world descriptor.
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct SmallWorldListDescriptor {
    /// Binding name populated with each materialized list.
    pub binding: String,
    /// Exact finite element representatives, in deterministic order.
    pub elements: Vec<Value>,
    /// Minimum list length to enumerate.
    pub min_len: usize,
    /// Maximum list length. Missing means open and must defer.
    pub max_len: Option<usize>,
}

/// Explicit finite role/capability inclusion-set descriptor.
#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
pub struct SmallWorldInclusionSetDescriptor {
    /// Finite role names, in deterministic order.
    pub roles: Vec<String>,
    /// Finite capability names, in deterministic order.
    pub capabilities: Vec<String>,
}

/// Explicit stable obligation lifecycle state-machine descriptor.
#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
pub struct SmallWorldLifecycleDescriptor {
    /// Obligation name or stable reference.
    pub obligation: String,
    /// Finite stable lifecycle states to materialize.
    pub states: Vec<SmallWorldLifecycleStateDescriptor>,
}

/// One stable obligation lifecycle state descriptor.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SmallWorldLifecycleStateDescriptor {
    /// Stable lifecycle world id.
    pub id: String,
    /// Executed terminal represented by this state.
    pub terminal: ObligationTerminalExpectation,
    /// Transition trace used to reach this state.
    pub transition_trace: Vec<String>,
}

/// Explicit stable policy-context descriptor.
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct SmallWorldPolicyContextDescriptor {
    /// Policy refs present in every materialized context.
    pub policies: Vec<String>,
    /// Finite stable contexts to materialize.
    pub contexts: Vec<SmallWorldPolicyContext>,
}

/// One stable policy context world.
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct SmallWorldPolicyContext {
    /// Stable context id.
    pub id: String,
    /// Roles present in this context.
    pub roles: Vec<String>,
    /// Capabilities present in this context.
    pub capabilities: Vec<String>,
    /// Finite bindings exposed to the executable target.
    pub bindings: BTreeMap<String, Value>,
    /// Optional evaluated policy control state.
    pub control_state: Option<String>,
}

/// Explicit executable small-world target metadata.
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct SmallWorldExecutableTarget {
    /// Target kind.
    pub kind: SmallWorldExecutableTargetKind,
    /// Stable target reference.
    pub target_ref: String,
    /// Explicit setup supported by this target.
    pub setup: ContractExecutionSetup,
    /// Narrow executable target body model.
    pub body: ContractTargetBody,
}

/// Supported small-world target kinds.
#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SmallWorldExecutableTargetKind {
    /// Pure target executed by evaluating a checked/lowered core expression over world bindings.
    PureExpression,
    /// Unsupported target kind.
    #[default]
    Unsupported,
}

/// Supported finite small-world domain kinds.
#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SmallWorldDomainKind {
    /// Explicit canonical states.
    ExplicitStates,
    /// Explicit values materialized as value-domain states.
    ExplicitValues,
    /// Boolean values in deterministic false/true order.
    Bool,
    /// Inclusive bounded integer range.
    BoundedInt,
    /// Bounded cartesian product of explicit finite axes.
    Product,
    /// Bounded finite list domain with explicit element representatives.
    List,
    /// Explicit finite role/capability inclusion-set worlds.
    RoleCapabilityInclusionSet,
    /// Explicit stable obligation lifecycle state-machine worlds.
    ObligationLifecycle,
    /// Explicit stable policy-context worlds.
    PolicyContext,
    /// Unsupported/deferred domain.
    #[default]
    Unsupported,
}

/// Canonical runner-facing small-world state.
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct SmallWorldState {
    /// Stable world id.
    pub id: String,
    /// World schema version.
    pub schema_version: String,
    /// World kind.
    pub world_kind: String,
    /// Value or symbolic bindings.
    pub bindings: BTreeMap<String, Value>,
    /// Capability names present in the world.
    pub capabilities: Vec<String>,
    /// Role names present in the world.
    pub roles: Vec<String>,
    /// Policy names or refs present in the world.
    pub policies: Vec<String>,
    /// Obligation names or refs present in the world.
    pub obligations: Vec<String>,
    /// Mailbox/messages present in the world.
    pub mailbox: Vec<Value>,
    /// Optional control state.
    pub control_state: Option<String>,
    /// Resource state snapshot.
    pub resource_state: BTreeMap<String, Value>,
    /// Transition trace used to reach the state.
    pub transition_trace: Vec<String>,
    /// Oracle refs attached to this world.
    pub oracle_refs: Vec<String>,
}

/// Small-world oracle descriptor.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SmallWorldOracle {
    /// Supported oracle kind.
    pub kind: SmallWorldOracleKind,
    /// Expected value.
    pub expected: Value,
}

/// Supported small-world oracle kinds.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SmallWorldOracleKind {
    /// The world `control_state` must equal the expected string.
    ControlStateEquals,
    /// The world `control_state` must be one of the expected strings.
    ControlStateIn,
    /// The world bindings must contain all expected object fields.
    BindingEquals,
    /// The executed target output must equal the expected value.
    TargetOutputEquals,
}

/// Source for generated representatives.
#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TypeGeneratorSource {
    /// Authored examples.
    AuthoredExamples,
    /// Exact finite domain.
    FiniteDomain,
    /// Representatives satisfying a contract.
    ContractValid,
    /// Nearby representatives violating a contract boundary.
    ContractInvalidNearby,
    /// Unsupported/deferred descriptor.
    #[default]
    Unsupported,
}

/// Supported synthesized oracle kinds.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SynthesizedOracleKind {
    /// Contract precondition boundary.
    PreconditionBoundary,
    /// Contract postcondition check.
    PostconditionHolds,
    /// Policy allow terminal.
    PolicyAllows,
    /// Policy deny terminal.
    PolicyDenies,
    /// Obligation introduction lifecycle check.
    ObligationIntroduced,
    /// Obligation discharge lifecycle check.
    ObligationDischarged,
    /// Missing discharge lifecycle rejection.
    ObligationMissingDischargeRejected,
    /// Double discharge lifecycle rejection.
    ObligationDoubleDischargeRejected,
}

/// Supported policy terminal outcome labels.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyTerminalOutcome {
    /// Allow terminal.
    Allow,
    /// Deny terminal.
    Deny,
    /// Approval terminal.
    Approval,
    /// Transform terminal.
    Transform,
    /// Unsupported terminal.
    Unsupported,
}

/// Supported policy oracle shape labels.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyOracleShape {
    /// Terminal outcome equality.
    TerminalEquals,
    /// Unsupported policy oracle.
    Unsupported,
}

/// Supported obligation terminal expectation labels.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ObligationTerminalExpectation {
    /// Obligation is introduced.
    Introduced,
    /// Obligation is discharged.
    Discharged,
    /// Missing discharge is rejected.
    MissingDischargeRejected,
    /// Double discharge is rejected.
    DoubleDischargeRejected,
    /// Unsupported lifecycle expectation.
    Unsupported,
}

/// Unsupported introspection row.
#[derive(Debug, Clone, Serialize, Default)]
pub struct IntrospectionUnsupportedReason {
    /// Metadata source kind.
    pub source_kind: String,
    /// Target name or id.
    pub target_name: String,
    /// Deferred reason.
    pub reason: String,
}
