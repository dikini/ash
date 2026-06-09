use std::path::PathBuf;

use ash_core::Expr as CoreExpr;
use serde_json::Value;

use crate::test_runner::types::{Outcome, TestKind};

use super::*;

fn parse_module_for_law_extraction(source: &str) -> ModuleFile {
    ash_parser::parse_surface_file(source)
        .unwrap_or_else(|errors| panic!("module should parse: {source}\nerrors: {errors:?}"))
}

#[test]
fn extract_laws_returns_interface_law_metadata() {
    let module = parse_module_for_law_extraction(
        r#"
            interface Monad<M> {
                bind(M<A>, (A) -> M<B>) -> M<B>
                law left_identity(x: A, f: (A) -> M<B>): bind(unit(x), f) == f(x)
            }
            "#,
    );

    let laws = extract_laws(&module);

    assert_eq!(laws.len(), 1);
    assert_eq!(laws[0].id, "law:interface:Monad:left_identity");
    assert_eq!(laws[0].name, "left_identity");
    assert_eq!(laws[0].scope, LawScope::Interface);
    assert_eq!(laws[0].owner.as_deref(), Some("Monad"));
    assert_eq!(laws[0].params, vec!["x: A", "f: (A) -> M<B>"]);
    assert_eq!(laws[0].proposition, "bind(unit(x), f) == f(x)");
}

#[test]
fn extract_laws_returns_std_algebra_law_metadata() {
    let semigroup = parse_module_for_law_extraction(include_str!(
        "../../../../../std/src/algebra/semigroup.ash"
    ));
    let monoid =
        parse_module_for_law_extraction(include_str!("../../../../../std/src/algebra/monoid.ash"));

    let semigroup_laws = extract_laws(&semigroup);
    let monoid_laws = extract_laws(&monoid);

    assert_eq!(semigroup_laws.len(), 1);
    assert_eq!(
        semigroup_laws[0].id,
        "law:interface:Semigroup:associativity"
    );
    assert_eq!(semigroup_laws[0].name, "associativity");
    assert_eq!(semigroup_laws[0].scope, LawScope::Interface);
    assert_eq!(semigroup_laws[0].owner.as_deref(), Some("Semigroup"));
    assert_eq!(
        semigroup_laws[0].params,
        vec!["a: A", "b: A", "c: A", "eq: Eq<A>"]
    );
    assert_eq!(
        semigroup_laws[0].proposition,
        "eq.equiv(append(append(a, b), c), append(a, append(b, c)))"
    );

    let monoid_names: std::collections::BTreeSet<_> =
        monoid_laws.iter().map(|law| law.name.as_str()).collect();
    assert_eq!(
        monoid_names,
        std::collections::BTreeSet::from(["left_identity", "right_identity"])
    );
    assert!(
        monoid_laws
            .iter()
            .all(|law| law.scope == LawScope::Interface)
    );
    assert!(
        monoid_laws
            .iter()
            .all(|law| law.owner.as_deref() == Some("Monoid"))
    );

    let left_identity = monoid_laws
        .iter()
        .find(|law| law.name == "left_identity")
        .expect("left_identity law should be extracted");
    assert_eq!(left_identity.params, vec!["a: A", "eq: Eq<A>"]);
    assert_eq!(left_identity.proposition, "eq.equiv(append(empty(), a), a)");

    let right_identity = monoid_laws
        .iter()
        .find(|law| law.name == "right_identity")
        .expect("right_identity law should be extracted");
    assert_eq!(right_identity.params, vec!["a: A", "eq: Eq<A>"]);
    assert_eq!(
        right_identity.proposition,
        "eq.equiv(append(a, empty()), a)"
    );
}

#[test]
fn extract_laws_returns_module_law_metadata() {
    let module = parse_module_for_law_extraction(
        r#"
            fn id(x: Int) -> Int { x }
            law id_reflexive(x: Int): id(x) == x
            "#,
    );

    let laws = extract_laws(&module);

    assert_eq!(laws.len(), 1);
    assert_eq!(laws[0].id, "law:module:id_reflexive");
    assert_eq!(laws[0].name, "id_reflexive");
    assert_eq!(laws[0].scope, LawScope::Module);
    assert_eq!(laws[0].owner, None);
    assert_eq!(laws[0].params, vec!["x: Int"]);
    assert_eq!(laws[0].proposition, "id(x) == x");
}

#[test]
fn extract_laws_omits_module_law_with_matching_proof() {
    let module = parse_module_for_law_extraction(
        r#"
            law id_reflexive(x: Int): x == x
            proof id_reflexive(x: Int) {
                by_definition
            }
            "#,
    );

    let laws = extract_laws(&module);

    assert!(
        laws.is_empty(),
        "proof-backed module laws should not synthesize fallback tests: {laws:#?}"
    );
}

#[test]
fn extract_laws_keeps_interface_law_when_only_module_proof_name_matches() {
    let module = parse_module_for_law_extraction(
        r#"
            interface Eq<A> {
                law reflexive(x: A): x == x
            }
            law reflexive(x: Int): x == x
            proof reflexive(x: Int) {
                by_definition
            }
            "#,
    );

    let laws = extract_laws(&module);

    assert_eq!(laws.len(), 1);
    assert_eq!(laws[0].id, "law:interface:Eq:reflexive");
    assert_eq!(laws[0].scope, LawScope::Interface);
}

#[test]
fn extract_laws_keeps_module_law_when_only_impl_proof_name_matches() {
    let module = parse_module_for_law_extraction(
        r#"
            interface Eq<A> {
                law reflexive(x: A): x == x
            }
            impl Eq<Int> {
                proof reflexive(x: Int) {
                    by_definition
                }
            }
            law reflexive(x: Int): x == x
            "#,
    );

    let laws = extract_laws(&module);

    assert_eq!(laws.len(), 1);
    assert_eq!(laws[0].id, "law:module:reflexive");
    assert_eq!(laws[0].scope, LawScope::Module);
}

#[test]
fn extract_laws_delegates_module_law_with_by_test_proof() {
    let module = parse_module_for_law_extraction(
        r#"
            law id_reflexive(x: Int): x == x
            proof id_reflexive(x: Int) {
                by test "id_reflexive_smallworld"
            }
            "#,
    );

    let laws = extract_laws(&module);

    assert_eq!(laws.len(), 1);
    assert_eq!(laws[0].id, "law:module:id_reflexive");
    assert_eq!(
        laws[0].delegated_test.as_deref(),
        Some("id_reflexive_smallworld")
    );
}

#[test]
fn extract_laws_delegates_interface_law_with_impl_by_test_proof() {
    let module = parse_module_for_law_extraction(
        r#"
            interface Eq<A> {
                law reflexive(x: A): x == x
            }
            impl Eq<Int> {
                proof reflexive(x: Int) {
                    by test "eq_int_reflexive_smallworld"
                }
            }
            "#,
    );

    let laws = extract_laws(&module);

    assert_eq!(laws.len(), 1);
    assert_eq!(laws[0].id, "law:interface:Eq:reflexive");
    assert_eq!(
        laws[0].delegated_test.as_deref(),
        Some("eq_int_reflexive_smallworld")
    );
}

fn law_snapshot(law: RunnerLawMetadata) -> RunnerIntrospectionSnapshot {
    RunnerIntrospectionSnapshot {
        schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
        module_identity: "module:laws".to_string(),
        source_artifact_id: "source:laws.ash".to_string(),
        check_summary_id: "checked:laws".to_string(),
        laws: vec![law],
        ..RunnerIntrospectionSnapshot::default()
    }
}

fn module_law(name: &str, params: Vec<&str>, proposition: &str) -> RunnerLawMetadata {
    RunnerLawMetadata {
        id: format!("law:module:{name}"),
        name: name.to_string(),
        scope: LawScope::Module,
        owner: None,
        params: params.into_iter().map(str::to_string).collect(),
        proposition: proposition.to_string(),
        delegated_test: None,
    }
}

#[test]
fn law_smallworld_generation_passes_valid_unproven_law() {
    let snapshot = law_snapshot(module_law("reflexive", vec!["x: Int"], "x == x"));

    let results = synthesize_from_snapshot_with_limits(
        Path::new("laws.ash"),
        &snapshot,
        Some(42),
        None,
        Some(3),
    );

    let law_results = results
        .iter()
        .filter(|result| result.name.starts_with("synthesized/law/reflexive/"))
        .collect::<Vec<_>>();
    assert_eq!(law_results.len(), 3);
    assert!(
        law_results
            .iter()
            .all(|result| result.outcome == Outcome::Pass)
    );
    assert!(
        law_results
            .iter()
            .all(|result| result.source == TestSource::Law)
    );
    assert!(
        law_results
            .iter()
            .all(|result| result.kind == TestKind::SmallWorld)
    );
    assert!(law_results.iter().all(|result| result.seed == Some(42)));
}

#[test]
fn law_smallworld_generation_carries_by_test_delegation_metadata() {
    let mut law = module_law("reflexive", vec!["x: Int"], "x == x");
    law.delegated_test = Some("reflexive_smallworld".to_string());
    let snapshot = law_snapshot(law);

    let results = synthesize_from_snapshot_with_limits(
        Path::new("laws.ash"),
        &snapshot,
        Some(42),
        None,
        Some(1),
    );

    assert_eq!(results.len(), 1);
    let repro = results[0]
        .repro_artifact
        .as_ref()
        .expect("delegated law result should include repro metadata");
    assert_eq!(
        repro.oracle_snapshot["delegated_test"],
        json!("reflexive_smallworld")
    );
}

