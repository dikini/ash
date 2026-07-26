//! TASK-2005: parity reports must account for every required observable.
//!
//! These are contract tests only. They deliberately do not claim that the
//! production direct runtime and checked Core/CPS evaluator currently agree.

use ash_core::cps::{
    Atom as CpsAtom, ContMultiplicity, EffectItem, EffectItemKind, EffectOp, EffectRow,
    Env as CpsEnv, HandlerChain, Term,
};
use ash_engine::differential::{
    DifferentialHarness, ObservableDimension, ParityDisposition, RustExecutionTarget,
};
use ash_engine::{ApplicationAdmissionOutcome, ApplicationAdmissionRequest, Engine};
use ash_interp::cps::{CpsError, CpsRunError, eval_checked_terminal};
use std::{fs, path::PathBuf};

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/differential/corpus")
}

const MISSING_DECLARED_CLOCK_SOURCE: &str = r"
interface Clock<T> { sleep(Int) -> Null }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = null }
fn main() -> Null { TestClock::sleep(0) }
";

fn application_request(entry: &ash_engine::Entry) -> ApplicationAdmissionRequest {
    ApplicationAdmissionRequest {
        entry_name: "main".to_string(),
        body: entry.core.clone(),
        application_id: None,
        run_id: None,
        active_role: None,
        admitted_role: None,
        required_capabilities: Vec::new(),
        requires: Vec::new(),
        ensures: Vec::new(),
    }
}

const REQUIRED_OBSERVABLES: [ObservableDimension; 8] = [
    ObservableDimension::Values,
    ObservableDimension::StructuredTraps,
    ObservableDimension::FrameOrdering,
    ObservableDimension::MissingDischarge,
    ObservableDimension::Rows,
    ObservableDimension::ContinuationUse,
    ObservableDimension::DynamicContracts,
    ObservableDimension::AllowedExternalOutcomes,
];

fn write_source_entry_schema_fixture(root: &std::path::Path, schema_version: &str) {
    let case_dir = root.join("source-entry-schema-version");
    fs::create_dir_all(&case_dir).expect("fixture directory created");
    fs::write(
        case_dir.join("case.json"),
        r#"{
  "schema_version": "ash-corpus-case/v1",
  "case_id": "source-entry-schema-version",
  "canonical_rule_ids": ["SEM-CPS-PRIM-001"],
  "input_file": "input.ir.json",
  "expected_file": "expected.json"
}"#,
    )
    .expect("manifest written");
    fs::write(
        case_dir.join("input.ir.json"),
        format!(
            r#"{{
  "schema_version": "ash-phase202-direct-runtime-input/v1",
  "direct_runtime": {{"source": "fn main() -> Int {{ 7 }}"}},
  "checked_core_cps": {{
    "schema_version": "{schema_version}",
    "source_entry": true
  }}
}}"#
        ),
    )
    .expect("input written");
    fs::write(
        case_dir.join("expected.json"),
        r#"{
  "schema_version": "ash-expected-result/v1",
  "case_id": "source-entry-schema-version",
  "canonical_rule_ids": ["SEM-CPS-PRIM-001"],
  "expectation": {
    "kind": "exact",
    "result": {
      "outcome_class": "return",
      "payload": {"kind": "value", "value": {"type": "int", "value": 7}}
    }
  }
}"#,
    )
    .expect("expectation written");
}

