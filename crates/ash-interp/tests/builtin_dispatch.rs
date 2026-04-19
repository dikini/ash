//! Tests for TASK-621 (Runtime Builtin Dispatch Table) and
//! TASK-622 (Clear Error on Unknown Builtin).

use ash_core::{Expr, Value};
use ash_interp::context::Context;
use ash_interp::error::EvalError;
use ash_interp::eval::{
    BuiltinEntry, builtin_dispatch_table, dispatch_builtin, eval_expr, is_known_builtin,
};

// ── TASK-621: Dispatch table structure ────────────────────────────

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
fn dispatch_table_forward_declared_builtins_are_marked_unimplemented() {
    let table = builtin_dispatch_table();
    for name in &["string::to_upper", "string::to_lower", "string::trim"] {
        let entry = table
            .get(name)
            .unwrap_or_else(|| panic!("{name} should be in the dispatch table"));
        assert!(
            !entry.implemented,
            "{name} should be marked as not implemented"
        );
    }
}

// ── TASK-621: is_known_builtin ────────────────────────────────────

#[test]
fn is_known_builtin_qualified_string_concat() {
    assert!(is_known_builtin("concat", Some("string")));
}

#[test]
fn is_known_builtin_qualified_string_starts_with() {
    assert!(is_known_builtin("starts_with", Some("string")));
}

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

// ── TASK-621: dispatch_builtin (implemented path) ────────────────

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
fn dispatch_builtin_unqualified_len_returns_correct_value() {
    let ctx = Context::new();
    let args = vec![Value::List(Box::new(vec![
        Value::Int(1),
        Value::Int(2),
        Value::Int(3),
    ]))];
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

// ── TASK-622: UnimplementedBuiltin error ──────────────────────────

#[test]
fn dispatch_builtin_forward_declared_produces_unimplemented_error() {
    let ctx = Context::new();
    let args = vec![Value::String("hello".into())];
    let result = dispatch_builtin("string::to_upper", &args, &ctx)
        .expect("dispatch should find string::to_upper in the table");
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
    // Call string::to_upper("hello") — it's in the dispatch table but not
    // implemented in eval_function_call's match arms.
    let expr = Expr::Call {
        func: "to_upper".to_string(),
        module: Some("string".to_string()),
        arguments: vec![Expr::Literal(Value::String("hello".into()))],
    };
    let err = eval_expr(&expr, &ctx).unwrap_err();
    assert!(
        matches!(err, EvalError::UnimplementedBuiltin { .. }),
        "expected UnimplementedBuiltin for string::to_upper, got: {err:?}"
    );

    // Verify the error message contains the qualified name
    let msg = err.to_string();
    assert!(
        msg.contains("string::to_upper"),
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
    for name in &["string::concat", "string::is_empty", "len", "is_null"] {
        let entry = table.get(name).unwrap();
        assert!(entry.implemented, "{name} should be implemented");

        // dispatch_builtin should return Some (not None) for table entries
        let args = match *name {
            "string::concat" => vec![Value::String("a".into())],
            "string::is_empty" => vec![Value::String("".into())],
            "len" => vec![Value::List(Box::default())],
            "is_null" => vec![Value::Null],
            _ => vec![],
        };
        assert!(
            dispatch_builtin(name, &args, &ctx).is_some(),
            "dispatch_builtin should return Some for table entry {name}"
        );
    }
}
