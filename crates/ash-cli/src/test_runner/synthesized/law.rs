//! Law-derived small-world synthesized rows.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use serde_json::{Value, json};

use super::eval::evaluate_simple_bool_expression;
use super::repro::{deferred_result_with_kind, repro_artifact};
use super::value_generation::{generated_cases, generated_domain_for_param, shrink_bindings};
use super::{
    LAW_SMALLWORLD_DEFAULT_MAX_WORLDS, LawEvidenceStatus, LawTestEvidence,
    RunnerIntrospectionSnapshot, RunnerLawMetadata,
};
use crate::test_runner::algebra_law_profile::{AlgebraInterface, CarrierType, LawProfile};
use crate::test_runner::types::{Outcome, TestKind, TestResult, TestSource};

const ALGEBRA_LAW_DEFAULT_MAX_CASES: usize = 32;

pub(super) fn algebra_law_profile_results(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    seed: Option<u64>,
    max_cases: Option<usize>,
) -> Vec<TestResult> {
    let seed = seed.unwrap_or(0);
    let max_cases = max_cases.unwrap_or(ALGEBRA_LAW_DEFAULT_MAX_CASES);
    let mut results = Vec::new();

    for law in &snapshot.laws {
        if matches!(law.test_evidence, Some(LawTestEvidence::Authored { .. })) {
            continue;
        }
        let Some(owner) = law.owner.as_deref() else {
            continue;
        };
        let Some(interface) = AlgebraInterface::from_name(owner) else {
            continue;
        };
        if !interface.law_names().contains(&law.name.as_str()) {
            results.push(unknown_algebra_law_result(path, snapshot, law, seed));
            continue;
        }

        for carrier in carriers_for_interface(interface) {
            let profile = LawProfile::new(interface, law.name.as_str(), carrier);
            results.push(execute_algebra_law_profile(
                path, snapshot, law, &profile, seed, max_cases,
            ));
        }
    }

    results
}

/// Resolve `by test "..."` authored evidence for laws against an executed authored-test registry.
pub(crate) fn authored_law_test_results(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    authored_tests: &BTreeMap<String, TestResult>,
) -> Vec<TestResult> {
    snapshot
        .laws
        .iter()
        .filter_map(|law| match &law.test_evidence {
            Some(LawTestEvidence::Authored { test_name }) => Some(authored_law_test_result(
                path,
                snapshot,
                law,
                test_name,
                authored_tests.get(test_name),
            )),
            _ => None,
        })
        .collect()
}

fn authored_law_test_result(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    law: &RunnerLawMetadata,
    test_name: &str,
    authored_result: Option<&TestResult>,
) -> TestResult {
    let case_id = format!("synthesized/law/{}/by-test-authored", law.name);
    let (outcome, status, message) = match authored_result {
        Some(result) if result.outcome == Outcome::Pass => (
            Outcome::Pass,
            LawEvidenceStatus::Satisfied,
            format!(
                "law {} satisfied by authored Ash test '{}'",
                law.name, test_name
            ),
        ),
        Some(result) if result.outcome == Outcome::Skip || result.outcome == Outcome::Xfail => (
            Outcome::Error,
            LawEvidenceStatus::InvalidEvidence,
            format!(
                "invalid law test evidence: authored Ash test '{}' did not run to pass ({})",
                test_name, result.outcome
            ),
        ),
        Some(result) => (
            Outcome::Fail,
            LawEvidenceStatus::Broken,
            format!(
                "law {} broken: authored Ash test '{}' reported {}{}",
                law.name,
                test_name,
                result.outcome,
                result
                    .message
                    .as_deref()
                    .map(|message| format!(": {message}"))
                    .unwrap_or_default()
            ),
        ),
        None => (
            Outcome::Error,
            LawEvidenceStatus::InvalidEvidence,
            format!(
                "invalid law test evidence: by test target '{}' was not discovered as an Ash authored test",
                test_name
            ),
        ),
    };

    let mut repro = repro_artifact(
        path,
        snapshot.source_artifact_id.clone(),
        snapshot.check_summary_id.clone(),
        format!("law:{}:by-test-authored", law.id),
        0,
        1,
        None,
        json!({
            "source": "law",
            "law": law.name,
            "proof_evidence_family": "test",
            "test_mode": "authored",
            "evidence_status": evidence_status_name(status),
            "delegated_test": test_name,
            "proposition": law.proposition,
            "authored_test_outcome": authored_result.map(|result| result.outcome.to_string()),
            "authored_test_path": authored_result.map(|result| result.path.display().to_string()),
        }),
        None,
    );
    repro.replay_command = format!(
        "ASH_UNDER_TEST=${{ASH_UNDER_TEST:?set Ash candidate binary}}; \"$ASH_UNDER_TEST\" test {} --include-synthesized laws",
        path.display()
    );

    let mut result = TestResult::new(case_id, path.to_path_buf())
        .with_outcome(outcome)
        .with_source(TestSource::Law)
        .with_kind(TestKind::Unit)
        .with_duration(Duration::ZERO)
        .with_message(message)
        .with_repro_artifact(repro);
    result.evidence_family = Some("test".to_string());
    result.test_mode = Some("authored".to_string());
    result.evidence_status = Some(evidence_status_name(status).to_string());
    result.tags = vec![
        "synthesized".to_string(),
        "law".to_string(),
        "by-test".to_string(),
        "authored".to_string(),
    ];
    result
}

