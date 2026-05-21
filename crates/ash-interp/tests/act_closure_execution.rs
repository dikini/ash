use std::sync::Arc;

use ash_core::{
    CapabilityBinding, CapabilityBindingId, CapabilityInterfaceId, Effect, Expr, Value,
};
use ash_interp::RuntimeState;
use ash_interp::act_env::ActEnv;
use ash_interp::capability::MockProvider;
use ash_interp::context::Context;
use ash_interp::eval::eval_expr;
use ash_parser::lower::lower_expr;
use ash_parser::surface::{ActStmt, Expr as SurfaceExpr, Literal};

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

fn simple_return_act_surface(value: i64) -> SurfaceExpr {
    SurfaceExpr::ActBlock {
        stmts: vec![ActStmt::Return {
            value: Box::new(SurfaceExpr::Literal(Literal::Int(value))),
            span: ash_parser::token::Span::default(),
        }],
        span: ash_parser::token::Span::default(),
    }
}

fn nested_bind_act_surface(value: i64) -> SurfaceExpr {
    SurfaceExpr::ActBlock {
        stmts: vec![
            ActStmt::Bind {
                name: "x".into(),
                value: Box::new(SurfaceExpr::ActBlock {
                    stmts: vec![ActStmt::Return {
                        value: Box::new(SurfaceExpr::Literal(Literal::Int(value))),
                        span: ash_parser::token::Span::default(),
                    }],
                    span: ash_parser::token::Span::default(),
                }),
                span: ash_parser::token::Span::default(),
            },
            ActStmt::Return {
                value: Box::new(SurfaceExpr::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                }),
                span: ash_parser::token::Span::default(),
            },
        ],
        span: ash_parser::token::Span::default(),
    }
}

fn force_act_value(value: Value) -> Value {
    let ctx = Context::new().with_act_env(ActEnv::default());
    eval_expr(
        &Expr::FnApply {
            func: Box::new(Expr::Literal(value)),
            args: vec![Expr::Literal(Value::ActEnvToken)],
        },
        &ctx,
    )
    .expect("forced act closure should evaluate")
}

fn force_act_value_without_hidden_runtime_act_env(value: Value) -> ash_interp::EvalResult<Value> {
    let ctx = Context::new();
    eval_expr(
        &Expr::FnApply {
            func: Box::new(Expr::Literal(value)),
            args: vec![Expr::Literal(Value::ActEnvToken)],
        },
        &ctx,
    )
}

#[test]
fn lowered_single_return_act_block_executes_via_closures() {
    let surface = simple_return_act_surface(7);
    let lowered = lower_expr(&surface).expect("single-return act block should lower");

    let act_value = eval_expr(&lowered, &Context::new()).expect("lowered act should evaluate");
    assert!(matches!(act_value, Value::Closure { .. }));

    let forced = force_act_value(act_value);
    assert_eq!(
        forced,
        Value::List(Box::new(vec![Value::ActEnvToken, Value::Int(7)]))
    );
}

#[test]
fn lowered_act_closure_rejects_visible_token_without_hidden_runtime_act_env() {
    let surface = simple_return_act_surface(5);
    let lowered = lower_expr(&surface).expect("single-return act block should lower");

    let act_value = eval_expr(&lowered, &Context::new()).expect("lowered act should evaluate");
    let forced = force_act_value_without_hidden_runtime_act_env(act_value);

    assert!(
        forced.is_err(),
        "visible ActEnvToken alone should not satisfy the hidden runtime carrier boundary"
    );
}

#[tokio::test]
async fn effectful_closure_composition_round_trips_through_force() {
    let composer = Expr::FnDef {
        params: vec![("act".to_string(), None)],
        return_type: None,
        body: Box::new(Expr::Variable {
            name: "act".to_string(),
            span: ash_core::ast::Span::default(),
        }),
    };

    let runtime_state = RuntimeState::new().with_provider(
        "sensor",
        Arc::new(
            MockProvider::new("sensor", Effect::Operational)
                .with_execute_result(Ok(Value::String("done".to_string()))),
        ),
    );
    let binding = CapabilityBinding::host_provider(
        CapabilityBindingId::new(),
        "sensor",
        CapabilityInterfaceId::new("Sensor"),
        "sensor",
        vec!["sensor.read".to_string()],
    );
    let binding_id = binding.id;
    runtime_state
        .admit_capability_binding(binding)
        .await
        .expect("sensor binding admission succeeds");
    let act_env = ActEnv::from_runtime_state_with_admitted_bindings(
        &runtime_state,
        &[binding_id],
        ash_interp::PolicyEvaluator::new(),
        ash_core::Provenance::new(),
    )
    .await
    .expect("admitted act env projection succeeds");

    let ctx = Context::new()
        .with_runtime_state(runtime_state)
        .with_admitted_capability_bindings(vec![binding_id])
        .with_act_env(act_env);
    let composed = eval_expr(
        &Expr::FnApply {
            func: Box::new(Expr::FnApply {
                func: Box::new(composer),
                args: vec![invoke_expr()],
            }),
            args: vec![Expr::Literal(Value::ActEnvToken)],
        },
        &ctx,
    )
    .expect("composed effectful closure should evaluate");

    assert_eq!(
        composed,
        Value::List(Box::new(vec![
            Value::ActEnvToken,
            Value::String("done".to_string())
        ]))
    );
}

#[test]
fn lowered_nested_act_block_executes_left_to_right() {
    let surface = nested_bind_act_surface(11);
    let lowered = lower_expr(&surface).expect("nested act block should lower");

    let act_value = eval_expr(&lowered, &Context::new()).expect("lowered act should evaluate");
    assert!(matches!(act_value, Value::Closure { .. }));

    let forced = force_act_value(act_value);
    assert_eq!(
        forced,
        Value::List(Box::new(vec![Value::ActEnvToken, Value::Int(11)]))
    );
}
