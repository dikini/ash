use ash_core::core_ash::{
    CoreAtom, CoreContRef, CoreExpr, CoreMultiplicity, CoreRow, CoreRowItem, CoreType,
};
use ash_core::core_ash_typecheck::{CoreTypeCheckEnv, CoreTypeCheckError, type_check_core_program};
use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};

fn int_ty() -> CoreType {
    CoreType::Base("Int".into())
}

fn string_ty() -> CoreType {
    CoreType::Base("String".into())
}

fn unit_ty() -> CoreType {
    CoreType::Base("Unit".into())
}

fn positive_int_ty() -> CoreType {
    CoreType::Refinement {
        base: Box::new(int_ty()),
        predicate: "result > 0".into(),
    }
}

fn cap(path: &[&str], operation: &str) -> CoreRowItem {
    CoreRowItem::Capability {
        path: path.iter().map(|part| (*part).to_owned()).collect(),
        operation: operation.to_owned(),
    }
}

fn cap_row(path: &[&str], operation: &str) -> CoreRow {
    CoreRow::closed(vec![cap(path, operation)])
}

fn function_ty(params: Vec<CoreType>, result: CoreType, row: CoreRow) -> CoreType {
    CoreType::Function {
        params,
        result: Box::new(result),
        row,
    }
}

fn cont_ty(input: CoreType, answer: CoreType, row: CoreRow) -> CoreType {
    CoreType::Cont {
        input: Box::new(input),
        answer: Box::new(answer),
        row,
        multiplicity: CoreMultiplicity::Affine,
    }
}

fn type_check(
    expr: CoreExpr,
    env: &CoreTypeCheckEnv,
) -> Result<ash_core::core_ash_typecheck::TypedCoreProgram, CoreTypeCheckError> {
    let valid =
        validate_core_program(RawCoreProgram::new(expr)).expect("Core expression validates");
    type_check_core_program(valid, env)
}

#[test]
fn let_call_binds_result_in_body_and_charges_function_latent_row_plus_body_local_row() {
    let function_row = cap_row(&["db"], "read");
    let body_row = cap_row(&["console"], "write");
    let expected_row = CoreRow::closed(vec![cap(&["db"], "read"), cap(&["console"], "write")]);
    let mut env = CoreTypeCheckEnv::default();
    env.values_mut().insert(
        "read_user",
        function_ty(vec![int_ty()], string_ty(), function_row),
    );
    env.values_mut().insert(
        "emit_user",
        function_ty(vec![string_ty()], unit_ty(), body_row),
    );

    let typed = type_check(
        CoreExpr::LetCall {
            name: "user".into(),
            func: CoreAtom::Var("read_user".into()),
            args: vec![CoreAtom::LitInt(7)],
            body: Box::new(CoreExpr::Call {
                func: CoreAtom::Var("emit_user".into()),
                args: vec![CoreAtom::Var("user".into())],
            }),
        },
        &env,
    )
    .expect("LetCall result binding and row accounting should type-check");

    assert_eq!(typed.ty(), &unit_ty());
    assert_eq!(typed.row(), &expected_row);
}

#[test]
fn let_call_function_arity_mismatch_reports_argument_count_mismatch() {
    let mut env = CoreTypeCheckEnv::default();
    env.values_mut().insert(
        "pair",
        function_ty(vec![int_ty(), string_ty()], unit_ty(), CoreRow::default()),
    );

    let err = type_check(
        CoreExpr::LetCall {
            name: "result".into(),
            func: CoreAtom::Var("pair".into()),
            args: vec![CoreAtom::LitInt(7)],
            body: Box::new(CoreExpr::Atom(CoreAtom::Var("result".into()))),
        },
        &env,
    )
    .expect_err("wrong function arity must be rejected");

    assert_eq!(
        err,
        CoreTypeCheckError::ArgumentCountMismatch {
            expected: 2,
            actual: 1
        }
    );
}

#[test]
fn function_call_accepts_refinement_argument_where_base_is_expected() {
    let mut env = CoreTypeCheckEnv::default();
    env.discharges_mut()
        .insert_refinement_predicate("result > 0");
    env.values_mut().insert("positive", positive_int_ty());
    env.values_mut().insert(
        "consume_int",
        function_ty(vec![int_ty()], unit_ty(), CoreRow::default()),
    );

    let typed = type_check(
        CoreExpr::Call {
            func: CoreAtom::Var("consume_int".into()),
            args: vec![CoreAtom::Var("positive".into())],
        },
        &env,
    )
    .expect("refinement-typed values should be usable where their base type is expected");

    assert_eq!(typed.ty(), &unit_ty());
    assert_eq!(typed.row(), &CoreRow::default());
}