fn evidence_status_name(status: LawEvidenceStatus) -> &'static str {
    match status {
        LawEvidenceStatus::Satisfied => "satisfied",
        LawEvidenceStatus::Broken => "broken",
        LawEvidenceStatus::InvalidEvidence => "invalid_evidence",
        LawEvidenceStatus::Deferred => "deferred",
        LawEvidenceStatus::Untested => "untested",
    }
}

fn carriers_for_interface(interface: AlgebraInterface) -> Vec<CarrierType> {
    match interface {
        AlgebraInterface::Semigroup | AlgebraInterface::Monoid => {
            vec![CarrierType::String, CarrierType::List]
        }
        AlgebraInterface::Functor | AlgebraInterface::Applicative | AlgebraInterface::Monad => {
            vec![
                CarrierType::List,
                CarrierType::Option,
                CarrierType::Result,
                CarrierType::Act,
                CarrierType::Proc,
                CarrierType::Workflow,
            ]
        }
        AlgebraInterface::Comonad | AlgebraInterface::Kleisli | AlgebraInterface::Cokleisli => {
            vec![CarrierType::Act, CarrierType::Proc, CarrierType::Workflow]
        }
    }
}

/// Execute `by test property` laws by treating the law proposition as the property oracle.
pub(super) fn law_property_results(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    seed: Option<u64>,
    max_cases: Option<usize>,
) -> Vec<TestResult> {
    let seed = seed.unwrap_or(0);
    let max_cases = max_cases.unwrap_or(ALGEBRA_LAW_DEFAULT_MAX_CASES);
    let mut results = Vec::new();
    for law in snapshot
        .laws
        .iter()
        .filter(|law| matches!(law.test_evidence, Some(LawTestEvidence::Property)))
    {
        let Some(param_domains) = law_generated_domains(law) else {
            results.push(deferred_law_property_result(path, snapshot, law, seed));
            continue;
        };
        let cases = generated_cases(&param_domains, max_cases);
        if cases.is_empty() {
            results.push(deferred_law_property_result(path, snapshot, law, seed));
            continue;
        }
        for case in cases {
            let case_index = case.case_index;
            let bindings = case.bindings;
            let case_id = format!("synthesized/law/{}/property-case-{case_index}", law.name);
            let outcome = match evaluate_simple_bool_expression(&law.proposition, &bindings) {
                Ok(true) => Outcome::Pass,
                Ok(false) => Outcome::Fail,
                Err(_) => Outcome::Skip,
            };
            let status = match outcome {
                Outcome::Pass => LawEvidenceStatus::Satisfied,
                Outcome::Fail => LawEvidenceStatus::Broken,
                Outcome::Skip => LawEvidenceStatus::Deferred,
                _ => LawEvidenceStatus::Untested,
            };
            let shrunk = (outcome == Outcome::Fail).then(|| {
                shrink_bindings(&bindings, |candidate| {
                    evaluate_simple_bool_expression(&law.proposition, candidate) == Ok(false)
                })
            });
            let raw_bindings_snapshot = Value::Object(bindings.clone().into_iter().collect());
            let shrunk_counterexample = shrunk
                .as_ref()
                .map(|shrunk| Value::Object(shrunk.bindings.clone().into_iter().collect()));
            let shrink_trace = shrunk
                .as_ref()
                .map(|shrunk| Value::Array(shrunk.trace.clone()));
            let generated_input_snapshot = json!({
                "bindings": raw_bindings_snapshot,
                "generators": case.generators,
                "shrunk_counterexample": shrunk_counterexample,
                "shrink_trace": shrink_trace,
            });
            let mut repro = repro_artifact(
                path,
                snapshot.source_artifact_id.clone(),
                snapshot.check_summary_id.clone(),
                format!("law:{}:property-case-{case_index}", law.id),
                seed,
                case_index,
                Some(generated_input_snapshot.clone()),
                json!({
                    "source": "law",
                    "law": law.name,
                    "proof_evidence_family": "test",
                    "test_mode": "property",
                    "evidence_status": evidence_status_name(status),
                    "proposition": law.proposition,
                    "expected": true,
                    "case_index": case_index,
                    "generator_schema_version": "ash-property-generation-v1.0",
                    "shrunk_counterexample": generated_input_snapshot["shrunk_counterexample"],
                    "shrink_trace": generated_input_snapshot["shrink_trace"],
                }),
                None,
            );
            repro.replay_command = format!(
                "ASH_UNDER_TEST=${{ASH_UNDER_TEST:?set Ash candidate binary}}; \\\"$ASH_UNDER_TEST\\\" test {} --only-synthesized laws --seed {seed} --max-cases {max_cases}",
                path.display()
            );
            let message = match outcome {
                Outcome::Pass => format!(
                    "law {} held for generated property case {case_index}",
                    law.name
                ),
                Outcome::Fail => format!(
                    "law {} counterexample at seed {seed}, case {case_index}: {}; shrunk: {}",
                    law.name,
                    generated_input_snapshot["bindings"],
                    generated_input_snapshot["shrunk_counterexample"]
                ),
                Outcome::Skip => format!(
                    "deferred: unsupported law proposition {:?} for generated property input {}",
                    law.proposition, generated_input_snapshot
                ),
                _ => unreachable!("law property generation only emits pass/fail/skip"),
            };
            let mut result = TestResult::new(case_id, path.to_path_buf())
                .with_outcome(outcome)
                .with_source(TestSource::Law)
                .with_kind(TestKind::Property)
                .with_duration(Duration::ZERO)
                .with_seed(seed)
                .with_message(message)
                .with_repro_artifact(repro);
            result.failing_case = outcome.is_failure().then_some(case_index);
            result.evidence_family = Some("test".to_string());
            result.test_mode = Some("property".to_string());
            result.evidence_status = Some(evidence_status_name(status).to_string());
            result.tags = vec![
                "synthesized".to_string(),
                "law".to_string(),
                "property".to_string(),
            ];
            results.push(result);
        }
    }
    results
}

