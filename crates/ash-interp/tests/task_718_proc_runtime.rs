use ash_core::{Expr, Value};
use ash_interp::context::Context;
use ash_interp::eval::eval_expr;

fn proc_unit(value: Value) -> Expr {
    Expr::Call {
        func: "unit".to_string(),
        module: Some("proc".to_string()),
        arguments: vec![Expr::Literal(value)],
    }
}

fn call_proc(expr: Expr) -> Value {
    let mut ctx = Context::new();
    let proc_value = eval_expr(&expr, &ctx).expect("proc expression should evaluate");
    let Value::Closure { params, .. } = &proc_value else {
        panic!("expected Proc runtime closure, got {proc_value:?}");
    };
    assert_eq!(params, &vec![("__proc_env".to_string(), None)]);

    ctx.set("p".to_string(), proc_value);
    eval_expr(
        &Expr::Call {
            func: "p".to_string(),
            module: None,
            arguments: vec![Expr::Literal(Value::Null)],
        },
        &ctx,
    )
    .expect("Proc runtime closure should be callable with opaque process env token")
}

#[test]
fn proc_unit_lifts_value_without_creating_process_handle() {
    let result = call_proc(proc_unit(Value::Int(42)));
    assert_eq!(result, Value::Int(42));
}

#[test]
fn proc_bind_sequences_dependent_proc_closures() {
    let continuation = Expr::FnDef {
        params: vec![("x".to_string(), None)],
        return_type: None,
        body: Box::new(proc_unit(Value::String("done".to_string()))),
    };

    let result = call_proc(Expr::Call {
        func: "bind".to_string(),
        module: Some("proc".to_string()),
        arguments: vec![proc_unit(Value::Int(1)), continuation],
    });

    assert_eq!(result, Value::String("done".to_string()));
}

#[test]
fn proc_then_discards_left_value_and_returns_right_value() {
    let result = call_proc(Expr::Call {
        func: "then".to_string(),
        module: Some("proc".to_string()),
        arguments: vec![
            proc_unit(Value::String("discarded".to_string())),
            proc_unit(Value::Bool(true)),
        ],
    });

    assert_eq!(result, Value::Bool(true));
}
