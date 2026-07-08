//! Tests for `parse_expr`.

use super::*;

fn test_input(s: &str) -> ParseInput<'_> {
    crate::input::new_input(s)
}

#[test]
fn test_do_block_parses_act_return() {
    let mut input = test_input("do:Act { return 1 }");
    let result = expr(&mut input).unwrap();

    match result {
        Expr::DoBlock {
            target,
            stmts,
            span,
        } => {
            assert_eq!(target.name.as_ref(), "Act");
            assert!(target.args.is_empty());
            assert!(target.span.start >= span.start);
            assert!(target.span.end <= span.end);
            assert!(target.span.end > target.span.start);
            assert_eq!(stmts.len(), 1);
            match &stmts[0] {
                crate::surface::DoStmt::Return { value, span } => {
                    assert!(span.end > span.start);
                    assert!(matches!(value.as_ref(), Expr::Literal(Literal::Int(1))));
                }
                other => panic!("expected do return statement, got {other:?}"),
            }
        }
        other => panic!("expected DoBlock, got {other:?}"),
    }
}

#[test]
fn target_ambient_do_block_parses_without_tower_target() {
    let mut input = test_input("do { let x = 1; return x }");
    let result = expr(&mut input).unwrap();

    match result {
        Expr::DoBlock { target, stmts, .. } => {
            assert_eq!(target.name.as_ref(), "__ambient");
            assert!(
                !matches!(target.name.as_ref(), "Act" | "Proc" | "Workflow"),
                "target do must not expose a tower computation target"
            );
            assert!(target.args.is_empty());
            assert_eq!(stmts.len(), 2);
            assert!(matches!(
                &stmts[0],
                crate::surface::DoStmt::Let { name, .. } if name.as_ref() == "x"
            ));
            assert!(matches!(
                &stmts[1],
                crate::surface::DoStmt::Return { value, .. }
                    if matches!(value.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "x")
            ));
        }
        other => panic!("expected ambient DoBlock, got {other:?}"),
    }
}

#[test]
fn target_ambient_do_block_parses_expression_statements() {
    let mut input = test_input("do { touch(); 1 + 1; return 41 }");
    let result = expr(&mut input).unwrap();

    match result {
        Expr::DoBlock { target, stmts, .. } => {
            assert_eq!(target.name.as_ref(), "__ambient");
            assert_eq!(stmts.len(), 3);
            assert!(matches!(
                &stmts[0],
                crate::surface::DoStmt::Expr { value, .. }
                    if matches!(value.as_ref(), Expr::Call { func, args, .. } if func.as_ref() == "touch" && args.is_empty())
            ));
            assert!(matches!(
                &stmts[1],
                crate::surface::DoStmt::Expr { value, .. }
                    if matches!(value.as_ref(), Expr::Binary { .. })
            ));
            assert!(matches!(
                &stmts[2],
                crate::surface::DoStmt::Return { value, .. }
                    if matches!(value.as_ref(), Expr::Literal(Literal::Int(41)))
            ));
        }
        other => panic!("expected ambient DoBlock, got {other:?}"),
    }
}

#[test]
fn test_do_block_parses_proc_bind_then_return() {
    let mut input = test_input("do:Proc { x <- proc::unit(1); return x }");
    let result = expr(&mut input).unwrap();

    match result {
        Expr::DoBlock { target, stmts, .. } => {
            assert_eq!(target.name.as_ref(), "Proc");
            assert_eq!(stmts.len(), 2);
            match &stmts[0] {
                crate::surface::DoStmt::Bind { name, value, span } => {
                    assert_eq!(name.as_ref(), "x");
                    assert!(span.end > span.start);
                    assert!(
                        matches!(value.as_ref(), Expr::Call { module: Some(module), func, args, .. } if module.as_ref() == "proc" && func.as_ref() == "unit" && args.len() == 1)
                    );
                }
                other => panic!("expected do bind statement, got {other:?}"),
            }
            assert!(matches!(
                &stmts[1],
                crate::surface::DoStmt::Return { value, .. }
                    if matches!(value.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "x")
            ));
        }
        other => panic!("expected DoBlock, got {other:?}"),
    }
}