fn deferred_law_property_result(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    law: &RunnerLawMetadata,
    seed: u64,
) -> TestResult {
    let mut result = deferred_law_result(path, snapshot, law, seed);
    result.kind = TestKind::Property;
    result.test_mode = Some("property".to_string());
    result.evidence_family = Some("test".to_string());
    result.evidence_status = Some("deferred".to_string());
    result
}

fn execute_algebra_law_profile(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    law: &RunnerLawMetadata,
    profile: &LawProfile,
    seed: u64,
    max_cases: usize,
) -> TestResult {
    let case_id = algebra_case_id(profile);
    if max_cases == 0 {
        return deferred_result_with_kind(
            path,
            TestSource::Law,
            TestKind::Property,
            case_id,
            format!(
                "deferred: {} law '{}' for {} carrier has zero configured cases",
                algebra_interface_name(profile.interface),
                profile.law_name,
                carrier_name(&profile.carrier),
            ),
            algebra_repro(path, snapshot, law, profile, seed, 1, None, None),
        );
    }
    if !profile.is_executable {
        return deferred_result_with_kind(
            path,
            TestSource::Law,
            TestKind::Property,
            case_id,
            format!(
                "deferred: {} law '{}' for {} carrier — bounded equivalence metadata required",
                algebra_interface_name(profile.interface),
                profile.law_name,
                carrier_name(&profile.carrier),
            ),
            algebra_repro(path, snapshot, law, profile, seed, 1, None, None),
        );
    }
    if law_requires_runtime_function_metadata(profile) {
        return deferred_result_with_kind(
            path,
            TestSource::Law,
            TestKind::Property,
            case_id,
            format!(
                "deferred: {} law '{}' for {} carrier requires executable function metadata",
                algebra_interface_name(profile.interface),
                profile.law_name,
                carrier_name(&profile.carrier),
            ),
            algebra_repro(path, snapshot, law, profile, seed, 1, None, None),
        );
    }

    let mut checked = 0usize;
    for (index, bindings) in algebra_law_bindings(profile).into_iter().enumerate() {
        if checked >= max_cases {
            break;
        }
        checked += 1;
        if !algebra_law_holds(profile, &bindings) {
            let mut result = TestResult::new(&case_id, path.to_path_buf())
                .with_outcome(Outcome::Fail)
                .with_source(TestSource::Law)
                .with_kind(TestKind::Property)
                .with_duration(Duration::ZERO)
                .with_seed(seed)
                .with_message(format!(
                    "{} law '{}' failed for {} carrier at seed {seed}, case {}: {}",
                    algebra_interface_name(profile.interface),
                    profile.law_name,
                    carrier_name(&profile.carrier),
                    index + 1,
                    Value::Object(bindings.clone().into_iter().collect())
                ))
                .with_repro_artifact(algebra_repro(
                    path,
                    snapshot,
                    law,
                    profile,
                    seed,
                    index + 1,
                    Some(Value::Object(bindings.into_iter().collect())),
                    Some(false),
                ));
            result.failing_case = Some(index + 1);
            result.tags = algebra_tags(profile, "failed");
            return result;
        }
    }

    let mut result = TestResult::new(&case_id, path.to_path_buf())
        .with_outcome(Outcome::Pass)
        .with_source(TestSource::Law)
        .with_kind(TestKind::Property)
        .with_duration(Duration::ZERO)
        .with_seed(seed)
        .with_message(format!(
            "{} law '{}' for {} carrier passed {checked} generated algebra cases",
            algebra_interface_name(profile.interface),
            profile.law_name,
            carrier_name(&profile.carrier),
        ))
        .with_repro_artifact(algebra_repro(
            path,
            snapshot,
            law,
            profile,
            seed,
            checked.max(1),
            None,
            Some(true),
        ));
    result.tags = algebra_tags(profile, "passed");
    result
}

