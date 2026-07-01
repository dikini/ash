use super::support::*;

#[test]
fn dispatch_table_contains_qualified_list_builtins() {
    let table = builtin_dispatch_table();
    let expected = [
        ("list::len", 1),
        ("list::head", 1),
        ("list::tail", 1),
        ("list::append", 2),
        ("list::concat", 2),
        ("list::filter", 2),
        ("list::map", 2),
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
fn is_known_builtin_qualified_list_len() {
    assert!(is_known_builtin("len", Some("list")));
}

#[test]
fn is_known_builtin_qualified_list_append() {
    assert!(is_known_builtin("append", Some("list")));
}

#[test]
fn is_known_builtin_qualified_list_map() {
    assert!(is_known_builtin("map", Some("list")));
}

#[test]
fn dispatch_builtin_qualified_list_len_returns_correct_value() {
    let ctx = Context::new();
    let args = vec![Value::list_from_vec(vec![
        Value::Int(1),
        Value::Int(2),
        Value::Int(3),
    ])];
    let result = dispatch_builtin("list::len", &args, &ctx)
        .expect("dispatch should find list::len")
        .expect("list::len should succeed");
    assert_eq!(result, Value::Int(3));
}

#[test]
fn dispatch_builtin_qualified_list_append_returns_correct_value() {
    let ctx = Context::new();
    let args = vec![
        Value::list_from_vec(vec![Value::Int(1), Value::Int(2)]),
        Value::Int(3),
    ];
    let result = dispatch_builtin("list::append", &args, &ctx)
        .expect("dispatch should find list::append")
        .expect("list::append should succeed");
    assert_eq!(
        result,
        Value::list_from_vec(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
    );
}

#[test]
fn dispatch_builtin_qualified_list_concat_returns_correct_value() {
    let ctx = Context::new();
    let args = vec![
        Value::list_from_vec(vec![Value::Int(1), Value::Int(2)]),
        Value::list_from_vec(vec![Value::Int(3), Value::Int(4)]),
    ];
    let result = dispatch_builtin("list::concat", &args, &ctx)
        .expect("dispatch should find list::concat")
        .expect("list::concat should succeed");
    assert_eq!(
        result,
        Value::list_from_vec(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4)
        ])
    );
}

#[test]
fn dispatch_builtin_qualified_list_head_returns_correct_value() {
    let ctx = Context::new();
    let args = vec![Value::list_from_vec(vec![Value::Int(42), Value::Int(7)])];
    let result = dispatch_builtin("list::head", &args, &ctx)
        .expect("dispatch should find list::head")
        .expect("list::head should succeed");
    assert_eq!(result, Value::Int(42));
}

#[test]
fn dispatch_builtin_qualified_list_tail_returns_correct_value() {
    let ctx = Context::new();
    let args = vec![Value::list_from_vec(vec![
        Value::Int(1),
        Value::Int(2),
        Value::Int(3),
    ])];
    let result = dispatch_builtin("list::tail", &args, &ctx)
        .expect("dispatch should find list::tail")
        .expect("list::tail should succeed");
    assert_eq!(
        result,
        Value::list_from_vec(vec![Value::Int(2), Value::Int(3)])
    );
}

#[test]
fn eval_function_call_qualified_list_len_via_expr() {
    let ctx = Context::new();
    let expr = Expr::Call {
        func: "len".to_string(),
        module: Some("list".to_string()),
        arguments: vec![Expr::Literal(Value::list_from_vec(vec![
            Value::Int(10),
            Value::Int(20),
            Value::Int(30),
        ]))],
    };
    let result = eval_expr(&expr, &ctx).expect("list::len should succeed");
    assert_eq!(result, Value::Int(3));
}

#[test]
fn eval_function_call_qualified_list_append_via_expr() {
    let ctx = Context::new();
    let expr = Expr::Call {
        func: "append".to_string(),
        module: Some("list".to_string()),
        arguments: vec![
            Expr::Literal(Value::list_from_vec(vec![Value::Int(1)])),
            Expr::Literal(Value::Int(2)),
        ],
    };
    let result = eval_expr(&expr, &ctx).expect("list::append should succeed");
    assert_eq!(
        result,
        Value::list_from_vec(vec![Value::Int(1), Value::Int(2)])
    );
}

#[test]
fn eval_function_call_qualified_list_concat_via_expr() {
    let ctx = Context::new();
    let expr = Expr::Call {
        func: "concat".to_string(),
        module: Some("list".to_string()),
        arguments: vec![
            Expr::Literal(Value::list_from_vec(vec![Value::Int(1)])),
            Expr::Literal(Value::list_from_vec(vec![Value::Int(2)])),
        ],
    };
    let result = eval_expr(&expr, &ctx).expect("list::concat should succeed");
    assert_eq!(
        result,
        Value::list_from_vec(vec![Value::Int(1), Value::Int(2)])
    );
}

#[test]
fn dispatch_builtin_qualified_predicate_is_list_returns_true() {
    let ctx = Context::new();
    let args = vec![Value::list_from_vec(vec![Value::Int(1), Value::Int(2)])];
    let result = dispatch_builtin("predicate::is_list", &args, &ctx)
        .expect("dispatch should find predicate::is_list")
        .expect("predicate::is_list should succeed");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn eval_function_call_qualified_predicate_is_list_via_expr() {
    let ctx = Context::new();
    let expr = Expr::Call {
        func: "is_list".to_string(),
        module: Some("predicate".to_string()),
        arguments: vec![Expr::Literal(Value::list_from_vec(vec![
            Value::Int(1),
            Value::Int(2),
        ]))],
    };
    let result = eval_expr(&expr, &ctx).expect("predicate::is_list should succeed");
    assert_eq!(result, Value::Bool(true));
}