#[test]
fn law_smallworld_generation_reports_counterexample_for_broken_law() {
    let snapshot = law_snapshot(module_law("not_reflexive", vec!["x: Int"], "x != x"));

    let results = synthesize_from_snapshot_with_limits(
        Path::new("laws.ash"),
        &snapshot,
        Some(7),
        None,
        Some(3),
    );

    let failing = results
        .iter()
        .find(|result| result.name == "synthesized/law/not_reflexive/world-1")
        .expect("broken law should generate a first small-world case");
    assert_eq!(failing.outcome, Outcome::Fail);
    assert_eq!(failing.source, TestSource::Law);
    assert_eq!(failing.kind, TestKind::SmallWorld);
    assert_eq!(failing.seed, Some(7));
    assert_eq!(failing.failing_case, Some(1));
    assert!(
        failing
            .message
            .as_deref()
            .is_some_and(|message| message.contains("counterexample")),
        "failure should report counterexample, got {:?}",
        failing.message
    );
    let repro = failing
        .repro_artifact
        .as_ref()
        .expect("law failures should include repro metadata");
    assert_eq!(repro.seed, 7);
    assert_eq!(repro.world_index, Some(1));
    assert_eq!(repro.generated_input_snapshot, Some(json!({ "x": -1 })));
}

#[test]
fn law_smallworld_generation_uses_default_cap_for_parameter_products() {
    let snapshot = law_snapshot(module_law(
        "bounded_product",
        vec!["x: Int", "y: Bool", "z: String"],
        "x == x",
    ));

    let results = synthesize_from_snapshot_with_limits(
        Path::new("laws.ash"),
        &snapshot,
        Some(11),
        None,
        None,
    );

    assert_eq!(
        results.len(),
        8,
        "uncapped law products should use the small default cap rather than materializing the full product"
    );
    assert_eq!(
        results.last().and_then(|result| result.world_index),
        Some(8)
    );
}

#[test]
fn law_smallworld_generation_runs_zero_parameter_law_once() {
    let snapshot = law_snapshot(module_law("zero_arg", vec![], "true == true"));

    let results = synthesize_from_snapshot_with_limits(
        Path::new("laws.ash"),
        &snapshot,
        Some(13),
        None,
        None,
    );

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "synthesized/law/zero_arg/world-1");
    assert_eq!(results[0].outcome, Outcome::Pass);
    assert_eq!(
        results[0]
            .repro_artifact
            .as_ref()
            .unwrap()
            .generated_input_snapshot,
        Some(json!({}))
    );
}

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
fn structured_contract_metadata_executes_postcondition_against_target_output() {
    let snapshot = postcondition_snapshot(
        Some(ContractExecutableTarget {
            kind: ContractExecutableTargetKind::PureFunction,
            target_ref: "identity".to_string(),
            setup: ContractExecutionSetup::PureNoSetup,
            body: ContractTargetBody::ReturnExpression {
                expression: core_var("x"),
            },
        }),
        "result == x",
    );

    let results = synthesize_from_snapshot(Path::new("test.ash"), &snapshot);
    let result = results
        .iter()
        .find(|result| result.name.contains("ensures"))
        .unwrap_or_else(|| panic!("postcondition result should be synthesized: {results:#?}"));

    assert_eq!(result.outcome, Outcome::Pass);
    let repro = result
        .repro_artifact
        .as_ref()
        .expect("postcondition execution should carry repro data");
    assert_eq!(
        repro.generated_input_snapshot.as_ref().unwrap()["bindings"]["x"],
        7
    );
    assert_eq!(repro.oracle_snapshot["target_output"], 7);
    assert_eq!(repro.oracle_snapshot["ensures"], "result == x");
}

#[test]
fn structured_contract_postcondition_failure_is_fail_not_skip_or_pass() {
    let snapshot = postcondition_snapshot(
        Some(ContractExecutableTarget {
            kind: ContractExecutableTargetKind::PureFunction,
            target_ref: "identity".to_string(),
            setup: ContractExecutionSetup::PureNoSetup,
            body: ContractTargetBody::ReturnExpression {
                expression: core_var("x"),
            },
        }),
        "result != x",
    );

    let results = synthesize_from_snapshot(Path::new("test.ash"), &snapshot);
    let result = results
        .iter()
        .find(|result| result.name.contains("ensures"))
        .unwrap_or_else(|| panic!("postcondition result should be synthesized: {results:#?}"));

    assert_eq!(result.outcome, Outcome::Fail);
    assert!(
        result
            .message
            .as_deref()
            .is_some_and(|message| message.contains("postcondition failed")),
        "failing postcondition should explain the evaluated oracle: {result:#?}"
    );
}

#[test]
fn contract_postcondition_without_executable_target_metadata_defers() {
    let snapshot = postcondition_snapshot(None, "result == x");

    let results = synthesize_from_snapshot(Path::new("test.ash"), &snapshot);

    assert!(
        results.iter().all(|result| result.outcome != Outcome::Pass),
        "missing executable target metadata must never pass: {results:#?}"
    );
    assert!(
        results.iter().any(|result| {
            result.name.contains("postcondition-deferred")
                && result.message.as_deref().is_some_and(|message| {
                    message.contains("lacks executable postcondition target metadata")
                })
        }),
        "missing executable target metadata must defer precisely: {results:#?}"
    );
}

#[test]
fn contract_postcondition_without_structured_oracle_metadata_defers() {
    let mut snapshot = postcondition_snapshot(
        Some(ContractExecutableTarget {
            kind: ContractExecutableTargetKind::PureFunction,
            target_ref: "identity".to_string(),
            setup: ContractExecutionSetup::PureNoSetup,
            body: ContractTargetBody::ReturnExpression {
                expression: core_var("x"),
            },
        }),
        "result == x",
    );
    snapshot.contracts[0].executable_postconditions.clear();

    let results = synthesize_from_snapshot(Path::new("test.ash"), &snapshot);

    assert!(
        results.iter().all(|result| result.outcome != Outcome::Pass),
        "string-only postcondition metadata must never pass: {results:#?}"
    );
    assert!(
        results.iter().any(|result| {
            result.name.contains("postcondition-deferred")
                && result.message.as_deref().is_some_and(|message| {
                    message.contains("postcondition metadata is not executable")
                })
        }),
        "missing structured postcondition oracle should defer precisely: {results:#?}"
    );
}

#[test]
fn contract_postcondition_with_unsupported_target_kind_defers() {
    let snapshot = postcondition_snapshot(
        Some(ContractExecutableTarget {
            kind: ContractExecutableTargetKind::WorkflowCallable,
            target_ref: "workflow_target".to_string(),
            setup: ContractExecutionSetup::ExplicitFinite,
            body: ContractTargetBody::ReturnExpression {
                expression: core_var("x"),
            },
        }),
        "result == x",
    );

    let results = synthesize_from_snapshot(Path::new("test.ash"), &snapshot);

    assert!(
        results.iter().all(|result| result.outcome != Outcome::Pass),
        "unsupported target kinds must never pass: {results:#?}"
    );
    assert!(
        results.iter().any(|result| {
            result.name.contains("postcondition-deferred")
                && result.message.as_deref().is_some_and(|message| {
                    message.contains("unsupported contract target kind workflow_callable")
                })
        }),
        "unsupported target kind should carry a precise skip reason: {results:#?}"
    );
}