fn write_source_entry_literal_if_fixture(root: &std::path::Path, case_id: &str, source: &str) {
    let case_dir = root.join(case_id);
    fs::create_dir_all(&case_dir).expect("fixture directory created");
    fs::write(
        case_dir.join("case.json"),
        format!(
            r#"{{
  "schema_version": "ash-corpus-case/v1",
  "case_id": "{case_id}",
  "canonical_rule_ids": ["SEM-CPS-IF-001"],
  "input_file": "input.ir.json",
  "expected_file": "expected.json"
}}"#
        ),
    )
    .expect("manifest written");
    fs::write(
        case_dir.join("input.ir.json"),
        format!(
            r#"{{
  "schema_version": "ash-phase202-direct-runtime-input/v1",
  "direct_runtime": {{"source": {source:?}}},
  "checked_core_cps": {{
    "source_entry": true,
    "observed_dimension": "values",
    "canonical_rule_id": "SEM-CPS-IF-001"
  }}
}}"#
        ),
    )
    .expect("input written");
    fs::write(
        case_dir.join("expected.json"),
        format!(
            r#"{{
  "schema_version": "ash-expected-result/v1",
  "case_id": "{case_id}",
  "canonical_rule_ids": ["SEM-CPS-IF-001"],
  "expectation": {{
    "kind": "exact",
    "result": {{
      "outcome_class": "return",
      "payload": {{"kind": "value", "value": {{"type": "int", "value": 7}}}}
    }}
  }}
}}"#
        ),
    )
    .expect("expectation written");
}

#[test]
fn source_entry_checked_core_cps_rejects_every_schema_version_during_corpus_load() {
    let root = tempfile::tempdir().expect("corpus root created");

    for schema_version in ["ash-cps-kernel-input/v1", "future-schema/v999"] {
        write_source_entry_schema_fixture(root.path(), schema_version);
        let error = DifferentialHarness::load(root.path())
            .expect_err("a source-entry input must not carry any checked Core/CPS schema version");
        assert!(
            error
                .to_string()
                .contains("source-entry checked Core/CPS input must not declare `schema_version`"),
            "unexpected schema-version rejection for {schema_version}: {error}"
        );
    }
}

#[test]
fn source_entry_if_values_claim_rejects_an_altered_literal_else_branch_during_corpus_load() {
    let root = tempfile::tempdir().expect("corpus root created");
    let case_id = "source-entry-if-altered-else-branch";
    write_source_entry_literal_if_fixture(
        root.path(),
        case_id,
        "fn main() -> Int { if true then 7 else 10 }",
    );

    let error = DifferentialHarness::load(root.path())
        .expect_err("an altered source-entry literal If branch must not claim SEM-CPS-IF-001");
    assert!(
        error.to_string().contains(&format!(
            "{case_id} cannot claim SEM-CPS-IF-001 source-entry values: checked source lowering is not the admitted literal Boolean If with answer jumps 7 and 9"
        )),
        "unexpected altered-branch rejection: {error}"
    );
}

#[test]
fn untrusted_corpus_root_cannot_invoke_the_legacy_direct_runtime_oracle() {
    let root = tempfile::tempdir().expect("untrusted corpus root created");
    let source_case = corpus_root().join("phase202-return-unit");
    let copied_case = root.path().join("phase202-return-unit");
    fs::create_dir_all(&copied_case).expect("copied fixture directory created");
    for file in ["case.json", "input.ir.json", "expected.json"] {
        fs::copy(source_case.join(file), copied_case.join(file))
            .expect("trusted fixture file copied into untrusted root");
    }

    let harness = DifferentialHarness::load(root.path()).expect("untrusted fixture still parses");
    let report = harness.run_case("phase202-return-unit", RustExecutionTarget::DirectRuntime);

    assert!(matches!(
        report.direct_runtime_status(),
        ash_engine::differential::CaseComparisonStatus::Unsupported { ref reason }
            if reason.contains("restricted to exact trusted built-in TASK-2005 reference cases")
    ));
    assert_eq!(report.actual_result(), None);
}

#[cfg(unix)]
#[test]
fn symlinked_builtin_corpus_root_cannot_invoke_the_legacy_direct_runtime_oracle() {
    let root = tempfile::tempdir().expect("symlink parent created");
    let linked_root = root.path().join("corpus-link");
    std::os::unix::fs::symlink(corpus_root(), &linked_root)
        .expect("built-in corpus symlink created without modifying it");

    let harness = DifferentialHarness::load(&linked_root).expect("linked corpus still parses");
    let report = harness.run_case("phase202-return-unit", RustExecutionTarget::DirectRuntime);

    assert!(matches!(
        report.direct_runtime_status(),
        ash_engine::differential::CaseComparisonStatus::Unsupported { ref reason }
            if reason.contains("restricted to exact trusted built-in TASK-2005 reference cases")
    ));
    assert_eq!(report.actual_result(), None);
}

