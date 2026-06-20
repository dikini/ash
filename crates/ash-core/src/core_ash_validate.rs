//! Validation boundary for raw Core Ash programs.
//!
//! The parser builds raw Core AST values. This module checks representation
//! invariants that should fail before lowering into the CPS substrate.

use crate::core_ash::{
    CoreAtom, CoreContRef, CoreEffectOp, CoreExpr, CoreMultiplicity, CoreParam, CoreRow,
    CoreRowItem, CoreTrapReason, CoreType, CoreValue,
};
use std::collections::HashSet;

/// Raw Core program before invariant validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawCoreProgram {
    expr: CoreExpr,
}

impl RawCoreProgram {
    /// Wraps a raw Core expression for validation.
    #[must_use]
    pub fn new(expr: CoreExpr) -> Self {
        Self { expr }
    }

    /// Returns the raw expression.
    #[must_use]
    pub fn expr(&self) -> &CoreExpr {
        &self.expr
    }

    /// Consumes the wrapper and returns the raw expression.
    #[must_use]
    pub fn into_expr(self) -> CoreExpr {
        self.expr
    }
}

/// Core program that has passed the basic validator boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidCoreProgram {
    expr: CoreExpr,
}

impl ValidCoreProgram {
    /// Returns the validated expression.
    #[must_use]
    pub fn expr(&self) -> &CoreExpr {
        &self.expr
    }

    /// Consumes the wrapper and returns the validated expression.
    #[must_use]
    pub fn into_expr(self) -> CoreExpr {
        self.expr
    }
}

/// Error returned by Core Ash validation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CoreValidationError {
    /// A row contains the same item more than once.
    #[error("duplicate row item: {item}")]
    DuplicateRowItem { item: String },

    /// A raised operation is malformed or not representable by the target IR.
    #[error("unsupported effect operation: {detail}")]
    UnsupportedEffectOperation { detail: String },

    /// An affine handler resume continuation is used outside the Phase 161 discipline.
    #[error("affine resume `{resume}` violation: {detail}")]
    AffineResumeViolation { resume: String, detail: String },
}

/// Validates a raw Core program before lowering.
///
/// # Errors
///
/// Returns [`CoreValidationError`] when a basic representation invariant is
/// violated.
pub fn validate_core_program(raw: RawCoreProgram) -> Result<ValidCoreProgram, CoreValidationError> {
    validate_expr(raw.expr())?;
    Ok(ValidCoreProgram {
        expr: raw.into_expr(),
    })
}

fn validate_expr(expr: &CoreExpr) -> Result<(), CoreValidationError> {
    match expr {
        CoreExpr::Atom(atom) => validate_data_atom(atom),
        CoreExpr::LetVal {
            ty, value, body, ..
        }
        | CoreExpr::LetRec {
            ty, value, body, ..
        } => {
            validate_type(ty)?;
            validate_value(value)?;
            validate_expr(body)
        }
        CoreExpr::LetPrim { args, body, .. } => {
            validate_data_atoms(args)?;
            validate_expr(body)
        }
        CoreExpr::If {
            cond,
            then_branch,
            else_branch,
        } => {
            validate_data_atom(cond)?;
            validate_expr(then_branch)?;
            validate_expr(else_branch)
        }
        CoreExpr::Call { func, args } => {
            validate_data_atom(func)?;
            validate_data_atoms(args)
        }
        CoreExpr::Jump { arg, .. } => validate_data_atom(arg),
        CoreExpr::Raise { op, args } => {
            validate_effect_op(op)?;
            validate_data_atoms(args)
        }
        CoreExpr::Handle { clause, body } => {
            validate_effect_op(&clause.op)?;
            validate_params(&clause.params)?;
            validate_type(&clause.resume.ty)?;
            validate_row(&clause.row)?;
            validate_expr(&clause.body)?;
            validate_handler_resume(&clause.resume, &clause.body)?;
            validate_expr(body)
        }
        CoreExpr::RecordDischarge { body, .. } => validate_expr(body),
        CoreExpr::Trap { reason } => {
            if let CoreTrapReason::UnhandledEffect(op) = reason {
                validate_effect_op(op)?;
            }
            Ok(())
        }
    }
}

fn validate_value(value: &CoreValue) -> Result<(), CoreValidationError> {
    match value {
        CoreValue::Atom(atom) => validate_data_atom(atom),
        CoreValue::Lam { params, body, row } => {
            validate_params(params)?;
            validate_row(row)?;
            validate_expr(body)
        }
        CoreValue::Record { fields } => {
            for (_, atom) in fields {
                validate_data_atom(atom)?;
            }
            Ok(())
        }
        CoreValue::Tuple { elems } => validate_data_atoms(elems),
        CoreValue::DischargeMarker { .. } => Ok(()),
    }
}

