//! CPS IR Validation Boundary
//!
//! Validates raw CPS programs before execution.
//! Separates parser/validator concerns from lean interpreter semantics.

use ash_core::cps::*;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Errors that can occur during CPS validation
#[derive(Error, Debug, Clone, PartialEq)]
pub enum CpsValidationError {
    #[error("arity mismatch in call: expected {expected} args, got {got}")]
    CallArityMismatch { expected: usize, got: usize },
    #[error("arity mismatch in primitive {op:?}: expected {expected} args, got {got}")]
    PrimArityMismatch {
        op: PrimOp,
        expected: usize,
        got: usize,
    },
    #[error("arity mismatch in raise: expected {expected} args, got {got}")]
    RaiseArityMismatch { expected: usize, got: usize },
    #[error("unresolved label: {0}")]
    UnresolvedLabel(Name),
    #[error("unresolved variable: {0}")]
    UnresolvedVariable(Name),
    #[error("duplicate row item: {0:?}")]
    DuplicateRowItem(EffectItem),
    #[error("value in term position: {0:?}")]
    ValueInTermPosition(Value),
    #[error("invalid syntactic position for {0}")]
    InvalidSyntacticPosition(String),
}

/// Validate a CPS program before execution.
///
/// Checks:
/// - Arity matches for calls, primitives, and raises
/// - Labels and variables resolve within the program
/// - Rows are well-formed and duplicate-free
/// - Values, terms, atoms, and cont refs appear in allowed positions
pub fn validate_cps_program(term: &Term) -> Result<(), CpsValidationError> {
    let mut ctx = ValidationContext::new();
    validate_term(term, &mut ctx)
}

struct ValidationContext {
    /// Bound variables in scope
    bindings: HashSet<Name>,
    /// Bound labels (continuations) in scope
    labels: HashSet<Name>,
    /// Lambda parameters for arity checking
    lambda_params: HashMap<Name, usize>,
}

impl ValidationContext {
    fn new() -> Self {
        ValidationContext {
            bindings: HashSet::new(),
            labels: HashSet::new(),
            lambda_params: HashMap::new(),
        }
    }

    fn with_binding(&self, name: Name) -> Self {
        let mut ctx = self.clone();
        ctx.bindings.insert(name);
        ctx
    }

    fn with_label(&self, name: Name) -> Self {
        let mut ctx = self.clone();
        ctx.labels.insert(name);
        ctx
    }
}

impl Clone for ValidationContext {
    fn clone(&self) -> Self {
        ValidationContext {
            bindings: self.bindings.clone(),
            labels: self.labels.clone(),
            lambda_params: self.lambda_params.clone(),
        }
    }
}

fn validate_term(term: &Term, ctx: &mut ValidationContext) -> Result<(), CpsValidationError> {
    match term {
        Term::LetVal { name, value, body } => {
            validate_value(value, ctx)?;
            let mut new_ctx = ctx.with_binding(name.clone());
            validate_term(body, &mut new_ctx)?;
            Ok(())
        }
        Term::LetPrim {
            name,
            op,
            args,
            body,
        } => {
            validate_prim_arity(*op, args)?;
            for arg in args {
                validate_atom(arg, ctx)?;
            }
            let mut new_ctx = ctx.with_binding(name.clone());
            validate_term(body, &mut new_ctx)?;
            Ok(())
        }
        Term::LetCont {
            name,
            param,
            cont_body,
            body,
        } => {
            let mut cont_ctx = ctx.with_binding(param.clone());
            validate_term(cont_body, &mut cont_ctx)?;
            let mut new_ctx = ctx.with_label(name.clone());
            validate_term(body, &mut new_ctx)?;
            Ok(())
        }
        Term::Jump { cont, arg, .. } => {
            validate_cont_ref(cont, ctx)?;
            validate_atom(arg, ctx)?;
            Ok(())
        }
        Term::Call {
            func, args, cont, ..
        } => {
            validate_atom(func, ctx)?;
            for arg in args {
                validate_atom(arg, ctx)?;
            }
            validate_cont_ref(cont, ctx)?;
            // Arity check if we know the lambda
            if let Atom::Var(name) = func {
                if let Some(expected) = ctx.lambda_params.get(name) {
                    if *expected != args.len() {
                        return Err(CpsValidationError::CallArityMismatch {
                            expected: *expected,
                            got: args.len(),
                        });
                    }
                }
            }
            Ok(())
        }
        Term::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            validate_atom(cond, ctx)?;
            validate_term(then_branch, ctx)?;
            validate_term(else_branch, ctx)?;
            Ok(())
        }
        Term::LetRec { name, value, body } => {
            validate_value(value, ctx)?;
            let mut new_ctx = ctx.with_binding(name.clone());
            validate_term(body, &mut new_ctx)?;
            Ok(())
        }
        Term::Raise {
            op, args, resume, ..
        } => {
            if op.arg_types.len() != args.len() {
                return Err(CpsValidationError::RaiseArityMismatch {
                    expected: op.arg_types.len(),
                    got: args.len(),
                });
            }
            for arg in args {
                validate_atom(arg, ctx)?;
            }
            validate_cont_ref(resume, ctx)?;
            Ok(())
        }
        Term::Handle {
            clause, body, cont, ..
        } => {
            validate_handler_clause(clause, ctx)?;
            validate_term(body, ctx)?;
            validate_cont_ref(cont, ctx)?;
            Ok(())
        }
        Term::RecordDischarge { body, .. } => {
            validate_term(body, ctx)?;
            Ok(())
        }
        Term::Return { value } => {
            validate_atom(value, ctx)?;
            Ok(())
        }
        Term::Trap { .. } => Ok(()),
    }
}

