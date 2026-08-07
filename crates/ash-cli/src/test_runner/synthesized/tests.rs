use crate::test_runner::types::Outcome;

use super::*;

fn assert_metadata_execution_deferred(results: &[TestResult]) {
    assert!(
        !results.is_empty(),
        "the metadata fixture must produce rows"
    );
    assert!(
        results.iter().all(|result| {
            result.outcome == Outcome::Skip
                && result
                    .message
                    .as_deref()
                    .is_some_and(|message| message.starts_with("deferred:"))
        }),
        "metadata-derived rows must defer without local evaluation: {results:#?}"
    );
}

#[test]
fn synthesized_metadata_parser_preserves_parser_owned_inline_module_structure() {
    let source = "pub mod child { pub fn nested() -> Int { 1 } }\npub fn root() -> Int { 0 }\n";
    let module = parse_synthesized_metadata_module(Path::new("fixture.ash"), source)
        .expect("parser-owned synthesized metadata should accept inline modules");

    assert!(
        module
            .module_decls
            .iter()
            .any(|declaration| declaration.name.as_ref() == "child"),
        "synthesized metadata must retain parser-owned inline module declarations"
    );
}

fn parse_module_for_law_extraction(source: &str) -> ModuleFile {
    let source = strip_synthesized_metadata_non_definition_lines(source);
    ash_parser::parse_surface_file(&source)
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
fn extract_laws_records_property_and_small_world_test_evidence() {
    let module = parse_module_for_law_extraction(
        r#"
            law id_property(x: Int): x == x
            proof id_property(x: Int) { by test property }

            law id_small_world(x: Bool): x == x
            proof id_small_world(x: Bool) { by test small_world }
            "#,
    );

    let laws = extract_laws(&module);

    assert_eq!(laws.len(), 2);
    assert!(matches!(
        laws[0].test_evidence,
        Some(LawTestEvidence::Property { strategies: _ }),
    ));
    assert!(matches!(
        laws[1].test_evidence,
        Some(LawTestEvidence::SmallWorld)
    ));
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

#[test]
fn contract_synthesis_finds_requires() {
    let source = r#"
fn test_workflow(x: Int) -> Int
    requires: x > 0
    ensures: result > 0
{
    x
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
fn test_workflow(x: Int) -> Int
    requires: x > 0
    ensures: result > 0
{
    x
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
fn test_workflow(x: Int) -> Int
    requires: x > 0
{
    x
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
fn test() {
    oblige MyObligation
    check MyObligation
    0
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
fn test() {
    oblige MyObligation
    check MyObligation
    0
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
fn test() {
    0
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

#[test]
fn compatibility_api_defers_each_metadata_category_without_execution_authority() {
    let snapshot = RunnerIntrospectionSnapshot {
        schema_version: RUNNER_SYNTHESIS_SCHEMA_VERSION.to_string(),
        module_identity: "compatibility-metadata".to_string(),
        source_artifact_id: "source:compatibility.ash".to_string(),
        check_summary_id: "checked:compatibility".to_string(),
        contracts: vec![RunnerContractMetadata {
            id: "compatibility-contract".to_string(),
            callable_name: "contract_target".to_string(),
            ..RunnerContractMetadata::default()
        }],
        policies: vec![RunnerPolicyMetadata {
            id: "compatibility-policy".to_string(),
            policy_name: "policy_target".to_string(),
            ..RunnerPolicyMetadata::default()
        }],
        obligations: vec![RunnerObligationMetadata {
            id: "compatibility-obligation".to_string(),
            obligation_name: "obligation_target".to_string(),
            ..RunnerObligationMetadata::default()
        }],
        laws: vec![RunnerLawMetadata {
            id: "compatibility-law".to_string(),
            name: "law_target".to_string(),
            scope: LawScope::Module,
            owner: None,
            params: Vec::new(),
            proposition: "true == true".to_string(),
            delegated_test: None,
            test_evidence: None,
        }],
        generators: vec![TypeGeneratorDescriptor {
            id: "compatibility-property".to_string(),
            target_type: "Int".to_string(),
            ..TypeGeneratorDescriptor::default()
        }],
        small_world_domains: vec![SmallWorldDomain {
            id: "compatibility-world".to_string(),
            ..SmallWorldDomain::default()
        }],
        unsupported: vec![IntrospectionUnsupportedReason {
            source_kind: "contract".to_string(),
            target_name: "compatibility-unsupported".to_string(),
            reason: "unsupported compatibility metadata".to_string(),
        }],
    };

    let results = synthesize_from_snapshot(Path::new("compatibility.ash"), &snapshot);

    assert_eq!(results.len(), 7);
    assert_metadata_execution_deferred(&results);
    assert!(results.iter().all(|result| {
        result.repro_artifact.as_ref().is_some_and(|repro| {
            repro.oracle_snapshot["execution_route"] == "deferred_before_execution"
        })
    }));
    let law = results
        .iter()
        .find(|result| result.source == TestSource::Law)
        .expect("law metadata should remain visible as an explicit deferral");
    assert!(
        law.repro_artifact
            .as_ref()
            .is_some_and(|repro| repro.replay_command.contains("--only-synthesized laws"))
    );
}
