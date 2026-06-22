use ash_core::core_ash::{
    CoreAtom, CoreContRef, CoreExpr, CoreParam, CorePrimOp, CoreRow, CoreRowItem, CoreType,
    CoreValue,
};
use ash_core::core_ash_lower::{CoreLoweringContext, lower_core_program_with_context};
use ash_core::core_ash_text::{core_expr_to_string, parse_core_expr};
use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};
use ash_core::cps::{Atom, ContRef, EffectItemKind, PrimOp, Term, Value};

fn int() -> CoreType {
    CoreType::Base("Int".to_string())
}

fn cap_row(path: &[&str], operation: &str) -> CoreRow {
    CoreRow::closed(vec![CoreRowItem::Capability {
        path: path.iter().map(|segment| (*segment).to_string()).collect(),
        operation: operation.to_string(),
    }])
}

fn validate(expr: CoreExpr) -> ash_core::core_ash_validate::ValidCoreProgram {
    validate_core_program(RawCoreProgram::new(expr)).expect("test Core program should validate")
}

#[test]
fn lowers_pure_let_prim_then_jump() {
    let program = validate(CoreExpr::LetPrim {
        name: "sum".to_string(),
        op: CorePrimOp::Add,
        args: vec![CoreAtom::Var("a".to_string()), CoreAtom::LitInt(2)],
        body: Box::new(CoreExpr::Jump {
            cont: CoreContRef::Label("exit".to_string()),
            arg: CoreAtom::Var("sum".to_string()),
        }),
    });

    let lowered = lower_core_program_with_context(
        program,
        CoreLoweringContext::new(ContRef::Label("exit".to_string()), CoreRow::default()),
    )
    .expect("lowering should succeed");

    assert_eq!(
        lowered,
        Term::LetPrim {
            name: "sum".to_string(),
            op: PrimOp::Add,
            args: vec![Atom::Var("a".to_string()), Atom::Int(2)],
            body: Box::new(Term::Jump {
                cont: ContRef::Label("exit".to_string()),
                arg: Atom::Var("sum".to_string()),
                row: Default::default(),
            }),
        }
    );
}

#[test]
fn lowers_if_with_local_branch_row_excluding_jump_continuation_row() {
    let program = validate(CoreExpr::If {
        cond: CoreAtom::Var("ok".to_string()),
        then_branch: Box::new(CoreExpr::Jump {
            cont: CoreContRef::Label("exit".to_string()),
            arg: CoreAtom::LitInt(1),
        }),
        else_branch: Box::new(CoreExpr::Jump {
            cont: CoreContRef::Label("exit".to_string()),
            arg: CoreAtom::LitInt(0),
        }),
    });
    let context = CoreLoweringContext::new(
        ContRef::Label("exit".to_string()),
        cap_row(&["console"], "write"),
    )
    .with_cont_row("exit", cap_row(&["console"], "write"));

    let lowered =
        lower_core_program_with_context(program, context).expect("lowering should succeed");

    let Term::If {
        cond,
        then_branch,
        else_branch,
        row,
    } = lowered
    else {
        panic!("expected CPS If");
    };
    assert_eq!(cond, Atom::Var("ok".to_string()));
    assert!(
        row.items.is_empty(),
        "If.row should contain only local branch effects"
    );
    assert!(matches!(*then_branch, Term::Jump { row, .. } if row.items.len() == 1));
    assert!(matches!(*else_branch, Term::Jump { row, .. } if row.items.len() == 1));
}

#[test]
fn lowers_tail_call_with_function_and_current_continuation_rows() {
    let function_row = cap_row(&["db"], "read");
    let continuation_row = cap_row(&["console"], "write");
    let program = validate(CoreExpr::LetVal {
        name: "read_user".to_string(),
        ty: CoreType::Function {
            params: vec![int()],
            result: Box::new(int()),
            row: function_row.clone(),
        },
        value: CoreValue::Atom(CoreAtom::Var("read_user_impl".to_string())),
        body: Box::new(CoreExpr::Call {
            func: CoreAtom::Var("read_user".to_string()),
            args: vec![CoreAtom::LitInt(7)],
        }),
    });
    let context =
        CoreLoweringContext::new(ContRef::Label("exit".to_string()), continuation_row.clone());

    let lowered =
        lower_core_program_with_context(program, context).expect("lowering should succeed");

    let Term::LetVal { body, .. } = lowered else {
        panic!("expected outer LetVal");
    };
    let Term::Call {
        func,
        args,
        cont,
        row,
    } = *body
    else {
        panic!("expected tail CPS Call");
    };
    assert_eq!(func, Atom::Var("read_user".to_string()));
    assert_eq!(args, vec![Atom::Int(7)]);
    assert_eq!(cont, ContRef::Label("exit".to_string()));
    assert_eq!(row.items.len(), 2);
    assert!(row.items.iter().any(|item| {
        item.namespace == "cap" && item.name == "db.read" && item.kind == EffectItemKind::Capability
    }));
    assert!(row.items.iter().any(|item| {
        item.namespace == "cap"
            && item.name == "console.write"
            && item.kind == EffectItemKind::Capability
    }));
}

