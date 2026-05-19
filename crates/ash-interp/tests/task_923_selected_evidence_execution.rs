use ash_core::{Expr, Value};
use ash_interp::{Context, eval_expr};

fn int(value: i64) -> Expr {
    Expr::Literal(Value::Int(value))
}

fn var(name: &str) -> Expr {
    Expr::Variable {
        name: name.to_string(),
        span: ash_core::Span::default(),
    }
}

fn some(value: Expr) -> Expr {
    Expr::Constructor {
        name: "Some".to_string(),
        fields: vec![("value".to_string(), value)],
    }
}

#[test]
fn task_923_selected_return_method_closure_executes_in_interpreter() {
    let expr = Expr::FnApply {
        func: Box::new(Expr::FnDef {
            params: vec![("value".to_string(), None)],
            return_type: None,
            body: Box::new(some(var("value"))),
        }),
        args: vec![int(7)],
    };

    let value = eval_expr(&expr, &Context::new()).expect("selected return closure should execute");

    assert_eq!(
        value,
        Value::Variant {
            name: "Some".to_string(),
            fields: Box::new(vec![("value".to_string(), Value::Int(7))]),
        }
    );
}

#[test]
fn task_923_selected_bind_method_closure_executes_in_interpreter() {
    let expr = Expr::FnApply {
        func: Box::new(Expr::FnDef {
            params: vec![("value".to_string(), None), ("_f".to_string(), None)],
            return_type: None,
            body: Box::new(var("value")),
        }),
        args: vec![
            some(int(7)),
            Expr::FnDef {
                params: vec![("next".to_string(), None)],
                return_type: None,
                body: Box::new(some(var("next"))),
            },
        ],
    };

    let value = eval_expr(&expr, &Context::new()).expect("selected bind closure should execute");

    assert_eq!(
        value,
        Value::Variant {
            name: "Some".to_string(),
            fields: Box::new(vec![("value".to_string(), Value::Int(7))]),
        }
    );
}
