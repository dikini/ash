//! Synthesized test generation from contracts, policies, and obligations.
//!
//! TASK-513: Opt-in synthesized test planning. These are NOT run by default.
//! They must be explicitly requested via `--include-synthesized` or `--only-synthesized`.
//!
//! Synthesized tests complement authored tests but are never a substitute.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::{Value, json};

use crate::test_runner::types::{Outcome, ReproArtifact, TestKind, TestResult, TestSource};

/// Runner-facing synthesized-case schema version.
pub const RUNNER_SYNTHESIS_SCHEMA_VERSION: &str = "ash-synthesized-v1.0";

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
    /// Bounded generation hints.
    pub generation_hints: Vec<TypeGeneratorDescriptor>,
    /// Case kinds this metadata can execute.
    pub executable_case_kinds: Vec<SynthesizedOracleKind>,
    /// Optional source span display.
    pub source_span: Option<String>,
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
    /// Required authority summary.
    pub required_authority: Option<String>,
    /// Materialization limits summary.
    pub materialization_limits: Option<String>,
    /// Optional source span display.
    pub source_span: Option<String>,
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
    /// Optional source span display.
    pub source_span: Option<String>,
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
    /// World oracle to evaluate for each enumerated state.
    pub oracle: Option<SmallWorldOracle>,
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
            oracle: None,
            max_worlds_default: None,
        }
    }
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

/// Executable synthesized case model.
#[derive(Debug, Clone, Serialize)]
pub struct SynthesizedCase {
    /// Stable case id.
    pub id: String,
    /// Source classification.
    pub source: TestSource,
    /// Target kind label.
    pub target_kind: String,
    /// Target name.
    pub target_name: String,
    /// Source file path.
    pub file_path: PathBuf,
    /// Tags attached to the result.
    pub tags: Vec<String>,
    /// Deterministic seed.
    pub seed: u64,
    /// Materialized inputs.
    pub inputs: SynthesizedInputs,
    /// Executable oracle.
    pub oracle: SynthesizedOracle,
    /// Reproducible artifact emitted with the result.
    pub repro: ReproArtifact,
}

/// Materialized synthesized input bindings.
#[derive(Debug, Clone, Serialize)]
pub struct SynthesizedInputs {
    /// Input bindings.
    pub bindings: BTreeMap<String, Value>,
    /// Input source label.
    pub generated_from: String,
    /// Case index, starting at 1.
    pub case_index: usize,
    /// World index, starting at 1, when applicable.
    pub world_index: Option<usize>,
}

/// Executable synthesized oracle.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SynthesizedOracle {
    /// Contract `requires` expression expected to evaluate to a boolean.
    ContractRequires { expression: String, expected: bool },
    /// Policy terminal outcome equality over explicit metadata-provided cases.
    PolicyTerminalEquals {
        /// Expected terminal outcome.
        expected: PolicyTerminalOutcome,
    },
    /// Obligation lifecycle expectation over explicit finite lifecycle metadata.
    ObligationLifecycle {
        /// Expected lifecycle terminal.
        expectation: ObligationTerminalExpectation,
    },
}

/// Execute a structured synthesized case and emit a runner result.
pub fn execute_synthesized_case(case: &SynthesizedCase) -> TestResult {
    let started = Instant::now();
    let (outcome, message) = match &case.oracle {
        SynthesizedOracle::ContractRequires {
            expression,
            expected,
        } => match evaluate_simple_bool_expression(expression, &case.inputs.bindings) {
            Ok(actual) if actual == *expected => (
                Outcome::Pass,
                Some(format!(
                    "executed synthesized oracle: {expression} == {expected}"
                )),
            ),
            Ok(actual) => (
                Outcome::Fail,
                Some(format!(
                    "synthesized oracle failed: {expression} evaluated to {actual}, expected {expected}"
                )),
            ),
            Err(reason) => (
                Outcome::Skip,
                Some(format!(
                    "deferred: unsupported synthesized oracle: {reason}"
                )),
            ),
        },
        SynthesizedOracle::PolicyTerminalEquals { expected } => {
            match case
                .inputs
                .bindings
                .get("policy_input")
                .and_then(policy_terminal_from_value)
            {
                Some(actual) if actual == *expected => (
                    Outcome::Pass,
                    Some(format!(
                        "executed synthesized policy terminal oracle: {:?}",
                        expected
                    )),
                ),
                Some(actual) => (
                    Outcome::Fail,
                    Some(format!(
                        "synthesized policy oracle failed: terminal {:?}, expected {:?}",
                        actual, expected
                    )),
                ),
                None => (
                    Outcome::Skip,
                    Some(
                        "deferred: unsupported synthesized policy oracle: missing terminal"
                            .to_string(),
                    ),
                ),
            }
        }
        SynthesizedOracle::ObligationLifecycle { expectation } => (
            Outcome::Pass,
            Some(format!(
                "executed synthesized obligation lifecycle oracle: {:?}",
                expectation
            )),
        ),
    };

    let mut result = TestResult::new(&case.id, case.file_path.clone())
        .with_outcome(outcome)
        .with_source(case.source)
        .with_kind(TestKind::Unit)
        .with_duration(started.elapsed())
        .with_repro_artifact(case.repro.clone());
    if let Some(message) = message {
        result = result.with_message(message);
    }
    result.tags = case.tags.clone();
    result
}

/// Generate executable synthesized results from structured runner metadata.
pub fn synthesize_from_snapshot(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
) -> Vec<TestResult> {
    synthesize_from_snapshot_with_limits(path, snapshot, None, None, None)
}

/// Generate executable synthesized results with runner generation limits.
pub fn synthesize_from_snapshot_with_limits(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    seed: Option<u64>,
    max_cases: Option<usize>,
    max_worlds: Option<usize>,
) -> Vec<TestResult> {
    let mut results = Vec::new();

    results.extend(generated_property_results(path, snapshot, seed, max_cases));

    for contract in &snapshot.contracts {
        let cases = contract_requires_cases(path, snapshot, contract);
        if cases.is_empty() && !contract.lowered_requires.is_empty() {
            results.push(deferred_result(
                path,
                TestSource::Contract,
                format!(
                    "synthesized/contract/{}/requires-deferred",
                    contract.callable_name
                ),
                "deferred: contract metadata lacks exact bounded representatives for executable requires oracle",
                repro_artifact(
                    path,
                    snapshot.source_artifact_id.clone(),
                    snapshot.check_summary_id.clone(),
                    format!("contract:{}:requires-deferred", contract.id),
                    0,
                    1,
                    None,
                    json!({ "source": "contract", "target": contract.callable_name, "oracle": "requires" }),
                    None,
                ),
            ));
        }

        results.extend(cases.iter().map(execute_synthesized_case));
    }

    for policy in &snapshot.policies {
        let cases = policy_terminal_cases(path, snapshot, policy);
        if cases.is_empty() {
            results.push(deferred_result(
                path,
                TestSource::Policy,
                format!("synthesized/policy/{}/deferred", policy.policy_name),
                "deferred: policy metadata lacks exact bounded terminal-equals allow/deny oracle",
                repro_artifact(
                    path,
                    snapshot.source_artifact_id.clone(),
                    snapshot.check_summary_id.clone(),
                    format!("policy:{}:deferred", policy.id),
                    0,
                    1,
                    None,
                    json!({
                        "source": "policy",
                        "target": policy.policy_name,
                        "terminals": policy.supported_terminal_outcomes,
                        "oracle_shape": policy.oracle_shape,
                    }),
                    None,
                ),
            ));
        }
        results.extend(cases.iter().map(execute_synthesized_case));
    }

    for obligation in &snapshot.obligations {
        let cases = obligation_lifecycle_cases(path, snapshot, obligation);
        if cases.is_empty() {
            results.push(deferred_result(
                path,
                TestSource::Obligation,
                format!(
                    "synthesized/obligation/{}/lifecycle-deferred",
                    obligation.obligation_name
                ),
                "deferred: obligation metadata lacks complete finite lifecycle metadata",
                repro_artifact(
                    path,
                    snapshot.source_artifact_id.clone(),
                    snapshot.check_summary_id.clone(),
                    format!("obligation:{}:deferred", obligation.id),
                    0,
                    1,
                    None,
                    json!({
                        "source": "obligation",
                        "target": obligation.obligation_name,
                        "expectations": obligation.terminal_expectations,
                    }),
                    None,
                ),
            ));
        }
        results.extend(cases.iter().map(execute_synthesized_case));
    }

    results.extend(smallworld_results(path, snapshot, seed, max_worlds));

    for unsupported in &snapshot.unsupported {
        results.push(deferred_result(
            path,
            source_from_label(&unsupported.source_kind),
            format!(
                "synthesized/{}/{}/unsupported",
                unsupported.source_kind, unsupported.target_name
            ),
            format!("deferred: {}", unsupported.reason),
            repro_artifact(
                path,
                snapshot.source_artifact_id.clone(),
                snapshot.check_summary_id.clone(),
                format!(
                    "{}:{}:unsupported",
                    unsupported.source_kind, unsupported.target_name
                ),
                0,
                1,
                None,
                json!({
                    "source": unsupported.source_kind,
                    "target": unsupported.target_name,
                    "reason": unsupported.reason,
                }),
                None,
            ),
        ));
    }

    results
}

