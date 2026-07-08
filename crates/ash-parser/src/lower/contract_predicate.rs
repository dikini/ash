//! Bridge from surface `Expr` to core `ContractPredicateExpr`.
//!
//! This module is TASK-1893 / TASK-1894 substrate. It converts parsed surface
//! predicate expressions into the core contract-position predicate AST, while
//! enforcing that predicates are authority-free and well-formed:
//!
//! * no capability/process/workflow/handler calls or operations
//! * no time/random observations or implicit force
//! * `old(...)` is rejected at the surface→core bridge (target-surface fn contracts
//!   do not support pre-state snapshots)
//! * `result` may only appear in `ensures` clauses
//! * only pure value-level expressions and admitted predicate calls are allowed
//!
//! The bridge does **not** call [`ash_core::core_ash_contract::lower_contract_predicate`];
//! that final lowering step is intentionally left for later phases.

use std::fmt;

use ash_core::{
    core_ash::{CoreName, CorePath, CoreSourceSpan, CoreType},
    core_ash_contract::{
        ContractPredicateExpr, CoreBoundaryId, PredicateBinderId, PredicateBinderRef,
        PredicateFunctionRef,
    },
};

use crate::surface::{BinaryOp, Expr, Literal, Spanned, UnaryOp};
use crate::token::Span;

/// Errors produced while translating a surface predicate expression into the
/// core contract-position subset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractPredicateBridgeError {
    /// The expression form is not allowed in a contract predicate.
    ForbiddenExpr { message: String, span: Option<Span> },
    /// `result` was used in a `requires` clause.
    ResultInRequires { span: Option<Span> },
    /// `old(...)` is not supported at the target surface for fn contracts.
    OldNotSupported { span: Option<Span> },
    /// A literal or operator type is not supported in the predicate subset.
    UnsupportedLiteralOrOp { message: String, span: Option<Span> },
}

impl fmt::Display for ContractPredicateBridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ForbiddenExpr { message, .. } => {
                write!(f, "forbidden expression in contract predicate: {message}")
            }
            Self::ResultInRequires { .. } => {
                write!(f, "`result` may only appear in `ensures` clauses")
            }
            Self::OldNotSupported { .. } => {
                write!(f, "`old(...)` snapshots are not supported in fn contracts")
            }
            Self::UnsupportedLiteralOrOp { message, .. } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for ContractPredicateBridgeError {}

/// Context governing which special binders are legal in a given clause.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredicateClauseKind {
    /// Precondition: parameters and global constants are visible; `result` is not.
    Requires,
    /// Postcondition: parameters, global constants, and `result` are visible.
    Ensures,
}

/// Translate a surface predicate expression into a core [`ContractPredicateExpr`].
///
/// * `boundary` identifies the contract boundary (used for binder ids).
/// * `clause_kind` controls whether `result` is accepted.
/// * `source_span` is used for generated binder references.
/// * `admitted_predicates` is the set of admitted predicate-function paths; only
///   calls whose path is in this set are accepted as pure predicate calls.
///
/// The returned expression is guaranteed to use only the authority-free subset
/// of [`ContractPredicateExpr`]; calls that would require provider/process/time
/// authority are rejected here.
#[allow(clippy::too_many_lines)]
pub fn surface_expr_to_contract_predicate(
    boundary: impl Into<CoreBoundaryId>,
    clause_kind: PredicateClauseKind,
    expr: &Expr,
    source_span: CoreSourceSpan,
    admitted_predicates: &[CorePath],
) -> Result<ContractPredicateExpr, ContractPredicateBridgeError> {
    let boundary = boundary.into();
    translate_expr(
        expr,
        &boundary,
        clause_kind,
        source_span,
        admitted_predicates,
    )
}

