use ash_core::core_ash::{
    CoreAtom, CoreContRef, CoreContractDischarge, CoreDischargeMode, CoreEffectOp, CoreExpr,
    CoreHandlerClause, CoreMultiplicity, CoreParam, CoreRow, CoreRowItem, CoreTrapReason, CoreType,
};
use ash_core::core_ash_lower::{CoreLoweringContext, lower_core_program_with_context};
use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};
use ash_core::cps::{
    Atom, ContRef, DischargeType, EffectItemKind, Term, TrapReason as CpsTrapReason,
};

fn base(name: &str) -> CoreType {
    CoreType::Base(name.to_string())
}

fn unit() -> CoreType {
    base("Unit")
}

fn string() -> CoreType {
    base("String")
}

fn row_cap(path: &[&str], operation: &str) -> CoreRow {
    CoreRow::closed(vec![CoreRowItem::Capability {
        path: path.iter().map(|part| (*part).to_string()).collect(),
        operation: operation.to_string(),
    }])
}

fn validate(expr: CoreExpr) -> ash_core::core_ash_validate::ValidCoreProgram {
    validate_core_program(RawCoreProgram::new(expr)).expect("test Core program should validate")
}

fn lower(expr: CoreExpr, current_row: CoreRow) -> Term {
    lower_core_program_with_context(
        validate(expr),
        CoreLoweringContext::new(ContRef::Label("exit".to_string()), current_row),
    )
    .expect("lowering should succeed")
}

fn console_read_op() -> CoreEffectOp {
    CoreEffectOp::Capability {
        path: vec!["console".to_string()],
        operation: "read".to_string(),
        arg_types: vec![string()],
        result_type: string(),
    }
}

#[test]
fn lowers_capability_raise_with_current_resume_and_operation_row_only() {
    let lowered = lower(
        CoreExpr::Raise {
            op: console_read_op(),
            args: vec![CoreAtom::LitString("prompt".to_string())],
        },
        row_cap(&["console"], "write"),
    );

    let Term::Raise {
        op,
        args,
        resume,
        row,
    } = lowered
    else {
        panic!("expected CPS Raise");
    };

    assert_eq!(args, vec![Atom::String("prompt".to_string())]);
    assert_eq!(resume, ContRef::Label("exit".to_string()));
    assert_eq!(op.item.namespace, "cap");
    assert_eq!(op.item.name, "console.read");
    assert_eq!(op.item.kind, EffectItemKind::Capability);
    assert_eq!(op.arg_types, vec!["String".to_string()]);
    assert_eq!(op.result_type, "String");
    assert_eq!(row.items.len(), 1);
    assert_eq!(row.items[0].namespace, "cap");
    assert_eq!(row.items[0].name, "console.read");
}

#[test]
fn lowers_failure_raise_as_explicit_fail_effect_row() {
    let lowered = lower(
        CoreExpr::Raise {
            op: CoreEffectOp::Failure {
                ty: Some(base("ConfigError")),
            },
            args: vec![CoreAtom::Var("err".to_string())],
        },
        CoreRow::default(),
    );

    let Term::Raise { op, row, .. } = lowered else {
        panic!("expected CPS Raise");
    };
    assert_eq!(op.item.namespace, "fail");
    assert_eq!(op.item.name, "ConfigError");
    assert_eq!(op.item.kind, EffectItemKind::Alias);
    assert_eq!(op.result_type, "Never");
    assert_eq!(row.items.len(), 1);
    assert_eq!(row.items[0].namespace, "fail");
    assert_eq!(row.items[0].name, "ConfigError");
}