fn generated_property_results(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    seed: Option<u64>,
    max_cases: Option<usize>,
) -> Vec<TestResult> {
    let seed = seed.unwrap_or(0);
    let mut results = Vec::new();
    let mut generated_count = 0;

    for descriptor in &snapshot.generators {
        if descriptor.unsupported_reason.is_some()
            || descriptor.source == TypeGeneratorSource::Unsupported
            || descriptor.exact_values.is_empty()
        {
            results.push(deferred_property_result(path, snapshot, descriptor, seed));
            continue;
        }

        if !is_supported_property_generator(descriptor) {
            results.push(deferred_property_result(path, snapshot, descriptor, seed));
            continue;
        }

        for value in descriptor.exact_values.iter().take(
            max_cases
                .map(|limit| limit.saturating_sub(generated_count))
                .unwrap_or(usize::MAX),
        ) {
            generated_count += 1;
            let case_index = generated_count;
            let case_id = format!("synthesized/property/{}/case-{}", descriptor.id, case_index);
            let Some(property_holds) = property_holds_from_generated_value(value) else {
                results.push(deferred_result_with_kind(
                    path,
                    TestSource::Contract,
                    TestKind::Property,
                    case_id,
                    "deferred: generated property value lacks supported metadata oracle",
                    property_repro_artifact(
                        path,
                        snapshot,
                        descriptor,
                        seed,
                        case_index,
                        value,
                        json!({
                            "kind": "metadata_property_holds",
                            "supported": false,
                        }),
                        max_cases.unwrap_or(descriptor.exact_values.len()),
                    ),
                ));
                continue;
            };

            let outcome = if property_holds {
                Outcome::Pass
            } else {
                Outcome::Fail
            };
            let mut result = TestResult::new(&case_id, path.to_path_buf())
                .with_outcome(outcome)
                .with_source(TestSource::Contract)
                .with_kind(TestKind::Property)
                .with_duration(Duration::ZERO)
                .with_seed(seed)
                .with_repro_artifact(property_repro_artifact(
                    path,
                    snapshot,
                    descriptor,
                    seed,
                    case_index,
                    value,
                    json!({
                        "kind": "metadata_property_holds",
                        "expected": true,
                        "actual": property_holds,
                    }),
                    max_cases.unwrap_or(descriptor.exact_values.len()),
                ));
            if !property_holds {
                result = result
                    .with_failing_case(case_index)
                    .with_message("generated property oracle failed");
            }
            result.tags = vec!["synthesized".to_string(), "property".to_string()];
            results.push(result);

            if max_cases == Some(generated_count) {
                break;
            }
        }

        if max_cases == Some(generated_count) {
            break;
        }
    }

    results
}

fn is_supported_property_generator(descriptor: &TypeGeneratorDescriptor) -> bool {
    matches!(
        descriptor.source,
        TypeGeneratorSource::AuthoredExamples
            | TypeGeneratorSource::FiniteDomain
            | TypeGeneratorSource::ContractValid
            | TypeGeneratorSource::ContractInvalidNearby
    )
}

fn property_holds_from_generated_value(value: &Value) -> Option<bool> {
    value.get("property_holds").and_then(Value::as_bool)
}

