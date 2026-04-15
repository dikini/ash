//! Tests for fn definition, fn type, and fn body expression parsing.

use ash_parser::input::new_input;
use ash_parser::lower::lower_expr;
use ash_parser::parse_module::parse_fn_definition;
use ash_parser::surface::{BlockStmt, Definition, Expr, Type};

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
// 10. fn accepts nested fn at parse time; lowering rejects Expr::Block
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
        matches!(pattern, ash_parser::surface::Pattern::Variable(name) if name.as_ref() == "inner"),
        "expected pattern Variable(\"inner\"), got: {:?}",
        pattern
    );
    assert!(tail_expr.is_some(), "expected tail expression (inner())");

    // Lowering the body should fail because Expr::Block lowering is not yet
    // supported in core IR (blocks are fn-body-only constructs).
    let lower_result = lower_expr(&f.body);
    assert!(
        lower_result.is_err(),
        "lowering of Expr::Block (containing nested fn) should fail, but got: {:?}",
        lower_result
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

// ===========================================================================
// TASK-556: Anonymous fn expression and named local fn parsing
// ===========================================================================

// ---------------------------------------------------------------------------
// TASK-556.1: Anonymous fn(x) { x + 1 } parses as Expr::FnDef
// ---------------------------------------------------------------------------
#[test]
fn task556_anon_fn_expr_parses_as_fn_def() {
    use ash_parser::parse_expr::expr;
    let mut input = new_input(r#"fn(x) { x + 1 }"#);
    let result = expr(&mut input).expect("anonymous fn expression should parse");
    assert!(
        matches!(result, Expr::FnDef { ref params, .. } if params.len() == 1),
        "expected FnDef with one param, got: {:?}",
        result
    );
    if let Expr::FnDef {
        params,
        return_type,
        ..
    } = result
    {
        assert_eq!(params[0].0.as_ref(), "x");
        assert!(params[0].1.is_none(), "expected no type annotation on x");
        assert!(return_type.is_none(), "expected no return type");
    }
}

// ---------------------------------------------------------------------------
// TASK-556.2: fn(x: Int) -> Int { x + 1 } parses with type annotations
// ---------------------------------------------------------------------------
#[test]
fn task556_anon_fn_expr_with_types() {
    use ash_parser::parse_expr::expr;
    let mut input = new_input(r#"fn(x: Int) -> Int { x + 1 }"#);
    let result = expr(&mut input).expect("typed anonymous fn expression should parse");
    if let Expr::FnDef {
        params,
        return_type,
        ..
    } = result
    {
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].0.as_ref(), "x");
        assert_eq!(params[0].1.as_deref(), Some("Int"));
        assert_eq!(return_type.as_deref(), Some("Int"));
    } else {
        panic!("expected FnDef, got: {:?}", result);
    }
}

// ---------------------------------------------------------------------------
// TASK-556.3: fn helper(x) { x + 1 } in workflow body -> Workflow::Let { expr: FnDef }
// ---------------------------------------------------------------------------
#[test]
fn task556_named_fn_in_workflow_desugars_to_let() {
    use ash_parser::parse_workflow::workflow;
    let src = r#"fn helper(x) { x + 1 }
done"#;
    let mut input = new_input(src);
    let result = workflow(&mut input).expect("workflow with named local fn should parse");
    // The workflow should be a Let wrapping a Done
    if let ash_parser::surface::Workflow::Let {
        ref pattern,
        ref expr,
        ..
    } = result
    {
        assert!(
            matches!(pattern, ash_parser::surface::Pattern::Variable(name) if name.as_ref() == "helper"),
            "expected Variable(\"helper\"), got: {:?}",
            pattern
        );
        assert!(
            matches!(expr, Expr::FnDef { params, .. } if params.len() == 1),
            "expected FnDef with one param, got: {:?}",
            expr
        );
    } else {
        panic!("expected Workflow::Let, got: {:?}", result);
    }
}

// ---------------------------------------------------------------------------
// TASK-556.4: fn helper(x) { x + 1 } in block -> BlockStmt::Let { expr: FnDef }
// ---------------------------------------------------------------------------
#[test]
fn task556_named_fn_in_block_desugars_to_let() {
    // A module-level fn whose body contains a named local fn as a block statement
    let src = r#"fn outer() -> Int {
    fn helper(x) { x + 1 }
    helper(0)
}"#;
    let mut input = new_input(src);
    let def = parse_fn_definition(&mut input).expect("fn with local fn in body should parse");
    let Definition::Function(f) = def else {
        panic!("expected Function definition");
    };
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
        "expected one block statement (the local fn)"
    );
    let BlockStmt::Let {
        ref pattern,
        ref expr,
        ..
    } = statements[0];
    assert!(
        matches!(pattern, ash_parser::surface::Pattern::Variable(name) if name.as_ref() == "helper"),
        "expected Variable(\"helper\"), got: {:?}",
        pattern
    );
    assert!(
        matches!(expr, Expr::FnDef { params, .. } if params.len() == 1),
        "expected FnDef with one param, got: {:?}",
        expr
    );
    assert!(tail_expr.is_some(), "expected tail expression (helper(0))");
}