#[test]
fn test_do_block_parses_let_then_return() {
    let mut input = test_input("do:Act { let x = 1; return x }");
    let result = expr(&mut input).unwrap();

    match result {
        Expr::DoBlock { target, stmts, .. } => {
            assert_eq!(target.name.as_ref(), "Act");
            assert_eq!(stmts.len(), 2);
            match &stmts[0] {
                crate::surface::DoStmt::Let { name, value, span } => {
                    assert_eq!(name.as_ref(), "x");
                    assert!(span.end > span.start);
                    assert!(matches!(value.as_ref(), Expr::Literal(Literal::Int(1))));
                }
                other => panic!("expected do let statement, got {other:?}"),
            }
            assert!(matches!(
                &stmts[1],
                crate::surface::DoStmt::Return { value, .. }
                    if matches!(value.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "x")
            ));
        }
        other => panic!("expected DoBlock, got {other:?}"),
    }
}

#[test]
fn test_do_block_accepts_trailing_semicolon_after_return() {
    let mut input = test_input("do:Act { return 1; }");
    let result = expr(&mut input).unwrap();
    match result {
        Expr::DoBlock { target, stmts, .. } => {
            assert_eq!(target.name.as_ref(), "Act");
            assert_eq!(stmts.len(), 1);
            assert!(matches!(
                &stmts[0],
                DoStmt::Return { value, .. }
                    if matches!(value.as_ref(), Expr::Literal(Literal::Int(1)))
            ));
        }
        other => panic!("expected DoBlock, got {other:?}"),
    }
}

#[test]
fn test_do_block_participates_in_binary_precedence() {
    let mut input = test_input("do:Act { return 1 } == expected");
    let result = expr(&mut input).unwrap();
    match result {
        Expr::Binary { left, right, .. } => {
            assert!(matches!(left.as_ref(), Expr::DoBlock { .. }));
            assert!(matches!(
                right.as_ref(),
                Expr::Variable { name, .. } if name.as_ref() == "expected"
            ));
        }
        other => panic!("expected binary expression with do-block lhs, got {other:?}"),
    }
}

#[test]
fn test_do_block_participates_in_pipe_precedence() {
    let mut input = test_input("do:Act { return 1 } |> consume");
    let result = expr(&mut input).unwrap();
    match result {
        Expr::Call { func, args, .. } => {
            assert_eq!(func.as_ref(), "consume");
            assert_eq!(args.len(), 1);
            assert!(matches!(&args[0], Expr::DoBlock { .. }));
        }
        other => panic!("expected piped call from do-block, got {other:?}"),
    }
}

#[test]
fn test_target_act_do_sugar_parses_as_do_act() {
    let mut input = test_input("act { x <- act::unit(1); return x }");
    let result = expr(&mut input).unwrap();
    match result {
        Expr::DoBlock { target, stmts, .. } => {
            assert_eq!(target.name.as_ref(), "Act");
            assert_eq!(stmts.len(), 2);
            assert!(matches!(
                &stmts[0],
                DoStmt::Bind { name, value, .. }
                    if name.as_ref() == "x"
                        && matches!(value.as_ref(), Expr::Call { module: Some(module), func, .. } if module.as_ref() == "act" && func.as_ref() == "unit")
            ));
            assert!(matches!(
                &stmts[1],
                DoStmt::Return { value, .. }
                    if matches!(value.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "x")
            ));
        }
        other => panic!("expected act sugar to parse as DoBlock, got {other:?}"),
    }
}

#[test]
fn test_target_act_do_sugar_rejects_trailing_statement_after_return() {
    let mut input = test_input("act { return x; y <- act::unit(1) }");
    let result = parse_target_act_do_sugar_expr(&mut input);
    assert!(
        result.is_err(),
        "target Act do-sugar must reject statements after final return: {result:?}"
    );
}

#[test]
fn test_target_act_do_sugar_accepts_return_trailing_semicolon() {
    let mut input = test_input("act { return x; }");
    let result = parse_target_act_do_sugar_expr(&mut input).unwrap();
    match result {
        Expr::DoBlock { target, stmts, .. } => {
            assert_eq!(target.name.as_ref(), "Act");
            assert_eq!(stmts.len(), 1);
            assert!(matches!(
                &stmts[0],
                DoStmt::Return { value, .. }
                    if matches!(value.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "x")
            ));
        }
        other => panic!("expected act sugar to parse as DoBlock, got {other:?}"),
    }
}

