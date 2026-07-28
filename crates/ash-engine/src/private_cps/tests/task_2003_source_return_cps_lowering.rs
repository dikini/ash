//! TASK-2003: source `return` must enter CPS through a continuation.
//!
//! This is deliberately a source-to-checked-CPS contract, rather than a
//! hand-assembled CPS fixture: `Term::Return` is permitted only as the final
//! observation after evaluation, never as the term emitted for a source
//! `return` statement.

use super::super::{CpsTerminalOutcome, eval_checked_terminal};
use crate::{Engine, EngineError};
use ash_core::cps::{Atom, ContRef, Env, HandlerChain, PrimOp, Term, Value as CpsValue};

#[test]
fn source_do_return_lowers_to_the_answer_continuation_not_cps_return() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(
            r"
            fn main() -> Int {
                do {
                    return 42;
                }
            }
            ",
        )
        .expect("source return should parse");
    engine
        .check(&mut entry)
        .expect("source return should typecheck before CPS lowering");

    let term = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("checked source lowering should materialize a CPS term");

    assert!(
        matches!(
            term,
            Term::Jump {
                cont: ContRef::Label(ref name),
                arg: Atom::Int(42),
                ..
            } if name == "__answer"
        ),
        "the source return site must jump to the answer continuation, not emit Term::Return"
    );
}

#[test]
fn source_do_let_return_lowers_to_the_answer_continuation_not_cps_return() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(
            r"
            fn main() -> Int {
                do {
                    let x = 41;
                    return x;
                }
            }
            ",
        )
        .expect("source return should parse");
    engine
        .check(&mut entry)
        .expect("source return should typecheck before CPS lowering");

    let term = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("checked source lowering should materialize a CPS term");

    assert!(
        matches!(
            term,
            Term::LetVal {
                ref name,
                body,
                ..
            } if matches!(
                *body,
                Term::Jump {
                    cont: ContRef::Label(ref answer),
                    arg: Atom::Var(ref argument),
                    ..
                } if answer == "__answer" && argument == name
            )
        ),
        "the source return site must jump to the answer continuation after binding"
    );
}

#[test]
fn source_if_return_lowers_each_branch_to_the_answer_continuation() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(
            r"
            fn main() -> Int { if true then 42 else 0 }
            ",
        )
        .expect("source conditional return should parse");
    engine
        .check(&mut entry)
        .expect("source conditional return should typecheck before CPS lowering");

    let term = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("checked source lowering should materialize a CPS term");

    assert!(
        matches!(
            term,
            Term::If {
                cond: Atom::Bool(true),
                then_branch,
                else_branch,
                ..
            } if matches!(
                *then_branch,
                Term::Jump {
                    cont: ContRef::Label(ref answer),
                    arg: Atom::Int(42),
                    ..
                } if answer == "__answer"
            ) && matches!(
                *else_branch,
                Term::Jump {
                    cont: ContRef::Label(ref answer),
                    arg: Atom::Int(0),
                    ..
                } if answer == "__answer"
            )
        ),
        "both source conditional return branches must jump to the answer continuation"
    );
}

#[test]
fn source_result_annotation_constrains_the_answer_continuation_input() {
    let engine = Engine::new().build().expect("engine builds");
    let entry = engine
        .parse(r#"fn main() -> Int { "not an integer" }"#)
        .expect("source should parse before checked CPS inspection");

    let error = engine
        .lower_entry_to_checked_cps(&entry)
        .expect_err("the answer continuation must reject a non-Int source result");

    assert!(
        matches!(error, EngineError::Type(ref message) if message.contains("type mismatch")),
        "the checked answer continuation should enforce the declared source result: {error}"
    );
}

#[test]
fn source_do_variable_let_return_preserves_answer_type_and_evaluates_through_answer_continuation() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(
            r"
            fn main() -> Int {
                do {
                    let x = 41;
                    let y = x;
                    return y;
                }
            }
            ",
        )
        .expect("source return should parse");
    engine
        .check(&mut entry)
        .expect("source return should typecheck before CPS lowering");

    let body = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("typed variable lets should lower through the checked source bridge");

    assert!(
        matches!(
            &body,
            Term::LetVal { name, body, .. }
                if name == "x" && matches!(
                    body.as_ref(),
                    Term::LetVal { name, body, .. }
                        if name == "y" && matches!(
                            body.as_ref(),
                            Term::Jump {
                                cont: ContRef::Label(answer),
                                arg: Atom::Var(argument),
                                ..
                            } if answer == "__answer" && argument == "y"
                        )
                )
        ),
        "source return must retain typed let bindings and jump to __answer, never emit Term::Return"
    );

    let term = Term::LetCont {
        name: "__answer".to_string(),
        param: "__answer_value".to_string(),
        cont_body: Box::new(Term::Return {
            value: CpsValue::Atom(Atom::Var("__answer_value".to_string())),
        }),
        body: Box::new(body),
        row: ash_core::cps::EffectRow::default(),
        multiplicity: ash_core::cps::ContMultiplicity::default(),
    };
    assert_eq!(
        eval_checked_terminal(&term, &Env::new(), &HandlerChain::new()),
        Ok(CpsTerminalOutcome::Return(CpsValue::Atom(Atom::Int(41)))),
        "the answer continuation, rather than a source-emitted Return term, owns the observation"
    );
}

