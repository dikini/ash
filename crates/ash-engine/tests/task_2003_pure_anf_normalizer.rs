//! TASK-2003/TASK-2004/TASK-2014 RED contracts for the composed `PureAnf` fragment.
//!
//! This is intentionally narrower than general expression lowering.  It admits
//! only pure, typed literal/variable atoms, approved integer binary trees, and
//! recursive Boolean `Not`, including where their ANF spine feeds a variable
//! binding or Boolean `if`.  Existing boundary tests retain the separate
//! call/operation/handler/provider/frame closure coverage.

use ash_core::{
    BinaryOp, Expr, Value,
    cps::{Atom, ContRef, PrimOp, Term, Value as CpsValue},
};
use ash_engine::{Engine, EngineError};

fn checked_cps(engine: &Engine, source: &str) -> Term {
    let mut entry = engine.parse(source).expect("PureAnf source must parse");
    engine
        .check(&mut entry)
        .expect("PureAnf source must typecheck");
    engine
        .lower_entry_to_checked_cps(&entry)
        .expect("the typed PureAnf fragment must lower through sealed checked CPS admission")
}

fn leading_prim_ops(term: &Term) -> Vec<PrimOp> {
    let mut ops = Vec::new();
    let mut cursor = term;
    while let Term::LetPrim { op, body, .. } = cursor {
        ops.push(op.clone());
        cursor = body;
    }
    ops
}

#[test]
fn composed_not_and_integer_comparison_lower_to_one_left_to_right_pure_anf_spine() {
    let engine = Engine::new().build().expect("engine builds");
    let term = checked_cps(&engine, "fn main() -> Bool { !!(1 + 2 < 4) }");

    let Term::LetPrim {
        name: add,
        op: PrimOp::Add,
        args: add_args,
        body: comparison_body,
    } = term
    else {
        panic!("the nested integer addition must be the first PureAnf binding");
    };
    assert_eq!(add_args, vec![Atom::Int(1), Atom::Int(2)]);

    let Term::LetPrim {
        name: comparison,
        op: PrimOp::Lt,
        args: comparison_args,
        body: inner_not_body,
    } = *comparison_body
    else {
        panic!("the comparison must consume the integer temporary before either Not");
    };
    assert_eq!(comparison_args, vec![Atom::Var(add), Atom::Int(4)]);

    let Term::LetPrim {
        name: inner_not,
        op: PrimOp::Not,
        args: inner_not_args,
        body: outer_not_body,
    } = *inner_not_body
    else {
        panic!("the inner Boolean Not must follow the comparison");
    };
    assert_eq!(inner_not_args, vec![Atom::Var(comparison)]);

    let Term::LetPrim {
        name: outer_not,
        op: PrimOp::Not,
        args: outer_not_args,
        body: answer_jump,
    } = *outer_not_body
    else {
        panic!("the outer Boolean Not must follow the inner Boolean Not");
    };
    assert_eq!(outer_not_args, vec![Atom::Var(inner_not)]);
    assert!(matches!(
        *answer_jump,
        Term::Jump {
            cont: ContRef::Label(ref answer),
            arg: Atom::Var(ref result),
            ..
        } if answer == "__answer" && result == &outer_not
    ));
}

