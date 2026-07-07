use super::support::*;

#[test]
fn dispatch_table_contains_unqualified_builtins() {
    let table = builtin_dispatch_table();
    let expected_unqualified = [
        "len",
        "head",
        "tail",
        "append",
        "concat",
        "filter",
        "map",
        "starts_with",
        "ends_with",
        "keys",
        "values",
        "is_int",
        "is_string",
        "is_bool",
        "is_list",
        "is_record",
        "is_null",
        "record",
    ];
    for name in &expected_unqualified {
        assert!(
            table.contains_key(name),
            "unqualified builtin '{name}' should be in the dispatch table"
        );
    }
}

#[test]
fn stdlib_pub_builtin_declarations_have_honest_dispatch_entries() {
    let table = builtin_dispatch_table();
    let mut declarations = Vec::new();
    collect_stdlib_builtin_declarations(&stdlib_root(), &mut declarations);
    declarations.sort();

    let missing: Vec<_> = declarations
        .iter()
        .filter(|name| !table.contains_key(name.as_str()))
        .cloned()
        .collect();

    assert!(
        missing.is_empty(),
        "every stdlib `pub builtin fn` must have a dispatch-table entry; missing {missing:?}"
    );
}

#[test]
fn provider_backed_stdlib_builtins_are_forward_declared_not_implemented() {
    let table = builtin_dispatch_table();
    for name in [
        "time::now",
        "time::now_iso",
        "time::epoch_millis",
        "time::sleep",
        "io::stdio::read_line",
        "io::buf::read_to_end",
    ] {
        let entry = table
            .get(name)
            .unwrap_or_else(|| panic!("{name} should have an honest forward dispatch entry"));
        assert!(
            !entry.implemented,
            "{name} should stay marked unimplemented until provider bridge shapes are implemented"
        );
    }
}

// ── TASK-621: is_known_builtin ────────────────────────────────────

#[test]
fn is_known_builtin_unqualified_len() {
    assert!(is_known_builtin("len", None));
}

#[test]
fn is_known_builtin_unqualified_map() {
    assert!(is_known_builtin("map", None));
}

#[test]
fn is_known_builtin_unknown_function() {
    assert!(!is_known_builtin("nonexistent_function_xyz", None));
}

#[test]
fn is_known_builtin_unknown_qualified() {
    assert!(!is_known_builtin("nonexistent", Some("math")));
}

#[test]
fn dispatch_builtin_unqualified_len_returns_correct_value() {
    let ctx = Context::new();
    let args = vec![Value::list_from_vec(vec![
        Value::Int(1),
        Value::Int(2),
        Value::Int(3),
    ])];
    let result = dispatch_builtin("len", &args, &ctx)
        .expect("dispatch should find len")
        .expect("len should succeed");
    assert_eq!(result, Value::Int(3));
}

#[test]
fn dispatch_builtin_unknown_returns_none() {
    let ctx = Context::new();
    let args = vec![Value::Int(42)];
    assert!(dispatch_builtin("nonexistent_xyz", &args, &ctx).is_none());
}

#[test]
fn dispatch_builtin_forward_declared_produces_unimplemented_error() {
    let ctx = Context::new();
    let result = dispatch_builtin("time::now", &[], &ctx)
        .expect("dispatch should find time::now in the table");
    assert!(
        matches!(result, Err(EvalError::UnimplementedBuiltin { .. })),
        "forward-declared builtin should produce UnimplementedBuiltin, got: {result:?}"
    );
}

#[test]
fn unimplemented_builtin_error_message_is_clear() {
    let err = EvalError::UnimplementedBuiltin {
        name: "string::to_upper".to_string(),
    };
    let msg = err.to_string();
    assert!(
        msg.contains("string::to_upper"),
        "error message should contain the builtin name, got: {msg}"
    );
    assert!(
        msg.contains("declared but not implemented"),
        "error message should explain the problem, got: {msg}"
    );
}

