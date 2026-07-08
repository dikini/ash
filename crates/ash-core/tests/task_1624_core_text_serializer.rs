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
    let op = CoreEffectOp::Operation {
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
        "(handle (clause (operation console.read : (String) -> Unit) ((line : String)) (resume k : (cont Unit Unit {} affine)) : {} (jump k (lit-unit))) (raise (operation console.read : (String) -> Unit) ((lit-string \"ok\"))))",
    );
}

#[test]
fn serializes_open_rows_canonically_and_parseably() {
    let open_row = CoreRow::open(
        vec![ash_core::core_ash::CoreRowItem::Operation {
            path: vec!["console".to_string()],
            operation: "read".to_string(),
        }],
        "r".to_string(),
    );
    let expr = CoreExpr::LetVal {
        name: "f".to_string(),
        ty: CoreType::Function {
            params: vec![base("Int")],
            result: Box::new(base("Int")),
            row: open_row.clone(),
        },
        value: CoreValue::Lam {
            params: vec![CoreParam {
                name: "x".to_string(),
                ty: base("Int"),
            }],
            body: Box::new(CoreExpr::Atom(CoreAtom::Var("x".to_string()))),
            row: open_row,
        },
        body: Box::new(CoreExpr::Jump {
            cont: CoreContRef::Label("exit".to_string()),
            arg: CoreAtom::Var("f".to_string()),
        }),
    };

    assert_canonical_round_trip(
        expr,
        "(let-val f : (fn (Int) -> Int {operation console.read, tail r}) (lam ((x : Int)) : {operation console.read, tail r} x) (jump (label exit) f))",
    );
}

#[test]
fn serializes_public_type_forms_canonically_and_parseably() {
    let expr = CoreExpr::LetVal {
        name: "x".to_string(),
        ty: CoreType::Record(vec![
            (
                "id".to_string(),
                CoreType::App {
                    name: "Box".to_string(),
                    args: vec![CoreType::Refinement {
                        base: Box::new(base("Int")),
                        predicate: "value > 0".to_string(),
                    }],
                },
            ),
            (
                "flag".to_string(),
                CoreType::Cont {
                    input: Box::new(base("Bool")),
                    answer: Box::new(base("Unit")),
                    row: CoreRow::open(Vec::new(), "resume".to_string()),
                    multiplicity: CoreMultiplicity::Affine,
                },
            ),
        ]),
        value: CoreValue::Record {
            fields: vec![("id".to_string(), CoreAtom::LitInt(1))],
        },
        body: Box::new(CoreExpr::Jump {
            cont: CoreContRef::Label("exit".to_string()),
            arg: CoreAtom::Var("x".to_string()),
        }),
    };

    assert_canonical_round_trip(
        expr,
        "(let-val x : (record-type (id : (type-app Box ((refine Int \"value > 0\")))) (flag : (cont Bool Unit {tail resume} affine))) (record (id (lit-int 1))) (jump (label exit) x))",
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
