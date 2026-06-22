//! TASK-1685: Type-Check Multi-Shot Handler Resumes
//!
//! Tests that handler clauses with legal `MultiShotPure` resume continuations
//! are accepted, and that illegal ones (non-empty row, input mismatch) are rejected.

use ash_core::core_ash::{
    CoreAtom, CoreContRef, CoreEffectOp, CoreExpr, CoreHandlerClause, CoreMultiplicity, CoreParam,
    CoreRow, CoreRowItem, CoreType,
};
use ash_core::core_ash_typecheck::{CoreTypeCheckEnv, CoreTypeCheckError, type_check_core_program};
use ash_core::core_ash_validate::{CoreValidationError, RawCoreProgram, validate_core_program};

fn string_ty() -> CoreType {
    CoreType::Base("String".into())
}

fn cap_item(path: &[&str], operation: &str) -> CoreRowItem {
    CoreRowItem::Capability {
        path: path.iter().map(|p| (*p).to_owned()).collect(),
        operation: operation.to_owned(),
    }
}

fn row(items: Vec<CoreRowItem>) -> CoreRow {
    CoreRow::closed(items)
}

fn kv_read_op() -> CoreEffectOp {
    CoreEffectOp::Capability {
        path: vec!["kv".into()],
        operation: "read".into(),
        arg_types: vec![string_ty()],
        result_type: string_ty(),
    }
}

fn cont_ty(
    input: CoreType,
    answer: CoreType,
    r: CoreRow,
    multiplicity: CoreMultiplicity,
) -> CoreType {
    CoreType::Cont {
        input: Box::new(input),
        answer: Box::new(answer),
        row: r,
        multiplicity,
    }
}

fn param(name: &str, ty: CoreType) -> CoreParam {
    CoreParam {
        name: name.to_owned(),
        ty,
    }
}

fn resume_param(
    input: CoreType,
    answer: CoreType,
    r: CoreRow,
    multiplicity: CoreMultiplicity,
) -> CoreParam {
    param("resume", cont_ty(input, answer, r, multiplicity))
}

fn handler_clause(
    params: Vec<CoreParam>,
    resume: CoreParam,
    clause_body: CoreExpr,
    clause_row: CoreRow,
) -> CoreHandlerClause {
    CoreHandlerClause {
        op: kv_read_op(),
        params,
        resume,
        body: Box::new(clause_body),
        row: clause_row,
    }
}

fn resume_with(value: CoreAtom) -> CoreExpr {
    CoreExpr::Jump {
        cont: CoreContRef::Var("resume".into()),
        arg: value,
    }
}

fn raise_read() -> CoreExpr {
    CoreExpr::Raise {
        op: kv_read_op(),
        args: vec![CoreAtom::LitString("user:7".into())],
    }
}

fn handle_with(clause: CoreHandlerClause, body: CoreExpr) -> CoreExpr {
    CoreExpr::Handle {
        clause,
        body: Box::new(body),
    }
}

fn base_env() -> CoreTypeCheckEnv {
    let mut env = CoreTypeCheckEnv::default();
    env.operations_mut().insert(kv_read_op());
    env
}

fn type_check(
    expr: CoreExpr,
    env: &CoreTypeCheckEnv,
) -> Result<ash_core::core_ash_typecheck::TypedCoreProgram, CoreTypeCheckError> {
    let valid =
        validate_core_program(RawCoreProgram::new(expr)).expect("Core expression validates");
    type_check_core_program(valid, env)
}

// ---------------------------------------------------------------------------
// Legal multi-shot resume acceptance
// ---------------------------------------------------------------------------

#[test]
fn legal_multishot_resume_empty_row_typechecks() {
    let clause = handler_clause(
        vec![param("key", string_ty())],
        resume_param(
            string_ty(),
            string_ty(),
            CoreRow::default(),
            CoreMultiplicity::MultiShotPure,
        ),
        resume_with(CoreAtom::Var("key".into())),
        CoreRow::default(),
    );

    let typed = type_check(handle_with(clause, raise_read()), &base_env())
        .expect("legal multi-shot resume should type-check");
    assert_eq!(typed.ty(), &string_ty());
    assert_eq!(typed.row(), &CoreRow::default());
}

// ---------------------------------------------------------------------------
// Illegal multi-shot resume rejection
// ---------------------------------------------------------------------------