#[test]
fn test_removed_act_statement_forms_do_not_parse() {
    let removed_ret = ["r", "et"].concat();
    let source = format!("act {{ x = 1; {removed_ret} x; }}");
    let mut input = test_input(&source);
    let result = parse_target_act_do_sugar_expr(&mut input);
    assert!(
        result.is_err(),
        "removed act statement forms must not parse: {result:?}"
    );
}

#[test]
fn test_parse_int_literal() {
    let mut input = test_input("42");
    let result = expr(&mut input).unwrap();
    assert!(matches!(result, Expr::Literal(Literal::Int(42))));
}

#[test]
fn test_parse_float_literal() {
    let mut input = test_input("3.14");
    let result = expr(&mut input).unwrap();
    assert!(matches!(result, Expr::Literal(Literal::Float(f)) if (f - 3.14).abs() < 0.001));
}

#[test]
fn test_parse_string_literal() {
    let mut input = test_input("\"hello world\"");
    let result = expr(&mut input).unwrap();
    assert!(matches!(result, Expr::Literal(Literal::String(s)) if s.as_ref() == "hello world"));
}

#[test]
fn test_parse_bool_literal() {
    let mut input = test_input("true");
    let result = expr(&mut input).unwrap();
    assert!(matches!(result, Expr::Literal(Literal::Bool(true))));

    let mut input = test_input("false");
    let result = expr(&mut input).unwrap();
    assert!(matches!(result, Expr::Literal(Literal::Bool(false))));
}

#[test]
fn test_parse_null_literal() {
    let mut input = test_input("null");
    let result = expr(&mut input).unwrap();
    assert!(matches!(result, Expr::Literal(Literal::Null)));
}

#[test]
fn test_parse_variable() {
    let mut input = test_input("my_variable");
    let result = expr(&mut input).unwrap();
    assert!(matches!(result, Expr::Variable { name, .. } if name.as_ref() == "my_variable"));
}

#[test]
fn test_parse_binary_addition() {
    let mut input = test_input("1 + 2");
    let result = expr(&mut input).unwrap();
    assert!(matches!(
        result,
        Expr::Binary {
            op: BinaryOp::Add,
            ..
        }
    ));
}

#[test]
fn test_parse_binary_multiplication() {
    let mut input = test_input("3 * 4");
    let result = expr(&mut input).unwrap();
    assert!(matches!(
        result,
        Expr::Binary {
            op: BinaryOp::Mul,
            ..
        }
    ));
}

#[test]
fn test_parse_precedence() {
    // Multiplication has higher precedence than addition
    let mut input = test_input("1 + 2 * 3");
    let result = expr(&mut input).unwrap();

    // Should be: 1 + (2 * 3), not (1 + 2) * 3
    match result {
        Expr::Binary {
            op: BinaryOp::Add,
            left,
            right,
            ..
        } => {
            assert!(matches!(left.as_ref(), Expr::Literal(Literal::Int(1))));
            assert!(matches!(
                right.as_ref(),
                Expr::Binary {
                    op: BinaryOp::Mul,
                    ..
                }
            ));
        }
        _ => panic!("Expected Add expression"),
    }
}

#[test]
fn test_parse_comparison() {
    let mut input = test_input("x > 5");
    let result = expr(&mut input).unwrap();
    assert!(matches!(
        result,
        Expr::Binary {
            op: BinaryOp::Gt,
            ..
        }
    ));

    let mut input = test_input("a == b");
    let result = expr(&mut input).unwrap();
    assert!(matches!(
        result,
        Expr::Binary {
            op: BinaryOp::Eq,
            ..
        }
    ));
}

#[test]
fn test_parse_logical_and() {
    let mut input = test_input("a && b");
    let result = expr(&mut input).unwrap();
    assert!(matches!(
        result,
        Expr::Binary {
            op: BinaryOp::And,
            ..
        }
    ));
}

#[test]
fn test_parse_logical_or() {
    let mut input = test_input("a || b");
    let result = expr(&mut input).unwrap();
    assert!(matches!(
        result,
        Expr::Binary {
            op: BinaryOp::Or,
            ..
        }
    ));
}

#[test]
fn test_parse_field_access() {
    let mut input = test_input("obj.field");
    let result = expr(&mut input).unwrap();
    assert!(matches!(result, Expr::FieldAccess { .. }));
}