fn translate_expr(
    expr: &Expr,
    boundary: &CoreBoundaryId,
    clause_kind: PredicateClauseKind,
    source_span: CoreSourceSpan,
    admitted_predicates: &[CorePath],
) -> Result<ContractPredicateExpr, ContractPredicateBridgeError> {
    match expr {
        Expr::Literal(Literal::Bool(value)) => Ok(ContractPredicateExpr::BoolLit(*value)),
        Expr::Literal(Literal::Int(value)) => Ok(ContractPredicateExpr::IntLit(i128::from(*value))),
        Expr::Literal(Literal::String(value)) => {
            Ok(ContractPredicateExpr::StringLit(value.to_string()))
        }
        Expr::Literal(Literal::Null) => Ok(ContractPredicateExpr::UnitLit),
        Expr::Literal(Literal::Float(_) | Literal::List(_)) => {
            Err(ContractPredicateBridgeError::UnsupportedLiteralOrOp {
                message: "float and list literals are not supported in contract predicates"
                    .to_string(),
                span: Some(expr.span()),
            })
        }
        Expr::Variable { name, .. } => translate_variable(name, boundary, clause_kind, expr),
        Expr::FieldAccess { base, field, .. } => {
            let base = translate_expr(
                base,
                boundary,
                clause_kind,
                source_span.clone(),
                admitted_predicates,
            )?;
            Ok(ContractPredicateExpr::Field {
                base: Box::new(base),
                field: CoreName::from(field.as_ref()),
            })
        }
        Expr::IndexAccess { .. } => Err(ContractPredicateBridgeError::ForbiddenExpr {
            message: "index access is not supported in contract predicates".to_string(),
            span: Some(expr.span()),
        }),
        Expr::Unary { op, operand, .. } => {
            let operand = translate_expr(
                operand,
                boundary,
                clause_kind,
                source_span.clone(),
                admitted_predicates,
            )?;
            match op {
                UnaryOp::Not => Ok(ContractPredicateExpr::Not(Box::new(operand))),
                UnaryOp::Neg => Err(ContractPredicateBridgeError::UnsupportedLiteralOrOp {
                    message: "arithmetic negation is not supported in contract predicates"
                        .to_string(),
                    span: Some(expr.span()),
                }),
            }
        }
        Expr::Binary {
            op, left, right, ..
        } => translate_binary(
            *op,
            left,
            right,
            boundary,
            clause_kind,
            source_span.clone(),
            admitted_predicates,
            expr,
        ),
        Expr::Call {
            func, module, args, ..
        } => translate_call(
            func.as_ref(),
            module.as_deref(),
            args,
            boundary,
            clause_kind,
            source_span.clone(),
            admitted_predicates,
            expr,
        ),
        Expr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            let condition = translate_expr(
                condition,
                boundary,
                clause_kind,
                source_span.clone(),
                admitted_predicates,
            )?;
            let then_branch = translate_expr(
                then_branch,
                boundary,
                clause_kind,
                source_span.clone(),
                admitted_predicates,
            )?;
            let else_branch = match else_branch {
                Some(else_branch) => translate_expr(
                    else_branch,
                    boundary,
                    clause_kind,
                    source_span.clone(),
                    admitted_predicates,
                )?,
                None => ContractPredicateExpr::UnitLit,
            };
            // Surface `if c then t else e` desugars to `(c => t) && (!c => e)`.
            Ok(ContractPredicateExpr::And(
                Box::new(ContractPredicateExpr::Implies(
                    Box::new(condition.clone()),
                    Box::new(then_branch),
                )),
                Box::new(ContractPredicateExpr::Implies(
                    Box::new(ContractPredicateExpr::Not(Box::new(condition))),
                    Box::new(else_branch),
                )),
            ))
        }
        Expr::Record { fields, .. } => {
            let mut translated_fields = Vec::with_capacity(fields.len());
            for (name, value) in fields {
                let value = translate_expr(
                    value,
                    boundary,
                    clause_kind,
                    source_span.clone(),
                    admitted_predicates,
                )?;
                translated_fields.push((CoreName::from(name.as_ref()), value));
            }
            // Surface records are not part of the core predicate subset; reject.
            Err(ContractPredicateBridgeError::ForbiddenExpr {
                message: "record literals are not supported in contract predicates".to_string(),
                span: Some(expr.span()),
            })
        }
        Expr::Constructor { name, payload, .. } => translate_constructor(
            name.as_ref(),
            payload,
            boundary,
            clause_kind,
            source_span.clone(),
            admitted_predicates,
            expr,
        ),
        Expr::List { items, .. } => {
            let mut translated = Vec::with_capacity(items.len());
            for item in items {
                translated.push(translate_expr(
                    item,
                    boundary,
                    clause_kind,
                    source_span.clone(),
                    admitted_predicates,
                )?);
            }
            Err(ContractPredicateBridgeError::ForbiddenExpr {
                message: "list literals are not supported in contract predicates".to_string(),
                span: Some(expr.span()),
            })
        }
        Expr::Match { .. }
        | Expr::IfLet { .. }
        | Expr::Block { .. }
        | Expr::FnDef { .. }
        | Expr::FnApply { .. }
        | Expr::DoBlock { .. }
        | Expr::Comprehension { .. }
        | Expr::OperatorSection { .. }
        | Expr::MacroInvocation { .. }
        | Expr::Policy(_)
        | Expr::CheckObligation { .. }
        | Expr::Panic { .. }
        | Expr::Fail { .. }
        | Expr::WithError { .. } => Err(ContractPredicateBridgeError::ForbiddenExpr {
            message: format!("{expr:?} is not allowed in contract predicates"),
            span: Some(expr.span()),
        }),
    }
}