fn law_requires_runtime_function_metadata(profile: &LawProfile) -> bool {
    matches!(profile.interface, AlgebraInterface::Monad)
        || (matches!(profile.interface, AlgebraInterface::Applicative)
            && !matches!(profile.law_name.as_str(), "identity"))
}

fn unknown_algebra_law_result(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    law: &RunnerLawMetadata,
    seed: u64,
) -> TestResult {
    let mut repro = repro_artifact(
        path,
        snapshot.source_artifact_id.clone(),
        snapshot.check_summary_id.clone(),
        format!("law:{}:unknown-algebra-law", law.id),
        seed,
        1,
        None,
        json!({
            "source": "law",
            "law": law.name,
            "interface": law.owner,
            "status": "unknown-law-profile",
        }),
        None,
    );
    repro.replay_command = format!(
        "ash test {} --only-synthesized laws --seed {seed}",
        path.display()
    );
    deferred_result_with_kind(
        path,
        TestSource::Law,
        TestKind::Property,
        format!(
            "synthesized/algebra/unknown/{}/{}",
            law.owner.as_deref().unwrap_or("module"),
            law.name
        ),
        "deferred: algebra law declaration has no matching law profile registry entry",
        repro,
    )
}

fn algebra_case_id(profile: &LawProfile) -> String {
    format!(
        "synthesized/algebra/{}/{}/{}",
        carrier_name(&profile.carrier),
        algebra_interface_name(profile.interface),
        profile.law_name,
    )
}