#[test]
fn lowers_handle_with_outer_continuation_and_local_residual_row() {
    let handler_row = row_cap(&["audit"], "write");
    let outer_row = row_cap(&["console"], "write");
    let lowered = lower(
        CoreExpr::Handle {
            clause: CoreHandlerClause {
                op: console_read_op(),
                params: vec![CoreParam {
                    name: "line".to_string(),
                    ty: string(),
                }],
                resume: CoreParam {
                    name: "k".to_string(),
                    ty: CoreType::Cont {
                        input: Box::new(string()),
                        answer: Box::new(unit()),
                        row: row_cap(&["resume"], "audit"),
                        multiplicity: CoreMultiplicity::Affine,
                    },
                },
                body: Box::new(CoreExpr::Jump {
                    cont: CoreContRef::Var("k".to_string()),
                    arg: CoreAtom::Var("line".to_string()),
                }),
                row: handler_row.clone(),
            },
            body: Box::new(CoreExpr::Raise {
                op: console_read_op(),
                args: vec![CoreAtom::LitString("prompt".to_string())],
            }),
        },
        outer_row,
    );

    let Term::Handle {
        clause,
        body,
        cont,
        row,
    } = lowered
    else {
        panic!("expected CPS Handle");
    };

    assert_eq!(cont, ContRef::Label("exit".to_string()));
    assert_eq!(row.items.len(), 1);
    assert_eq!(row.items[0].namespace, "cap");
    assert_eq!(row.items[0].name, "audit.write");
    assert_eq!(clause.params, vec!["line".to_string()]);
    assert_eq!(clause.resume, "k");
    assert_eq!(clause.row.items[0].name, "audit.write");
    assert!(matches!(
        *clause.body,
        Term::Jump {
            cont: ContRef::Var(ref name),
            ref row,
            ..
        } if name == "k" && row.items.len() == 1 && row.items[0].name == "resume.audit"
    ));
    assert!(matches!(*body, Term::Raise { .. }));
    assert!(
        row.items.iter().all(|item| item.name != "console.write"),
        "Handle.row must exclude the outer continuation row"
    );
}

#[test]
fn lowers_dynamic_contract_record_discharge_and_contract_trap() {
    let lowered = lower(
        CoreExpr::RecordDischarge {
            discharge: CoreContractDischarge {
                contract: "requires-positive".to_string(),
                mode: CoreDischargeMode::Dynamic,
                evidence: None,
                source_span: None,
            },
            body: Box::new(CoreExpr::Trap {
                reason: CoreTrapReason::ContractViolation("requires-positive".to_string()),
            }),
        },
        CoreRow::default(),
    );

    let Term::RecordDischarge { discharge, body } = lowered else {
        panic!("expected CPS RecordDischarge");
    };
    assert_eq!(discharge.contract, "requires-positive");
    assert_eq!(discharge.discharge_type, DischargeType::Dynamic);
    assert!(matches!(
        *body,
        Term::Trap {
            reason: CpsTrapReason::ContractViolation
        }
    ));
}

#[test]
fn contract_violation_trap_does_not_create_effect_row_item() {
    let lowered = lower(
        CoreExpr::If {
            cond: CoreAtom::LitBool(true),
            then_branch: Box::new(CoreExpr::Trap {
                reason: CoreTrapReason::ContractViolation("requires-positive".to_string()),
            }),
            else_branch: Box::new(CoreExpr::Raise {
                op: CoreEffectOp::Failure {
                    ty: Some(base("RecoverableContractFailure")),
                },
                args: vec![CoreAtom::LitUnit],
            }),
        },
        CoreRow::default(),
    );

    let Term::If { row, .. } = lowered else {
        panic!("expected CPS If");
    };
    assert_eq!(row.items.len(), 1);
    assert_eq!(row.items[0].namespace, "fail");
    assert_eq!(row.items[0].name, "RecoverableContractFailure");
    assert!(
        row.items
            .iter()
            .all(|item| item.name != "requires-positive" && item.namespace != "contract"),
        "ContractViolation must remain trap metadata, not a row item"
    );
}

#[test]
fn lowers_static_discharge_as_static_cps_discharge() {
    let lowered = lower(
        CoreExpr::RecordDischarge {
            discharge: CoreContractDischarge {
                contract: "ensures-id".to_string(),
                mode: CoreDischargeMode::Static,
                evidence: None,
                source_span: None,
            },
            body: Box::new(CoreExpr::Jump {
                cont: CoreContRef::Label("exit".to_string()),
                arg: CoreAtom::LitInt(1),
            }),
        },
        CoreRow::default(),
    );

    let Term::RecordDischarge { discharge, .. } = lowered else {
        panic!("expected CPS RecordDischarge");
    };
    assert_eq!(discharge.contract, "ensures-id");
    assert_eq!(discharge.discharge_type, DischargeType::Static);
}

#[test]
fn lowers_unhandled_effect_and_panic_traps() {
    let lowered = lower(
        CoreExpr::Trap {
            reason: CoreTrapReason::UnhandledEffect(CoreEffectOp::Failure { ty: None }),
        },
        CoreRow::default(),
    );
    assert!(matches!(
        lowered,
        Term::Trap {
            reason: CpsTrapReason::Custom(_)
        }
    ));

    let lowered = lower(
        CoreExpr::Trap {
            reason: CoreTrapReason::Panic("boom".to_string()),
        },
        CoreRow::default(),
    );
    assert!(matches!(
        lowered,
        Term::Trap {
            reason: CpsTrapReason::Custom(ref reason)
        } if reason == "panic: boom"
    ));
}