#[test]
fn contract_postcondition_with_missing_setup_defers() {
    let snapshot = postcondition_snapshot(
        Some(ContractExecutableTarget {
            kind: ContractExecutableTargetKind::PureFunction,
            target_ref: "identity".to_string(),
            setup: ContractExecutionSetup::Missing,
            body: ContractTargetBody::ReturnExpression {
                expression: core_var("x"),
            },
        }),
        "result == x",
    );

    let results = synthesize_from_snapshot(Path::new("test.ash"), &snapshot);

    assert!(
        results.iter().all(|result| result.outcome != Outcome::Pass),
        "missing setup must never pass: {results:#?}"
    );
    assert!(
        results.iter().any(|result| {
            result.name.contains("postcondition-deferred")
                && result
                    .message
                    .as_deref()
                    .is_some_and(|message| message.contains("execution setup is missing"))
        }),
        "missing setup should carry a precise skip reason: {results:#?}"
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
                kind: SmallWorldOracleKind::TargetOutputEquals,
                expected: json!(true),
            }),
            executable_target: Some(smallworld_literal_target(json!(true))),
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
fn smallworld_target_output_drives_oracle_not_claimed_control_state() {
    let snapshot = RunnerIntrospectionSnapshot {
        source_artifact_id: "source:worlds.ash".to_string(),
        check_summary_id: "check:world-summary".to_string(),
        small_world_domains: vec![SmallWorldDomain {
            id: "target-output-worlds".to_string(),
            domain_kind: SmallWorldDomainKind::ExplicitStates,
            source: TestSource::Obligation,
            explicit_states: vec![SmallWorldState {
                id: "claimed-allowed".to_string(),
                world_kind: "policy_context".to_string(),
                control_state: Some("allowed".to_string()),
                bindings: BTreeMap::from([("smallworld_ok".to_string(), json!(false))]),
                ..SmallWorldState::default()
            }],
            oracle: Some(SmallWorldOracle {
                kind: SmallWorldOracleKind::TargetOutputEquals,
                expected: json!(true),
            }),
            executable_target: Some(smallworld_expr_target(core_var("smallworld_ok"))),
            ..SmallWorldDomain::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot(Path::new("worlds.ash"), &snapshot);

    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].outcome,
        Outcome::Fail,
        "small-world pass/fail must come from executed target output, not claimed control_state: {results:#?}"
    );
    let oracle_snapshot = &results[0]
        .repro_artifact
        .as_ref()
        .expect("smallworld result should include repro")
        .oracle_snapshot;
    assert_eq!(
        oracle_snapshot["target_execution"]["target_output"],
        json!(false)
    );
}

#[test]
fn smallworld_metadata_only_oracle_with_executable_target_defers() {
    let snapshot = RunnerIntrospectionSnapshot {
        source_artifact_id: "source:worlds.ash".to_string(),
        check_summary_id: "check:world-summary".to_string(),
        small_world_domains: vec![SmallWorldDomain {
            id: "metadata-only-oracle-worlds".to_string(),
            domain_kind: SmallWorldDomainKind::ExplicitStates,
            source: TestSource::Policy,
            explicit_states: vec![SmallWorldState {
                id: "claimed-allowed".to_string(),
                world_kind: "policy_context".to_string(),
                control_state: Some("allowed".to_string()),
                bindings: BTreeMap::from([("smallworld_ok".to_string(), json!(false))]),
                ..SmallWorldState::default()
            }],
            oracle: Some(SmallWorldOracle {
                kind: SmallWorldOracleKind::ControlStateEquals,
                expected: json!("allowed"),
            }),
            executable_target: Some(smallworld_expr_target(core_var("smallworld_ok"))),
            ..SmallWorldDomain::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot(Path::new("worlds.ash"), &snapshot);

    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].outcome,
        Outcome::Skip,
        "TASK-1016 must not allow legacy metadata-only small-world oracles to pass after decorative target execution: {results:#?}"
    );
    assert!(
        results[0]
            .message
            .as_deref()
            .unwrap_or_default()
            .contains("deferred"),
        "metadata-only oracle must defer with an honest reason: {results:#?}"
    );
}

#[test]
fn smallworld_without_executable_target_defers() {
    let snapshot = RunnerIntrospectionSnapshot {
        source_artifact_id: "source:worlds.ash".to_string(),
        check_summary_id: "check:world-summary".to_string(),
        small_world_domains: vec![SmallWorldDomain {
            id: "missing-target-worlds".to_string(),
            domain_kind: SmallWorldDomainKind::ExplicitStates,
            source: TestSource::Policy,
            explicit_states: vec![SmallWorldState {
                id: "allowed".to_string(),
                world_kind: "policy_context".to_string(),
                control_state: Some("allowed".to_string()),
                ..SmallWorldState::default()
            }],
            oracle: Some(SmallWorldOracle {
                kind: SmallWorldOracleKind::ControlStateEquals,
                expected: json!("allowed"),
            }),
            ..SmallWorldDomain::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot(Path::new("worlds.ash"), &snapshot);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].outcome, Outcome::Skip);
    assert!(
        results[0]
            .message
            .as_deref()
            .unwrap_or_default()
            .contains("deferred"),
        "missing executable target metadata must defer instead of passing: {results:#?}"
    );
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
                kind: SmallWorldOracleKind::TargetOutputEquals,
                expected: json!(0),
            }),
            executable_target: Some(smallworld_expr_target(core_var("value"))),
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
fn uncapped_bounded_int_world_enumeration_defers_instead_of_materializing_range() {
    let snapshot = RunnerIntrospectionSnapshot {
        source_artifact_id: "source:uncapped-bounded-worlds.ash".to_string(),
        check_summary_id: "check:uncapped-bounded-world-summary".to_string(),
        small_world_domains: vec![SmallWorldDomain {
            id: "uncapped-int-worlds".to_string(),
            domain_kind: SmallWorldDomainKind::BoundedInt,
            source: TestSource::Policy,
            value_type: Some("Int".to_string()),
            bounds: BTreeMap::from([("min".to_string(), 0), ("max".to_string(), 50_000)]),
            oracle: Some(SmallWorldOracle {
                kind: SmallWorldOracleKind::BindingEquals,
                expected: json!({ "value": 0 }),
            }),
            ..SmallWorldDomain::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot_with_limits(
        Path::new("uncapped-bounded-worlds.ash"),
        &snapshot,
        None,
        None,
        None,
    );

    assert_eq!(
        results.len(),
        1,
        "uncapped bounded-int domains must not materialize every value"
    );
    assert_eq!(results[0].outcome, Outcome::Skip);
    assert!(
        results[0]
            .message
            .as_deref()
            .unwrap_or_default()
            .contains("deferred"),
        "uncapped bounded-int domains should defer with an explicit reason: {results:#?}"
    );
}

#[test]
fn bounded_product_domain_materializes_cartesian_world_bindings() {
    let snapshot = RunnerIntrospectionSnapshot {
        source_artifact_id: "source:product-worlds.ash".to_string(),
        check_summary_id: "check:product-world-summary".to_string(),
        small_world_domains: vec![SmallWorldDomain {
            id: "product-worlds".to_string(),
            domain_kind: SmallWorldDomainKind::Product,
            source: TestSource::Contract,
            product_axes: vec![
                SmallWorldProductAxis {
                    binding: "flag".to_string(),
                    values: vec![json!(false), json!(true)],
                },
                SmallWorldProductAxis {
                    binding: "level".to_string(),
                    values: vec![json!(1), json!(2)],
                },
            ],
            max_worlds_default: Some(4),
            oracle: Some(SmallWorldOracle {
                kind: SmallWorldOracleKind::TargetOutputEquals,
                expected: json!(true),
            }),
            executable_target: Some(smallworld_literal_target(json!(true))),
            ..SmallWorldDomain::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot(Path::new("product-worlds.ash"), &snapshot);

    assert_eq!(results.len(), 4);
    let bindings: Vec<_> = results
        .iter()
        .map(|result| {
            result
                .repro_artifact
                .as_ref()
                .and_then(|repro| repro.world_snapshot.as_ref())
                .map(|snapshot| snapshot["bindings"].clone())
                .expect("product worlds should include materialized bindings")
        })
        .collect();
    assert_eq!(
        bindings,
        vec![
            json!({ "flag": false, "level": 1 }),
            json!({ "flag": false, "level": 2 }),
            json!({ "flag": true, "level": 1 }),
            json!({ "flag": true, "level": 2 }),
        ]
    );
    assert!(results.iter().all(|result| result.outcome == Outcome::Pass));
}

#[test]
fn oversized_product_domain_defers_before_deep_axis_recursion() {
    let snapshot = RunnerIntrospectionSnapshot {
        source_artifact_id: "source:oversized-product-worlds.ash".to_string(),
        check_summary_id: "check:oversized-product-world-summary".to_string(),
        small_world_domains: vec![SmallWorldDomain {
            id: "oversized-product-worlds".to_string(),
            domain_kind: SmallWorldDomainKind::Product,
            source: TestSource::Contract,
            product_axes: (0..65)
                .map(|index| SmallWorldProductAxis {
                    binding: format!("axis_{index}"),
                    values: vec![json!(index)],
                })
                .collect(),
            max_worlds_default: Some(1),
            oracle: Some(SmallWorldOracle {
                kind: SmallWorldOracleKind::TargetOutputEquals,
                expected: json!(true),
            }),
            executable_target: Some(smallworld_literal_target(json!(true))),
            ..SmallWorldDomain::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot(Path::new("oversized-product-worlds.ash"), &snapshot);

    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].outcome,
        Outcome::Skip,
        "oversized product descriptors must defer before recursively walking every axis: {results:#?}"
    );
}

#[test]
fn bounded_list_domain_materializes_length_capped_lists() {
    let snapshot = RunnerIntrospectionSnapshot {
        source_artifact_id: "source:list-worlds.ash".to_string(),
        check_summary_id: "check:list-world-summary".to_string(),
        small_world_domains: vec![SmallWorldDomain {
            id: "list-worlds".to_string(),
            domain_kind: SmallWorldDomainKind::List,
            source: TestSource::Contract,
            list_descriptor: Some(SmallWorldListDescriptor {
                binding: "items".to_string(),
                elements: vec![json!(0), json!(1)],
                min_len: 0,
                max_len: Some(2),
            }),
            max_worlds_default: Some(4),
            oracle: Some(SmallWorldOracle {
                kind: SmallWorldOracleKind::TargetOutputEquals,
                expected: json!(true),
            }),
            executable_target: Some(smallworld_literal_target(json!(true))),
            ..SmallWorldDomain::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot(Path::new("list-worlds.ash"), &snapshot);

    assert_eq!(results.len(), 4);
    let lists: Vec<_> = results
        .iter()
        .map(|result| {
            result
                .repro_artifact
                .as_ref()
                .and_then(|repro| repro.world_snapshot.as_ref())
                .map(|snapshot| snapshot["bindings"]["items"].clone())
                .expect("list worlds should include materialized list binding")
        })
        .collect();
    assert_eq!(
        lists,
        vec![json!([]), json!([0]), json!([1]), json!([0, 0])]
    );
}

#[test]
fn oversized_bounded_list_domain_defers_before_deep_materialization() {
    let snapshot = RunnerIntrospectionSnapshot {
        source_artifact_id: "source:oversized-list-worlds.ash".to_string(),
        check_summary_id: "check:oversized-list-world-summary".to_string(),
        small_world_domains: vec![SmallWorldDomain {
            id: "oversized-list-worlds".to_string(),
            domain_kind: SmallWorldDomainKind::List,
            source: TestSource::Contract,
            list_descriptor: Some(SmallWorldListDescriptor {
                binding: "items".to_string(),
                elements: vec![json!(0)],
                min_len: 65,
                max_len: Some(65),
            }),
            max_worlds_default: Some(1),
            oracle: Some(SmallWorldOracle {
                kind: SmallWorldOracleKind::TargetOutputEquals,
                expected: json!(true),
            }),
            executable_target: Some(smallworld_literal_target(json!(true))),
            ..SmallWorldDomain::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot(Path::new("oversized-list-worlds.ash"), &snapshot);

    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].outcome,
        Outcome::Skip,
        "oversized list descriptors must defer before allocating or recursively materializing: {results:#?}"
    );
}

#[test]
fn policy_and_lifecycle_worlds_require_stable_explicit_ids() {
    let snapshot = RunnerIntrospectionSnapshot {
        source_artifact_id: "source:missing-id-worlds.ash".to_string(),
        check_summary_id: "check:missing-id-world-summary".to_string(),
        small_world_domains: vec![
            SmallWorldDomain {
                id: "policy-context-missing-id".to_string(),
                domain_kind: SmallWorldDomainKind::PolicyContext,
                source: TestSource::Policy,
                policy_context_descriptor: Some(SmallWorldPolicyContextDescriptor {
                    policies: vec!["review_policy".to_string()],
                    contexts: vec![SmallWorldPolicyContext {
                        id: String::new(),
                        roles: vec!["reviewer".to_string()],
                        capabilities: Vec::new(),
                        bindings: BTreeMap::from([("smallworld_ok".to_string(), json!(true))]),
                        control_state: Some("allow".to_string()),
                    }],
                }),
                max_worlds_default: Some(1),
                oracle: Some(SmallWorldOracle {
                    kind: SmallWorldOracleKind::TargetOutputEquals,
                    expected: json!(true),
                }),
                executable_target: Some(smallworld_expr_target(core_var("smallworld_ok"))),
                ..SmallWorldDomain::default()
            },
            SmallWorldDomain {
                id: "lifecycle-missing-id".to_string(),
                domain_kind: SmallWorldDomainKind::ObligationLifecycle,
                source: TestSource::Obligation,
                lifecycle_descriptor: Some(SmallWorldLifecycleDescriptor {
                    obligation: "Ticket".to_string(),
                    states: vec![SmallWorldLifecycleStateDescriptor {
                        id: String::new(),
                        terminal: ObligationTerminalExpectation::Discharged,
                        transition_trace: vec!["introduce:Ticket".to_string()],
                    }],
                }),
                max_worlds_default: Some(1),
                oracle: Some(SmallWorldOracle {
                    kind: SmallWorldOracleKind::TargetOutputEquals,
                    expected: json!(true),
                }),
                executable_target: Some(smallworld_literal_target(json!(true))),
                ..SmallWorldDomain::default()
            },
        ],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot(Path::new("missing-id-worlds.ash"), &snapshot);

    assert_eq!(results.len(), 2);
    assert!(
        results.iter().all(|result| result.outcome == Outcome::Skip),
        "policy/lifecycle worlds without stable explicit IDs must defer instead of receiving fallback IDs: {results:#?}"
    );
}

#[test]
fn role_capability_inclusion_domain_materializes_explicit_finite_sets() {
    let snapshot = RunnerIntrospectionSnapshot {
        source_artifact_id: "source:inclusion-worlds.ash".to_string(),
        check_summary_id: "check:inclusion-world-summary".to_string(),
        small_world_domains: vec![SmallWorldDomain {
            id: "role-capability-worlds".to_string(),
            domain_kind: SmallWorldDomainKind::RoleCapabilityInclusionSet,
            source: TestSource::Policy,
            inclusion_descriptor: Some(SmallWorldInclusionSetDescriptor {
                roles: vec!["author".to_string(), "reviewer".to_string()],
                capabilities: vec!["read".to_string()],
            }),
            max_worlds_default: Some(5),
            oracle: Some(SmallWorldOracle {
                kind: SmallWorldOracleKind::TargetOutputEquals,
                expected: json!(true),
            }),
            executable_target: Some(smallworld_literal_target(json!(true))),
            ..SmallWorldDomain::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot(Path::new("inclusion-worlds.ash"), &snapshot);

    assert_eq!(results.len(), 5);
    let snapshots: Vec<_> = results
        .iter()
        .map(|result| {
            result
                .repro_artifact
                .as_ref()
                .and_then(|repro| repro.world_snapshot.as_ref())
                .cloned()
                .expect("inclusion worlds should include world snapshots")
        })
        .collect();
    assert_eq!(snapshots[0]["roles"], json!([]));
    assert_eq!(snapshots[0]["capabilities"], json!([]));
    assert_eq!(snapshots[1]["roles"], json!(["author"]));
    assert_eq!(snapshots[4]["capabilities"], json!(["read"]));
    assert!(results.iter().all(|result| result.outcome == Outcome::Pass));
}

#[test]
fn policy_context_domain_materializes_stable_context_descriptors() {
    let snapshot = RunnerIntrospectionSnapshot {
        source_artifact_id: "source:policy-context-worlds.ash".to_string(),
        check_summary_id: "check:policy-context-world-summary".to_string(),
        small_world_domains: vec![SmallWorldDomain {
            id: "policy-context-worlds".to_string(),
            domain_kind: SmallWorldDomainKind::PolicyContext,
            source: TestSource::Policy,
            policy_context_descriptor: Some(SmallWorldPolicyContextDescriptor {
                policies: vec!["review_policy".to_string()],
                contexts: vec![
                    SmallWorldPolicyContext {
                        id: "allowed-reviewer".to_string(),
                        roles: vec!["reviewer".to_string()],
                        capabilities: vec!["review".to_string()],
                        bindings: BTreeMap::from([("smallworld_ok".to_string(), json!(true))]),
                        control_state: Some("allow".to_string()),
                    },
                    SmallWorldPolicyContext {
                        id: "denied-author".to_string(),
                        roles: vec!["author".to_string()],
                        capabilities: Vec::new(),
                        bindings: BTreeMap::from([("smallworld_ok".to_string(), json!(false))]),
                        control_state: Some("deny".to_string()),
                    },
                ],
            }),
            max_worlds_default: Some(2),
            oracle: Some(SmallWorldOracle {
                kind: SmallWorldOracleKind::TargetOutputEquals,
                expected: json!(true),
            }),
            executable_target: Some(smallworld_expr_target(core_var("smallworld_ok"))),
            ..SmallWorldDomain::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot(Path::new("policy-context-worlds.ash"), &snapshot);

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].outcome, Outcome::Pass);
    assert_eq!(results[1].outcome, Outcome::Fail);
    let repro = results[0]
        .repro_artifact
        .as_ref()
        .and_then(|repro| repro.world_snapshot.as_ref())
        .expect("policy-context worlds should include materialized context snapshots");
    assert_eq!(repro["policies"], json!(["review_policy"]));
    assert_eq!(repro["roles"], json!(["reviewer"]));
    assert_eq!(repro["capabilities"], json!(["review"]));
}

#[test]
fn obligation_lifecycle_domain_materializes_stable_state_machine_descriptors() {
    let snapshot = RunnerIntrospectionSnapshot {
        source_artifact_id: "source:lifecycle-worlds.ash".to_string(),
        check_summary_id: "check:lifecycle-world-summary".to_string(),
        small_world_domains: vec![SmallWorldDomain {
            id: "obligation-lifecycle-worlds".to_string(),
            domain_kind: SmallWorldDomainKind::ObligationLifecycle,
            source: TestSource::Obligation,
            lifecycle_descriptor: Some(SmallWorldLifecycleDescriptor {
                obligation: "Ticket".to_string(),
                states: vec![
                    SmallWorldLifecycleStateDescriptor {
                        id: "introduced".to_string(),
                        terminal: ObligationTerminalExpectation::Introduced,
                        transition_trace: vec!["introduce:Ticket".to_string()],
                    },
                    SmallWorldLifecycleStateDescriptor {
                        id: "discharged".to_string(),
                        terminal: ObligationTerminalExpectation::Discharged,
                        transition_trace: vec![
                            "introduce:Ticket".to_string(),
                            "discharge:Ticket".to_string(),
                        ],
                    },
                ],
            }),
            max_worlds_default: Some(2),
            oracle: Some(SmallWorldOracle {
                kind: SmallWorldOracleKind::TargetOutputEquals,
                expected: json!(true),
            }),
            executable_target: Some(smallworld_literal_target(json!(true))),
            ..SmallWorldDomain::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot(Path::new("lifecycle-worlds.ash"), &snapshot);

    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|result| result.outcome == Outcome::Pass));
    let discharged = results[1]
        .repro_artifact
        .as_ref()
        .and_then(|repro| repro.world_snapshot.as_ref())
        .expect("lifecycle worlds should include materialized state snapshots");
    assert_eq!(discharged["control_state"], json!("discharged"));
    assert_eq!(discharged["obligations"], json!(["Ticket"]));
    assert_eq!(
        discharged["transition_trace"],
        json!(["introduce:Ticket", "discharge:Ticket"])
    );
}

#[test]
fn uncapped_or_open_richer_domains_defer_before_materialization() {
    let snapshot = RunnerIntrospectionSnapshot {
        source_artifact_id: "source:open-worlds.ash".to_string(),
        check_summary_id: "check:open-world-summary".to_string(),
        small_world_domains: vec![
            SmallWorldDomain {
                id: "uncapped-product".to_string(),
                domain_kind: SmallWorldDomainKind::Product,
                source: TestSource::Contract,
                product_axes: vec![SmallWorldProductAxis {
                    binding: "value".to_string(),
                    values: vec![json!(1), json!(2)],
                }],
                oracle: Some(SmallWorldOracle {
                    kind: SmallWorldOracleKind::TargetOutputEquals,
                    expected: json!(true),
                }),
                executable_target: Some(smallworld_literal_target(json!(true))),
                ..SmallWorldDomain::default()
            },
            SmallWorldDomain {
                id: "open-list".to_string(),
                domain_kind: SmallWorldDomainKind::List,
                source: TestSource::Contract,
                list_descriptor: Some(SmallWorldListDescriptor {
                    binding: "items".to_string(),
                    elements: vec![json!(1)],
                    min_len: 0,
                    max_len: None,
                }),
                max_worlds_default: Some(4),
                oracle: Some(SmallWorldOracle {
                    kind: SmallWorldOracleKind::TargetOutputEquals,
                    expected: json!(true),
                }),
                executable_target: Some(smallworld_literal_target(json!(true))),
                ..SmallWorldDomain::default()
            },
            SmallWorldDomain {
                id: "open-inclusion".to_string(),
                domain_kind: SmallWorldDomainKind::RoleCapabilityInclusionSet,
                source: TestSource::Policy,
                inclusion_descriptor: Some(SmallWorldInclusionSetDescriptor {
                    roles: Vec::new(),
                    capabilities: Vec::new(),
                }),
                max_worlds_default: Some(4),
                oracle: Some(SmallWorldOracle {
                    kind: SmallWorldOracleKind::TargetOutputEquals,
                    expected: json!(true),
                }),
                executable_target: Some(smallworld_literal_target(json!(true))),
                ..SmallWorldDomain::default()
            },
        ],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot_with_limits(
        Path::new("open-worlds.ash"),
        &snapshot,
        None,
        None,
        None,
    );

    assert_eq!(results.len(), 3);
    assert!(
        results.iter().all(|result| result.outcome == Outcome::Skip),
        "uncapped/open richer domains should defer instead of materializing worlds: {results:#?}"
    );
    assert!(
        results.iter().all(|result| {
            result
                .message
                .as_deref()
                .unwrap_or_default()
                .contains("deferred")
        }),
        "deferred richer domains should report fail-closed reasons: {results:#?}"
    );
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
                    bindings: BTreeMap::from([("smallworld_ok".to_string(), json!(true))]),
                    ..SmallWorldState::default()
                },
                SmallWorldState {
                    id: "denied".to_string(),
                    world_kind: "policy_context".to_string(),
                    control_state: Some("denied".to_string()),
                    bindings: BTreeMap::from([("smallworld_ok".to_string(), json!(false))]),
                    ..SmallWorldState::default()
                },
            ],
            oracle: Some(SmallWorldOracle {
                kind: SmallWorldOracleKind::TargetOutputEquals,
                expected: json!(true),
            }),
            executable_target: Some(smallworld_expr_target(core_var("smallworld_ok"))),
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
                    json!({ "decision": "allow" }),
                    json!({ "decision": "deny" }),
                ],
                ..TypeGeneratorDescriptor::default()
            }],
            lowered_policy_ref: Some("policy:review:terminal".to_string()),
            supported_terminal_outcomes: vec![
                PolicyTerminalOutcome::Allow,
                PolicyTerminalOutcome::Deny,
            ],
            oracle_shape: Some(PolicyOracleShape::TerminalEquals),
            executable_target: Some(PolicyExecutableTarget {
                kind: PolicyExecutableTargetKind::TerminalOracle,
                target_ref: "policy:review:terminal".to_string(),
                authority_setup: PolicyAuthoritySetup::NoAuthorityRequired,
                terminal_oracle: PolicyTerminalOracle::ExactMatchTable {
                    input_binding: "policy_input".to_string(),
                    rows: vec![
                        PolicyTerminalOracleRow {
                            when: BTreeMap::from([("decision".to_string(), json!("allow"))]),
                            terminal: PolicyTerminalOutcome::Allow,
                        },
                        PolicyTerminalOracleRow {
                            when: BTreeMap::from([("decision".to_string(), json!("deny"))]),
                            terminal: PolicyTerminalOutcome::Deny,
                        },
                    ],
                },
            }),
            ..RunnerPolicyMetadata::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot(Path::new("policy.ash"), &snapshot);

    assert_eq!(results.len(), 2);
    assert!(
        results
            .iter()
            .all(|result| result.source == TestSource::Policy && result.outcome == Outcome::Pass),
        "terminal-equals policy metadata should execute narrow allow/deny cases: {results:#?}"
    );
}

#[test]
fn structured_policy_terminal_oracle_evaluates_input_fields_instead_of_terminal_metadata() {
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
                    json!({ "subject": "admin", "terminal": "deny" }),
                    json!({ "subject": "guest", "terminal": "allow" }),
                ],
                ..TypeGeneratorDescriptor::default()
            }],
            lowered_policy_ref: Some("policy:review:terminal".to_string()),
            supported_terminal_outcomes: vec![
                PolicyTerminalOutcome::Allow,
                PolicyTerminalOutcome::Deny,
            ],
            oracle_shape: Some(PolicyOracleShape::TerminalEquals),
            executable_target: Some(PolicyExecutableTarget {
                kind: PolicyExecutableTargetKind::TerminalOracle,
                target_ref: "policy:review:terminal".to_string(),
                authority_setup: PolicyAuthoritySetup::NoAuthorityRequired,
                terminal_oracle: PolicyTerminalOracle::ExactMatchTable {
                    input_binding: "policy_input".to_string(),
                    rows: vec![
                        PolicyTerminalOracleRow {
                            when: BTreeMap::from([("subject".to_string(), json!("admin"))]),
                            terminal: PolicyTerminalOutcome::Allow,
                        },
                        PolicyTerminalOracleRow {
                            when: BTreeMap::from([("subject".to_string(), json!("guest"))]),
                            terminal: PolicyTerminalOutcome::Deny,
                        },
                    ],
                },
            }),
            ..RunnerPolicyMetadata::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot(Path::new("policy.ash"), &snapshot);

    assert_eq!(results.len(), 2);
    assert!(
        results
            .iter()
            .all(|result| result.source == TestSource::Policy && result.outcome == Outcome::Pass),
        "exact-match policy oracle should execute supported allow/deny cases: {results:#?}"
    );
    let allow = results
        .iter()
        .find(|result| result.name.contains("terminal-allow"))
        .expect("allow case should be generated from evaluated oracle");
    let repro = allow
        .repro_artifact
        .as_ref()
        .expect("executed policy case should include repro artifact");
    assert_eq!(
        repro.generated_input_snapshot.as_ref().unwrap()["bindings"]["policy_input"]["subject"],
        json!("admin"),
        "allow case must come from evaluated oracle metadata, not the input terminal field"
    );
    assert_eq!(
        repro.oracle_snapshot["target_execution"]["substrate"],
        json!("finite_policy_terminal_oracle")
    );
    assert_eq!(repro.oracle_snapshot["expected_terminal"], json!("allow"));
    assert_eq!(repro.oracle_snapshot["actual_terminal"], json!("allow"));
}

#[test]
fn policy_terminal_expected_mismatch_fails_even_if_input_terminal_matches_expected() {
    let mut bindings = BTreeMap::new();
    bindings.insert(
        "policy_input".to_string(),
        json!({ "subject": "guest", "terminal": "allow" }),
    );
    let case = SynthesizedCase {
        id: "synthesized/policy/review/terminal-allow-mismatch".to_string(),
        source: TestSource::Policy,
        target_kind: "policy".to_string(),
        target_name: "ReviewPolicy".to_string(),
        file_path: Path::new("policy.ash").to_path_buf(),
        tags: vec!["synthesized".to_string(), "policy".to_string()],
        seed: 0,
        inputs: SynthesizedInputs {
            bindings,
            generated_from: "exact_policy_input_domain".to_string(),
            case_index: 1,
            world_index: None,
        },
        oracle: SynthesizedOracle::PolicyTerminalEquals {
            expected: PolicyTerminalOutcome::Allow,
            policy_ref: "policy:review:terminal".to_string(),
            terminal_oracle: PolicyTerminalOracle::ExactMatchTable {
                input_binding: "policy_input".to_string(),
                rows: vec![PolicyTerminalOracleRow {
                    when: BTreeMap::from([("subject".to_string(), json!("guest"))]),
                    terminal: PolicyTerminalOutcome::Deny,
                }],
            },
        },
        repro: repro_artifact(
            Path::new("policy.ash"),
            "source:policy.ash".to_string(),
            "check:summary".to_string(),
            "synthesized/policy/review/terminal-allow-mismatch".to_string(),
            0,
            1,
            Some(json!({
                "bindings": {
                    "policy_input": { "subject": "guest", "terminal": "allow" }
                },
                "generated_from": "exact_policy_input_domain",
            })),
            json!({
                "kind": "policy_terminal_equals",
                "policy_ref": "policy:review:terminal",
                "expected_terminal": "allow",
                "actual_terminal": "deny",
                "target_execution": {
                    "substrate": "finite_policy_terminal_oracle",
                },
            }),
            None,
        ),
    };

    let result = execute_synthesized_case(&case);

    assert_eq!(
        result.outcome,
        Outcome::Fail,
        "policy execution must fail on evaluated terminal mismatch, even when input terminal metadata matches the expectation"
    );
}

#[test]
fn policy_with_empty_executable_target_ref_defers() {
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
                exact_values: vec![json!({ "subject": "admin" })],
                ..TypeGeneratorDescriptor::default()
            }],
            lowered_policy_ref: Some("policy:review:terminal".to_string()),
            supported_terminal_outcomes: vec![PolicyTerminalOutcome::Allow],
            oracle_shape: Some(PolicyOracleShape::TerminalEquals),
            executable_target: Some(PolicyExecutableTarget {
                kind: PolicyExecutableTargetKind::TerminalOracle,
                target_ref: String::new(),
                authority_setup: PolicyAuthoritySetup::NoAuthorityRequired,
                terminal_oracle: PolicyTerminalOracle::ExactMatchTable {
                    input_binding: "policy_input".to_string(),
                    rows: vec![PolicyTerminalOracleRow {
                        when: BTreeMap::from([("subject".to_string(), json!("admin"))]),
                        terminal: PolicyTerminalOutcome::Allow,
                    }],
                },
            }),
            ..RunnerPolicyMetadata::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot(Path::new("policy.ash"), &snapshot);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].outcome, Outcome::Skip);
    assert!(
        results[0]
            .message
            .as_deref()
            .unwrap_or_default()
            .contains("target_ref"),
        "missing executable target_ref must defer instead of passing: {results:#?}"
    );
}

#[test]
fn policy_with_mismatched_executable_target_ref_defers() {
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
                exact_values: vec![json!({ "subject": "admin" })],
                ..TypeGeneratorDescriptor::default()
            }],
            lowered_policy_ref: Some("policy:review:terminal".to_string()),
            supported_terminal_outcomes: vec![PolicyTerminalOutcome::Allow],
            oracle_shape: Some(PolicyOracleShape::TerminalEquals),
            executable_target: Some(PolicyExecutableTarget {
                kind: PolicyExecutableTargetKind::TerminalOracle,
                target_ref: "policy:other:terminal".to_string(),
                authority_setup: PolicyAuthoritySetup::NoAuthorityRequired,
                terminal_oracle: PolicyTerminalOracle::ExactMatchTable {
                    input_binding: "policy_input".to_string(),
                    rows: vec![PolicyTerminalOracleRow {
                        when: BTreeMap::from([("subject".to_string(), json!("admin"))]),
                        terminal: PolicyTerminalOutcome::Allow,
                    }],
                },
            }),
            ..RunnerPolicyMetadata::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot(Path::new("policy.ash"), &snapshot);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].outcome, Outcome::Skip);
    assert!(
        results[0]
            .message
            .as_deref()
            .unwrap_or_default()
            .contains("does not match lowered policy ref"),
        "mismatched executable target_ref must defer instead of passing: {results:#?}"
    );
}

#[test]
fn policy_with_required_authority_without_explicit_setup_defers() {
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
                exact_values: vec![json!({ "subject": "admin" })],
                ..TypeGeneratorDescriptor::default()
            }],
            lowered_policy_ref: Some("policy:review:terminal".to_string()),
            supported_terminal_outcomes: vec![PolicyTerminalOutcome::Allow],
            oracle_shape: Some(PolicyOracleShape::TerminalEquals),
            required_authority: Some("role:reviewer".to_string()),
            executable_target: Some(PolicyExecutableTarget {
                kind: PolicyExecutableTargetKind::TerminalOracle,
                target_ref: "policy:review:terminal".to_string(),
                authority_setup: PolicyAuthoritySetup::Missing,
                terminal_oracle: PolicyTerminalOracle::ExactMatchTable {
                    input_binding: "policy_input".to_string(),
                    rows: vec![PolicyTerminalOracleRow {
                        when: BTreeMap::from([("subject".to_string(), json!("admin"))]),
                        terminal: PolicyTerminalOutcome::Allow,
                    }],
                },
            }),
            ..RunnerPolicyMetadata::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot(Path::new("policy.ash"), &snapshot);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].outcome, Outcome::Skip);
    assert!(
        results[0]
            .message
            .as_deref()
            .unwrap_or_default()
            .contains("authority"),
        "missing explicit authority setup must defer instead of passing: {results:#?}"
    );
}