#[allow(clippy::too_many_arguments)]
fn algebra_repro(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    law: &RunnerLawMetadata,
    profile: &LawProfile,
    seed: u64,
    case_index: usize,
    generated_input_snapshot: Option<Value>,
    expected: Option<bool>,
) -> crate::test_runner::types::ReproArtifact {
    let mut repro = repro_artifact(
        path,
        snapshot.source_artifact_id.clone(),
        snapshot.check_summary_id.clone(),
        format!(
            "law:{}:{}:{}",
            law.id,
            carrier_name(&profile.carrier),
            case_index
        ),
        seed,
        case_index,
        generated_input_snapshot.clone(),
        json!({
            "source": "law",
            "law": law.name,
            "interface": algebra_interface_name(profile.interface),
            "carrier": carrier_name(&profile.carrier),
            "proposition": law.proposition,
            "expected": expected,
            "executor": "algebra_law_profile",
        }),
        generated_input_snapshot,
    );
    repro.replay_command = format!(
        "ash test {} --only-synthesized laws --seed {seed}",
        path.display()
    );
    repro
}

fn algebra_tags(profile: &LawProfile, status: &str) -> Vec<String> {
    vec![
        "synthesized".to_string(),
        "law".to_string(),
        "algebra".to_string(),
        algebra_interface_name(profile.interface).to_ascii_lowercase(),
        carrier_name(&profile.carrier).to_ascii_lowercase(),
        status.to_string(),
    ]
}

fn algebra_law_bindings(profile: &LawProfile) -> Vec<BTreeMap<String, Value>> {
    match profile.interface {
        AlgebraInterface::Semigroup => semigroup_bindings(&profile.carrier),
        AlgebraInterface::Monoid => monoid_bindings(&profile.carrier),
        AlgebraInterface::Functor | AlgebraInterface::Applicative | AlgebraInterface::Monad => {
            higher_kinded_bindings(&profile.carrier)
        }
        AlgebraInterface::Comonad | AlgebraInterface::Kleisli | AlgebraInterface::Cokleisli => {
            Vec::new()
        }
    }
}

fn semigroup_bindings(carrier: &CarrierType) -> Vec<BTreeMap<String, Value>> {
    carrier_values(carrier)
        .into_iter()
        .take(3)
        .flat_map(|a| {
            carrier_values(carrier)
                .into_iter()
                .take(2)
                .flat_map(move |b| {
                    let a = a.clone();
                    carrier_values(carrier).into_iter().take(2).map(move |c| {
                        BTreeMap::from([
                            ("a".to_string(), a.clone()),
                            ("b".to_string(), b.clone()),
                            ("c".to_string(), c),
                        ])
                    })
                })
        })
        .collect()
}

fn monoid_bindings(carrier: &CarrierType) -> Vec<BTreeMap<String, Value>> {
    carrier_values(carrier)
        .into_iter()
        .map(|a| BTreeMap::from([("a".to_string(), a)]))
        .collect()
}

fn higher_kinded_bindings(carrier: &CarrierType) -> Vec<BTreeMap<String, Value>> {
    carrier_values(carrier)
        .into_iter()
        .map(|value| BTreeMap::from([("m".to_string(), value)]))
        .collect()
}