// ---------------------------------------------------------------------------
// TASK-556.5: fn(x) { x } at module scope -> lowering error
// ---------------------------------------------------------------------------
#[test]
fn task556_anon_fn_at_module_scope_lower_error() {
    // An anonymous fn(x){x} used at module scope must be rejected.
    // FnDef is valid inside function bodies / let bindings, but NOT at
    // module scope where only named `pub fn` definitions are allowed.
    use ash_parser::lower::{LoweringError, lower_module_expr};
    use ash_parser::parse_expr::expr;
    let mut input = new_input(r#"fn(x) { x }"#);
    let parsed = expr(&mut input).expect("anonymous fn should parse");
    assert!(
        matches!(parsed, Expr::FnDef { .. }),
        "expected FnDef, got: {:?}",
        parsed
    );
    let lower_result = lower_module_expr(&parsed);
    assert!(
        matches!(
            lower_result,
            Err(LoweringError::FnDefNotAllowedAtModuleScope)
        ),
        "lowering FnDef at module scope should produce FnDefNotAllowedAtModuleScope, got: {:?}",
        lower_result
    );
}

// ===========================================================================
// TASK-557: Closure syntax |params| => body
// ===========================================================================

// ---------------------------------------------------------------------------
// TASK-557.1: |x| => x + 1 parses as Expr::FnDef with one param
// ---------------------------------------------------------------------------
#[test]
fn task557_closure_single_param() {
    use ash_parser::parse_expr::expr;
    let mut input = new_input(r#"|x| => x + 1"#);
    let result = expr(&mut input).expect("|x| => x + 1 should parse");
    match result {
        Expr::FnDef {
            ref params,
            ref return_type,
            ..
        } => {
            assert_eq!(params.len(), 1, "expected 1 param");
            assert_eq!(params[0].0.as_ref(), "x");
            assert!(params[0].1.is_none(), "expected no type annotation");
            assert!(
                return_type.is_none(),
                "closure shorthand has no return type"
            );
        }
        other => panic!("expected FnDef, got: {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// TASK-557.2: |x, y| => x + y parses with two params
// ---------------------------------------------------------------------------
#[test]
fn task557_closure_two_params() {
    use ash_parser::parse_expr::expr;
    let mut input = new_input(r#"|x, y| => x + y"#);
    let result = expr(&mut input).expect("|x, y| => x + y should parse");
    match result {
        Expr::FnDef { ref params, .. } => {
            assert_eq!(params.len(), 2, "expected 2 params");
            assert_eq!(params[0].0.as_ref(), "x");
            assert_eq!(params[1].0.as_ref(), "y");
        }
        other => panic!("expected FnDef, got: {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// TASK-557.3: closure in call position parses: apply(|x| => x * 2, 5)
// ---------------------------------------------------------------------------
#[test]
fn task557_closure_in_call_position() {
    use ash_parser::parse_expr::expr;
    let mut input = new_input(r#"apply(|x| => x * 2, 5)"#);
    let result = expr(&mut input).expect("call with closure arg should parse");
    // The outer expression is a function call; the first argument should be FnDef
    match result {
        Expr::Call { ref args, .. } | Expr::FnApply { ref args, .. } => {
            assert!(
                !args.is_empty(),
                "expected at least one argument (the closure)"
            );
            assert!(
                matches!(args[0], Expr::FnDef { .. }),
                "first arg should be FnDef (desugared closure), got: {:?}",
                args[0]
            );
        }
        other => panic!("expected Call or FnApply, got: {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// TASK-557.4: closure in let binding: let f = |x| => x + 1;
// ---------------------------------------------------------------------------
#[test]
fn task557_closure_in_let_binding() {
    // Parse a fn body that contains `let f = |x| => x + 1;`
    let src = r#"fn wrap() -> Int {
    let f = |x| => x + 1;
    f(0)
}"#;
    let mut input = new_input(src);
    let def = parse_fn_definition(&mut input).expect("fn with closure let should parse");
    let Definition::Function(f) = def else {
        panic!("expected Function definition");
    };
    let Expr::Block {
        ref statements,
        ref tail_expr,
        ..
    } = f.body
    else {
        panic!("expected Block body, got: {:?}", f.body);
    };
    assert_eq!(statements.len(), 1, "expected one let statement");
    let BlockStmt::Let { ref expr, .. } = statements[0];
    assert!(
        matches!(expr, Expr::FnDef { params, .. } if params.len() == 1),
        "expected FnDef with one param in let binding, got: {:?}",
        expr
    );
    assert!(tail_expr.is_some(), "expected tail expr (f(0))");
}
