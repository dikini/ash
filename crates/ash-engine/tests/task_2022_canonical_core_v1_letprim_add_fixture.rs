//! TASK-2022: fixed canonical-Core V1 `LetPrim(Add)` control.
//!
//! The fixture must traverse the private checked Core/CPS prototype and must
//! not turn the closed canonical-Core adapter into a general primitive loader.

use ash_core::{
    core_ash::{CoreMultiplicity, CoreRow, CoreType},
    core_ash_lower::CoreLoweringContext,
    core_ash_typecheck::CoreTypeCheckEnv,
    cps::{
        Atom as CpsAtom, ContRef, EffectRow as CpsEffectRow, PrimOp as CpsPrimOp, Term as CpsTerm,
    },
};
use ash_engine::differential::{CaseComparisonStatus, DifferentialHarness, RustExecutionTarget};
use std::{fs, path::PathBuf};
use tempfile::TempDir;

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/differential/corpus")
}

fn letprim_manifest(core_text: &str) -> String {
    let escaped_core_text = core_text.replace('\\', "\\\\").replace('"', "\\\"");
    format!(
        r#"{{
  "schema_version": "ash-canonical-core-fixture/v1",
  "case_id": "canonical-core-v1-letprim-add-return-int-7",
  "target": "rust-checked-core-cps-prototype",
  "canonical_rule_ids": ["SEM-CPS-PRIM-001", "SEM-CPS-RETURN-001", "CONF-IMPLEMENTATION-001"],
  "core_text": "{escaped_core_text}"
}}"#
    )
}

fn load_error(core_text: &str) -> String {
    let corpus = TempDir::new().expect("temporary corpus directory");
    let case_dir = corpus.path().join("canonical-case");
    fs::create_dir(&case_dir).expect("fixture case directory");
    fs::write(
        case_dir.join("canonical-core.json"),
        letprim_manifest(core_text),
    )
    .expect("fixture manifest");
    DifferentialHarness::load(corpus.path())
        .expect_err("altered closed V1 LetPrim fixture must not load")
        .to_string()
}

#[test]
fn v1_letprim_add_fixture_runs_only_through_checked_core_cps_and_returns_int_7() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");

    let checked = harness.run_case(
        "canonical-core-v1-letprim-add-return-int-7",
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
        "canonical-core-v1-letprim-add-return-int-7",
        RustExecutionTarget::DirectRuntime,
    );
    assert!(matches!(
        direct.direct_runtime_status(),
        CaseComparisonStatus::Unsupported { .. }
    ));
    assert!(direct.actual_result().is_none());
}

#[test]
fn v1_letprim_add_fixture_checked_lowering_is_exactly_add_then_answer_jump() {
    let text = "(let-prim sum add ((lit-int 2) (lit-int 5)) sum)";
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
        CpsTerm::LetPrim {
            name,
            op: CpsPrimOp::Add,
            args,
            body,
        } if name == "sum" && args == vec![CpsAtom::Int(2), CpsAtom::Int(5)] && matches!(
            body.as_ref(),
            CpsTerm::Jump {
                cont: ContRef::Label(answer),
                arg: CpsAtom::Var(bound),
                row,
            } if answer == "__answer" && bound == "sum" && row == &CpsEffectRow::default()
        )
    ));
}

#[test]
fn v1_letprim_add_control_rejects_every_altered_structure_before_comparison() {
    for (name, core_text) in [
        (
            "binder",
            "(let-prim other add ((lit-int 2) (lit-int 5)) other)",
        ),
        (
            "primitive",
            "(let-prim sum sub ((lit-int 2) (lit-int 5)) sum)",
        ),
        ("arity", "(let-prim sum add ((lit-int 2)) sum)"),
        (
            "left operand",
            "(let-prim sum add ((lit-int 3) (lit-int 5)) sum)",
        ),
        (
            "right operand",
            "(let-prim sum add ((lit-int 2) (lit-int 4)) sum)",
        ),
        ("literal form", "(let-prim sum add (2 (lit-int 5)) sum)"),
        (
            "body",
            "(let-prim sum add ((lit-int 2) (lit-int 5)) (lit-int 7))",
        ),
        ("structural form", "(lit-int 7)"),
    ] {
        let error = load_error(core_text);
        assert!(
            error.contains("canonical Core"),
            "altered {name} must reject before terminal comparison: {error}"
        );
    }
}

/// The fixed control is a fixed *textual* Core artifact, not merely one AST
/// shape. In particular, parser-normalized integer spellings must not widen
/// the canonical fixture route, and neither can a different literal type
/// reach a type-check phase that would obscure the identity failure.
#[test]
fn v1_letprim_add_control_rejects_noncanonical_literal_spellings_before_parsing() {
    const FIXED_TEXT_ERROR: &str =
        "canonical Core V1 fixture must use the exact fixed text for its admitted control";

    for (name, core_text) in [
        (
            "explicitly signed left integer",
            "(let-prim sum add ((lit-int +2) (lit-int 5)) sum)",
        ),
        (
            "zero-padded left integer",
            "(let-prim sum add ((lit-int 02) (lit-int 5)) sum)",
        ),
        (
            "string left operand",
            "(let-prim sum add ((lit-string \"2\") (lit-int 5)) sum)",
        ),
    ] {
        let error = load_error(core_text);
        assert!(
            error.contains(FIXED_TEXT_ERROR),
            "{name} must reject at fixed-text control admission before parsing, validation, type checking, or comparison: {error}"
        );
    }
}
