//! TASK-1689: Motivational multi-shot Core fixtures.

use std::path::{Path, PathBuf};

use ash_core::core_ash::{CoreEffectOp, CoreType};
use ash_core::core_ash_lower::CoreLoweringContext;
use ash_core::core_ash_text::parse_core_file;
use ash_core::core_ash_typecheck::{CoreTypeCheckEnv, type_check_and_lower_core_program};
use ash_core::core_ash_validate::{CoreValidationError, RawCoreProgram, validate_core_program};
use ash_core::cps::ContRef;

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

fn unit_ty() -> CoreType {
    CoreType::Base("Unit".to_string())
}

fn int_ty() -> CoreType {
    CoreType::Base("Int".to_string())
}

fn choice_op() -> CoreEffectOp {
    CoreEffectOp::Capability {
        path: vec!["choice".to_string()],
        operation: "pick".to_string(),
        arg_types: vec![unit_ty()],
        result_type: int_ty(),
    }
}

fn env_with_choice() -> CoreTypeCheckEnv {
    let mut env = CoreTypeCheckEnv::default();
    env.operations_mut().insert(choice_op());
    env
}

fn lowering_context() -> CoreLoweringContext {
    CoreLoweringContext::new(ContRef::Label("halt".to_string()), Default::default())
}

fn assert_positive_fixture_lowers(name: &str) {
    let expr = parse_core_file(fixture_path(name)).expect("fixture should parse");
    let valid =
        validate_core_program(RawCoreProgram::new(expr)).expect("positive fixture should validate");
    let lowered = type_check_and_lower_core_program(valid, &env_with_choice(), lowering_context())
        .expect("positive fixture should type-check and lower");

    assert_eq!(lowered.typed().ty(), &int_ty());
}

#[test]
fn choice_all_outcomes_fixture_typechecks_and_lowers() {
    assert_positive_fixture_lowers("motivational_choice_all_outcomes.core");
}

#[test]
fn backtracking_find_first_fixture_typechecks_and_lowers() {
    assert_positive_fixture_lowers("motivational_backtracking_find_first.core");
}

#[test]
fn nested_choice_fixture_typechecks_and_lowers() {
    assert_positive_fixture_lowers("motivational_nested_choice.core");
}

#[test]
fn discard_resume_fixture_typechecks_and_lowers() {
    assert_positive_fixture_lowers("motivational_discard_resume.core");
}

#[test]
fn affine_choice_all_outcomes_shape_rejects_repeated_resume() {
    let expr = parse_core_file(fixture_path(
        "motivational_affine_choice_all_outcomes_invalid.core",
    ))
    .expect("fixture should parse");
    let err = validate_core_program(RawCoreProgram::new(expr))
        .expect_err("affine all-outcomes shape should reject repeated resume");

    assert!(
        matches!(err, CoreValidationError::AffineResumeViolation { .. })
            && err.to_string().contains("jumped to more than once"),
        "unexpected error: {err}"
    );
}

#[test]
fn effectful_multishot_resume_rejects_non_empty_row() {
    let expr = parse_core_file(fixture_path(
        "motivational_effectful_multishot_invalid.core",
    ))
    .expect("fixture should parse");
    let err = validate_core_program(RawCoreProgram::new(expr))
        .expect_err("multi-shot-pure resume with non-empty row should reject");

    assert!(
        err.to_string().contains("multi-shot-pure") && err.to_string().contains("closed empty row"),
        "unexpected error: {err}"
    );
}