#[test]
fn policy_with_required_authority_and_matching_explicit_setup_executes() {
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
                exact_values: vec![json!({ "subject": "admin" })],
                ..TypeGeneratorDescriptor::default()
            }],
            lowered_policy_ref: Some("policy:review:terminal".to_string()),
            supported_terminal_outcomes: vec![PolicyTerminalOutcome::Allow],
            oracle_shape: Some(PolicyOracleShape::TerminalEquals),
            required_authority: Some("role:reviewer".to_string()),
            executable_target: Some(PolicyExecutableTarget {
                kind: PolicyExecutableTargetKind::TerminalOracle,
                target_ref: "policy:review:terminal".to_string(),
                authority_setup: PolicyAuthoritySetup::ExplicitAuthority {
                    authority: "role:reviewer".to_string(),
                },
                terminal_oracle: PolicyTerminalOracle::ExactMatchTable {
                    input_binding: "policy_input".to_string(),
                    rows: vec![PolicyTerminalOracleRow {
                        when: BTreeMap::from([("subject".to_string(), json!("admin"))]),
                        terminal: PolicyTerminalOutcome::Allow,
                    }],
                },
            }),
            ..RunnerPolicyMetadata::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot(Path::new("policy.ash"), &snapshot);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].outcome, Outcome::Pass);
    let repro = results[0]
        .repro_artifact
        .as_ref()
        .expect("executed authority-backed policy should include repro");
    assert_eq!(
        repro.oracle_snapshot["target"]["authority_setup"]["explicit_authority"]["authority"],
        json!("role:reviewer")
    );
}

