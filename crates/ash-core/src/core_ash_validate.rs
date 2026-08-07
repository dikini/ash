//! Validation boundary for raw Core Ash programs.
//!
//! The parser builds raw Core AST values. This module checks representation
//! invariants that should fail before lowering into the CPS substrate.

use crate::core_ash::{
    CoreAtom, CoreContRef, CoreEffectOp, CoreEvalMode, CoreExpr, CoreMultiplicity, CoreParam,
    CoreRow, CoreRowItem, CoreTrapReason, CoreType, CoreValue,
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

    /// A binder name is reused in a program scope.
    #[error("duplicate {kind} binding `{name}`")]
    DuplicateBinding { kind: String, name: String },

    /// `let-mode` uses a computation mode that does not match its annotation type.
    #[error("`let-mode` mode `{mode}` does not match mode type `{ty}`")]
    LetModeTypeMismatch { mode: String, ty: String },

    /// `force` requires a variable thunk reference.
    #[error("force expects a variable thunk atom, found `{atom}`")]
    ForceRequiresVariableThunk { atom: String },

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
    let mut bindings = HashSet::new();
    validate_expr(raw.expr(), &mut bindings)?;
    Ok(ValidCoreProgram {
        expr: raw.into_expr(),
    })
}

fn validate_expr(
    expr: &CoreExpr,
    bindings: &mut HashSet<String>,
) -> Result<(), CoreValidationError> {
    match expr {
        CoreExpr::Atom(atom) => validate_data_atom(atom),
        CoreExpr::LetMode {
            name,
            mode,
            ty,
            expr,
            body,
            ..
        } => {
            validate_binding_name("mode", name, bindings)?;
            validate_letmode_type(*mode, ty)?;
            validate_type(ty)?;
            validate_expr(expr, bindings)?;
            let mut body_bindings = bindings.clone();
            body_bindings.insert(name.clone());
            validate_expr(body, &mut body_bindings)
        }
        CoreExpr::LetVal {
            ty, value, body, ..
        }
        | CoreExpr::LetRec {
            ty, value, body, ..
        } => {
            let name = match expr {
                CoreExpr::LetVal { name, .. } | CoreExpr::LetRec { name, .. } => name.as_str(),
                _ => unreachable!(),
            };
            validate_binding_name("value", name, bindings)?;
            validate_type(ty)?;
            let mut value_bindings = bindings.clone();
            validate_value(value, &mut value_bindings)?;
            validate_expr(body, bindings)
        }
        CoreExpr::LetPrim { args, body, .. } => {
            if let CoreExpr::LetPrim { name, .. } = expr {
                validate_binding_name("primitive", name, bindings)?;
            }
            validate_data_atoms(args)?;
            validate_expr(body, bindings)
        }
        CoreExpr::LetCall {
            func, args, body, ..
        } => {
            if let CoreExpr::LetCall { name, .. } = expr {
                validate_binding_name("call result", name, bindings)?;
            }
            validate_data_atom(func)?;
            validate_data_atoms(args)?;
            validate_expr(body, bindings)
        }
        CoreExpr::Force { thunk, name, body } => {
            match thunk {
                CoreAtom::Var(_) => {}
                _ => {
                    return Err(CoreValidationError::ForceRequiresVariableThunk {
                        atom: format!("{thunk:?}"),
                    });
                }
            }
            validate_data_atom(thunk)?;
            let mut body_bindings = bindings.clone();
            validate_binding_name("force", name, &mut body_bindings)?;
            validate_expr(body, &mut body_bindings)
        }
        CoreExpr::If {
            cond,
            then_branch,
            else_branch,
        } => {
            validate_data_atom(cond)?;
            let mut then_bindings = bindings.clone();
            validate_expr(then_branch, &mut then_bindings)?;

            let mut else_bindings = bindings.clone();
            validate_expr(else_branch, &mut else_bindings)?;
            Ok(())
        }
        CoreExpr::Call { func, args } => {
            validate_data_atom(func)?;
            validate_data_atoms(args)
        }
        CoreExpr::Jump { arg, .. } => validate_data_atom(arg),
        CoreExpr::LetContCall { arg, body, .. } => {
            validate_data_atom(arg)?;
            validate_expr(body, bindings)
        }
        CoreExpr::Raise { op, args } => {
            validate_effect_op(op)?;
            validate_data_atoms(args)
        }
        CoreExpr::Handle { clause, body } => {
            validate_effect_op(&clause.op)?;
            let mut clause_bindings = bindings.clone();
            validate_params(&clause.params, &mut clause_bindings)?;
            validate_binding_name("resume", &clause.resume.name, &mut clause_bindings)?;
            validate_type(&clause.resume.ty)?;
            validate_row(&clause.row)?;
            validate_expr(&clause.body, &mut clause_bindings)?;
            validate_handler_resume(&clause.resume, &clause.body)?;
            validate_expr(body, bindings)
        }
        CoreExpr::RecordDischarge { body, .. } => validate_expr(body, bindings),
        CoreExpr::Trap { reason } => {
            if let CoreTrapReason::UnhandledEffect(op) = reason {
                validate_effect_op(op)?;
            }
            Ok(())
        }
    }
}

