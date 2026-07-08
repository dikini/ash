//! TASK-1686: Core LetContCall and multiplicity-aware resume use discipline.

use ash_core::core_ash::{
    CoreAtom, CoreContRef, CoreEffectOp, CoreExpr, CoreHandlerClause, CoreMultiplicity, CoreParam,
    CoreRow, CoreRowItem, CoreType,
};
use ash_core::core_ash_text::{core_expr_to_string, parse_core_expr};
use ash_core::core_ash_typecheck::{CoreTypeCheckEnv, type_check_core_program};
use ash_core::core_ash_validate::{CoreValidationError, RawCoreProgram, validate_core_program};

fn int_ty() -> CoreType {
    CoreType::Base("Int".to_string())
}

fn string_ty() -> CoreType {
    CoreType::Base("String".to_string())
}

fn cont_ty(
    input: CoreType,
    answer: CoreType,
    row: CoreRow,
    multiplicity: CoreMultiplicity,
) -> CoreType {
    CoreType::Cont {
        input: Box::new(input),
        answer: Box::new(answer),
        row,
        multiplicity,
    }
}

fn operation_item(path: &[&str], operation: &str) -> CoreRowItem {
    CoreRowItem::Operation {
        path: path.iter().map(|segment| (*segment).to_string()).collect(),
        operation: operation.to_string(),
    }
}

fn cap_row() -> CoreRow {
    CoreRow::closed(vec![operation_item(&["kv"], "read")])
}

fn read_op() -> CoreEffectOp {
    CoreEffectOp::Operation {
        path: vec!["kv".to_string()],
        operation: "read".to_string(),
        arg_types: vec![string_ty()],
        result_type: string_ty(),
    }
}

fn param(name: &str, ty: CoreType) -> CoreParam {
    CoreParam {
        name: name.to_string(),
        ty,
    }
}

fn resume_param(multiplicity: CoreMultiplicity) -> CoreParam {
    param(
        "resume",
        cont_ty(string_ty(), string_ty(), CoreRow::default(), multiplicity),
    )
}

fn let_cont_call(name: &str, arg: CoreAtom, body: CoreExpr) -> CoreExpr {
    CoreExpr::LetContCall {
        name: name.to_string(),
        cont: CoreContRef::Var("resume".to_string()),
        arg,
        body: Box::new(body),
    }
}

fn handler_with_resume_body(multiplicity: CoreMultiplicity, body: CoreExpr) -> CoreExpr {
    CoreExpr::Handle {
        clause: CoreHandlerClause {
            op: read_op(),
            params: vec![param("key", string_ty())],
            resume: resume_param(multiplicity),
            body: Box::new(body),
            row: CoreRow::default(),
        },
        body: Box::new(CoreExpr::Raise {
            op: read_op(),
            args: vec![CoreAtom::LitString("user:7".to_string())],
        }),
    }
}

fn env_with_continuation(row: CoreRow, multiplicity: CoreMultiplicity) -> CoreTypeCheckEnv {
    let mut env = CoreTypeCheckEnv::default();
    env.continuations_mut().insert(
        "k".to_string(),
        cont_ty(int_ty(), string_ty(), row, multiplicity),
    );
    env
}

fn env_with_operation() -> CoreTypeCheckEnv {
    let mut env = CoreTypeCheckEnv::default();
    env.operations_mut().insert(read_op());
    env
}

#[test]
fn parses_and_serializes_let_cont_call() {
    let parsed = parse_core_expr("(let-cont-call answer (label k) (lit-int 7) answer)").unwrap();

    assert_eq!(
        parsed,
        CoreExpr::LetContCall {
            name: "answer".to_string(),
            cont: CoreContRef::Label("k".to_string()),
            arg: CoreAtom::LitInt(7),
            body: Box::new(CoreExpr::Atom(CoreAtom::Var("answer".to_string()))),
        }
    );
    assert_eq!(
        core_expr_to_string(&parsed),
        "(let-cont-call answer (label k) (lit-int 7) answer)"
    );
}

#[test]
fn let_cont_call_typechecks_answer_binding_and_contributes_invocation_row() {
    let expr = CoreExpr::LetContCall {
        name: "answer".to_string(),
        cont: CoreContRef::Label("k".to_string()),
        arg: CoreAtom::LitInt(5),
        body: Box::new(CoreExpr::Atom(CoreAtom::Var("answer".to_string()))),
    };
    let valid = validate_core_program(RawCoreProgram::new(expr)).unwrap();
    let typed = type_check_core_program(
        valid,
        &env_with_continuation(cap_row(), CoreMultiplicity::Affine),
    )
    .expect("let-cont-call should type-check against continuation target");

    assert_eq!(typed.ty(), &string_ty());
    assert_eq!(typed.row(), &cap_row());
}

