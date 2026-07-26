//! TASK-2003/TASK-2004/TASK-2014 RED contracts for nested pure binary ANF lowering.
//!
//! This slice extends only the existing handler-free binary primitive family.
//! Its temporary values must remain internal to sealed checked Core/CPS
//! admission; calls, effects, frames, and unary expressions stay closed.

use ash_core::{
    Value,
    cps::{Atom, ContRef, PrimOp, Term, Value as CpsValue},
};
use ash_engine::{Engine, EngineError};

const HANDLED_OPERATION: &str = r"
interface Clock<T> { sleep(Int) -> Int }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = milliseconds }
handler resume_sleep(comp: () -> { TestClock::sleep } Int) -> Int {
    on comp {
        TestClock::sleep(milliseconds, resume) => resume(milliseconds),
        done(value) => value,
    }
}
fn main() -> Int { handle TestClock::sleep(0) with resume_sleep }
";

fn checked_cps(engine: &Engine, source: &str) -> Term {
    let mut entry = engine
        .parse(source)
        .expect("nested primitive source must parse");
    engine
        .check(&mut entry)
        .expect("nested primitive source must typecheck");
    engine
        .lower_entry_to_checked_cps(&entry)
        .expect("nested pure binary primitives must lower through checked CPS")
}

const COMPUTED_LET_SOURCE: &str = r"
fn main() -> Int {
    do {
        let __checked_add_result = 99;
        let computed = (1 + 2) * 3;
        return computed + 4;
    }
}
";

#[test]
fn nested_binary_primitives_lower_to_a_fresh_left_to_right_anf_letprim_spine() {
    let engine = Engine::new().build().expect("engine builds");
    let term = checked_cps(&engine, "fn main() -> Int { (1 + 2) + 3 }");

    let Term::LetPrim {
        name: inner_result,
        op: inner_op,
        args: inner_args,
        body: outer_body,
    } = term
    else {
        panic!("the inner binary expression must bind a CPS temporary");
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
        panic!("the outer binary expression must consume the inner temporary");
    };
    assert_ne!(
        inner_result, outer_result,
        "each nested primitive result must have a fresh ANF temporary"
    );
    assert_eq!(outer_op, PrimOp::Add);
    assert_eq!(
        outer_args,
        vec![Atom::Var(inner_result), Atom::Int(3)],
        "the outer primitive must consume the inner result left-to-right"
    );
    assert!(
        matches!(
            *answer_jump,
            Term::Jump {
                cont: ContRef::Label(ref answer),
                arg: Atom::Var(ref result),
                ..
            } if answer == "__answer" && result == &outer_result
        ),
        "only the final nested primitive result may jump to the sealed answer continuation"
    );
}

#[test]
fn mixed_nested_integer_and_boolean_binary_primitives_preserve_their_anf_spine() {
    let engine = Engine::new().build().expect("engine builds");
    let term = checked_cps(&engine, "fn main() -> Bool { (1 + 2) >= (2 * 3) }");

    let Term::LetPrim {
        name: add_result,
        op: add_op,
        args: add_args,
        body: multiply_body,
    } = term
    else {
        panic!("the left nested integer expression must bind first");
    };
    assert_eq!(add_op, PrimOp::Add);
    assert_eq!(add_args, vec![Atom::Int(1), Atom::Int(2)]);

    let Term::LetPrim {
        name: multiply_result,
        op: multiply_op,
        args: multiply_args,
        body: comparison_body,
    } = *multiply_body
    else {
        panic!("the right nested integer expression must bind second");
    };
    assert_ne!(add_result, multiply_result);
    assert_eq!(multiply_op, PrimOp::Mul);
    assert_eq!(multiply_args, vec![Atom::Int(2), Atom::Int(3)]);

    let Term::LetPrim {
        name: comparison_result,
        op: comparison_op,
        args: comparison_args,
        body: answer_jump,
    } = *comparison_body
    else {
        panic!("the Boolean comparison must consume both nested primitive results");
    };
    assert_ne!(comparison_result, add_result);
    assert_ne!(comparison_result, multiply_result);
    assert_eq!(comparison_op, PrimOp::Ge);
    assert_eq!(
        comparison_args,
        vec![Atom::Var(add_result), Atom::Var(multiply_result)]
    );
    assert!(matches!(
        *answer_jump,
        Term::Jump {
            cont: ContRef::Label(ref answer),
            arg: Atom::Var(ref result),
            ..
        } if answer == "__answer" && result == &comparison_result
    ));
}

