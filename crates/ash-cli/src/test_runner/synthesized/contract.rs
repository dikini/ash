//! Contract-derived synthesized rows.

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{Value, json};

use super::eval::{evaluate_core_expression, evaluate_simple_bool_expression};
use super::repro::{deferred_result, repro_artifact};
use super::{
    ContractExecutableTarget, ContractExecutableTargetKind, ContractExecutionSetup,
    ContractTargetBody, RunnerContractMetadata, RunnerIntrospectionSnapshot, SynthesizedCase,
    SynthesizedInputs, SynthesizedOracle, SynthesizedOracleKind, TypeGeneratorDescriptor,
    TypeGeneratorSource,
};
use crate::test_runner::types::{TestResult, TestSource};

pub(super) fn contract_requires_cases(
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

pub(super) fn contract_postcondition_cases(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    contract: &RunnerContractMetadata,
) -> Vec<SynthesizedCase> {
    if !contract
        .executable_case_kinds
        .contains(&SynthesizedOracleKind::PostconditionHolds)
    {
        return Vec::new();
    }

    let Some(target) = &contract.executable_target else {
        return Vec::new();
    };
    if !contract_target_metadata_is_supported(target) {
        return Vec::new();
    }

    let Some(bindings) = exact_contract_valid_bindings(snapshot, contract) else {
        return Vec::new();
    };
    if !contract_requires_accept_inputs(&contract.lowered_requires, &bindings) {
        return Vec::new();
    }
    let Ok(target_output) = execute_contract_target(target, &bindings) else {
        return Vec::new();
    };

    contract
        .executable_postconditions
        .iter()
        .enumerate()
        .map(|(index, postcondition)| {
            let case_index = index + 1;
            let case_id = format!(
                "synthesized/contract/{}/ensures-{}",
                contract.callable_name, case_index
            );
            let input_snapshot = json!({
                "bindings": bindings.clone(),
                "generated_from": "exact_contract_valid_descriptor",
            });
            let oracle_snapshot = json!({
                "kind": "postcondition_holds",
                "ensures": postcondition.display,
                "target": {
                    "kind": target.kind,
                    "target_ref": target.target_ref,
                    "setup": target.setup,
                },
                "target_execution": {
                    "substrate": "ash_interp_core_expr",
                    "representation": "ash_core::Expr",
                },
                "target_output": target_output,
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
            SynthesizedCase {
                id: case_id,
                source: TestSource::Contract,
                target_kind: contract.callable_kind.clone(),
                target_name: contract.callable_name.clone(),
                file_path: path.to_path_buf(),
                tags: vec!["synthesized".to_string(), "contract".to_string()],
                seed: 0,
                inputs: SynthesizedInputs {
                    bindings: bindings.clone(),
                    generated_from: "exact_contract_valid_descriptor".to_string(),
                    case_index,
                    world_index: None,
                },
                oracle: SynthesizedOracle::ContractEnsures {
                    expression: postcondition.display.clone(),
                    oracle: postcondition.expression.clone(),
                    target_output: target_output.clone(),
                },
                repro,
            }
        })
        .collect()
}

fn contract_target_metadata_is_supported(target: &ContractExecutableTarget) -> bool {
    matches!(target.kind, ContractExecutableTargetKind::PureFunction)
        && matches!(target.setup, ContractExecutionSetup::PureNoSetup)
        && !matches!(target.body, ContractTargetBody::Unsupported)
}

fn exact_contract_valid_bindings(
    snapshot: &RunnerIntrospectionSnapshot,
    contract: &RunnerContractMetadata,
) -> Option<BTreeMap<String, Value>> {
    let mut bindings = BTreeMap::new();
    for (param, param_type) in contract.param_names.iter().zip(&contract.param_types) {
        let duplicate_type_count = contract
            .param_types
            .iter()
            .filter(|candidate| *candidate == param_type)
            .count();
        let value = exact_generator_value(
            snapshot,
            contract,
            param,
            param_type,
            duplicate_type_count > 1,
            TypeGeneratorSource::ContractValid,
        )?;
        bindings.insert(param.clone(), value);
    }
    Some(bindings)
}

fn contract_requires_accept_inputs(
    lowered_requires: &[String],
    bindings: &BTreeMap<String, Value>,
) -> bool {
    lowered_requires
        .iter()
        .all(|expression| evaluate_simple_bool_expression(expression, bindings) == Ok(true))
}

fn execute_contract_target(
    target: &ContractExecutableTarget,
    bindings: &BTreeMap<String, Value>,
) -> Result<Value, String> {
    match target.kind {
        ContractExecutableTargetKind::PureFunction => {}
        ContractExecutableTargetKind::ActFunction => {
            return Err("unsupported contract target kind act_function".to_string());
        }
        ContractExecutableTargetKind::RuntimeCallable => {
            return Err("unsupported contract target kind runtime_callable".to_string());
        }
        ContractExecutableTargetKind::Unsupported => {
            return Err("unsupported contract target kind".to_string());
        }
    }

    match target.setup {
        ContractExecutionSetup::PureNoSetup => {}
        ContractExecutionSetup::ExplicitFinite => {
            return Err(
                "explicit finite setup is not executable for pure target slice".to_string(),
            );
        }
        ContractExecutionSetup::Missing => {
            return Err("contract target execution setup is missing".to_string());
        }
        ContractExecutionSetup::Unsupported => {
            return Err("contract target execution setup is unsupported".to_string());
        }
    }

    match &target.body {
        ContractTargetBody::ReturnExpression { expression } => {
            evaluate_core_expression(expression, bindings, None)
        }
        ContractTargetBody::ReturnLiteral { value } => Ok(value.clone()),
        ContractTargetBody::Unsupported => {
            Err("contract target body is not executable".to_string())
        }
    }
}

pub(super) fn deferred_contract_postcondition_result(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    contract: &RunnerContractMetadata,
) -> TestResult {
    let reason = contract_postcondition_deferred_reason(snapshot, contract);
    deferred_result(
        path,
        TestSource::Contract,
        format!(
            "synthesized/contract/{}/postcondition-deferred",
            contract.callable_name
        ),
        format!("deferred: {reason}"),
        repro_artifact(
            path,
            snapshot.source_artifact_id.clone(),
            snapshot.check_summary_id.clone(),
            format!("contract:{}:postcondition-deferred", contract.id),
            0,
            1,
            None,
            json!({
                "source": "contract",
                "target": contract.callable_name,
                "oracle": "ensures",
                "reason": reason,
            }),
            None,
        ),
    )
}

fn contract_postcondition_deferred_reason(
    snapshot: &RunnerIntrospectionSnapshot,
    contract: &RunnerContractMetadata,
) -> String {
    if !contract
        .executable_case_kinds
        .contains(&SynthesizedOracleKind::PostconditionHolds)
    {
        return "contract metadata does not enable executable postcondition cases".to_string();
    }
    let Some(target) = &contract.executable_target else {
        return "contract metadata lacks executable postcondition target metadata".to_string();
    };
    if let Err(reason) = execute_contract_target(target, &BTreeMap::new())
        && matches!(
            target.kind,
            ContractExecutableTargetKind::ActFunction
                | ContractExecutableTargetKind::RuntimeCallable
                | ContractExecutableTargetKind::Unsupported
        )
    {
        return reason;
    }
    if !matches!(target.setup, ContractExecutionSetup::PureNoSetup) {
        return match target.setup {
            ContractExecutionSetup::ExplicitFinite => {
                "explicit finite setup is not executable for pure target slice".to_string()
            }
            ContractExecutionSetup::Missing => {
                "contract target execution setup is missing".to_string()
            }
            ContractExecutionSetup::Unsupported => {
                "contract target execution setup is unsupported".to_string()
            }
            ContractExecutionSetup::PureNoSetup => unreachable!(),
        };
    }
    if matches!(target.body, ContractTargetBody::Unsupported) {
        return "contract target body is not executable".to_string();
    }
    if exact_contract_valid_bindings(snapshot, contract).is_none() {
        return "contract postcondition oracle lacks exact valid input representatives".to_string();
    }
    "contract postcondition metadata is not executable".to_string()
}

pub(super) fn expression_parameter(expression: &str) -> Option<String> {
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
