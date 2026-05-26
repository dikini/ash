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

#[test]
fn parse_fn_with_keyword_name_then() {
    let def = parse_fn(r#"fn then(a: Int, b: Int) -> Int { b }"#);
    let Definition::Function(f) = def else {
        panic!("expected Function definition");
    };
    assert_eq!(f.name.as_ref(), "then");
    assert_eq!(f.params.len(), 2);
}

#[test]
fn parse_fn_with_keyword_name_guard() {
    let def = parse_fn(r#"fn guard(a: Int) -> Int { a }"#);
    let Definition::Function(f) = def else {
        panic!("expected Function definition");
    };
    assert_eq!(f.name.as_ref(), "guard");
    assert_eq!(f.params.len(), 1);
}

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
// 10. fn accepts nested fn at parse time; lowering desugars Expr::Block to nested Expr::Let
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
            matches!(pattern, ash_parser::surface::Pattern::Variable { name, .. } if name.as_ref() == "helper"),
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
    let BlockStmt::Let { ref expr, .. } = statements[0];
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

#[test]
fn task590_debug_scan_tree_parse_minimal() {
    use ash_parser::input::new_input;
    use ash_parser::parse_module::parse_fn_definition;
    use winnow::prelude::Parser;

    let cases = [
        r#"pub fn scan_tree(root: String) -> FileTree { root }"#,
        r#"pub fn scan_tree(root: String) { FileTree { spec_files: [] } }"#,
        r#"pub fn scan_tree(root: String) -> FileTree { FileTree { spec_files: [] } }"#,
        r#"pub fn scan_tree(root: String) -> FileTree { FileTree { spec_files: [], plan_files: [] } }"#,
    ];
    for (i, snippet) in cases.iter().enumerate() {
        let mut input = new_input(snippet);
        let result = parse_fn_definition.parse_next(&mut input);
        eprintln!(
            "Case {}: {:?} -> {}",
            i,
            snippet,
            if result.is_ok() { "OK" } else { "FAIL" }
        );
        if let Err(ref e) = result {
            eprintln!("  Error: {}", e);
        }
    }
}

#[test]
#[ignore = "TODO(TASK-590)"]
fn task590_debug_collect_ash_file_parse() {
    use ash_parser::input::new_input;
    use ash_parser::parse_module::parse_fn_definition;
    use winnow::prelude::Parser;

    let source_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/plan/PLAN-090-SPEC-PROCESSOR.md");
    let source = std::fs::read_to_string(&source_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", source_path.display()));
    let lines: Vec<&str> = source.lines().collect();
    let mut snippet = String::new();
    let mut in_snippet = false;
    let mut brace_depth = 0usize;
    let mut seen_open = false;

    for line in &lines {
        let trimmed = line.trim_start();
        if !in_snippet && trimmed.starts_with("pub fn ") {
            in_snippet = true;
            snippet.clear();
            brace_depth = 0;
            seen_open = false;
        }
        if in_snippet {
            if !snippet.is_empty() {
                snippet.push('\n');
            }
            snippet.push_str(line);
            for ch in line.chars() {
                match ch {
                    '{' => {
                        brace_depth += 1;
                        seen_open = true;
                    }
                    '}' => {
                        brace_depth -= 1;
                    }
                    _ => {}
                }
            }
            if seen_open && brace_depth == 0 {
                break;
            }
        }
    }

    eprintln!("Extracted snippet:\n{}", snippet);
    eprintln!("Snippet bytes: {:?}", snippet.as_bytes());

    let mut input = new_input(&snippet);
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

#[test]
fn task590_debug_multiline_record_constructor() {
    use ash_parser::input::new_input;
    use ash_parser::parse_module::parse_fn_definition;
    use winnow::prelude::Parser;

    let cases = [
        (
            r#"pub fn scan_tree(root: String) -> FileTree { FileTree { spec_files: [] } }"#,
            true,
        ),
        (
            r#"pub fn scan_tree(root: String) -> FileTree {
    FileTree { spec_files: [] }
}"#,
            true,
        ),
        (
            r#"pub fn scan_tree(root: String) -> FileTree {
    FileTree {
        spec_files: []
    }
}"#,
            true,
        ),
    ];
    for (i, (snippet, expected)) in cases.iter().enumerate() {
        let mut input = new_input(snippet);
        let result = parse_fn_definition.parse_next(&mut input);
        eprintln!(
            "Case {}: expected={} actual={}",
            i,
            expected,
            result.is_ok()
        );
        if let Err(ref e) = result {
            eprintln!("  Error: {}", e);
        }
        assert_eq!(result.is_ok(), *expected);
    }
}

// TODO(TASK-590): known failure — parser gap with multiline record constructor + trailing comma.
#[test]
#[ignore = "TODO(TASK-590)"]
fn task590_debug_exact_file_snippet() {
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
    eprintln!("Result: {}", if result.is_ok() { "OK" } else { "FAIL" });
    if let Err(ref e) = result {
        eprintln!("Error: {}", e);
    }
    assert!(result.is_ok(), "got: {:?}", result);
}

#[test]
fn task590_debug_field_count_isolation() {
    use ash_parser::input::new_input;
    use ash_parser::parse_module::parse_fn_definition;
    use winnow::prelude::Parser;

    let cases = [
        (
            "1 field",
            r#"pub fn scan_tree(root: String) -> FileTree {
    FileTree { spec_files: [] }
}"#,
        ),
        (
            "2 fields inline",
            r#"pub fn scan_tree(root: String) -> FileTree {
    FileTree { spec_files: [], plan_files: [] }
}"#,
        ),
        (
            "2 fields multiline",
            r#"pub fn scan_tree(root: String) -> FileTree {
    FileTree {
        spec_files: [],
        plan_files: []
    }
}"#,
        ),
        (
            "3 fields multiline trailing comma",
            r#"pub fn scan_tree(root: String) -> FileTree {
    FileTree {
        spec_files: [],
        plan_files: [],
        example_files: [],
    }
}"#,
        ),
        (
            "4 fields multiline trailing comma",
            r#"pub fn scan_tree(root: String) -> FileTree {
    FileTree {
        spec_files: [],
        plan_files: [],
        example_files: [],
        changelog_files: [],
    }
}"#,
        ),
    ];
    for (name, snippet) in cases.iter() {
        let mut input = new_input(snippet);
        let result = parse_fn_definition.parse_next(&mut input);
        eprintln!("{}: {}", name, if result.is_ok() { "OK" } else { "FAIL" });
    }
}