#[test]
fn parses_and_serializes_let_call_text_form() {
    let source = "(let-call user read_user ((lit-int 7)) (jump (label exit) user))";
    let parsed = parse_core_expr(source).expect("let-call text should parse");

    assert!(matches!(parsed, CoreExpr::LetCall { .. }));
    assert_eq!(core_expr_to_string(&parsed), source);
}

#[test]
fn lowers_non_tail_call_by_introducing_let_cont() {
    let function_row = cap_row(&["db"], "read");
    let continuation_row = cap_row(&["console"], "write");
    let program = validate(CoreExpr::LetCall {
        name: "user".to_string(),
        func: CoreAtom::Var("read_user".to_string()),
        args: vec![CoreAtom::LitInt(7)],
        body: Box::new(CoreExpr::Jump {
            cont: CoreContRef::Label("exit".to_string()),
            arg: CoreAtom::Var("user".to_string()),
        }),
    });
    let context =
        CoreLoweringContext::new(ContRef::Label("exit".to_string()), continuation_row.clone())
            .with_function_row("read_user", function_row);

    let lowered =
        lower_core_program_with_context(program, context).expect("lowering should succeed");

    let Term::LetCont {
        name,
        param,
        cont_body,
        body,
        ..
    } = lowered
    else {
        panic!("expected LetCont for non-tail call");
    };
    assert_eq!(name, "__k0");
    assert_eq!(param, "user");
    assert!(matches!(
        *cont_body,
        Term::Jump {
            cont: ContRef::Label(ref label),
            arg: Atom::Var(ref arg),
            ref row,
        } if label == "exit" && arg == "user" && row.items.len() == 1
    ));

    let Term::Call {
        func,
        args,
        cont,
        row,
    } = *body
    else {
        panic!("expected LetCont rest to call function");
    };
    assert_eq!(func, Atom::Var("read_user".to_string()));
    assert_eq!(args, vec![Atom::Int(7)]);
    assert_eq!(cont, ContRef::Label("__k0".to_string()));
    assert_eq!(row.items.len(), 2);
}

#[test]
fn lowers_lambda_body_atom_to_jump_to_fresh_continuation_parameter() {
    let program = validate(CoreExpr::LetVal {
        name: "id".to_string(),
        ty: CoreType::Function {
            params: vec![int()],
            result: Box::new(int()),
            row: CoreRow::default(),
        },
        value: CoreValue::Lam {
            params: vec![CoreParam {
                name: "x".to_string(),
                ty: int(),
            }],
            row: CoreRow::default(),
            body: Box::new(CoreExpr::Atom(CoreAtom::Var("x".to_string()))),
        },
        body: Box::new(CoreExpr::Jump {
            cont: CoreContRef::Label("exit".to_string()),
            arg: CoreAtom::LitUnit,
        }),
    });

    let lowered = lower_core_program_with_context(
        program,
        CoreLoweringContext::new(ContRef::Label("exit".to_string()), CoreRow::default()),
    )
    .expect("lowering should succeed");

    let Term::LetVal { value, .. } = lowered else {
        panic!("expected outer LetVal");
    };
    let Value::Lam {
        params, cont, body, ..
    } = value
    else {
        panic!("expected lowered lambda value");
    };
    assert_eq!(params, vec!["x".to_string()]);
    assert_eq!(cont, "__k0".to_string());
    assert!(matches!(
        *body,
        Term::Jump {
            cont: ContRef::Var(ref name),
            arg: Atom::Var(ref arg),
            ..
        } if name == "__k0" && arg == "x"
    ));
}

#[test]
fn rejects_unrepresentable_core_primitive_names_during_lowering() {
    let program = validate(CoreExpr::LetPrim {
        name: "tag".to_string(),
        op: CorePrimOp::ConstructorTag("Some".to_string()),
        args: vec![CoreAtom::Var("value".to_string())],
        body: Box::new(CoreExpr::Atom(CoreAtom::Var("tag".to_string()))),
    });

    let error = lower_core_program_with_context(
        program,
        CoreLoweringContext::new(ContRef::Label("exit".to_string()), CoreRow::default()),
    )
    .unwrap_err();

    assert!(
        error.to_string().contains("primitive cannot lower"),
        "unexpected error: {error}"
    );
}
