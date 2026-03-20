use ash_core::Value;
use ash_macros::workflow;

#[workflow]
fn workflow_returns_ok_value() -> Result<Value, &'static str> {
    Ok(Value::Int(7))
}

#[workflow]
fn workflow_returns_err_value() -> Result<Value, &'static str> {
    Err("boom")
}

#[test]
fn workflow_macro_expands_for_successful_result_functions() {
    let result = workflow_returns_ok_value();
    assert_eq!(result, Ok(Value::Int(7)));
}

#[test]
fn workflow_macro_expands_for_error_result_functions() {
    let result = workflow_returns_err_value();
    assert_eq!(result, Err("boom"));
}
