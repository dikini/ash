//! Source-compatibility coverage for existing `ExecResult<Value>` engine APIs.

use ash_core::{Expr, Value, Workflow};
use ash_engine::Engine;
use ash_interp::ExecResult;

#[tokio::test]
async fn engine_execute_core_workflow_remains_exec_result_value_compatible() {
    let engine = Engine::new().build().expect("engine builds");
    let workflow = Workflow::Ret {
        expr: Expr::Literal(Value::Int(7)),
    };

    let result: ExecResult<Value> = engine.execute_core_workflow(&workflow).await;

    assert_eq!(result.expect("workflow succeeds"), Value::Int(7));
}