#[test]
fn source_do_variable_let_return_rejects_an_incompatible_answer_type() {
    let engine = Engine::new().build().expect("engine builds");
    let entry = engine
        .parse(
            r"
            fn main() -> String {
                do {
                    let x = 41;
                    let y = x;
                    return y;
                }
            }
            ",
        )
        .expect("source should parse before checked CPS inspection");

    let error = engine
        .lower_entry_to_checked_cps(&entry)
        .expect_err("the answer continuation must reject an Int variable for a String result");
    assert!(
        matches!(error, EngineError::Type(ref message) if message.contains("type mismatch")),
        "the checked answer continuation should enforce the declared source result: {error}"
    );
}

#[test]
fn source_integer_addition_lowers_to_letprim_then_the_answer_continuation() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse("fn main() -> Int { 2 + 5 }")
        .expect("integer addition source should parse");
    engine
        .check(&mut entry)
        .expect("integer addition source should typecheck before CPS lowering");

    let term = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("checked source lowering should materialize integer addition");

    assert!(
        matches!(
            term,
            Term::LetPrim {
                op: PrimOp::Add,
                args,
                body,
                ..
            } if args == vec![Atom::Int(2), Atom::Int(5)] && matches!(
                *body,
                Term::Jump {
                    cont: ContRef::Label(ref answer),
                    arg: Atom::Var(_),
                    ..
                } if answer == "__answer"
            )
        ),
        "source integer addition must bind a LetPrim Add result and jump to __answer"
    );
}

#[test]
fn source_lexical_integer_addition_lowers_atoms_to_letprim_then_the_answer_continuation() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(
            r"
            fn main() -> Int {
                do {
                    let x = 2;
                    let y = 5;
                    return x + y;
                }
            }
            ",
        )
        .expect("lexical integer addition source should parse");
    engine
        .check(&mut entry)
        .expect("lexical integer addition source should typecheck before CPS lowering");

    let term = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("checked source lowering should materialize lexical integer addition");

    let Term::LetVal {
        name: x,
        body: x_body,
        ..
    } = term
    else {
        panic!("lexical addition must preserve its first let binding");
    };
    assert_eq!(x, "x");
    let Term::LetVal {
        name: y,
        body: y_body,
        ..
    } = *x_body
    else {
        panic!("lexical addition must preserve its second let binding");
    };
    assert_eq!(y, "y");
    let Term::LetPrim {
        op: PrimOp::Add,
        args,
        body,
        ..
    } = *y_body
    else {
        panic!("lexical addition must lower to LetPrim Add");
    };
    assert_eq!(
        args,
        vec![Atom::Var("x".to_string()), Atom::Var("y".to_string())]
    );
    assert!(
        matches!(
            *body,
            Term::Jump {
                cont: ContRef::Label(ref answer),
                arg: Atom::Var(_),
                ..
            } if answer == "__answer"
        ),
        "lexically bound source addition must jump to __answer after LetPrim"
    );
}

