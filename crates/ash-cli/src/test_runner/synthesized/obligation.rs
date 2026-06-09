//! Obligation lifecycle synthesized rows.

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::json;

use super::execution::{
    execute_obligation_lifecycle_trace, expected_obligation_lifecycle_terminal,
};
use super::repro::repro_artifact;
use super::{
    ObligationCloseoutBehavior, ObligationLifecycleModelKind, ObligationLifecycleTransitionPlan,
    ObligationTerminalExpectation, RUNNER_SYNTHESIS_SCHEMA_VERSION, RunnerIntrospectionSnapshot,
    RunnerObligationMetadata, SmallWorldState, SynthesizedCase, SynthesizedInputs,
    SynthesizedOracle,
};
use crate::test_runner::types::TestSource;

pub(super) fn obligation_lifecycle_cases(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    obligation: &RunnerObligationMetadata,
) -> Vec<SynthesizedCase> {
    let Some(transition_plan) = &obligation.lifecycle_transition_plan else {
        return Vec::new();
    };
    if obligation.lifecycle_model.is_none()
        || obligation.introduction_sites.is_empty()
        || obligation.discharge_sites.is_empty()
        || obligation.check_sites.is_empty()
        || obligation.required_closeout_behavior.is_none()
    {
        return Vec::new();
    }
    if !obligation_lifecycle_plan_is_supported(obligation, transition_plan) {
        return Vec::new();
    }

    let supported = [
        ObligationTerminalExpectation::Introduced,
        ObligationTerminalExpectation::Discharged,
        ObligationTerminalExpectation::MissingDischargeRejected,
        ObligationTerminalExpectation::DoubleDischargeRejected,
    ];
    let supported_expectation_count = obligation
        .terminal_expectations
        .iter()
        .filter(|expectation| supported.contains(expectation))
        .count();
    if obligation.lifecycle_worlds.len() < supported_expectation_count
        || obligation.lifecycle_transition_traces.len() < supported_expectation_count
    {
        return Vec::new();
    }
    let supported_worlds = obligation
        .terminal_expectations
        .iter()
        .zip(obligation.lifecycle_worlds.iter())
        .filter(|(expectation, _)| supported.contains(expectation))
        .map(|(_, world)| world)
        .collect::<Vec<_>>();
    if supported_worlds
        .iter()
        .any(|world| !obligation_lifecycle_world_is_supported(obligation, world))
    {
        return Vec::new();
    }

    let mut cases = Vec::new();
    for ((expectation, world), transition_trace) in obligation
        .terminal_expectations
        .iter()
        .cloned()
        .zip(obligation.lifecycle_worlds.iter().cloned())
        .zip(obligation.lifecycle_transition_traces.iter().cloned())
        .filter(|((expectation, _), _)| supported.contains(expectation))
    {
        let Some(expected_terminal) = expected_obligation_lifecycle_terminal(&expectation) else {
            continue;
        };
        let actual_execution =
            execute_obligation_lifecycle_trace(transition_plan, &transition_trace);
        let actual_executed_terminal = match &actual_execution {
            Ok(actual_terminal) => json!({
                "control_state": actual_terminal.control_state(),
                "terminal": actual_terminal,
            }),
            Err(reason) => json!({
                "execution_error": reason,
            }),
        };
        let case_index = cases.len() + 1;
        let case_id = format!(
            "synthesized/obligation/{}/lifecycle-{:?}-{}",
            obligation.obligation_name, expectation, case_index
        )
        .to_lowercase();
        let mut bindings = BTreeMap::new();
        if let Some(control_state) = &world.control_state {
            bindings.insert("lifecycle_control_state".to_string(), json!(control_state));
        }
        let world_snapshot =
            serde_json::to_value(&world).expect("obligation lifecycle world should serialize");
        let mut repro = repro_artifact(
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
                "required_closeout_behavior": obligation.required_closeout_behavior,
                "expectation": expectation,
                "execution_substrate": "typed_lifecycle_transition_plan",
                "expected_terminal": expected_terminal,
                "expected_control_state": expected_terminal.control_state(),
                "actual_executed_terminal": actual_executed_terminal,
                "transition_plan": transition_plan,
                "transition_trace": transition_trace,
            }),
            Some(world_snapshot),
        );
        repro.world_index = Some(case_index);

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
                generated_from: "typed_obligation_lifecycle_transition_trace".to_string(),
                case_index,
                world_index: Some(case_index),
            },
            oracle: SynthesizedOracle::ObligationLifecycle {
                expectation,
                transition_plan: transition_plan.clone(),
                transition_trace,
            },
            repro,
        });
    }

    cases
}

fn obligation_lifecycle_plan_is_supported(
    obligation: &RunnerObligationMetadata,
    plan: &ObligationLifecycleTransitionPlan,
) -> bool {
    obligation.lifecycle_model.as_deref() == Some("finite:introduced-discharged")
        && obligation.required_closeout_behavior.as_deref() == Some("reject_if_open")
        && plan.model == ObligationLifecycleModelKind::IntroduceDischargeCheck
        && plan.required_closeout == ObligationCloseoutBehavior::RejectIfOpen
        && !plan.introduction_sites.is_empty()
        && !plan.discharge_sites.is_empty()
        && !plan.check_sites.is_empty()
}

fn obligation_lifecycle_world_is_supported(
    obligation: &RunnerObligationMetadata,
    world: &SmallWorldState,
) -> bool {
    world.schema_version == RUNNER_SYNTHESIS_SCHEMA_VERSION
        && world.world_kind == "obligation_lifecycle"
        && !world.id.is_empty()
        && world
            .obligations
            .iter()
            .any(|name| name == &obligation.obligation_name)
}