#[test]
fn every_task_2005_observable_is_compared_or_has_an_owned_non_parity_disposition() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");
    let report = harness.run_case("phase202-return-unit", RustExecutionTarget::DirectRuntime);
    let parity = report.parity_report();

    assert_eq!(parity.source_fixture(), "phase202-return-unit");
    for dimension in REQUIRED_OBSERVABLES {
        let disposition = parity
            .disposition_for(dimension)
            .unwrap_or_else(|| panic!("missing TASK-2005 disposition for {dimension:?}"));

        assert!(
            matches!(
                disposition,
                ParityDisposition::Compared { .. }
                    | ParityDisposition::BoundedDivergence { .. }
                    | ParityDisposition::Unsupported { .. }
            ),
            "{dimension:?} must be compared or explicitly accounted for"
        );
        assert!(
            !disposition.canonical_rule_id().is_empty(),
            "{dimension:?} disposition must name a canonical rule"
        );
        assert!(
            !disposition.owner().is_empty(),
            "{dimension:?} disposition must name an owner"
        );
    }
}

#[test]
fn fixture_without_checked_core_cps_execution_is_an_owned_non_passing_divergence() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");
    let report = harness.run_case(
        "phase202-bounded-external",
        RustExecutionTarget::DirectRuntime,
    );
    let parity = report.parity_report();
    let core_cps = parity
        .disposition_for(ObservableDimension::CheckedCoreCpsExecution)
        .expect("Core/CPS execution must have an explicit TASK-2005 disposition");

    assert!(
        matches!(
            core_cps,
            ParityDisposition::BoundedDivergence { .. } | ParityDisposition::Unsupported { .. }
        ),
        "unavailable checked Core/CPS execution cannot be reported as parity"
    );
    assert_eq!(core_cps.canonical_rule_id(), "SEM-TARGET-CORE-CPS-001");
    assert_eq!(core_cps.owner(), "TASK-2004");
}

#[test]
fn paired_literal_fixture_compares_direct_runtime_and_checked_core_cps_values() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");
    let report = harness.run_case("phase202-return-unit", RustExecutionTarget::DirectRuntime);
    let parity = report.parity_report();

    assert!(matches!(
        report.checked_core_cps_relation(),
        ash_engine::differential::RelationStatus::Passed
    ));
    assert!(matches!(
        parity.disposition_for(ObservableDimension::Values),
        Some(ParityDisposition::Compared {
            canonical_rule_id,
            owner,
        }) if canonical_rule_id == "SEM-CPS-RETURN-001" && owner == "TASK-2005"
    ));
    assert!(matches!(
        parity.disposition_for(ObservableDimension::CheckedCoreCpsExecution),
        Some(ParityDisposition::Compared {
            canonical_rule_id,
            owner,
        }) if canonical_rule_id == "SEM-TARGET-CORE-CPS-001" && owner == "TASK-2004"
    ));
}

#[test]
fn paired_v3_int_add_fixture_compares_primitive_values_under_the_primitive_rule() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");
    let report = harness.run_case(
        "phase202-v3-int-add-return-7",
        RustExecutionTarget::DirectRuntime,
    );
    let parity = report.parity_report();

    assert!(matches!(
        report.checked_core_cps_relation(),
        ash_engine::differential::RelationStatus::Passed
    ));
    assert!(matches!(
        parity.disposition_for(ObservableDimension::Values),
        Some(ParityDisposition::Compared {
            canonical_rule_id,
            owner,
        }) if canonical_rule_id == "SEM-CPS-PRIM-001" && owner == "TASK-2005"
    ));
    assert!(matches!(
        parity.disposition_for(ObservableDimension::CheckedCoreCpsExecution),
        Some(ParityDisposition::Compared {
            canonical_rule_id,
            owner,
        }) if canonical_rule_id == "SEM-TARGET-CORE-CPS-001" && owner == "TASK-2004"
    ));
}

