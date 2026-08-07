use ash_core::contract::{ArithConstraint, PostPredicate, Requirement};
use std::fmt;

use crate::surface::{BinaryOp, Expr, Literal};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractClassificationError {
    UnsupportedExpression,
    NonResultPostconditionTarget,
}

impl ContractClassificationError {
    pub fn requirement_message(&self) -> &'static str {
        match self {
            Self::UnsupportedExpression => {
                "Requirement classifier could not classify this workflow requires contract expression; supported forms include simple arithmetic predicates"
            }
            Self::NonResultPostconditionTarget => {
                "Requirement classifier could not classify this workflow requires contract expression"
            }
        }
    }

    pub fn postcondition_message(&self) -> &'static str {
        match self {
            Self::NonResultPostconditionTarget => {
                "OpenPostcondition for function ensures must target result; Function result postconditions currently require predicates over result"
            }
            Self::UnsupportedExpression => {
                "OpenPostcondition classifier could not classify this function ensures contract expression; Function result postconditions currently require a supported predicate over result"
            }
        }
    }
}

impl fmt::Display for ContractClassificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnsupportedExpression => "unsupported contract expression",
            Self::NonResultPostconditionTarget => "OpenPostcondition target is not result",
        })
    }
}

pub fn classify_requirement(expr: &Expr) -> Result<Requirement, ContractClassificationError> {
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
        Some(_) => Err(ContractClassificationError::NonResultPostconditionTarget),
        _ => Err(ContractClassificationError::UnsupportedExpression),
    }
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
