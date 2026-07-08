use super::support::*;

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
// TASK-556.3: fn helper(x) { x + 1 } in block -> BlockStmt::Let { expr: FnDef }
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
    } = statements[0]
    else {
        panic!("expected let statement, got: {:?}", statements[0]);
    };
    assert!(
        matches!(pattern, ash_parser::surface::Pattern::Variable { name, .. } if name.as_ref() == "helper"),
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
// TASK-557/TASK-959: Closure syntax |params| -> body
// ===========================================================================

// ---------------------------------------------------------------------------
// TASK-557.1: |x| -> x + 1 parses as Expr::FnDef with one param
// ---------------------------------------------------------------------------

#[test]
fn task557_closure_single_param() {
    use ash_parser::parse_expr::expr;
    let mut input = new_input(r#"|x| -> x + 1"#);
    let result = expr(&mut input).expect("|x| -> x + 1 should parse");
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
// TASK-557.2: |x, y| -> x + y parses with two params
// ---------------------------------------------------------------------------

#[test]
fn task557_closure_two_params() {
    use ash_parser::parse_expr::expr;
    let mut input = new_input(r#"|x, y| -> x + y"#);
    let result = expr(&mut input).expect("|x, y| -> x + y should parse");
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
// TASK-557.3: closure in call position parses: apply(|x| -> x * 2, 5)
// ---------------------------------------------------------------------------

#[test]
fn task557_closure_in_call_position() {
    use ash_parser::parse_expr::expr;
    let mut input = new_input(r#"apply(|x| -> x * 2, 5)"#);
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
// TASK-557.4: closure in let binding: let f = |x| -> x + 1;
// ---------------------------------------------------------------------------

#[test]
fn task557_closure_in_let_binding() {
    // Parse a fn body that contains `let f = |x| -> x + 1;`
    let src = r#"fn wrap() -> Int {
    let f = |x| -> x + 1;
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
    let BlockStmt::Let { ref expr, .. } = statements[0] else {
        panic!("expected let statement, got: {:?}", statements[0]);
    };
    assert!(
        matches!(expr, Expr::FnDef { params, .. } if params.len() == 1),
        "expected FnDef with one param in let binding, got: {:?}",
        expr
    );
    assert!(tail_expr.is_some(), "expected tail expr (f(0))");
}

#[test]
fn task689c_projected_callable_invocation_parses_as_fnapply() {
    use ash_parser::parse_expr::expr;

    let mut input = new_input(r#"env.policies.check(p)"#);
    let result = expr(&mut input).expect("projected callable invocation should parse");

    match result {
        Expr::FnApply { func, args, .. } => {
            assert_eq!(args.len(), 1, "expected one argument to projected callable");
            assert!(
                matches!(&args[0], Expr::Variable { name, .. } if name.as_ref() == "p"),
                "expected p variable as argument, got: {:?}",
                args[0]
            );
            assert!(
                matches!(
                    func.as_ref(),
                    Expr::FieldAccess { base, field, .. }
                        if field.as_ref() == "check"
                            && matches!(
                                base.as_ref(),
                                Expr::FieldAccess { base, field, .. }
                                    if field.as_ref() == "policies"
                                        && matches!(
                                            base.as_ref(),
                                            Expr::Variable { name, .. } if name.as_ref() == "env"
                                        )
                            )
                ),
                "expected nested field access callee, got: {:?}",
                func
            );
        }
        other => panic!(
            "expected FnApply for projected callable invocation, got: {:?}",
            other
        ),
    }
}

// TODO(TASK-590): known failure — parser gap with multiline record constructor + trailing comma.
#[test]
#[ignore = "TODO(TASK-590)"]
fn task590_debug_scan_tree_parse() {
    use ash_parser::input::new_input;
    use ash_parser::parse_module::parse_fn_definition;
    use winnow::prelude::Parser;

    let snippet = r#"pub fn scan_tree(root: String) -> FileTree {
    FileTree {
        spec_files: [],
        plan_files: [],
        example_files: [],
        changelog_files: [],
    }
}"#;
    let mut input = new_input(snippet);
    let result = parse_fn_definition.parse_next(&mut input);
    if let Err(ref e) = result {
        eprintln!("Parse failed: {}", e);
    }
    assert!(
        result.is_ok(),
        "expected parse to succeed, got: {:?}",
        result
    );
}