#[test]
fn policy_approval_and_transform_terminals_defer_without_stable_exact_oracle_slice() {
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
                exact_values: vec![json!({ "subject": "manager" })],
                ..TypeGeneratorDescriptor::default()
            }],
            lowered_policy_ref: Some("policy:review:terminal".to_string()),
            supported_terminal_outcomes: vec![
                PolicyTerminalOutcome::Approval,
                PolicyTerminalOutcome::Transform,
            ],
            oracle_shape: Some(PolicyOracleShape::TerminalEquals),
            executable_target: Some(PolicyExecutableTarget {
                kind: PolicyExecutableTargetKind::TerminalOracle,
                target_ref: "policy:review:terminal".to_string(),
                authority_setup: PolicyAuthoritySetup::NoAuthorityRequired,
                terminal_oracle: PolicyTerminalOracle::ExactMatchTable {
                    input_binding: "policy_input".to_string(),
                    rows: vec![PolicyTerminalOracleRow {
                        when: BTreeMap::from([("subject".to_string(), json!("manager"))]),
                        terminal: PolicyTerminalOutcome::Approval,
                    }],
                },
            }),
            ..RunnerPolicyMetadata::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot(Path::new("policy.ash"), &snapshot);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].outcome, Outcome::Skip);
    assert!(
        results[0]
            .message
            .as_deref()
            .unwrap_or_default()
            .contains("allow/deny"),
        "approval/transform terminals should defer until a stable exact oracle slice exists: {results:#?}"
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
            required_closeout_behavior: Some("reject_if_open".to_string()),
            terminal_expectations: vec![
                ObligationTerminalExpectation::Introduced,
                ObligationTerminalExpectation::Discharged,
                ObligationTerminalExpectation::MissingDischargeRejected,
                ObligationTerminalExpectation::DoubleDischargeRejected,
            ],
            lifecycle_transition_plan: Some(ObligationLifecycleTransitionPlan {
                model: ObligationLifecycleModelKind::IntroduceDischargeCheck,
                introduction_sites: vec!["open_ticket".to_string()],
                discharge_sites: vec!["close_ticket".to_string()],
                check_sites: vec!["finish".to_string()],
                required_closeout: ObligationCloseoutBehavior::RejectIfOpen,
            }),
            lifecycle_transition_traces: vec![
                ObligationLifecycleTransitionTrace {
                    id: "ticket:introduced".to_string(),
                    transitions: vec![ObligationLifecycleTransition::Introduce {
                        site: "open_ticket".to_string(),
                    }],
                },
                ObligationLifecycleTransitionTrace {
                    id: "ticket:discharged".to_string(),
                    transitions: vec![
                        ObligationLifecycleTransition::Introduce {
                            site: "open_ticket".to_string(),
                        },
                        ObligationLifecycleTransition::Discharge {
                            site: "close_ticket".to_string(),
                        },
                        ObligationLifecycleTransition::Check {
                            site: "finish".to_string(),
                        },
                    ],
                },
                ObligationLifecycleTransitionTrace {
                    id: "ticket:missing-discharge".to_string(),
                    transitions: vec![
                        ObligationLifecycleTransition::Introduce {
                            site: "open_ticket".to_string(),
                        },
                        ObligationLifecycleTransition::Check {
                            site: "finish".to_string(),
                        },
                        ObligationLifecycleTransition::Reject {
                            reason: ObligationLifecycleRejection::MissingDischarge,
                        },
                    ],
                },
                ObligationLifecycleTransitionTrace {
                    id: "ticket:double-discharge".to_string(),
                    transitions: vec![
                        ObligationLifecycleTransition::Introduce {
                            site: "open_ticket".to_string(),
                        },
                        ObligationLifecycleTransition::Discharge {
                            site: "close_ticket".to_string(),
                        },
                        ObligationLifecycleTransition::Discharge {
                            site: "close_ticket".to_string(),
                        },
                        ObligationLifecycleTransition::Reject {
                            reason: ObligationLifecycleRejection::DoubleDischarge,
                        },
                    ],
                },
            ],
            lifecycle_worlds: vec![
                SmallWorldState {
                    id: "ticket:introduced".to_string(),
                    schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
                    world_kind: "obligation_lifecycle".to_string(),
                    obligations: vec!["Ticket".to_string()],
                    control_state: Some("introduced".to_string()),
                    transition_trace: vec!["introduce:open_ticket".to_string()],
                    ..SmallWorldState::default()
                },
                SmallWorldState {
                    id: "ticket:discharged".to_string(),
                    schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
                    world_kind: "obligation_lifecycle".to_string(),
                    obligations: vec!["Ticket".to_string()],
                    control_state: Some("discharged".to_string()),
                    transition_trace: vec![
                        "introduce:open_ticket".to_string(),
                        "discharge:close_ticket".to_string(),
                        "check:finish".to_string(),
                    ],
                    ..SmallWorldState::default()
                },
                SmallWorldState {
                    id: "ticket:missing-discharge".to_string(),
                    schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
                    world_kind: "obligation_lifecycle".to_string(),
                    obligations: vec!["Ticket".to_string()],
                    control_state: Some("rejected".to_string()),
                    transition_trace: vec![
                        "introduce:open_ticket".to_string(),
                        "check:finish".to_string(),
                        "reject:missing_discharge".to_string(),
                    ],
                    ..SmallWorldState::default()
                },
                SmallWorldState {
                    id: "ticket:double-discharge".to_string(),
                    schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
                    world_kind: "obligation_lifecycle".to_string(),
                    obligations: vec!["Ticket".to_string()],
                    control_state: Some("rejected".to_string()),
                    transition_trace: vec![
                        "introduce:open_ticket".to_string(),
                        "discharge:close_ticket".to_string(),
                        "discharge:close_ticket".to_string(),
                        "reject:double_discharge".to_string(),
                    ],
                    ..SmallWorldState::default()
                },
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
fn obligation_lifecycle_requires_typed_transition_execution_not_claimed_world_state() {
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
            required_closeout_behavior: Some("reject_if_open".to_string()),
            terminal_expectations: vec![ObligationTerminalExpectation::Discharged],
            lifecycle_transition_plan: Some(ObligationLifecycleTransitionPlan {
                model: ObligationLifecycleModelKind::IntroduceDischargeCheck,
                introduction_sites: vec!["open_ticket".to_string()],
                discharge_sites: vec!["close_ticket".to_string()],
                check_sites: vec!["finish".to_string()],
                required_closeout: ObligationCloseoutBehavior::RejectIfOpen,
            }),
            lifecycle_transition_traces: vec![ObligationLifecycleTransitionTrace {
                id: "ticket:claimed-discharged-but-only-introduced".to_string(),
                transitions: vec![ObligationLifecycleTransition::Introduce {
                    site: "open_ticket".to_string(),
                }],
            }],
            lifecycle_worlds: vec![SmallWorldState {
                id: "ticket:claimed-discharged".to_string(),
                schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
                world_kind: "obligation_lifecycle".to_string(),
                obligations: vec!["Ticket".to_string()],
                control_state: Some("discharged".to_string()),
                transition_trace: vec!["introduce:open_ticket".to_string()],
                ..SmallWorldState::default()
            }],
            ..RunnerObligationMetadata::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot(Path::new("obligation.ash"), &snapshot);

    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].outcome,
        Outcome::Fail,
        "claimed lifecycle_worlds.control_state must not pass without matching typed transition execution: {results:#?}"
    );
    let oracle_snapshot = results[0]
        .repro_artifact
        .as_ref()
        .and_then(|repro| repro.oracle_snapshot.as_object())
        .expect("obligation execution repro should include oracle snapshot");
    assert_eq!(
        oracle_snapshot
            .get("execution_substrate")
            .and_then(Value::as_str),
        Some("typed_lifecycle_transition_plan")
    );
    assert_eq!(
        oracle_snapshot
            .get("actual_executed_terminal")
            .and_then(|terminal| terminal.get("control_state"))
            .and_then(Value::as_str),
        Some("introduced")
    );
}

