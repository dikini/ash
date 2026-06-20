use ash_core::core_ash::{
    CoreAtom, CoreContRef, CoreEffectOp, CoreExpr, CoreHandlerClause, CoreMultiplicity, CoreParam,
    CoreRow, CoreType, CoreValue,
};
use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};

fn unit() -> CoreType {
    CoreType::Base("Unit".to_string())
}

fn empty_row() -> CoreRow {
    CoreRow::default()
}

fn resume_type() -> CoreType {
    CoreType::Cont {
        input: Box::new(unit()),
        answer: Box::new(unit()),
        row: empty_row(),
        multiplicity: CoreMultiplicity::Affine,
    }
}

fn console_read_op() -> CoreEffectOp {
    CoreEffectOp::Capability {
        path: vec!["console".to_string()],
        operation: "read".to_string(),
        arg_types: vec![unit()],
        result_type: unit(),
    }
}

fn handler_with_body(body: CoreExpr) -> CoreExpr {
    CoreExpr::Handle {
        clause: CoreHandlerClause {
            op: console_read_op(),
            params: vec![CoreParam {
                name: "line".to_string(),
                ty: unit(),
            }],
            resume: CoreParam {
                name: "k".to_string(),
                ty: resume_type(),
            },
            body: Box::new(body),
            row: empty_row(),
        },
        body: Box::new(CoreExpr::Raise {
            op: console_read_op(),
            args: vec![CoreAtom::LitUnit],
        }),
    }
}

fn assert_valid(expr: CoreExpr) {
    validate_core_program(RawCoreProgram::new(expr)).expect("Core expression should validate");
}

fn assert_affine_error(expr: CoreExpr) {
    let error = validate_core_program(RawCoreProgram::new(expr)).unwrap_err();
    assert!(
        error.to_string().contains("affine resume"),
        "unexpected error: {error}"
    );
}

#[test]
fn one_direct_resume_jump_is_accepted() {
    assert_valid(handler_with_body(CoreExpr::Jump {
        cont: CoreContRef::Var("k".to_string()),
        arg: CoreAtom::LitUnit,
    }));
}

#[test]
fn no_resume_jump_is_accepted_for_non_resumptive_handler() {
    assert_valid(handler_with_body(CoreExpr::Trap {
        reason: ash_core::core_ash::CoreTrapReason::Panic("stop".to_string()),
    }));
}

#[test]
fn duplicate_resume_jumps_are_rejected_conservatively() {
    assert_affine_error(handler_with_body(CoreExpr::If {
        cond: CoreAtom::LitBool(true),
        then_branch: Box::new(CoreExpr::Jump {
            cont: CoreContRef::Var("k".to_string()),
            arg: CoreAtom::LitUnit,
        }),
        else_branch: Box::new(CoreExpr::Jump {
            cont: CoreContRef::Var("k".to_string()),
            arg: CoreAtom::LitUnit,
        }),
    }));
}

#[test]
fn resume_passed_as_ordinary_call_argument_is_rejected() {
    assert_affine_error(handler_with_body(CoreExpr::Call {
        func: CoreAtom::Var("use_cont".to_string()),
        args: vec![CoreAtom::Var("k".to_string())],
    }));
}

#[test]
fn resume_stored_in_record_value_is_rejected() {
    assert_affine_error(handler_with_body(CoreExpr::LetVal {
        name: "saved".to_string(),
        ty: unit(),
        value: CoreValue::Record {
            fields: vec![("resume".to_string(), CoreAtom::Var("k".to_string()))],
        },
        body: Box::new(CoreExpr::Trap {
            reason: ash_core::core_ash::CoreTrapReason::Panic("stop".to_string()),
        }),
    }));
}