#[test]
fn computed_boolean_let_rhs_places_its_pure_anf_spine_before_the_source_letval() {
    let engine = Engine::new().build().expect("engine builds");
    let term = checked_cps(
        &engine,
        r"
        fn main() -> Bool {
            do {
                let flag = !(1 + 2 < 4);
                return flag;
            }
        }
        ",
    );

    let Term::LetPrim {
        name: add,
        op: PrimOp::Add,
        body: comparison_body,
        ..
    } = term
    else {
        panic!("the computed let RHS must start with integer addition");
    };
    let Term::LetPrim {
        name: comparison,
        op: PrimOp::Lt,
        args: comparison_args,
        body: not_body,
    } = *comparison_body
    else {
        panic!("the computed let RHS must compare before complementing");
    };
    assert_eq!(comparison_args, vec![Atom::Var(add), Atom::Int(4)]);
    let Term::LetPrim {
        name: not,
        op: PrimOp::Not,
        args: not_args,
        body: let_body,
    } = *not_body
    else {
        panic!("the computed let RHS must materialize its Boolean Not");
    };
    assert_eq!(not_args, vec![Atom::Var(comparison)]);
    let Term::LetVal {
        name: flag,
        value: CpsValue::Atom(Atom::Var(bound_result)),
        body: answer_jump,
    } = *let_body
    else {
        panic!("the source let must bind the completed PureAnf result");
    };
    assert_eq!(flag, "flag");
    assert_eq!(bound_result, not);
    assert!(matches!(
        *answer_jump,
        Term::Jump {
            cont: ContRef::Label(ref answer),
            arg: Atom::Var(ref result),
            ..
        } if answer == "__answer" && result == "flag"
    ));
}

#[test]
fn computed_boolean_if_scrutinee_places_its_pure_anf_spine_outside_the_if() {
    let engine = Engine::new().build().expect("engine builds");
    let term = checked_cps(
        &engine,
        "fn main() -> Int { if !(1 + 2 < 4) then 7 else 8 }",
    );

    let Term::LetPrim {
        name: add,
        op: PrimOp::Add,
        body: comparison_body,
        ..
    } = term
    else {
        panic!("the computed condition must begin outside the If with addition");
    };
    let Term::LetPrim {
        name: comparison,
        op: PrimOp::Lt,
        args: comparison_args,
        body: not_body,
    } = *comparison_body
    else {
        panic!("the computed condition must compare before complementing");
    };
    assert_eq!(comparison_args, vec![Atom::Var(add), Atom::Int(4)]);
    let Term::LetPrim {
        name: condition,
        op: PrimOp::Not,
        args: condition_args,
        body: if_body,
    } = *not_body
    else {
        panic!("the computed condition must bind Not before the If");
    };
    assert_eq!(condition_args, vec![Atom::Var(comparison)]);
    assert!(matches!(
        *if_body,
        Term::If {
            cond: Atom::Var(ref condition_atom),
            then_branch,
            else_branch,
            ..
        } if condition_atom == &condition
            && matches!(
                *then_branch,
                Term::Jump {
                    cont: ContRef::Label(ref answer),
                    arg: Atom::Int(7),
                    ..
                } if answer == "__answer"
            )
            && matches!(
                *else_branch,
                Term::Jump {
                    cont: ContRef::Label(ref answer),
                    arg: Atom::Int(8),
                    ..
                } if answer == "__answer"
            )
    ));
}

#[test]
fn recursive_boolean_not_is_admitted_inside_each_boolean_if_branch() {
    let engine = Engine::new().build().expect("engine builds");
    let term = checked_cps(
        &engine,
        "fn main() -> Bool { if true then !!(1 + 2 < 4) else !!false }",
    );

    let Term::If {
        cond: Atom::Bool(true),
        then_branch,
        else_branch,
        ..
    } = term
    else {
        panic!("the literal condition may remain directly on the If");
    };
    assert_eq!(
        leading_prim_ops(&then_branch),
        vec![PrimOp::Add, PrimOp::Lt, PrimOp::Not, PrimOp::Not],
        "the true branch must carry its complete composed PureAnf spine"
    );
    assert_eq!(
        leading_prim_ops(&else_branch),
        vec![PrimOp::Not, PrimOp::Not],
        "the false branch must retain recursive Boolean Not"
    );
}