#[test]
fn task590_debug_let_then_constructor() {
    use ash_parser::input::new_input;
    use ash_parser::parse_module::parse_fn_definition;
    use winnow::prelude::Parser;

    let cases = [
        r#"pub fn scan_tree(root: String) -> FileTree {
    let all = collect_files([], root);
    FileTree { spec_files: all }
}"#,
        r#"pub fn scan_tree(root: String) -> FileTree {
    let all = [];
    FileTree { spec_files: all }
}"#,
        r#"pub fn scan_tree(root: String) -> FileTree {
    let all = root;
    FileTree { spec_files: [], plan_files: [], example_files: [], changelog_files: [] }
}"#,
    ];
    for (i, snippet) in cases.iter().enumerate() {
        let mut input = new_input(snippet);
        let result = parse_fn_definition.parse_next(&mut input);
        eprintln!("Case {}: {}", i, if result.is_ok() { "OK" } else { "FAIL" });
        if let Err(ref e) = result {
            eprintln!("  Error: {}", e);
        }
    }
}

// TODO(TASK-590): known failure — parser gap with record constructor containing closure arguments.
#[test]
#[ignore = "TODO(TASK-590)"]
fn task590_debug_long_constructor() {
    use ash_parser::input::new_input;
    use ash_parser::parse_module::parse_fn_definition;
    use winnow::prelude::Parser;

    let snippet = r#"pub fn scan_tree(root: String) -> FileTree {
    let all = collect_files([], root);
    FileTree { spec_files: filter(all, fn(p) { starts_with(p, "SPEC-") && ends_with(p, ".md") }), plan_files: filter(all, fn(p) { starts_with(p, "PLAN-") && ends_with(p, ".md") }), example_files: filter(all, fn(p) { ends_with(p, ".ash") }), changelog_files: filter(all, fn(p) { ends_with(p, "CHANGELOG.md") }) }
}"#;
    let mut input = new_input(snippet);
    let result = parse_fn_definition.parse_next(&mut input);
    eprintln!("Result: {}", if result.is_ok() { "OK" } else { "FAIL" });
    if let Err(ref e) = result {
        eprintln!("Error: {}", e);
    }
    assert!(result.is_ok(), "got: {:?}", result);
}

