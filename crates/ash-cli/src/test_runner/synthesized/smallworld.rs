//! General small-world synthesized rows and world enumeration.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use serde_json::{Value, json};

use super::eval::evaluate_core_expression;
use super::repro::{deferred_result_with_kind, repro_artifact};
use super::{
    ContractExecutionSetup, ContractTargetBody, ObligationTerminalExpectation,
    RUNNER_SYNTHESIS_SCHEMA_VERSION, RunnerIntrospectionSnapshot, SMALLWORLD_MAX_LIST_LEN,
    SMALLWORLD_MAX_PRODUCT_AXES, SmallWorldDomain, SmallWorldDomainKind,
    SmallWorldExecutableTarget, SmallWorldExecutableTargetKind, SmallWorldListDescriptor,
    SmallWorldOracle, SmallWorldOracleKind, SmallWorldState,
};
use crate::test_runner::types::{Outcome, ReproArtifact, TestKind, TestResult};

pub(super) fn smallworld_results(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    seed: Option<u64>,
    max_worlds: Option<usize>,
) -> Vec<TestResult> {
    let seed = seed.unwrap_or(0);
    let mut results = Vec::new();

    for domain in &snapshot.small_world_domains {
        let limit = max_worlds.or(domain.max_worlds_default);
        if domain_requires_explicit_world_cap(domain) && limit.is_none() {
            results.push(deferred_uncapped_smallworld_domain_result(
                path, snapshot, domain, seed,
            ));
            continue;
        }
        let worlds = enumerate_worlds(domain, limit);
        if domain.unsupported_reason.is_some()
            || domain.domain_kind == SmallWorldDomainKind::Unsupported
            || worlds.is_empty()
            || domain.oracle.is_none()
            || domain.executable_target.is_none()
        {
            results.push(deferred_smallworld_result(path, snapshot, domain, seed));
            continue;
        }

        let oracle = domain
            .oracle
            .as_ref()
            .expect("checked Some above before executing worlds");
        let target = domain
            .executable_target
            .as_ref()
            .expect("checked Some above before executing worlds");
        if !smallworld_target_metadata_is_supported(target)
            || !smallworld_oracle_is_supported_after_target_execution(oracle)
            || !smallworld_worlds_are_supported_for_target(&worlds)
        {
            results.push(deferred_smallworld_result(path, snapshot, domain, seed));
            continue;
        }
        for (index, world) in worlds.iter().enumerate() {
            let world_index = index + 1;
            let case_id = format!("synthesized/smallworld/{}/world-{}", domain.id, world_index);
            let (target_output, execution_error) = match execute_smallworld_target(target, world) {
                Ok(output) => (Some(output), None),
                Err(reason) => (None, Some(reason)),
            };
            let (outcome, message) = match (&target_output, &execution_error) {
                (Some(output), None) => evaluate_smallworld_oracle(world, oracle, output),
                (None, Some(reason)) => (
                    Outcome::Skip,
                    Some(format!(
                        "deferred: unsupported small-world target execution for world {}: {reason}",
                        world.id
                    )),
                ),
                _ => unreachable!("target output and execution error are mutually exclusive"),
            };
            let repro = smallworld_repro_artifact(
                path,
                snapshot,
                domain,
                world,
                oracle,
                target,
                target_output.as_ref(),
                execution_error.as_deref(),
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
    let limit = match (domain.domain_kind.clone(), max_worlds) {
        (kind, None) if domain_kind_requires_explicit_world_cap(&kind) => return Vec::new(),
        (_, Some(limit)) => limit,
        (_, None) => usize::MAX,
    };
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
        SmallWorldDomainKind::Product => product_worlds(domain, limit),
        SmallWorldDomainKind::List => list_worlds(domain, limit),
        SmallWorldDomainKind::RoleCapabilityInclusionSet => inclusion_set_worlds(domain, limit),
        SmallWorldDomainKind::ObligationLifecycle => lifecycle_worlds(domain, limit),
        SmallWorldDomainKind::PolicyContext => policy_context_worlds(domain, limit),
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

fn domain_requires_explicit_world_cap(domain: &SmallWorldDomain) -> bool {
    domain_kind_requires_explicit_world_cap(&domain.domain_kind)
}

fn domain_kind_requires_explicit_world_cap(kind: &SmallWorldDomainKind) -> bool {
    matches!(
        kind,
        SmallWorldDomainKind::BoundedInt
            | SmallWorldDomainKind::Product
            | SmallWorldDomainKind::List
            | SmallWorldDomainKind::RoleCapabilityInclusionSet
            | SmallWorldDomainKind::ObligationLifecycle
            | SmallWorldDomainKind::PolicyContext
    )
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

fn product_worlds(domain: &SmallWorldDomain, limit: usize) -> Vec<SmallWorldState> {
    if limit == 0
        || domain.product_axes.is_empty()
        || domain.product_axes.len() > SMALLWORLD_MAX_PRODUCT_AXES
        || domain
            .product_axes
            .iter()
            .any(|axis| axis.binding.is_empty() || axis.values.is_empty())
    {
        return Vec::new();
    }

    let mut worlds = Vec::new();
    let mut bindings = BTreeMap::new();
    append_product_worlds(domain, limit, 0, &mut bindings, &mut worlds);
    worlds
}

fn append_product_worlds(
    domain: &SmallWorldDomain,
    limit: usize,
    axis_index: usize,
    bindings: &mut BTreeMap<String, Value>,
    worlds: &mut Vec<SmallWorldState>,
) {
    if worlds.len() >= limit {
        return;
    }
    if axis_index == domain.product_axes.len() {
        let world_index = worlds.len() + 1;
        worlds.push(SmallWorldState {
            id: format!("{}:product-{world_index}", domain.id),
            schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
            world_kind: "product_domain".to_string(),
            bindings: bindings.clone(),
            ..SmallWorldState::default()
        });
        return;
    }

    let axis = &domain.product_axes[axis_index];
    for value in &axis.values {
        bindings.insert(axis.binding.clone(), value.clone());
        append_product_worlds(domain, limit, axis_index + 1, bindings, worlds);
        if worlds.len() >= limit {
            break;
        }
    }
    bindings.remove(&axis.binding);
}

fn list_worlds(domain: &SmallWorldDomain, limit: usize) -> Vec<SmallWorldState> {
    let Some(descriptor) = &domain.list_descriptor else {
        return Vec::new();
    };
    let Some(max_len) = descriptor.max_len else {
        return Vec::new();
    };
    if limit == 0
        || descriptor.binding.is_empty()
        || descriptor.elements.is_empty()
        || descriptor.min_len > max_len
        || max_len > SMALLWORLD_MAX_LIST_LEN
    {
        return Vec::new();
    }

    let mut worlds = Vec::new();
    for len in descriptor.min_len..=max_len {
        let mut current = Vec::with_capacity(len);
        append_list_worlds(domain, descriptor, limit, len, &mut current, &mut worlds);
        if worlds.len() >= limit {
            break;
        }
    }
    worlds
}

fn append_list_worlds(
    domain: &SmallWorldDomain,
    descriptor: &SmallWorldListDescriptor,
    limit: usize,
    target_len: usize,
    current: &mut Vec<Value>,
    worlds: &mut Vec<SmallWorldState>,
) {
    if worlds.len() >= limit {
        return;
    }
    if current.len() == target_len {
        let world_index = worlds.len() + 1;
        let mut bindings = BTreeMap::new();
        bindings.insert(descriptor.binding.clone(), Value::Array(current.clone()));
        worlds.push(SmallWorldState {
            id: format!("{}:list-{world_index}", domain.id),
            schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
            world_kind: "list_domain".to_string(),
            bindings,
            ..SmallWorldState::default()
        });
        return;
    }

    for value in &descriptor.elements {
        current.push(value.clone());
        append_list_worlds(domain, descriptor, limit, target_len, current, worlds);
        current.pop();
        if worlds.len() >= limit {
            break;
        }
    }
}

fn inclusion_set_worlds(domain: &SmallWorldDomain, limit: usize) -> Vec<SmallWorldState> {
    let Some(descriptor) = &domain.inclusion_descriptor else {
        return Vec::new();
    };
    let item_count = descriptor.roles.len() + descriptor.capabilities.len();
    if limit == 0 || item_count == 0 || item_count >= usize::BITS as usize {
        return Vec::new();
    }

    let total_sets = 1usize << item_count;
    (0..total_sets)
        .take(limit)
        .enumerate()
        .map(|(index, mask)| {
            let roles = descriptor
                .roles
                .iter()
                .enumerate()
                .filter(|(role_index, _role)| (mask & (1usize << role_index)) != 0)
                .map(|(_role_index, role)| role.clone())
                .collect::<Vec<_>>();
            let role_count = descriptor.roles.len();
            let capabilities = descriptor
                .capabilities
                .iter()
                .enumerate()
                .filter(|(capability_index, _capability)| {
                    let bit_index = role_count + capability_index;
                    (mask & (1usize << bit_index)) != 0
                })
                .map(|(_capability_index, capability)| capability.clone())
                .collect::<Vec<_>>();
            SmallWorldState {
                id: format!("{}:inclusion-{}", domain.id, index + 1),
                schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
                world_kind: "role_capability_inclusion_set".to_string(),
                roles,
                capabilities,
                ..SmallWorldState::default()
            }
        })
        .collect()
}

fn lifecycle_worlds(domain: &SmallWorldDomain, limit: usize) -> Vec<SmallWorldState> {
    let Some(descriptor) = &domain.lifecycle_descriptor else {
        return Vec::new();
    };
    if limit == 0
        || descriptor.obligation.is_empty()
        || descriptor.states.is_empty()
        || descriptor.states.iter().any(|state| state.id.is_empty())
    {
        return Vec::new();
    }

    descriptor
        .states
        .iter()
        .take(limit)
        .map(|state| SmallWorldState {
            id: state.id.clone(),
            schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
            world_kind: "obligation_lifecycle".to_string(),
            obligations: vec![descriptor.obligation.clone()],
            control_state: Some(lifecycle_control_state(&state.terminal).to_string()),
            transition_trace: state.transition_trace.clone(),
            ..SmallWorldState::default()
        })
        .collect()
}

fn lifecycle_control_state(terminal: &ObligationTerminalExpectation) -> &'static str {
    match terminal {
        ObligationTerminalExpectation::Introduced => "introduced",
        ObligationTerminalExpectation::Discharged => "discharged",
        ObligationTerminalExpectation::MissingDischargeRejected => "missing_discharge_rejected",
        ObligationTerminalExpectation::DoubleDischargeRejected => "double_discharge_rejected",
        ObligationTerminalExpectation::Unsupported => "unsupported",
    }
}

fn policy_context_worlds(domain: &SmallWorldDomain, limit: usize) -> Vec<SmallWorldState> {
    let Some(descriptor) = &domain.policy_context_descriptor else {
        return Vec::new();
    };
    if limit == 0
        || descriptor.policies.is_empty()
        || descriptor.contexts.is_empty()
        || descriptor
            .contexts
            .iter()
            .any(|context| context.id.is_empty())
    {
        return Vec::new();
    }

    descriptor
        .contexts
        .iter()
        .take(limit)
        .map(|context| SmallWorldState {
            id: context.id.clone(),
            schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
            world_kind: "policy_context".to_string(),
            bindings: context.bindings.clone(),
            capabilities: context.capabilities.clone(),
            roles: context.roles.clone(),
            policies: descriptor.policies.clone(),
            control_state: context.control_state.clone(),
            ..SmallWorldState::default()
        })
        .collect()
}

fn evaluate_smallworld_oracle(
    world: &SmallWorldState,
    oracle: &SmallWorldOracle,
    target_output: &Value,
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
        SmallWorldOracleKind::TargetOutputEquals => target_output == &oracle.expected,
    };

    if passed {
        (Outcome::Pass, None)
    } else {
        (
            Outcome::Fail,
            Some(format!(
                "small-world oracle failed for world {} with target output {}",
                world.id, target_output
            )),
        )
    }
}

fn smallworld_target_metadata_is_supported(target: &SmallWorldExecutableTarget) -> bool {
    matches!(target.kind, SmallWorldExecutableTargetKind::PureExpression)
        && matches!(target.setup, ContractExecutionSetup::PureNoSetup)
        && !matches!(target.body, ContractTargetBody::Unsupported)
        && !target.target_ref.is_empty()
}

fn smallworld_oracle_is_supported_after_target_execution(oracle: &SmallWorldOracle) -> bool {
    matches!(oracle.kind, SmallWorldOracleKind::TargetOutputEquals)
}

fn smallworld_worlds_are_supported_for_target(worlds: &[SmallWorldState]) -> bool {
    worlds
        .iter()
        .all(|world| world.mailbox.is_empty() && world.resource_state.is_empty())
}

fn execute_smallworld_target(
    target: &SmallWorldExecutableTarget,
    world: &SmallWorldState,
) -> Result<Value, String> {
    if !smallworld_target_metadata_is_supported(target) {
        return Err("small-world executable target metadata is unsupported".to_string());
    }
    match &target.body {
        ContractTargetBody::ReturnExpression { expression } => {
            evaluate_core_expression(expression, &world.bindings, None)
        }
        ContractTargetBody::ReturnLiteral { value } => Ok(value.clone()),
        ContractTargetBody::Unsupported => {
            Err("small-world target body is not executable".to_string())
        }
    }
}

fn deferred_smallworld_result(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    domain: &SmallWorldDomain,
    seed: u64,
) -> TestResult {
    let reason = smallworld_deferred_reason(domain);
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

fn smallworld_deferred_reason(domain: &SmallWorldDomain) -> String {
    if let Some(reason) = &domain.unsupported_reason {
        return reason.clone();
    }
    if domain.domain_kind == SmallWorldDomainKind::Unsupported {
        return "domain is not an explicit supported finite world model".to_string();
    }
    if domain.oracle.is_none() {
        return "small-world domain lacks executable oracle metadata".to_string();
    }
    let Some(target) = &domain.executable_target else {
        return "small-world domain lacks executable target metadata".to_string();
    };
    if !smallworld_target_metadata_is_supported(target) {
        return "small-world executable target metadata is unsupported".to_string();
    }
    if let Some(oracle) = &domain.oracle
        && !smallworld_oracle_is_supported_after_target_execution(oracle)
    {
        return "small-world oracle is not executable target-output metadata".to_string();
    }
    match domain.domain_kind {
        SmallWorldDomainKind::Product => {
            "bounded product domain lacks non-empty explicit finite axes".to_string()
        }
        SmallWorldDomainKind::List => {
            "bounded list domain lacks explicit finite elements or max_len".to_string()
        }
        SmallWorldDomainKind::RoleCapabilityInclusionSet => {
            "role/capability inclusion-set domain lacks explicit finite roles or capabilities"
                .to_string()
        }
        SmallWorldDomainKind::ObligationLifecycle => {
            "obligation lifecycle domain lacks stable finite state-machine descriptor".to_string()
        }
        SmallWorldDomainKind::PolicyContext => {
            "policy-context domain lacks stable finite context descriptor".to_string()
        }
        _ => "small-world domain lacks supported finite worlds for target execution".to_string(),
    }
}

fn deferred_uncapped_smallworld_domain_result(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    domain: &SmallWorldDomain,
    seed: u64,
) -> TestResult {
    let case_id = format!("synthesized/smallworld/{}/deferred", domain.id);
    deferred_result_with_kind(
        path,
        domain.source,
        TestKind::SmallWorld,
        case_id,
        "deferred: small-world domain requires explicit max_worlds or metadata max_worlds_default",
        ReproArtifact {
            replay_command: format!(
                "ash test {} --only-synthesized contracts,policies,obligations --seed {} --max-worlds <n>",
                path.display(),
                seed
            ),
            ..repro_artifact(
                path,
                snapshot.source_artifact_id.clone(),
                snapshot.check_summary_id.clone(),
                format!("smallworld:{}:bounded-int-uncapped", domain.id),
                seed,
                1,
                None,
                json!({
                    "kind": "small_world",
                    "supported": false,
                    "reason": "domain requires explicit max_worlds or max_worlds_default before materialization",
                    "domain_kind": domain.domain_kind,
                    "bounds": domain.bounds,
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
    target: &SmallWorldExecutableTarget,
    target_output: Option<&Value>,
    execution_error: Option<&str>,
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
            "target_execution": {
                "substrate": "ash_interp_core_expr",
                "target": target,
                "target_output": target_output,
                "execution_error": execution_error,
            },
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
