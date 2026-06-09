use super::support::*;

#[test]
fn dispatch_table_contains_qualified_predicate_builtins() {
    let table = builtin_dispatch_table();
    let expected = [
        ("predicate::is_int", 1),
        ("predicate::is_string", 1),
        ("predicate::is_bool", 1),
        ("predicate::is_list", 1),
        ("predicate::is_record", 1),
        ("predicate::is_null", 1),
    ];
    for (name, arity) in &expected {
        let entry = table
            .get(name)
            .unwrap_or_else(|| panic!("{name} should be in the dispatch table"));
        assert_eq!(entry.arity, *arity, "{name} arity should be {arity}");
        assert!(entry.implemented, "{name} should be implemented");
    }
}

#[test]
fn is_known_builtin_qualified_predicate_is_int() {
    assert!(is_known_builtin("is_int", Some("predicate")));
}

#[test]
fn is_known_builtin_qualified_predicate_is_string() {
    assert!(is_known_builtin("is_string", Some("predicate")));
}

#[test]
fn dispatch_builtin_qualified_predicate_is_int_returns_true() {
    let ctx = Context::new();
    let args = vec![Value::Int(42)];
    let result = dispatch_builtin("predicate::is_int", &args, &ctx)
        .expect("dispatch should find predicate::is_int")
        .expect("predicate::is_int should succeed");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn dispatch_builtin_qualified_predicate_is_int_returns_false_for_string() {
    let ctx = Context::new();
    let args = vec![Value::String("hello".into())];
    let result = dispatch_builtin("predicate::is_int", &args, &ctx)
        .expect("dispatch should find predicate::is_int")
        .expect("predicate::is_int should succeed");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn dispatch_builtin_qualified_predicate_is_string_returns_true() {
    let ctx = Context::new();
    let args = vec![Value::String("hello".into())];
    let result = dispatch_builtin("predicate::is_string", &args, &ctx)
        .expect("dispatch should find predicate::is_string")
        .expect("predicate::is_string should succeed");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn dispatch_builtin_qualified_predicate_is_bool_returns_true() {
    let ctx = Context::new();
    let args = vec![Value::Bool(true)];
    let result = dispatch_builtin("predicate::is_bool", &args, &ctx)
        .expect("dispatch should find predicate::is_bool")
        .expect("predicate::is_bool should succeed");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn dispatch_builtin_qualified_predicate_is_null_returns_true() {
    let ctx = Context::new();
    let args = vec![Value::Null];
    let result = dispatch_builtin("predicate::is_null", &args, &ctx)
        .expect("dispatch should find predicate::is_null")
        .expect("predicate::is_null should succeed");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn dispatch_builtin_qualified_predicate_is_record_returns_true() {
    let ctx = Context::new();
    let args = vec![Value::Record(Box::default())];
    let result = dispatch_builtin("predicate::is_record", &args, &ctx)
        .expect("dispatch should find predicate::is_record")
        .expect("predicate::is_record should succeed");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn dispatch_builtin_unqualified_is_int_still_works() {
    let ctx = Context::new();
    let args = vec![Value::Int(42)];
    let result = dispatch_builtin("is_int", &args, &ctx)
        .expect("dispatch should find unqualified is_int")
        .expect("unqualified is_int should succeed");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn eval_function_call_qualified_predicate_is_int_via_expr() {
    let ctx = Context::new();
    let expr = Expr::Call {
        func: "is_int".to_string(),
        module: Some("predicate".to_string()),
        arguments: vec![Expr::Literal(Value::Int(42))],
    };
    let result = eval_expr(&expr, &ctx).expect("predicate::is_int should succeed");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn eval_function_call_qualified_predicate_is_int_rejects_string() {
    let ctx = Context::new();
    let expr = Expr::Call {
        func: "is_int".to_string(),
        module: Some("predicate".to_string()),
        arguments: vec![Expr::Literal(Value::String("hello".into()))],
    };
    let result = eval_expr(&expr, &ctx).expect("predicate::is_int should succeed");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn eval_function_call_qualified_predicate_is_string_via_expr() {
    let ctx = Context::new();
    let expr = Expr::Call {
        func: "is_string".to_string(),
        module: Some("predicate".to_string()),
        arguments: vec![Expr::Literal(Value::String("hello".into()))],
    };
    let result = eval_expr(&expr, &ctx).expect("predicate::is_string should succeed");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn eval_function_call_qualified_predicate_is_null_via_expr() {
    let ctx = Context::new();
    let expr = Expr::Call {
        func: "is_null".to_string(),
        module: Some("predicate".to_string()),
        arguments: vec![Expr::Literal(Value::Null)],
    };
    let result = eval_expr(&expr, &ctx).expect("predicate::is_null should succeed");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn eval_function_call_unqualified_is_int_still_works_via_expr() {
    let ctx = Context::new();
    let expr = Expr::Call {
        func: "is_int".to_string(),
        module: None,
        arguments: vec![Expr::Literal(Value::Int(42))],
    };
    let result = eval_expr(&expr, &ctx).expect("unqualified is_int should succeed");
    assert_eq!(result, Value::Bool(true));
}