fn translate_variable(
    name: &crate::surface::Name,
    boundary: &CoreBoundaryId,
    clause_kind: PredicateClauseKind,
    expr: &Expr,
) -> Result<ContractPredicateExpr, ContractPredicateBridgeError> {
    if name.as_ref() == "result" {
        if clause_kind == PredicateClauseKind::Requires {
            return Err(ContractPredicateBridgeError::ResultInRequires {
                span: Some(expr.span()),
            });
        }
        let binder = PredicateBinderId::new(boundary.clone(), "result");
        return Ok(ContractPredicateExpr::Result(PredicateBinderRef::new(
            binder,
        )));
    }
    if name.as_ref() == "message" {
        let binder = PredicateBinderId::new(boundary.clone(), "message");
        return Ok(ContractPredicateExpr::Message(PredicateBinderRef::new(
            binder,
        )));
    }
    let binder = PredicateBinderId::new(boundary.clone(), name.as_ref());
    Ok(ContractPredicateExpr::Binder(PredicateBinderRef::new(
        binder,
    )))
}

#[allow(clippy::too_many_arguments)]
fn translate_binary(
    op: BinaryOp,
    left: &Expr,
    right: &Expr,
    boundary: &CoreBoundaryId,
    clause_kind: PredicateClauseKind,
    source_span: CoreSourceSpan,
    admitted_predicates: &[CorePath],
    expr: &Expr,
) -> Result<ContractPredicateExpr, ContractPredicateBridgeError> {
    let left = translate_expr(
        left,
        boundary,
        clause_kind,
        source_span.clone(),
        admitted_predicates,
    )?;
    let right = translate_expr(
        right,
        boundary,
        clause_kind,
        source_span.clone(),
        admitted_predicates,
    )?;
    let mk = |node: fn(
        Box<ContractPredicateExpr>,
        Box<ContractPredicateExpr>,
    ) -> ContractPredicateExpr| {
        Ok(node(Box::new(left.clone()), Box::new(right.clone())))
    };
    match op {
        BinaryOp::Eq => mk(ContractPredicateExpr::Eq),
        BinaryOp::Neq => mk(ContractPredicateExpr::Ne),
        BinaryOp::Lt => mk(ContractPredicateExpr::Lt),
        BinaryOp::Gt => mk(ContractPredicateExpr::Gt),
        BinaryOp::Leq => mk(ContractPredicateExpr::Le),
        BinaryOp::Geq => mk(ContractPredicateExpr::Ge),
        BinaryOp::Add => mk(ContractPredicateExpr::Add),
        BinaryOp::Sub => mk(ContractPredicateExpr::Sub),
        BinaryOp::Mul => mk(ContractPredicateExpr::Mul),
        BinaryOp::Div => mk(ContractPredicateExpr::Div),
        BinaryOp::Mod => mk(ContractPredicateExpr::Rem),
        BinaryOp::And => mk(ContractPredicateExpr::And),
        BinaryOp::Or => mk(ContractPredicateExpr::Or),
        BinaryOp::Pipe | BinaryOp::In => {
            Err(ContractPredicateBridgeError::UnsupportedLiteralOrOp {
                message: "pipe and membership operators are not supported in contract predicates"
                    .to_string(),
                span: Some(expr.span()),
            })
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn translate_call(
    func: &str,
    module: Option<&str>,
    args: &[Expr],
    boundary: &CoreBoundaryId,
    clause_kind: PredicateClauseKind,
    source_span: CoreSourceSpan,
    admitted_predicates: &[CorePath],
    expr: &Expr,
) -> Result<ContractPredicateExpr, ContractPredicateBridgeError> {
    // Reject authority-bearing call shapes at the surface bridge.
    if func == "old" && module.is_none() {
        return Err(ContractPredicateBridgeError::OldNotSupported {
            span: Some(expr.span()),
        });
    }
    if is_forbidden_operation_name(func) && module.is_none() {
        return Err(ContractPredicateBridgeError::ForbiddenExpr {
            message: format!(
                "`{func}(...)` requires authority and is not allowed in contract predicates"
            ),
            span: Some(expr.span()),
        });
    }

    let path = call_path(func, module);

    // Only calls whose path is explicitly admitted are accepted as predicate calls.
    if admitted_predicates.contains(&path) {
        let translated_args = args
            .iter()
            .map(|arg| {
                translate_expr(
                    arg,
                    boundary,
                    clause_kind,
                    source_span.clone(),
                    admitted_predicates,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let callee = PredicateFunctionRef::new(
            path,
            vec![], // arg_types are resolved later by type checking
            CoreType::Base("Bool".to_string()),
        );
        return Ok(ContractPredicateExpr::PredicateCall {
            callee,
            args: translated_args,
            smt_safe: true,
        });
    }

    // Any other call is rejected because it may hide capability/process/time authority.
    Err(ContractPredicateBridgeError::ForbiddenExpr {
        message: format!(
            "call `{}{func}` is not an admitted predicate function",
            module.map_or_else(String::new, |m| format!("{m}::"))
        ),
        span: Some(expr.span()),
    })
}

fn is_forbidden_operation_name(name: &str) -> bool {
    // These names are known to require authority in the surface syntax.
    matches!(
        name,
        "force"
            | "now"
            | "random"
            | "time"
            | "timestamp"
            | "observe"
            | "propose"
            | "act"
            | "check"
            | "send"
            | "receive"
            | "set"
            | "with"
            | "do"
    )
}

fn call_path(func: &str, module: Option<&str>) -> CorePath {
    let mut path = CorePath::new();
    if let Some(module) = module {
        path.push(CoreName::from(module));
    }
    path.push(CoreName::from(func));
    path
}

fn translate_constructor(
    name: &str,
    payload: &crate::surface::ConstructorPayload,
    boundary: &CoreBoundaryId,
    clause_kind: PredicateClauseKind,
    source_span: CoreSourceSpan,
    admitted_predicates: &[CorePath],
    expr: &Expr,
) -> Result<ContractPredicateExpr, ContractPredicateBridgeError> {
    // Constructors that are not built-in are rejected.
    if name != "Some" && name != "None" {
        return Err(ContractPredicateBridgeError::ForbiddenExpr {
            message: format!("constructor `{name}` is not supported in contract predicates"),
            span: Some(expr.span()),
        });
    }

    match payload {
        crate::surface::ConstructorPayload::Unit => {
            if name == "None" {
                Ok(ContractPredicateExpr::UnitLit)
            } else {
                Err(ContractPredicateBridgeError::ForbiddenExpr {
                    message: "constructor `Some` requires a payload".to_string(),
                    span: Some(expr.span()),
                })
            }
        }
        crate::surface::ConstructorPayload::Record(fields) => {
            let mut translated = Vec::with_capacity(fields.len());
            for (field_name, field_expr) in fields {
                let value = translate_expr(
                    field_expr,
                    boundary,
                    clause_kind,
                    source_span.clone(),
                    admitted_predicates,
                )?;
                translated.push((CoreName::from(field_name.as_ref()), value));
            }
            if name == "Some" && translated.len() == 1 && translated[0].0 == "value" {
                Ok(translated
                    .into_iter()
                    .next()
                    .map(|(_, v)| v)
                    .unwrap_or(ContractPredicateExpr::UnitLit))
            } else {
                Err(ContractPredicateBridgeError::ForbiddenExpr {
                    message: "only `Some { value: ... }` is supported in contract predicates"
                        .to_string(),
                    span: Some(expr.span()),
                })
            }
        }
        crate::surface::ConstructorPayload::Tuple(items) => {
            if name == "Some" && items.len() == 1 {
                translate_expr(
                    &items[0],
                    boundary,
                    clause_kind,
                    source_span,
                    admitted_predicates,
                )
            } else {
                Err(ContractPredicateBridgeError::ForbiddenExpr {
                    message:
                        "only `Some(...)` with one argument is supported in contract predicates"
                            .to_string(),
                    span: Some(expr.span()),
                })
            }
        }
    }
}
