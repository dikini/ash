//! TASK-2005: parity reports must account for every required observable.
//!
//! These are contract tests only. They deliberately do not claim that the
//! production direct runtime and checked Core/CPS evaluator currently agree.

use super::{
    CpsError, CpsRunError, DifferentialHarness, ObservableDimension, ParityDisposition,
    RustExecutionTarget, eval_checked_terminal,
};
use crate::{ApplicationAdmissionOutcome, ApplicationAdmissionRequest, Engine};
use ash_core::cps::{
    Atom as CpsAtom, ContMultiplicity, EffectItem, EffectItemKind, EffectOp, EffectRow,
    Env as CpsEnv, HandlerChain, Term, Value as CpsValue,
};
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

fn application_request(entry: &crate::Entry) -> ApplicationAdmissionRequest {
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
fn source_entry_bool_not_values_claim_rejects_an_altered_boolean_literal_during_corpus_load() {
    let root = tempfile::tempdir().expect("temporary corpus root created");
    let case_id = "phase202-source-bool-not-bridge-return-false";
    let source_case = corpus_root().join(case_id);
    let copied_case = root.path().join(case_id);
    fs::create_dir_all(&copied_case).expect("copied fixture directory created");
    for file in ["case.json", "input.ir.json", "expected.json"] {
        fs::copy(source_case.join(file), copied_case.join(file))
            .expect("Boolean-not fixture file copied into temporary corpus");
    }

    let input_path = copied_case.join("input.ir.json");
    let input = fs::read_to_string(&input_path).expect("copied Boolean-not input read");
    let altered = input.replace(
        "fn main() -> Bool { !true }",
        "fn main() -> Bool { !false }",
    );
    assert_ne!(
        altered, input,
        "the temporary control must alter only the Boolean literal"
    );
    fs::write(input_path, altered).expect("altered Boolean-not input written");

    let error = DifferentialHarness::load(root.path()).expect_err(
        "an altered Boolean-not source-entry claim must reject before direct or checked execution",
    );
    assert!(
        error.to_string().contains(&format!(
            "{case_id} cannot claim SEM-CPS-PRIM-001 source-entry values"
        )),
        "unexpected altered-Boolean-not rejection: {error}"
    );
}

#[test]
fn source_entry_bool_not_values_claim_rejects_a_nested_boolean_literal_during_corpus_load() {
    let root = tempfile::tempdir().expect("temporary corpus root created");
    let case_id = "phase202-source-bool-not-bridge-return-false";
    let source_case = corpus_root().join(case_id);
    let copied_case = root.path().join(case_id);
    fs::create_dir_all(&copied_case).expect("copied fixture directory created");
    for file in ["case.json", "input.ir.json", "expected.json"] {
        fs::copy(source_case.join(file), copied_case.join(file))
            .expect("Boolean-not fixture file copied into temporary corpus");
    }

    let input_path = copied_case.join("input.ir.json");
    let input = fs::read_to_string(&input_path).expect("copied Boolean-not input read");
    let altered = input.replace(
        "fn main() -> Bool { !true }",
        "fn main() -> Bool { !!true }",
    );
    assert_ne!(
        altered, input,
        "the temporary control must alter only the Boolean unary nesting"
    );
    fs::write(input_path, altered).expect("altered Boolean-not input written");

    let error = DifferentialHarness::load(root.path()).expect_err(
        "a nested Boolean-not source-entry claim must reject before direct or checked execution",
    );
    assert!(
        error.to_string().contains(&format!(
            "{case_id} cannot claim SEM-CPS-PRIM-001 source-entry values"
        )),
        "unexpected nested-Boolean-not rejection: {error}"
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
        crate::differential::CaseComparisonStatus::Unsupported { ref reason }
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
        crate::differential::CaseComparisonStatus::Unsupported { ref reason }
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
        crate::differential::RelationStatus::Passed
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
        crate::differential::RelationStatus::Passed
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
        crate::differential::RelationStatus::Passed
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
fn source_int_sub_fixture_compares_differential_only_primitive_value_parity() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");
    let report = harness.run_case(
        "phase202-source-int-sub-bridge-return-5",
        RustExecutionTarget::DirectRuntime,
    );
    let parity = report.parity_report();

    assert_eq!(
        report.actual_result(),
        Some(&serde_json::json!({
            "outcome_class": "return",
            "payload": {"kind": "value", "value": {"type": "int", "value": 5}},
        })),
        "the direct differential oracle must observe 7 - 2 as Int(5)"
    );
    assert!(matches!(
        report.checked_core_cps_relation(),
        crate::differential::RelationStatus::Passed
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

const NESTED_BINARY_SOURCE: &str = "fn main() -> Bool { (1 + 2) >= (2 * 3) }";
const NESTED_BINARY_CASE_ID: &str = "phase202-source-nested-binary-anf-bridge-return-false";

#[test]
fn source_nested_binary_anf_fixture_compares_only_its_exact_private_differential_witness() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(NESTED_BINARY_SOURCE)
        .expect("nested binary source parses");
    engine
        .check(&mut entry)
        .expect("nested binary source typechecks before checked CPS inspection");
    let lowered = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("nested binary source lowers to checked CPS");

    let Term::LetPrim {
        name: add_result,
        op: ash_core::cps::PrimOp::Add,
        args: add_args,
        body: multiply_body,
    } = lowered
    else {
        panic!("the exact differential source must start with LetPrim(Add)");
    };
    assert_eq!(add_args, vec![CpsAtom::Int(1), CpsAtom::Int(2)]);
    let Term::LetPrim {
        name: multiply_result,
        op: ash_core::cps::PrimOp::Mul,
        args: multiply_args,
        body: comparison_body,
    } = *multiply_body
    else {
        panic!("the exact differential source must evaluate its right operand second");
    };
    assert_eq!(multiply_args, vec![CpsAtom::Int(2), CpsAtom::Int(3)]);
    let Term::LetPrim {
        name: comparison_result,
        op: ash_core::cps::PrimOp::Ge,
        args: comparison_args,
        body: answer_jump,
    } = *comparison_body
    else {
        panic!("the exact differential source must finish with LetPrim(Ge)");
    };
    assert_eq!(
        comparison_args,
        vec![CpsAtom::Var(add_result), CpsAtom::Var(multiply_result)]
    );
    assert!(matches!(
        *answer_jump,
        Term::Jump {
            cont: ash_core::cps::ContRef::Label(ref answer),
            arg: CpsAtom::Var(ref result),
            ..
        } if answer == "__answer" && result == &comparison_result
    ));

    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");
    let report = harness.run_case(NESTED_BINARY_CASE_ID, RustExecutionTarget::DirectRuntime);
    let parity = report.parity_report();

    assert_eq!(
        report.actual_result(),
        Some(&serde_json::json!({
            "outcome_class": "return",
            "payload": {"kind": "value", "value": {"type": "bool", "value": false}},
        })),
        "the direct differential oracle must observe (1 + 2) >= (2 * 3) as Bool(false)"
    );
    assert!(matches!(
        report.checked_core_cps_relation(),
        crate::differential::RelationStatus::Passed
    ));
    assert!(matches!(
        parity.disposition_for(ObservableDimension::Values),
        Some(ParityDisposition::Compared {
            canonical_rule_id,
            owner,
        }) if canonical_rule_id == "SEM-CPS-PRIM-001" && owner == "TASK-2005"
    ));
}

fn assert_source_nested_binary_anf_entry_rejects(replacement: &str) {
    let source_case = corpus_root().join(NESTED_BINARY_CASE_ID);
    let root = tempfile::tempdir().expect("temporary corpus root created");
    let copied_case = root.path().join(NESTED_BINARY_CASE_ID);
    fs::create_dir_all(&copied_case).expect("copied nested binary fixture directory created");
    for file in ["case.json", "input.ir.json", "expected.json"] {
        fs::copy(source_case.join(file), copied_case.join(file))
            .expect("nested binary fixture file copied into temporary corpus");
    }

    let input_path = copied_case.join("input.ir.json");
    let input = fs::read_to_string(&input_path).expect("copied nested binary input read");
    let altered = input.replace(NESTED_BINARY_SOURCE, replacement);
    assert_ne!(
        altered, input,
        "the temporary control must alter the exact nested binary source"
    );
    fs::write(input_path, altered).expect("altered nested binary input written");

    let error = DifferentialHarness::load(root.path()).expect_err(
        "a tampered nested binary source-entry claim must reject before direct or checked execution",
    );
    assert!(
        error.to_string().contains(&format!(
            "{NESTED_BINARY_CASE_ID} cannot claim SEM-CPS-PRIM-001 source-entry values"
        )),
        "unexpected {NESTED_BINARY_CASE_ID} rejection: {error}"
    );
}

#[test]
fn source_entry_nested_binary_anf_rejects_a_source_text_tamper_during_corpus_load() {
    assert_source_nested_binary_anf_entry_rejects("fn main() -> Bool { ((1 + 2) >= (2 * 3)) }");
}

#[test]
fn source_entry_nested_binary_anf_rejects_a_primitive_operator_tamper_during_corpus_load() {
    assert_source_nested_binary_anf_entry_rejects("fn main() -> Bool { (1 + 2) > (2 * 3) }");
}

#[test]
fn source_entry_nested_binary_anf_rejects_an_operand_tamper_during_corpus_load() {
    assert_source_nested_binary_anf_entry_rejects("fn main() -> Bool { (1 + 2) >= (3 * 2) }");
}

#[test]
fn source_entry_nested_binary_anf_rejects_a_letprim_spine_tamper_during_corpus_load() {
    assert_source_nested_binary_anf_entry_rejects("fn main() -> Bool { (1 + 2) >= 6 }");
}

const COMPUTED_BINARY_LET_SOURCE: &str = "fn main() -> Int { do { let __checked_add_result = 99; let computed = (1 + 2) * 3; return computed + 4; } }";
const COMPUTED_BINARY_LET_CASE_ID: &str = "phase202-source-computed-binary-let-bridge-return-13";

#[test]
fn source_computed_binary_let_fixture_compares_only_its_exact_private_differential_witness() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(COMPUTED_BINARY_LET_SOURCE)
        .expect("computed binary let source parses");
    engine
        .check(&mut entry)
        .expect("computed binary let source typechecks before checked CPS inspection");
    let lowered = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("computed binary let source lowers to checked CPS");

    let Term::LetVal {
        name: source_collision,
        value: CpsValue::Atom(CpsAtom::Int(99)),
        body: computed_rhs,
    } = lowered
    else {
        panic!("the collision binder must remain the outer LetVal");
    };
    assert_eq!(source_collision, "__checked_add_result");
    let Term::LetPrim {
        name: add_result,
        op: ash_core::cps::PrimOp::Add,
        args: add_args,
        body: multiply_body,
    } = *computed_rhs
    else {
        panic!("the computed RHS must start with Add(1, 2)");
    };
    assert_eq!(add_args, vec![CpsAtom::Int(1), CpsAtom::Int(2)]);
    assert_ne!(add_result, source_collision);
    let Term::LetPrim {
        name: multiply_result,
        op: ash_core::cps::PrimOp::Mul,
        args: multiply_args,
        body: computed_binding,
    } = *multiply_body
    else {
        panic!("the computed RHS must next multiply the Add result by 3");
    };
    assert_eq!(
        multiply_args,
        vec![CpsAtom::Var(add_result), CpsAtom::Int(3)]
    );
    let Term::LetVal {
        name: computed,
        value: CpsValue::Atom(CpsAtom::Var(bound_computed_result)),
        body: final_add_body,
    } = *computed_binding
    else {
        panic!("the computed source binder must bind the multiplication result");
    };
    assert_eq!(computed, "computed");
    assert_eq!(bound_computed_result, multiply_result);
    let Term::LetPrim {
        name: final_add_result,
        op: ash_core::cps::PrimOp::Add,
        args: final_add_args,
        body: answer_jump,
    } = *final_add_body
    else {
        panic!("the final body must add four to the computed source binder");
    };
    assert_eq!(
        final_add_args,
        vec![CpsAtom::Var("computed".to_string()), CpsAtom::Int(4)]
    );
    assert!(matches!(
        *answer_jump,
        Term::Jump {
            cont: ash_core::cps::ContRef::Label(ref answer),
            arg: CpsAtom::Var(ref result),
            ..
        } if answer == "__answer" && result == &final_add_result
    ));

    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");
    let report = harness.run_case(
        COMPUTED_BINARY_LET_CASE_ID,
        RustExecutionTarget::DirectRuntime,
    );
    assert_eq!(
        report.actual_result(),
        Some(&serde_json::json!({
            "outcome_class": "return",
            "payload": {"kind": "value", "value": {"type": "int", "value": 13}},
        })),
        "the direct differential oracle must observe the computed binary let as Int(13)"
    );
    assert!(matches!(
        report.checked_core_cps_relation(),
        crate::differential::RelationStatus::Passed
    ));
}

fn copy_computed_binary_let_fixture(root: &std::path::Path) -> std::path::PathBuf {
    let source_case = corpus_root().join(COMPUTED_BINARY_LET_CASE_ID);
    let copied_case = root.join(COMPUTED_BINARY_LET_CASE_ID);
    fs::create_dir_all(&copied_case).expect("copied computed binary let fixture directory created");
    for file in ["case.json", "input.ir.json", "expected.json"] {
        fs::copy(source_case.join(file), copied_case.join(file))
            .expect("computed binary let fixture file copied into temporary corpus");
    }
    copied_case
}

fn assert_computed_binary_let_source_entry_rejects(replacement: &str) {
    let root = tempfile::tempdir().expect("temporary corpus root created");
    let copied_case = copy_computed_binary_let_fixture(root.path());
    let input_path = copied_case.join("input.ir.json");
    let input = fs::read_to_string(&input_path).expect("copied computed binary let input read");
    let altered = input.replace(COMPUTED_BINARY_LET_SOURCE, replacement);
    assert_ne!(
        altered, input,
        "the temporary control must alter the exact source"
    );
    fs::write(input_path, altered).expect("tampered computed binary let input written");

    let error = DifferentialHarness::load(root.path()).expect_err(
        "a tampered computed binary let source-entry claim must reject before either target executes",
    );
    assert!(
        error.to_string().contains(&format!(
            "{COMPUTED_BINARY_LET_CASE_ID} cannot claim SEM-CPS-PRIM-001 source-entry values"
        )),
        "unexpected computed binary let rejection: {error}"
    );
}

#[test]
fn source_entry_computed_binary_let_rejects_a_source_text_tamper_during_corpus_load() {
    assert_computed_binary_let_source_entry_rejects(&format!("{COMPUTED_BINARY_LET_SOURCE} "));
}

#[test]
fn source_entry_computed_binary_let_rejects_a_collision_binder_tamper_during_corpus_load() {
    assert_computed_binary_let_source_entry_rejects(
        "fn main() -> Int { do { let collision = 99; let computed = (1 + 2) * 3; return computed + 4; } }",
    );
}

#[test]
fn source_entry_computed_binary_let_rejects_an_operand_tamper_during_corpus_load() {
    assert_computed_binary_let_source_entry_rejects(
        "fn main() -> Int { do { let __checked_add_result = 99; let computed = (1 + 3) * 3; return computed + 4; } }",
    );
}

#[test]
fn source_entry_computed_binary_let_rejects_an_operand_order_tamper_during_corpus_load() {
    assert_computed_binary_let_source_entry_rejects(
        "fn main() -> Int { do { let __checked_add_result = 99; let computed = (2 + 1) * 3; return computed + 4; } }",
    );
}

#[test]
fn source_entry_computed_binary_let_rejects_a_final_binding_tamper_during_corpus_load() {
    assert_computed_binary_let_source_entry_rejects(
        "fn main() -> Int { do { let __checked_add_result = 99; let computed = (1 + 2) * 3; return __checked_add_result + 4; } }",
    );
}

#[test]
fn source_entry_computed_binary_let_rejects_a_checked_schema_tamper_during_corpus_load() {
    let root = tempfile::tempdir().expect("temporary corpus root created");
    let copied_case = copy_computed_binary_let_fixture(root.path());
    let input_path = copied_case.join("input.ir.json");
    let input = fs::read_to_string(&input_path).expect("copied computed binary let input read");
    let altered = input.replace(
        "\"source_entry\": true,",
        "\"schema_version\": \"ash-cps-kernel-input/v1\",\n    \"source_entry\": true,",
    );
    assert_ne!(
        altered, input,
        "the temporary control must add a checked schema version"
    );
    fs::write(input_path, altered).expect("schema-tampered computed binary let input written");

    let error = DifferentialHarness::load(root.path()).expect_err(
        "a source-entry computed binary let must not accept a checked Core/CPS schema version",
    );
    assert!(
        error
            .to_string()
            .contains("source-entry checked Core/CPS input must not declare `schema_version`"),
        "unexpected computed binary let schema rejection: {error}"
    );
}

#[test]
fn source_bool_not_fixture_compares_bridge_derived_primitive_values_under_the_primitive_rule() {
    const SOURCE: &str = "fn main() -> Bool { !true }";

    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine.parse(SOURCE).expect("Boolean-not source parses");
    engine
        .check(&mut entry)
        .expect("Boolean-not source typechecks before checked CPS inspection");
    let lowered = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("Boolean-not source lowers to checked CPS");
    let Term::LetPrim {
        name,
        op: ash_core::cps::PrimOp::Not,
        args,
        body,
    } = lowered
    else {
        panic!("the checked source bridge must lower !true through LetPrim(Not)");
    };
    assert_eq!(args, vec![CpsAtom::Bool(true)]);
    assert!(matches!(
        *body,
        Term::Jump {
            cont: ash_core::cps::ContRef::Label(ref answer),
            arg: CpsAtom::Var(ref result),
            ..
        } if answer == "__answer" && result == &name
    ));

    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");
    let report = harness.run_case(
        "phase202-source-bool-not-bridge-return-false",
        RustExecutionTarget::DirectRuntime,
    );
    let parity = report.parity_report();

    assert_eq!(
        report.actual_result(),
        Some(&serde_json::json!({
            "outcome_class": "return",
            "payload": {"kind": "value", "value": {"type": "bool", "value": false}},
        })),
        "the direct differential oracle must observe !true as Bool(false)"
    );
    assert!(matches!(
        report.checked_core_cps_relation(),
        crate::differential::RelationStatus::Passed
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
fn source_complementary_bool_not_fixture_compares_bridge_derived_primitive_values_under_the_primitive_rule()
 {
    const SOURCE: &str = "fn main() -> Bool { !false }";

    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(SOURCE)
        .expect("complementary Boolean-not source parses");
    engine
        .check(&mut entry)
        .expect("complementary Boolean-not source typechecks before checked CPS inspection");
    let lowered = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("complementary Boolean-not source lowers to checked CPS");
    let Term::LetPrim {
        name,
        op: ash_core::cps::PrimOp::Not,
        args,
        body,
    } = lowered
    else {
        panic!("the checked source bridge must lower !false through LetPrim(Not)");
    };
    assert_eq!(args, vec![CpsAtom::Bool(false)]);
    assert!(matches!(
        *body,
        Term::Jump {
            cont: ash_core::cps::ContRef::Label(ref answer),
            arg: CpsAtom::Var(ref result),
            ..
        } if answer == "__answer" && result == &name
    ));

    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");
    let report = harness.run_case(
        "phase202-source-bool-not-bridge-return-true",
        RustExecutionTarget::DirectRuntime,
    );
    let parity = report.parity_report();

    assert_eq!(
        report.actual_result(),
        Some(&serde_json::json!({
            "outcome_class": "return",
            "payload": {"kind": "value", "value": {"type": "bool", "value": true}},
        })),
        "the direct differential oracle must observe !false as Bool(true)"
    );
    assert!(matches!(
        report.checked_core_cps_relation(),
        crate::differential::RelationStatus::Passed
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

const NESTED_BOOL_NOT_SOURCE: &str = "fn main() -> Bool { !!true }";
const NESTED_BOOL_NOT_CASE_ID: &str = "phase202-source-nested-bool-not-bridge-return-true";

#[test]
fn source_nested_bool_not_fixture_compares_only_its_exact_private_differential_witness() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(NESTED_BOOL_NOT_SOURCE)
        .expect("nested Boolean-not source parses");
    engine
        .check(&mut entry)
        .expect("nested Boolean-not source typechecks before checked CPS inspection");
    let lowered = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("nested Boolean-not source lowers to checked CPS");

    let Term::LetPrim {
        name: first_result,
        op: ash_core::cps::PrimOp::Not,
        args: first_args,
        body: first_body,
    } = lowered
    else {
        panic!("the nested Boolean-not source must first lower through LetPrim(Not)");
    };
    assert_eq!(first_args, vec![CpsAtom::Bool(true)]);
    let Term::LetPrim {
        name: second_result,
        op: ash_core::cps::PrimOp::Not,
        args: second_args,
        body: answer_jump,
    } = *first_body
    else {
        panic!("the first Boolean-not result must feed a second LetPrim(Not)");
    };
    assert_eq!(second_args, vec![CpsAtom::Var(first_result)]);
    assert!(matches!(
        *answer_jump,
        Term::Jump {
            cont: ash_core::cps::ContRef::Label(ref answer),
            arg: CpsAtom::Var(ref result),
            ..
        } if answer == "__answer" && result == &second_result
    ));

    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");
    let report = harness.run_case(NESTED_BOOL_NOT_CASE_ID, RustExecutionTarget::DirectRuntime);
    let parity = report.parity_report();

    assert_eq!(
        report.actual_result(),
        Some(&serde_json::json!({
            "outcome_class": "return",
            "payload": {"kind": "value", "value": {"type": "bool", "value": true}},
        })),
        "the direct differential oracle must observe !!true as Bool(true)"
    );
    assert!(matches!(
        report.checked_core_cps_relation(),
        crate::differential::RelationStatus::Passed
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

fn copy_nested_bool_not_fixture(root: &std::path::Path) -> std::path::PathBuf {
    let source_case = corpus_root().join(NESTED_BOOL_NOT_CASE_ID);
    let copied_case = root.join(NESTED_BOOL_NOT_CASE_ID);
    fs::create_dir_all(&copied_case).expect("copied nested Boolean-not fixture directory created");
    for file in ["case.json", "input.ir.json", "expected.json"] {
        fs::copy(source_case.join(file), copied_case.join(file))
            .expect("nested Boolean-not fixture file copied into temporary corpus");
    }
    copied_case
}

fn assert_nested_bool_not_source_entry_rejects(replacement: &str) {
    let root = tempfile::tempdir().expect("temporary corpus root created");
    let copied_case = copy_nested_bool_not_fixture(root.path());
    let input_path = copied_case.join("input.ir.json");
    let input = fs::read_to_string(&input_path).expect("copied nested Boolean-not input read");
    let altered = input.replace(NESTED_BOOL_NOT_SOURCE, replacement);
    assert_ne!(
        altered, input,
        "the temporary control must alter the exact nested Boolean-not source"
    );
    fs::write(input_path, altered).expect("tampered nested Boolean-not input written");

    let error = DifferentialHarness::load(root.path()).expect_err(
        "a tampered nested Boolean-not source-entry claim must reject before either target executes",
    );
    assert!(
        error.to_string().contains(&format!(
            "{NESTED_BOOL_NOT_CASE_ID} cannot claim SEM-CPS-PRIM-001 source-entry values"
        )),
        "unexpected nested Boolean-not rejection: {error}"
    );
    assert!(
        error.to_string().contains(
            "source does not match this nested Boolean-not fixture's exact canonical witness"
        ),
        "the exact nested Boolean-not source identity must gate every private witness: {error}"
    );
}

#[test]
fn source_entry_nested_bool_not_rejects_a_source_text_tamper_during_corpus_load() {
    assert_nested_bool_not_source_entry_rejects(&format!("{NESTED_BOOL_NOT_SOURCE} "));
}

#[test]
fn source_entry_nested_bool_not_rejects_an_operand_tamper_during_corpus_load() {
    assert_nested_bool_not_source_entry_rejects("fn main() -> Bool { !!false }");
}

#[test]
fn source_entry_nested_bool_not_rejects_a_letprim_spine_tamper_during_corpus_load() {
    assert_nested_bool_not_source_entry_rejects("fn main() -> Bool { !true }");
}

#[test]
fn source_entry_nested_bool_not_rejects_a_checked_schema_tamper_during_corpus_load() {
    let root = tempfile::tempdir().expect("temporary corpus root created");
    let copied_case = copy_nested_bool_not_fixture(root.path());
    let input_path = copied_case.join("input.ir.json");
    let input = fs::read_to_string(&input_path).expect("copied nested Boolean-not input read");
    let altered = input.replace(
        "\"source_entry\": true,",
        "\"schema_version\": \"ash-cps-kernel-input/v1\",\n    \"source_entry\": true,",
    );
    assert_ne!(
        altered, input,
        "the temporary control must add a checked schema version"
    );
    fs::write(input_path, altered).expect("schema-tampered nested Boolean-not input written");

    let error = DifferentialHarness::load(root.path()).expect_err(
        "a source-entry nested Boolean-not case must not accept a checked Core/CPS schema version",
    );
    assert!(
        error
            .to_string()
            .contains("source-entry checked Core/CPS input must not declare `schema_version`"),
        "unexpected nested Boolean-not schema rejection: {error}"
    );
}

fn assert_source_int_sub_entry_rejects(replacement: &str) {
    let case_id = "phase202-source-int-sub-bridge-return-5";
    let source_case = corpus_root().join(case_id);
    let root = tempfile::tempdir().expect("temporary corpus root created");
    let copied_case = root.path().join(case_id);
    fs::create_dir_all(&copied_case).expect("copied subtraction fixture directory created");
    for file in ["case.json", "input.ir.json", "expected.json"] {
        fs::copy(source_case.join(file), copied_case.join(file))
            .expect("subtraction fixture file copied into temporary corpus");
    }

    let input_path = copied_case.join("input.ir.json");
    let input = fs::read_to_string(&input_path).expect("copied subtraction input read");
    let altered = input.replace("fn main() -> Int { 7 - 2 }", replacement);
    assert_ne!(
        altered, input,
        "the temporary control must alter the exact subtraction source"
    );
    fs::write(input_path, altered).expect("altered subtraction input written");

    let error = DifferentialHarness::load(root.path()).expect_err(
        "an altered subtraction source-entry claim must reject before direct or checked execution",
    );
    assert!(
        error.to_string().contains(&format!(
            "{case_id} cannot claim SEM-CPS-PRIM-001 source-entry values"
        )),
        "unexpected {case_id} rejection: {error}"
    );
}

#[test]
fn source_entry_int_sub_rejects_swapped_operands_during_corpus_load() {
    assert_source_int_sub_entry_rejects("fn main() -> Int { 2 - 7 }");
}

#[test]
fn source_entry_int_sub_rejects_a_wrong_primitive_operator_during_corpus_load() {
    assert_source_int_sub_entry_rejects("fn main() -> Int { 7 + 2 }");
}

fn assert_complementary_bool_not_source_entry_rejects(replacement: &str) {
    let case_id = "phase202-source-bool-not-bridge-return-true";
    let source_case = corpus_root().join(case_id);
    let root = tempfile::tempdir().expect("temporary corpus root created");
    let copied_case = root.path().join(case_id);
    fs::create_dir_all(&copied_case).expect("copied fixture directory created");
    for file in ["case.json", "input.ir.json", "expected.json"] {
        fs::copy(source_case.join(file), copied_case.join(file))
            .expect("complementary Boolean-not fixture file copied into temporary corpus");
    }

    let input_path = copied_case.join("input.ir.json");
    let input = fs::read_to_string(&input_path).expect("copied Boolean-not input read");
    let altered = input.replace("fn main() -> Bool { !false }", replacement);
    assert_ne!(
        altered, input,
        "the temporary control must alter only the unary source form"
    );
    fs::write(input_path, altered).expect("altered complementary Boolean-not input written");

    let error = DifferentialHarness::load(root.path()).expect_err(
        "an altered or nested complementary Boolean-not source-entry claim must reject before execution",
    );
    assert!(
        error.to_string().contains(&format!(
            "{case_id} cannot claim SEM-CPS-PRIM-001 source-entry values"
        )),
        "unexpected {case_id} rejection: {error}"
    );
}

#[test]
fn source_entry_complementary_bool_not_rejects_an_altered_literal_during_corpus_load() {
    assert_complementary_bool_not_source_entry_rejects("fn main() -> Bool { !true }");
}

#[test]
fn source_entry_complementary_bool_not_rejects_a_nested_form_during_corpus_load() {
    assert_complementary_bool_not_source_entry_rejects("fn main() -> Bool { !!false }");
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
        crate::differential::RelationStatus::Passed
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
fn source_lexical_bool_not_fixture_preserves_letval_then_not_before_primitive_value_parity() {
    const SOURCE: &str = "fn main() -> Bool { do { let flag = true; return !flag; } }";

    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(SOURCE)
        .expect("lexical Boolean-not source parses");
    engine
        .check(&mut entry)
        .expect("lexical Boolean-not source typechecks before checked CPS inspection");
    let lowered = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("lexical Boolean-not source lowers to checked CPS");
    let Term::LetVal {
        name: flag,
        value: CpsValue::Atom(CpsAtom::Bool(true)),
        body: flag_body,
    } = lowered
    else {
        panic!("the checked source bridge must preserve flag = true as LetVal");
    };
    assert_eq!(flag, "flag");
    let Term::LetPrim {
        name: result,
        op: ash_core::cps::PrimOp::Not,
        args,
        body,
    } = *flag_body
    else {
        panic!("the lexical binding must enclose LetPrim(Not)");
    };
    assert_eq!(args, vec![CpsAtom::Var("flag".to_string())]);
    assert!(matches!(
        *body,
        Term::Jump {
            cont: ash_core::cps::ContRef::Label(ref answer),
            arg: CpsAtom::Var(ref argument),
            ..
        } if answer == "__answer" && argument == &result
    ));

    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");
    let report = harness.run_case(
        "phase202-source-lexical-bool-not-bridge-return-false",
        RustExecutionTarget::DirectRuntime,
    );
    let parity = report.parity_report();

    assert_eq!(
        report.actual_result(),
        Some(&serde_json::json!({
            "outcome_class": "return",
            "payload": {"kind": "value", "value": {"type": "bool", "value": false}},
        })),
        "the direct differential oracle must observe the lexical !flag as Bool(false)"
    );
    assert!(matches!(
        report.checked_core_cps_relation(),
        crate::differential::RelationStatus::Passed
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
fn source_lexical_false_bool_not_fixture_preserves_letval_then_not_before_primitive_value_parity() {
    const SOURCE: &str = "fn main() -> Bool { do { let flag = false; return !flag; } }";

    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(SOURCE)
        .expect("lexical false Boolean-not source parses");
    engine
        .check(&mut entry)
        .expect("lexical false Boolean-not source typechecks before checked CPS inspection");
    let lowered = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("lexical false Boolean-not source lowers to checked CPS");
    let Term::LetVal {
        name: flag,
        value: CpsValue::Atom(CpsAtom::Bool(false)),
        body: flag_body,
    } = lowered
    else {
        panic!("the checked source bridge must preserve flag = false as LetVal");
    };
    assert_eq!(flag, "flag");
    let Term::LetPrim {
        name: result,
        op: ash_core::cps::PrimOp::Not,
        args,
        body,
    } = *flag_body
    else {
        panic!("the lexical binding must enclose LetPrim(Not)");
    };
    assert_eq!(args, vec![CpsAtom::Var("flag".to_string())]);
    assert!(matches!(
        *body,
        Term::Jump {
            cont: ash_core::cps::ContRef::Label(ref answer),
            arg: CpsAtom::Var(ref argument),
            ..
        } if answer == "__answer" && argument == &result
    ));

    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");
    let report = harness.run_case(
        "phase202-source-lexical-bool-not-bridge-return-true",
        RustExecutionTarget::DirectRuntime,
    );
    let parity = report.parity_report();

    assert_eq!(
        report.actual_result(),
        Some(&serde_json::json!({
            "outcome_class": "return",
            "payload": {"kind": "value", "value": {"type": "bool", "value": true}},
        })),
        "the direct differential oracle must observe the lexical !flag as Bool(true)"
    );
    assert!(matches!(
        report.checked_core_cps_relation(),
        crate::differential::RelationStatus::Passed
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

fn copy_lexical_bool_not_fixture(root: &std::path::Path) -> std::path::PathBuf {
    let case_id = "phase202-source-lexical-bool-not-bridge-return-false";
    let source_case = corpus_root().join(case_id);
    let copied_case = root.join(case_id);
    fs::create_dir_all(&copied_case).expect("copied lexical Boolean-not fixture directory created");
    for file in ["case.json", "input.ir.json", "expected.json"] {
        fs::copy(source_case.join(file), copied_case.join(file))
            .expect("lexical Boolean-not fixture file copied into temporary corpus");
    }
    copied_case
}

fn copy_lexical_false_bool_not_fixture(root: &std::path::Path) -> std::path::PathBuf {
    let case_id = "phase202-source-lexical-bool-not-bridge-return-true";
    let source_case = corpus_root().join(case_id);
    let copied_case = root.join(case_id);
    fs::create_dir_all(&copied_case)
        .expect("copied lexical false Boolean-not fixture directory created");
    for file in ["case.json", "input.ir.json", "expected.json"] {
        fs::copy(source_case.join(file), copied_case.join(file))
            .expect("lexical false Boolean-not fixture file copied into temporary corpus");
    }
    copied_case
}

#[test]
fn source_entry_lexical_false_bool_not_rejects_a_tampered_source_before_execution() {
    const SOURCE: &str = "fn main() -> Bool { do { let flag = false; return !flag; } }";
    let root = tempfile::tempdir().expect("temporary corpus root created");
    let copied_case = copy_lexical_false_bool_not_fixture(root.path());
    let input_path = copied_case.join("input.ir.json");
    let input =
        fs::read_to_string(&input_path).expect("copied lexical false Boolean-not input read");
    let altered = input.replace(
        SOURCE,
        "fn main() -> Bool { do { let flag = false; return !flag; } } ",
    );
    assert_ne!(
        altered, input,
        "the control must alter the exact source text"
    );
    fs::write(input_path, altered).expect("tampered lexical false Boolean-not input written");

    let error = DifferentialHarness::load(root.path()).expect_err(
        "a non-canonical lexical false Boolean-not source must reject before either target executes",
    );
    assert!(
        error.to_string().contains(
            "phase202-source-lexical-bool-not-bridge-return-true cannot claim SEM-CPS-PRIM-001 source-entry values"
        ),
        "unexpected lexical false source-tamper rejection: {error}"
    );
}

#[test]
fn source_entry_lexical_false_bool_not_rejects_a_tampered_binder_before_execution() {
    const SOURCE: &str = "fn main() -> Bool { do { let flag = false; return !flag; } }";
    let root = tempfile::tempdir().expect("temporary corpus root created");
    let copied_case = copy_lexical_false_bool_not_fixture(root.path());
    let input_path = copied_case.join("input.ir.json");
    let input =
        fs::read_to_string(&input_path).expect("copied lexical false Boolean-not input read");
    let altered = input.replace(
        SOURCE,
        "fn main() -> Bool { do { let value = false; return !value; } }",
    );
    assert_ne!(altered, input, "the control must alter the lexical binder");
    fs::write(input_path, altered).expect("tampered lexical false Boolean-not input written");

    let error = DifferentialHarness::load(root.path()).expect_err(
        "a tampered lexical false Boolean-not binder must reject before either target executes",
    );
    assert!(
        error.to_string().contains(
            "phase202-source-lexical-bool-not-bridge-return-true cannot claim SEM-CPS-PRIM-001 source-entry values"
        ),
        "unexpected lexical false binder-tamper rejection: {error}"
    );
}

#[test]
fn source_entry_lexical_false_bool_not_rejects_an_unbound_case_identity_before_execution() {
    let root = tempfile::tempdir().expect("temporary corpus root created");
    let copied_case = copy_lexical_false_bool_not_fixture(root.path());
    let original_case_id = "phase202-source-lexical-bool-not-bridge-return-true";
    let unbound_case_id = "phase202-source-lexical-bool-not-unbound-case";
    let renamed_case = root.path().join(unbound_case_id);
    fs::rename(&copied_case, &renamed_case).expect("temporary fixture directory renamed");
    for file in ["case.json", "expected.json"] {
        let path = renamed_case.join(file);
        let contents = fs::read_to_string(&path).expect("copied fixture metadata read");
        let altered = contents.replace(original_case_id, unbound_case_id);
        assert_ne!(
            altered, contents,
            "the control must alter the fixture identity"
        );
        fs::write(path, altered).expect("unbound fixture metadata written");
    }

    let error = DifferentialHarness::load(root.path()).expect_err(
        "an exact lexical false Boolean-not source must not claim primitive parity under an unbound case identity",
    );
    assert!(
        error.to_string().contains(
            "phase202-source-lexical-bool-not-unbound-case cannot claim SEM-CPS-PRIM-001 source-entry values"
        ),
        "unexpected unbound lexical false case rejection: {error}"
    );
}

#[test]
fn source_entry_lexical_false_bool_not_rejects_a_tampered_nested_not_before_execution() {
    const SOURCE: &str = "fn main() -> Bool { do { let flag = false; return !flag; } }";
    let root = tempfile::tempdir().expect("temporary corpus root created");
    let copied_case = copy_lexical_false_bool_not_fixture(root.path());
    let input_path = copied_case.join("input.ir.json");
    let input =
        fs::read_to_string(&input_path).expect("copied lexical false Boolean-not input read");
    let altered = input.replace(
        SOURCE,
        "fn main() -> Bool { do { let flag = false; return !!flag; } }",
    );
    assert_ne!(altered, input, "the control must alter unary nesting");
    fs::write(input_path, altered).expect("nested lexical false Boolean-not input written");

    let error = DifferentialHarness::load(root.path()).expect_err(
        "a nested lexical false Boolean-not source-entry claim must reject before either target executes",
    );
    assert!(
        error.to_string().contains(
            "phase202-source-lexical-bool-not-bridge-return-true cannot claim SEM-CPS-PRIM-001 source-entry values"
        ),
        "unexpected lexical false nested-Not rejection: {error}"
    );
}

#[test]
fn source_entry_lexical_bool_not_rejects_a_tampered_binding_before_execution() {
    const SOURCE: &str = "fn main() -> Bool { do { let flag = true; return !flag; } }";
    let root = tempfile::tempdir().expect("temporary corpus root created");
    let copied_case = copy_lexical_bool_not_fixture(root.path());
    let input_path = copied_case.join("input.ir.json");
    let input = fs::read_to_string(&input_path).expect("copied lexical Boolean-not input read");
    let altered = input.replace(
        SOURCE,
        "fn main() -> Bool { do { let flag = false; return !flag; } }",
    );
    assert_ne!(altered, input, "the control must alter the lexical binding");
    fs::write(input_path, altered).expect("tampered lexical Boolean-not input written");

    let error = DifferentialHarness::load(root.path()).expect_err(
        "a tampered lexical Boolean-not source-entry claim must reject before either target executes",
    );
    assert!(
        error.to_string().contains(
            "phase202-source-lexical-bool-not-bridge-return-false cannot claim SEM-CPS-PRIM-001 source-entry values"
        ),
        "unexpected lexical-binding tamper rejection: {error}"
    );
}

#[test]
fn source_entry_lexical_bool_not_rejects_an_unbound_case_identity_before_execution() {
    let root = tempfile::tempdir().expect("temporary corpus root created");
    let copied_case = copy_lexical_bool_not_fixture(root.path());
    let original_case_id = "phase202-source-lexical-bool-not-bridge-return-false";
    let unbound_case_id = "phase202-source-lexical-bool-not-unbound-case";
    let renamed_case = root.path().join(unbound_case_id);
    fs::rename(&copied_case, &renamed_case).expect("temporary fixture directory renamed");
    for file in ["case.json", "expected.json"] {
        let path = renamed_case.join(file);
        let contents = fs::read_to_string(&path).expect("copied fixture metadata read");
        let altered = contents.replace(original_case_id, unbound_case_id);
        assert_ne!(
            altered, contents,
            "the control must alter the fixture identity"
        );
        fs::write(path, altered).expect("unbound fixture metadata written");
    }

    let error = DifferentialHarness::load(root.path()).expect_err(
        "an exact lexical Boolean-not source must not claim primitive parity under an unbound case identity",
    );
    assert!(
        error.to_string().contains(
            "phase202-source-lexical-bool-not-unbound-case cannot claim SEM-CPS-PRIM-001 source-entry values"
        ),
        "unexpected unbound lexical-case rejection: {error}"
    );
}

#[test]
fn source_entry_lexical_bool_not_rejects_a_tampered_nested_not_before_execution() {
    const SOURCE: &str = "fn main() -> Bool { do { let flag = true; return !flag; } }";
    let root = tempfile::tempdir().expect("temporary corpus root created");
    let copied_case = copy_lexical_bool_not_fixture(root.path());
    let input_path = copied_case.join("input.ir.json");
    let input = fs::read_to_string(&input_path).expect("copied lexical Boolean-not input read");
    let altered = input.replace(
        SOURCE,
        "fn main() -> Bool { do { let flag = true; return !!flag; } }",
    );
    assert_ne!(altered, input, "the control must alter unary nesting");
    fs::write(input_path, altered).expect("nested lexical Boolean-not input written");

    let error = DifferentialHarness::load(root.path()).expect_err(
        "a nested lexical Boolean-not source-entry claim must reject before either target executes",
    );
    assert!(
        error.to_string().contains(
            "phase202-source-lexical-bool-not-bridge-return-false cannot claim SEM-CPS-PRIM-001 source-entry values"
        ),
        "unexpected lexical nested-Not rejection: {error}"
    );
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
        crate::differential::RelationStatus::Passed
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
        crate::differential::RelationStatus::Passed
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
        crate::differential::RelationStatus::Passed
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
        crate::differential::RelationStatus::Passed
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
        crate::differential::RelationStatus::Failed {
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
        crate::differential::RelationStatus::Failed {
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
        crate::differential::RelationStatus::Failed {
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
        crate::differential::RelationStatus::Passed
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
        crate::differential::RelationStatus::Passed
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
