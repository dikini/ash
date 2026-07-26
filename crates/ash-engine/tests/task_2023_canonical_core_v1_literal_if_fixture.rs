//! TASK-2023: fixed canonical-Core V1 literal `If` controls.
//!
//! These two controls exercise only the private checked Core/CPS prototype;
//! they do not establish general conditional execution or a production route.

use ash_core::{
    core_ash::{CoreAtom, CoreExpr, CoreMultiplicity, CoreRow, CoreType},
    core_ash_lower::CoreLoweringContext,
    core_ash_typecheck::CoreTypeCheckEnv,
    cps::{Atom as CpsAtom, ContRef, EffectRow as CpsEffectRow, Term as CpsTerm},
};
use ash_engine::differential::{CaseComparisonStatus, DifferentialHarness, RustExecutionTarget};
use std::{fs, path::PathBuf};
use tempfile::TempDir;

const IF_RULES: &str = "[\"SEM-CPS-IF-001\", \"SEM-CPS-RETURN-001\", \"CONF-IMPLEMENTATION-001\"]";

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/differential/corpus")
}

fn if_manifest(case_id: &str, rule_ids: &str, core_text: &str) -> String {
    let escaped_core_text = core_text.replace('\\', "\\\\").replace('"', "\\\"");
    format!(
        r#"{{
  "schema_version": "ash-canonical-core-fixture/v1",
  "case_id": "{case_id}",
  "target": "rust-checked-core-cps-prototype",
  "canonical_rule_ids": {rule_ids},
  "core_text": "{escaped_core_text}"
}}"#
    )
}

fn load_error(case_id: &str, rule_ids: &str, core_text: &str) -> String {
    let corpus = TempDir::new().expect("temporary corpus directory");
    let case_dir = corpus.path().join("canonical-case");
    fs::create_dir(&case_dir).expect("fixture case directory");
    fs::write(
        case_dir.join("canonical-core.json"),
        if_manifest(case_id, rule_ids, core_text),
    )
    .expect("fixture manifest");
    DifferentialHarness::load(corpus.path())
        .expect_err("altered closed V1 literal If fixture must not load")
        .to_string()
}

#[test]
fn v1_literal_if_fixtures_run_only_through_checked_core_cps_and_select_their_branch() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");

    for (case_id, selected) in [
        ("canonical-core-v1-if-true-return-int-7", 7),
        ("canonical-core-v1-if-false-return-int-9", 9),
    ] {
        let checked = harness.run_case(case_id, RustExecutionTarget::CheckedCoreCpsPrototype);
        assert_eq!(
            checked.direct_runtime_status(),
            CaseComparisonStatus::Passed,
            "{case_id} must only compare the selected private terminal"
        );
        assert_eq!(
            checked.actual_result(),
            Some(&serde_json::json!({
                "outcome_class": "return",
                "payload": {"kind": "value", "value": {"type": "int", "value": selected}},
            })),
            "{case_id} must project only its selected literal branch"
        );

        let direct = harness.run_case(case_id, RustExecutionTarget::DirectRuntime);
        assert!(matches!(
            direct.direct_runtime_status(),
            CaseComparisonStatus::Unsupported { .. }
        ));
        assert!(direct.actual_result().is_none());
    }
}

#[test]
fn v1_literal_if_controls_have_exact_parsed_and_checked_cps_if_shapes() {
    for (condition, selected) in [(true, 7), (false, 9)] {
        let text = format!("(if (lit-bool {condition}) (lit-int 7) (lit-int 9))");
        let parsed = ash_core::core_ash_text::parse_core_expr(&text).expect("canonical Core text");
        assert!(matches!(
            &parsed,
            CoreExpr::If { cond: CoreAtom::LitBool(value), then_branch, else_branch }
                if *value == condition
                    && matches!(then_branch.as_ref(), CoreExpr::Atom(CoreAtom::LitInt(7)))
                    && matches!(else_branch.as_ref(), CoreExpr::Atom(CoreAtom::LitInt(9)))
        ));
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
            CpsTerm::If { cond: CpsAtom::Bool(value), then_branch, else_branch, row }
                if value == condition
                    && row == CpsEffectRow::default()
                    && is_answer_jump(then_branch.as_ref(), 7)
                    && is_answer_jump(else_branch.as_ref(), 9)
        ));
        assert_eq!(selected, if condition { 7 } else { 9 });
    }
}

fn is_answer_jump(term: &CpsTerm, literal: i64) -> bool {
    matches!(
        term,
        CpsTerm::Jump { cont: ContRef::Label(answer), arg: CpsAtom::Int(value), row }
            if answer == "__answer" && *value == literal && row == &CpsEffectRow::default()
    )
}

#[test]
fn v1_literal_if_controls_reject_altered_identity_rules_and_text_before_comparison() {
    const TRUE_ID: &str = "canonical-core-v1-if-true-return-int-7";
    const TRUE_TEXT: &str = "(if (lit-bool true) (lit-int 7) (lit-int 9))";
    const FIXED_TEXT_ERROR: &str =
        "canonical Core V1 fixture must use the exact fixed text for its admitted control";

    for (name, case_id, rules, text, expected) in [
        (
            "unknown case",
            "canonical-core-v1-if-true-return-int-9",
            IF_RULES,
            TRUE_TEXT,
            "unsupported case ID",
        ),
        (
            "missing If evidence",
            TRUE_ID,
            "[\"SEM-CPS-RETURN-001\", \"CONF-IMPLEMENTATION-001\"]",
            TRUE_TEXT,
            "unsupported canonical rule",
        ),
        (
            "wrong rule order",
            TRUE_ID,
            "[\"SEM-CPS-RETURN-001\", \"SEM-CPS-IF-001\", \"CONF-IMPLEMENTATION-001\"]",
            TRUE_TEXT,
            "canonical rule",
        ),
        (
            "false condition",
            TRUE_ID,
            IF_RULES,
            "(if (lit-bool false) (lit-int 7) (lit-int 9))",
            FIXED_TEXT_ERROR,
        ),
        (
            "noncanonical true spelling",
            TRUE_ID,
            IF_RULES,
            "(if (lit-bool true)  (lit-int 7) (lit-int 9))",
            FIXED_TEXT_ERROR,
        ),
        (
            "condition type",
            TRUE_ID,
            IF_RULES,
            "(if (lit-int 1) (lit-int 7) (lit-int 9))",
            FIXED_TEXT_ERROR,
        ),
        (
            "then branch",
            TRUE_ID,
            IF_RULES,
            "(if (lit-bool true) (lit-int 8) (lit-int 9))",
            FIXED_TEXT_ERROR,
        ),
        (
            "else branch",
            TRUE_ID,
            IF_RULES,
            "(if (lit-bool true) (lit-int 7) (lit-int 8))",
            FIXED_TEXT_ERROR,
        ),
        (
            "branch order",
            TRUE_ID,
            IF_RULES,
            "(if (lit-bool true) (lit-int 9) (lit-int 7))",
            FIXED_TEXT_ERROR,
        ),
        (
            "branch form",
            TRUE_ID,
            IF_RULES,
            "(if (lit-bool true) 7 (lit-int 9))",
            FIXED_TEXT_ERROR,
        ),
    ] {
        let error = load_error(case_id, rules, text);
        assert!(
            error.contains(expected),
            "altered {name} must reject before terminal comparison or direct execution: {error}"
        );
    }
}
