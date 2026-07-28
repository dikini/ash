//! TASK-2021: fixed canonical-Core V1 `LetVal` control.
//!
//! This proves the second closed V1 artifact reaches only the checked,
//! private Core/CPS prototype.  It is not a production Core execution route.

use super::{CaseComparisonStatus, DifferentialHarness, RustExecutionTarget};
use ash_core::{
    core_ash::{CoreMultiplicity, CoreRow, CoreType},
    core_ash_lower::CoreLoweringContext,
    core_ash_typecheck::CoreTypeCheckEnv,
    cps::{
        Atom as CpsAtom, ContRef, EffectRow as CpsEffectRow, Term as CpsTerm, Value as CpsValue,
    },
};
use std::{fs, path::PathBuf};
use tempfile::TempDir;

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/differential/corpus")
}

fn letval_manifest(core_text: &str) -> String {
    format!(
        r#"{{
  "schema_version": "ash-canonical-core-fixture/v1",
  "case_id": "canonical-core-v1-letval-return-int-7",
  "target": "rust-checked-core-cps-prototype",
  "canonical_rule_ids": ["SEM-CPS-RETURN-001", "CONF-IMPLEMENTATION-001"],
  "core_text": "{core_text}"
}}"#
    )
}

fn load_error(core_text: &str) -> String {
    let corpus = TempDir::new().expect("temporary corpus directory");
    let case_dir = corpus.path().join("canonical-case");
    fs::create_dir(&case_dir).expect("fixture case directory");
    fs::write(
        case_dir.join("canonical-core.json"),
        letval_manifest(core_text),
    )
    .expect("fixture manifest");
    DifferentialHarness::load(corpus.path())
        .expect_err("altered closed V1 LetVal fixture must not load")
        .to_string()
}

#[test]
fn v1_letval_fixture_runs_only_through_checked_core_cps_and_returns_int_7() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");

    let checked = harness.run_case(
        "canonical-core-v1-letval-return-int-7",
        RustExecutionTarget::CheckedCoreCpsPrototype,
    );
    assert_eq!(
        checked.direct_runtime_status(),
        CaseComparisonStatus::Passed
    );
    assert_eq!(
        checked.actual_result(),
        Some(&serde_json::json!({
            "outcome_class": "return",
            "payload": {"kind": "value", "value": {"type": "int", "value": 7}},
        }))
    );

    let direct = harness.run_case(
        "canonical-core-v1-letval-return-int-7",
        RustExecutionTarget::DirectRuntime,
    );
    assert!(matches!(
        direct.direct_runtime_status(),
        CaseComparisonStatus::Unsupported { .. }
    ));
    assert!(direct.actual_result().is_none());
}

#[test]
fn v1_letval_fixture_checked_lowering_is_exactly_letval_then_answer_jump() {
    let text = "(let-val value : Int (lit-int 7) value)";
    let parsed = ash_core::core_ash_text::parse_core_expr(text).expect("canonical Core text");
    let validated = ash_core::core_ash_validate::validate_core_program(
        ash_core::core_ash_validate::RawCoreProgram::new(parsed),
    )
    .expect("canonical Core validates");
    let mut type_env = CoreTypeCheckEnv::default();
    type_env.continuations_mut().insert(
        "__answer",
        CoreType::Cont {
            input: Box::new(CoreType::Base("Int".to_string())),
            answer: Box::new(CoreType::Base("Unit".to_string())),
            row: CoreRow::default(),
            multiplicity: CoreMultiplicity::Affine,
        },
    );
    let checked = ash_core::core_ash_typecheck::type_check_and_lower_core_program(
        validated,
        &type_env,
        CoreLoweringContext::new(ContRef::Label("__answer".to_string()), CoreRow::default()),
    )
    .expect("canonical Core checks and lowers");
    let (_, lowered) = checked.into_parts();

    assert!(matches!(
        lowered,
        CpsTerm::LetVal {
            name,
            value: CpsValue::Atom(CpsAtom::Int(7)),
            body,
        } if name == "value" && matches!(
            body.as_ref(),
            CpsTerm::Jump {
                cont: ContRef::Label(answer),
                arg: CpsAtom::Var(bound),
                row,
            } if answer == "__answer" && bound == "value" && row == &CpsEffectRow::default()
        )
    ));
}

#[test]
fn v1_letval_control_rejects_every_altered_structure_before_comparison() {
    for (name, core_text) in [
        ("binder", "(let-val other : Int (lit-int 7) other)"),
        ("literal", "(let-val value : Int (lit-int 8) value)"),
        ("annotation", "(let-val value : Bool (lit-int 7) value)"),
        ("body variable", "(let-val value : Int (lit-int 7) other)"),
        ("structural form", "(lit-int 7)"),
    ] {
        let error = load_error(core_text);
        assert!(
            error.contains("canonical Core"),
            "altered {name} must reject before terminal comparison: {error}"
        );
    }
}