#[test]
fn source_int_add_fixture_compares_bridge_derived_primitive_values_under_the_primitive_rule() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");
    let report = harness.run_case(
        "phase202-source-int-add-bridge-return-7",
        RustExecutionTarget::DirectRuntime,
    );
    let parity = report.parity_report();

    assert!(matches!(
        report.checked_core_cps_relation(),
        ash_engine::differential::RelationStatus::Passed
    ));
    assert!(
        matches!(
            parity.disposition_for(ObservableDimension::Values),
            Some(ParityDisposition::Compared {
                canonical_rule_id,
                owner,
            }) if canonical_rule_id == "SEM-CPS-PRIM-001" && owner == "TASK-2005"
        ),
        "a source_entry primitive pair must attribute its value comparison to SEM-CPS-PRIM-001"
    );
    assert!(matches!(
        parity.disposition_for(ObservableDimension::CheckedCoreCpsExecution),
        Some(ParityDisposition::Compared {
            canonical_rule_id,
            owner,
        }) if canonical_rule_id == "SEM-TARGET-CORE-CPS-001" && owner == "TASK-2004"
    ));
}

#[test]
fn source_lexical_int_add_fixture_preserves_letval_bindings_before_primitive_value_parity() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(
            r"
            fn main() -> Int {
                do {
                    let x = 2;
                    let y = 5;
                    return x + y;
                }
            }
            ",
        )
        .expect("lexical addition source parses");
    engine
        .check(&mut entry)
        .expect("lexical addition source typechecks");

    let lowered = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("lexical addition source lowers to checked CPS");
    let Term::LetVal {
        name: x,
        body: x_body,
        ..
    } = lowered
    else {
        panic!("the source bridge must preserve the x LetVal binding");
    };
    assert_eq!(x, "x");
    let Term::LetVal {
        name: y,
        body: y_body,
        ..
    } = *x_body
    else {
        panic!("the source bridge must preserve the y LetVal binding");
    };
    assert_eq!(y, "y");
    let Term::LetPrim { op, args, body, .. } = *y_body else {
        panic!("the lexical bindings must enclose LetPrim(Add)");
    };
    assert!(matches!(op, ash_core::cps::PrimOp::Add));
    assert_eq!(
        args,
        vec![CpsAtom::Var("x".to_string()), CpsAtom::Var("y".to_string())]
    );
    assert!(matches!(
        *body,
        Term::Jump {
            cont: ash_core::cps::ContRef::Label(ref answer),
            arg: CpsAtom::Var(_),
            ..
        } if answer == "__answer"
    ));

    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");
    let report = harness.run_case(
        "phase202-source-lexical-int-add-bridge-return-7",
        RustExecutionTarget::DirectRuntime,
    );
    let parity = report.parity_report();

    assert!(matches!(
        report.checked_core_cps_relation(),
        ash_engine::differential::RelationStatus::Passed
    ));
    assert!(matches!(
        parity.disposition_for(ObservableDimension::Values),
        Some(ParityDisposition::Compared {
            canonical_rule_id,
            owner,
        }) if canonical_rule_id == "SEM-CPS-PRIM-001" && owner == "TASK-2005"
    ));
}

#[test]
fn paired_v4_if_fixture_compares_selected_branch_values_under_the_if_rule() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");
    let report = harness.run_case(
        "phase202-v4-if-true-return-int-7",
        RustExecutionTarget::DirectRuntime,
    );
    let parity = report.parity_report();

    assert!(matches!(
        report.checked_core_cps_relation(),
        ash_engine::differential::RelationStatus::Passed
    ));
    assert!(matches!(
        parity.disposition_for(ObservableDimension::Values),
        Some(ParityDisposition::Compared {
            canonical_rule_id,
            owner,
        }) if canonical_rule_id == "SEM-CPS-IF-001" && owner == "TASK-2005"
    ));
    assert!(matches!(
        parity.disposition_for(ObservableDimension::CheckedCoreCpsExecution),
        Some(ParityDisposition::Compared {
            canonical_rule_id,
            owner,
        }) if canonical_rule_id == "SEM-TARGET-CORE-CPS-001" && owner == "TASK-2004"
    ));
}

