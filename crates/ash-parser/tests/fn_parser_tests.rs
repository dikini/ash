//! Tests for fn definition, fn type, and fn body expression parsing.

use ash_parser::input::new_input;
use ash_parser::parse_module::parse_fn_definition;
use ash_parser::surface::{Definition, Expr, Type};

// ---------------------------------------------------------------------------
// Helper: parse a fn definition from source text
// ---------------------------------------------------------------------------
fn parse_fn(input_str: &str) -> Definition {
    let mut input = new_input(input_str);
    parse_fn_definition(&mut input).expect("fn definition should parse")
}

// ---------------------------------------------------------------------------
// 1. Simple fn definition
// ---------------------------------------------------------------------------
#[test]
fn parse_simple_fn() {
    let def = parse_fn(r#"fn add(a: Int, b: Int) -> Int { a + b }"#);
    let Definition::Function(f) = def else {
        panic!("expected Function definition");
    };
    assert_eq!(f.name.as_ref(), "add");
    assert_eq!(f.params.len(), 2);
    assert_eq!(f.params[0].name.as_ref(), "a");
    assert_eq!(f.params[1].name.as_ref(), "b");
    assert!(f.return_type.is_some());
}

// ---------------------------------------------------------------------------
// 2. fn with contract (requires)
// ---------------------------------------------------------------------------
#[test]
fn parse_fn_with_requires() {
    let def = parse_fn(r#"fn safe_div(n: Int, d: Int) -> Int requires: d != 0 { n / d }"#);
    let Definition::Function(f) = def else {
        panic!("expected Function definition");
    };
    assert_eq!(f.name.as_ref(), "safe_div");
    assert!(f.contract.is_some());
    let contract = f.contract.unwrap();
    assert_eq!(contract.requires.len(), 1);
    assert!(contract.ensures.is_empty());
}

#[test]
fn normalize_comma_separated_requires_and_ensures() {
    let def = parse_fn(
        r#"fn classify(n: Int) -> Int requires: n >= 0, n != 0 ensures: result >= 0, result != 0 { n }"#,
    );
    let Definition::Function(f) = def else {
        panic!("expected Function definition");
    };

    let contract = f.contract.expect("expected fn contract");
    assert_eq!(contract.requires.len(), 2);
    assert_eq!(contract.ensures.len(), 2);
}

#[test]
fn normalize_repeated_and_comma_separated_requires_to_same_shape() {
    let repeated = parse_fn(r#"fn a(n: Int) -> Int requires: n >= 0 requires: n != 0 { n }"#);
    let comma = parse_fn(r#"fn b(n: Int) -> Int requires: n >= 0, n != 0 { n }"#);

    let Definition::Function(repeated_fn) = repeated else {
        panic!("expected repeated fn definition");
    };
    let Definition::Function(comma_fn) = comma else {
        panic!("expected comma fn definition");
    };

    let repeated_contract = repeated_fn.contract.expect("expected repeated contract");
    let comma_contract = comma_fn.contract.expect("expected comma contract");

    assert_eq!(
        repeated_contract.requires.len(),
        comma_contract.requires.len()
    );
    for (left, right) in repeated_contract
        .requires
        .iter()
        .zip(comma_contract.requires.iter())
    {
        match (left, right) {
            (
                ash_parser::surface::Requirement::Arithmetic {
                    expr: Expr::Binary { op: left_op, .. },
                },
                ash_parser::surface::Requirement::Arithmetic {
                    expr: Expr::Binary { op: right_op, .. },
                },
            ) => assert_eq!(left_op, right_op),
            other => panic!("expected normalized arithmetic predicates, got {other:?}"),
        }
    }
}

// ---------------------------------------------------------------------------
// 3. Fn type syntax
// ---------------------------------------------------------------------------
#[test]
fn parse_fn_type() {
    // Parse via a wrapper fn to exercise the type parser
    let def = parse_fn(r#"fn _dummy() -> Fn(Int, Int) -> Int { 0 }"#);
    let Definition::Function(f) = def else {
        panic!("expected Function definition");
    };
    let rt = f.return_type.expect("should have return type");
    match rt {
        Type::Fn(params, ref _ret) => {
            assert_eq!(params.len(), 2, "expected 2 params in Fn type");
        }
        other => panic!("expected Type::Fn, got: {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 4. if expression in fn body
// ---------------------------------------------------------------------------
#[test]
fn parse_fn_if_expr() {
    let def = parse_fn(r#"fn abs(n: Int) -> Int { if n < 0 then 0 - n else n }"#);
    let Definition::Function(f) = def else {
        panic!("expected Function definition");
    };
    match &f.body {
        Expr::Block { tail_expr, .. } => {
            let tail = tail_expr.as_ref().expect("should have tail expr");
            assert!(
                matches!(tail.as_ref(), Expr::If { .. }),
                "expected If expr, got: {:?}",
                tail
            );
        }
        other => panic!("expected Block, got: {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 5. One-armed if (no else)
// ---------------------------------------------------------------------------
#[test]
fn parse_fn_one_armed_if() {
    let def = parse_fn(r#"fn maybe_inc(n: Int) -> Int { if n > 0 then n + 1 }"#);
    let Definition::Function(f) = def else {
        panic!("expected Function definition");
    };
    match &f.body {
        Expr::Block { tail_expr, .. } => {
            let tail = tail_expr.as_ref().expect("should have tail expr");
            match tail.as_ref() {
                Expr::If { else_branch, .. } => {
                    assert!(else_branch.is_none(), "one-armed if should have no else");
                }
                other => panic!("expected If, got: {:?}", other),
            }
        }
        other => panic!("expected Block, got: {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 6. match expression in fn body
// ---------------------------------------------------------------------------
#[test]
fn parse_fn_match_expr() {
    // Single arm match with int literal
    let def = parse_fn(r#"fn describe(n: Int) -> Int { match n { 0 => 1 } }"#);
    let Definition::Function(f) = def else {
        panic!("expected Function definition");
    };
    match &f.body {
        Expr::Block { tail_expr, .. } => {
            let tail = tail_expr.as_ref().expect("should have tail expr");
            match tail.as_ref() {
                Expr::Match { arms, .. } => {
                    assert_eq!(arms.len(), 1, "expected 1 match arm");
                }
                other => panic!("expected Match, got: {:?}", other),
            }
        }
        other => panic!("expected Block, got: {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 7. panic expression
// ---------------------------------------------------------------------------
#[test]
fn parse_fn_panic() {
    let def = parse_fn(r#"fn fail() -> Int { panic "unreachable" }"#);
    let Definition::Function(f) = def else {
        panic!("expected Function definition");
    };
    match &f.body {
        Expr::Block { tail_expr, .. } => {
            let tail = tail_expr.as_ref().expect("should have tail expr");
            match tail.as_ref() {
                Expr::Panic { message, .. } => {
                    assert_eq!(message.as_ref(), "unreachable");
                }
                other => panic!("expected Panic, got: {:?}", other),
            }
        }
        other => panic!("expected Block, got: {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 8. Block with let bindings
// ---------------------------------------------------------------------------
#[test]
fn parse_fn_block_with_let() {
    let def = parse_fn(r#"fn compute(x: Int) -> Int { let y = x + 1; y * 2 }"#);
    let Definition::Function(f) = def else {
        panic!("expected Function definition");
    };
    match &f.body {
        Expr::Block {
            statements,
            tail_expr,
            ..
        } => {
            assert_eq!(statements.len(), 1, "expected 1 let statement");
            assert!(tail_expr.is_some(), "expected tail expr");
        }
        other => panic!("expected Block, got: {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 9. pub fn
// ---------------------------------------------------------------------------
#[test]
fn parse_pub_fn() {
    let def = parse_fn(r#"pub fn helper(n: Int) -> Int { n + 1 }"#);
    let Definition::Function(f) = def else {
        panic!("expected Function definition");
    };
    assert_eq!(f.name.as_ref(), "helper");
    // Visibility should not be Inherited (the default)
    assert!(
        !matches!(f.visibility, ash_parser::surface::Visibility::Inherited),
        "expected pub visibility"
    );
}

// ---------------------------------------------------------------------------
// 10. fn rejects nested fn
// ---------------------------------------------------------------------------
#[test]
fn parse_fn_rejects_nested_fn() {
    let mut input = new_input(r#"fn outer() -> Int { fn inner() -> Int { 1 } inner() }"#);
    let result = parse_fn_definition(&mut input);
    // The nested "fn" keyword should cause a parse error because "fn" is now
    // a keyword and cannot be parsed as an identifier or expression.
    assert!(
        result.is_err(),
        "nested fn should be rejected, but got: {:?}",
        result
    );
}

// ---------------------------------------------------------------------------
// Additional: empty fn body
// ---------------------------------------------------------------------------
#[test]
fn parse_fn_empty_body() {
    let def = parse_fn(r#"fn noop() -> Int { }"#);
    let Definition::Function(f) = def else {
        panic!("expected Function definition");
    };
    match &f.body {
        Expr::Block {
            statements,
            tail_expr,
            ..
        } => {
            assert!(statements.is_empty());
            assert!(tail_expr.is_none());
        }
        other => panic!("expected Block, got: {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Additional: fn with type params
// ---------------------------------------------------------------------------
#[test]
fn parse_fn_with_type_params() {
    let def = parse_fn(r#"fn identity<T>(x: T) -> T { x }"#);
    let Definition::Function(f) = def else {
        panic!("expected Function definition");
    };
    assert_eq!(f.name.as_ref(), "identity");
    assert_eq!(f.type_params.len(), 1);
    assert_eq!(f.type_params[0].as_ref(), "T");
}
