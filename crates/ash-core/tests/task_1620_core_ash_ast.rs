use ash_core::core_ash::{
    CoreAtom, CoreContRef, CoreContractDischarge, CoreDischargeMode, CoreEffectOp, CoreExpr,
    CoreHandlerClause, CoreMultiplicity, CoreParam, CorePrimOp, CoreRow, CoreRowItem,
    CoreTrapReason, CoreType, CoreValue,
};

fn int_type() -> CoreType {
    CoreType::Base("Int".to_string())
}

fn unit_type() -> CoreType {
    CoreType::Base("Unit".to_string())
}

fn cap_op(name: &str) -> CoreEffectOp {
    CoreEffectOp::Capability {
        path: vec!["test".to_string()],
        operation: name.to_string(),
        arg_types: vec![int_type()],
        result_type: unit_type(),
    }
}

#[test]
fn constructs_direct_style_let_val_and_jump_core_expression() {
    let expr = CoreExpr::LetVal {
        name: "x".to_string(),
        ty: int_type(),
        value: CoreValue::Atom(CoreAtom::LitInt(1)),
        body: Box::new(CoreExpr::Jump {
            cont: CoreContRef::Label("exit".to_string()),
            arg: CoreAtom::Var("x".to_string()),
        }),
    };

    let cloned = expr.clone();
    assert_eq!(expr, cloned);

    match expr {
        CoreExpr::LetVal {
            name, value, body, ..
        } => {
            assert_eq!(name, "x");
            assert_eq!(value, CoreValue::Atom(CoreAtom::LitInt(1)));
            assert!(matches!(
                *body,
                CoreExpr::Jump {
                    cont: CoreContRef::Label(_),
                    arg: CoreAtom::Var(_)
                }
            ));
        }
        other => panic!("expected CoreExpr::LetVal, got {other:?}"),
    }
}

#[test]
fn handler_clause_represents_affine_resume_parameter_metadata() {
    let resume_ty = CoreType::Cont {
        input: Box::new(unit_type()),
        answer: Box::new(unit_type()),
        row: CoreRow::closed(vec![CoreRowItem::Failure { ty: None }]),
        multiplicity: CoreMultiplicity::Affine,
    };
    let clause = CoreHandlerClause {
        op: cap_op("read"),
        params: vec![CoreParam {
            name: "result".to_string(),
            ty: unit_type(),
        }],
        resume: CoreParam {
            name: "resume".to_string(),
            ty: resume_ty.clone(),
        },
        body: Box::new(CoreExpr::Jump {
            cont: CoreContRef::Var("resume".to_string()),
            arg: CoreAtom::LitUnit,
        }),
        row: CoreRow::closed(vec![CoreRowItem::Failure { ty: None }]),
    };

    assert_eq!(clause.resume.ty, resume_ty);
    assert!(matches!(
        clause.resume.ty,
        CoreType::Cont {
            multiplicity: CoreMultiplicity::Affine,
            ..
        }
    ));
}

#[test]
fn row_items_and_effect_ops_exclude_contract_violation_operation() {
    let row = CoreRow::closed(vec![
        CoreRowItem::Capability {
            path: vec!["io".to_string()],
            operation: "write".to_string(),
        },
        CoreRowItem::Failure {
            ty: Some(Box::new(CoreType::Named("Error".to_string()))),
        },
    ]);

    let ops = [
        CoreEffectOp::Capability {
            path: vec!["io".to_string()],
            operation: "write".to_string(),
            arg_types: vec![CoreType::Base("String".to_string())],
            result_type: unit_type(),
        },
        CoreEffectOp::Channel {
            path: vec!["jobs".to_string()],
            mode: "send".to_string(),
            payload_type: CoreType::Named("Job".to_string()),
            result_type: unit_type(),
        },
        CoreEffectOp::Process {
            operation: "spawn".to_string(),
            arg_types: vec![CoreType::Named("Command".to_string())],
            result_type: CoreType::Named("ProcessHandle".to_string()),
        },
        CoreEffectOp::Failure {
            ty: Some(CoreType::Named("Error".to_string())),
        },
    ];

    assert_eq!(row.items.len(), 2);
    assert_eq!(ops.len(), 4);
    assert!(ops.iter().all(CoreEffectOp::is_raised_operation));
}

#[test]
fn contract_violation_is_only_a_trap_reason() {
    let discharge = CoreContractDischarge {
        contract: "requires-positive".to_string(),
        mode: CoreDischargeMode::Dynamic,
        evidence: None,
        source_span: None,
    };

    let trap = CoreExpr::Trap {
        reason: CoreTrapReason::ContractViolation(discharge.contract.clone()),
    };
    let recorded = CoreExpr::RecordDischarge {
        discharge,
        body: Box::new(trap.clone()),
    };

    assert!(matches!(
        recorded,
        CoreExpr::RecordDischarge {
            body,
            ..
        } if *body == trap
    ));
}

#[test]
fn primitive_names_are_core_specific_not_cps_reexports() {
    let expr = CoreExpr::LetPrim {
        name: "sum".to_string(),
        op: CorePrimOp::Add,
        args: vec![CoreAtom::LitInt(1), CoreAtom::LitInt(2)],
        body: Box::new(CoreExpr::Atom(CoreAtom::Var("sum".to_string()))),
    };

    assert!(matches!(
        expr,
        CoreExpr::LetPrim {
            op: CorePrimOp::Add,
            ..
        }
    ));
}