#[test]
fn paired_v4_false_if_fixture_compares_the_else_branch_under_the_if_rule() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");
    let report = harness.run_case(
        "phase202-v4-if-false-return-int-9",
        RustExecutionTarget::DirectRuntime,
    );
    let parity = report.parity_report();

    assert!(matches!(
        report.checked_core_cps_relation(),
        ash_engine::differential::RelationStatus::Passed
    ));
    assert!(matches!(
        parity.disposition_for(ObservableDimension::Values),
        Some(ParityDisposition::Compared {
            canonical_rule_id,
            owner,
        }) if canonical_rule_id == "SEM-CPS-IF-001" && owner == "TASK-2005"
    ));
    assert!(matches!(
        parity.disposition_for(ObservableDimension::CheckedCoreCpsExecution),
        Some(ParityDisposition::Compared {
            canonical_rule_id,
            owner,
        }) if canonical_rule_id == "SEM-TARGET-CORE-CPS-001" && owner == "TASK-2004"
    ));
}

fn assert_source_literal_if_bridge(
    source: &str,
    condition: bool,
    expected_value: i64,
    case_id: &str,
) {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine.parse(source).expect("literal if source parses");
    engine
        .check(&mut entry)
        .expect("literal if source typechecks before checked CPS inspection");

    let lowered = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("literal if source lowers to checked CPS");
    assert!(
        matches!(
            lowered,
            Term::If {
                cond: CpsAtom::Bool(actual_condition),
                then_branch,
                else_branch,
                ..
            } if actual_condition == condition
                && matches!(
                    *then_branch,
                    Term::Jump {
                        cont: ash_core::cps::ContRef::Label(ref answer),
                        arg: CpsAtom::Int(7),
                        ..
                    } if answer == "__answer"
                )
                && matches!(
                    *else_branch,
                    Term::Jump {
                        cont: ash_core::cps::ContRef::Label(ref answer),
                        arg: CpsAtom::Int(9),
                        ..
                    } if answer == "__answer"
                )
        ),
        "the admitted source bridge must preserve literal If branches as answer jumps"
    );

    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");
    let report = harness.run_case(case_id, RustExecutionTarget::DirectRuntime);
    let parity = report.parity_report();

    assert_eq!(
        report.actual_result(),
        Some(&serde_json::json!({
            "outcome_class": "return",
            "payload": {"kind": "value", "value": {"type": "int", "value": expected_value}},
        })),
        "the direct source side must select its literal branch"
    );
    assert!(matches!(
        report.checked_core_cps_relation(),
        ash_engine::differential::RelationStatus::Passed
    ));
    assert!(matches!(
        parity.disposition_for(ObservableDimension::Values),
        Some(ParityDisposition::Compared {
            canonical_rule_id,
            owner,
        }) if canonical_rule_id == "SEM-CPS-IF-001" && owner == "TASK-2005"
    ));
}

#[test]
fn source_true_literal_if_fixture_compares_the_checked_cps_branch_under_the_if_rule() {
    assert_source_literal_if_bridge(
        "fn main() -> Int { if true then 7 else 9 }",
        true,
        7,
        "phase202-source-if-true-bridge-return-7",
    );
}

#[test]
fn source_false_literal_if_fixture_compares_the_checked_cps_branch_under_the_if_rule() {
    assert_source_literal_if_bridge(
        "fn main() -> Int { if false then 7 else 9 }",
        false,
        9,
        "phase202-source-if-false-bridge-return-9",
    );
}

