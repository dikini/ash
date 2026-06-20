use std::fs;

use ash_core::core_ash::{
    CoreAtom, CoreContRef, CoreContractDischarge, CoreDischargeMode, CoreEffectOp, CoreExpr,
    CoreHandlerClause, CoreMultiplicity, CoreParam, CorePrimOp, CoreRow, CoreTrapReason, CoreType,
    CoreValue,
};
use ash_core::core_ash_text::{core_expr_to_string, parse_core_expr, write_core_expr_to_file};

fn base(name: &str) -> CoreType {
    CoreType::Base(name.to_string())
}

fn empty_row() -> CoreRow {
    CoreRow::default()
}

fn contract(name: &str, mode: CoreDischargeMode) -> CoreContractDischarge {
    CoreContractDischarge {
        contract: name.to_string(),
        mode,
        evidence: None,
        source_span: None,
    }
}

fn assert_canonical_round_trip(expr: CoreExpr, expected: &str) {
    let text = core_expr_to_string(&expr);
    assert_eq!(text, expected);
    assert_eq!(core_expr_to_string(&expr), text);

    let parsed = parse_core_expr(&text).expect("serialized Core text should parse");
    assert_eq!(parsed, expr);
}

#[test]
fn serializes_simple_let_jump_expression_canonically() {
    let expr = CoreExpr::LetVal {
        name: "x".to_string(),
        ty: base("Int"),
        value: CoreValue::Atom(CoreAtom::LitInt(1)),
        body: Box::new(CoreExpr::Jump {
            cont: CoreContRef::Label("exit".to_string()),
            arg: CoreAtom::Var("x".to_string()),
        }),
    };

    assert_canonical_round_trip(expr, "(let-val x : Int (lit-int 1) (jump (label exit) x))");
}

#[test]
fn serializes_nested_let_if_expression_canonically() {
    let expr = CoreExpr::LetPrim {
        name: "cond".to_string(),
        op: CorePrimOp::Lt,
        args: vec![CoreAtom::LitInt(1), CoreAtom::LitInt(10)],
        body: Box::new(CoreExpr::If {
            cond: CoreAtom::Var("cond".to_string()),
            then_branch: Box::new(CoreExpr::Jump {
                cont: CoreContRef::Label("yes".to_string()),
                arg: CoreAtom::LitUnit,
            }),
            else_branch: Box::new(CoreExpr::Jump {
                cont: CoreContRef::Label("no".to_string()),
                arg: CoreAtom::LitUnit,
            }),
        }),
    };

    assert_canonical_round_trip(
        expr,
        "(let-prim cond lt ((lit-int 1) (lit-int 10)) (if cond (jump (label yes) (lit-unit)) (jump (label no) (lit-unit))))",
    );
}

#[test]
fn serializes_handler_expression_with_affine_resume_canonically() {
    let op = CoreEffectOp::Capability {
        path: vec!["console".to_string()],
        operation: "read".to_string(),
        arg_types: vec![base("String")],
        result_type: base("Unit"),
    };
    let expr = CoreExpr::Handle {
        clause: CoreHandlerClause {
            op: op.clone(),
            params: vec![CoreParam {
                name: "line".to_string(),
                ty: base("String"),
            }],
            resume: CoreParam {
                name: "k".to_string(),
                ty: CoreType::Cont {
                    input: Box::new(base("Unit")),
                    answer: Box::new(base("Unit")),
                    row: empty_row(),
                    multiplicity: CoreMultiplicity::Affine,
                },
            },
            body: Box::new(CoreExpr::Jump {
                cont: CoreContRef::Var("k".to_string()),
                arg: CoreAtom::LitUnit,
            }),
            row: empty_row(),
        },
        body: Box::new(CoreExpr::Raise {
            op,
            args: vec![CoreAtom::LitString("ok".to_string())],
        }),
    };

    assert_canonical_round_trip(
        expr,
        "(handle (clause (cap console.read : (String) -> Unit) ((line : String)) (resume k : (cont Unit Unit {} affine)) : {} (jump k (lit-unit))) (raise (cap console.read : (String) -> Unit) ((lit-string \"ok\"))))",
    );
}

#[test]
fn serializes_contract_trap_expression_canonically() {
    let expr = CoreExpr::RecordDischarge {
        discharge: contract("requires-positive", CoreDischargeMode::Dynamic),
        body: Box::new(CoreExpr::Trap {
            reason: CoreTrapReason::ContractViolation("requires-positive".to_string()),
        }),
    };

    assert_canonical_round_trip(
        expr,
        "(record-discharge (contract requires-positive dynamic) (trap (contract-violation requires-positive)))",
    );
}

#[test]
fn writes_core_expression_files_with_canonical_text() {
    let expr = CoreExpr::Atom(CoreAtom::LitBool(true));
    let path = std::env::temp_dir().join(format!(
        "ash-task-1624-serializer-{}.core",
        std::process::id()
    ));

    write_core_expr_to_file(&path, &expr).expect("write should succeed");
    let written = fs::read_to_string(&path).expect("written file should be readable");
    let _ = fs::remove_file(&path);

    assert_eq!(written, "(lit-bool true)\n");
    assert_eq!(parse_core_expr(&written).unwrap(), expr);
}