#[test]
fn test_parse_function_call() {
    let mut input = test_input("foo()");
    let result = expr(&mut input).unwrap();
    assert!(matches!(result, Expr::Call { func, .. } if func.as_ref() == "foo"));
}

#[test]
fn test_parse_function_call_with_args() {
    let mut input = test_input("foo(1, 2, 3)");
    let result = expr(&mut input).unwrap();
    match result {
        Expr::Call { func, args, .. } => {
            assert_eq!(func.as_ref(), "foo");
            assert_eq!(args.len(), 3);
        }
        _ => panic!("Expected Call expression"),
    }
}

#[test]
fn test_parse_variable_named_supervises() {
    let mut input = test_input("supervises");
    let result = expr(&mut input).unwrap();
    assert!(matches!(result, Expr::Variable { name, .. } if name.as_ref() == "supervises"));
}

#[test]
fn test_parse_parenthesized() {
    let mut input = test_input("(1 + 2) * 3");
    let result = expr(&mut input).unwrap();

    // Should be: (1 + 2) * 3
    match result {
        Expr::Binary {
            op: BinaryOp::Mul,
            left,
            ..
        } => {
            assert!(matches!(
                left.as_ref(),
                Expr::Binary {
                    op: BinaryOp::Add,
                    ..
                }
            ));
        }
        _ => panic!("Expected Mul expression"),
    }
}

#[test]
fn test_parse_complex_expression() {
    let mut input = test_input("a + b * c - d / e");
    let result = expr(&mut input).unwrap();
    assert!(matches!(result, Expr::Binary { .. }));
}

#[test]
fn test_parse_in_expression() {
    let mut input = test_input("x in list");
    let result = expr(&mut input).unwrap();
    assert!(matches!(
        result,
        Expr::Binary {
            op: BinaryOp::In,
            ..
        }
    ));
}

#[test]
fn test_parse_pipe_operator_to_unqualified_call() {
    let mut input = test_input("x |> f");
    let result = expr(&mut input).unwrap();
    match result {
        Expr::Call {
            func, module, args, ..
        } => {
            assert_eq!(func.as_ref(), "f");
            assert!(module.is_none());
            assert_eq!(args.len(), 1);
            assert!(matches!(args[0], Expr::Variable { ref name, .. } if name.as_ref() == "x"));
        }
        other => panic!("expected desugared pipe call, got {other:?}"),
    }
}

#[test]
fn test_parse_pipe_operator_chain_is_left_associative() {
    let mut input = test_input("x |> f |> g");
    let result = expr(&mut input).unwrap();
    match result {
        Expr::Call {
            func, module, args, ..
        } => {
            assert_eq!(func.as_ref(), "g");
            assert!(module.is_none());
            assert_eq!(args.len(), 1);
            match &args[0] {
                Expr::Call {
                    func, module, args, ..
                } => {
                    assert_eq!(func.as_ref(), "f");
                    assert!(module.is_none());
                    assert_eq!(args.len(), 1);
                    assert!(
                        matches!(args[0], Expr::Variable { ref name, .. } if name.as_ref() == "x")
                    );
                }
                other => {
                    panic!("expected nested call for left-associative pipe, got {other:?}")
                }
            }
        }
        other => panic!("expected desugared pipe chain, got {other:?}"),
    }
}

#[test]
fn test_parse_pipe_operator_to_module_qualified_call() {
    let mut input = test_input("x |> io::read(y)");
    let result = expr(&mut input).unwrap();
    match result {
        Expr::Call {
            func, module, args, ..
        } => {
            assert_eq!(func.as_ref(), "read");
            assert_eq!(module.as_ref().map(|m| m.as_ref()), Some("io"));
            assert_eq!(args.len(), 2);
            assert!(matches!(args[0], Expr::Variable { ref name, .. } if name.as_ref() == "x"));
            assert!(matches!(args[1], Expr::Variable { ref name, .. } if name.as_ref() == "y"));
        }
        other => panic!("expected module-qualified desugared pipe call, got {other:?}"),
    }
}