fn carrier_values(carrier: &CarrierType) -> Vec<Value> {
    match carrier {
        CarrierType::String => vec![json!(""), json!("a"), json!("ash")],
        CarrierType::List => vec![json!([]), json!([1]), json!([1, 2])],
        CarrierType::Option => vec![json!(null), json!(1), json!(2)],
        CarrierType::Result => vec![json!({"Ok": 1}), json!({"Ok": 2}), json!({"Err": "e"})],
        CarrierType::Act | CarrierType::Proc | CarrierType::Workflow => Vec::new(),
    }
}

fn algebra_law_holds(profile: &LawProfile, bindings: &BTreeMap<String, Value>) -> bool {
    match (profile.interface, profile.law_name.as_str()) {
        (AlgebraInterface::Semigroup, "associativity") => {
            let a = &bindings["a"];
            let b = &bindings["b"];
            let c = &bindings["c"];
            append(&append(a, b, &profile.carrier), c, &profile.carrier)
                == append(a, &append(b, c, &profile.carrier), &profile.carrier)
        }
        (AlgebraInterface::Monoid, "left_identity") => {
            let a = &bindings["a"];
            append(&empty(&profile.carrier), a, &profile.carrier) == *a
        }
        (AlgebraInterface::Monoid, "right_identity") => {
            let a = &bindings["a"];
            append(a, &empty(&profile.carrier), &profile.carrier) == *a
        }
        (AlgebraInterface::Functor, "identity") => {
            fmap(bindings.get("m").unwrap(), Function::Id, &profile.carrier) == bindings["m"]
        }
        (AlgebraInterface::Functor, "composition") => {
            let m = &bindings["m"];
            fmap(
                &fmap(m, Function::Double, &profile.carrier),
                Function::Inc,
                &profile.carrier,
            ) == fmap(m, Function::IncAfterDouble, &profile.carrier)
        }
        (AlgebraInterface::Applicative, "identity") => {
            fmap(&bindings["m"], Function::Id, &profile.carrier) == bindings["m"]
        }
        (AlgebraInterface::Monad, "left_identity") => {
            bind(
                &pure(json!(1), &profile.carrier),
                Function::IncPure,
                &profile.carrier,
            ) == pure(json!(2), &profile.carrier)
        }
        (AlgebraInterface::Monad, "right_identity") => {
            bind(&bindings["m"], Function::Pure, &profile.carrier) == bindings["m"]
        }
        (AlgebraInterface::Monad, "associativity") => {
            let m = &bindings["m"];
            bind(
                &bind(m, Function::IncPure, &profile.carrier),
                Function::DoublePure,
                &profile.carrier,
            ) == bind(m, Function::IncThenDoublePure, &profile.carrier)
        }
        _ => false,
    }
}

#[derive(Clone, Copy)]
enum Function {
    Id,
    Inc,
    Double,
    IncAfterDouble,
    Pure,
    IncPure,
    DoublePure,
    IncThenDoublePure,
}

fn apply_function(value: &Value, function: Function) -> Value {
    let Some(n) = value.as_i64() else {
        return value.clone();
    };
    match function {
        Function::Id | Function::Pure => json!(n),
        Function::Inc | Function::IncPure => json!(n + 1),
        Function::Double | Function::DoublePure => json!(n * 2),
        Function::IncAfterDouble => json!((n * 2) + 1),
        Function::IncThenDoublePure => json!((n + 1) * 2),
    }
}

fn fmap(value: &Value, function: Function, carrier: &CarrierType) -> Value {
    match carrier {
        CarrierType::List => Value::Array(
            value
                .as_array()
                .into_iter()
                .flatten()
                .map(|item| apply_function(item, function))
                .collect(),
        ),
        CarrierType::Option => {
            if value.is_null() {
                Value::Null
            } else {
                apply_function(value, function)
            }
        }
        CarrierType::Result => {
            if let Some(ok) = value.get("Ok") {
                json!({"Ok": apply_function(ok, function)})
            } else {
                value.clone()
            }
        }
        _ => value.clone(),
    }
}