#[test]
fn source_boolean_not_lowers_to_letprim_then_the_answer_continuation_and_evaluates() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse("fn main() -> Bool { !true }")
        .expect("boolean not source should parse");
    engine
        .check(&mut entry)
        .expect("boolean not source should typecheck before CPS lowering");

    let body = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("checked source lowering should materialize atomic boolean not");

    let Term::LetPrim {
        name,
        op,
        args,
        body: not_body,
    } = &body
    else {
        panic!("source boolean not must lower to LetPrim Not");
    };
    assert_eq!(*op, PrimOp::Not);
    assert_eq!(args, &vec![Atom::Bool(true)]);
    assert!(
        matches!(
            not_body.as_ref(),
            Term::Jump {
                cont: ContRef::Label(answer),
                arg: Atom::Var(result),
                ..
            } if answer == "__answer" && result == name
        ),
        "source boolean not must jump to __answer with its LetPrim result, never emit Term::Return"
    );
    assert!(
        !matches!(body, Term::Return { .. }),
        "the source bridge must not emit a source Term::Return for boolean not"
    );

    let term = Term::LetCont {
        name: "__answer".to_string(),
        param: "__answer_value".to_string(),
        cont_body: Box::new(Term::Return {
            value: CpsValue::Atom(Atom::Var("__answer_value".to_string())),
        }),
        body: Box::new(body),
        row: ash_core::cps::EffectRow::default(),
        multiplicity: ash_core::cps::ContMultiplicity::default(),
    };
    assert_eq!(
        eval_checked_terminal(&term, &Env::new(), &HandlerChain::new()),
        Ok(CpsTerminalOutcome::Return(CpsValue::Atom(Atom::Bool(
            false
        )))),
        "the answer continuation must observe the boolean Not result"
    );
}

#[test]
fn source_lexical_boolean_not_preserves_letval_then_letprim_spine() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(
            r"
            fn main() -> Bool {
                do {
                    let flag = true;
                    return !flag;
                }
            }
            ",
        )
        .expect("lexical boolean not source should parse");
    engine
        .check(&mut entry)
        .expect("lexical boolean not source should typecheck before CPS lowering");

    let term = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("checked source lowering should materialize lexical boolean not");

    let Term::LetVal {
        name: flag,
        body: flag_body,
        ..
    } = term
    else {
        panic!("lexical boolean not must preserve its LetVal binding");
    };
    assert_eq!(flag, "flag");
    let Term::LetPrim {
        name: result,
        op,
        args,
        body,
    } = *flag_body
    else {
        panic!("lexical boolean not must lower its final result to LetPrim Not");
    };
    assert_eq!(op, PrimOp::Not);
    assert_eq!(args, vec![Atom::Var("flag".to_string())]);
    assert!(
        matches!(
            *body,
            Term::Jump {
                cont: ContRef::Label(ref answer),
                arg: Atom::Var(ref argument),
                ..
            } if answer == "__answer" && argument == &result
        ),
        "lexical boolean not must jump to __answer after its LetPrim result"
    );
}

#[test]
fn source_nested_boolean_not_lowers_to_an_ordered_letprim_not_spine_then_the_answer_continuation() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse("fn main() -> Bool { !!true }")
        .expect("nested boolean not source should parse");
    engine
        .check(&mut entry)
        .expect("nested boolean not source should typecheck before CPS inspection");

    let term = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("nested Boolean Not must lower through the checked CPS ANF boundary");

    let Term::LetPrim {
        name: inner_result,
        op: inner_op,
        args: inner_args,
        body: inner_body,
    } = term
    else {
        panic!("the inner Boolean Not must materialize the first LetPrim");
    };
    assert!(
        inner_op == PrimOp::Not && inner_args == vec![Atom::Bool(true)],
        "the first LetPrim must complement the literal Boolean atom"
    );
    let Term::LetPrim {
        name: outer_result,
        op: outer_op,
        args: outer_args,
        body: outer_body,
    } = *inner_body
    else {
        panic!("the outer Boolean Not must materialize the second LetPrim");
    };
    assert_eq!(outer_op, PrimOp::Not);
    assert_eq!(outer_args, vec![Atom::Var(inner_result)]);
    assert!(
        matches!(
            *outer_body,
            Term::Jump {
                cont: ContRef::Label(ref answer),
                arg: Atom::Var(ref result),
                ..
            } if answer == "__answer" && result == &outer_result
        ),
        "the outer Boolean Not result must be the sole answer-continuation argument"
    );
}