fn validate_params(params: &[CoreParam]) -> Result<(), CoreValidationError> {
    for param in params {
        validate_type(&param.ty)?;
    }
    Ok(())
}

fn validate_type(ty: &CoreType) -> Result<(), CoreValidationError> {
    match ty {
        CoreType::Base(_) | CoreType::Named(_) | CoreType::Var(_) => Ok(()),
        CoreType::Function {
            params,
            result,
            row,
        } => {
            for param in params {
                validate_type(param)?;
            }
            validate_type(result)?;
            validate_row(row)
        }
        CoreType::Refinement { base, .. } => validate_type(base),
        CoreType::Cont {
            input, answer, row, ..
        } => {
            validate_type(input)?;
            validate_type(answer)?;
            validate_row(row)
        }
        CoreType::Tuple(elems) => {
            for elem in elems {
                validate_type(elem)?;
            }
            Ok(())
        }
        CoreType::Record(fields) => {
            for (_, field_ty) in fields {
                validate_type(field_ty)?;
            }
            Ok(())
        }
        CoreType::App { args, .. } => {
            for arg in args {
                validate_type(arg)?;
            }
            Ok(())
        }
    }
}

fn validate_row(row: &CoreRow) -> Result<(), CoreValidationError> {
    let mut seen = HashSet::new();
    for item in &row.items {
        if !seen.insert(item) {
            return Err(CoreValidationError::DuplicateRowItem {
                item: format!("{item:?}"),
            });
        }
        validate_row_item(item)?;
    }
    Ok(())
}

fn validate_row_item(item: &CoreRowItem) -> Result<(), CoreValidationError> {
    match item {
        CoreRowItem::Channel { payload_type, .. } => validate_type(payload_type),
        CoreRowItem::Failure { ty } => {
            if let Some(ty) = ty {
                validate_type(ty)?;
            }
            Ok(())
        }
        CoreRowItem::Capability { .. }
        | CoreRowItem::Resource { .. }
        | CoreRowItem::Role { .. }
        | CoreRowItem::Policy { .. }
        | CoreRowItem::Contract { .. }
        | CoreRowItem::Process { .. }
        | CoreRowItem::Evidence { .. }
        | CoreRowItem::EffectGroupRef { .. } => Ok(()),
    }
}

fn validate_effect_op(op: &CoreEffectOp) -> Result<(), CoreValidationError> {
    match op {
        CoreEffectOp::Capability {
            path,
            operation,
            arg_types,
            result_type,
        } => {
            validate_non_empty_path("capability", path)?;
            validate_non_empty_name("capability operation", operation)?;
            validate_signature(arg_types, result_type)
        }
        CoreEffectOp::Channel {
            path,
            mode,
            payload_type,
            result_type,
        } => {
            validate_non_empty_path("channel", path)?;
            validate_non_empty_name("channel mode", mode)?;
            validate_type(payload_type)?;
            validate_type(result_type)
        }
        CoreEffectOp::Process {
            operation,
            arg_types,
            result_type,
        } => {
            validate_non_empty_name("process operation", operation)?;
            validate_signature(arg_types, result_type)
        }
        CoreEffectOp::Failure { ty } => {
            if let Some(ty) = ty {
                validate_type(ty)?;
            }
            Ok(())
        }
    }
}

fn validate_signature(
    arg_types: &[CoreType],
    result_type: &CoreType,
) -> Result<(), CoreValidationError> {
    for ty in arg_types {
        validate_type(ty)?;
    }
    validate_type(result_type)
}

fn validate_non_empty_path(kind: &str, path: &[String]) -> Result<(), CoreValidationError> {
    if path.is_empty() || path.iter().any(|segment| segment.is_empty()) {
        return Err(CoreValidationError::UnsupportedEffectOperation {
            detail: format!("{kind} operation requires a non-empty path"),
        });
    }
    Ok(())
}

fn validate_non_empty_name(kind: &str, name: &str) -> Result<(), CoreValidationError> {
    if name.is_empty() {
        return Err(CoreValidationError::UnsupportedEffectOperation {
            detail: format!("{kind} must not be empty"),
        });
    }
    Ok(())
}

fn validate_data_atoms(atoms: &[CoreAtom]) -> Result<(), CoreValidationError> {
    for atom in atoms {
        validate_data_atom(atom)?;
    }
    Ok(())
}

fn validate_data_atom(_atom: &CoreAtom) -> Result<(), CoreValidationError> {
    // Labels have a distinct CoreContRef carrier, so the current AST cannot
    // place a label inside an ordinary atom slot. Parser tests cover the raw
    // fixture boundary for attempted `(label ...)` data atoms.
    Ok(())
}

