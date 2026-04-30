use ash_core::workflow_contract::{ArithConstraint, PostPredicate, Requirement, RolePolicy};

use crate::surface::{BinaryOp, Expr, Literal};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractClassificationError {
    UnsupportedExpression,
    EmptyAnyRole,
}

pub fn classify_requirement(expr: &Expr) -> Result<Requirement, ContractClassificationError> {
    if let Some(role) = classify_role_call(expr) {
        return Ok(Requirement::HasRole(role));
    }
    if let Some(roles) = classify_any_role_call(expr)? {
        return Ok(Requirement::AnyRole(RolePolicy { roles }));
    }
    if let Some((var, constraint)) = classify_arithmetic_predicate(expr) {
        return Ok(Requirement::Arithmetic { var, constraint });
    }
    Err(ContractClassificationError::UnsupportedExpression)
}

pub fn classify_postcondition(expr: &Expr) -> Result<PostPredicate, ContractClassificationError> {
    match classify_arithmetic_predicate(expr) {
        Some((var, constraint)) if var == "result" => {
            Ok(PostPredicate::ResultSatisfies(constraint))
        }
        _ => Err(ContractClassificationError::UnsupportedExpression),
    }
}

fn classify_role_call(expr: &Expr) -> Option<String> {
    let Expr::Call {
        func,
        args,
        module: None,
        ..
    } = expr
    else {
        return None;
    };
    if func.as_ref() != "role" || args.len() != 1 {
        return None;
    }
    symbolic_name(&args[0])
}

fn classify_any_role_call(expr: &Expr) -> Result<Option<Vec<String>>, ContractClassificationError> {
    let Expr::Call {
        func,
        args,
        module: None,
        ..
    } = expr
    else {
        return Ok(None);
    };
    if func.as_ref() != "any_role" || args.len() != 1 {
        return Ok(None);
    }
    let Expr::List { items, .. } = &args[0] else {
        return Err(ContractClassificationError::UnsupportedExpression);
    };
    if items.is_empty() {
        return Err(ContractClassificationError::EmptyAnyRole);
    }
    let mut roles = Vec::with_capacity(items.len());
    for item in items {
        roles.push(symbolic_name(item).ok_or(ContractClassificationError::UnsupportedExpression)?);
    }
    Ok(Some(roles))
}

fn classify_arithmetic_predicate(expr: &Expr) -> Option<(String, ArithConstraint)> {
    let Expr::Binary {
        left, op, right, ..
    } = expr
    else {
        return None;
    };
    let var = symbolic_name(left)?;
    let value = int_literal(right)?;
    let constraint = match op {
        BinaryOp::Gt => ArithConstraint::Gt(value),
        BinaryOp::Lt => ArithConstraint::Lt(value),
        BinaryOp::Geq => ArithConstraint::Gte(value),
        BinaryOp::Leq => ArithConstraint::Lte(value),
        BinaryOp::Eq => ArithConstraint::Eq(value),
        BinaryOp::Neq => ArithConstraint::NotEq(value),
        _ => return None,
    };
    Some((var, constraint))
}

fn symbolic_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Variable { name, .. } => Some(name.to_string()),
        Expr::Literal(Literal::String(value)) => Some(value.to_string()),
        _ => None,
    }
}

fn int_literal(expr: &Expr) -> Option<i64> {
    match expr {
        Expr::Literal(Literal::Int(value)) => Some(*value),
        _ => None,
    }
}