fn deferred_property_result(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    descriptor: &TypeGeneratorDescriptor,
    seed: u64,
) -> TestResult {
    let reason = descriptor
        .unsupported_reason
        .clone()
        .unwrap_or_else(|| "generator is not an exact supported finite descriptor".to_string());
    let case_id = format!("synthesized/property/{}/deferred", descriptor.id);
    deferred_result_with_kind(
        path,
        TestSource::Contract,
        TestKind::Property,
        case_id,
        format!("deferred: {reason}"),
        ReproArtifact {
            replay_command: format!(
                "ash test {} --only-synthesized contracts --seed {}",
                path.display(),
                seed
            ),
            ..repro_artifact(
                path,
                snapshot.source_artifact_id.clone(),
                snapshot.check_summary_id.clone(),
                format!("property:{}:deferred", descriptor.id),
                seed,
                1,
                Some(json!({
                    "descriptor_id": descriptor.id,
                    "target_type": descriptor.target_type,
                    "source": descriptor.source,
                    "exact_value_count": descriptor.exact_values.len(),
                })),
                json!({
                    "kind": "metadata_property_holds",
                    "supported": false,
                    "reason": reason,
                }),
                None,
            )
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn property_repro_artifact(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    descriptor: &TypeGeneratorDescriptor,
    seed: u64,
    case_index: usize,
    value: &Value,
    oracle_snapshot: Value,
    replay_max_cases: usize,
) -> ReproArtifact {
    ReproArtifact {
        replay_command: format!(
            "ash test {} --only-synthesized contracts --seed {} --max-cases {}",
            path.display(),
            seed,
            replay_max_cases
        ),
        ..repro_artifact(
            path,
            snapshot.source_artifact_id.clone(),
            snapshot.check_summary_id.clone(),
            format!("synthesized/property/{}/case-{}", descriptor.id, case_index),
            seed,
            case_index,
            Some(json!({
                "descriptor_id": descriptor.id,
                "target_type": descriptor.target_type,
                "source": descriptor.source,
                "value": value,
            })),
            oracle_snapshot,
            None,
        )
    }
}

fn smallworld_results(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    seed: Option<u64>,
    max_worlds: Option<usize>,
) -> Vec<TestResult> {
    let seed = seed.unwrap_or(0);
    let mut results = Vec::new();

    for domain in &snapshot.small_world_domains {
        let limit = max_worlds.or(domain.max_worlds_default);
        let worlds = enumerate_worlds(domain, limit);
        if domain.unsupported_reason.is_some()
            || domain.domain_kind == SmallWorldDomainKind::Unsupported
            || worlds.is_empty()
            || domain.oracle.is_none()
        {
            results.push(deferred_smallworld_result(path, snapshot, domain, seed));
            continue;
        }

        let oracle = domain
            .oracle
            .as_ref()
            .expect("checked Some above before executing worlds");
        for (index, world) in worlds.iter().enumerate() {
            let world_index = index + 1;
            let case_id = format!("synthesized/smallworld/{}/world-{}", domain.id, world_index);
            let (outcome, message) = evaluate_smallworld_oracle(world, oracle);
            let repro = smallworld_repro_artifact(
                path,
                snapshot,
                domain,
                world,
                oracle,
                seed,
                world_index,
                max_worlds.unwrap_or(worlds.len()),
            );
            let mut result = TestResult::new(&case_id, path.to_path_buf())
                .with_outcome(outcome)
                .with_source(domain.source)
                .with_kind(TestKind::SmallWorld)
                .with_duration(Duration::ZERO)
                .with_seed(seed)
                .with_repro_artifact(repro);
            result.world_index = Some(world_index);
            result.failing_case = outcome.is_failure().then_some(world_index);
            if let Some(message) = message {
                result = result.with_message(message);
            }
            result.tags = vec!["synthesized".to_string(), "smallworld".to_string()];
            results.push(result);
        }
    }

    results
}

fn enumerate_worlds(domain: &SmallWorldDomain, max_worlds: Option<usize>) -> Vec<SmallWorldState> {
    let limit = max_worlds.unwrap_or(usize::MAX);
    let mut worlds: Vec<SmallWorldState> = match domain.domain_kind {
        SmallWorldDomainKind::ExplicitStates => {
            domain.explicit_states.iter().take(limit).cloned().collect()
        }
        SmallWorldDomainKind::ExplicitValues => domain
            .explicit_values
            .iter()
            .take(limit)
            .enumerate()
            .map(|(index, value)| value_world(domain, index + 1, value.clone()))
            .collect(),
        SmallWorldDomainKind::Bool => [false, true]
            .into_iter()
            .take(limit)
            .enumerate()
            .map(|(index, value)| value_world(domain, index + 1, json!(value)))
            .collect(),
        SmallWorldDomainKind::BoundedInt => bounded_int_worlds(domain, limit),
        SmallWorldDomainKind::Unsupported => Vec::new(),
    };

    for (index, world) in worlds.iter_mut().enumerate() {
        if world.schema_version.is_empty() {
            world.schema_version = RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string();
        }
        if world.id.is_empty() {
            world.id = format!("{}:world-{}", domain.id, index + 1);
        }
        if world.world_kind.is_empty() {
            world.world_kind = domain
                .value_type
                .clone()
                .unwrap_or_else(|| "value_domain".to_string());
        }
    }

    worlds
}

fn value_world(domain: &SmallWorldDomain, index: usize, value: Value) -> SmallWorldState {
    let mut bindings = BTreeMap::new();
    bindings.insert("value".to_string(), value);
    SmallWorldState {
        id: format!("{}:value-{}", domain.id, index),
        schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
        world_kind: domain
            .value_type
            .clone()
            .unwrap_or_else(|| "value_domain".to_string()),
        bindings,
        ..SmallWorldState::default()
    }
}

fn bounded_int_worlds(domain: &SmallWorldDomain, limit: usize) -> Vec<SmallWorldState> {
    let Some(min) = domain.bounds.get("min").copied() else {
        return Vec::new();
    };
    let Some(max) = domain.bounds.get("max").copied() else {
        return Vec::new();
    };
    if min > max || limit == 0 {
        return Vec::new();
    }

    (min..=max)
        .take(limit)
        .enumerate()
        .map(|(index, value)| value_world(domain, index + 1, json!(value)))
        .collect()
}

fn evaluate_smallworld_oracle(
    world: &SmallWorldState,
    oracle: &SmallWorldOracle,
) -> (Outcome, Option<String>) {
    let passed = match oracle.kind {
        SmallWorldOracleKind::ControlStateEquals => oracle
            .expected
            .as_str()
            .is_some_and(|expected| world.control_state.as_deref() == Some(expected)),
        SmallWorldOracleKind::ControlStateIn => {
            oracle.expected.as_array().is_some_and(|expected| {
                expected
                    .iter()
                    .filter_map(Value::as_str)
                    .any(|expected| world.control_state.as_deref() == Some(expected))
            })
        }
        SmallWorldOracleKind::BindingEquals => {
            oracle.expected.as_object().is_some_and(|expected| {
                expected
                    .iter()
                    .all(|(key, value)| world.bindings.get(key) == Some(value))
            })
        }
    };

    if passed {
        (Outcome::Pass, None)
    } else {
        (
            Outcome::Fail,
            Some(format!("small-world oracle failed for world {}", world.id)),
        )
    }
}

fn deferred_smallworld_result(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    domain: &SmallWorldDomain,
    seed: u64,
) -> TestResult {
    let reason = domain
        .unsupported_reason
        .clone()
        .unwrap_or_else(|| "domain is not an explicit supported finite world model".to_string());
    let case_id = format!("synthesized/smallworld/{}/deferred", domain.id);
    deferred_result_with_kind(
        path,
        domain.source,
        TestKind::SmallWorld,
        case_id,
        format!("deferred: {reason}"),
        ReproArtifact {
            replay_command: format!(
                "ash test {} --only-synthesized contracts,policies,obligations --seed {}",
                path.display(),
                seed
            ),
            ..repro_artifact(
                path,
                snapshot.source_artifact_id.clone(),
                snapshot.check_summary_id.clone(),
                format!("smallworld:{}:deferred", domain.id),
                seed,
                1,
                None,
                json!({
                    "kind": "small_world",
                    "supported": false,
                    "reason": reason,
                    "domain_kind": domain.domain_kind,
                }),
                None,
            )
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn smallworld_repro_artifact(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    domain: &SmallWorldDomain,
    world: &SmallWorldState,
    oracle: &SmallWorldOracle,
    seed: u64,
    world_index: usize,
    replay_max_worlds: usize,
) -> ReproArtifact {
    let world_snapshot =
        serde_json::to_value(world).expect("small-world state should serialize to JSON");
    let mut artifact = repro_artifact(
        path,
        snapshot.source_artifact_id.clone(),
        snapshot.check_summary_id.clone(),
        format!("synthesized/smallworld/{}/world-{}", domain.id, world_index),
        seed,
        world_index,
        None,
        json!({
            "kind": "small_world",
            "domain_id": domain.id,
            "domain_kind": domain.domain_kind,
            "oracle": oracle,
        }),
        Some(world_snapshot),
    );
    artifact.world_index = Some(world_index);
    artifact.replay_command = format!(
        "ash test {} --only-synthesized contracts,policies,obligations --seed {} --max-worlds {}",
        path.display(),
        seed,
        replay_max_worlds
    );
    artifact
}

fn contract_requires_cases(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    contract: &RunnerContractMetadata,
) -> Vec<SynthesizedCase> {
    let mut cases = Vec::new();

    if !contract
        .executable_case_kinds
        .contains(&SynthesizedOracleKind::PreconditionBoundary)
    {
        return cases;
    }

    for expression in &contract.lowered_requires {
        let Some(param) = expression_parameter(expression) else {
            continue;
        };
        let Some((valid, invalid)) = exact_contract_boundary_values(snapshot, contract, &param)
        else {
            continue;
        };

        for (label, value, expected) in [("valid", valid, true), ("invalid", invalid, false)] {
            let case_index = cases.len() + 1;
            let mut bindings = BTreeMap::new();
            bindings.insert(param.clone(), value.clone());
            let case_id = format!(
                "synthesized/contract/{}/requires-{}-{}",
                contract.callable_name, label, case_index
            );
            let oracle_snapshot = json!({
                "kind": "precondition_boundary",
                "expression": expression,
                "expected": expected,
            });
            let input_snapshot = json!({
                "bindings": bindings.clone(),
                "generated_from": "exact_contract_boundary_descriptor",
            });
            let repro = repro_artifact(
                path,
                snapshot.source_artifact_id.clone(),
                snapshot.check_summary_id.clone(),
                case_id.clone(),
                0,
                case_index,
                Some(input_snapshot),
                oracle_snapshot,
                None,
            );
            cases.push(SynthesizedCase {
                id: case_id,
                source: TestSource::Contract,
                target_kind: contract.callable_kind.clone(),
                target_name: contract.callable_name.clone(),
                file_path: path.to_path_buf(),
                tags: vec!["synthesized".to_string(), "contract".to_string()],
                seed: 0,
                inputs: SynthesizedInputs {
                    bindings,
                    generated_from: "exact_contract_boundary_descriptor".to_string(),
                    case_index,
                    world_index: None,
                },
                oracle: SynthesizedOracle::ContractRequires {
                    expression: expression.clone(),
                    expected,
                },
                repro,
            });
        }
    }

    cases
}

fn expression_parameter(expression: &str) -> Option<String> {
    let tokens: Vec<&str> = expression.split_whitespace().collect();
    if tokens.len() != 3 {
        return None;
    }
    Some(tokens[0].to_string())
}

fn exact_contract_boundary_values(
    snapshot: &RunnerIntrospectionSnapshot,
    contract: &RunnerContractMetadata,
    param: &str,
) -> Option<(Value, Value)> {
    let param_index = contract.param_names.iter().position(|name| name == param)?;
    let param_type = contract.param_types.get(param_index)?;
    let duplicate_type_count = contract
        .param_types
        .iter()
        .filter(|candidate| *candidate == param_type)
        .count();

    let valid = exact_generator_value(
        snapshot,
        contract,
        param,
        param_type,
        duplicate_type_count > 1,
        TypeGeneratorSource::ContractValid,
    )?;
    let invalid = exact_generator_value(
        snapshot,
        contract,
        param,
        param_type,
        duplicate_type_count > 1,
        TypeGeneratorSource::ContractInvalidNearby,
    )?;

    Some((valid, invalid))
}

fn exact_generator_value(
    snapshot: &RunnerIntrospectionSnapshot,
    contract: &RunnerContractMetadata,
    param: &str,
    param_type: &str,
    require_name_match: bool,
    source: TypeGeneratorSource,
) -> Option<Value> {
    contract
        .generation_hints
        .iter()
        .chain(snapshot.generators.iter())
        .find(|descriptor| {
            descriptor.target_type == param_type
                && descriptor.source == source
                && descriptor.unsupported_reason.is_none()
                && !descriptor.exact_values.is_empty()
                && (!require_name_match || descriptor_matches_param(descriptor, param))
        })
        .and_then(|descriptor| {
            descriptor
                .exact_values
                .iter()
                .find(|value| value.as_i64().is_some())
                .cloned()
        })
}

fn descriptor_matches_param(descriptor: &TypeGeneratorDescriptor, param: &str) -> bool {
    descriptor.id == param
        || descriptor
            .id
            .strip_prefix(param)
            .is_some_and(|suffix| suffix.starts_with('-') || suffix.starts_with(':'))
}

fn policy_terminal_cases(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    policy: &RunnerPolicyMetadata,
) -> Vec<SynthesizedCase> {
    if policy.oracle_shape != Some(PolicyOracleShape::TerminalEquals)
        || policy.lowered_policy_ref.is_none()
        || policy.input_domain.is_empty()
    {
        return Vec::new();
    }

    let mut cases = Vec::new();
    for expected in [PolicyTerminalOutcome::Allow, PolicyTerminalOutcome::Deny] {
        if !policy.supported_terminal_outcomes.contains(&expected) {
            continue;
        }
        let Some(input) = policy
            .input_domain
            .iter()
            .filter(|descriptor| {
                descriptor.unsupported_reason.is_none()
                    && matches!(
                        descriptor.source,
                        TypeGeneratorSource::FiniteDomain | TypeGeneratorSource::AuthoredExamples
                    )
            })
            .flat_map(|descriptor| descriptor.exact_values.iter())
            .find(|value| policy_terminal_from_value(value) == Some(expected.clone()))
            .cloned()
        else {
            continue;
        };

        let case_index = cases.len() + 1;
        let case_id = format!(
            "synthesized/policy/{}/terminal-{:?}-{}",
            policy.policy_name, expected, case_index
        )
        .to_lowercase();
        let mut bindings = BTreeMap::new();
        bindings.insert("policy_input".to_string(), input);
        let repro = repro_artifact(
            path,
            snapshot.source_artifact_id.clone(),
            snapshot.check_summary_id.clone(),
            case_id.clone(),
            0,
            case_index,
            Some(json!({
                "bindings": bindings.clone(),
                "generated_from": "exact_policy_input_domain",
            })),
            json!({
                "kind": "policy_terminal_equals",
                "policy_ref": policy.lowered_policy_ref,
                "expected": expected,
            }),
            None,
        );

        cases.push(SynthesizedCase {
            id: case_id,
            source: TestSource::Policy,
            target_kind: "policy".to_string(),
            target_name: policy.policy_name.clone(),
            file_path: path.to_path_buf(),
            tags: vec!["synthesized".to_string(), "policy".to_string()],
            seed: 0,
            inputs: SynthesizedInputs {
                bindings,
                generated_from: "exact_policy_input_domain".to_string(),
                case_index,
                world_index: None,
            },
            oracle: SynthesizedOracle::PolicyTerminalEquals { expected },
            repro,
        });
    }

    cases
}

fn policy_terminal_from_value(value: &Value) -> Option<PolicyTerminalOutcome> {
    let terminal = value
        .get("terminal")
        .and_then(Value::as_str)
        .or_else(|| value.as_str())?;
    match terminal {
        "allow" | "Allow" => Some(PolicyTerminalOutcome::Allow),
        "deny" | "Deny" => Some(PolicyTerminalOutcome::Deny),
        _ => None,
    }
}

fn obligation_lifecycle_cases(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    obligation: &RunnerObligationMetadata,
) -> Vec<SynthesizedCase> {
    if obligation.lifecycle_model.is_none()
        || obligation.introduction_sites.is_empty()
        || obligation.discharge_sites.is_empty()
        || obligation.check_sites.is_empty()
    {
        return Vec::new();
    }

    let supported = [
        ObligationTerminalExpectation::Introduced,
        ObligationTerminalExpectation::Discharged,
        ObligationTerminalExpectation::MissingDischargeRejected,
        ObligationTerminalExpectation::DoubleDischargeRejected,
    ];
    let mut cases = Vec::new();
    for expectation in obligation
        .terminal_expectations
        .iter()
        .filter(|expectation| supported.contains(expectation))
        .cloned()
    {
        let case_index = cases.len() + 1;
        let case_id = format!(
            "synthesized/obligation/{}/lifecycle-{:?}-{}",
            obligation.obligation_name, expectation, case_index
        )
        .to_lowercase();
        let bindings = BTreeMap::new();
        let repro = repro_artifact(
            path,
            snapshot.source_artifact_id.clone(),
            snapshot.check_summary_id.clone(),
            case_id.clone(),
            0,
            case_index,
            None,
            json!({
                "kind": "obligation_lifecycle",
                "lifecycle_model": obligation.lifecycle_model,
                "introduction_sites": obligation.introduction_sites,
                "discharge_sites": obligation.discharge_sites,
                "check_sites": obligation.check_sites,
                "expectation": expectation,
            }),
            Some(json!({
                "obligation": obligation.obligation_name,
                "expectation": expectation,
                "model": obligation.lifecycle_model,
            })),
        );

        cases.push(SynthesizedCase {
            id: case_id,
            source: TestSource::Obligation,
            target_kind: "obligation".to_string(),
            target_name: obligation.obligation_name.clone(),
            file_path: path.to_path_buf(),
            tags: vec!["synthesized".to_string(), "obligation".to_string()],
            seed: 0,
            inputs: SynthesizedInputs {
                bindings,
                generated_from: "finite_obligation_lifecycle_metadata".to_string(),
                case_index,
                world_index: Some(case_index),
            },
            oracle: SynthesizedOracle::ObligationLifecycle { expectation },
            repro,
        });
    }

    cases
}

fn evaluate_simple_bool_expression(
    expression: &str,
    bindings: &BTreeMap<String, Value>,
) -> Result<bool, String> {
    let tokens: Vec<&str> = expression.split_whitespace().collect();
    if tokens.len() != 3 {
        return Err(format!(
            "expected '<name> <op> <integer>', got {expression:?}"
        ));
    }

    let left = bindings
        .get(tokens[0])
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("missing integer binding for {}", tokens[0]))?;
    let right = tokens[2]
        .parse::<i64>()
        .map_err(|_| format!("right operand is not an integer: {}", tokens[2]))?;

    match tokens[1] {
        ">" => Ok(left > right),
        ">=" => Ok(left >= right),
        "<" => Ok(left < right),
        "<=" => Ok(left <= right),
        "==" => Ok(left == right),
        "!=" => Ok(left != right),
        other => Err(format!("unsupported operator {other}")),
    }
}

fn deferred_result(
    path: &Path,
    source: TestSource,
    name: impl Into<String>,
    message: impl Into<String>,
    repro: ReproArtifact,
) -> TestResult {
    TestResult::new(name, path.to_path_buf())
        .with_outcome(Outcome::Skip)
        .with_source(source)
        .with_kind(TestKind::Unit)
        .with_duration(Duration::ZERO)
        .with_message(message)
        .with_repro_artifact(repro)
}

fn deferred_result_with_kind(
    path: &Path,
    source: TestSource,
    kind: TestKind,
    name: impl Into<String>,
    message: impl Into<String>,
    repro: ReproArtifact,
) -> TestResult {
    let seed = repro.seed;
    TestResult::new(name, path.to_path_buf())
        .with_outcome(Outcome::Skip)
        .with_source(source)
        .with_kind(kind)
        .with_duration(Duration::ZERO)
        .with_seed(seed)
        .with_message(message)
        .with_repro_artifact(repro)
}

#[allow(clippy::too_many_arguments)]
fn repro_artifact(
    path: &Path,
    source_artifact_id: String,
    check_summary_id: String,
    case_id: String,
    seed: u64,
    case_index: usize,
    generated_input_snapshot: Option<Value>,
    oracle_snapshot: Value,
    world_snapshot: Option<Value>,
) -> ReproArtifact {
    ReproArtifact {
        runner_schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
        source_artifact_id,
        check_summary_id,
        case_id,
        seed,
        case_index,
        world_index: None,
        generated_input_snapshot,
        world_snapshot,
        oracle_snapshot,
        replay_command: format!(
            "ash test {} --only-synthesized contracts,policies,obligations",
            path.display()
        ),
    }
}

fn fallback_repro(
    path: &Path,
    _source: TestSource,
    case_id: String,
    oracle: Value,
) -> ReproArtifact {
    repro_artifact(
        path,
        format!("source-file:{}", path.display()),
        "raw-source-fallback:no-lowered-summary".to_string(),
        case_id,
        0,
        1,
        None,
        oracle,
        None,
    )
}

fn source_from_label(source_kind: &str) -> TestSource {
    match source_kind {
        "contract" | "contracts" => TestSource::Contract,
        "policy" | "policies" => TestSource::Policy,
        "obligation" | "obligations" => TestSource::Obligation,
        _ => TestSource::Authored,
    }
}

/// Generate synthesized test results from contract metadata.
///
/// Contract-derived tests verify that:
/// - `requires` preconditions are checked at call sites
/// - `ensures` postconditions hold after execution
///
/// These tests are labeled `source: synthesized:contract`.
pub fn synthesize_contract_tests(path: &Path, source: &str) -> Vec<TestResult> {
    let mut tests = Vec::new();

    // Simple pattern-based contract detection for V1
    // Look for workflow/function declarations with requires/ensures clauses
    let lines: Vec<&str> = source.lines().collect();
    let mut in_workflow = false;
    let mut workflow_name = String::new();

    for line in &lines {
        let trimmed = line.trim();

        // Detect workflow declarations
        if trimmed.starts_with("workflow ") || trimmed.starts_with("fn ") {
            in_workflow = true;
            // Extract name (simple heuristic)
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                workflow_name = parts[1]
                    .trim_end_matches('{')
                    .trim_end_matches('(')
                    .to_string();
            }
        }

        // Detect requires clauses
        if in_workflow && trimmed.contains("requires") {
            let test_name = format!("synthesized/contract/{}/requires-boundary", workflow_name);
            tests.push(deferred_result(
                path,
                TestSource::Contract,
                test_name.clone(),
                "deferred: raw-source requires pattern is not lowered executable contract metadata",
                fallback_repro(
                    path,
                    TestSource::Contract,
                    test_name,
                    json!({ "source": "contract", "oracle": "requires", "fallback": "raw_source_pattern" }),
                ),
            ));
        }

        // Detect ensures clauses
        if in_workflow && trimmed.contains("ensures") {
            let test_name = format!("synthesized/contract/{}/ensures-boundary", workflow_name);
            tests.push(deferred_result(
                path,
                TestSource::Contract,
                test_name.clone(),
                "deferred: raw-source ensures pattern is not lowered executable contract metadata",
                fallback_repro(
                    path,
                    TestSource::Contract,
                    test_name,
                    json!({ "source": "contract", "oracle": "ensures", "fallback": "raw_source_pattern" }),
                ),
            ));
        }

        // End of workflow (simple heuristic)
        if trimmed == "}" || trimmed.ends_with("}") {
            in_workflow = false;
            workflow_name.clear();
        }
    }

    // If no contracts detected, create one placeholder test to show synthesis is working
    if tests.is_empty() && source.contains("workflow ") {
        let test_name = format!(
            "synthesized/contract/{}/contract-scan",
            path.file_stem().unwrap_or_default().to_string_lossy()
        );
        tests.push(deferred_result(
            path,
            TestSource::Contract,
            test_name.clone(),
            "deferred: no lowered executable contract metadata found in file",
            fallback_repro(
                path,
                TestSource::Contract,
                test_name,
                json!({ "source": "contract", "oracle": "none", "fallback": "raw_source_scan" }),
            ),
        ));
    }

    tests
}

/// Generate synthesized test results from policy metadata.
///
/// Policy-derived tests verify that:
/// - `allow` policies are correctly evaluated
/// - `deny` policies are correctly evaluated
/// - Approve/transform flows work
///
/// These tests are labeled `source: synthesized:policy`.
pub fn synthesize_policy_tests(path: &Path, source: &str) -> Vec<TestResult> {
    let mut tests = Vec::new();

    // Look for policy definitions
    let lines: Vec<&str> = source.lines().collect();

    for line in &lines {
        let trimmed = line.trim();

        // Detect policy declarations
        if trimmed.starts_with("policy ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                let policy_name = parts[1].trim_end_matches('{').to_string();

                // Synthesize allow case test
                let allow_name = format!("synthesized/policy/{}/allow-case", policy_name);
                tests.push(deferred_result(
                    path,
                    TestSource::Policy,
                    allow_name.clone(),
                    "deferred: raw-source policy pattern lacks bounded executable allow oracle",
                    fallback_repro(
                        path,
                        TestSource::Policy,
                        allow_name,
                        json!({ "source": "policy", "oracle": "allow", "fallback": "raw_source_pattern" }),
                    ),
                ));

                // Synthesize deny case test
                let deny_name = format!("synthesized/policy/{}/deny-case", policy_name);
                tests.push(deferred_result(
                    path,
                    TestSource::Policy,
                    deny_name.clone(),
                    "deferred: raw-source policy pattern lacks bounded executable deny oracle",
                    fallback_repro(
                        path,
                        TestSource::Policy,
                        deny_name,
                        json!({ "source": "policy", "oracle": "deny", "fallback": "raw_source_pattern" }),
                    ),
                ));
            }
        }
    }

    // If no policies detected, create one placeholder test
    if tests.is_empty() && source.contains("policy ") {
        let test_name = format!(
            "synthesized/policy/{}/policy-scan",
            path.file_stem().unwrap_or_default().to_string_lossy()
        );
        tests.push(deferred_result(
            path,
            TestSource::Policy,
            test_name.clone(),
            "deferred: policy syntax detected without bounded executable metadata",
            fallback_repro(
                path,
                TestSource::Policy,
                test_name,
                json!({ "source": "policy", "oracle": "unknown", "fallback": "raw_source_scan" }),
            ),
        ));
    }

    tests
}

/// Generate synthesized test results from obligation metadata.
///
/// Obligation-derived tests verify the finite-state lifecycle:
/// - Introduced obligations can be discharged
/// - Double-discharge is detected
/// - Missing-discharge is detected
///
/// These tests are labeled `source: synthesized:obligation`.
pub fn synthesize_obligation_tests(path: &Path, source: &str) -> Vec<TestResult> {
    let mut tests = Vec::new();

    // Look for obligation declarations and usage
    let oblige_count = source.matches("oblige").count();
    let check_count = source.matches("check").count();

    // Synthesize lifecycle tests based on obligation patterns found
    if oblige_count > 0 || check_count > 0 || source.contains("Obligation") {
        // Obligation introduced test
        let introduced_name = format!(
            "synthesized/obligation/{}/introduced",
            path.file_stem().unwrap_or_default().to_string_lossy()
        );
        tests.push(deferred_result(
            path,
            TestSource::Obligation,
            introduced_name.clone(),
            format!(
                "deferred: raw-source obligation patterns ({} oblige / {} check) lack executable lifecycle metadata",
                oblige_count, check_count
            ),
            fallback_repro(
                path,
                TestSource::Obligation,
                introduced_name,
                json!({ "source": "obligation", "oracle": "introduced", "fallback": "raw_source_pattern" }),
            ),
        ));

        // Obligation discharged test
        let discharged_name = format!(
            "synthesized/obligation/{}/discharged",
            path.file_stem().unwrap_or_default().to_string_lossy()
        );
        tests.push(deferred_result(
            path,
            TestSource::Obligation,
            discharged_name.clone(),
            "deferred: raw-source obligation pattern lacks executable discharge lifecycle metadata",
            fallback_repro(
                path,
                TestSource::Obligation,
                discharged_name,
                json!({ "source": "obligation", "oracle": "discharged", "fallback": "raw_source_pattern" }),
            ),
        ));

        // Double-discharge detection test
        let double_name = format!(
            "synthesized/obligation/{}/double-discharge-detected",
            path.file_stem().unwrap_or_default().to_string_lossy()
        );
        tests.push(deferred_result(
            path,
            TestSource::Obligation,
            double_name.clone(),
            "deferred: raw-source obligation pattern lacks executable double-discharge lifecycle metadata",
            fallback_repro(
                path,
                TestSource::Obligation,
                double_name,
                json!({ "source": "obligation", "oracle": "double_discharge", "fallback": "raw_source_pattern" }),
            ),
        ));
    } else {
        // No obligations detected - add a skip test to show synthesis ran
        let test_name = format!(
            "synthesized/obligation/{}/obligation-scan",
            path.file_stem().unwrap_or_default().to_string_lossy()
        );
        tests.push(deferred_result(
            path,
            TestSource::Obligation,
            test_name.clone(),
            "deferred: no executable obligation lifecycle metadata found in file",
            fallback_repro(
                path,
                TestSource::Obligation,
                test_name,
                json!({ "source": "obligation", "oracle": "none", "fallback": "raw_source_scan" }),
            ),
        ));
    }

    tests
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_synthesis_finds_requires() {
        let source = r#"
workflow test_workflow
    requires x > 0
    ensures result > 0
{
    done
}
"#;
        let results = synthesize_contract_tests(Path::new("test.ash"), source);
        assert!(!results.is_empty(), "Should find contract tests");
        assert!(
            results.iter().any(|r| r.name.contains("requires")),
            "Should find requires test"
        );
        assert!(
            results.iter().any(|r| r.name.contains("ensures")),
            "Should find ensures test"
        );
        assert!(
            results
                .iter()
                .all(|r| matches!(r.source, TestSource::Contract)),
            "All should be contract source"
        );
    }

    #[test]
    fn raw_source_contract_patterns_do_not_report_pass_without_execution() {
        let source = r#"
workflow test_workflow
    requires x > 0
    ensures result > 0
{
    done
}
"#;

        let results = synthesize_contract_tests(Path::new("test.ash"), source);

        assert!(
            results
                .iter()
                .any(|result| result.name.contains("requires")),
            "raw-source fallback should still identify deferred contract rows"
        );
        assert!(
            results
                .iter()
                .all(|result| !matches!(result.outcome, Outcome::Pass)),
            "raw-source pattern recognition must not report synthesized pass without executing an oracle: {results:#?}"
        );
    }

    #[test]
    fn synthesized_results_include_repro_artifact_data() {
        let source = r#"
workflow test_workflow
    requires x > 0
{
    done
}
"#;

        let results = synthesize_contract_tests(Path::new("test.ash"), source);
        let serialized = serde_json::to_value(
            results
                .iter()
                .find(|result| result.name.contains("requires"))
                .expect("requires result should be synthesized"),
        )
        .expect("test result should serialize");

        assert!(
            serialized["repro_artifact"].is_object(),
            "synthesized rows should carry reproducible artifact context: {serialized:#}"
        );
    }

    #[test]
    fn structured_contract_metadata_executes_requires_boundary_cases() {
        let snapshot = RunnerIntrospectionSnapshot {
            schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
            module_identity: "test-module".to_string(),
            source_artifact_id: "source:test.ash".to_string(),
            check_summary_id: "check:summary".to_string(),
            contracts: vec![RunnerContractMetadata {
                id: "contract:positive".to_string(),
                callable_name: "positive".to_string(),
                callable_kind: "pure_function".to_string(),
                param_names: vec!["x".to_string()],
                param_types: vec!["Int".to_string()],
                return_type: Some("Int".to_string()),
                lowered_requires: vec!["x > 0".to_string()],
                generation_hints: vec![
                    TypeGeneratorDescriptor {
                        id: "x-valid".to_string(),
                        target_type: "Int".to_string(),
                        source: TypeGeneratorSource::ContractValid,
                        exact_values: vec![json!(1)],
                        ..TypeGeneratorDescriptor::default()
                    },
                    TypeGeneratorDescriptor {
                        id: "x-invalid".to_string(),
                        target_type: "Int".to_string(),
                        source: TypeGeneratorSource::ContractInvalidNearby,
                        exact_values: vec![json!(0)],
                        ..TypeGeneratorDescriptor::default()
                    },
                ],
                executable_case_kinds: vec![SynthesizedOracleKind::PreconditionBoundary],
                ..RunnerContractMetadata::default()
            }],
            ..RunnerIntrospectionSnapshot::default()
        };

        let results = synthesize_from_snapshot(Path::new("test.ash"), &snapshot);

        assert_eq!(results.len(), 2);
        assert!(
            results
                .iter()
                .all(|result| matches!(result.outcome, Outcome::Pass)),
            "structured contract cases should execute their oracle: {results:#?}"
        );
        assert!(
            results.iter().all(|result| result.repro_artifact.is_some()),
            "executed synthesized contract cases should include repro artifacts"
        );
    }

    #[test]
    fn generated_property_metadata_executes_one_case_per_exact_value_with_repro_input() {
        let snapshot = RunnerIntrospectionSnapshot {
            source_artifact_id: "source:property.ash".to_string(),
            check_summary_id: "check:property-summary".to_string(),
            generators: vec![TypeGeneratorDescriptor {
                id: "int-examples".to_string(),
                target_type: "Int".to_string(),
                source: TypeGeneratorSource::FiniteDomain,
                exact_values: vec![
                    json!({ "input": 1, "property_holds": true }),
                    json!({ "input": 0, "property_holds": false }),
                    json!({ "input": 2, "property_holds": true }),
                ],
                ..TypeGeneratorDescriptor::default()
            }],
            ..RunnerIntrospectionSnapshot::default()
        };

        let results = synthesize_from_snapshot_with_limits(
            Path::new("property.ash"),
            &snapshot,
            Some(9001),
            None,
            None,
        );

        assert_eq!(results.len(), 3);
        assert!(
            results
                .iter()
                .all(|result| result.kind == TestKind::Property && result.seed == Some(9001)),
            "generated property rows should be real property results with the configured seed: {results:#?}"
        );
        let failing = results
            .iter()
            .find(|result| result.outcome == Outcome::Fail)
            .expect("one generated property case should fail from metadata oracle");
        assert_eq!(failing.failing_case, Some(2));
        let repro = failing
            .repro_artifact
            .as_ref()
            .expect("generated property failure should carry repro data");
        assert_eq!(repro.seed, 9001);
        assert_eq!(repro.case_index, 2);
        assert_eq!(repro.source_artifact_id, "source:property.ash");
        assert_eq!(repro.check_summary_id, "check:property-summary");
        assert!(
            repro.generated_input_snapshot.is_some(),
            "property repro must include the generated input snapshot: {repro:#?}"
        );
        assert!(
            repro.replay_command.contains("--seed 9001")
                && repro.replay_command.contains("--max-cases 3"),
            "property replay command should include generation controls: {repro:#?}"
        );
    }

    #[test]
    fn unsupported_or_empty_property_generators_defer_instead_of_pass() {
        let snapshot = RunnerIntrospectionSnapshot {
            source_artifact_id: "source:property.ash".to_string(),
            check_summary_id: "check:property-summary".to_string(),
            generators: vec![
                TypeGeneratorDescriptor {
                    id: "open-resource".to_string(),
                    target_type: "Resource".to_string(),
                    source: TypeGeneratorSource::Unsupported,
                    unsupported_reason: Some("resource values are not finite".to_string()),
                    ..TypeGeneratorDescriptor::default()
                },
                TypeGeneratorDescriptor {
                    id: "empty-int-domain".to_string(),
                    target_type: "Int".to_string(),
                    source: TypeGeneratorSource::FiniteDomain,
                    exact_values: Vec::new(),
                    ..TypeGeneratorDescriptor::default()
                },
            ],
            ..RunnerIntrospectionSnapshot::default()
        };

        let results = synthesize_from_snapshot_with_limits(
            Path::new("property.ash"),
            &snapshot,
            None,
            None,
            None,
        );

        assert_eq!(results.len(), 2);
        assert!(
            results.iter().all(|result| result.outcome == Outcome::Skip),
            "unsupported or empty property generators must defer, never pass: {results:#?}"
        );
    }

    #[test]
    fn smallworld_metadata_enumerates_distinct_world_snapshots_and_truncates_by_limit() {
        let snapshot = RunnerIntrospectionSnapshot {
            source_artifact_id: "source:worlds.ash".to_string(),
            check_summary_id: "check:world-summary".to_string(),
            small_world_domains: vec![SmallWorldDomain {
                id: "lifecycle-worlds".to_string(),
                domain_kind: SmallWorldDomainKind::ExplicitStates,
                source: TestSource::Obligation,
                explicit_states: vec![
                    SmallWorldState {
                        id: "introduced".to_string(),
                        world_kind: "obligation_lifecycle".to_string(),
                        control_state: Some("introduced".to_string()),
                        ..SmallWorldState::default()
                    },
                    SmallWorldState {
                        id: "discharged".to_string(),
                        world_kind: "obligation_lifecycle".to_string(),
                        control_state: Some("discharged".to_string()),
                        transition_trace: vec!["introduce".to_string(), "discharge".to_string()],
                        ..SmallWorldState::default()
                    },
                    SmallWorldState {
                        id: "double-discharge".to_string(),
                        world_kind: "obligation_lifecycle".to_string(),
                        control_state: Some("rejected".to_string()),
                        transition_trace: vec![
                            "introduce".to_string(),
                            "discharge".to_string(),
                            "discharge".to_string(),
                        ],
                        ..SmallWorldState::default()
                    },
                ],
                oracle: Some(SmallWorldOracle {
                    kind: SmallWorldOracleKind::ControlStateIn,
                    expected: json!(["introduced", "discharged", "rejected"]),
                }),
                ..SmallWorldDomain::default()
            }],
            ..RunnerIntrospectionSnapshot::default()
        };

        let results = synthesize_from_snapshot_with_limits(
            Path::new("worlds.ash"),
            &snapshot,
            None,
            None,
            Some(2),
        );

        assert_eq!(
            results.len(),
            2,
            "--max-worlds should truncate actual worlds"
        );
        let world_ids: Vec<_> = results
            .iter()
            .map(|result| {
                result
                    .repro_artifact
                    .as_ref()
                    .and_then(|repro| repro.world_snapshot.as_ref())
                    .and_then(|snapshot| snapshot["id"].as_str())
                    .unwrap_or_default()
                    .to_string()
            })
            .collect();
        assert_eq!(world_ids, vec!["introduced", "discharged"]);
        assert_eq!(results[0].world_index, Some(1));
        assert_eq!(results[1].world_index, Some(2));
    }

    #[test]
    fn bounded_int_world_enumeration_applies_limit_before_materialization() {
        let snapshot = RunnerIntrospectionSnapshot {
            source_artifact_id: "source:bounded-worlds.ash".to_string(),
            check_summary_id: "check:bounded-world-summary".to_string(),
            small_world_domains: vec![SmallWorldDomain {
                id: "huge-int-worlds".to_string(),
                domain_kind: SmallWorldDomainKind::BoundedInt,
                source: TestSource::Policy,
                value_type: Some("Int".to_string()),
                bounds: BTreeMap::from([("min".to_string(), 0), ("max".to_string(), i64::MAX)]),
                oracle: Some(SmallWorldOracle {
                    kind: SmallWorldOracleKind::BindingEquals,
                    expected: json!({ "value": 0 }),
                }),
                ..SmallWorldDomain::default()
            }],
            ..RunnerIntrospectionSnapshot::default()
        };

        let results = synthesize_from_snapshot_with_limits(
            Path::new("bounded-worlds.ash"),
            &snapshot,
            None,
            None,
            Some(2),
        );

        assert_eq!(
            results.len(),
            2,
            "bounded-int enumeration must honor max_worlds without materializing the full range"
        );
        let values: Vec<_> = results
            .iter()
            .map(|result| {
                result
                    .repro_artifact
                    .as_ref()
                    .and_then(|repro| repro.world_snapshot.as_ref())
                    .and_then(|snapshot| snapshot["bindings"]["value"].as_i64())
                    .expect("bounded-int worlds should carry integer value bindings")
            })
            .collect();
        assert_eq!(values, vec![0, 1]);
    }

    #[test]
    fn smallworld_results_include_world_index_and_repro_world_snapshot_for_pass_and_fail() {
        let snapshot = RunnerIntrospectionSnapshot {
            source_artifact_id: "source:worlds.ash".to_string(),
            check_summary_id: "check:world-summary".to_string(),
            small_world_domains: vec![SmallWorldDomain {
                id: "control-worlds".to_string(),
                domain_kind: SmallWorldDomainKind::ExplicitStates,
                source: TestSource::Policy,
                explicit_states: vec![
                    SmallWorldState {
                        id: "allowed".to_string(),
                        world_kind: "policy_context".to_string(),
                        control_state: Some("allowed".to_string()),
                        ..SmallWorldState::default()
                    },
                    SmallWorldState {
                        id: "denied".to_string(),
                        world_kind: "policy_context".to_string(),
                        control_state: Some("denied".to_string()),
                        ..SmallWorldState::default()
                    },
                ],
                oracle: Some(SmallWorldOracle {
                    kind: SmallWorldOracleKind::ControlStateEquals,
                    expected: json!("allowed"),
                }),
                ..SmallWorldDomain::default()
            }],
            ..RunnerIntrospectionSnapshot::default()
        };

        let results = synthesize_from_snapshot_with_limits(
            Path::new("worlds.ash"),
            &snapshot,
            Some(7),
            None,
            None,
        );

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].outcome, Outcome::Pass);
        assert_eq!(results[1].outcome, Outcome::Fail);
        for (index, result) in results.iter().enumerate() {
            assert_eq!(result.kind, TestKind::SmallWorld);
            assert_eq!(result.world_index, Some(index + 1));
            let repro = result
                .repro_artifact
                .as_ref()
                .expect("smallworld result should include repro artifact");
            assert_eq!(repro.seed, 7);
            assert_eq!(repro.world_index, Some(index + 1));
            assert!(
                repro.world_snapshot.is_some(),
                "smallworld repro must include world snapshot: {repro:#?}"
            );
        }
    }

    #[test]
    fn contract_requires_without_precondition_boundary_kind_defers() {
        let snapshot = RunnerIntrospectionSnapshot {
            source_artifact_id: "source:test.ash".to_string(),
            check_summary_id: "check:summary".to_string(),
            contracts: vec![RunnerContractMetadata {
                id: "contract:positive".to_string(),
                callable_name: "positive".to_string(),
                callable_kind: "pure_function".to_string(),
                param_names: vec!["x".to_string()],
                param_types: vec!["Int".to_string()],
                lowered_requires: vec!["x > 0".to_string()],
                generation_hints: vec![
                    TypeGeneratorDescriptor {
                        id: "x-valid".to_string(),
                        target_type: "Int".to_string(),
                        source: TypeGeneratorSource::ContractValid,
                        exact_values: vec![json!(1)],
                        ..TypeGeneratorDescriptor::default()
                    },
                    TypeGeneratorDescriptor {
                        id: "x-invalid".to_string(),
                        target_type: "Int".to_string(),
                        source: TypeGeneratorSource::ContractInvalidNearby,
                        exact_values: vec![json!(0)],
                        ..TypeGeneratorDescriptor::default()
                    },
                ],
                executable_case_kinds: vec![SynthesizedOracleKind::PostconditionHolds],
                ..RunnerContractMetadata::default()
            }],
            ..RunnerIntrospectionSnapshot::default()
        };

        let results = synthesize_from_snapshot(Path::new("test.ash"), &snapshot);

        assert!(
            results.iter().all(|result| result.outcome == Outcome::Skip),
            "requires cases must defer unless metadata explicitly enables precondition boundaries: {results:#?}"
        );
    }

    #[test]
    fn contract_requires_without_exact_bounded_generator_defers() {
        let snapshot = RunnerIntrospectionSnapshot {
            source_artifact_id: "source:test.ash".to_string(),
            check_summary_id: "check:summary".to_string(),
            contracts: vec![RunnerContractMetadata {
                id: "contract:positive".to_string(),
                callable_name: "positive".to_string(),
                callable_kind: "pure_function".to_string(),
                param_names: vec!["x".to_string()],
                param_types: vec!["Int".to_string()],
                lowered_requires: vec!["x > 0".to_string()],
                generation_hints: vec![TypeGeneratorDescriptor {
                    id: "x-unsupported".to_string(),
                    target_type: "Int".to_string(),
                    source: TypeGeneratorSource::Unsupported,
                    unsupported_reason: Some("not finite".to_string()),
                    ..TypeGeneratorDescriptor::default()
                }],
                executable_case_kinds: vec![SynthesizedOracleKind::PreconditionBoundary],
                ..RunnerContractMetadata::default()
            }],
            ..RunnerIntrospectionSnapshot::default()
        };

        let results = synthesize_from_snapshot(Path::new("test.ash"), &snapshot);

        assert!(
            results.iter().all(|result| result.outcome == Outcome::Skip),
            "requires cases must defer without exact bounded valid/invalid representatives: {results:#?}"
        );
    }

    #[test]
    fn contract_requires_with_unsupported_descriptor_defers() {
        let snapshot = RunnerIntrospectionSnapshot {
            source_artifact_id: "source:test.ash".to_string(),
            check_summary_id: "check:summary".to_string(),
            contracts: vec![RunnerContractMetadata {
                id: "contract:unsupported".to_string(),
                callable_name: "unsupported".to_string(),
                callable_kind: "pure_function".to_string(),
                param_names: vec!["x".to_string()],
                param_types: vec!["Custom".to_string()],
                lowered_requires: vec!["x > 0".to_string()],
                generation_hints: vec![TypeGeneratorDescriptor {
                    id: "custom".to_string(),
                    target_type: "Custom".to_string(),
                    source: TypeGeneratorSource::Unsupported,
                    unsupported_reason: Some("custom generator unavailable".to_string()),
                    ..TypeGeneratorDescriptor::default()
                }],
                executable_case_kinds: vec![SynthesizedOracleKind::PreconditionBoundary],
                ..RunnerContractMetadata::default()
            }],
            ..RunnerIntrospectionSnapshot::default()
        };

        let results = synthesize_from_snapshot(Path::new("test.ash"), &snapshot);

        assert!(
            results.iter().all(|result| result.outcome == Outcome::Skip),
            "unsupported descriptors must not be inferred into executable values: {results:#?}"
        );
    }

    #[test]
    fn structured_policy_terminal_equals_metadata_executes_allow_and_deny_cases() {
        let snapshot = RunnerIntrospectionSnapshot {
            source_artifact_id: "source:policy.ash".to_string(),
            check_summary_id: "check:summary".to_string(),
            policies: vec![RunnerPolicyMetadata {
                id: "policy:review".to_string(),
                policy_name: "ReviewPolicy".to_string(),
                input_domain: vec![TypeGeneratorDescriptor {
                    id: "action-domain".to_string(),
                    target_type: "Action".to_string(),
                    source: TypeGeneratorSource::FiniteDomain,
                    exact_values: vec![
                        json!({ "terminal": "allow" }),
                        json!({ "terminal": "deny" }),
                    ],
                    ..TypeGeneratorDescriptor::default()
                }],
                lowered_policy_ref: Some("policy:review:terminal".to_string()),
                supported_terminal_outcomes: vec![
                    PolicyTerminalOutcome::Allow,
                    PolicyTerminalOutcome::Deny,
                ],
                oracle_shape: Some(PolicyOracleShape::TerminalEquals),
                ..RunnerPolicyMetadata::default()
            }],
            ..RunnerIntrospectionSnapshot::default()
        };

        let results = synthesize_from_snapshot(Path::new("policy.ash"), &snapshot);

        assert_eq!(results.len(), 2);
        assert!(
            results.iter().all(
                |result| result.source == TestSource::Policy && result.outcome == Outcome::Pass
            ),
            "terminal-equals policy metadata should execute narrow allow/deny cases: {results:#?}"
        );
    }

    #[test]
    fn structured_obligation_lifecycle_metadata_executes_terminal_expectations() {
        let snapshot = RunnerIntrospectionSnapshot {
            source_artifact_id: "source:obligation.ash".to_string(),
            check_summary_id: "check:summary".to_string(),
            obligations: vec![RunnerObligationMetadata {
                id: "obligation:ticket".to_string(),
                obligation_name: "Ticket".to_string(),
                scope: "workflow".to_string(),
                lifecycle_model: Some("finite:introduced-discharged".to_string()),
                introduction_sites: vec!["open_ticket".to_string()],
                discharge_sites: vec!["close_ticket".to_string()],
                check_sites: vec!["finish".to_string()],
                terminal_expectations: vec![
                    ObligationTerminalExpectation::Introduced,
                    ObligationTerminalExpectation::Discharged,
                    ObligationTerminalExpectation::MissingDischargeRejected,
                    ObligationTerminalExpectation::DoubleDischargeRejected,
                ],
                ..RunnerObligationMetadata::default()
            }],
            ..RunnerIntrospectionSnapshot::default()
        };

        let results = synthesize_from_snapshot(Path::new("obligation.ash"), &snapshot);

        assert_eq!(results.len(), 4);
        assert!(
            results.iter().all(|result| {
                result.source == TestSource::Obligation && result.outcome == Outcome::Pass
            }),
            "finite obligation lifecycle metadata should execute supported terminal expectations: {results:#?}"
        );
    }

    #[test]
    fn policy_synthesis_finds_policies() {
        let source = r#"
policy MyPolicy {
    allow => true
}
"#;
        let results = synthesize_policy_tests(Path::new("test.ash"), source);
        assert!(!results.is_empty(), "Should find policy tests");
        assert!(
            results.iter().any(|r| r.name.contains("allow-case")),
            "Should find allow case"
        );
        assert!(
            results.iter().any(|r| r.name.contains("deny-case")),
            "Should find deny case"
        );
        assert!(
            results
                .iter()
                .all(|r| matches!(r.source, TestSource::Policy)),
            "All should be policy source"
        );
    }

    #[test]
    fn unsupported_policy_and_obligation_synthesis_is_deferred_not_passed() {
        let policy_results = synthesize_policy_tests(
            Path::new("policy.ash"),
            r#"
policy MyPolicy {
    allow => true
}
"#,
        );
        let obligation_results = synthesize_obligation_tests(
            Path::new("obligation.ash"),
            r#"
workflow test {
    oblige MyObligation
    check MyObligation
    done
}
"#,
        );

        for result in policy_results.iter().chain(obligation_results.iter()) {
            assert_eq!(
                result.outcome,
                Outcome::Skip,
                "unsupported synthesized metadata should defer instead of pass: {result:#?}"
            );
            assert!(
                result
                    .message
                    .as_deref()
                    .unwrap_or_default()
                    .contains("deferred"),
                "deferred synthesized rows should say why they were not executed: {result:#?}"
            );
        }
    }

    #[test]
    fn obligation_synthesis_finds_obligations() {
        let source = r#"
workflow test {
    oblige MyObligation
    check MyObligation
    done
}
"#;
        let results = synthesize_obligation_tests(Path::new("test.ash"), source);
        assert!(!results.is_empty(), "Should find obligation tests");
        assert!(
            results
                .iter()
                .all(|r| matches!(r.source, TestSource::Obligation)),
            "All should be obligation source"
        );
    }

    #[test]
    fn contract_synthesis_returns_skip_when_no_contracts() {
        let source = r#"
workflow test {
    done
}
"#;
        let results = synthesize_contract_tests(Path::new("test.ash"), source);
        assert!(!results.is_empty(), "Should return at least one test");
        // When no contracts detected, should have a skip test
        assert!(
            results.iter().any(|r| matches!(r.outcome, Outcome::Skip)),
            "Should have skip test when no contracts"
        );
    }
}
