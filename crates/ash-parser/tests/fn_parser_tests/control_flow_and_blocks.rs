use super::support::*;

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

#[test]
fn parse_fn_match_constructor_expression_scrutinee() {
    let def = parse_fn(
        r#"fn describe() -> Int { match Some { value: 41 } { Some { value: value } => value, None => 0 } }"#,
    );
    let Definition::Function(f) = def else {
        panic!("expected Function definition");
    };
    match &f.body {
        Expr::Block { tail_expr, .. } => {
            let tail = tail_expr.as_ref().expect("should have tail expr");
            match tail.as_ref() {
                Expr::Match {
                    scrutinee, arms, ..
                } => {
                    assert!(
                        matches!(scrutinee.as_ref(), Expr::Constructor { .. }),
                        "expected constructor scrutinee, got: {scrutinee:?}"
                    );
                    assert_eq!(arms.len(), 2, "expected two match arms");
                }
                other => panic!("expected Match, got: {:?}", other),
            }
        }
        other => panic!("expected Block, got: {:?}", other),
    }
}

#[test]
fn parse_fn_match_call_field_and_binary_scrutinees() {
    let cases = [
        (
            r#"fn describe() -> Int { match make() { Some { value: value } => value, None => 0 } }"#,
            "call",
        ),
        (
            r#"fn describe() -> Int { match holder.inner { Box { item: item } => item } }"#,
            "field projection",
        ),
        (
            r#"fn describe() -> Int { match 40 + 1 { 41 => 1, _ => 0 } }"#,
            "binary",
        ),
    ];

    for (source, label) in cases {
        let def = parse_fn(source);
        let Definition::Function(f) = def else {
            panic!("expected Function definition for {label}");
        };
        match &f.body {
            Expr::Block { tail_expr, .. } => {
                let tail = tail_expr.as_ref().expect("should have tail expr");
                assert!(
                    matches!(tail.as_ref(), Expr::Match { .. }),
                    "expected Match for {label}, got: {tail:?}"
                );
            }
            other => panic!("expected Block for {label}, got: {:?}", other),
        }
    }
}

// ---------------------------------------------------------------------------
// 7. panic expression
// ---------------------------------------------------------------------------

#[test]
fn parse_fn_panic() {
    let def = parse_fn(r#"fn unreachable_panic() -> Int { panic "unreachable" }"#);
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
fn parse_fn_rejects_nested_fn() {
    // After TASK-556, nested `fn` inside a fn body is parsed successfully as a
    // BlockStmt::Let { expr: Expr::FnDef { ... } }.  The parse stage no longer
    // rejects it.  Rejection of Expr::Block / nested-fn usage happens during
    // lowering, not during parsing.
    let mut input = new_input(r#"fn outer() -> Int { fn inner() -> Int { 1 } inner() }"#);
    let result = parse_fn_definition(&mut input);
    let def = result.expect("nested fn should parse successfully after TASK-556");

    let Definition::Function(f) = def else {
        panic!("expected Function definition");
    };

    // The body should be an Expr::Block with a BlockStmt::Let containing Expr::FnDef
    let Expr::Block {
        ref statements,
        ref tail_expr,
        ..
    } = f.body
    else {
        panic!("expected Block body, got: {:?}", f.body);
    };

    assert_eq!(
        statements.len(),
        1,
        "expected one BlockStmt (the inner fn let-binding)"
    );
    let BlockStmt::Let {
        ref expr,
        ref pattern,
        ..
    } = statements[0];
    assert!(
        matches!(expr, Expr::FnDef { .. }),
        "expected FnDef expression in let-binding, got: {:?}",
        expr
    );
    assert!(
        matches!(pattern, ash_parser::surface::Pattern::Variable { name, .. } if name.as_ref() == "inner"),
        "expected pattern Variable(\"inner\"), got: {:?}",
        pattern
    );
    assert!(tail_expr.is_some(), "expected tail expression (inner())");

    // After TASK-649, Expr::Block is desugared to nested Expr::Let during lowering.
    // The nested fn definition is preserved as Expr::FnDef inside the let-binding's expr.
    let lower_result = lower_expr(&f.body);
    assert!(
        lower_result.is_ok(),
        "lowering of Expr::Block (containing nested fn) should succeed, but got: {:?}",
        lower_result
    );

    // The desugared result should be CoreExpr::Let { pattern: inner, expr: FnDef, body: inner() }
    let lowered = lower_result.unwrap();
    let ash_core::ast::Expr::Let {
        ref pattern,
        ref expr,
        ref body,
        ..
    } = lowered
    else {
        panic!("expected CoreExpr::Let, got: {:?}", lowered);
    };
    let ash_core::ast::Pattern::Variable { name, .. } = pattern else {
        panic!("expected Variable pattern, got: {:?}", pattern);
    };
    assert_eq!(name, "inner", "expected pattern name 'inner'");
    assert!(
        matches!(expr.as_ref(), ash_core::ast::Expr::FnDef { .. }),
        "expected FnDef as bound expression, got: {:?}",
        expr
    );
    assert!(
        matches!(body.as_ref(), ash_core::ast::Expr::FnApply { .. }),
        "expected FnApply as body (inner()), got: {:?}",
        body
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