fn pure(value: Value, carrier: &CarrierType) -> Value {
    match carrier {
        CarrierType::List => Value::Array(vec![value]),
        CarrierType::Option => value,
        CarrierType::Result => json!({"Ok": value}),
        _ => value,
    }
}

fn bind(value: &Value, function: Function, carrier: &CarrierType) -> Value {
    match carrier {
        CarrierType::List => Value::Array(
            value
                .as_array()
                .into_iter()
                .flatten()
                .flat_map(|item| match apply_function(item, function) {
                    Value::Array(items) => items,
                    item => vec![item],
                })
                .collect(),
        ),
        CarrierType::Option => {
            if value.is_null() {
                Value::Null
            } else {
                apply_function(value, function)
            }
        }
        CarrierType::Result => {
            if let Some(ok) = value.get("Ok") {
                json!({"Ok": apply_function(ok, function)})
            } else {
                value.clone()
            }
        }
        _ => value.clone(),
    }
}

fn append(left: &Value, right: &Value, carrier: &CarrierType) -> Value {
    match carrier {
        CarrierType::String => json!(format!(
            "{}{}",
            left.as_str().unwrap_or_default(),
            right.as_str().unwrap_or_default()
        )),
        CarrierType::List => {
            let mut items = left.as_array().cloned().unwrap_or_default();
            items.extend(right.as_array().cloned().unwrap_or_default());
            Value::Array(items)
        }
        _ => left.clone(),
    }
}

fn empty(carrier: &CarrierType) -> Value {
    match carrier {
        CarrierType::String => json!(""),
        CarrierType::List => json!([]),
        _ => Value::Null,
    }
}

fn algebra_interface_name(interface: AlgebraInterface) -> &'static str {
    match interface {
        AlgebraInterface::Semigroup => "Semigroup",
        AlgebraInterface::Monoid => "Monoid",
        AlgebraInterface::Functor => "Functor",
        AlgebraInterface::Applicative => "Applicative",
        AlgebraInterface::Monad => "Monad",
        AlgebraInterface::Comonad => "Comonad",
        AlgebraInterface::Kleisli => "Kleisli",
        AlgebraInterface::Cokleisli => "Cokleisli",
    }
}

fn carrier_name(carrier: &CarrierType) -> &'static str {
    match carrier {
        CarrierType::String => "String",
        CarrierType::List => "List",
        CarrierType::Option => "Option",
        CarrierType::Result => "Result",
        CarrierType::Act => "Act",
        CarrierType::Proc => "Proc",
        CarrierType::Workflow => "Workflow",
    }
}

pub(super) fn law_smallworld_results(
    path: &Path,
    snapshot: &RunnerIntrospectionSnapshot,
    seed: Option<u64>,
    max_worlds: Option<usize>,
) -> Vec<TestResult> {
    let seed = seed.unwrap_or(0);
    let mut results = Vec::new();

    for law in &snapshot.laws {
        if matches!(
            law.test_evidence,
            Some(LawTestEvidence::Authored { .. } | LawTestEvidence::Property)
        ) {
            continue;
        }
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
            let status = match outcome {
                Outcome::Pass => LawEvidenceStatus::Satisfied,
                Outcome::Fail => LawEvidenceStatus::Broken,
                Outcome::Skip => LawEvidenceStatus::Deferred,
                _ => LawEvidenceStatus::Untested,
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
                    "proof_evidence_family": "test",
                    "test_mode": "small_world",
                    "evidence_status": evidence_status_name(status),
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
            result.evidence_family = Some("test".to_string());
            result.test_mode = Some("small_world".to_string());
            result.evidence_status = Some(evidence_status_name(status).to_string());
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

fn law_generated_domains(
    law: &RunnerLawMetadata,
) -> Option<Vec<super::value_generation::GeneratedValueDomain>> {
    law.params
        .iter()
        .map(|param| generated_domain_for_param(param))
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