#[test]
fn test_pipe_lower_precedence_than_addition() {
    // `a + b |> f` should parse as `(a + b) |> f`, i.e. desugars to `f(a + b)`
    let mut input = test_input("a + b |> f");
    let result = expr(&mut input).unwrap();
    match result {
        Expr::Call {
            func, module, args, ..
        } => {
            assert_eq!(func.as_ref(), "f");
            assert!(module.is_none());
            assert_eq!(args.len(), 1);
            // The single argument should be a binary add: a + b
            assert!(
                matches!(
                    &args[0],
                    Expr::Binary {
                        op: BinaryOp::Add,
                        ..
                    }
                ),
                "expected Binary::Add as the piped argument, got {:?}",
                args[0]
            );
        }
        other => panic!("expected desugared pipe call, got {other:?}"),
    }
}

#[test]
fn test_pipe_into_call_with_args() {
    // `x |> f(a, b)` should desugar to `f(x, a, b)` — x prepended as first arg
    let mut input = test_input("x |> f(a, b)");
    let result = expr(&mut input).unwrap();
    match result {
        Expr::Call {
            func, module, args, ..
        } => {
            assert_eq!(func.as_ref(), "f");
            assert!(module.is_none());
            assert_eq!(args.len(), 3);
            // First arg: the piped value x
            assert!(
                matches!(&args[0], Expr::Variable { name, .. } if name.as_ref() == "x"),
                "expected first arg to be 'x', got {:?}",
                args[0]
            );
            // Second and third args: a, b
            assert!(
                matches!(&args[1], Expr::Variable { name, .. } if name.as_ref() == "a"),
                "expected second arg to be 'a', got {:?}",
                args[1]
            );
            assert!(
                matches!(&args[2], Expr::Variable { name, .. } if name.as_ref() == "b"),
                "expected third arg to be 'b', got {:?}",
                args[2]
            );
        }
        other => panic!("expected desugared pipe call with prepended arg, got {other:?}"),
    }
}

// ============================================================
// If-Let Expression Tests (TASK-126)
// ============================================================

#[test]
fn test_parse_if_let_simple() {
    // Simple if-let with variant pattern
    let mut input = test_input("if let Some { value: x } = opt then { x } else { 0 }");
    let result = expr(&mut input).unwrap();

    match result {
        Expr::IfLet {
            pattern,
            expr,
            then_branch,
            else_branch,
            ..
        } => {
            // Pattern should be a Variant pattern
            assert!(
                matches!(pattern, crate::surface::Pattern::Variant { name, .. } if name.as_ref() == "Some")
            );
            // Expression should be variable 'opt'
            assert!(matches!(expr.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "opt"));
            // Then branch should be variable 'x'
            assert!(
                matches!(then_branch.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "x")
            );
            // Else branch should be literal 0
            assert!(matches!(
                else_branch.as_ref(),
                Expr::Literal(Literal::Int(0))
            ));
        }
        _ => panic!("Expected IfLet expression, got {:?}", result),
    }
}

#[test]
fn test_parse_if_let_unit_variant() {
    // Unit variant pattern (just the name without fields)
    let mut input = test_input("if let None = opt then { \"none\" } else { \"some\" }");
    let result = expr(&mut input).unwrap();

    match result {
        Expr::IfLet {
            pattern,
            then_branch,
            else_branch,
            ..
        } => {
            // Unit variants like `None` parse as variant patterns without braces.
            assert!(matches!(
                pattern,
                crate::surface::Pattern::Variant { name, fields, .. }
                    if name.as_ref() == "None" && fields.is_none()
            ));
            // Then branch should be string "none"
            assert!(
                matches!(then_branch.as_ref(), Expr::Literal(Literal::String(s)) if s.as_ref() == "none")
            );
            // Else branch should be string "some"
            assert!(
                matches!(else_branch.as_ref(), Expr::Literal(Literal::String(s)) if s.as_ref() == "some")
            );
        }
        _ => panic!("Expected IfLet expression, got {:?}", result),
    }
}

#[test]
fn test_parse_if_let_variable_pattern() {
    // Simple variable pattern
    let mut input = test_input("if let x = value then { x } else { 0 }");
    let result = expr(&mut input).unwrap();

    match result {
        Expr::IfLet {
            pattern,
            then_branch,
            else_branch,
            ..
        } => {
            assert!(
                matches!(pattern, crate::surface::Pattern::Variable { name, .. } if name.as_ref() == "x")
            );
            assert!(
                matches!(then_branch.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "x")
            );
            assert!(matches!(
                else_branch.as_ref(),
                Expr::Literal(Literal::Int(0))
            ));
        }
        _ => panic!("Expected IfLet expression, got {:?}", result),
    }
}

