use ash_core::core_ash::{
    CoreAtom, CoreExpr, CorePrimOp, CoreRow, CoreTrapReason, CoreType, CoreValue,
};
use ash_core::core_ash_typecheck::{CoreTypeCheckEnv, CoreTypeCheckError, type_check_core_program};
use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};

fn int_ty() -> CoreType {
    CoreType::Base("Int".into())
}

fn bool_ty() -> CoreType {
    CoreType::Base("Bool".into())
}

fn positive_bool_ty() -> CoreType {
    CoreType::Refinement {
        base: Box::new(bool_ty()),
        predicate: "is-ready".into(),
    }
}

fn type_check_with_env(
    expr: CoreExpr,
    env: &CoreTypeCheckEnv,
) -> Result<(CoreType, CoreRow), CoreTypeCheckError> {
    let valid =
        validate_core_program(RawCoreProgram::new(expr)).expect("Core expression validates");
    let typed = type_check_core_program(valid, env)?;

    Ok((typed.ty().clone(), typed.row().clone()))
}

fn type_check(expr: CoreExpr) -> Result<(CoreType, CoreRow), CoreTypeCheckError> {
    type_check_with_env(expr, &CoreTypeCheckEnv::default())
}

#[test]
fn let_bound_literal_typechecks_and_extends_body_environment() {
    let expr = CoreExpr::LetVal {
        name: "x".into(),
        ty: int_ty(),
        value: CoreValue::Atom(CoreAtom::LitInt(42)),
        body: Box::new(CoreExpr::Atom(CoreAtom::Var("x".into()))),
    };

    let (ty, row) = type_check(expr).expect("let-bound int literal type-checks");

    assert_eq!(ty, int_ty());
    assert_eq!(row, CoreRow::default());
}

#[test]
fn let_val_declared_type_mismatch_fails() {
    let expr = CoreExpr::LetVal {
        name: "x".into(),
        ty: bool_ty(),
        value: CoreValue::Atom(CoreAtom::LitInt(42)),
        body: Box::new(CoreExpr::Atom(CoreAtom::Var("x".into()))),
    };

    type_check(expr).expect_err("declared Bool binding cannot accept an Int literal");
}

#[test]
fn pure_add_letprim_binds_int_result_with_empty_row() {
    let expr = CoreExpr::LetPrim {
        name: "sum".into(),
        op: CorePrimOp::Add,
        args: vec![CoreAtom::LitInt(40), CoreAtom::LitInt(2)],
        body: Box::new(CoreExpr::Atom(CoreAtom::Var("sum".into()))),
    };

    let (ty, row) = type_check(expr).expect("pure Add over Int arguments type-checks");

    assert_eq!(ty, int_ty());
    assert_eq!(row, CoreRow::default());
}

#[test]
fn pure_add_letprim_rejects_non_int_argument() {
    let expr = CoreExpr::LetPrim {
        name: "sum".into(),
        op: CorePrimOp::Add,
        args: vec![CoreAtom::LitInt(40), CoreAtom::LitBool(true)],
        body: Box::new(CoreExpr::Atom(CoreAtom::Var("sum".into()))),
    };

    type_check(expr).expect_err("pure Add requires Int arguments");
}

#[test]
fn if_rejects_non_bool_condition() {
    let expr = CoreExpr::If {
        cond: CoreAtom::LitInt(1),
        then_branch: Box::new(CoreExpr::Atom(CoreAtom::LitInt(1))),
        else_branch: Box::new(CoreExpr::Atom(CoreAtom::LitInt(0))),
    };

    type_check(expr).expect_err("If condition must be Bool");
}

#[test]
fn if_accepts_refined_bool_condition() {
    let mut env = CoreTypeCheckEnv::default();
    env.discharges_mut().insert_refinement_predicate("is-ready");
    env.values_mut().insert("ready", positive_bool_ty());
    let expr = CoreExpr::If {
        cond: CoreAtom::Var("ready".into()),
        then_branch: Box::new(CoreExpr::Atom(CoreAtom::LitInt(1))),
        else_branch: Box::new(CoreExpr::Atom(CoreAtom::LitInt(0))),
    };

    let (ty, row) = type_check_with_env(expr, &env)
        .expect("refinement-typed Bool values should be usable as If conditions");

    assert_eq!(ty, int_ty());
    assert_eq!(row, CoreRow::default());
}

#[test]
fn trap_branch_checks_against_expected_int_result_with_empty_row() {
    let expr = CoreExpr::If {
        cond: CoreAtom::LitBool(true),
        then_branch: Box::new(CoreExpr::Atom(CoreAtom::LitInt(1))),
        else_branch: Box::new(CoreExpr::Trap {
            reason: CoreTrapReason::Panic("unreachable branch".into()),
        }),
    };

    let (ty, row) = type_check(expr).expect("Trap branch checks at expected Int result type");

    assert_eq!(ty, int_ty());
    assert_eq!(row, CoreRow::default());
}