#[test]
fn missing_declared_operation_discharge_compares_typed_admission_and_unhandled_raise() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(MISSING_DECLARED_CLOCK_SOURCE)
        .expect("declared clock source parses");
    engine
        .check(&mut entry)
        .expect("declared clock source resolves before admission");

    let ApplicationAdmissionOutcome::Rejected { failure, .. } = tokio_test::block_on(
        engine.admit_application_with_explicit_rows(application_request(&entry), &entry),
    ) else {
        panic!("a declared operation without a binding must reject at admission");
    };
    assert_eq!(
        failure.kind,
        ash_core::runtime::ApplicationFailureKind::CapabilityAdmissionFailure,
        "the direct side must preserve its typed pre-execution admission class"
    );
    let declared = entry
        .declared_concrete_operation
        .as_ref()
        .expect("the rejected source entry retains its resolved operation carrier");
    assert_eq!(declared.impl_type, "TestClock");
    assert_eq!(declared.operation, "sleep");
    assert_eq!(declared.params.len(), 1);
    assert_eq!(declared.params[0].to_string(), "Int");
    assert_eq!(declared.result_type.to_string(), "Null");

    let expected_operation = EffectOp {
        item: EffectItem {
            namespace: "TestClock".to_string(),
            name: "sleep".to_string(),
            kind: EffectItemKind::Capability,
        },
        arg_types: vec!["Int".to_string()],
        result_type: "Null".to_string(),
    };
    let lowered = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("the same resolved source entry lowers to Raise");
    let Term::Raise { op, .. } = &lowered else {
        panic!("declared operation must lower to a CPS Raise");
    };
    assert_eq!(*op, expected_operation);
    let executable_lowered = Term::LetCont {
        name: "__answer".to_string(),
        param: "__answer_value".to_string(),
        cont_body: Box::new(Term::Return {
            value: ash_core::cps::Value::Atom(CpsAtom::Var("__answer_value".to_string())),
        }),
        body: Box::new(lowered),
        row: EffectRow::default(),
        multiplicity: ContMultiplicity::Affine,
    };
    let cps_error =
        eval_checked_terminal(&executable_lowered, &CpsEnv::new(), &HandlerChain::new())
            .expect_err("an empty handler chain must leave the typed Raise unhandled");
    assert!(matches!(
        cps_error,
        CpsRunError::Runtime(CpsError::UnhandledEffect(ref operation)) if operation == &expected_operation
    ));

    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");
    let report = harness.run_case(
        "phase202-missing-declared-operation-discharge",
        RustExecutionTarget::DirectRuntime,
    );

    assert!(matches!(
        report
            .parity_report()
            .disposition_for(ObservableDimension::MissingDischarge),
        Some(ParityDisposition::Compared {
            canonical_rule_id,
            owner,
        }) if canonical_rule_id == "SEM-EFFECT-MISSDISCHARGE-001" && owner == "TASK-2005"
    ));
    assert!(matches!(
        report.checked_core_cps_relation(),
        ash_engine::differential::RelationStatus::Passed
    ));
}

#[test]
fn explicit_admission_and_source_entry_do_not_implicitly_select_missing_discharge_projection() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");
    let report = harness.run_case(
        "phase202-explicit-admission-source-entry-control",
        RustExecutionTarget::DirectRuntime,
    );

    assert!(matches!(
        report
            .parity_report()
            .disposition_for(ObservableDimension::MissingDischarge),
        Some(ParityDisposition::Unsupported {
            canonical_rule_id,
            owner,
        }) if canonical_rule_id == "SEM-EFFECT-MISSDISCHARGE-001" && owner == "TASK-2005"
    ));
}

