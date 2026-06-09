use super::support::*;

#[test]
fn task689d_parse_fn_parameter_with_arrow_function_type() {
    let def = parse_fn(r#"fn keep(f: Int -> Int) -> Int { 1 }"#);
    let Definition::Function(f) = def else {
        panic!("expected Function definition");
    };
    assert_eq!(f.params.len(), 1);
    match &f.params[0].ty {
        Type::Fn(params, ret) => {
            assert_eq!(params.len(), 1);
            match &params[0] {
                Type::Name(name) => assert_eq!(name.as_ref(), "Int"),
                other => panic!("expected Int parameter type, got {other:?}"),
            }
            match ret.as_ref() {
                Type::Name(name) => assert_eq!(name.as_ref(), "Int"),
                other => panic!("expected Int return type, got {other:?}"),
            }
        }
        other => panic!("expected arrow function type, got {other:?}"),
    }
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
fn parse_fn_with_type_params() {
    let def = parse_fn(r#"fn identity<T>(x: T) -> T { x }"#);
    let Definition::Function(f) = def else {
        panic!("expected Function definition");
    };
    assert_eq!(f.name.as_ref(), "identity");
    assert_eq!(f.type_params.len(), 1);
    assert_eq!(f.type_params[0].as_ref(), "T");
}

// ===========================================================================
// TASK-556: Anonymous fn expression and named local fn parsing
// ===========================================================================

// ---------------------------------------------------------------------------
// TASK-556.1: Anonymous fn(x) { x + 1 } parses as Expr::FnDef
// ---------------------------------------------------------------------------
