//! Law-derived small-world synthesized rows.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use serde_json::{Value, json};

use super::eval::evaluate_simple_bool_expression;
use super::repro::{deferred_result_with_kind, repro_artifact};
use super::{LAW_SMALLWORLD_DEFAULT_MAX_WORLDS, RunnerIntrospectionSnapshot, RunnerLawMetadata};
use crate::test_runner::types::{Outcome, TestKind, TestResult, TestSource};

pub(super) fn law_smallworld_results(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    seed: Option<u64>,
    max_worlds: Option<usize>,
) -> Vec<TestResult> {
    let seed = seed.unwrap_or(0);
    let mut results = Vec::new();

    for law in &snapshot.laws {
        let Some(param_domains) = law_param_domains(law) else {
            results.push(deferred_law_result(path, snapshot, law, seed));
            continue;
        };
        let worlds = law_binding_worlds(
            &param_domains,
            max_worlds.unwrap_or(LAW_SMALLWORLD_DEFAULT_MAX_WORLDS),
        );
        if worlds.is_empty() {
            results.push(deferred_law_result(path, snapshot, law, seed));
            continue;
        }

        for (index, bindings) in worlds.into_iter().enumerate() {
            let world_index = index + 1;
            let case_id = format!("synthesized/law/{}/world-{}", law.name, world_index);
            let outcome = match evaluate_simple_bool_expression(&law.proposition, &bindings) {
                Ok(true) => Outcome::Pass,
                Ok(false) => Outcome::Fail,
                Err(_) => Outcome::Skip,
            };
            let message = match outcome {
                Outcome::Pass => format!(
                    "law {} held for generated small-world binding {}",
                    law.name,
                    Value::Object(bindings.clone().into_iter().collect())
                ),
                Outcome::Fail => format!(
                    "law {} counterexample at seed {seed}, world {world_index}: {}",
                    law.name,
                    Value::Object(bindings.clone().into_iter().collect())
                ),
                Outcome::Skip => format!(
                    "deferred: unsupported law proposition {:?} for generated binding {}",
                    law.proposition,
                    Value::Object(bindings.clone().into_iter().collect())
                ),
                _ => unreachable!("law small-world generation only emits pass/fail/skip"),
            };
            let generated_input_snapshot = Value::Object(bindings.clone().into_iter().collect());
            let mut repro = repro_artifact(
                path,
                snapshot.source_artifact_id.clone(),
                snapshot.check_summary_id.clone(),
                format!("law:{}:world-{world_index}", law.id),
                seed,
                world_index,
                Some(generated_input_snapshot.clone()),
                json!({
                    "source": "law",
                    "law": law.name,
                    "delegated_test": law.delegated_test,
                    "proposition": law.proposition,
                    "expected": true,
                    "world_index": world_index,
                }),
                Some(generated_input_snapshot.clone()),
            );
            repro.world_index = Some(world_index);

            let mut result = TestResult::new(&case_id, path.to_path_buf())
                .with_outcome(outcome)
                .with_source(TestSource::Law)
                .with_kind(TestKind::SmallWorld)
                .with_duration(Duration::ZERO)
                .with_seed(seed)
                .with_message(message)
                .with_repro_artifact(repro);
            result.world_index = Some(world_index);
            result.failing_case = outcome.is_failure().then_some(world_index);
            result.tags = vec!["synthesized".to_string(), "law".to_string()];
            results.push(result);
        }
    }

    results
}

fn law_param_domains(law: &RunnerLawMetadata) -> Option<Vec<(String, Vec<Value>)>> {
    law.params
        .iter()
        .map(|param| law_param_domain(param))
        .collect()
}

fn law_param_domain(param: &str) -> Option<(String, Vec<Value>)> {
    let (name, ty) = param.split_once(':')?;
    let name = name.trim().to_string();
    let ty = ty.trim();
    let values = match ty {
        "Int" => vec![json!(-1), json!(0), json!(1)],
        "Bool" => vec![json!(false), json!(true)],
        "String" => vec![json!(""), json!("ash")],
        _ => return None,
    };
    Some((name, values))
}

fn law_binding_worlds(
    param_domains: &[(String, Vec<Value>)],
    limit: usize,
) -> Vec<BTreeMap<String, Value>> {
    if limit == 0 {
        return Vec::new();
    }
    if param_domains.is_empty() {
        return vec![BTreeMap::new()];
    }
    let mut worlds = Vec::new();
    let mut bindings = BTreeMap::new();
    append_law_binding_worlds(param_domains, limit, 0, &mut bindings, &mut worlds);
    worlds
}

fn append_law_binding_worlds(
    param_domains: &[(String, Vec<Value>)],
    limit: usize,
    axis_index: usize,
    bindings: &mut BTreeMap<String, Value>,
    worlds: &mut Vec<BTreeMap<String, Value>>,
) {
    if worlds.len() >= limit {
        return;
    }
    if axis_index == param_domains.len() {
        worlds.push(bindings.clone());
        return;
    }
    let (name, values) = &param_domains[axis_index];
    for value in values {
        if worlds.len() >= limit {
            return;
        }
        bindings.insert(name.clone(), value.clone());
        append_law_binding_worlds(param_domains, limit, axis_index + 1, bindings, worlds);
        bindings.remove(name);
    }
}

fn deferred_law_result(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    law: &RunnerLawMetadata,
    seed: u64,
) -> TestResult {
    let mut repro = repro_artifact(
        path,
        snapshot.source_artifact_id.clone(),
        snapshot.check_summary_id.clone(),
        format!("law:{}:deferred", law.id),
        seed,
        1,
        None,
        json!({
            "source": "law",
            "law": law.name,
            "delegated_test": law.delegated_test,
            "proposition": law.proposition,
            "params": law.params,
        }),
        None,
    );
    repro.replay_command = format!(
        "ash test {} --only-synthesized laws --seed {}",
        path.display(),
        seed
    );

    deferred_result_with_kind(
        path,
        TestSource::Law,
        TestKind::SmallWorld,
        format!("synthesized/law/{}/deferred", law.name),
        "deferred: law metadata lacks supported finite parameter domains or executable proposition",
        repro,
    )
}
