//! TASK-1671: Add Core mode end-to-end fixtures.

use std::path::{Path, PathBuf};

use ash_core::core_ash::{CoreAtom, CoreEvalMode, CoreExpr, CoreRow, CoreType};
use ash_core::core_ash_lower::CoreLoweringContext;
use ash_core::core_ash_text::parse_core_file;
use ash_core::core_ash_typecheck::{
    CoreTypeCheckEnv, TypedCoreProgram, type_check_and_lower_core_program,
};
use ash_core::core_ash_validate::CoreValidationError;
use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};
use ash_core::cps::{ContRef, PrimOp, Term};

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

fn parse_validate_lower_fixture(name: &str) -> (TypedCoreProgram, Term) {
    let expr = parse_core_file(fixture_path(name)).expect("fixture should parse");
    let valid = validate_core_program(RawCoreProgram::new(expr)).expect("fixture should validate");

    let context = CoreLoweringContext::new(ContRef::Label("halt".to_string()), CoreRow::default());
    let typed = type_check_and_lower_core_program(valid, &CoreTypeCheckEnv::default(), context)
        .expect("fixture should type-check and lower");
    typed.into_parts()
}

fn count_force_thunk(term: &Term) -> usize {
    match term {
        Term::LetVal { value: _, body, .. } => count_force_thunk(body),
        Term::LetRec { body, .. } => count_force_thunk(body),
        Term::RecordDischarge { body, .. } => count_force_thunk(body),
        Term::LetPrim {
            op: PrimOp::ForceThunk,
            body,
            ..
        } => 1 + count_force_thunk(body),
        Term::LetPrim { body, .. } => count_force_thunk(body),
        Term::LetCont {
            cont_body, body, ..
        } => count_force_thunk(cont_body).saturating_add(count_force_thunk(body)),
        Term::If {
            then_branch,
            else_branch,
            ..
        } => count_force_thunk(then_branch) + count_force_thunk(else_branch),
        Term::Handle { body, .. } => count_force_thunk(body),
        _ => 0,
    }
}

#[test]
fn lazy_fixture_forces_are_explicitly_lowered_twice() {
    let (typed, lowered) = parse_validate_lower_fixture("lazy_reruns.core");

    assert_eq!(typed.ty(), &CoreType::Base("Int".to_string()));
    assert_eq!(typed.row(), &CoreRow::default());

    assert_eq!(
        count_force_thunk(&lowered),
        2,
        "lazy body should be forced twice when used twice"
    );
}

#[test]
fn memo_fixture_forces_are_explicitly_lowered_twice() {
    let (typed, lowered) = parse_validate_lower_fixture("memo_runs_once.core");

    assert_eq!(typed.ty(), &CoreType::Base("Int".to_string()));
    assert_eq!(typed.row(), &CoreRow::default());

    assert_eq!(
        count_force_thunk(&lowered),
        2,
        "memo body should still require two force sites in syntax"
    );
}

#[test]
fn mode_mismatch_fixture_is_rejected_by_validation() {
    let expr = parse_core_file(fixture_path("mode_invalid_type_mismatch.core")).unwrap();
    let error = validate_core_program(RawCoreProgram::new(expr))
        .expect_err("mode mismatch fixture should fail validation");

    assert!(
        matches!(error, CoreValidationError::LetModeTypeMismatch { .. }),
        "expected let-mode type-mismatch validation error, got {error}"
    );
}

#[test]
fn let_mode_mode_carries_match_mode_wrapper_shape() {
    let expr = CoreExpr::LetMode {
        name: "memoized".to_string(),
        mode: CoreEvalMode::Memo,
        ty: CoreType::Mode {
            mode: CoreEvalMode::Memo,
            inner: Box::new(CoreType::Base("Int".to_string())),
            latent_row: Some(CoreRow::default()),
        },
        expr: Box::new(CoreExpr::Atom(CoreAtom::LitInt(3))),
        body: Box::new(CoreExpr::Force {
            name: "forced".to_string(),
            thunk: CoreAtom::Var("memoized".to_string()),
            body: Box::new(CoreExpr::Atom(CoreAtom::Var("forced".to_string()))),
        }),
    };

    let valid = validate_core_program(RawCoreProgram::new(expr)).expect("expr should validate");
    let env = CoreTypeCheckEnv::default();
    let context = CoreLoweringContext::new(ContRef::Label("halt".to_string()), CoreRow::default());
    let typed = type_check_and_lower_core_program(valid, &env, context)
        .expect("memo let-mode should lower")
        .into_parts()
        .0;

    assert_eq!(typed.ty(), &CoreType::Base("Int".to_string()));
    assert_eq!(typed.row(), &CoreRow::default());
}
