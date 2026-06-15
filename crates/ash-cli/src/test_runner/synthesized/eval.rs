//! Shared expression and JSON/core-value evaluators for synthesized rows.

use std::collections::{BTreeMap, HashMap};

use ash_core::{Expr as CoreExpr, Value as CoreValue};
use ash_interp::{Context as InterpContext, eval_expr};
use serde_json::{Value, json};

pub(crate) fn evaluate_simple_bool_expression(
    expression: &str,
    bindings: &BTreeMap<String, Value>,
) -> Result<bool, String> {
    let tokens: Vec<&str> = expression.split_whitespace().collect();
    if tokens.len() != 3 {
        return Err(format!("expected '<term> <op> <term>', got {expression:?}"));
    }

    let left = resolve_simple_value(tokens[0], bindings)?;
    let right = resolve_simple_value(tokens[2], bindings)?;

    match tokens[1] {
        "==" => Ok(left == right),
        "!=" => Ok(left != right),
        ">" => compare_i64(&left, &right, |left, right| left > right),
        ">=" => compare_i64(&left, &right, |left, right| left >= right),
        "<" => compare_i64(&left, &right, |left, right| left < right),
        "<=" => compare_i64(&left, &right, |left, right| left <= right),
        other => Err(format!("unsupported operator {other}")),
    }
}

fn resolve_simple_value(term: &str, bindings: &BTreeMap<String, Value>) -> Result<Value, String> {
    if let Some(value) = bindings.get(term) {
        return Ok(value.clone());
    }
    if let Ok(value) = term.parse::<i64>() {
        return Ok(json!(value));
    }
    match term {
        "true" => Ok(json!(true)),
        "false" => Ok(json!(false)),
        "null" => Ok(Value::Null),
        _ if term.starts_with('"') && term.ends_with('"') && term.len() >= 2 => {
            Ok(json!(term.trim_matches('"')))
        }
        _ => Err(format!("missing binding or unsupported literal for {term}")),
    }
}

fn compare_i64(
    left: &Value,
    right: &Value,
    compare: impl FnOnce(i64, i64) -> bool,
) -> Result<bool, String> {
    let left = left
        .as_i64()
        .ok_or_else(|| format!("left operand is not an integer: {left}"))?;
    let right = right
        .as_i64()
        .ok_or_else(|| format!("right operand is not an integer: {right}"))?;
    Ok(compare(left, right))
}

pub(super) fn evaluate_contract_postcondition(
    oracle: &CoreExpr,
    bindings: &BTreeMap<String, Value>,
    target_output: &Value,
) -> Result<bool, String> {
    let output = json_value_to_core_value(target_output)?;
    match evaluate_core_value(oracle, bindings, Some(output))? {
        CoreValue::Bool(value) => Ok(value),
        other => Err(format!(
            "postcondition oracle evaluated to non-bool {other:?}"
        )),
    }
}

pub(super) fn evaluate_core_expression(
    expression: &CoreExpr,
    bindings: &BTreeMap<String, Value>,
    target_output: Option<CoreValue>,
) -> Result<Value, String> {
    evaluate_core_value(expression, bindings, target_output).and_then(core_value_to_json)
}

fn evaluate_core_value(
    expression: &CoreExpr,
    bindings: &BTreeMap<String, Value>,
    target_output: Option<CoreValue>,
) -> Result<CoreValue, String> {
    let mut runtime_bindings = HashMap::new();
    for (name, value) in bindings {
        runtime_bindings.insert(name.clone(), json_value_to_core_value(value)?);
    }
    if let Some(output) = target_output {
        runtime_bindings.insert("result".to_string(), output);
    }
    let ctx = InterpContext::with_bindings(runtime_bindings);
    eval_expr(expression, &ctx).map_err(|error| error.to_string())
}

fn json_value_to_core_value(value: &Value) -> Result<CoreValue, String> {
    match value {
        Value::Null => Ok(CoreValue::Null),
        Value::Bool(value) => Ok(CoreValue::Bool(*value)),
        Value::Number(value) => value
            .as_i64()
            .map(CoreValue::Int)
            .ok_or_else(|| format!("unsupported non-integer JSON number {value}")),
        Value::String(value) => Ok(CoreValue::String(value.clone())),
        Value::Array(values) => values
            .iter()
            .map(json_value_to_core_value)
            .collect::<Result<Vec<_>, _>>()
            .map(|values| CoreValue::List(Box::new(values))),
        Value::Object(values) => values
            .iter()
            .map(|(name, value)| Ok((name.clone(), json_value_to_core_value(value)?)))
            .collect::<Result<HashMap<_, _>, _>>()
            .map(|values| CoreValue::Record(Box::new(values))),
    }
}

fn core_value_to_json(value: CoreValue) -> Result<Value, String> {
    match value {
        CoreValue::Null => Ok(Value::Null),
        CoreValue::Bool(value) => Ok(json!(value)),
        CoreValue::Int(value) => Ok(json!(value)),
        CoreValue::Float(value) => Ok(json!(value)),
        CoreValue::String(value) => Ok(json!(value)),
        CoreValue::List(values) => values
            .into_iter()
            .map(core_value_to_json)
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        CoreValue::Record(values) => values
            .into_iter()
            .map(|(name, value)| Ok((name, core_value_to_json(value)?)))
            .collect::<Result<serde_json::Map<_, _>, _>>()
            .map(Value::Object),
        other => Err(format!("unsupported interpreter output {other:?}")),
    }
}
