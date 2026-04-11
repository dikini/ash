#![allow(
    clippy::clone_on_copy,
    clippy::map_unwrap_or,
    clippy::option_if_let_else,
    clippy::or_fun_call,
    clippy::redundant_clone,
    clippy::single_match_else,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    clippy::unused_async
)]

use std::collections::HashMap;

use ash_core::Value;
use ash_interp::{Context, ExecError, ExecResult};
use ash_parser::input::new_input;
use ash_parser::parse_module::parse_fn_definition;
use ash_parser::parse_utils::skip_whitespace_and_comments;
use ash_parser::parse_workflow::workflow_def;
use ash_parser::surface::{
    BinaryOp, BlockStmt, ConstructorPayload, Definition, Expr, FnDef, Literal, Pattern, Program,
    UnaryOp, Workflow, WorkflowDef,
};
use ash_typeck::requirements::{RequirementContext, check_contract};
use winnow::Parser;

pub fn parse_program_with_functions(source: &str) -> Result<Program, String> {
    let mut input = new_input(source);
    skip_whitespace_and_comments(&mut input);

    let mut definitions = Vec::new();
    loop {
        let snapshot = input.clone();
        match parse_fn_definition.parse_next(&mut input) {
            Ok(definition) => {
                definitions.push(definition);
                skip_whitespace_and_comments(&mut input);
            }
            Err(_) => {
                input = snapshot;
                break;
            }
        }
    }

    let workflow = workflow_def
        .parse_next(&mut input)
        .map_err(|error| error.to_string())?;
    skip_whitespace_and_comments(&mut input);
    if !input.input.is_empty() {
        return Err("unexpected trailing input after workflow definition".to_string());
    }

    Ok(Program {
        definitions,
        workflow,
    })
}

pub async fn execute_program(
    program: &Program,
    input_bindings: HashMap<String, Value>,
) -> ExecResult<Value> {
    let functions = program
        .definitions
        .iter()
        .filter_map(|definition| match definition {
            Definition::Function(function) => Some((function.name.to_string(), function.clone())),
            _ => None,
        })
        .collect::<HashMap<_, _>>();

    let ctx = Context::with_bindings(input_bindings);
    execute_workflow(&program.workflow, &ctx, &functions)
}

fn execute_workflow(
    workflow: &WorkflowDef,
    ctx: &Context,
    functions: &HashMap<String, FnDef>,
) -> ExecResult<Value> {
    execute_workflow_node(&workflow.body, ctx, functions)
}

fn execute_workflow_node(
    workflow: &Workflow,
    ctx: &Context,
    functions: &HashMap<String, FnDef>,
) -> ExecResult<Value> {
    match workflow {
        Workflow::Done { .. } => Ok(Value::Null),
        Workflow::Ret { expr, .. } => eval_expr(expr, ctx, functions),
        Workflow::Let {
            pattern,
            expr,
            continuation,
            ..
        } => {
            let value = eval_expr(expr, ctx, functions)?;
            let bindings = bind_pattern(pattern, &value)?;
            let mut next_ctx = ctx.extend();
            next_ctx.set_many(bindings);
            match continuation {
                Some(continuation) => execute_workflow_node(continuation, &next_ctx, functions),
                None => Ok(Value::Null),
            }
        }
        Workflow::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => match eval_expr(condition, ctx, functions)? {
            Value::Bool(true) => execute_workflow_node(then_branch, ctx, functions),
            Value::Bool(false) => {
                if let Some(else_branch) = else_branch {
                    execute_workflow_node(else_branch, ctx, functions)
                } else {
                    Ok(Value::Null)
                }
            }
            other => Err(ExecError::ExecutionFailed(format!(
                "workflow if condition must be Bool, got {other:?}"
            ))),
        },
        Workflow::Seq { first, second, .. } => {
            let _ = execute_workflow_node(first, ctx, functions)?;
            execute_workflow_node(second, ctx, functions)
        }
        other => Err(ExecError::ExecutionFailed(format!(
            "pure-functions runtime integration does not yet execute workflow form {other:?}"
        ))),
    }
}