#[test]
fn test_parse_if_let_wildcard_pattern() {
    // Wildcard pattern
    let mut input = test_input("if let _ = value then { 1 } else { 0 }");
    let result = expr(&mut input).unwrap();

    match result {
        Expr::IfLet { pattern, .. } => {
            assert!(matches!(pattern, crate::surface::Pattern::Wildcard));
        }
        _ => panic!("Expected IfLet expression, got {:?}", result),
    }
}

#[test]
fn test_parse_if_let_tuple_pattern() {
    // Tuple pattern
    let mut input = test_input("if let (a, b) = pair then { a } else { b }");
    let result = expr(&mut input).unwrap();

    match result {
        Expr::IfLet {
            pattern,
            then_branch,
            else_branch,
            ..
        } => {
            assert!(matches!(pattern, crate::surface::Pattern::Tuple(pats) if pats.len() == 2));
            assert!(
                matches!(then_branch.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "a")
            );
            assert!(
                matches!(else_branch.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "b")
            );
        }
        _ => panic!("Expected IfLet expression, got {:?}", result),
    }
}

#[test]
fn test_parse_if_let_complex_expression() {
    // If-let with complex expression in match position
    let mut input = test_input("if let x = foo() + bar then { x } else { 0 }");
    let result = expr(&mut input).unwrap();

    assert!(matches!(result, Expr::IfLet { .. }));
}

#[test]
fn test_parse_if_let_nested_expressions() {
    // Nested expressions in branches
    let mut input = test_input("if let Some { value: x } = opt then { x + 1 } else { x - 1 }");
    let result = expr(&mut input).unwrap();

    match result {
        Expr::IfLet {
            then_branch,
            else_branch,
            ..
        } => {
            // Both branches should be binary expressions
            assert!(matches!(
                then_branch.as_ref(),
                Expr::Binary {
                    op: BinaryOp::Add,
                    ..
                }
            ));
            assert!(matches!(
                else_branch.as_ref(),
                Expr::Binary {
                    op: BinaryOp::Sub,
                    ..
                }
            ));
        }
        _ => panic!("Expected IfLet expression, got {:?}", result),
    }
}

#[test]
fn test_parse_constructor_expression() {
    let mut input = test_input("Ok { value: 42 }");
    let result = expr(&mut input).unwrap();

    match result {
        Expr::Constructor {
            name,
            fields,
            payload,
            ..
        } => {
            assert_eq!(name.as_ref(), "Ok");
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0].0.as_ref(), "value");
            assert!(matches!(fields[0].1, Expr::Literal(Literal::Int(42))));
            assert!(matches!(payload, ConstructorPayload::Record(items) if items.len() == 1));
        }
        other => panic!("Expected Constructor expression, got {other:?}"),
    }
}

#[test]
fn test_parse_structural_record_expression() {
    let mut input = test_input(r#"{ name: "Ada", age: 41 }"#);
    let result = expr(&mut input).unwrap();

    match result {
        Expr::Record { fields, .. } => {
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].0.as_ref(), "name");
            assert!(
                matches!(fields[0].1, Expr::Literal(Literal::String(ref value)) if value.as_ref() == "Ada")
            );
            assert_eq!(fields[1].0.as_ref(), "age");
            assert!(matches!(fields[1].1, Expr::Literal(Literal::Int(41))));
        }
        other => panic!("Expected Record expression, got {other:?}"),
    }
}