#[test]
fn affine_resume_repeated_let_cont_call_is_rejected() {
    let body = let_cont_call(
        "first",
        CoreAtom::LitString("a".to_string()),
        let_cont_call(
            "second",
            CoreAtom::LitString("b".to_string()),
            CoreExpr::Atom(CoreAtom::Var("second".to_string())),
        ),
    );
    let err = validate_core_program(RawCoreProgram::new(handler_with_resume_body(
        CoreMultiplicity::Affine,
        body,
    )))
    .expect_err("affine resume must reject repeated answer-binding calls");

    assert!(
        matches!(err, CoreValidationError::AffineResumeViolation { .. }),
        "expected affine resume violation, got {err:?}"
    );
}

#[test]
fn multishot_resume_repeated_let_cont_call_is_accepted() {
    let body = let_cont_call(
        "first",
        CoreAtom::LitString("a".to_string()),
        let_cont_call(
            "second",
            CoreAtom::LitString("b".to_string()),
            CoreExpr::Atom(CoreAtom::Var("second".to_string())),
        ),
    );
    let valid = validate_core_program(RawCoreProgram::new(handler_with_resume_body(
        CoreMultiplicity::MultiShotPure,
        body,
    )))
    .expect("multi-shot-pure resume may be invoked repeatedly");
    let typed = type_check_core_program(valid, &env_with_operation())
        .expect("multi-shot-pure repeated let-cont-call should type-check");

    assert_eq!(typed.ty(), &string_ty());
    assert_eq!(typed.row(), &CoreRow::default());
}

#[test]
fn discarded_affine_and_multishot_resumes_are_accepted() {
    for multiplicity in [CoreMultiplicity::Affine, CoreMultiplicity::MultiShotPure] {
        let expr = handler_with_resume_body(
            multiplicity,
            CoreExpr::Atom(CoreAtom::LitString("fallback".to_string())),
        );
        let valid = validate_core_program(RawCoreProgram::new(expr))
            .expect("discarded resume should validate");
        let typed = type_check_core_program(valid, &env_with_operation())
            .expect("discarded resume should type-check");

        assert_eq!(typed.ty(), &string_ty());
    }
}

#[test]
fn affine_branch_local_let_cont_call_uses_are_rejected_conservatively() {
    let expr = handler_with_resume_body(
        CoreMultiplicity::Affine,
        CoreExpr::If {
            cond: CoreAtom::LitBool(true),
            then_branch: Box::new(let_cont_call(
                "left",
                CoreAtom::LitString("left".to_string()),
                CoreExpr::Atom(CoreAtom::Var("left".to_string())),
            )),
            else_branch: Box::new(let_cont_call(
                "right",
                CoreAtom::LitString("right".to_string()),
                CoreExpr::Atom(CoreAtom::Var("right".to_string())),
            )),
        },
    );

    let err = validate_core_program(RawCoreProgram::new(expr))
        .expect_err("current affine use discipline rejects branch-merged duplicate uses");
    assert!(
        matches!(err, CoreValidationError::AffineResumeViolation { .. }),
        "expected affine resume violation, got {err:?}"
    );
}

#[test]
fn use_discipline_depends_on_resume_type_not_name() {
    let expr = CoreExpr::Handle {
        clause: CoreHandlerClause {
            op: read_op(),
            params: vec![param("key", string_ty())],
            resume: CoreParam {
                name: "again".to_string(),
                ty: cont_ty(
                    string_ty(),
                    string_ty(),
                    CoreRow::default(),
                    CoreMultiplicity::MultiShotPure,
                ),
            },
            body: Box::new(CoreExpr::LetContCall {
                name: "first".to_string(),
                cont: CoreContRef::Var("again".to_string()),
                arg: CoreAtom::LitString("a".to_string()),
                body: Box::new(CoreExpr::LetContCall {
                    name: "second".to_string(),
                    cont: CoreContRef::Var("again".to_string()),
                    arg: CoreAtom::LitString("b".to_string()),
                    body: Box::new(CoreExpr::Atom(CoreAtom::Var("second".to_string()))),
                }),
            }),
            row: CoreRow::default(),
        },
        body: Box::new(CoreExpr::Raise {
            op: read_op(),
            args: vec![CoreAtom::LitString("user:7".to_string())],
        }),
    };

    let valid = validate_core_program(RawCoreProgram::new(expr))
        .expect("multi-shot use should not depend on the variable being named resume");
    type_check_core_program(valid, &env_with_operation())
        .expect("non-conventional multi-shot resume name should type-check");
}