#[test]
fn source_lexical_nested_boolean_not_preserves_letval_then_an_ordered_letprim_not_spine() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(
            r"
            fn main() -> Bool {
                do {
                    let flag = true;
                    return !!flag;
                }
            }
            ",
        )
        .expect("lexical nested Boolean Not source should parse");
    engine
        .check(&mut entry)
        .expect("lexical nested Boolean Not source should typecheck before CPS lowering");

    let term = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("lexical nested Boolean Not must lower through the checked CPS ANF boundary");

    let Term::LetVal {
        name: flag,
        body: flag_body,
        ..
    } = term
    else {
        panic!("the lexical Boolean binding must remain a LetVal");
    };
    assert_eq!(flag, "flag");
    let Term::LetPrim {
        name: inner_result,
        op: inner_op,
        args: inner_args,
        body: inner_body,
    } = *flag_body
    else {
        panic!("the lexical inner Boolean Not must be the first LetPrim");
    };
    assert_eq!(inner_op, PrimOp::Not);
    assert_eq!(inner_args, vec![Atom::Var("flag".to_string())]);
    let Term::LetPrim {
        name: outer_result,
        op: outer_op,
        args: outer_args,
        body: outer_body,
    } = *inner_body
    else {
        panic!("the lexical outer Boolean Not must be the second LetPrim");
    };
    assert_eq!(outer_op, PrimOp::Not);
    assert_eq!(outer_args, vec![Atom::Var(inner_result)]);
    assert!(
        matches!(
            *outer_body,
            Term::Jump {
                cont: ContRef::Label(ref answer),
                arg: Atom::Var(ref result),
                ..
            } if answer == "__answer" && result == &outer_result
        ),
        "the lexical outer Boolean Not result must jump to the answer continuation"
    );
}

#[tokio::test]
async fn source_nested_boolean_not_executes_through_sealed_checked_cps_for_run_and_run_file() {
    let engine = Engine::new().build().expect("engine builds");
    let source = "fn main() -> Bool { !!true }";

    assert_eq!(
        engine
            .run(source)
            .await
            .expect("nested Boolean Not must execute through sealed checked CPS admission"),
        ash_core::Value::Bool(true),
        "run must observe the nested Boolean Not terminal value"
    );

    let directory = tempfile::tempdir().expect("temporary source directory creates");
    let path = directory.path().join("nested_boolean_not.ash");
    std::fs::write(&path, source).expect("nested Boolean Not source file writes");
    assert_eq!(
        engine
            .run_file(&path)
            .await
            .expect("nested Boolean Not file must execute through sealed checked CPS admission"),
        ash_core::Value::Bool(true),
        "run_file must observe the nested Boolean Not terminal value"
    );
}

#[test]
fn source_nested_boolean_not_in_a_boolean_if_branch_has_its_complete_anf_spine() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse("fn main() -> Bool { if true then !!true else false }")
        .expect("nested Boolean Not branch source parses");
    engine
        .check(&mut entry)
        .expect("nested Boolean Not branch source typechecks");
    let term = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("nested Boolean Not branch must lower through checked CPS admission");

    assert!(matches!(
        term,
        Term::If {
            then_branch,
            else_branch,
            ..
        } if matches!(
            then_branch.as_ref(),
            Term::LetPrim {
                op: PrimOp::Not,
                body,
                ..
            } if matches!(
                body.as_ref(),
                Term::LetPrim {
                    op: PrimOp::Not,
                    body,
                    ..
                } if matches!(body.as_ref(), Term::Jump { .. })
            )
        ) && matches!(*else_branch, Term::Jump { .. })
    ));
}

#[test]
fn source_nested_boolean_not_as_a_computed_let_rhs_precedes_its_source_binding() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(
            r"
        fn main() -> Bool {
            do {
                let flag = !!true;
                return flag;
            }
        }
        ",
        )
        .expect("nested Boolean Not let RHS source parses");
    engine
        .check(&mut entry)
        .expect("nested Boolean Not let RHS source typechecks");
    let term = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("nested Boolean Not let RHS must lower through checked CPS admission");

    assert!(matches!(
        term,
        Term::LetPrim {
            op: PrimOp::Not,
            body,
            ..
        } if matches!(
            body.as_ref(),
            Term::LetPrim {
                op: PrimOp::Not,
                body,
                ..
            } if matches!(
                body.as_ref(),
                Term::LetVal { name, .. } if name == "flag"
            )
        )
    ));
}

#[test]
fn source_nested_boolean_not_as_a_binary_operand_retains_its_complete_anf_spine() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse("fn main() -> Bool { !!true == true }")
        .expect("nested Boolean Not binary operand source parses");
    engine
        .check(&mut entry)
        .expect("nested Boolean Not binary operand source typechecks");
    let term = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("recursive Boolean Not is an admitted pure ANF operand");

    assert!(matches!(
        term,
        Term::LetPrim {
            op: PrimOp::Not,
            body,
            ..
        } if matches!(
            body.as_ref(),
            Term::LetPrim {
                op: PrimOp::Not,
                body,
                ..
            } if matches!(
                body.as_ref(),
                Term::LetPrim {
                    op: PrimOp::Eq,
                    args,
                    body,
                    ..
                } if matches!(args.as_slice(), [Atom::Var(_), Atom::Bool(true)])
                    && matches!(body.as_ref(), Term::Jump { .. })
            )
        )
    ));
}