#[test]
fn function_call_refinement_argument_obligation_records_argument_name() {
    let mut env = CoreTypeCheckEnv::default();
    env.discharges_mut()
        .insert_refinement_predicate("result > 0");
    env.values_mut().insert("x", int_ty());
    env.values_mut().insert(
        "consume_positive",
        function_ty(vec![positive_int_ty()], unit_ty(), CoreRow::default()),
    );

    let typed = type_check(
        CoreExpr::Call {
            func: CoreAtom::Var("consume_positive".into()),
            args: vec![CoreAtom::Var("x".into())],
        },
        &env,
    )
    .expect("plain variable arguments should emit refinement obligations");

    let obligations = typed.obligations();
    assert_eq!(obligations.len(), 1);
    assert_eq!(obligations[0].value_name(), Some("x"));
    assert_eq!(obligations[0].predicate(), "result > 0");
}

#[test]
fn function_call_refinement_argument_obligation_for_literal_stays_anonymous() {
    let mut env = CoreTypeCheckEnv::default();
    env.discharges_mut()
        .insert_refinement_predicate("result > 0");
    env.values_mut().insert(
        "consume_positive",
        function_ty(vec![positive_int_ty()], unit_ty(), CoreRow::default()),
    );

    let typed = type_check(
        CoreExpr::Call {
            func: CoreAtom::Var("consume_positive".into()),
            args: vec![CoreAtom::LitInt(7)],
        },
        &env,
    )
    .expect("literal arguments should still emit anonymous refinement obligations");

    let obligations = typed.obligations();
    assert_eq!(obligations.len(), 1);
    assert_eq!(obligations[0].value_name(), None);
}

#[test]
fn tail_call_reports_callee_local_row_not_continuation_row() {
    let function_row = cap_row(&["db"], "read");
    let continuation_row = cap_row(&["console"], "write");
    let mut env = CoreTypeCheckEnv::default();
    env.values_mut().insert(
        "read_user",
        function_ty(vec![int_ty()], string_ty(), function_row.clone()),
    );
    env.continuations_mut()
        .insert("exit", cont_ty(string_ty(), unit_ty(), continuation_row));

    let typed = type_check(
        CoreExpr::Call {
            func: CoreAtom::Var("read_user".into()),
            args: vec![CoreAtom::LitInt(7)],
        },
        &env,
    )
    .expect("tail call should type-check");

    assert_eq!(typed.ty(), &string_ty());
    assert_eq!(typed.row(), &function_row);
}

#[test]
fn jump_checks_argument_type_and_rejects_mismatch() {
    let mut env = CoreTypeCheckEnv::default();
    env.continuations_mut()
        .insert("exit", cont_ty(int_ty(), unit_ty(), CoreRow::default()));

    let err = type_check(
        CoreExpr::Jump {
            cont: CoreContRef::Label("exit".into()),
            arg: CoreAtom::LitString("not an int".into()),
        },
        &env,
    )
    .expect_err("jump argument must match continuation input type");

    assert!(matches!(err, CoreTypeCheckError::TypeMismatch { .. }));
}

#[test]
fn jump_has_empty_local_row_and_exposes_target_continuation_row_for_lowering() {
    let continuation_row = cap_row(&["console"], "write");
    let target = CoreContRef::Label("exit".into());
    let mut env = CoreTypeCheckEnv::default();
    env.continuations_mut().insert(
        "exit",
        cont_ty(int_ty(), unit_ty(), continuation_row.clone()),
    );

    let typed = type_check(
        CoreExpr::Jump {
            cont: target.clone(),
            arg: CoreAtom::LitInt(7),
        },
        &env,
    )
    .expect("jump to known continuation should type-check");

    assert_eq!(typed.ty(), &unit_ty());
    assert_eq!(typed.row(), &CoreRow::default());
    assert_eq!(
        typed.facts().jump_continuation_rows().get(&target),
        Some(&continuation_row)
    );
}