#[test]
fn obligation_lifecycle_missing_typed_transition_trace_defers_even_when_world_state_matches() {
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
            required_closeout_behavior: Some("reject_if_open".to_string()),
            terminal_expectations: vec![ObligationTerminalExpectation::Discharged],
            lifecycle_transition_plan: Some(ObligationLifecycleTransitionPlan {
                model: ObligationLifecycleModelKind::IntroduceDischargeCheck,
                introduction_sites: vec!["open_ticket".to_string()],
                discharge_sites: vec!["close_ticket".to_string()],
                check_sites: vec!["finish".to_string()],
                required_closeout: ObligationCloseoutBehavior::RejectIfOpen,
            }),
            lifecycle_worlds: vec![SmallWorldState {
                id: "ticket:discharged".to_string(),
                schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
                world_kind: "obligation_lifecycle".to_string(),
                obligations: vec!["Ticket".to_string()],
                control_state: Some("discharged".to_string()),
                transition_trace: vec![
                    "introduce:open_ticket".to_string(),
                    "discharge:close_ticket".to_string(),
                    "check:finish".to_string(),
                ],
                ..SmallWorldState::default()
            }],
            ..RunnerObligationMetadata::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot(Path::new("obligation.ash"), &snapshot);

    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].outcome,
        Outcome::Skip,
        "typed transition traces are required; matching world control_state alone must defer: {results:#?}"
    );
}

