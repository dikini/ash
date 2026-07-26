//! TASK-2004/TASK-2014 RED contracts for the sealed atom-only binary primitive family.
//!
//! The positive cases are pure and handler-free. They must lower as one
//! `LetPrim` followed immediately by the sealed answer jump and execute only
//! through the selected checked Core/CPS admission boundary.

use ash_core::{
    Value,
    cps::{Atom, ContRef, PrimOp, Term},
};
use ash_engine::{Engine, ProductionExecutionBoundary};

#[derive(Clone, Copy)]
enum ExpectedValue {
    Int(i64),
    Bool(bool),
}

impl ExpectedValue {
    const fn engine_value(self) -> Value {
        match self {
            Self::Int(value) => Value::Int(value),
            Self::Bool(value) => Value::Bool(value),
        }
    }
}

struct PrimitiveCase {
    name: &'static str,
    source: &'static str,
    operation: PrimOp,
    arguments: Vec<Atom>,
    expected: ExpectedValue,
}

fn primitive_cases() -> Vec<PrimitiveCase> {
    vec![
        PrimitiveCase {
            name: "add",
            source: "fn main() -> Int { 7 + 2 }",
            operation: PrimOp::Add,
            arguments: vec![Atom::Int(7), Atom::Int(2)],
            expected: ExpectedValue::Int(9),
        },
        PrimitiveCase {
            name: "sub",
            source: "fn main() -> Int { 7 - 2 }",
            operation: PrimOp::Sub,
            arguments: vec![Atom::Int(7), Atom::Int(2)],
            expected: ExpectedValue::Int(5),
        },
        PrimitiveCase {
            name: "mul",
            source: "fn main() -> Int { 7 * 2 }",
            operation: PrimOp::Mul,
            arguments: vec![Atom::Int(7), Atom::Int(2)],
            expected: ExpectedValue::Int(14),
        },
        PrimitiveCase {
            name: "div",
            source: "fn main() -> Int { 8 / 2 }",
            operation: PrimOp::Div,
            arguments: vec![Atom::Int(8), Atom::Int(2)],
            expected: ExpectedValue::Int(4),
        },
        PrimitiveCase {
            name: "eq",
            source: "fn main() -> Bool { 7 == 7 }",
            operation: PrimOp::Eq,
            arguments: vec![Atom::Int(7), Atom::Int(7)],
            expected: ExpectedValue::Bool(true),
        },
        PrimitiveCase {
            name: "ne",
            source: "fn main() -> Bool { 7 != 2 }",
            operation: PrimOp::Ne,
            arguments: vec![Atom::Int(7), Atom::Int(2)],
            expected: ExpectedValue::Bool(true),
        },
        PrimitiveCase {
            name: "lt",
            source: "fn main() -> Bool { 2 < 7 }",
            operation: PrimOp::Lt,
            arguments: vec![Atom::Int(2), Atom::Int(7)],
            expected: ExpectedValue::Bool(true),
        },
        PrimitiveCase {
            name: "le",
            source: "fn main() -> Bool { 7 <= 7 }",
            operation: PrimOp::Le,
            arguments: vec![Atom::Int(7), Atom::Int(7)],
            expected: ExpectedValue::Bool(true),
        },
        PrimitiveCase {
            name: "gt",
            source: "fn main() -> Bool { 7 > 2 }",
            operation: PrimOp::Gt,
            arguments: vec![Atom::Int(7), Atom::Int(2)],
            expected: ExpectedValue::Bool(true),
        },
        PrimitiveCase {
            name: "ge",
            source: "fn main() -> Bool { 7 >= 7 }",
            operation: PrimOp::Ge,
            arguments: vec![Atom::Int(7), Atom::Int(7)],
            expected: ExpectedValue::Bool(true),
        },
    ]
}

#[test]
fn atom_only_binary_primitive_family_lowers_to_exact_letprim_answer_jumps() {
    let engine = Engine::new().build().expect("engine builds");

    for case in primitive_cases() {
        let mut entry = engine
            .parse(case.source)
            .unwrap_or_else(|error| panic!("{} source must parse: {error}", case.name));
        engine
            .check(&mut entry)
            .unwrap_or_else(|error| panic!("{} source must typecheck: {error}", case.name));
        let lowered = engine
            .lower_entry_to_checked_cps(&entry)
            .unwrap_or_else(|error| {
                panic!(
                    "{} must lower through the checked atom-only primitive bridge: {error}",
                    case.name
                )
            });

        let Term::LetPrim {
            name,
            op,
            args,
            body,
        } = lowered
        else {
            panic!(
                "{} must lower to one LetPrim before the answer continuation",
                case.name
            );
        };
        assert_eq!(
            op, case.operation,
            "{} must retain its exact CPS PrimOp",
            case.name
        );
        assert_eq!(
            args, case.arguments,
            "{} must retain its literal atom arguments",
            case.name
        );
        assert!(
            matches!(
                *body,
                Term::Jump {
                    cont: ContRef::Label(ref answer),
                    arg: Atom::Var(ref result),
                    ..
                } if answer == "__answer" && result == &name
            ),
            "{} must jump its primitive result to the sealed answer continuation",
            case.name
        );
    }
}

#[tokio::test]
async fn engine_run_executes_the_full_atom_only_binary_primitive_family_through_sealed_checked_cps()
{
    let engine = Engine::new().build().expect("engine builds");

    for case in primitive_cases() {
        let value = engine.run(case.source).await.unwrap_or_else(|error| {
            panic!(
                "{} must execute through sealed checked Core/CPS admission: {error}",
                case.name
            )
        });
        assert_eq!(
            value,
            case.expected.engine_value(),
            "{} must retain its checked CPS terminal value",
            case.name
        );
    }

    assert_eq!(
        engine.production_execution_boundary(),
        ProductionExecutionBoundary::CheckedCoreCpsClosedAdmission,
        "the primitive family must not reopen the legacy direct evaluator"
    );
}

#[tokio::test]
async fn engine_run_file_executes_representative_atom_only_binary_primitives_through_sealed_checked_cps()
 {
    let engine = Engine::new().build().expect("engine builds");
    let directory = tempfile::tempdir().expect("temporary source directory creates");

    for (name, source, expected) in [
        ("mul", "fn main() -> Int { 7 * 2 }", Value::Int(14)),
        (
            "comparison",
            "fn main() -> Bool { 7 >= 7 }",
            Value::Bool(true),
        ),
        ("div", "fn main() -> Int { 8 / 2 }", Value::Int(4)),
    ] {
        let path = directory.path().join(format!("{name}.ash"));
        std::fs::write(&path, source).expect("primitive source file writes");
        let value = engine.run_file(&path).await.unwrap_or_else(|error| {
            panic!(
                "{name} file entry must execute through sealed checked Core/CPS admission: {error}"
            )
        });
        assert_eq!(
            value, expected,
            "{name} must retain its checked CPS terminal value"
        );
    }
}
