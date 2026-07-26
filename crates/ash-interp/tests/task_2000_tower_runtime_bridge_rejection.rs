//! TASK-2000: retired tower bridge names must not reach interpreter dispatch.

use ash_core::{Expr, Value};
use ash_interp::eval::{builtin_dispatch_table, is_known_builtin};
use ash_interp::{Context, EvalError, eval_expr, eval_expr_async};

const RETIRED_BRIDGES: [(&str, &str); 14] = [
    ("act", "unit"),
    ("act", "bind"),
    ("act", "__guard"),
    ("act", "policy_check"),
    ("proc", "unit"),
    ("proc", "from_act"),
    ("proc", "bind"),
    ("proc", "then"),
    ("proc", "await"),
    ("proc", "yield"),
    ("proc", "par"),
    ("proc", "scatter"),
    ("proc", "join"),
    ("proc", "gather"),
];

fn call(module: &str, func: &str, arguments: Vec<Expr>) -> Expr {
    Expr::Call {
        func: func.to_string(),
        module: Some(module.to_string()),
        arguments,
    }
}

#[test]
fn retired_tower_bridges_are_not_builtin_names() {
    let table = builtin_dispatch_table();

    for (module, func) in RETIRED_BRIDGES {
        let qualified = format!("{module}::{func}");
        assert!(
            !table.contains_key(qualified.as_str()),
            "retired bridge {qualified} must not remain in builtin metadata"
        );
        assert!(
            !is_known_builtin(func, Some(module)),
            "retired bridge {qualified} must not remain a known builtin"
        );
    }
}

#[test]
fn retired_tower_bridges_do_not_bypass_sync_or_async_builtin_rejection() {
    let sync = eval_expr(
        &call("act", "unit", vec![Expr::Literal(Value::Int(7))]),
        &Context::new(),
    )
    .expect_err("the sync evaluator must not retain an act bridge fast path");
    assert!(matches!(sync, EvalError::UnknownFunction(ref name) if name == "unit"));

    let async_error = tokio_test::block_on(eval_expr_async(
        &call("proc", "yield", Vec::new()),
        &Context::new(),
    ))
    .expect_err("the async evaluator must not retain a proc bridge fast path");
    assert!(matches!(async_error, EvalError::UnknownFunction(ref name) if name == "yield"));
}

#[test]
fn canonical_non_wrapper_builtin_remains_available() {
    let value = eval_expr(
        &call(
            "string",
            "concat",
            vec![
                Expr::Literal(Value::String("ash".to_string())),
                Expr::Literal(Value::String("!".to_string())),
            ],
        ),
        &Context::new(),
    )
    .expect("canonical string builtin remains available");

    assert_eq!(value, Value::String("ash!".to_string()));
}