#[test]
fn obligation_lifecycle_missing_required_closeout_behavior_defers() {
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
            terminal_expectations: vec![ObligationTerminalExpectation::Discharged],
            lifecycle_transition_plan: Some(ObligationLifecycleTransitionPlan {
                model: ObligationLifecycleModelKind::IntroduceDischargeCheck,
                introduction_sites: vec!["open_ticket".to_string()],
                discharge_sites: vec!["close_ticket".to_string()],
                check_sites: vec!["finish".to_string()],
                required_closeout: ObligationCloseoutBehavior::RejectIfOpen,
            }),
            lifecycle_transition_traces: vec![ObligationLifecycleTransitionTrace {
                id: "ticket:discharged".to_string(),
                transitions: vec![
                    ObligationLifecycleTransition::Introduce {
                        site: "open_ticket".to_string(),
                    },
                    ObligationLifecycleTransition::Discharge {
                        site: "close_ticket".to_string(),
                    },
                ],
            }],
            lifecycle_worlds: vec![SmallWorldState {
                id: "ticket:discharged".to_string(),
                schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
                world_kind: "obligation_lifecycle".to_string(),
                obligations: vec!["Ticket".to_string()],
                control_state: Some("discharged".to_string()),
                transition_trace: vec![
                    "introduce:open_ticket".to_string(),
                    "discharge:close_ticket".to_string(),
                ],
                ..SmallWorldState::default()
            }],
            ..RunnerObligationMetadata::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot(Path::new("obligation.ash"), &snapshot);

    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].outcome,
        Outcome::Skip,
        "required closeout behavior is mandatory for runtime-backed obligation lifecycle execution: {results:#?}"
    );
}

#[test]
fn obligation_lifecycle_unsupported_model_defers() {
    let snapshot = RunnerIntrospectionSnapshot {
        source_artifact_id: "source:obligation.ash".to_string(),
        check_summary_id: "check:summary".to_string(),
        obligations: vec![RunnerObligationMetadata {
            id: "obligation:ticket".to_string(),
            obligation_name: "Ticket".to_string(),
            scope: "workflow".to_string(),
            lifecycle_model: Some("unsupported-model".to_string()),
            introduction_sites: vec!["open_ticket".to_string()],
            discharge_sites: vec!["close_ticket".to_string()],
            check_sites: vec!["finish".to_string()],
            required_closeout_behavior: Some("reject_if_open".to_string()),
            terminal_expectations: vec![ObligationTerminalExpectation::Discharged],
            lifecycle_transition_plan: Some(ObligationLifecycleTransitionPlan {
                model: ObligationLifecycleModelKind::IntroduceDischargeCheck,
                introduction_sites: vec!["open_ticket".to_string()],
                discharge_sites: vec!["close_ticket".to_string()],
                check_sites: vec!["finish".to_string()],
                required_closeout: ObligationCloseoutBehavior::RejectIfOpen,
            }),
            lifecycle_transition_traces: vec![ObligationLifecycleTransitionTrace {
                id: "ticket:discharged".to_string(),
                transitions: vec![
                    ObligationLifecycleTransition::Introduce {
                        site: "open_ticket".to_string(),
                    },
                    ObligationLifecycleTransition::Discharge {
                        site: "close_ticket".to_string(),
                    },
                    ObligationLifecycleTransition::Check {
                        site: "finish".to_string(),
                    },
                ],
            }],
            lifecycle_worlds: vec![SmallWorldState {
                id: "ticket:discharged".to_string(),
                schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
                world_kind: "obligation_lifecycle".to_string(),
                obligations: vec!["Ticket".to_string()],
                control_state: Some("discharged".to_string()),
                transition_trace: vec![
                    "introduce:open_ticket".to_string(),
                    "discharge:close_ticket".to_string(),
                    "check:finish".to_string(),
                ],
                ..SmallWorldState::default()
            }],
            ..RunnerObligationMetadata::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot(Path::new("obligation.ash"), &snapshot);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].outcome, Outcome::Skip);
    assert!(
        results[0]
            .message
            .as_deref()
            .unwrap_or_default()
            .contains("deferred"),
        "unsupported lifecycle_model must defer instead of passing: {results:#?}"
    );
}

#[test]
fn obligation_lifecycle_non_lifecycle_world_defers() {
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
            required_closeout_behavior: Some("reject_if_open".to_string()),
            terminal_expectations: vec![ObligationTerminalExpectation::Discharged],
            lifecycle_transition_plan: Some(ObligationLifecycleTransitionPlan {
                model: ObligationLifecycleModelKind::IntroduceDischargeCheck,
                introduction_sites: vec!["open_ticket".to_string()],
                discharge_sites: vec!["close_ticket".to_string()],
                check_sites: vec!["finish".to_string()],
                required_closeout: ObligationCloseoutBehavior::RejectIfOpen,
            }),
            lifecycle_transition_traces: vec![ObligationLifecycleTransitionTrace {
                id: "ticket:discharged".to_string(),
                transitions: vec![
                    ObligationLifecycleTransition::Introduce {
                        site: "open_ticket".to_string(),
                    },
                    ObligationLifecycleTransition::Discharge {
                        site: "close_ticket".to_string(),
                    },
                    ObligationLifecycleTransition::Check {
                        site: "finish".to_string(),
                    },
                ],
            }],
            lifecycle_worlds: vec![SmallWorldState {
                id: "not-a-lifecycle-world".to_string(),
                schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
                world_kind: "generic".to_string(),
                control_state: Some("discharged".to_string()),
                transition_trace: vec![
                    "introduce:open_ticket".to_string(),
                    "discharge:close_ticket".to_string(),
                    "check:finish".to_string(),
                ],
                ..SmallWorldState::default()
            }],
            ..RunnerObligationMetadata::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot(Path::new("obligation.ash"), &snapshot);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].outcome, Outcome::Skip);
    assert!(
        results[0]
            .message
            .as_deref()
            .unwrap_or_default()
            .contains("deferred"),
        "non-lifecycle world metadata must defer instead of passing: {results:#?}"
    );
}

#[test]
fn obligation_lifecycle_without_explicit_world_metadata_defers() {
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
            terminal_expectations: vec![ObligationTerminalExpectation::Discharged],
            ..RunnerObligationMetadata::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot(Path::new("obligation.ash"), &snapshot);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].outcome, Outcome::Skip);
    assert!(
        results[0]
            .message
            .as_deref()
            .unwrap_or_default()
            .contains("deferred"),
        "obligation lifecycle metadata without explicit finite worlds must defer: {results:#?}"
    );
}

#[test]
fn obligation_lifecycle_snapshot_world_state_disagreement_fails_on_normal_path() {
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
            required_closeout_behavior: Some("reject_if_open".to_string()),
            terminal_expectations: vec![ObligationTerminalExpectation::Discharged],
            lifecycle_transition_plan: Some(ObligationLifecycleTransitionPlan {
                model: ObligationLifecycleModelKind::IntroduceDischargeCheck,
                introduction_sites: vec!["open_ticket".to_string()],
                discharge_sites: vec!["close_ticket".to_string()],
                check_sites: vec!["finish".to_string()],
                required_closeout: ObligationCloseoutBehavior::RejectIfOpen,
            }),
            lifecycle_transition_traces: vec![ObligationLifecycleTransitionTrace {
                id: "ticket:introduced".to_string(),
                transitions: vec![ObligationLifecycleTransition::Introduce {
                    site: "open_ticket".to_string(),
                }],
            }],
            lifecycle_worlds: vec![SmallWorldState {
                id: "ticket:introduced".to_string(),
                schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
                world_kind: "obligation_lifecycle".to_string(),
                obligations: vec!["Ticket".to_string()],
                control_state: Some("introduced".to_string()),
                transition_trace: vec!["introduce:open_ticket".to_string()],
                ..SmallWorldState::default()
            }],
            ..RunnerObligationMetadata::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot(Path::new("obligation.ash"), &snapshot);

    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].outcome,
        Outcome::Fail,
        "normal snapshot obligation generation must evaluate supplied finite worlds rather than manufacturing a matching pass row"
    );
}

