use super::support::*;

#[test]
fn dispatch_table_contains_process_run() {
    let table = builtin_dispatch_table();
    let entry = table
        .get("process::run")
        .expect("process::run should be in the dispatch table");
    assert_eq!(entry.arity, 2);
    assert!(!entry.variadic);
    assert!(entry.implemented);
}

#[test]
fn is_known_builtin_qualified_process_run() {
    assert!(is_known_builtin("run", Some("process")));
}

#[test]
fn dispatch_builtin_process_run_echo() {
    let ctx = Context::new();
    let args = vec![
        Value::String("echo".into()),
        Value::List(Box::new(vec![Value::String("hello".into())])),
    ];
    let result = dispatch_builtin("process::run", &args, &ctx)
        .expect("dispatch should find process::run")
        .expect("process::run should succeed");
    assert_process_run_record(result, "hello\n", "", 0);
}

#[test]
fn dispatch_builtin_process_run_with_multiple_args() {
    let ctx = Context::new();
    let args = vec![
        Value::String("echo".into()),
        Value::List(Box::new(vec![
            Value::String("hello".into()),
            Value::String("world".into()),
        ])),
    ];
    let result = dispatch_builtin("process::run", &args, &ctx)
        .expect("dispatch should find process::run")
        .expect("process::run should succeed");
    assert_process_run_record(result, "hello world\n", "", 0);
}

#[test]
fn dispatch_builtin_process_run_empty_args() {
    let ctx = Context::new();
    let args = vec![Value::String("echo".into()), Value::List(Box::default())];
    let result = dispatch_builtin("process::run", &args, &ctx)
        .expect("dispatch should find process::run")
        .expect("process::run should succeed");
    assert_process_run_record(result, "\n", "", 0);
}

#[test]
fn dispatch_builtin_process_run_wrong_arity() {
    let ctx = Context::new();
    let args = vec![Value::String("echo".into())];
    let result = dispatch_builtin("process::run", &args, &ctx)
        .expect("dispatch should find process::run in the table");
    assert!(
        result.is_err(),
        "process::run with 1 arg should produce an error"
    );
}

#[test]
fn dispatch_builtin_process_run_nonexistent_command() {
    let ctx = Context::new();
    let args = vec![
        Value::String("nonexistent_command_xyz_12345".into()),
        Value::List(Box::default()),
    ];
    let result = dispatch_builtin("process::run", &args, &ctx)
        .expect("dispatch should find process::run")
        .expect_err("nonexistent command should fail");
    let msg = result.to_string();
    assert!(
        msg.contains("process::run failed"),
        "error should mention process::run failed, got: {msg}"
    );
}

#[test]
fn eval_function_call_process_run_via_expr() {
    let ctx = Context::new();
    let expr = Expr::Call {
        func: "run".to_string(),
        module: Some("process".to_string()),
        arguments: vec![
            Expr::Literal(Value::String("echo".into())),
            Expr::Literal(Value::List(Box::new(vec![Value::String("hello".into())]))),
        ],
    };
    let result = eval_expr(&expr, &ctx).expect("process::run should succeed");
    assert_process_run_record(result, "hello\n", "", 0);
}

// ── TASK-596: Markdown builtin dispatch tests ────────────────────
