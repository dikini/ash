//! TASK-1688: Core text fixtures for continuation multiplicity.

use std::path::{Path, PathBuf};

use ash_core::core_ash::{CoreEffectOp, CoreExpr, CoreMultiplicity, CoreType};
use ash_core::core_ash_lower::CoreLoweringContext;
use ash_core::core_ash_text::{core_expr_to_string, parse_core_file};
use ash_core::core_ash_typecheck::{CoreTypeCheckEnv, type_check_and_lower_core_program};
use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};
use ash_core::cps::{ContRef, Term};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("ash-core crate lives under crates/ash-core")
        .to_path_buf()
}

fn fixture_path(name: &str) -> PathBuf {
    repo_root()
        .join("crates/ash-core/tests/fixtures/core")
        .join(name)
}

fn string_ty() -> CoreType {
    CoreType::Base("String".to_string())
}

fn int_ty() -> CoreType {
    CoreType::Base("Int".to_string())
}

fn kv_read_op() -> CoreEffectOp {
    CoreEffectOp::Operation {
        path: vec!["kv".to_string()],
        operation: "read".to_string(),
        arg_types: vec![string_ty()],
        result_type: string_ty(),
    }
}

fn env_with_ops() -> CoreTypeCheckEnv {
    let mut env = CoreTypeCheckEnv::default();
    env.operations_mut().insert(kv_read_op());
    env
}

fn env_with_label_k() -> CoreTypeCheckEnv {
    let mut env = CoreTypeCheckEnv::default();
    env.continuations_mut().insert(
        "k".to_string(),
        CoreType::Cont {
            input: Box::new(int_ty()),
            answer: Box::new(int_ty()),
            row: Default::default(),
            multiplicity: CoreMultiplicity::Affine,
        },
    );
    env
}

fn round_trip_fixture(name: &str) -> CoreExpr {
    let expr = parse_core_file(fixture_path(name)).expect("fixture should parse");
    let text = core_expr_to_string(&expr);
    let reparsed =
        ash_core::core_ash_text::parse_core_expr(&text).expect("serialized fixture should parse");
    assert_eq!(reparsed, expr, "fixture should round-trip canonically");
    expr
}

fn first_let_cont_call(term: &Term) -> Option<&Term> {
    match term {
        Term::LetContCall { .. } => Some(term),
        Term::LetVal { body, .. }
        | Term::LetPrim { body, .. }
        | Term::LetRec { body, .. }
        | Term::RecordDischarge { body, .. }
        | Term::Handle { body, .. } => first_let_cont_call(body),
        Term::LetCont {
            cont_body, body, ..
        } => first_let_cont_call(cont_body).or_else(|| first_let_cont_call(body)),
        Term::If {
            then_branch,
            else_branch,
            ..
        } => first_let_cont_call(then_branch).or_else(|| first_let_cont_call(else_branch)),
        Term::Match { arms, default, .. } => arms
            .iter()
            .find_map(|(_, arm)| first_let_cont_call(arm))
            .or_else(|| default.as_deref().and_then(first_let_cont_call)),
        Term::Jump { .. }
        | Term::JumpValue { .. }
        | Term::Call { .. }
        | Term::Raise { .. }
        | Term::Return { .. }
        | Term::Trap { .. } => None,
    }
}

#[test]
fn legal_continuation_multiplicity_fixtures_round_trip_and_typecheck() {
    for fixture in [
        "multishot_resume_text_roundtrip.core",
        "affine_empty_row_remains_affine.core",
    ] {
        let expr = round_trip_fixture(fixture);
        let valid = validate_core_program(RawCoreProgram::new(expr))
            .expect("legal fixture should validate");
        type_check_and_lower_core_program(valid, &env_with_ops(), lowering_context())
            .expect("legal fixture should type-check and lower");
    }
}

#[test]
fn let_cont_call_fixture_round_trips_and_lowers_to_cps_let_cont_call() {
    let expr = round_trip_fixture("let_cont_call_text_roundtrip.core");
    let valid = validate_core_program(RawCoreProgram::new(expr)).expect("fixture should validate");
    let lowered = type_check_and_lower_core_program(valid, &env_with_label_k(), lowering_context())
        .expect("fixture should type-check and lower")
        .into_parts()
        .1;

    assert!(
        matches!(
            first_let_cont_call(&lowered),
            Some(Term::LetContCall { .. })
        ),
        "Core let-cont-call fixture should lower to CPS LetContCall"
    );
}

#[test]
fn invalid_multishot_nonempty_row_fixture_rejects_with_multiplicity_error() {
    let expr = parse_core_file(fixture_path("invalid_multishot_nonempty_row.core"))
        .expect("invalid fixture should still parse");
    let err = validate_core_program(RawCoreProgram::new(expr))
        .expect_err("multi-shot-pure with non-empty row should reject");

    assert!(
        err.to_string().contains("multi-shot-pure") && err.to_string().contains("closed empty row"),
        "unexpected error: {err}"
    );
}

#[test]
fn invalid_multishot_open_row_fixture_rejects_with_multiplicity_error() {
    let expr = parse_core_file(fixture_path("invalid_multishot_open_row.core"))
        .expect("invalid fixture should still parse");
    let err = validate_core_program(RawCoreProgram::new(expr))
        .expect_err("multi-shot-pure with open row should reject");

    assert!(
        err.to_string().contains("multi-shot-pure") && err.to_string().contains("closed empty row"),
        "unexpected error: {err}"
    );
}

fn lowering_context() -> CoreLoweringContext {
    CoreLoweringContext::new(ContRef::Label("halt".to_string()), Default::default())
}
