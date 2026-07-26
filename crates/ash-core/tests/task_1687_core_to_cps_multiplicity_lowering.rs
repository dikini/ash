//! TASK-1687: Core-to-CPS continuation multiplicity lowering.

use ash_core::core_ash::{
    CoreAtom, CoreContRef, CoreEffectOp, CoreExpr, CoreHandlerClause, CoreMultiplicity, CoreParam,
    CorePrimOp, CoreRow, CoreRowItem, CoreType, CoreValue,
};
use ash_core::core_ash_lower::CoreLoweringContext;
use ash_core::core_ash_typecheck::{CoreTypeCheckEnv, type_check_and_lower_core_program};
use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};
use ash_core::cps::{
    Atom, ContMultiplicity, ContRef, EffectItem, EffectItemKind, EffectRow, HandlerClause,
    ResumeRowMetadata, Term,
};

fn int_ty() -> CoreType {
    CoreType::Base("Int".to_string())
}

fn string_ty() -> CoreType {
    CoreType::Base("String".to_string())
}

fn unit_ty() -> CoreType {
    CoreType::Base("Unit".to_string())
}

fn function_ty(params: Vec<CoreType>, result: CoreType, row: CoreRow) -> CoreType {
    CoreType::Function {
        params,
        result: Box::new(result),
        row,
    }
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

fn param(name: &str, ty: CoreType) -> CoreParam {
    CoreParam {
        name: name.to_string(),
        ty,
    }
}

fn operation_item(path: &[&str], operation: &str) -> CoreRowItem {
    CoreRowItem::Operation {
        path: path.iter().map(|segment| (*segment).to_string()).collect(),
        operation: operation.to_string(),
    }
}

fn cap_row(path: &[&str], operation: &str) -> CoreRow {
    CoreRow::closed(vec![operation_item(path, operation)])
}

fn cps_cap_row(path: &[&str], operation: &str) -> EffectRow {
    EffectRow {
        items: vec![EffectItem {
            namespace: "cap".to_string(),
            name: format!("{}.{}", path.join("."), operation),
            kind: EffectItemKind::Capability,
        }],
    }
}

fn read_op() -> CoreEffectOp {
    CoreEffectOp::Operation {
        path: vec!["kv".to_string()],
        operation: "read".to_string(),
        arg_types: vec![string_ty()],
        result_type: string_ty(),
    }
}

fn audit_op() -> CoreEffectOp {
    CoreEffectOp::Operation {
        path: vec!["audit".to_string()],
        operation: "emit".to_string(),
        arg_types: vec![string_ty()],
        result_type: unit_ty(),
    }
}

fn lowering_context() -> CoreLoweringContext {
    CoreLoweringContext::new(ContRef::Label("halt".to_string()), CoreRow::default())
}

fn env_with_ops() -> CoreTypeCheckEnv {
    let mut env = CoreTypeCheckEnv::default();
    env.operations_mut().insert(read_op());
    env.operations_mut().insert(audit_op());
    env
}

fn checked_lower(expr: CoreExpr, env: &CoreTypeCheckEnv) -> Term {
    let program =
        validate_core_program(RawCoreProgram::new(expr)).expect("Core expression should validate");
    let (_typed, lowered) = type_check_and_lower_core_program(program, env, lowering_context())
        .expect("Core expression should type-check and lower")
        .into_parts();
    lowered
}

fn resume_param(row: CoreRow, multiplicity: CoreMultiplicity) -> CoreParam {
    param(
        "resume",
        cont_ty(string_ty(), string_ty(), row, multiplicity),
    )
}

fn handler_expr(resume: CoreParam, handler_body: CoreExpr) -> CoreExpr {
    CoreExpr::Handle {
        clause: CoreHandlerClause {
            op: read_op(),
            params: vec![param("key", string_ty())],
            resume,
            body: Box::new(handler_body),
            row: CoreRow::default(),
        },
        body: Box::new(CoreExpr::Raise {
            op: read_op(),
            args: vec![CoreAtom::LitString("user:7".to_string())],
        }),
    }
}

fn first_handler_clause(term: &Term) -> Option<&HandlerClause> {
    match term {
        Term::Handle { clause, .. } => Some(clause),
        Term::LetVal { body, .. }
        | Term::LetPrim { body, .. }
        | Term::LetRec { body, .. }
        | Term::RecordDischarge { body, .. }
        | Term::LetContCall { body, .. } => first_handler_clause(body),
        Term::LetCont {
            cont_body, body, ..
        } => first_handler_clause(cont_body).or_else(|| first_handler_clause(body)),
        Term::If {
            then_branch,
            else_branch,
            ..
        } => first_handler_clause(then_branch).or_else(|| first_handler_clause(else_branch)),
        Term::Match { arms, default, .. } => arms
            .iter()
            .find_map(|(_, arm)| first_handler_clause(arm))
            .or_else(|| default.as_deref().and_then(first_handler_clause)),
        Term::Jump { .. }
        | Term::JumpValue { .. }
        | Term::Call { .. }
        | Term::Raise { .. }
        | Term::Return { .. }
        | Term::Trap { .. } => None,
    }
}

fn first_let_cont(term: &Term) -> Option<&Term> {
    match term {
        Term::LetCont { .. } => Some(term),
        Term::LetVal { body, .. }
        | Term::LetPrim { body, .. }
        | Term::LetRec { body, .. }
        | Term::RecordDischarge { body, .. }
        | Term::Handle { body, .. }
        | Term::LetContCall { body, .. } => first_let_cont(body),
        Term::If {
            then_branch,
            else_branch,
            ..
        } => first_let_cont(then_branch).or_else(|| first_let_cont(else_branch)),
        Term::Match { arms, default, .. } => arms
            .iter()
            .find_map(|(_, arm)| first_let_cont(arm))
            .or_else(|| default.as_deref().and_then(first_let_cont)),
        Term::Jump { .. }
        | Term::JumpValue { .. }
        | Term::Call { .. }
        | Term::Raise { .. }
        | Term::Return { .. }
        | Term::Trap { .. } => None,
    }
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
fn checked_handler_resume_lowers_known_row_and_multishot_multiplicity() {
    let resume_row = CoreRow::default();
    let lowered = checked_lower(
        handler_expr(
            resume_param(resume_row.clone(), CoreMultiplicity::MultiShotPure),
            CoreExpr::LetContCall {
                name: "answer".to_string(),
                cont: CoreContRef::Var("resume".to_string()),
                arg: CoreAtom::LitString("ok".to_string()),
                body: Box::new(CoreExpr::Atom(CoreAtom::Var("answer".to_string()))),
            },
        ),
        &env_with_ops(),
    );

    let clause = first_handler_clause(&lowered).expect("lowered term should contain Handle");
    assert_eq!(
        clause.resume_row,
        ResumeRowMetadata::Known(EffectRow::default()),
        "checked lowering must emit known resume-row metadata"
    );
    assert_eq!(
        clause.resume_multiplicity,
        ContMultiplicity::MultiShotPure,
        "checked lowering must preserve multi-shot-pure resume multiplicity"
    );
}

#[test]
fn affine_empty_resume_row_stays_affine_and_known() {
    let lowered = checked_lower(
        handler_expr(
            resume_param(CoreRow::default(), CoreMultiplicity::Affine),
            CoreExpr::Jump {
                cont: CoreContRef::Var("resume".to_string()),
                arg: CoreAtom::LitString("ok".to_string()),
            },
        ),
        &env_with_ops(),
    );

    let clause = first_handler_clause(&lowered).expect("lowered term should contain Handle");
    assert_eq!(
        clause.resume_row,
        ResumeRowMetadata::Known(EffectRow::default())
    );
    assert_eq!(
        clause.resume_multiplicity,
        ContMultiplicity::Affine,
        "empty row alone must not imply multi-shot-pure"
    );
}

#[test]
fn checked_handler_resume_lowers_non_empty_known_row() {
    let resume_row = cap_row(&["audit"], "emit");
    let lowered = checked_lower(
        handler_expr(
            resume_param(resume_row, CoreMultiplicity::Affine),
            CoreExpr::Jump {
                cont: CoreContRef::Var("resume".to_string()),
                arg: CoreAtom::LitString("ok".to_string()),
            },
        ),
        &env_with_ops(),
    );

    let clause = first_handler_clause(&lowered).expect("lowered term should contain Handle");
    assert_eq!(
        clause.resume_row,
        ResumeRowMetadata::Known(cps_cap_row(&["audit"], "emit"))
    );
    assert_eq!(clause.resume_multiplicity, ContMultiplicity::Affine);
}

#[test]
fn core_let_call_binder_lowers_to_let_cont_with_checked_row_and_affine_multiplicity() {
    let body_row = cap_row(&["audit"], "emit");
    let id_lam = CoreValue::Lam {
        params: vec![param("input", string_ty())],
        body: Box::new(CoreExpr::Atom(CoreAtom::Var("input".to_string()))),
        row: CoreRow::default(),
    };
    let expr = CoreExpr::LetVal {
        name: "id".to_string(),
        ty: function_ty(vec![string_ty()], string_ty(), CoreRow::default()),
        value: id_lam,
        body: Box::new(CoreExpr::LetCall {
            name: "value".to_string(),
            func: CoreAtom::Var("id".to_string()),
            args: vec![CoreAtom::LitString("ok".to_string())],
            body: Box::new(CoreExpr::Raise {
                op: audit_op(),
                args: vec![CoreAtom::Var("value".to_string())],
            }),
        }),
    };
    let lowered = checked_lower(expr, &env_with_ops());

    let let_cont = first_let_cont(&lowered).expect("LetCall should lower through LetCont");
    let Term::LetCont {
        row, multiplicity, ..
    } = let_cont
    else {
        unreachable!("first_let_cont returns only LetCont")
    };
    assert_eq!(row, &cps_cap_row(&["audit"], "emit"));
    assert_eq!(
        *multiplicity,
        ContMultiplicity::Affine,
        "ordinary Core call continuations are affine unless explicitly typed otherwise"
    );
    assert_eq!(
        row,
        &cps_cap_row(&["audit"], "emit"),
        "sanity: body row used"
    );
    assert_eq!(
        body_row,
        cap_row(&["audit"], "emit"),
        "sanity: expected Core row"
    );
}

#[test]
fn core_let_cont_call_lowers_to_cps_let_cont_call_with_checked_row() {
    let mut env = CoreTypeCheckEnv::default();
    env.continuations_mut().insert(
        "k".to_string(),
        cont_ty(
            int_ty(),
            string_ty(),
            cap_row(&["audit"], "emit"),
            CoreMultiplicity::Affine,
        ),
    );
    let lowered = checked_lower(
        CoreExpr::LetContCall {
            name: "answer".to_string(),
            cont: CoreContRef::Label("k".to_string()),
            arg: CoreAtom::LitInt(7),
            body: Box::new(CoreExpr::Atom(CoreAtom::Var("answer".to_string()))),
        },
        &env,
    );

    let call = first_let_cont_call(&lowered).expect("Core LetContCall should lower directly");
    let Term::LetContCall {
        name,
        cont,
        arg,
        row,
        ..
    } = call
    else {
        unreachable!("first_let_cont_call returns only LetContCall")
    };
    assert_eq!(name, "answer");
    assert_eq!(cont, "k");
    assert_eq!(arg, &Atom::Int(7));
    assert_eq!(row, &cps_cap_row(&["audit"], "emit"));
}

#[test]
fn checked_multishot_handler_metadata_uses_known_resume_row() {
    let lowered = checked_lower(
        handler_expr(
            resume_param(CoreRow::default(), CoreMultiplicity::MultiShotPure),
            CoreExpr::LetPrim {
                name: "same".to_string(),
                op: CorePrimOp::Add,
                args: vec![CoreAtom::LitInt(1), CoreAtom::LitInt(1)],
                body: Box::new(CoreExpr::LetContCall {
                    name: "answer".to_string(),
                    cont: CoreContRef::Var("resume".to_string()),
                    arg: CoreAtom::LitString("ok".to_string()),
                    body: Box::new(CoreExpr::Atom(CoreAtom::Var("answer".to_string()))),
                }),
            },
        ),
        &env_with_ops(),
    );

    let clause = first_handler_clause(&lowered).expect("lowered term should contain Handle");
    assert!(
        !matches!(clause.resume_row, ResumeRowMetadata::InheritFromTarget),
        "checked multi-shot lowering must emit known resume-row metadata"
    );
    assert_eq!(clause.resume_multiplicity, ContMultiplicity::MultiShotPure);
}