fn eval_expr(expr: &Expr, ctx: &Context, functions: &HashMap<String, FnDef>) -> ExecResult<Value> {
    match expr {
        Expr::Literal(literal) => Ok(eval_literal(literal)),
        Expr::Variable(name) => ctx
            .get(name.as_ref())
            .cloned()
            .ok_or_else(|| ExecError::ExecutionFailed(format!("undefined variable: {name}"))),
        Expr::Unary { op, operand, .. } => {
            let operand = eval_expr(operand, ctx, functions)?;
            match (op, operand) {
                (UnaryOp::Not, Value::Bool(value)) => Ok(Value::Bool(!value)),
                (UnaryOp::Neg, Value::Int(value)) => Ok(Value::Int(-value)),
                _ => Err(ExecError::ExecutionFailed(
                    "invalid unary operation".to_string(),
                )),
            }
        }
        Expr::Binary {
            op, left, right, ..
        } => {
            let left = eval_expr(left, ctx, functions)?;
            let right = eval_expr(right, ctx, functions)?;
            eval_binary(*op, left, right)
        }
        Expr::Call {
            func, module, args, ..
        } => {
            let callee = module
                .as_ref()
                .map(|module| format!("{module}::{func}"))
                .unwrap_or_else(|| func.to_string());
            let function = functions
                .get(&callee)
                .or_else(|| functions.get(func.as_ref()))
                .ok_or_else(|| ExecError::ExecutionFailed(format!("unknown function: {callee}")))?;
            let values = args
                .iter()
                .map(|arg| eval_expr(arg, ctx, functions))
                .collect::<Result<Vec<_>, _>>()?;
            eval_function(function, &values, functions)
        }
        Expr::Match {
            scrutinee, arms, ..
        } => {
            let value = eval_expr(scrutinee, ctx, functions)?;
            for arm in arms {
                if let Ok(bindings) = bind_pattern(&arm.pattern, &value) {
                    let mut arm_ctx = ctx.extend();
                    arm_ctx.set_many(bindings);
                    return eval_expr(&arm.body, &arm_ctx, functions);
                }
            }
            Err(ExecError::ExecutionFailed(format!(
                "non-exhaustive match for value {value:?}"
            )))
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => match eval_expr(condition, ctx, functions)? {
            Value::Bool(true) => eval_expr(then_branch, ctx, functions),
            Value::Bool(false) => match else_branch {
                Some(else_branch) => eval_expr(else_branch, ctx, functions),
                None => Ok(Value::Null),
            },
            other => Err(ExecError::ExecutionFailed(format!(
                "if condition must be Bool, got {other:?}"
            ))),
        },
        Expr::Panic { message, .. } => {
            Err(ExecError::ExecutionFailed(format!("panic: {}", message)))
        }
        Expr::Block {
            statements,
            tail_expr,
            ..
        } => {
            let mut block_ctx = ctx.extend();
            for statement in statements {
                let BlockStmt::Let { pattern, expr, .. } = statement;
                let value = eval_expr(expr, &block_ctx, functions)?;
                let bindings = bind_pattern(pattern, &value)?;
                block_ctx.set_many(bindings);
            }
            match tail_expr {
                Some(tail_expr) => eval_expr(tail_expr, &block_ctx, functions),
                None => Ok(Value::Null),
            }
        }
        Expr::Constructor {
            name,
            fields,
            payload,
            ..
        } => {
            let evaluated_fields = match payload {
                ConstructorPayload::Unit => fields
                    .iter()
                    .map(|(name, expr)| {
                        eval_expr(expr, ctx, functions).map(|value| (name.to_string(), value))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                ConstructorPayload::Record(fields) => fields
                    .iter()
                    .map(|(name, expr)| {
                        eval_expr(expr, ctx, functions).map(|value| (name.to_string(), value))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                ConstructorPayload::Tuple(items) => items
                    .iter()
                    .enumerate()
                    .map(|(index, expr)| {
                        eval_expr(expr, ctx, functions)
                            .map(|value| (ash_core::adt::tuple_field_name(index), value))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            };
            Ok(Value::Variant {
                name: name.to_string(),
                fields: Box::new(evaluated_fields),
            })
        }
        Expr::IfLet {
            pattern,
            expr,
            then_branch,
            else_branch,
            ..
        } => {
            let value = eval_expr(expr, ctx, functions)?;
            if let Ok(bindings) = bind_pattern(pattern, &value) {
                let mut nested = ctx.extend();
                nested.set_many(bindings);
                eval_expr(then_branch, &nested, functions)
            } else {
                eval_expr(else_branch, ctx, functions)
            }
        }
        other => Err(ExecError::ExecutionFailed(format!(
            "pure-functions runtime integration does not yet evaluate expression {other:?}"
        ))),
    }
}

fn eval_function(
    function: &FnDef,
    args: &[Value],
    functions: &HashMap<String, FnDef>,
) -> ExecResult<Value> {
    if function.params.len() != args.len() {
        return Err(ExecError::ExecutionFailed(format!(
            "wrong arity calling '{}': expected {}, got {}",
            function.name,
            function.params.len(),
            args.len()
        )));
    }

    let mut fn_ctx = Context::new();
    let mut requirement_ctx = RequirementContext::new();
    for (param, arg) in function.params.iter().zip(args.iter()) {
        fn_ctx.set(param.name.to_string(), arg.clone());
        if let Value::Int(value) = arg {
            requirement_ctx = requirement_ctx.with_fact(param.name.to_string(), *value);
        }
    }

    if let Some(contract) = function.contract.as_ref() {
        let lowered = ash_parser::lower_fn_contract(Some(contract))
            .map_err(|error| ExecError::ExecutionFailed(error.to_string()))?;
        let preconditions = check_contract(&lowered.contract, &requirement_ctx);
        if !preconditions.is_success() {
            let details = preconditions
                .errors()
                .into_iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            return Err(ExecError::ExecutionFailed(format!(
                "fn precondition failed for '{}': {details}",
                function.name
            )));
        }

        let result = eval_expr(&function.body, &fn_ctx, functions)?;
        let mut ensures_ctx = requirement_ctx.clone();
        if let Value::Int(value) = result {
            ensures_ctx = ensures_ctx.with_fact("result", value);
        }
        let ensures = check_contract(
            &ash_core::workflow_contract::Contract {
                requires: lowered
                    .runtime_postconditions
                    .predicates
                    .iter()
                    .filter_map(|predicate| match predicate {
                        ash_core::workflow_contract::PostPredicate::ResultSatisfies(constraint) => {
                            Some(ash_core::workflow_contract::Requirement::Arithmetic {
                                var: "result".to_string(),
                                constraint: constraint.clone(),
                            })
                        }
                        _ => None,
                    })
                    .collect(),
                ensures: vec![],
            },
            &ensures_ctx,
        );
        if !ensures.is_success() {
            let details = ensures
                .errors()
                .into_iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            return Err(ExecError::ExecutionFailed(format!(
                "fn ensures failed for '{}': {details}",
                function.name
            )));
        }
        for predicate in lowered.runtime_postconditions.predicates {
            if let ash_core::workflow_contract::PostPredicate::Eq(left, right) = predicate {
                let left = eval_post_value(&left, &fn_ctx, result.clone());
                let right = eval_post_value(&right, &fn_ctx, result.clone());
                if left != right {
                    return Err(ExecError::ExecutionFailed(format!(
                        "fn ensures failed for '{}': expected {left:?} == {right:?}",
                        function.name
                    )));
                }
            }
        }
        Ok(result)
    } else {
        eval_expr(&function.body, &fn_ctx, functions)
    }
}

fn eval_post_value(name: &str, ctx: &Context, result: Value) -> Value {
    if name == "result" {
        return result;
    }
    if name == "true" {
        return Value::Bool(true);
    }
    if name == "false" {
        return Value::Bool(false);
    }
    if name == "null" {
        return Value::Null;
    }
    if let Ok(value) = name.parse::<i64>() {
        return Value::Int(value);
    }
    ctx.get(name)
        .cloned()
        .unwrap_or(Value::String(name.to_string()))
}

fn bind_pattern(pattern: &Pattern, value: &Value) -> ExecResult<HashMap<String, Value>> {
    let core_pattern = ash_parser::lower_pattern(pattern)
        .map_err(|error| ExecError::ExecutionFailed(error.to_string()))?;
    ash_interp::match_pattern(&core_pattern, value)
        .map_err(|error| ExecError::ExecutionFailed(format!("pattern match failed: {error}")))
}

fn eval_literal(literal: &Literal) -> Value {
    match literal {
        Literal::Int(value) => Value::Int(*value),
        Literal::Float(value) => Value::String(value.to_string()),
        Literal::String(value) => Value::String(value.to_string()),
        Literal::Bool(value) => Value::Bool(*value),
        Literal::List(items) => Value::List(Box::new(items.iter().map(eval_literal).collect())),
        Literal::Null => Value::Null,
    }
}

fn eval_binary(op: BinaryOp, left: Value, right: Value) -> ExecResult<Value> {
    match op {
        BinaryOp::Add => match (left, right) {
            (Value::Int(left), Value::Int(right)) => Ok(Value::Int(left + right)),
            (Value::String(left), Value::String(right)) => {
                Ok(Value::String(format!("{left}{right}")))
            }
            _ => Err(ExecError::ExecutionFailed(
                "invalid add operands".to_string(),
            )),
        },
        BinaryOp::Sub => ints(left, right, |l, r| Value::Int(l - r)),
        BinaryOp::Mul => ints(left, right, |l, r| Value::Int(l * r)),
        BinaryOp::Div => ints_checked(left, right, |l, r| l.checked_div(r).map(Value::Int)),
        BinaryOp::Mod => ints_checked(left, right, |l, r| l.checked_rem(r).map(Value::Int)),
        BinaryOp::And => bools(left, right, |l, r| Value::Bool(l && r)),
        BinaryOp::Or => bools(left, right, |l, r| Value::Bool(l || r)),
        BinaryOp::Eq => Ok(Value::Bool(left == right)),
        BinaryOp::Neq => Ok(Value::Bool(left != right)),
        BinaryOp::Lt => ints(left, right, |l, r| Value::Bool(l < r)),
        BinaryOp::Gt => ints(left, right, |l, r| Value::Bool(l > r)),
        BinaryOp::Leq => ints(left, right, |l, r| Value::Bool(l <= r)),
        BinaryOp::Geq => ints(left, right, |l, r| Value::Bool(l >= r)),
        BinaryOp::In => Err(ExecError::ExecutionFailed(
            "'in' not supported in pure runtime".to_string(),
        )),
    }
}

fn ints(left: Value, right: Value, f: impl FnOnce(i64, i64) -> Value) -> ExecResult<Value> {
    match (left, right) {
        (Value::Int(left), Value::Int(right)) => Ok(f(left, right)),
        _ => Err(ExecError::ExecutionFailed(
            "integer operands required".to_string(),
        )),
    }
}

fn ints_checked(
    left: Value,
    right: Value,
    f: impl FnOnce(i64, i64) -> Option<Value>,
) -> ExecResult<Value> {
    match (left, right) {
        (Value::Int(left), Value::Int(right)) => f(left, right)
            .ok_or_else(|| ExecError::ExecutionFailed("panic: division by zero".to_string())),
        _ => Err(ExecError::ExecutionFailed(
            "integer operands required".to_string(),
        )),
    }
}

fn bools(left: Value, right: Value, f: impl FnOnce(bool, bool) -> Value) -> ExecResult<Value> {
    match (left, right) {
        (Value::Bool(left), Value::Bool(right)) => Ok(f(left, right)),
        _ => Err(ExecError::ExecutionFailed(
            "boolean operands required".to_string(),
        )),
    }
}