#[test]
fn computed_variable_let_rhs_preserves_the_nested_anf_spine_before_its_admitted_body() {
    let engine = Engine::new().build().expect("engine builds");
    let term = checked_cps(&engine, COMPUTED_LET_SOURCE);

    let Term::LetVal {
        name: source_collision,
        value: CpsValue::Atom(Atom::Int(99)),
        body: rhs_body,
    } = term
    else {
        panic!("the existing source binding must remain outside the computed let RHS");
    };
    assert_eq!(source_collision, "__checked_add_result");

    let Term::LetPrim {
        name: rhs_add_result,
        op: rhs_add_op,
        args: rhs_add_args,
        body: rhs_multiply_body,
    } = *rhs_body
    else {
        panic!("the computed RHS must lower its left nested primitive first");
    };
    assert_eq!(rhs_add_op, PrimOp::Add);
    assert_eq!(rhs_add_args, vec![Atom::Int(1), Atom::Int(2)]);
    assert_ne!(rhs_add_result, source_collision);

    let Term::LetPrim {
        name: rhs_multiply_result,
        op: rhs_multiply_op,
        args: rhs_multiply_args,
        body: computed_let_body,
    } = *rhs_multiply_body
    else {
        panic!("the computed RHS must consume its nested result before binding the source name");
    };
    assert_eq!(rhs_multiply_op, PrimOp::Mul);
    assert_eq!(
        rhs_multiply_args,
        vec![Atom::Var(rhs_add_result.clone()), Atom::Int(3)]
    );
    assert_ne!(rhs_multiply_result, source_collision);
    assert_ne!(rhs_multiply_result, rhs_add_result);

    let Term::LetVal {
        name: computed,
        value: CpsValue::Atom(Atom::Var(bound_rhs_result)),
        body: source_body,
    } = *computed_let_body
    else {
        panic!("the source variable let must bind the final typed RHS temporary");
    };
    assert_eq!(computed, "computed");
    assert_eq!(bound_rhs_result, rhs_multiply_result);

    let Term::LetPrim {
        name: body_add_result,
        op: body_add_op,
        args: body_add_args,
        body: answer_jump,
    } = *source_body
    else {
        panic!("the already-admitted pure body must lower after its source let binding");
    };
    assert_eq!(body_add_op, PrimOp::Add);
    assert_eq!(
        body_add_args,
        vec![Atom::Var("computed".to_string()), Atom::Int(4)]
    );
    assert_ne!(body_add_result, source_collision);
    assert_ne!(body_add_result, rhs_add_result);
    assert_ne!(body_add_result, rhs_multiply_result);
    assert!(matches!(
        *answer_jump,
        Term::Jump {
            cont: ContRef::Label(ref answer),
            arg: Atom::Var(ref result),
            ..
        } if answer == "__answer" && result == &body_add_result
    ));
}

#[tokio::test]
async fn engine_run_executes_nested_pure_binary_primitives_only_through_sealed_checked_cps() {
    let engine = Engine::new().build().expect("engine builds");

    for (source, expected) in [
        ("fn main() -> Int { (1 + 2) + 3 }", Value::Int(6)),
        (
            "fn main() -> Bool { (1 + 2) >= (2 * 3) }",
            Value::Bool(false),
        ),
    ] {
        let value = engine
            .run(source)
            .await
            .expect("nested pure binary source must execute through sealed checked CPS admission");
        assert_eq!(value, expected);
    }
}

#[tokio::test]
async fn engine_run_file_executes_a_nested_pure_binary_primitive_through_sealed_checked_cps() {
    let directory = tempfile::tempdir().expect("temporary source directory creates");
    let path = directory.path().join("nested.ash");
    std::fs::write(&path, "fn main() -> Bool { (1 + 2) >= (2 * 3) }")
        .expect("nested primitive source file writes");
    let engine = Engine::new().build().expect("engine builds");

    let value = engine
        .run_file(&path)
        .await
        .expect("nested pure binary file entry must execute through sealed checked CPS admission");

    assert_eq!(value, Value::Bool(false));
}

#[tokio::test]
async fn engine_run_and_run_file_execute_a_computed_nested_binary_variable_let_through_sealed_checked_cps()
 {
    let engine = Engine::new().build().expect("engine builds");

    let direct = engine
        .run(COMPUTED_LET_SOURCE)
        .await
        .expect("computed variable let must execute through sealed checked CPS admission");
    assert_eq!(direct, Value::Int(13));

    let directory = tempfile::tempdir().expect("temporary source directory creates");
    let path = directory.path().join("computed-let.ash");
    std::fs::write(&path, COMPUTED_LET_SOURCE).expect("computed let source file writes");
    let file = engine
        .run_file(&path)
        .await
        .expect("computed variable let file must execute through sealed checked CPS admission");
    assert_eq!(file, Value::Int(13));
}

#[tokio::test]
async fn nested_binary_anf_lowering_keeps_calls_raises_and_handler_or_provider_frames_closed() {
    let engine = Engine::new().build().expect("engine builds");

    let mut call = engine
        .parse("fn helper() -> Int { 7 } fn main() -> Int { helper() }")
        .expect("local call source must parse");
    engine
        .check(&mut call)
        .expect("local call source must typecheck");
    let call_error = engine
        .lower_entry_to_checked_cps(&call)
        .expect_err("binary ANF must not lower calls");
    assert!(matches!(call_error, EngineError::Type(_)));

    for (name, source) in [
        (
            "direct provider raise",
            "fn main() -> Null { time::sleep(0) }",
        ),
        ("handled provider raise", HANDLED_OPERATION),
    ] {
        let error = engine.run(source).await.expect_err(
            "binary ANF must not turn an operation row into a provider or handler frame",
        );
        assert!(
            error.to_string().contains("checked Core/CPS admission"),
            "{name} must reject at sealed admission rather than install a frame: {error}"
        );
    }
}