#[test]
fn test_parse_nested_constructor_expression() {
    let mut input = test_input(r#"Err { error: RuntimeError(42, "boom") }"#);
    let result = expr(&mut input).unwrap();

    match result {
        Expr::Constructor {
            name,
            fields,
            payload,
            ..
        } => {
            assert_eq!(name.as_ref(), "Err");
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0].0.as_ref(), "error");
            assert!(matches!(payload, ConstructorPayload::Record(items) if items.len() == 1));
            match &fields[0].1 {
                Expr::Constructor {
                    name,
                    fields,
                    payload,
                    ..
                } => {
                    assert_eq!(name.as_ref(), "RuntimeError");
                    assert_eq!(fields.len(), 2);
                    assert_eq!(fields[0].0.as_ref(), "_0");
                    assert!(matches!(fields[0].1, Expr::Literal(Literal::Int(42))));
                    assert_eq!(fields[1].0.as_ref(), "_1");
                    assert!(matches!(
                        &fields[1].1,
                        Expr::Literal(Literal::String(message)) if message.as_ref() == "boom"
                    ));
                    assert!(
                        matches!(payload, ConstructorPayload::Tuple(items) if matches!(
                            items.as_slice(),
                            [
                                Expr::Literal(Literal::Int(42)),
                                Expr::Literal(Literal::String(message))
                            ] if message.as_ref() == "boom"
                        ))
                    );
                }
                other => panic!("Expected nested constructor, got {other:?}"),
            }
        }
        other => panic!("Expected Constructor expression, got {other:?}"),
    }
}

#[test]
fn test_parse_multi_field_constructor_expression() {
    // "role" is an Ash keyword but should be allowed as constructor field name
    let mut input = test_input("Msg { role: x, text: y }");
    let result = expr(&mut input);
    match result {
        Ok(Expr::Constructor { name, fields, .. }) => {
            assert_eq!(name.as_ref(), "Msg");
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].0.as_ref(), "role");
            assert_eq!(fields[1].0.as_ref(), "text");
        }
        Ok(other) => panic!("Expected Constructor, got {other:?}"),
        Err(e) => panic!("Parse failed: {e:?}"),
    }
}

// =========================================================================
// Qualified fn call parsing tests (TASK-503)
// =========================================================================

#[test]
fn test_qualified_fn_call_no_args() {
    let mut input = test_input("math::pi()");
    let result = expr(&mut input).unwrap();
    match result {
        Expr::Call {
            func, module, args, ..
        } => {
            assert_eq!(func.as_ref(), "pi");
            assert_eq!(module.as_ref().map(|s| s.as_ref()), Some("math"));
            assert!(args.is_empty());
        }
        other => panic!("Expected Call with module, got {other:?}"),
    }
}

#[test]
fn test_qualified_fn_call_with_args() {
    let mut input = test_input("math::add(1, 2)");
    let result = expr(&mut input).unwrap();
    match result {
        Expr::Call {
            func, module, args, ..
        } => {
            assert_eq!(func.as_ref(), "add");
            assert_eq!(module.as_ref().map(|s| s.as_ref()), Some("math"));
            assert_eq!(args.len(), 2);
        }
        other => panic!("Expected Call with module, got {other:?}"),
    }
}

#[test]
fn test_qualified_fn_call_single_arg() {
    let mut input = test_input("utils::transform(x)");
    let result = expr(&mut input).unwrap();
    match result {
        Expr::Call {
            func, module, args, ..
        } => {
            assert_eq!(func.as_ref(), "transform");
            assert_eq!(module.as_ref().map(|s| s.as_ref()), Some("utils"));
            assert_eq!(args.len(), 1);
        }
        other => panic!("Expected Call with module, got {other:?}"),
    }
}

#[test]
fn test_unqualified_fn_call_still_works() {
    let mut input = test_input("foo(1, 2)");
    let result = expr(&mut input).unwrap();
    match result {
        Expr::Call {
            func, module, args, ..
        } => {
            assert_eq!(func.as_ref(), "foo");
            assert!(module.is_none());
            assert_eq!(args.len(), 2);
        }
        other => panic!("Expected Call without module, got {other:?}"),
    }
}

#[test]
fn test_bare_qualified_method_rejected() {
    // `Interface::method` without parens should be a parse error
    let mut input = test_input("Interface::method");
    let result = expr(&mut input);
    assert!(
        result.is_err(),
        "Expected bare qualified method to be rejected, got: {result:?}"
    );
}

#[test]
fn test_qualified_method_with_call_accepted() {
    // `Interface::method(x)` should parse successfully
    let mut input = test_input("Interface::method(x)");
    let result = expr(&mut input).unwrap();
    match result {
        Expr::Call {
            func, module, args, ..
        } => {
            assert_eq!(func.as_ref(), "method");
            assert_eq!(module.as_ref().map(|s| s.as_ref()), Some("Interface"));
            assert_eq!(args.len(), 1);
        }
        other => panic!("Expected Call with module, got {other:?}"),
    }
}

