use super::support::*;

#[test]
fn dispatch_table_contains_act_guard() {
    let table = builtin_dispatch_table();
    let entry = table
        .get("act::__guard")
        .expect("act::__guard should be in the dispatch table");
    assert_eq!(entry.arity, 2);
    assert!(entry.implemented);
}

#[test]
fn is_known_builtin_qualified_act_guard() {
    assert!(is_known_builtin("__guard", Some("act")));
}

// ── TASK-621: dispatch_builtin (implemented path) ────────────────

#[test]
fn dispatch_builtin_act_guard_returns_closure() {
    let ctx = Context::new();
    let guarded = eval_expr(
        &Expr::Call {
            func: "__guard".to_string(),
            module: Some("act".to_string()),
            arguments: vec![
                Expr::Literal(Value::String("policy".to_string())),
                Expr::Call {
                    func: "unit".to_string(),
                    module: None,
                    arguments: vec![Expr::Literal(Value::Int(7))],
                },
            ],
        },
        &ctx,
    )
    .expect("act::__guard should dispatch through builtin metadata");

    assert!(matches!(guarded, Value::Closure { .. }));
}

#[test]
fn dispatch_builtin_act_guard_wrong_arity_is_reported() {
    let ctx = Context::new();
    let error = dispatch_builtin("act::__guard", &[Value::String("policy".to_string())], &ctx)
        .expect("dispatch should find act::__guard")
        .expect_err("act::__guard should reject malformed arity");

    assert!(matches!(
        error,
        EvalError::WrongArity {
            expected: 2,
            actual: 1,
            callee: Some(ref callee),
        } if callee == "act::__guard"
    ));
}

// ── TASK-622: UnimplementedBuiltin error ──────────────────────────