#[test]
fn obligation_lifecycle_unsupported_expectations_do_not_shift_world_alignment() {
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
            required_closeout_behavior: Some("reject_if_open".to_string()),
            terminal_expectations: vec![
                ObligationTerminalExpectation::Unsupported,
                ObligationTerminalExpectation::Discharged,
            ],
            lifecycle_transition_plan: Some(ObligationLifecycleTransitionPlan {
                model: ObligationLifecycleModelKind::IntroduceDischargeCheck,
                introduction_sites: vec!["open_ticket".to_string()],
                discharge_sites: vec!["close_ticket".to_string()],
                check_sites: vec!["finish".to_string()],
                required_closeout: ObligationCloseoutBehavior::RejectIfOpen,
            }),
            lifecycle_transition_traces: vec![
                ObligationLifecycleTransitionTrace {
                    id: "ticket:unsupported".to_string(),
                    transitions: vec![ObligationLifecycleTransition::Introduce {
                        site: "open_ticket".to_string(),
                    }],
                },
                ObligationLifecycleTransitionTrace {
                    id: "ticket:discharged".to_string(),
                    transitions: vec![
                        ObligationLifecycleTransition::Introduce {
                            site: "open_ticket".to_string(),
                        },
                        ObligationLifecycleTransition::Discharge {
                            site: "close_ticket".to_string(),
                        },
                        ObligationLifecycleTransition::Check {
                            site: "finish".to_string(),
                        },
                    ],
                },
            ],
            lifecycle_worlds: vec![
                SmallWorldState {
                    id: "ticket:unsupported".to_string(),
                    schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
                    world_kind: "obligation_lifecycle".to_string(),
                    control_state: Some("unsupported".to_string()),
                    ..SmallWorldState::default()
                },
                SmallWorldState {
                    id: "ticket:discharged".to_string(),
                    schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
                    world_kind: "obligation_lifecycle".to_string(),
                    obligations: vec!["Ticket".to_string()],
                    control_state: Some("discharged".to_string()),
                    transition_trace: vec![
                        "introduce:open_ticket".to_string(),
                        "discharge:close_ticket".to_string(),
                        "check:finish".to_string(),
                    ],
                    ..SmallWorldState::default()
                },
            ],
            ..RunnerObligationMetadata::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    };

    let results = synthesize_from_snapshot(Path::new("obligation.ash"), &snapshot);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].outcome, Outcome::Pass);
    let world_id = results[0]
        .repro_artifact
        .as_ref()
        .and_then(|repro| repro.world_snapshot.as_ref())
        .and_then(|world| world.get("id"))
        .and_then(Value::as_str);
    assert_eq!(world_id, Some("ticket:discharged"));
}

#[test]
fn obligation_lifecycle_oracle_fails_when_executed_trace_disagrees_with_expectation() {
    let mut bindings = BTreeMap::new();
    bindings.insert("lifecycle_control_state".to_string(), json!("introduced"));
    let case = SynthesizedCase {
        id: "synthesized/obligation/ticket/lifecycle-discharged-1".to_string(),
        source: TestSource::Obligation,
        target_kind: "obligation".to_string(),
        target_name: "Ticket".to_string(),
        file_path: PathBuf::from("obligation.ash"),
        tags: vec!["synthesized".to_string(), "obligation".to_string()],
        seed: 0,
        inputs: SynthesizedInputs {
            bindings,
            generated_from: "finite_obligation_lifecycle_metadata".to_string(),
            case_index: 1,
            world_index: Some(1),
        },
        oracle: SynthesizedOracle::ObligationLifecycle {
            expectation: ObligationTerminalExpectation::Discharged,
            transition_plan: ObligationLifecycleTransitionPlan {
                model: ObligationLifecycleModelKind::IntroduceDischargeCheck,
                introduction_sites: vec!["open_ticket".to_string()],
                discharge_sites: vec!["close_ticket".to_string()],
                check_sites: vec!["finish".to_string()],
                required_closeout: ObligationCloseoutBehavior::RejectIfOpen,
            },
            transition_trace: ObligationLifecycleTransitionTrace {
                id: "ticket:introduced".to_string(),
                transitions: vec![ObligationLifecycleTransition::Introduce {
                    site: "open_ticket".to_string(),
                }],
            },
        },
        repro: repro_artifact(
            Path::new("obligation.ash"),
            "source:obligation.ash".to_string(),
            "check:summary".to_string(),
            "synthesized/obligation/ticket/lifecycle-discharged-1".to_string(),
            0,
            1,
            None,
            json!({
                "kind": "obligation_lifecycle",
                "expectation": ObligationTerminalExpectation::Discharged,
                "expected_control_state": "discharged",
            }),
            Some(json!({
                "id": "ticket:discharged",
                "control_state": "discharged",
            })),
        ),
    };

    let result = execute_synthesized_case(&case);

    assert_eq!(
        result.outcome,
        Outcome::Fail,
        "typed transition execution must fail when the executed terminal disagrees with the expected terminal"
    );
}

#[test]
fn obligation_lifecycle_oracle_fails_when_world_state_disagrees_with_expectation() {
    let mut bindings = BTreeMap::new();
    bindings.insert("lifecycle_control_state".to_string(), json!("introduced"));
    let case = SynthesizedCase {
        id: "synthesized/obligation/ticket/lifecycle-discharged-1".to_string(),
        source: TestSource::Obligation,
        target_kind: "obligation".to_string(),
        target_name: "Ticket".to_string(),
        file_path: PathBuf::from("obligation.ash"),
        tags: vec!["synthesized".to_string(), "obligation".to_string()],
        seed: 0,
        inputs: SynthesizedInputs {
            bindings,
            generated_from: "finite_obligation_lifecycle_metadata".to_string(),
            case_index: 1,
            world_index: Some(1),
        },
        oracle: SynthesizedOracle::ObligationLifecycle {
            expectation: ObligationTerminalExpectation::Discharged,
            transition_plan: ObligationLifecycleTransitionPlan {
                model: ObligationLifecycleModelKind::IntroduceDischargeCheck,
                introduction_sites: vec!["open_ticket".to_string()],
                discharge_sites: vec!["close_ticket".to_string()],
                check_sites: vec!["finish".to_string()],
                required_closeout: ObligationCloseoutBehavior::RejectIfOpen,
            },
            transition_trace: ObligationLifecycleTransitionTrace {
                id: "ticket:introduced".to_string(),
                transitions: vec![ObligationLifecycleTransition::Introduce {
                    site: "open_ticket".to_string(),
                }],
            },
        },
        repro: repro_artifact(
            Path::new("obligation.ash"),
            "source:obligation.ash".to_string(),
            "check:summary".to_string(),
            "synthesized/obligation/ticket/lifecycle-discharged-1".to_string(),
            0,
            1,
            None,
            json!({
                "kind": "obligation_lifecycle",
                "expectation": ObligationTerminalExpectation::Discharged,
                "expected_control_state": "discharged",
            }),
            Some(json!({
                "id": "ticket:introduced",
                "control_state": "introduced",
            })),
        ),
    };

    let result = execute_synthesized_case(&case);

    assert_eq!(
        result.outcome,
        Outcome::Fail,
        "obligation lifecycle pass must be backed by evaluated finite world state"
    );
    assert!(
        result
            .message
            .as_deref()
            .unwrap_or_default()
            .contains("failed"),
        "wrong lifecycle metadata should explain the oracle failure: {result:#?}"
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

fn postcondition_snapshot(
    executable_target: Option<ContractExecutableTarget>,
    ensures: &str,
) -> RunnerIntrospectionSnapshot {
    RunnerIntrospectionSnapshot {
        schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
        module_identity: "test-module".to_string(),
        source_artifact_id: "source:test.ash".to_string(),
        check_summary_id: "check:summary".to_string(),
        contracts: vec![RunnerContractMetadata {
            id: "contract:identity".to_string(),
            callable_name: "identity".to_string(),
            callable_kind: "pure_function".to_string(),
            param_names: vec!["x".to_string()],
            param_types: vec!["Int".to_string()],
            return_type: Some("Int".to_string()),
            lowered_ensures: vec![ensures.to_string()],
            executable_postconditions: vec![ContractPostconditionOracle {
                display: ensures.to_string(),
                expression: match ensures {
                    "result == x" => core_result_compare(ash_core::BinaryOp::Eq),
                    "result != x" => core_result_compare(ash_core::BinaryOp::Ne),
                    _ => core_result_compare(ash_core::BinaryOp::Eq),
                },
            }],
            executable_target,
            generation_hints: vec![TypeGeneratorDescriptor {
                id: "x-valid".to_string(),
                target_type: "Int".to_string(),
                source: TypeGeneratorSource::ContractValid,
                exact_values: vec![json!(7)],
                ..TypeGeneratorDescriptor::default()
            }],
            executable_case_kinds: vec![SynthesizedOracleKind::PostconditionHolds],
            ..RunnerContractMetadata::default()
        }],
        ..RunnerIntrospectionSnapshot::default()
    }
}

fn smallworld_expr_target(expression: CoreExpr) -> SmallWorldExecutableTarget {
    SmallWorldExecutableTarget {
        kind: SmallWorldExecutableTargetKind::PureExpression,
        target_ref: "smallworld:target".to_string(),
        setup: ContractExecutionSetup::PureNoSetup,
        body: ContractTargetBody::ReturnExpression { expression },
    }
}

fn smallworld_literal_target(value: Value) -> SmallWorldExecutableTarget {
    SmallWorldExecutableTarget {
        kind: SmallWorldExecutableTargetKind::PureExpression,
        target_ref: "smallworld:target".to_string(),
        setup: ContractExecutionSetup::PureNoSetup,
        body: ContractTargetBody::ReturnLiteral { value },
    }
}

fn core_var(name: &str) -> CoreExpr {
    CoreExpr::Variable {
        name: name.to_string(),
        span: ash_core::Span::default(),
    }
}

fn core_result_compare(op: ash_core::BinaryOp) -> CoreExpr {
    CoreExpr::Binary {
        op,
        left: Box::new(core_var("result")),
        right: Box::new(core_var("x")),
    }
}