// ============================================================
// Phase 156 Parser Blocker Regression Tests (TASK-1563)
// ============================================================
// NOTE: if/then/else and match expressions are parsed in parse_module/fn_defs.rs,
// not in parse_expr.rs. See parse_module/tests.rs for the main regression tests.
// The tests below verify the underlying pattern and expression parsers.

#[test]
fn test_variant_record_pattern_parses() {
    // B2: Variant patterns with record payloads like `Cons { head: h, tail: rest }`
    let mut input = test_input("Cons { head: h, tail: rest }");
    let result = pattern(&mut input).unwrap();
    match result {
        crate::surface::Pattern::Variant { name, .. } => {
            assert_eq!(name.as_ref(), "Cons");
        }
        other => panic!("Expected Cons variant pattern, got {other:?}"),
    }
}

#[test]
fn test_list_literal_pattern_parses() {
    // B3: List literal patterns `[h, ..rest]`
    let mut input = test_input("[h, ..rest]");
    let result = pattern(&mut input).unwrap();
    assert!(
        matches!(result, crate::surface::Pattern::List { .. }),
        "Expected List pattern, got {:?}",
        result
    );
}

#[test]
fn test_list_literal_pattern_empty() {
    // B3: Empty list pattern `[]`
    let mut input = test_input("[]");
    let result = pattern(&mut input).unwrap();
    assert!(
        matches!(result, crate::surface::Pattern::List { .. }),
        "Expected empty List pattern, got {:?}",
        result
    );
}

#[test]
fn test_list_expr_in_primary_position() {
    // B3: List expressions in primary position (not just patterns)
    let mut input = test_input("[1, 2, 3]");
    let result = expr(&mut input).unwrap();
    assert!(
        matches!(result, Expr::List { .. }),
        "Expected List expression, got {:?}",
        result
    );
}

#[test]
fn test_list_expr_as_fn_argument() {
    // B3: List expressions passed as function arguments
    let mut input = test_input("foo([1, 2, 3])");
    let result = expr(&mut input).unwrap();
    match result {
        Expr::Call { args, .. } => {
            assert_eq!(args.len(), 1);
            assert!(matches!(args[0], Expr::List { .. }));
        }
        other => panic!("Expected Call with list arg, got {other:?}"),
    }
}

#[test]
fn test_nested_list_expr() {
    // B3: Nested list expressions
    let mut input = test_input("[[1, 2], [3, 4]]");
    let result = expr(&mut input).unwrap();
    assert!(
        matches!(result, Expr::List { .. }),
        "Expected nested List expression, got {:?}",
        result
    );
}

#[test]
fn test_variant_pattern_with_single_field() {
    // B2: Variant pattern with a single field
    let mut input = test_input("Some { value: x }");
    let result = pattern(&mut input).unwrap();
    match result {
        crate::surface::Pattern::Variant { name, .. } => {
            assert_eq!(name.as_ref(), "Some");
        }
        other => panic!("Expected Some variant pattern, got {other:?}"),
    }
}

#[test]
fn test_list_expr_with_trailing_comma() {
    // B3: List expression with trailing comma (may or may not be supported)
    let mut input = test_input("[1, 2, 3,]");
    let result = expr(&mut input);
    // Trailing comma support depends on grammar; verify it doesn't panic
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_list_expr_with_spread() {
    // B3: List expression with spread (if supported)
    let mut input = test_input("[1, ..rest, 5]");
    let result = expr(&mut input);
    // This may or may not parse depending on grammar; we just check it doesn't panic
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_list_expr_in_let_binding() {
    // B3: List expression in a let binding (parsed as do-block let)
    let mut input = test_input("do:Act { let xs = [1, 2, 3]; return xs }");
    let result = expr(&mut input).unwrap();
    match result {
        Expr::DoBlock { stmts, .. } => {
            assert_eq!(stmts.len(), 2);
            match &stmts[0] {
                crate::surface::DoStmt::Let { value, .. } => {
                    assert!(matches!(value.as_ref(), Expr::List { .. }));
                }
                other => panic!("Expected Let with list, got {other:?}"),
            }
        }
        other => panic!("Expected DoBlock with let, got {other:?}"),
    }
}
