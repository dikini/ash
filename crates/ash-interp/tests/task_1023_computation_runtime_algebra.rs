//! TASK-1023 runtime checks for public computation algebra operations.

use ash_core::{Expr, Value};
use ash_interp::act_env::ActEnv;
use ash_interp::context::Context;
use ash_interp::eval::eval_expr;

fn lit(value: Value) -> Expr {
    Expr::Literal(value)
}

fn call(module: &str, func: &str, arguments: Vec<Expr>) -> Expr {
    Expr::Call {
        func: func.to_string(),
        module: Some(module.to_string()),
        arguments,
    }
}

fn fn_def(param: &str, body: Expr) -> Expr {
    Expr::FnDef {
        params: vec![(param.to_string(), None)],
        return_type: None,
        body: Box::new(body),
    }
}

fn force_closure(value: Value, env_token: Value) -> Value {
    let mut ctx = Context::new();
    ctx.set("thunk".to_string(), value);
    eval_expr(
        &Expr::Call {
            func: "thunk".to_string(),
            module: None,
            arguments: vec![lit(env_token)],
        },
        &ctx,
    )
    .expect("runtime carrier closure should force")
}

fn force_act_closure(value: Value) -> Value {
    let mut ctx = Context::new().with_act_env(ActEnv::default());
    ctx.set("thunk".to_string(), value);
    eval_expr(
        &Expr::Call {
            func: "thunk".to_string(),
            module: None,
            arguments: vec![lit(Value::ActEnvToken)],
        },
        &ctx,
    )
    .expect("act carrier closure should force through hidden runtime state")
}

fn public_unit(module: &str, value: Value) -> Value {
    eval_expr(&call(module, "unit", vec![lit(value)]), &Context::new())
        .unwrap_or_else(|error| panic!("{module}::unit should evaluate: {error}"))
}

fn public_bind(module: &str, left: Expr, continuation_body: Expr) -> Value {
    eval_expr(
        &call(module, "bind", vec![left, fn_def("x", continuation_body)]),
        &Context::new(),
    )
    .unwrap_or_else(|error| panic!("{module}::bind should evaluate: {error}"))
}

fn assert_closure_param(value: &Value, expected: &str) {
    let Value::Closure { params, .. } = value else {
        panic!("expected runtime carrier closure, got {value:?}");
    };
    assert_eq!(params, &vec![(expected.to_string(), None)]);
}

#[test]
fn task1023_act_computation_runtime_public_unit_preserves_act_opacity() {
    let act = public_unit("act", Value::Int(7));

    assert_closure_param(&act, "__act_env");
    assert_eq!(
        force_act_closure(act),
        Value::list_from_vec(vec![Value::ActEnvToken, Value::Int(7)])
    );
}

#[test]
fn task1023_act_computation_runtime_public_bind_sequences_without_exposing_actenv() {
    let act = public_bind(
        "act",
        call("act", "unit", vec![lit(Value::Int(1))]),
        call("act", "unit", vec![lit(Value::String("{};".to_string()))]),
    );

    assert_closure_param(&act, "__act_env");
    assert_eq!(
        force_act_closure(act),
        Value::list_from_vec(vec![Value::ActEnvToken, Value::String("{};".to_string())])
    );
}

#[test]
fn task1023_proc_computation_runtime_public_unit_and_bind_preserve_process_carrier() {
    let proc = public_unit("proc", Value::Int(7));
    assert_closure_param(&proc, "__proc_env");
    assert_eq!(force_closure(proc, Value::Null), Value::Int(7));

    let bound = public_bind(
        "proc",
        call("proc", "unit", vec![lit(Value::Int(1))]),
        call("proc", "unit", vec![lit(Value::String("{};".to_string()))]),
    );
    assert_closure_param(&bound, "__proc_env");
    assert_eq!(
        force_closure(bound, Value::Null),
        Value::String("{};".to_string())
    );
}

#[test]
fn task1023_application_computation_runtime_public_unit_and_bind_preserve_application_carrier() {
    let application = public_unit("application", Value::Int(7));
    assert_closure_param(&application, "__proc_env");
    assert_eq!(force_closure(application, Value::Null), Value::Int(7));

    let bound = public_bind(
        "application",
        call("application", "unit", vec![lit(Value::Int(1))]),
        call(
            "application",
            "unit",
            vec![lit(Value::String("admitted".to_string()))],
        ),
    );
    assert_closure_param(&bound, "__proc_env");
    assert_eq!(
        force_closure(bound, Value::Null),
        Value::String("admitted".to_string())
    );
}