#[tokio::test]
async fn engine_run_executes_the_composed_pure_anf_fragment_only_through_checked_cps_admission() {
    let engine = Engine::new().build().expect("engine builds");
    for (source, expected) in [
        ("fn main() -> Bool { !!(1 + 2 < 4) }", Value::Bool(true)),
        (
            "fn main() -> Bool { do { let flag = !(1 + 2 < 4); return flag; } }",
            Value::Bool(false),
        ),
        (
            "fn main() -> Int { if !(1 + 2 < 4) then 7 else 8 }",
            Value::Int(8),
        ),
        (
            "fn main() -> Bool { if true then !!(1 + 2 < 4) else !!false }",
            Value::Bool(true),
        ),
    ] {
        assert_eq!(
            engine
                .run(source)
                .await
                .expect("the composed PureAnf source must execute through sealed admission"),
            expected
        );
    }
}

#[test]
fn typed_boolean_equality_and_inequality_lower_to_exact_cps_primitives_and_the_answer_jump() {
    // TASK-2003 lines 106-140 define the typed PureAnf/answer-Jump shape and
    // identify Boolean equality as the next fail-closed boundary. This test
    // narrows only that boundary; the adjacent negative controls retain every
    // other Boolean/binary and effectful form as closed.
    let engine = Engine::new().build().expect("engine builds");

    for (name, source, core_operation, cps_operation) in [
        (
            "equality",
            "fn main() -> Bool { true == false }",
            BinaryOp::Eq,
            PrimOp::Eq,
        ),
        (
            "inequality",
            "fn main() -> Bool { true != false }",
            BinaryOp::Ne,
            PrimOp::Ne,
        ),
    ] {
        let mut entry = engine
            .parse(source)
            .unwrap_or_else(|error| panic!("{name} source must parse: {error}"));
        engine
            .check(&mut entry)
            .unwrap_or_else(|error| panic!("{name} source must typecheck: {error}"));
        assert!(
            matches!(
                &entry.core,
                Expr::Binary { op, left, right }
                    if *op == core_operation
                        && matches!(left.as_ref(), Expr::Literal(Value::Bool(true)))
                        && matches!(right.as_ref(), Expr::Literal(Value::Bool(false)))
            ),
            "{name} source must retain its exact typed Boolean equality Core operator"
        );

        let lowered = engine
            .lower_entry_to_checked_cps(&entry)
            .unwrap_or_else(|error| {
                panic!(
                    "{name} must lower through the bounded checked Core/CPS inspection bridge: {error}"
                )
            });
        let Term::LetPrim {
            name: result,
            op,
            args,
            body,
        } = lowered
        else {
            panic!("{name} must lower to one Boolean equality LetPrim");
        };
        assert_eq!(
            op, cps_operation,
            "{name} must preserve its exact Core equality operation in CPS"
        );
        assert_eq!(
            args,
            vec![Atom::Bool(true), Atom::Bool(false)],
            "{name} must retain exactly its two typed Boolean atoms"
        );
        assert!(
            matches!(
                *body,
                Term::Jump {
                    cont: ContRef::Label(ref answer),
                    arg: Atom::Var(ref jump_result),
                    ..
                } if answer == "__answer" && jump_result == &result
            ),
            "{name} must jump only its Boolean equality result to the sealed answer continuation"
        );
    }
}

#[test]
fn composed_pure_anf_does_not_admit_calls_or_unapproved_boolean_operators() {
    let engine = Engine::new().build().expect("engine builds");
    for (name, source) in [
        ("Boolean And", "fn main() -> Bool { true && false }"),
        ("Boolean Or", "fn main() -> Bool { true || false }"),
        (
            "local call",
            "fn helper() -> Bool { true } fn main() -> Bool { helper() }",
        ),
        (
            "unary Neg",
            "fn main() -> Int { do { let value = 1; return - value; } }",
        ),
    ] {
        let mut entry = engine.parse(source).unwrap_or_else(|error| {
            panic!("{name} source must remain syntactically resolvable: {error}")
        });
        if engine.check(&mut entry).is_ok() {
            let error = engine
                .lower_entry_to_checked_cps(&entry)
                .expect_err("non-PureAnf form must remain closed at checked CPS admission");
            assert!(matches!(error, EngineError::Type(_)), "{name}: {error}");
        }
    }
}
