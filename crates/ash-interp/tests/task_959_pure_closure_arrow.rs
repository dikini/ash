//! TASK-959 interpreter coverage for preferred pure closure arrow syntax.

use ash_core::{Expr as CoreExpr, Value};
use ash_interp::context::Context;
use ash_interp::eval::eval_expr;
use ash_parser::input::new_input;
use ash_parser::lower::lower_expr;
use ash_parser::parse_expr::expr;

#[test]
fn pure_closure_arrow_executes_existing_closure_runtime_path() {
    let mut input = new_input("fn() { let inc = |x| -> x + 1; inc(5) }");
    let surface = expr(&mut input).expect("anonymous function with closure arrow should parse");
    assert!(
        input.input.trim().is_empty(),
        "parser left trailing input: {:?}",
        input.input
    );
    let lowered = lower_expr(&surface).expect("closure arrow should lower through FnDef");
    let call = CoreExpr::FnApply {
        func: Box::new(lowered),
        args: vec![],
    };

    let result = eval_expr(&call, &Context::new()).expect("closure arrow should execute");

    assert_eq!(result, Value::Int(6));
}