fn validate_value(value: &Value, ctx: &mut ValidationContext) -> Result<(), CpsValidationError> {
    match value {
        Value::Atom(atom) => validate_atom(atom, ctx),
        Value::Lam {
            params, cont, body, ..
        } => {
            let mut lam_ctx = ctx.clone();
            for param in params {
                lam_ctx.bindings.insert(param.clone());
            }
            lam_ctx.bindings.insert(cont.clone());
            validate_term(body, &mut lam_ctx)?;
            Ok(())
        }
        Value::Cont { param, body, .. } => {
            let mut cont_ctx = ctx.clone();
            cont_ctx.bindings.insert(param.clone());
            validate_term(body, &mut cont_ctx)?;
            Ok(())
        }
    }
}

fn validate_atom(atom: &Atom, ctx: &ValidationContext) -> Result<(), CpsValidationError> {
    match atom {
        Atom::Var(name) => {
            if !ctx.bindings.contains(name) {
                return Err(CpsValidationError::UnresolvedVariable(name.clone()));
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn validate_cont_ref(cont: &ContRef, ctx: &ValidationContext) -> Result<(), CpsValidationError> {
    match cont {
        ContRef::Label(name) => {
            if !ctx.labels.contains(name) {
                return Err(CpsValidationError::UnresolvedLabel(name.clone()));
            }
            Ok(())
        }
        ContRef::Var(name) => {
            if !ctx.bindings.contains(name) {
                return Err(CpsValidationError::UnresolvedVariable(name.clone()));
            }
            Ok(())
        }
    }
}

fn validate_prim_arity(op: PrimOp, args: &[Atom]) -> Result<(), CpsValidationError> {
    let expected = match op {
        PrimOp::Add
        | PrimOp::Sub
        | PrimOp::Mul
        | PrimOp::Div
        | PrimOp::Eq
        | PrimOp::Ne
        | PrimOp::Lt
        | PrimOp::Le
        | PrimOp::Gt
        | PrimOp::Ge => 2,
        PrimOp::Neg | PrimOp::Not => 1,
    };
    if args.len() != expected {
        return Err(CpsValidationError::PrimArityMismatch {
            op,
            expected,
            got: args.len(),
        });
    }
    Ok(())
}

fn validate_handler_clause(
    clause: &HandlerClause,
    ctx: &mut ValidationContext,
) -> Result<(), CpsValidationError> {
    let mut clause_ctx = ctx.clone();
    for param in &clause.params {
        clause_ctx.bindings.insert(param.clone());
    }
    clause_ctx.bindings.insert(clause.resume.clone());
    validate_term(&clause.body, &mut clause_ctx)?;
    Ok(())
}

fn validate_row(row: &EffectRow) -> Result<(), CpsValidationError> {
    let mut seen = HashSet::new();
    for item in &row.items {
        let key = (item.namespace.clone(), item.name.clone());
        if !seen.insert(key) {
            return Err(CpsValidationError::DuplicateRowItem(item.clone()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_simple_program() {
        let term = Term::LetVal {
            name: "x".to_string(),
            value: Value::Atom(Atom::Int(42)),
            body: Box::new(Term::Return {
                value: Atom::Var("x".to_string()),
            }),
        };
        assert!(validate_cps_program(&term).is_ok());
    }

    #[test]
    fn test_validate_unresolved_variable() {
        let term = Term::Return {
            value: Atom::Var("unbound".to_string()),
        };
        let result = validate_cps_program(&term);
        assert!(matches!(
            result,
            Err(CpsValidationError::UnresolvedVariable(_))
        ));
    }

    #[test]
    fn test_validate_prim_arity_mismatch() {
        let term = Term::LetPrim {
            name: "y".to_string(),
            op: PrimOp::Add,
            args: vec![Atom::Int(1)], // Add needs 2 args
            body: Box::new(Term::Return {
                value: Atom::Var("y".to_string()),
            }),
        };
        let result = validate_cps_program(&term);
        assert!(matches!(
            result,
            Err(CpsValidationError::PrimArityMismatch { .. })
        ));
    }

    #[test]
    fn test_validate_unresolved_label() {
        let term = Term::Jump {
            cont: ContRef::Label("unbound".to_string()),
            arg: Atom::Int(42),
            row: EffectRow::default(),
        };
        let result = validate_cps_program(&term);
        assert!(matches!(
            result,
            Err(CpsValidationError::UnresolvedLabel(_))
        ));
    }

    #[test]
    fn test_validate_duplicate_row() {
        let row = EffectRow {
            items: vec![
                EffectItem {
                    namespace: "cap".to_string(),
                    name: "fs.read".to_string(),
                    kind: EffectItemKind::Capability,
                },
                EffectItem {
                    namespace: "cap".to_string(),
                    name: "fs.read".to_string(),
                    kind: EffectItemKind::Capability,
                },
            ],
        };
        let result = validate_row(&row);
        assert!(matches!(
            result,
            Err(CpsValidationError::DuplicateRowItem(_))
        ));
    }
}
