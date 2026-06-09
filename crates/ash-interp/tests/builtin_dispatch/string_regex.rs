use super::support::*;

#[test]
fn dispatch_table_contains_string_concat() {
    let table = builtin_dispatch_table();
    let entry = table
        .get("string::concat")
        .expect("string::concat should be in the dispatch table");
    assert!(entry.variadic);
    assert!(entry.implemented);
}

#[test]
fn dispatch_table_contains_string_starts_with() {
    let table = builtin_dispatch_table();
    let entry = table
        .get("string::starts_with")
        .expect("string::starts_with should be in the dispatch table");
    assert_eq!(entry.arity, 2);
    assert!(entry.implemented);
}

#[test]
fn dispatch_table_contains_string_ends_with() {
    let table = builtin_dispatch_table();
    assert!(builtin_dispatch_table().contains_key("string::ends_with"));
    let entry = table.get("string::ends_with").unwrap();
    assert_eq!(entry.arity, 2);
    assert!(entry.implemented);
}

#[test]
fn dispatch_table_contains_string_is_empty() {
    let table = builtin_dispatch_table();
    let entry = table
        .get("string::is_empty")
        .expect("string::is_empty should be in the dispatch table");
    assert_eq!(entry.arity, 1);
    assert!(entry.implemented);
}

#[test]
fn dispatch_table_contains_regex_builtins() {
    let table = builtin_dispatch_table();
    for (name, arity) in [
        ("regex::find", 2),
        ("regex::matches", 2),
        ("regex::replace", 3),
    ] {
        let entry = table
            .get(name)
            .unwrap_or_else(|| panic!("{name} should be in the dispatch table"));
        assert_eq!(entry.arity, arity, "{name} arity should match");
        assert!(entry.implemented, "{name} should be implemented");
    }
}

#[test]
fn dispatch_table_string_case_and_whitespace_builtins_are_implemented() {
    let table = builtin_dispatch_table();
    for name in &["string::to_upper", "string::to_lower", "string::trim"] {
        let entry = table
            .get(name)
            .unwrap_or_else(|| panic!("{name} should be in the dispatch table"));
        assert!(
            entry.implemented,
            "{name} should be marked implemented because eval_function_call handles it"
        );
    }
}

#[test]
fn is_known_builtin_qualified_string_concat() {
    assert!(is_known_builtin("concat", Some("string")));
}

#[test]
fn is_known_builtin_qualified_string_starts_with() {
    assert!(is_known_builtin("starts_with", Some("string")));
}

#[test]
fn is_known_builtin_qualified_regex_find() {
    assert!(is_known_builtin("find", Some("regex")));
}

#[test]
fn dispatch_builtin_string_concat_returns_correct_value() {
    let ctx = Context::new();
    let args = vec![
        Value::String("hello ".into()),
        Value::String("world".into()),
    ];
    let result = dispatch_builtin("string::concat", &args, &ctx)
        .expect("dispatch should find string::concat")
        .expect("string::concat should succeed");
    assert_eq!(result, Value::String("hello world".to_string()));
}

#[test]
fn dispatch_builtin_regex_find_returns_first_match() {
    let ctx = Context::new();
    let args = vec![Value::String("a+".into()), Value::String("baaac".into())];
    let result = dispatch_builtin("regex::find", &args, &ctx)
        .expect("dispatch should find regex::find")
        .expect("regex::find should succeed");
    assert_eq!(
        result,
        Value::Variant {
            name: "Some".to_string(),
            fields: Box::new(vec![(
                "value".to_string(),
                Value::String("aaa".to_string())
            )]),
        }
    );
}

#[test]
fn eval_function_call_regex_matches_via_expr_succeeds() {
    let ctx = Context::new();
    let expr = Expr::Call {
        func: "matches".to_string(),
        module: Some("regex".to_string()),
        arguments: vec![
            Expr::Literal(Value::String(r"\d+".into())),
            Expr::Literal(Value::String("abc123def".into())),
        ],
    };

    let result = eval_expr(&expr, &ctx).expect("regex::matches should succeed");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn eval_function_call_regex_replace_via_expr_succeeds() {
    let ctx = Context::new();
    let expr = Expr::Call {
        func: "replace".to_string(),
        module: Some("regex".to_string()),
        arguments: vec![
            Expr::Literal(Value::String(r"\d+".into())),
            Expr::Literal(Value::String("#".into())),
            Expr::Literal(Value::String("abc123def456".into())),
        ],
    };

    let result = eval_expr(&expr, &ctx).expect("regex::replace should succeed");
    assert_eq!(result, Value::String("abc#def#".to_string()));
}

#[test]
fn eval_function_call_regex_invalid_pattern_is_clear() {
    let ctx = Context::new();
    let expr = Expr::Call {
        func: "find".to_string(),
        module: Some("regex".to_string()),
        arguments: vec![
            Expr::Literal(Value::String("(".into())),
            Expr::Literal(Value::String("abc".into())),
        ],
    };

    let err = eval_expr(&expr, &ctx).expect_err("invalid regex should fail honestly");
    let msg = err.to_string();
    assert!(
        msg.contains("Invalid regex pattern"),
        "error message should mention invalid regex pattern, got: {msg}"
    );
}

#[test]
fn eval_function_call_starts_with_via_existing_path() {
    let ctx = Context::new();
    // string::starts_with("hello", "he") — implemented
    let expr = Expr::Call {
        func: "starts_with".to_string(),
        module: Some("string".to_string()),
        arguments: vec![
            Expr::Literal(Value::String("hello".into())),
            Expr::Literal(Value::String("he".into())),
        ],
    };
    let result = eval_expr(&expr, &ctx).expect("string::starts_with should succeed");
    assert_eq!(result, Value::Bool(true));
}