// TODO(TASK-590): known failure — parser gap with let-then-record-constructor pattern.
#[test]
#[ignore = "TODO(TASK-590)"]
fn task590_debug_let_bindings_then_constructor() {
    use ash_parser::input::new_input;
    use ash_parser::parse_module::parse_fn_definition;
    use winnow::prelude::Parser;

    let snippet = r#"pub fn scan_tree(root: String) -> FileTree {
    let all = collect_files([], root);
    let specs = filter(all, fn(p) { starts_with(p, "SPEC-") && ends_with(p, ".md") });
    let plans = filter(all, fn(p) { starts_with(p, "PLAN-") && ends_with(p, ".md") });
    let examples = filter(all, fn(p) { ends_with(p, ".ash") });
    let changelogs = filter(all, fn(p) { ends_with(p, "CHANGELOG.md") });
    FileTree { spec_files: specs, plan_files: plans, example_files: examples, changelog_files: changelogs }
}"#;
    let mut input = new_input(snippet);
    let result = parse_fn_definition.parse_next(&mut input);
    eprintln!("Result: {}", if result.is_ok() { "OK" } else { "FAIL" });
    if let Err(ref e) = result {
        eprintln!("Error: {}", e);
    }
    assert!(result.is_ok(), "got: {:?}", result);
}

#[test]
fn task590_debug_let_closure() {
    use ash_parser::input::new_input;
    use ash_parser::parse_module::parse_fn_definition;
    use winnow::prelude::Parser;

    let cases = [
        (
            "simple let",
            r#"pub fn f() {
    let x = filter([], fn(p) { p });
    x
}"#,
        ),
        (
            "let with string ops",
            r#"pub fn f() {
    let x = filter([], fn(p) { starts_with(p, "a") });
    x
}"#,
        ),
        (
            "let with &&",
            r#"pub fn f() {
    let x = filter([], fn(p) { starts_with(p, "a") && ends_with(p, "b") });
    x
}"#,
        ),
        (
            "two lets",
            r#"pub fn f() {
    let x = filter([], fn(p) { starts_with(p, "a") });
    let y = filter([], fn(p) { ends_with(p, "b") });
    x
}"#,
        ),
    ];
    for (name, snippet) in cases.iter() {
        let mut input = new_input(snippet);
        let result = parse_fn_definition.parse_next(&mut input);
        eprintln!("{}: {}", name, if result.is_ok() { "OK" } else { "FAIL" });
        if let Err(ref e) = result {
            eprintln!("  Error: {}", e);
        }
    }
}

#[test]
fn task590_debug_closure_in_call_arg() {
    use ash_parser::input::new_input;
    use ash_parser::parse_expr::expr;

    let cases = [
        r#"filter([], fn(p) { p })"#,
        r#"filter([], |p| -> p)"#,
        r#"filter([], fn(p) { starts_with(p, "a") })"#,
    ];
    for (i, snippet) in cases.iter().enumerate() {
        let mut input = new_input(snippet);
        let result = expr(&mut input);
        eprintln!("Case {}: {}", i, if result.is_ok() { "OK" } else { "FAIL" });
        if let Err(ref e) = result {
            eprintln!("  Error: {}", e);
        }
    }
}

#[test]
fn task590_debug_pipe_closure_in_call_arg() {
    use ash_parser::input::new_input;
    use ash_parser::parse_expr::expr;

    let cases = [
        r#"filter([], |p| -> p)"#,
        r#"filter([], |p| -> starts_with(p, "a"))"#,
        r#"filter([], |p| -> starts_with(p, "a") && ends_with(p, "b"))"#,
    ];
    for (i, snippet) in cases.iter().enumerate() {
        let mut input = new_input(snippet);
        let result = expr(&mut input);
        eprintln!("Case {}: {}", i, if result.is_ok() { "OK" } else { "FAIL" });
    }
}