#[test]
fn multishot_resume_with_nonempty_row_rejected_at_validation() {
    let clause = handler_clause(
        vec![param("key", string_ty())],
        resume_param(
            string_ty(),
            string_ty(),
            row(vec![cap_item(&["kv"], "read")]),
            CoreMultiplicity::MultiShotPure,
        ),
        resume_with(CoreAtom::Var("key".into())),
        CoreRow::default(),
    );

    let err = validate_core_program(RawCoreProgram::new(handle_with(clause, raise_read())))
        .expect_err("multi-shot resume with non-empty row should fail validation");
    assert!(
        matches!(err, CoreValidationError::AffineResumeViolation { .. }),
        "should be AffineResumeViolation: {err:?}"
    );
    assert!(
        err.to_string()
            .contains("multi-shot-pure resume must declare a closed empty row"),
        "error should mention row legality: {err}"
    );
}

#[test]
fn multishot_resume_with_input_type_mismatch_rejected() {
    // Resume expects Int input, but the operation returns String.
    let clause = handler_clause(
        vec![param("key", string_ty())],
        resume_param(
            CoreType::Base("Int".into()),
            string_ty(),
            CoreRow::default(),
            CoreMultiplicity::MultiShotPure,
        ),
        resume_with(CoreAtom::Var("key".into())),
        CoreRow::default(),
    );

    let err = type_check(handle_with(clause, raise_read()), &base_env())
        .expect_err("input type mismatch should be rejected");
    assert!(
        matches!(err, CoreTypeCheckError::TypeMismatch { .. }),
        "expected type mismatch: {err:?}"
    );
}

// ---------------------------------------------------------------------------
// Multi-shot resume with repeated jumps (no affine violation)
// ---------------------------------------------------------------------------

#[test]
fn multishot_resume_allows_repeated_jumps_in_validation() {
    // A handler body that jumps to resume twice (in if/else) is legal for
    // multi-shot-pure resumes — validation must not reject it.
    let clause = handler_clause(
        vec![param("key", string_ty())],
        resume_param(
            string_ty(),
            string_ty(),
            CoreRow::default(),
            CoreMultiplicity::MultiShotPure,
        ),
        CoreExpr::If {
            cond: CoreAtom::LitBool(true),
            then_branch: Box::new(resume_with(CoreAtom::LitString("a".into()))),
            else_branch: Box::new(resume_with(CoreAtom::Var("key".into()))),
        },
        CoreRow::default(),
    );

    let result = validate_core_program(RawCoreProgram::new(handle_with(clause, raise_read())));
    assert!(
        result.is_ok(),
        "repeated jumps to multishot resume should pass validation: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// Affine resume with repeated jumps still rejected
// ---------------------------------------------------------------------------

#[test]
fn affine_resume_repeated_jumps_still_rejected() {
    let clause = handler_clause(
        vec![param("key", string_ty())],
        resume_param(
            string_ty(),
            string_ty(),
            CoreRow::default(),
            CoreMultiplicity::Affine,
        ),
        CoreExpr::If {
            cond: CoreAtom::LitBool(true),
            then_branch: Box::new(resume_with(CoreAtom::LitString("a".into()))),
            else_branch: Box::new(resume_with(CoreAtom::Var("key".into()))),
        },
        CoreRow::default(),
    );

    let err = validate_core_program(RawCoreProgram::new(handle_with(clause, raise_read())))
        .expect_err("repeated jumps to affine resume should fail");
    assert!(
        matches!(err, CoreValidationError::AffineResumeViolation { .. }),
        "should be AffineResumeViolation: {err:?}"
    );
    assert!(
        err.to_string().contains("jumped to more than once"),
        "error should mention repeated jumps: {err}"
    );
}

// ---------------------------------------------------------------------------
// Affine resume behavior unchanged
// ---------------------------------------------------------------------------

#[test]
fn affine_resume_single_jump_typechecks() {
    let clause = handler_clause(
        vec![param("key", string_ty())],
        resume_param(
            string_ty(),
            string_ty(),
            CoreRow::default(),
            CoreMultiplicity::Affine,
        ),
        resume_with(CoreAtom::Var("key".into())),
        CoreRow::default(),
    );

    let typed = type_check(handle_with(clause, raise_read()), &base_env())
        .expect("affine resume with single jump should type-check");
    assert_eq!(typed.ty(), &string_ty());
    assert_eq!(typed.row(), &CoreRow::default());
}
