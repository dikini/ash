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
    #[allow(dead_code)]
    #[error("value in term position: {0:?}")]
    ValueInTermPosition(Value),
    #[error("arity mismatch in handler: expected {expected} params, got {got}")]
    HandlerArityMismatch { expected: usize, got: usize },
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
#[allow(clippy::result_large_err)]
pub fn validate_cps_program(term: &Term) -> Result<(), CpsValidationError> {
    validate_cps_program_with_bindings(term, &[])
}

/// Validate an open CPS body with checker-provided parameter bindings.
///
/// Module transport stores callable bodies as non-authorizing terms rather
/// than lambdas. A selected non-root callable may therefore retain its
/// finalized parameters as free variables until Engine linking binds the
/// checked call arguments. This validator accepts only those explicit
/// parameter names; every other variable remains fail-closed.
#[allow(clippy::result_large_err)]
pub fn validate_cps_program_with_bindings(
    term: &Term,
    bindings: &[Name],
) -> Result<(), CpsValidationError> {
    let mut ctx = ValidationContext::new();
    ctx.bindings.extend(bindings.iter().cloned());
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

#[allow(clippy::collapsible_if)]
#[allow(clippy::result_large_err)]
fn validate_term(term: &Term, ctx: &mut ValidationContext) -> Result<(), CpsValidationError> {
    match term {
        Term::LetVal { name, value, body } => {
            validate_value(value, ctx)?;
            let mut new_ctx = ctx.with_binding(name.clone());
            // Record lambda arity for call validation
            if let Value::Lam { params, .. } = value {
                new_ctx.lambda_params.insert(name.clone(), params.len());
            }
            validate_term(body, &mut new_ctx)?;
            Ok(())
        }
        Term::LetPrim {
            name,
            op,
            args,
            body,
        } => {
            validate_prim_arity(op.clone(), args)?;
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
            row,
            multiplicity,
        } => {
            // Validate row and multiplicity legality for LetCont.
            validate_cont_value(row, multiplicity, cont_body)?;
            let mut cont_ctx = ctx.with_binding(param.clone());
            validate_term(cont_body, &mut cont_ctx)?;
            let mut new_ctx = ctx.with_label(name.clone());
            validate_term(body, &mut new_ctx)?;
            Ok(())
        }
        Term::Jump { cont, arg, row } => {
            validate_cont_ref(cont, ctx)?;
            validate_atom(arg, ctx)?;
            validate_row(row)?;
            Ok(())
        }
        Term::JumpValue { cont, arg, row } => {
            validate_cont_ref(cont, ctx)?;
            validate_value(arg, ctx)?;
            validate_row(row)?;
            Ok(())
        }
        Term::Call {
            func,
            args,
            cont,
            row,
            ..
        } => {
            validate_atom(func, ctx)?;
            for arg in args {
                validate_atom(arg, ctx)?;
            }
            validate_cont_ref(cont, ctx)?;
            if let Atom::Var(func_name) = func {
                let expected = ctx.lambda_params.get(func_name);
                if let Some(expected) = expected {
                    if *expected != args.len() {
                        return Err(CpsValidationError::CallArityMismatch {
                            expected: *expected,
                            got: args.len(),
                        });
                    }
                }
            }
            validate_row(row)?;
            Ok(())
        }
        Term::If {
            cond,
            then_branch,
            else_branch,
            row,
            ..
        } => {
            validate_atom(cond, ctx)?;
            validate_term(then_branch, ctx)?;
            validate_term(else_branch, ctx)?;
            validate_row(row)?;
            Ok(())
        }
        Term::LetRec { name, value, body } => {
            // Bind the recursive name BEFORE validating the value and body
            let mut new_ctx = ctx.with_binding(name.clone());
            // Record lambda arity for call validation
            if let Value::Lam { params, .. } = value {
                new_ctx.lambda_params.insert(name.clone(), params.len());
            }
            validate_value(value, &mut new_ctx)?;
            validate_term(body, &mut new_ctx)?;
            Ok(())
        }
        Term::Raise {
            op,
            args,
            resume,
            row,
            ..
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
            validate_row(row)?;
            Ok(())
        }
        Term::Handle {
            clause,
            body,
            cont,
            row,
            ..
        } => {
            validate_handler_clause(clause, ctx)?;
            validate_term(body, ctx)?;
            validate_cont_ref(cont, ctx)?;
            validate_row(row)?;
            Ok(())
        }
        Term::RecordDischarge { body, .. } => {
            validate_term(body, ctx)?;
            Ok(())
        }
        Term::Return { value } => {
            validate_value(value, ctx)?;
            Ok(())
        }
        Term::Match {
            scrutinee,
            arms,
            default,
        } => {
            validate_atom(scrutinee, ctx)?;
            for (_, arm_body) in arms {
                validate_term(arm_body, ctx)?;
            }
            if let Some(default_body) = default {
                validate_term(default_body, ctx)?;
            }
            Ok(())
        }
        Term::LetContCall {
            name,
            cont,
            arg,
            row,
            body,
        } => {
            validate_row(row)?;
            validate_atom(arg, ctx)?;
            let mut new_ctx = ctx.with_binding(name.clone());
            // The continuation being invoked must be in scope as a binding.
            if !ctx.bindings.contains(cont) {
                return Err(CpsValidationError::UnresolvedVariable(cont.clone()));
            }
            validate_term(body, &mut new_ctx)?;
            Ok(())
        }
        Term::Trap { .. } => Ok(()),
    }
}

/// Validate continuation value legality.
///
/// - Multi-shot-pure continuations require a closed empty declared row.
/// - Affine continuations are valid with any row.
#[allow(clippy::result_large_err)]
fn validate_cont_value(
    row: &EffectRow,
    multiplicity: &ContMultiplicity,
    body: &Term,
) -> Result<(), CpsValidationError> {
    validate_row(row)?;
    if *multiplicity == ContMultiplicity::MultiShotPure {
        // Declared row must be closed empty.
        if !row.items.is_empty() {
            return Err(CpsValidationError::InvalidSyntacticPosition(format!(
                "multi-shot-pure continuation must declare a closed empty row, got {row:?}"
            )));
        }
        // Effective row of body must also be empty (or unresolvable -> reject).
        let effective = effective_term_row(body);
        if !effective.items.is_empty() {
            return Err(CpsValidationError::InvalidSyntacticPosition(format!(
                "multi-shot-pure continuation body has non-empty effective row {effective:?}"
            )));
        }
    }
    Ok(())
}

/// Best-effort effective row computation for a continuation body.
///
/// Returns the row of the innermost terminal term if statically resolvable.
/// For `Jump`, `Call`, `Raise`, `Handle`, `If` — returns the row field.
/// For `LetVal`, `LetPrim`, `LetCont`, `LetRec`, `LetContCall`, `RecordDischarge`
/// — recurses into the body. For `Return` and `Trap` — returns empty.
fn effective_term_row(term: &Term) -> EffectRow {
    match term {
        Term::Jump { row, .. }
        | Term::Call { row, .. }
        | Term::Raise { row, .. }
        | Term::Handle { row, .. }
        | Term::If { row, .. } => row.clone(),
        Term::LetVal { body, .. }
        | Term::LetPrim { body, .. }
        | Term::LetCont { body, .. }
        | Term::LetRec { body, .. }
        | Term::LetContCall { body, .. }
        | Term::RecordDischarge { body, .. } => effective_term_row(body),
        Term::JumpValue { row, .. } => row.clone(),
        Term::Return { .. } | Term::Trap { .. } => EffectRow::default(),
        Term::Match { arms, default, .. } => {
            // Return the row of the first arm, or empty if none.
            if let Some((_, arm_body)) = arms.first() {
                effective_term_row(arm_body)
            } else if let Some(default_body) = default {
                effective_term_row(default_body)
            } else {
                EffectRow::default()
            }
        }
    }
}

/// Validate handler clause resume row/multiplicity legality.
///
/// - Multi-shot-pure resumes require a known closed empty row.
/// - Inherited target rows are affine-only.
/// - Known non-empty rows are valid only for affine.
#[allow(clippy::result_large_err)]
fn validate_resume_metadata(
    resume_row: &ResumeRowMetadata,
    resume_multiplicity: &ContMultiplicity,
) -> Result<(), CpsValidationError> {
    if *resume_multiplicity == ContMultiplicity::MultiShotPure {
        match resume_row {
            ResumeRowMetadata::Known(row) => {
                if !row.items.is_empty() {
                    return Err(CpsValidationError::InvalidSyntacticPosition(format!(
                        "multi-shot-pure resume requires a known empty row, got {row:?}"
                    )));
                }
            }
            ResumeRowMetadata::InheritFromTarget => {
                return Err(CpsValidationError::InvalidSyntacticPosition(
                    "multi-shot-pure resume requires a known row; inherited target row is not valid"
                        .to_string(),
                ));
            }
        }
    }
    Ok(())
}

#[allow(clippy::result_large_err)]
fn validate_value(value: &Value, ctx: &mut ValidationContext) -> Result<(), CpsValidationError> {
    match value {
        Value::Atom(atom) => validate_atom(atom, ctx),
        Value::Lam {
            params,
            cont,
            body,
            row,
            ..
        } => {
            validate_row(row)?;
            let mut lam_ctx = ctx.clone();
            for param in params {
                lam_ctx.bindings.insert(param.clone());
            }
            lam_ctx.bindings.insert(cont.clone());
            validate_term(body, &mut lam_ctx)?;
            Ok(())
        }
        Value::Cont {
            body,
            row,
            multiplicity,
            ..
        } => {
            validate_cont_value(row, multiplicity, body)?;
            Ok(())
        }
        Value::ThunkClosure {
            body,
            captured_env: _,
            captured_chain: _,
            row,
            ..
        } => {
            if let Value::Lam {
                params,
                cont,
                body: lam_body,
                row: lam_row,
                ..
            } = body.as_ref()
            {
                if !params.is_empty() {
                    return Err(CpsValidationError::InvalidSyntacticPosition(
                        "thunk closure body must be a zero-argument lambda".to_string(),
                    ));
                }
                if lam_row != row {
                    return Err(CpsValidationError::InvalidSyntacticPosition(
                        "thunk closure body and closure rows must match".to_string(),
                    ));
                }
                let _ = lam_body;
                let _ = cont;
                // Reuse existing lambda validation behavior for parameter/body checking.
                let mut lam_ctx = ctx.clone();
                lam_ctx.bindings.insert(cont.clone());
                validate_term(lam_body, &mut lam_ctx)?;
                Ok(())
            } else {
                Err(CpsValidationError::InvalidSyntacticPosition(
                    "thunk closure body must be a lambda".to_string(),
                ))
            }
        }
        Value::Record { fields } => {
            for (_, field_value) in fields {
                validate_value(field_value, ctx)?;
            }
            Ok(())
        }
        Value::Tuple { elems } => {
            for elem in elems {
                validate_value(elem, ctx)?;
            }
            Ok(())
        }
        Value::Constructor { fields, .. } => {
            for (_, field_value) in fields {
                validate_value(field_value, ctx)?;
            }
            Ok(())
        }
    }
}

#[allow(clippy::result_large_err)]
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

#[allow(clippy::result_large_err)]
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

#[allow(clippy::result_large_err)]
fn validate_prim_arity(op: PrimOp, args: &[Atom]) -> Result<(), CpsValidationError> {
    let expected = match op {
        PrimOp::Add
        | PrimOp::Sub
        | PrimOp::Mul
        | PrimOp::Div
        | PrimOp::Rem
        | PrimOp::Eq
        | PrimOp::Ne
        | PrimOp::Lt
        | PrimOp::Le
        | PrimOp::Gt
        | PrimOp::Ge => 2,
        PrimOp::Neg | PrimOp::Not => 1,
        PrimOp::RecordGet(_) | PrimOp::TupleGet(_) => 1,
        PrimOp::ForceThunk => 1,
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

#[allow(clippy::result_large_err)]
fn validate_handler_clause(
    clause: &HandlerClause,
    ctx: &mut ValidationContext,
) -> Result<(), CpsValidationError> {
    validate_row(&clause.row)?;
    // Validate resume row/multiplicity legality (SPEC-102 §6.4).
    validate_resume_metadata(&clause.resume_row, &clause.resume_multiplicity)?;
    // Check handler parameter arity matches effect operation argument types
    if clause.params.len() != clause.op.arg_types.len() {
        return Err(CpsValidationError::HandlerArityMismatch {
            expected: clause.op.arg_types.len(),
            got: clause.params.len(),
        });
    }
    let mut clause_ctx = ctx.clone();
    for param in &clause.params {
        clause_ctx.bindings.insert(param.clone());
    }
    clause_ctx.bindings.insert(clause.resume.clone());
    validate_term(&clause.body, &mut clause_ctx)?;
    Ok(())
}

#[allow(clippy::result_large_err)]
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
                value: Value::Atom(Atom::Var("x".to_string())),
            }),
        };
        assert!(validate_cps_program(&term).is_ok());
    }

    #[test]
    fn test_validate_unresolved_variable() {
        let term = Term::Return {
            value: Value::Atom(Atom::Var("unbound".to_string())),
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
                value: Value::Atom(Atom::Var("y".to_string())),
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
