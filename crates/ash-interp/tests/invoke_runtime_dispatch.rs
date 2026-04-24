use std::sync::Arc;

use ash_core::{Effect, Expr, Value};
use ash_interp::RuntimeState;
use ash_interp::act_env::ActEnv;
use ash_interp::capability::MockProvider;
use ash_interp::context::Context;
use ash_interp::eval::{eval_expr, eval_expr_async};

fn shadowing_closure(marker: &str) -> Value {
    eval_expr(
        &Expr::FnDef {
            params: vec![("_ignored".to_string(), None)],
            return_type: None,
            body: Box::new(Expr::Literal(Value::String(marker.to_string()))),
        },
        &Context::new(),
    )
    .expect("closure definition should evaluate")
}

fn one_arg_call(name: &str) -> Expr {
    Expr::Call {
        func: name.to_string(),
        module: None,
        arguments: vec![Expr::Literal(Value::String("arg".to_string()))],
    }
}

fn invoke_expr() -> Expr {
    Expr::Call {
        func: "invoke".into(),
        module: None,
        arguments: vec![
            Expr::Literal(Value::String("sensor".to_string())),
            Expr::Literal(Value::String("read".to_string())),
            Expr::Literal(Value::List(Box::new(vec![Value::Int(1), Value::Int(2)]))),
        ],
    }
}

#[tokio::test]
async fn invoke_dispatch_returns_closure_with_captured_state() {
    let ctx = Context::new();
    let result = eval_expr(&invoke_expr(), &ctx).expect("invoke should dispatch");

    let Value::Closure { params, .. } = result.clone() else {
        panic!("expected closure from invoke, got {result:?}");
    };
    assert!(
        params == vec![("__act_env".to_string(), None)],
        "invoke closure should require the hidden ActEnv carrier"
    );

    let runtime_state = RuntimeState::new().with_provider(
        "sensor",
        Arc::new(
            MockProvider::new("sensor", Effect::Operational)
                .with_execute_result(Ok(Value::String("done".to_string()))),
        ),
    );
    let act_env = ActEnv::from_runtime_state(
        &runtime_state,
        ash_interp::PolicyEvaluator::new(),
        ash_core::Provenance::new(),
    )
    .await;

    let mut call_ctx = Context::new().with_act_env(act_env);
    call_ctx.set("act".to_string(), result);
    let applied = eval_expr(
        &Expr::Call {
            func: "act".into(),
            module: None,
            arguments: vec![Expr::Literal(Value::ActEnvToken)],
        },
        &call_ctx,
    )
    .expect("invoke closure should be callable");

    assert_eq!(
        applied,
        Value::List(Box::new(vec![
            Value::ActEnvToken,
            Value::String("done".to_string())
        ]))
    );
}

#[test]
fn invoke_closure_rejects_visible_token_without_hidden_runtime_act_env() {
    let ctx = Context::new();
    let result = eval_expr(&invoke_expr(), &ctx).expect("invoke should dispatch");

    let mut call_ctx = Context::new();
    call_ctx.set("act".to_string(), result);
    let applied = eval_expr(
        &Expr::Call {
            func: "act".into(),
            module: None,
            arguments: vec![Expr::Literal(Value::ActEnvToken)],
        },
        &call_ctx,
    );

    assert!(
        applied.is_err(),
        "visible ActEnvToken alone should not satisfy invoke's hidden runtime carrier boundary"
    );
}

#[test]
fn invoke_does_not_break_existing_builtin_dispatch() {
    let ctx = Context::new();
    let concat = Expr::Call {
        func: "concat".into(),
        module: Some("string".into()),
        arguments: vec![
            Expr::Literal(Value::String("a".to_string())),
            Expr::Literal(Value::String("b".to_string())),
        ],
    };

    let result = eval_expr(&concat, &ctx).expect("string::concat should still work");
    assert_eq!(result, Value::String("ab".to_string()));
}

#[test]
fn unqualified_public_runtime_names_resolve_context_closures_before_primitives_sync() {
    for name in ["unit", "bind", "invoke", "policy_check"] {
        let marker = format!("user {name}");
        let mut ctx = Context::new();
        ctx.set(name.to_string(), shadowing_closure(&marker));

        let result = eval_expr(&one_arg_call(name), &ctx)
            .unwrap_or_else(|err| panic!("{name} should resolve through context first: {err:?}"));

        assert_eq!(result, Value::String(marker));
    }
}

#[tokio::test]
async fn unqualified_public_runtime_names_resolve_context_closures_before_primitives_async() {
    for name in ["unit", "bind", "invoke", "policy_check"] {
        let marker = format!("async user {name}");
        let mut ctx = Context::new();
        ctx.set(name.to_string(), shadowing_closure(&marker));

        let result = eval_expr_async(&one_arg_call(name), &ctx)
            .await
            .unwrap_or_else(|err| panic!("{name} should resolve through context first: {err:?}"));

        assert_eq!(result, Value::String(marker));
    }
}