#[test]
fn source_nonboolean_not_rejects_at_typecheck_or_checked_cps_boundary() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse("fn main() -> Bool { !1 }")
        .expect("nonboolean not source should parse");

    if engine.check(&mut entry).is_ok() {
        let error = engine
            .lower_entry_to_checked_cps(&entry)
            .expect_err("nonboolean not must not lower when source checking accepts it");
        assert!(
            matches!(error, EngineError::Type(_)),
            "nonboolean Not must reject through a checked source/CPS boundary: {error}"
        );
    }
}

#[test]
fn source_unary_neg_remains_fail_closed_at_the_checked_cps_boundary() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(
            r"
            fn main() -> Int {
                do {
                    let value = 1;
                    return - value;
                }
            }
            ",
        )
        .expect("unary neg source should parse");
    engine
        .check(&mut entry)
        .expect("unary neg source should typecheck before CPS inspection");

    let error = engine
        .lower_entry_to_checked_cps(&entry)
        .expect_err("unary Neg must remain outside the bounded checked CPS lowering subset");
    assert!(
        matches!(error, EngineError::Type(_)),
        "the private bridge must reject unary Neg rather than silently lowering it: {error}"
    );
}

#[test]
fn source_literal_subtraction_lowers_to_letprim_then_the_answer_continuation() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse("fn main() -> Int { 1 - 0 }")
        .expect("integer subtraction source should parse");
    engine
        .check(&mut entry)
        .expect("integer subtraction source should typecheck before checked CPS inspection");

    let term = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("public checked CPS lowering must materialize literal subtraction");
    assert!(
        matches!(
            term,
            Term::LetPrim {
                op: PrimOp::Sub,
                args,
                body,
                ..
            } if args == vec![Atom::Int(1), Atom::Int(0)] && matches!(
                *body,
                Term::Jump {
                    cont: ContRef::Label(ref answer),
                    arg: Atom::Var(_),
                    ..
                } if answer == "__answer"
            )
        ),
        "public literal subtraction must bind LetPrim(Sub) and jump to __answer"
    );
}

#[test]
fn source_nested_integer_addition_lowers_to_a_fresh_anf_letprim_spine_and_evaluates() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse("fn main() -> Int { (1 + 2) + 3 }")
        .expect("nested integer addition source should parse");
    engine
        .check(&mut entry)
        .expect("nested integer addition source should typecheck before CPS inspection");

    let body = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("nested pure addition must lower through the checked binary ANF bridge");

    let Term::LetPrim {
        name: inner_result,
        op: inner_op,
        args: inner_args,
        body: outer_body,
    } = body
    else {
        panic!("the inner addition must bind the first ANF temporary");
    };
    assert_eq!(inner_op, PrimOp::Add);
    assert_eq!(inner_args, vec![Atom::Int(1), Atom::Int(2)]);

    let Term::LetPrim {
        name: outer_result,
        op: outer_op,
        args: outer_args,
        body: answer_jump,
    } = *outer_body
    else {
        panic!("the outer addition must consume the inner ANF temporary");
    };
    assert_ne!(inner_result, outer_result);
    assert_eq!(outer_op, PrimOp::Add);
    assert!(
        outer_args == vec![Atom::Var(inner_result.clone()), Atom::Int(3)],
        "the outer addition must consume the inner result left-to-right"
    );
    assert!(matches!(
        &*answer_jump,
        Term::Jump {
            cont: ContRef::Label(answer),
            arg: Atom::Var(result),
            ..
        } if answer == "__answer" && result == &outer_result
    ));

    let term = Term::LetCont {
        name: "__answer".to_string(),
        param: "__answer_value".to_string(),
        cont_body: Box::new(Term::Return {
            value: CpsValue::Atom(Atom::Var("__answer_value".to_string())),
        }),
        body: Box::new(Term::LetPrim {
            name: inner_result,
            op: PrimOp::Add,
            args: vec![Atom::Int(1), Atom::Int(2)],
            body: Box::new(Term::LetPrim {
                name: outer_result,
                op: PrimOp::Add,
                args: outer_args,
                body: answer_jump,
            }),
        }),
        row: ash_core::cps::EffectRow::default(),
        multiplicity: ash_core::cps::ContMultiplicity::default(),
    };
    assert_eq!(
        eval_checked_terminal(&term, &Env::new(), &HandlerChain::new()),
        Ok(CpsTerminalOutcome::Return(CpsValue::Atom(Atom::Int(6)))),
        "the sealed answer continuation must observe the nested ANF result"
    );
}