fn validate_binding_name(
    kind: &str,
    name: &str,
    bindings: &mut HashSet<String>,
) -> Result<(), CoreValidationError> {
    if bindings.contains(name) {
        return Err(CoreValidationError::DuplicateBinding {
            kind: kind.to_string(),
            name: name.to_string(),
        });
    }
    bindings.insert(name.to_string());
    Ok(())
}

fn validate_letmode_type(mode: CoreEvalMode, ty: &CoreType) -> Result<(), CoreValidationError> {
    match (mode, ty) {
        (
            CoreEvalMode::Strict,
            CoreType::Mode {
                mode: CoreEvalMode::Strict,
                ..
            },
        )
        | (
            CoreEvalMode::Lazy,
            CoreType::Mode {
                mode: CoreEvalMode::Lazy,
                ..
            },
        )
        | (
            CoreEvalMode::Memo,
            CoreType::Mode {
                mode: CoreEvalMode::Memo,
                ..
            },
        ) => Ok(()),
        (_, _) => Err(CoreValidationError::LetModeTypeMismatch {
            mode: format!("{mode:?}"),
            ty: format!("{ty:?}"),
        }),
    }
}

fn validate_value(
    value: &CoreValue,
    bindings: &mut HashSet<String>,
) -> Result<(), CoreValidationError> {
    match value {
        CoreValue::Atom(atom) => validate_data_atom(atom),
        CoreValue::Thunk {
            result_ty,
            body,
            row,
            ..
        } => {
            validate_type(result_ty)?;
            validate_row(row)?;
            let mut body_bindings = bindings.clone();
            validate_expr(body, &mut body_bindings)
        }
        CoreValue::Lam { params, body, row } => {
            let mut lambda_bindings = bindings.clone();
            validate_params(params, &mut lambda_bindings)?;
            validate_row(row)?;
            validate_expr(body, &mut lambda_bindings)
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

fn validate_params(
    params: &[CoreParam],
    bindings: &mut HashSet<String>,
) -> Result<(), CoreValidationError> {
    for param in params {
        validate_binding_name("param", &param.name, bindings)?;
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
        CoreType::Mode {
            inner, latent_row, ..
        } => {
            validate_type(inner)?;
            if let Some(row) = latent_row {
                validate_row(row)?;
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
        CoreRowItem::Operation { .. }
        | CoreRowItem::Resource { .. }
        | CoreRowItem::Contract { .. }
        | CoreRowItem::Process { .. }
        | CoreRowItem::Evidence { .. }
        | CoreRowItem::EffectGroupRef { .. } => Ok(()),
    }
}

fn validate_effect_op(op: &CoreEffectOp) -> Result<(), CoreValidationError> {
    match op {
        CoreEffectOp::Operation {
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
    let multiplicity = match &resume.ty {
        CoreType::Cont {
            multiplicity: CoreMultiplicity::Affine,
            ..
        } => CoreMultiplicity::Affine,
        CoreType::Cont {
            multiplicity: CoreMultiplicity::MultiShotPure,
            row,
            ..
        } => {
            // SPEC-102: multi-shot-pure resumes must declare a closed empty row.
            if !row.items.is_empty() || row.tail.is_some() {
                return Err(CoreValidationError::AffineResumeViolation {
                    resume: resume.name.clone(),
                    detail: "multi-shot-pure resume must declare a closed empty row".to_string(),
                });
            }
            CoreMultiplicity::MultiShotPure
        }
        _ => {
            return Err(CoreValidationError::AffineResumeViolation {
                resume: resume.name.clone(),
                detail: "handler resume parameter must have continuation type".to_string(),
            });
        }
    };

    let uses = count_affine_resume_uses(&resume.name, body)?;
    if multiplicity == CoreMultiplicity::Affine && uses > 1 {
        return Err(CoreValidationError::AffineResumeViolation {
            resume: resume.name.clone(),
            detail: "resume is jumped to more than once".to_string(),
        });
    }
    // Multi-shot-pure resumes may be used zero or more times — no limit.
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
        CoreExpr::LetCall {
            func, args, body, ..
        } => {
            reject_resume_atom(resume, func, "used as ordinary call target")?;
            reject_resume_atoms(resume, args, "passed as ordinary function argument")?;
            count_affine_resume_uses(resume, body)
        }
        CoreExpr::LetMode { expr, body, .. } => {
            Ok(count_affine_resume_uses(resume, expr)? + count_affine_resume_uses(resume, body)?)
        }
        CoreExpr::Force { thunk, body, .. } => {
            reject_resume_atom(resume, thunk, "used as force target")?;
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
        CoreExpr::LetContCall {
            cont, arg, body, ..
        } => {
            reject_resume_atom(resume, arg, "used as ordinary cont-call argument")?;
            let cont_use = usize::from(matches!(cont, CoreContRef::Var(name) if name == resume));
            Ok(cont_use + count_affine_resume_uses(resume, body)?)
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
        CoreValue::Thunk { body, .. } => {
            if count_affine_resume_uses(resume, body)? > 0 {
                return Err(CoreValidationError::AffineResumeViolation {
                    resume: resume.to_string(),
                    detail: "captured by thunk value".to_string(),
                });
            }
            Ok(())
        }
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