#[test]
fn paired_literal_mismatch_is_reported_as_value_drift_with_its_canonical_rule() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");
    let report = harness.run_case(
        "phase202-return-unit-mismatch",
        RustExecutionTarget::DirectRuntime,
    );

    assert!(matches!(
        report.checked_core_cps_relation(),
        ash_engine::differential::RelationStatus::Failed {
            canonical_rule_id,
            reason,
        } if canonical_rule_id == "SEM-CPS-RETURN-001"
            && reason.contains("SEM-CPS-RETURN-001")
            && reason.contains("phase202-return-unit-mismatch")
            && reason.contains("did not match checked Core/CPS result")
            && reason.contains("\"value\":7")
            && reason.contains("\"value\":8")
    ));
    assert!(matches!(
        report
            .parity_report()
            .disposition_for(ObservableDimension::Values),
        Some(ParityDisposition::Unsupported {
            canonical_rule_id,
            owner,
        }) if canonical_rule_id == "SEM-CPS-RETURN-001" && owner == "TASK-2005"
    ));
}

#[test]
fn direct_runtime_relation_failure_diagnostic_names_its_rule_and_manifest_case() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");
    let report = harness.run_case(
        "phase202-direct-runtime-failure-attribution",
        RustExecutionTarget::DirectRuntime,
    );

    assert!(matches!(
        report.checked_core_cps_relation(),
        ash_engine::differential::RelationStatus::Failed {
            canonical_rule_id,
            reason,
        } if canonical_rule_id == "SEM-CPS-RETURN-001"
            && reason.contains("SEM-CPS-RETURN-001")
            && reason.contains("phase202-direct-runtime-failure-attribution")
            && reason.contains("direct runtime could not produce an observable")
    ));
}

#[test]
fn checked_core_cps_relation_failure_diagnostic_names_its_rule_and_manifest_case() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");
    let report = harness.run_case(
        "phase202-checked-core-cps-failure-attribution",
        RustExecutionTarget::DirectRuntime,
    );

    assert!(matches!(
        report.checked_core_cps_relation(),
        ash_engine::differential::RelationStatus::Failed {
            canonical_rule_id,
            reason,
        } if canonical_rule_id == "SEM-CPS-RETURN-001"
            && reason.contains("SEM-CPS-RETURN-001")
            && reason.contains("phase202-checked-core-cps-failure-attribution")
            && reason.contains("validation error")
            && reason.contains("missing_terminal")
    ));
}

#[test]
fn paired_primitive_domain_trap_compares_structured_terminal_observables() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");
    let report = harness.run_case(
        "phase202-primitive-domain-trap",
        RustExecutionTarget::DirectRuntime,
    );

    assert!(matches!(
        report.checked_core_cps_relation(),
        ash_engine::differential::RelationStatus::Passed
    ));
    assert!(matches!(
        report
            .parity_report()
            .disposition_for(ObservableDimension::StructuredTraps),
        Some(ParityDisposition::Compared {
            canonical_rule_id,
            owner,
        }) if canonical_rule_id == "SEM-CPS-TRAP-001" && owner == "TASK-2005"
    ));
    assert!(
        matches!(
            report
                .parity_report()
                .disposition_for(ObservableDimension::Values),
            Some(ParityDisposition::Unsupported { .. })
        ),
        "a trap pair must not be recorded as value-parity evidence"
    );
}

#[test]
fn source_return_pair_executes_the_checked_answer_continuation_before_comparison() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");
    let report = harness.run_case(
        "phase202-source-return-continuation",
        RustExecutionTarget::DirectRuntime,
    );

    assert!(matches!(
        report.checked_core_cps_relation(),
        ash_engine::differential::RelationStatus::Passed
    ));
    assert!(matches!(
        report
            .parity_report()
            .disposition_for(ObservableDimension::ContinuationUse),
        Some(ParityDisposition::Compared {
            canonical_rule_id,
            owner,
        }) if canonical_rule_id == "SEM-CPS-JUMP-001" && owner == "TASK-2005"
    ));
    assert!(
        matches!(
            report
                .parity_report()
                .disposition_for(ObservableDimension::Values),
            Some(ParityDisposition::Unsupported { .. })
        ),
        "the answer-continuation case is continuation evidence, not a second value-only claim"
    );
}