#[test]
fn eval_function_call_forward_declared_via_expr_produces_unimplemented() {
    let ctx = Context::new();
    // Call time::now() — it is forward-declared in
    // the dispatch table but deliberately not bridged in the interpreter.
    let expr = Expr::Call {
        func: "now".to_string(),
        module: Some("time".to_string()),
        arguments: vec![],
    };
    let err = eval_expr(&expr, &ctx).unwrap_err();
    assert!(
        matches!(err, EvalError::UnimplementedBuiltin { .. }),
        "expected UnimplementedBuiltin for time::now, got: {err:?}"
    );

    // Verify the error message contains the qualified name
    let msg = err.to_string();
    assert!(
        msg.contains("time::now"),
        "error message should contain qualified name, got: {msg}"
    );
}

#[test]
fn eval_function_call_existing_builtin_still_works() {
    let ctx = Context::new();
    // string::concat("hello ", "world") — implemented, should work fine
    let expr = Expr::Call {
        func: "concat".to_string(),
        module: Some("string".to_string()),
        arguments: vec![
            Expr::Literal(Value::String("hello ".into())),
            Expr::Literal(Value::String("world".into())),
        ],
    };
    let result = eval_expr(&expr, &ctx).expect("string::concat should succeed");
    assert_eq!(result, Value::String("hello world".to_string()));
}

#[test]
fn unimplemented_builtin_with_unqualified_name() {
    // Verify that an unqualified builtin name still in the table but not
    // implemented in eval_function_call's match produces UnknownFunction
    // (since it won't match any arm and is_known_builtin checks the table).
    // For this test we verify the error variant exists and can be constructed.
    let err = EvalError::UnimplementedBuiltin {
        name: "future_builtin".to_string(),
    };
    assert_eq!(
        err.to_string(),
        "builtin function 'future_builtin' declared but not implemented in runtime"
    );
}

// ── BuiltinEntry properties ──────────────────────────────────────

#[test]
fn builtin_entry_derives_correctly() {
    let e1 = BuiltinEntry {
        arity: 2,
        variadic: false,
        implemented: true,
    };
    let e2 = BuiltinEntry {
        arity: 2,
        variadic: false,
        implemented: true,
    };
    assert_eq!(e1, e2);

    let e3 = BuiltinEntry {
        arity: 1,
        variadic: false,
        implemented: true,
    };
    assert_ne!(e1, e3);
}

#[test]
fn dispatch_table_all_implemented_entries_dispatch_correctly() {
    let ctx = Context::new();
    let table = builtin_dispatch_table();

    // Spot-check a few implemented entries via dispatch_builtin
    for name in &[
        "string::concat",
        "string::is_empty",
        "regex::matches",
        "len",
        "is_null",
    ] {
        let entry = table.get(name).unwrap();
        assert!(entry.implemented, "{name} should be implemented");

        // dispatch_builtin should return Some (not None) for table entries
        let args = match *name {
            "string::concat" => vec![Value::String("a".into())],
            "string::is_empty" => vec![Value::String("".into())],
            "regex::matches" => vec![Value::String("a+".into()), Value::String("aaa".into())],
            "len" => vec![Value::list_nil()],
            "is_null" => vec![Value::Null],
            _ => vec![],
        };
        assert!(
            dispatch_builtin(name, &args, &ctx).is_some(),
            "dispatch_builtin should return Some for table entry {name}"
        );
    }
}

// ── TASK-637/TASK-638: Qualified list builtin dispatch ──────────

#[test]
fn dispatch_builtin_unqualified_len_still_works() {
    let ctx = Context::new();
    let args = vec![Value::list_from_vec(vec![
        Value::Int(1),
        Value::Int(2),
        Value::Int(3),
    ])];
    let result = dispatch_builtin("len", &args, &ctx)
        .expect("dispatch should find len")
        .expect("len should succeed");
    assert_eq!(result, Value::Int(3));
}

#[test]
fn eval_function_call_unqualified_len_still_works_via_expr() {
    let ctx = Context::new();
    let expr = Expr::Call {
        func: "len".to_string(),
        module: None,
        arguments: vec![Expr::Literal(Value::list_from_vec(vec![
            Value::Int(5),
            Value::Int(6),
        ]))],
    };
    let result = eval_expr(&expr, &ctx).expect("unqualified len should succeed");
    assert_eq!(result, Value::Int(2));
}

// ── TASK-642: Qualified predicate builtin dispatch ──────────────