fn validate_handler_resume(resume: &CoreParam, body: &CoreExpr) -> Result<(), CoreValidationError> {
    match &resume.ty {
        CoreType::Cont {
            multiplicity: CoreMultiplicity::Affine,
            ..
        } => {}
        CoreType::Cont { .. } => {
            return Err(CoreValidationError::AffineResumeViolation {
                resume: resume.name.clone(),
                detail: "Phase 161 supports only affine handler resumes".to_string(),
            });
        }
        _ => {
            return Err(CoreValidationError::AffineResumeViolation {
                resume: resume.name.clone(),
                detail: "handler resume parameter must have continuation type".to_string(),
            });
        }
    }

    let uses = count_affine_resume_uses(&resume.name, body)?;
    if uses > 1 {
        return Err(CoreValidationError::AffineResumeViolation {
            resume: resume.name.clone(),
            detail: "resume is jumped to more than once".to_string(),
        });
    }
    Ok(())
}

fn count_affine_resume_uses(resume: &str, expr: &CoreExpr) -> Result<usize, CoreValidationError> {
    match expr {
        CoreExpr::Atom(atom) => {
            reject_resume_atom(resume, atom, "used as ordinary data")?;
            Ok(0)
        }
        CoreExpr::LetVal { value, body, .. } | CoreExpr::LetRec { value, body, .. } => {
            reject_resume_value(resume, value)?;
            count_affine_resume_uses(resume, body)
        }
        CoreExpr::LetPrim { args, body, .. } => {
            reject_resume_atoms(resume, args, "passed to primitive operation")?;
            count_affine_resume_uses(resume, body)
        }
        CoreExpr::If {
            cond,
            then_branch,
            else_branch,
        } => {
            reject_resume_atom(resume, cond, "used as ordinary condition data")?;
            Ok(count_affine_resume_uses(resume, then_branch)?
                + count_affine_resume_uses(resume, else_branch)?)
        }
        CoreExpr::Call { func, args } => {
            reject_resume_atom(resume, func, "used as ordinary call target")?;
            reject_resume_atoms(resume, args, "passed as ordinary function argument")?;
            Ok(0)
        }
        CoreExpr::Jump { cont, arg } => {
            reject_resume_atom(resume, arg, "used as ordinary jump argument")?;
            Ok(usize::from(
                matches!(cont, CoreContRef::Var(name) if name == resume),
            ))
        }
        CoreExpr::Raise { args, .. } => {
            reject_resume_atoms(resume, args, "passed as raised operation argument")?;
            Ok(0)
        }
        CoreExpr::Handle { clause, body } => {
            let clause_uses = if clause.resume.name == resume {
                0
            } else {
                count_affine_resume_uses(resume, &clause.body)?
            };
            Ok(clause_uses + count_affine_resume_uses(resume, body)?)
        }
        CoreExpr::RecordDischarge { body, .. } => count_affine_resume_uses(resume, body),
        CoreExpr::Trap { .. } => Ok(0),
    }
}

fn reject_resume_value(resume: &str, value: &CoreValue) -> Result<(), CoreValidationError> {
    match value {
        CoreValue::Atom(atom) => reject_resume_atom(resume, atom, "stored as ordinary value"),
        CoreValue::Lam { body, .. } => {
            if count_affine_resume_uses(resume, body)? > 0 {
                return Err(CoreValidationError::AffineResumeViolation {
                    resume: resume.to_string(),
                    detail: "captured by lambda value".to_string(),
                });
            }
            Ok(())
        }
        CoreValue::Record { fields } => {
            for (_, atom) in fields {
                reject_resume_atom(resume, atom, "stored in record value")?;
            }
            Ok(())
        }
        CoreValue::Tuple { elems } => reject_resume_atoms(resume, elems, "stored in tuple value"),
        CoreValue::DischargeMarker { .. } => Ok(()),
    }
}

fn reject_resume_atoms(
    resume: &str,
    atoms: &[CoreAtom],
    detail: &str,
) -> Result<(), CoreValidationError> {
    for atom in atoms {
        reject_resume_atom(resume, atom, detail)?;
    }
    Ok(())
}

fn reject_resume_atom(
    resume: &str,
    atom: &CoreAtom,
    detail: &str,
) -> Result<(), CoreValidationError> {
    if matches!(atom, CoreAtom::Var(name) if name == resume) {
        return Err(CoreValidationError::AffineResumeViolation {
            resume: resume.to_string(),
            detail: detail.to_string(),
        });
    }
    Ok(())
}
