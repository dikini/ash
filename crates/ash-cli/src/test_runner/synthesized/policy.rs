//! Policy-derived synthesized rows.

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{Value, json};

use super::execution::evaluate_policy_terminal_oracle;
use super::repro::repro_artifact;
use super::{
    PolicyAuthoritySetup, PolicyExecutableTarget, PolicyExecutableTargetKind, PolicyOracleShape,
    PolicyTerminalOracle, PolicyTerminalOutcome, RunnerIntrospectionSnapshot, RunnerPolicyMetadata,
    SynthesizedCase, SynthesizedInputs, SynthesizedOracle, TypeGeneratorSource,
};
use crate::test_runner::types::TestSource;

pub(super) fn policy_terminal_cases(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    policy: &RunnerPolicyMetadata,
) -> Vec<SynthesizedCase> {
    if policy.oracle_shape != Some(PolicyOracleShape::TerminalEquals)
        || policy.input_domain.is_empty()
    {
        return Vec::new();
    }
    let Some(policy_ref) = policy.lowered_policy_ref.clone() else {
        return Vec::new();
    };
    let Some(target) = &policy.executable_target else {
        return Vec::new();
    };
    if policy_target_metadata_is_supported(policy, target).is_err() {
        return Vec::new();
    }

    let mut cases = Vec::new();
    for expected in [PolicyTerminalOutcome::Allow, PolicyTerminalOutcome::Deny] {
        if !policy.supported_terminal_outcomes.contains(&expected) {
            continue;
        }
        let Some((input, actual)) = exact_policy_input_values(policy).find_map(|input| {
            let mut candidate_bindings = BTreeMap::new();
            candidate_bindings.insert("policy_input".to_string(), input.clone());
            let actual =
                evaluate_policy_terminal_oracle(&target.terminal_oracle, &candidate_bindings)?;
            (actual == expected).then_some((input.clone(), actual))
        }) else {
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
                "policy_ref": policy_ref,
                "target": {
                    "kind": target.kind,
                    "target_ref": target.target_ref,
                    "authority_setup": target.authority_setup,
                    "required_authority": policy.required_authority,
                },
                "target_execution": {
                    "substrate": "finite_policy_terminal_oracle",
                },
                "expected_terminal": expected,
                "actual_terminal": actual,
                "terminal_oracle": target.terminal_oracle,
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
            oracle: SynthesizedOracle::PolicyTerminalEquals {
                expected,
                policy_ref: policy_ref.clone(),
                terminal_oracle: target.terminal_oracle.clone(),
            },
            repro,
        });
    }

    cases
}

fn exact_policy_input_values(policy: &RunnerPolicyMetadata) -> impl Iterator<Item = &Value> {
    policy
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
}

fn policy_target_metadata_is_supported(
    policy: &RunnerPolicyMetadata,
    target: &PolicyExecutableTarget,
) -> Result<(), String> {
    if !matches!(target.kind, PolicyExecutableTargetKind::TerminalOracle) {
        return Err("policy target kind is not a supported terminal oracle".to_string());
    }
    if !matches!(
        target.terminal_oracle,
        PolicyTerminalOracle::ExactMatchTable { .. }
    ) {
        return Err("policy terminal oracle is not a supported exact-match table".to_string());
    }
    let Some(lowered_policy_ref) = policy.lowered_policy_ref.as_deref() else {
        return Err("policy metadata lacks lowered policy reference".to_string());
    };
    if target.target_ref.is_empty() {
        return Err("policy executable target metadata lacks target_ref".to_string());
    }
    if target.target_ref != lowered_policy_ref {
        return Err(format!(
            "policy executable target_ref {:?} does not match lowered policy ref {:?}",
            target.target_ref, lowered_policy_ref
        ));
    }

    match (&policy.required_authority, &target.authority_setup) {
        (Some(required), PolicyAuthoritySetup::ExplicitAuthority { authority })
            if authority == required =>
        {
            Ok(())
        }
        (Some(required), PolicyAuthoritySetup::ExplicitAuthority { authority }) => Err(format!(
            "policy required authority {required:?} does not match explicit authority setup {authority:?}"
        )),
        (Some(_), PolicyAuthoritySetup::NoAuthorityRequired | PolicyAuthoritySetup::Missing) => {
            Err("policy required authority lacks explicit supported authority setup".to_string())
        }
        (Some(_), PolicyAuthoritySetup::Unsupported) => {
            Err("policy required authority setup is unsupported".to_string())
        }
        (None, PolicyAuthoritySetup::NoAuthorityRequired) => Ok(()),
        (None, PolicyAuthoritySetup::ExplicitAuthority { .. }) => Ok(()),
        (None, PolicyAuthoritySetup::Missing) => {
            Err("policy authority setup metadata is missing".to_string())
        }
        (None, PolicyAuthoritySetup::Unsupported) => {
            Err("policy authority setup metadata is unsupported".to_string())
        }
    }
}

pub(super) fn policy_terminal_deferred_reason(policy: &RunnerPolicyMetadata) -> String {
    if policy.oracle_shape != Some(PolicyOracleShape::TerminalEquals) {
        return "policy metadata lacks supported terminal-equals oracle shape".to_string();
    }
    if policy.lowered_policy_ref.is_none() {
        return "policy metadata lacks lowered policy reference".to_string();
    }
    if policy.input_domain.is_empty() {
        return "policy metadata lacks exact bounded input domain".to_string();
    }
    let Some(target) = &policy.executable_target else {
        return "policy metadata lacks executable target/oracle metadata".to_string();
    };
    if let Err(reason) = policy_target_metadata_is_supported(policy, target) {
        return reason;
    }
    if !policy.supported_terminal_outcomes.iter().any(|terminal| {
        matches!(
            terminal,
            PolicyTerminalOutcome::Allow | PolicyTerminalOutcome::Deny
        )
    }) {
        return "policy metadata lacks supported allow/deny terminal outcomes".to_string();
    }
    "policy metadata lacks finite inputs that evaluate to supported allow/deny terminals"
        .to_string()
}
