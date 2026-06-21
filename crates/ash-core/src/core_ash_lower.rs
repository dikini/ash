//! Lowering from Core Ash direct-style IR into the existing CPS IR.
//!
//! Phase 161 covers the Core fixture/debug subset needed by the first
//! end-to-end Core-to-CPS tests. It lowers validated Core programs into
//! materialized CPS terms with explicit continuation and row fields.

use crate::core_ash::{
    CoreAtom, CoreContRef, CoreContractDischarge, CoreDischargeMode, CoreEffectOp, CoreExpr,
    CoreHandlerClause, CorePrimOp, CoreRow, CoreRowItem, CoreTrapReason, CoreType, CoreValue,
};
use crate::core_ash_validate::ValidCoreProgram;
use crate::cps::{
    Atom, ContRef, ContractDischarge, DischargeType, EffectItem, EffectItemKind, EffectOp,
    EffectRow, Env, HandlerChain, HandlerClause, PrimOp, Term, ThunkMode, TrapReason, Value,
};
use std::collections::HashMap;

/// Error returned while lowering validated Core Ash into CPS IR.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CoreLoweringError {
    /// The Core form is valid but belongs to a later lowering task.
    #[error("unsupported Core lowering form: {detail}")]
    UnsupportedForm { detail: String },

    /// A Core carrier has no representable equivalent in the current CPS IR.
    #[error("Core value cannot lower to CPS: {detail}")]
    UnrepresentableValue { detail: String },
}

/// Explicit row/context information needed to synthesize CPS continuation fields.
#[derive(Debug, Clone, PartialEq)]
pub struct CoreLoweringContext {
    current_cont: ContRef,
    current_cont_row: CoreRow,
    cont_rows: HashMap<String, CoreRow>,
    function_rows: HashMap<String, CoreRow>,
    mode_rows: HashMap<String, CoreRow>,
    mode_function_rows: HashMap<String, CoreRow>,
}

impl CoreLoweringContext {
    /// Creates a lowering context for a direct-style region.
    #[must_use]
    pub fn new(current_cont: ContRef, current_cont_row: CoreRow) -> Self {
        let mut cont_rows = HashMap::new();
        cont_rows.insert(
            cont_ref_name(&current_cont).to_string(),
            current_cont_row.clone(),
        );
        Self {
            current_cont,
            current_cont_row,
            cont_rows,
            function_rows: HashMap::new(),
            mode_rows: HashMap::new(),
            mode_function_rows: HashMap::new(),
        }
    }

    /// Registers the row for a continuation visible to `Jump`.
    #[must_use]
    pub fn with_cont_row(mut self, name: impl Into<String>, row: CoreRow) -> Self {
        self.cont_rows.insert(name.into(), row);
        self
    }

    /// Registers the local body row for a callable function.
    #[must_use]
    pub fn with_function_row(mut self, name: impl Into<String>, row: CoreRow) -> Self {
        self.function_rows.insert(name.into(), row);
        self
    }

    /// Register a local latent row captured by a lazy/memo mode binding.
    #[must_use]
    pub fn with_mode_binding_latent_row(mut self, name: impl Into<String>, row: CoreRow) -> Self {
        self.mode_rows.insert(name.into(), row);
        self
    }

    /// Registers a local latent row captured by `LetMode`.
    ///
    /// Deprecated: use [`with_mode_binding_latent_row`] directly.
    #[must_use]
    pub fn with_mode_row(self, name: impl Into<String>, row: CoreRow) -> Self {
        self.with_mode_binding_latent_row(name, row)
    }

    /// Returns the latent row for a `LetMode` binding in scope.
    pub fn mode_binding_latent_row(&self, name: &str) -> Option<&CoreRow> {
        self.mode_rows.get(name)
    }

    /// Registers the strict inner function row for a lazy/memo mode binding.
    #[must_use]
    pub fn with_mode_binding_function_row(mut self, name: impl Into<String>, row: CoreRow) -> Self {
        self.mode_function_rows.insert(name.into(), row);
        self
    }

    /// Returns the strict inner function row for a lazy/memo mode binding.
    pub fn mode_binding_function_row(&self, name: &str) -> Option<&CoreRow> {
        self.mode_function_rows.get(name)
    }
}

#[derive(Debug, Clone)]
struct LoweringState {
    context: CoreLoweringContext,
    next_cont_id: usize,
}

impl LoweringState {
    fn new(context: CoreLoweringContext) -> Self {
        Self {
            context,
            next_cont_id: 0,
        }
    }

    fn fresh_cont_name(&mut self) -> String {
        let name = format!("__k{}", self.next_cont_id);
        self.next_cont_id += 1;
        name
    }

    fn with_current_cont(&mut self, current_cont: ContRef, row: CoreRow) -> CurrentContGuard {
        let previous_cont = std::mem::replace(&mut self.context.current_cont, current_cont);
        let previous_row = std::mem::replace(&mut self.context.current_cont_row, row.clone());
        self.context
            .cont_rows
            .insert(cont_ref_name(&self.context.current_cont).to_string(), row);
        CurrentContGuard {
            previous_cont,
            previous_row,
        }
    }

    fn restore_current_cont(&mut self, guard: CurrentContGuard) {
        self.context.current_cont = guard.previous_cont;
        self.context.current_cont_row = guard.previous_row;
    }
}

#[derive(Debug, Clone)]
struct CurrentContGuard {
    previous_cont: ContRef,
    previous_row: CoreRow,
}

/// Lowers a validated Core program using an empty `halt` continuation row.
///
/// # Errors
///
/// Returns [`CoreLoweringError`] if the validated Core program uses a form
/// outside the Phase 161 lowering subset.
pub fn lower_core_program(program: ValidCoreProgram) -> Result<Term, CoreLoweringError> {
    lower_core_program_with_context(
        program,
        CoreLoweringContext::new(ContRef::Label("halt".to_string()), CoreRow::default()),
    )
}

/// Lowers a validated Core program with explicit continuation/function row context.
///
/// # Errors
///
/// Returns [`CoreLoweringError`] if the validated Core program uses a form
/// outside the Phase 161 lowering subset or a Core carrier cannot be represented
/// by the current CPS IR.
pub fn lower_core_program_with_context(
    program: ValidCoreProgram,
    context: CoreLoweringContext,
) -> Result<Term, CoreLoweringError> {
    lower_core_program_with_context_and_letcall_rows(program, context, &HashMap::new())
}

pub fn lower_core_program_with_context_and_letcall_rows(
    program: ValidCoreProgram,
    context: CoreLoweringContext,
    letcall_rows: &HashMap<Vec<usize>, CoreRow>,
) -> Result<Term, CoreLoweringError> {
    let mut state = LoweringState::new(context);
    lower_expr_with_letcall_rows(&program.into_expr(), &mut state, &Vec::new(), letcall_rows)
}

fn lower_expr_with_letcall_rows(
    expr: &CoreExpr,
    state: &mut LoweringState,
    path: &[usize],
    letcall_rows: &HashMap<Vec<usize>, CoreRow>,
) -> Result<Term, CoreLoweringError> {
    match expr {
        CoreExpr::Atom(atom) => Ok(Term::Jump {
            cont: state.context.current_cont.clone(),
            arg: lower_atom(atom)?,
            row: lower_row(&state.context.current_cont_row),
        }),
        CoreExpr::LetVal {
            name,
            ty,
            value,
            body,
        } => {
            let mut lexical_state = state.clone();
            record_local_function_row(name, ty, value, &mut lexical_state);
            record_mode_binding_rows(name, ty, &mut lexical_state);
            let value_path = with_child_path(path, 0);
            let body_path = with_child_path(path, 1);
            let lowered_value = lower_value_with_letcall_rows(
                value,
                &mut lexical_state,
                &value_path,
                letcall_rows,
            )?;
            let lowered_body =
                lower_expr_with_letcall_rows(body, &mut lexical_state, &body_path, letcall_rows)?;
            Ok(Term::LetVal {
                name: name.clone(),
                value: lowered_value,
                body: Box::new(lowered_body),
            })
        }
        CoreExpr::LetRec {
            name,
            ty,
            value,
            body,
        } => {
            let mut lexical_state = state.clone();
            record_local_function_row(name, ty, value, &mut lexical_state);
            record_mode_binding_rows(name, ty, &mut lexical_state);
            let value_path = with_child_path(path, 0);
            let body_path = with_child_path(path, 1);
            let lowered_value = lower_value_with_letcall_rows(
                value,
                &mut lexical_state,
                &value_path,
                letcall_rows,
            )?;
            let lowered_body =
                lower_expr_with_letcall_rows(body, &mut lexical_state, &body_path, letcall_rows)?;
            Ok(Term::LetRec {
                name: name.clone(),
                value: lowered_value,
                body: Box::new(lowered_body),
            })
        }
        CoreExpr::LetPrim {
            name,
            op,
            args,
            body,
        } => Ok(Term::LetPrim {
            name: name.clone(),
            op: lower_prim_op(op)?,
            args: lower_atoms(args)?,
            body: Box::new(lower_expr_with_letcall_rows(
                body,
                state,
                &with_child_path(path, 0),
                letcall_rows,
            )?),
        }),
        CoreExpr::LetCall {
            name,
            func,
            args,
            body,
        } => {
            let cont_name = state.fresh_cont_name();
            let body_path = with_child_path(path, 0);
            let mut body_state = state.clone();
            if let Some(row) = letcall_rows.get(path) {
                body_state
                    .context
                    .function_rows
                    .insert(name.clone(), row.clone());
            }

            let cont_row =
                total_row_with_letcall_rows(body, &body_state, &body_path, letcall_rows)?;
            let cont_body =
                lower_expr_with_letcall_rows(body, &mut body_state, &body_path, letcall_rows)?;
            let guard =
                state.with_current_cont(ContRef::Label(cont_name.clone()), cont_row.clone());
            let call_row = union_rows(
                &function_row_for_atom(func, state).unwrap_or_default(),
                &cont_row,
            );
            let call = Term::Call {
                func: lower_atom(func)?,
                args: lower_atoms(args)?,
                cont: ContRef::Label(cont_name.clone()),
                row: lower_row(&call_row),
            };
            state.restore_current_cont(guard);
            Ok(Term::LetCont {
                name: cont_name,
                param: name.clone(),
                cont_body: Box::new(cont_body),
                body: Box::new(call),
            })
        }
        CoreExpr::If {
            cond,
            then_branch,
            else_branch,
        } => {
            let then_path = with_child_path(path, 0);
            let else_path = with_child_path(path, 1);
            let then_local =
                local_row_with_letcall_rows(then_branch, state, &then_path, letcall_rows)?;
            let else_local =
                local_row_with_letcall_rows(else_branch, state, &else_path, letcall_rows)?;

            let mut then_state = state.clone();
            let mut else_state = state.clone();

            Ok(Term::If {
                cond: lower_atom(cond)?,
                then_branch: Box::new(lower_expr_with_letcall_rows(
                    then_branch,
                    &mut then_state,
                    &then_path,
                    letcall_rows,
                )?),
                else_branch: Box::new(lower_expr_with_letcall_rows(
                    else_branch,
                    &mut else_state,
                    &else_path,
                    letcall_rows,
                )?),
                row: lower_row(&union_rows(&then_local, &else_local)),
            })
        }
        CoreExpr::Call { func, args } => {
            let call_row = union_rows(
                &function_row_for_atom(func, state).unwrap_or_default(),
                &state.context.current_cont_row,
            );
            Ok(Term::Call {
                func: lower_atom(func)?,
                args: lower_atoms(args)?,
                cont: state.context.current_cont.clone(),
                row: lower_row(&call_row),
            })
        }
        CoreExpr::LetMode {
            name,
            mode,
            ty,
            expr,
            body,
        } => {
            let expr_path = with_child_path(path, 0);
            let body_path = with_child_path(path, 1);

            if matches!(mode, crate::core_ash::CoreEvalMode::Strict) {
                let mut body_state = state.clone();
                if let CoreType::Function { row, .. } = ty {
                    body_state
                        .context
                        .function_rows
                        .insert(name.clone(), row.clone());
                }
                let body_row =
                    total_row_with_letcall_rows(body, &body_state, &body_path, letcall_rows)?;

                let mut expr_state = state.clone();
                let cont_name = expr_state.fresh_cont_name();
                let lowered_expr = {
                    let guard = expr_state
                        .with_current_cont(ContRef::Var(cont_name.clone()), body_row.clone());
                    let lowered = lower_expr_with_letcall_rows(
                        expr,
                        &mut expr_state,
                        &expr_path,
                        letcall_rows,
                    )?;
                    expr_state.restore_current_cont(guard);
                    lowered
                };

                let lowered_body =
                    lower_expr_with_letcall_rows(body, &mut body_state, &body_path, letcall_rows)?;

                Ok(Term::LetCont {
                    name: cont_name,
                    param: name.clone(),
                    cont_body: Box::new(lowered_body),
                    body: Box::new(lowered_expr),
                })
            } else {
                let latent_row = match mode {
                    crate::core_ash::CoreEvalMode::Lazy => match ty {
                        CoreType::Mode {
                            latent_row: Some(row),
                            ..
                        } => row.clone(),
                        _ => {
                            return Err(CoreLoweringError::UnsupportedForm {
                                detail: "lazy mode annotation requires latent row".to_string(),
                            });
                        }
                    },
                    crate::core_ash::CoreEvalMode::Memo => match ty {
                        CoreType::Mode {
                            latent_row: Some(row),
                            ..
                        } => row.clone(),
                        _ => {
                            return Err(CoreLoweringError::UnsupportedForm {
                                detail: "memo mode annotation requires latent row".to_string(),
                            });
                        }
                    },
                    crate::core_ash::CoreEvalMode::Strict => {
                        unreachable!("handled above")
                    }
                };

                let mut body_state = state.clone();
                let lexical_state = body_state
                    .context
                    .with_mode_binding_latent_row(name.clone(), latent_row.clone());
                body_state.context = lexical_state;
                record_mode_binding_rows(name, ty, &mut body_state);

                let thunk_mode = match mode {
                    crate::core_ash::CoreEvalMode::Lazy => ThunkMode::Lazy,
                    crate::core_ash::CoreEvalMode::Memo => ThunkMode::Memo,
                    crate::core_ash::CoreEvalMode::Strict => {
                        unreachable!("handled above")
                    }
                };

                let lowered_body =
                    lower_expr_with_letcall_rows(body, &mut body_state, &body_path, letcall_rows)?;
                let lowered_thunk = lower_core_mode_thunk(
                    state,
                    &expr_path,
                    letcall_rows,
                    thunk_mode,
                    &latent_row,
                    expr,
                )?;

                Ok(Term::LetVal {
                    name: name.clone(),
                    value: lowered_thunk,
                    body: Box::new(lowered_body),
                })
            }
        }
        CoreExpr::Force { name, thunk, body } => {
            let _latent_row = mode_binding_row_for_force(state, thunk)?;
            let mut body_state = state.clone();
            record_force_result_function_row(name, thunk, &mut body_state);

            let letprim = Term::LetPrim {
                name: name.clone(),
                op: PrimOp::ForceThunk,
                args: vec![lower_atom(thunk)?],
                body: Box::new(lower_expr_with_letcall_rows(
                    body,
                    &mut body_state,
                    &with_child_path(path, 0),
                    letcall_rows,
                )?),
            };

            Ok(letprim)
        }
        CoreExpr::Jump { cont, arg } => {
            let row = cont_row(cont, state);
            Ok(Term::Jump {
                cont: lower_cont_ref(cont),
                arg: lower_atom(arg)?,
                row: lower_row(&row),
            })
        }
        CoreExpr::Raise { op, args } => Ok(Term::Raise {
            op: lower_effect_op(op),
            args: lower_atoms(args)?,
            resume: state.context.current_cont.clone(),
            row: lower_row(&effect_op_row(op)),
        }),
        CoreExpr::Handle { clause, body } => {
            let clause_path = with_child_path(path, 0);
            let body_path = with_child_path(path, 1);
            let mut clause_state = state.clone();
            let residual_row = handle_residual_row_with_letcall_rows(
                clause,
                body,
                &mut clause_state,
                &body_path,
                letcall_rows,
            )?;

            let mut lowered_body_state = state.clone();
            let lowered_body = lower_expr_with_letcall_rows(
                body,
                &mut lowered_body_state,
                &body_path,
                letcall_rows,
            )?;

            Ok(Term::Handle {
                clause: lower_handler_clause_with_letcall_rows(
                    clause,
                    &mut clause_state,
                    &clause_path,
                    letcall_rows,
                )?,
                body: Box::new(lowered_body),
                cont: state.context.current_cont.clone(),
                row: lower_row(&residual_row),
            })
        }
        CoreExpr::RecordDischarge { discharge, body } => Ok(Term::RecordDischarge {
            discharge: lower_contract_discharge(discharge),
            body: Box::new(lower_expr_with_letcall_rows(
                body,
                state,
                &with_child_path(path, 0),
                letcall_rows,
            )?),
        }),
        CoreExpr::Trap { reason } => Ok(Term::Trap {
            reason: lower_trap_reason(reason),
        }),
    }
}

fn lower_value_with_letcall_rows(
    value: &CoreValue,
    state: &mut LoweringState,
    path: &[usize],
    letcall_rows: &HashMap<Vec<usize>, CoreRow>,
) -> Result<Value, CoreLoweringError> {
    match value {
        CoreValue::Atom(atom) => Ok(Value::Atom(lower_atom(atom)?)),
        CoreValue::Thunk {
            mode, row, body, ..
        } => {
            let thunk_mode = match mode {
                crate::core_ash::CoreThunkMode::Lazy => ThunkMode::Lazy,
                crate::core_ash::CoreThunkMode::Memo => ThunkMode::Memo,
            };
            lower_core_mode_thunk(
                state,
                &with_child_path(path, 0),
                letcall_rows,
                thunk_mode,
                row,
                body,
            )
        }
        CoreValue::Lam { params, body, row } => {
            let cont = state.fresh_cont_name();
            let guard = state.with_current_cont(ContRef::Var(cont.clone()), CoreRow::default());
            let lowered_body =
                lower_expr_with_letcall_rows(body, state, &with_child_path(path, 0), letcall_rows);
            state.restore_current_cont(guard);
            Ok(Value::Lam {
                params: params.iter().map(|param| param.name.clone()).collect(),
                cont,
                body: Box::new(lowered_body?),
                captured_env: Env::default(),
                rec_binding: None,
                row: lower_row(row),
            })
        }
        CoreValue::Record { fields } => {
            let mut lowered = Vec::with_capacity(fields.len());
            for (name, atom) in fields {
                lowered.push((name.clone(), Value::Atom(lower_atom(atom)?)));
            }
            Ok(Value::Record { fields: lowered })
        }
        CoreValue::Tuple { elems } => {
            let mut lowered = Vec::with_capacity(elems.len());
            for elem in elems {
                lowered.push(Value::Atom(lower_atom(elem)?));
            }
            Ok(Value::Tuple { elems: lowered })
        }
        CoreValue::DischargeMarker { discharge } => Ok(Value::Record {
            fields: vec![
                (
                    "contract".to_string(),
                    Value::Atom(Atom::String(discharge.contract.clone())),
                ),
                (
                    "mode".to_string(),
                    Value::Atom(Atom::String(
                        discharge_mode_name(discharge.mode).to_string(),
                    )),
                ),
            ],
        }),
    }
}

fn lower_core_mode_thunk(
    state: &mut LoweringState,
    path: &[usize],
    letcall_rows: &HashMap<Vec<usize>, CoreRow>,
    thunk_mode: ThunkMode,
    row: &CoreRow,
    expr: &CoreExpr,
) -> Result<Value, CoreLoweringError> {
    let cont = state.fresh_cont_name();
    let lowered_expr = {
        let guard = state.with_current_cont(ContRef::Var(cont.clone()), CoreRow::default());
        let lowered = lower_expr_with_letcall_rows(expr, state, path, letcall_rows)?;
        state.restore_current_cont(guard);
        lowered
    };
    let thunk_body = Value::Lam {
        params: Vec::new(),
        cont: cont.clone(),
        body: Box::new(lowered_expr),
        captured_env: Env::default(),
        rec_binding: None,
        row: lower_row(row),
    };
    Ok(Value::ThunkClosure {
        mode: thunk_mode,
        body: Box::new(thunk_body),
        captured_env: Env::default(),
        captured_chain: HandlerChain::new(),
        row: lower_row(row),
        memo_cell: None,
    })
}

fn mode_binding_row_for_force(
    state: &LoweringState,
    thunk: &CoreAtom,
) -> Result<CoreRow, CoreLoweringError> {
    let CoreAtom::Var(thunk_name) = thunk else {
        return Err(CoreLoweringError::UnsupportedForm {
            detail: "Force requires a variable thunk atom".to_string(),
        });
    };

    state
        .context
        .mode_binding_latent_row(thunk_name)
        .cloned()
        .ok_or_else(|| CoreLoweringError::UnsupportedForm {
            detail: format!("force has no checked latent row for `{thunk_name}`"),
        })
}

fn lower_atom(atom: &CoreAtom) -> Result<Atom, CoreLoweringError> {
    match atom {
        CoreAtom::Var(name) => Ok(Atom::Var(name.clone())),
        CoreAtom::LitInt(value) => Ok(Atom::Int(*value)),
        CoreAtom::LitString(value) => Ok(Atom::String(value.clone())),
        CoreAtom::LitBool(value) => Ok(Atom::Bool(*value)),
        CoreAtom::LitUnit => Ok(Atom::Null),
        CoreAtom::ConstructorName(name) => Ok(Atom::ConstructorName(name.clone())),
        CoreAtom::PrimName(_) => Err(CoreLoweringError::UnrepresentableValue {
            detail: "primitive names are not CPS atoms".to_string(),
        }),
    }
}

fn lower_atoms(atoms: &[CoreAtom]) -> Result<Vec<Atom>, CoreLoweringError> {
    atoms.iter().map(lower_atom).collect()
}

fn lower_cont_ref(cont: &CoreContRef) -> ContRef {
    match cont {
        CoreContRef::Label(name) => ContRef::Label(name.clone()),
        CoreContRef::Var(name) => ContRef::Var(name.clone()),
    }
}

fn cont_ref_name(cont: &ContRef) -> &str {
    match cont {
        ContRef::Label(name) | ContRef::Var(name) => name,
    }
}

fn lower_prim_op(op: &CorePrimOp) -> Result<PrimOp, CoreLoweringError> {
    match op {
        CorePrimOp::Add => Ok(PrimOp::Add),
        CorePrimOp::Sub => Ok(PrimOp::Sub),
        CorePrimOp::Mul => Ok(PrimOp::Mul),
        CorePrimOp::Div => Ok(PrimOp::Div),
        CorePrimOp::Eq => Ok(PrimOp::Eq),
        CorePrimOp::Ne => Ok(PrimOp::Ne),
        CorePrimOp::Lt => Ok(PrimOp::Lt),
        CorePrimOp::Le => Ok(PrimOp::Le),
        CorePrimOp::Gt => Ok(PrimOp::Gt),
        CorePrimOp::Ge => Ok(PrimOp::Ge),
        CorePrimOp::Neg => Ok(PrimOp::Neg),
        CorePrimOp::Not => Ok(PrimOp::Not),
        CorePrimOp::RecordGet(name) => Ok(PrimOp::RecordGet(name.clone())),
        CorePrimOp::TupleGet(index) => Ok(PrimOp::TupleGet(*index)),
        CorePrimOp::ConstructorTag(_) => Err(CoreLoweringError::UnrepresentableValue {
            detail: "primitive cannot lower to current CPS PrimOp: ConstructorTag".to_string(),
        }),
    }
}

fn lower_handler_clause_with_letcall_rows(
    clause: &CoreHandlerClause,
    state: &mut LoweringState,
    path: &[usize],
    letcall_rows: &HashMap<Vec<usize>, CoreRow>,
) -> Result<HandlerClause, CoreLoweringError> {
    let resume_row = resume_row(&clause.resume.ty);
    let guard = state.with_current_cont(ContRef::Var(clause.resume.name.clone()), resume_row);
    let body = lower_expr_with_letcall_rows(&clause.body, state, path, letcall_rows);
    state.restore_current_cont(guard);

    Ok(HandlerClause {
        op: lower_effect_op(&clause.op),
        params: clause
            .params
            .iter()
            .map(|param| param.name.clone())
            .collect(),
        resume: clause.resume.name.clone(),
        body: Box::new(body?),
        row: lower_row(&clause.row),
    })
}

fn resume_row(ty: &CoreType) -> CoreRow {
    match ty {
        CoreType::Cont { row, .. } => row.clone(),
        _ => CoreRow::default(),
    }
}

fn lower_effect_op(op: &CoreEffectOp) -> EffectOp {
    match op {
        CoreEffectOp::Capability {
            path,
            operation,
            arg_types,
            result_type,
        } => EffectOp {
            item: EffectItem {
                namespace: "cap".to_string(),
                name: dotted_name(path, operation),
                kind: EffectItemKind::Capability,
            },
            arg_types: lower_type_names(arg_types),
            result_type: lower_type_name(result_type),
        },
        CoreEffectOp::Channel {
            path,
            mode,
            payload_type,
            result_type,
        } => EffectOp {
            item: EffectItem {
                namespace: "channel".to_string(),
                name: dotted_name(path, mode),
                kind: EffectItemKind::Channel,
            },
            arg_types: vec![lower_type_name(payload_type)],
            result_type: lower_type_name(result_type),
        },
        CoreEffectOp::Process {
            operation,
            arg_types,
            result_type,
        } => EffectOp {
            item: EffectItem {
                namespace: "proc".to_string(),
                name: operation.clone(),
                kind: EffectItemKind::Alias,
            },
            arg_types: lower_type_names(arg_types),
            result_type: lower_type_name(result_type),
        },
        CoreEffectOp::Failure { ty } => EffectOp {
            item: EffectItem {
                namespace: "fail".to_string(),
                name: ty
                    .as_ref()
                    .map(lower_type_name)
                    .unwrap_or_else(|| "failure".to_string()),
                kind: EffectItemKind::Alias,
            },
            arg_types: ty
                .as_ref()
                .map(|ty| vec![lower_type_name(ty)])
                .unwrap_or_default(),
            result_type: "Never".to_string(),
        },
    }
}

fn effect_op_row(op: &CoreEffectOp) -> CoreRow {
    match op {
        CoreEffectOp::Capability {
            path, operation, ..
        } => CoreRow::closed(vec![CoreRowItem::Capability {
            path: path.clone(),
            operation: operation.clone(),
        }]),
        CoreEffectOp::Channel {
            path,
            mode,
            payload_type,
            ..
        } => CoreRow::closed(vec![CoreRowItem::Channel {
            path: path.clone(),
            mode: mode.clone(),
            payload_type: Box::new(payload_type.clone()),
        }]),
        CoreEffectOp::Process { operation, .. } => CoreRow::closed(vec![CoreRowItem::Process {
            operation: operation.clone(),
        }]),
        CoreEffectOp::Failure { ty } => CoreRow::closed(vec![CoreRowItem::Failure {
            ty: ty.clone().map(Box::new),
        }]),
    }
}

fn lower_type_names(types: &[CoreType]) -> Vec<String> {
    types.iter().map(lower_type_name).collect()
}

fn lower_type_name(ty: &CoreType) -> String {
    match ty {
        CoreType::Base(name) | CoreType::Named(name) | CoreType::Var(name) => name.clone(),
        CoreType::Function { .. } => "Function".to_string(),
        CoreType::Refinement { base, .. } => lower_type_name(base),
        CoreType::Cont { .. } => "Cont".to_string(),
        CoreType::Mode { inner, .. } => lower_type_name(inner),
        CoreType::Tuple(elems) => {
            let elems = elems
                .iter()
                .map(lower_type_name)
                .collect::<Vec<_>>()
                .join(",");
            format!("({elems})")
        }
        CoreType::Record(fields) => {
            let fields = fields
                .iter()
                .map(|(name, ty)| format!("{name}:{}", lower_type_name(ty)))
                .collect::<Vec<_>>()
                .join(",");
            format!("{{{fields}}}")
        }
        CoreType::App { name, args } => {
            if args.is_empty() {
                name.clone()
            } else {
                let args = args
                    .iter()
                    .map(lower_type_name)
                    .collect::<Vec<_>>()
                    .join(",");
                format!("{name}<{args}>")
            }
        }
    }
}

fn lower_contract_discharge(discharge: &CoreContractDischarge) -> ContractDischarge {
    ContractDischarge {
        contract: discharge.contract.clone(),
        discharge_type: match discharge.mode {
            CoreDischargeMode::Dynamic => DischargeType::Dynamic,
            CoreDischargeMode::Static | CoreDischargeMode::Evidence => DischargeType::Static,
        },
    }
}

fn lower_trap_reason(reason: &CoreTrapReason) -> TrapReason {
    match reason {
        CoreTrapReason::ContractViolation(_) => TrapReason::ContractViolation,
        CoreTrapReason::UnhandledEffect(op) => {
            TrapReason::Custom(format!("unhandled effect: {:?}", lower_effect_op(op).item))
        }
        CoreTrapReason::Panic(message) => TrapReason::Custom(format!("panic: {message}")),
        CoreTrapReason::NonExhaustiveMatch => {
            TrapReason::Custom("non-exhaustive match".to_string())
        }
    }
}

fn discharge_mode_name(mode: CoreDischargeMode) -> &'static str {
    match mode {
        CoreDischargeMode::Static => "static",
        CoreDischargeMode::Evidence => "evidence",
        CoreDischargeMode::Dynamic => "dynamic",
    }
}

fn local_function_row_from_binding<'a>(ty: &'a CoreType, value: &'a CoreValue) -> Option<CoreRow> {
    if let CoreType::Function { row, .. } = ty {
        return Some(row.clone());
    }

    if let CoreValue::Lam { row, .. } = value {
        return Some(row.clone());
    }

    None
}

fn record_local_function_row(
    name: &str,
    ty: &CoreType,
    value: &CoreValue,
    state: &mut LoweringState,
) {
    if let Some(row) = local_function_row_from_binding(ty, value) {
        state.context.function_rows.insert(name.to_string(), row);
    }
}

fn record_mode_binding_rows(name: &str, ty: &CoreType, state: &mut LoweringState) {
    if let CoreType::Mode {
        mode: crate::core_ash::CoreEvalMode::Lazy | crate::core_ash::CoreEvalMode::Memo,
        inner,
        latent_row,
        ..
    } = ty
    {
        if let Some(row) = latent_row {
            state
                .context
                .mode_rows
                .insert(name.to_string(), row.clone());
        }
        if let CoreType::Function { row, .. } = inner.as_ref() {
            state
                .context
                .mode_function_rows
                .insert(name.to_string(), row.clone());
        }
    }
}

fn record_force_result_function_row(name: &str, thunk: &CoreAtom, state: &mut LoweringState) {
    let CoreAtom::Var(thunk_name) = thunk else {
        return;
    };
    if let Some(row) = state.context.mode_binding_function_row(thunk_name).cloned() {
        state.context.function_rows.insert(name.to_string(), row);
    }
}

fn function_row_for_atom(atom: &CoreAtom, state: &LoweringState) -> Option<CoreRow> {
    let CoreAtom::Var(name) = atom else {
        return None;
    };
    state.context.function_rows.get(name).cloned()
}

fn cont_row(cont: &CoreContRef, state: &LoweringState) -> CoreRow {
    let name = match cont {
        CoreContRef::Label(name) | CoreContRef::Var(name) => name,
    };
    state
        .context
        .cont_rows
        .get(name)
        .cloned()
        .unwrap_or_default()
}

fn with_child_path(path: &[usize], child: usize) -> Vec<usize> {
    let mut child_path = path.to_vec();
    child_path.push(child);
    child_path
}

fn local_row_with_letcall_rows(
    expr: &CoreExpr,
    state: &LoweringState,
    path: &[usize],
    letcall_rows: &HashMap<Vec<usize>, CoreRow>,
) -> Result<CoreRow, CoreLoweringError> {
    match expr {
        CoreExpr::Atom(_) | CoreExpr::Jump { .. } | CoreExpr::Trap { .. } => Ok(CoreRow::default()),
        CoreExpr::LetMode {
            name,
            mode,
            ty,
            expr,
            body,
        } => match mode {
            crate::core_ash::CoreEvalMode::Strict => Ok(union_rows(
                &local_row_with_letcall_rows(expr, state, &with_child_path(path, 0), letcall_rows)?,
                &local_row_with_letcall_rows(body, state, &with_child_path(path, 1), letcall_rows)?,
            )),
            crate::core_ash::CoreEvalMode::Lazy | crate::core_ash::CoreEvalMode::Memo => {
                let mut lexical_state = state.clone();
                record_mode_binding_rows(name, ty, &mut lexical_state);
                local_row_with_letcall_rows(
                    body,
                    &lexical_state,
                    &with_child_path(path, 1),
                    letcall_rows,
                )
            }
        },
        CoreExpr::LetVal {
            name,
            ty,
            value,
            body,
        }
        | CoreExpr::LetRec {
            name,
            ty,
            value,
            body,
        } => {
            let mut lexical_state = state.clone();
            record_local_function_row(name, ty, value, &mut lexical_state);
            record_mode_binding_rows(name, ty, &mut lexical_state);
            local_row_with_letcall_rows(
                body,
                &lexical_state,
                &with_child_path(path, 1),
                letcall_rows,
            )
        }
        CoreExpr::LetPrim { op, body, .. } => {
            lower_prim_op(op)?;
            local_row_with_letcall_rows(body, state, &with_child_path(path, 0), letcall_rows)
        }
        CoreExpr::LetCall {
            func, body, name, ..
        } => {
            let mut lexical_state = state.clone();
            if let Some(row) = letcall_rows.get(path) {
                lexical_state
                    .context
                    .function_rows
                    .insert(name.clone(), row.clone());
            }
            Ok(union_rows(
                &function_row_for_atom(func, state).unwrap_or_default(),
                &local_row_with_letcall_rows(
                    body,
                    &lexical_state,
                    &with_child_path(path, 0),
                    letcall_rows,
                )?,
            ))
        }
        CoreExpr::If {
            then_branch,
            else_branch,
            ..
        } => Ok(union_rows(
            &local_row_with_letcall_rows(
                then_branch,
                state,
                &with_child_path(path, 0),
                letcall_rows,
            )?,
            &local_row_with_letcall_rows(
                else_branch,
                state,
                &with_child_path(path, 1),
                letcall_rows,
            )?,
        )),
        CoreExpr::Call { func, .. } => Ok(function_row_for_atom(func, state).unwrap_or_default()),
        CoreExpr::Raise { op, .. } => Ok(effect_op_row(op)),
        CoreExpr::Handle { clause, body } => {
            let mut clause_state = state.clone();
            let body_path = with_child_path(path, 1);
            let clause_row = handle_residual_row_with_letcall_rows(
                clause,
                body,
                &mut clause_state,
                &body_path,
                letcall_rows,
            )?;
            Ok(clause_row)
        }
        CoreExpr::RecordDischarge { discharge, body } => Ok(subtract_rows(
            &local_row_with_letcall_rows(body, state, &with_child_path(path, 0), letcall_rows)?,
            &contract_row(&discharge.contract),
        )),
        CoreExpr::Force { name, thunk, body } => {
            let thunk_row = mode_binding_row_for_force(state, thunk)?;
            let mut body_state = state.clone();
            record_force_result_function_row(name, thunk, &mut body_state);
            Ok(union_rows(
                &local_row_with_letcall_rows(
                    body,
                    &body_state,
                    &with_child_path(path, 0),
                    letcall_rows,
                )?,
                &thunk_row,
            ))
        }
    }
}

fn total_row_with_letcall_rows(
    expr: &CoreExpr,
    state: &LoweringState,
    path: &[usize],
    letcall_rows: &HashMap<Vec<usize>, CoreRow>,
) -> Result<CoreRow, CoreLoweringError> {
    match expr {
        CoreExpr::Atom(_) => Ok(state.context.current_cont_row.clone()),
        CoreExpr::LetMode {
            name,
            mode,
            ty,
            expr,
            body,
        } => match mode {
            crate::core_ash::CoreEvalMode::Strict => Ok(union_rows(
                &total_row_with_letcall_rows(expr, state, &with_child_path(path, 0), letcall_rows)?,
                &total_row_with_letcall_rows(body, state, &with_child_path(path, 1), letcall_rows)?,
            )),
            crate::core_ash::CoreEvalMode::Lazy | crate::core_ash::CoreEvalMode::Memo => {
                let mut lexical_state = state.clone();
                record_mode_binding_rows(name, ty, &mut lexical_state);
                total_row_with_letcall_rows(
                    body,
                    &lexical_state,
                    &with_child_path(path, 1),
                    letcall_rows,
                )
            }
        },
        CoreExpr::LetVal {
            name,
            ty,
            value,
            body,
        }
        | CoreExpr::LetRec {
            name,
            ty,
            value,
            body,
        } => {
            let mut lexical_state = state.clone();
            record_local_function_row(name, ty, value, &mut lexical_state);
            record_mode_binding_rows(name, ty, &mut lexical_state);
            total_row_with_letcall_rows(
                body,
                &lexical_state,
                &with_child_path(path, 1),
                letcall_rows,
            )
        }
        CoreExpr::LetPrim { op, body, .. } => {
            lower_prim_op(op)?;
            total_row_with_letcall_rows(body, state, &with_child_path(path, 0), letcall_rows)
        }
        CoreExpr::LetCall {
            func, body, name, ..
        } => {
            let mut lexical_state = state.clone();
            if let Some(row) = letcall_rows.get(path) {
                lexical_state
                    .context
                    .function_rows
                    .insert(name.clone(), row.clone());
            }

            Ok(union_rows(
                &function_row_for_atom(func, state).unwrap_or_default(),
                &total_row_with_letcall_rows(
                    body,
                    &lexical_state,
                    &with_child_path(path, 0),
                    letcall_rows,
                )?,
            ))
        }
        CoreExpr::If {
            then_branch,
            else_branch,
            ..
        } => Ok(union_rows(
            &total_row_with_letcall_rows(
                then_branch,
                state,
                &with_child_path(path, 0),
                letcall_rows,
            )?,
            &total_row_with_letcall_rows(
                else_branch,
                state,
                &with_child_path(path, 1),
                letcall_rows,
            )?,
        )),
        CoreExpr::Call { func, .. } => Ok(union_rows(
            &function_row_for_atom(func, state).unwrap_or_default(),
            &state.context.current_cont_row,
        )),
        CoreExpr::Jump { cont, .. } => Ok(cont_row(cont, state)),
        CoreExpr::Trap { .. } => Ok(CoreRow::default()),
        CoreExpr::Raise { op, .. } => Ok(union_rows(
            &effect_op_row(op),
            &state.context.current_cont_row,
        )),
        CoreExpr::Handle { clause, body } => {
            let mut clause_state = state.clone();
            let body_path = with_child_path(path, 1);
            let clause_row = handle_residual_row_with_letcall_rows(
                clause,
                body,
                &mut clause_state,
                &body_path,
                letcall_rows,
            )?;
            Ok(union_rows(&clause_row, &state.context.current_cont_row))
        }
        CoreExpr::RecordDischarge { discharge, body } => Ok(subtract_rows(
            &total_row_with_letcall_rows(body, state, &with_child_path(path, 0), letcall_rows)?,
            &contract_row(&discharge.contract),
        )),
        CoreExpr::Force { name, thunk, body } => {
            let thunk_row = mode_binding_row_for_force(state, thunk)?;
            let mut body_state = state.clone();
            record_force_result_function_row(name, thunk, &mut body_state);
            Ok(union_rows(
                &total_row_with_letcall_rows(
                    body,
                    &body_state,
                    &with_child_path(path, 0),
                    letcall_rows,
                )?,
                &thunk_row,
            ))
        }
    }
}

fn union_rows(left: &CoreRow, right: &CoreRow) -> CoreRow {
    let mut items = left.items.clone();
    for item in &right.items {
        if !items.contains(item) {
            items.push(item.clone());
        }
    }
    CoreRow {
        items,
        tail: left.tail.clone().or_else(|| right.tail.clone()),
    }
}

fn handle_residual_row_with_letcall_rows(
    clause: &CoreHandlerClause,
    body: &CoreExpr,
    state: &mut LoweringState,
    path: &[usize],
    letcall_rows: &HashMap<Vec<usize>, CoreRow>,
) -> Result<CoreRow, CoreLoweringError> {
    let body_row = local_row_with_letcall_rows(body, state, path, letcall_rows)?;
    let body_without_op = subtract_rows_structural(&body_row, &effect_op_row(&clause.op));
    Ok(union_rows(
        &union_rows(&body_without_op, &resume_row(&clause.resume.ty)),
        &clause.row,
    ))
}

fn subtract_rows(left: &CoreRow, right: &CoreRow) -> CoreRow {
    CoreRow {
        items: left
            .items
            .iter()
            .filter(|item| !right.items.contains(item))
            .cloned()
            .collect(),
        tail: left.tail.clone(),
    }
}

fn subtract_rows_structural(left: &CoreRow, right: &CoreRow) -> CoreRow {
    let mut used = vec![false; right.items.len()];
    let mut items = Vec::with_capacity(left.items.len());

    'left_items: for left_item in &left.items {
        for (index, right_item) in right.items.iter().enumerate() {
            if used[index] {
                continue;
            }
            if row_items_equivalent_for_lowering(left_item, right_item) {
                used[index] = true;
                continue 'left_items;
            }
        }
        items.push(left_item.clone());
    }

    CoreRow {
        items,
        tail: left.tail.clone(),
    }
}

fn row_items_equivalent_for_lowering(lhs: &CoreRowItem, rhs: &CoreRowItem) -> bool {
    match (lhs, rhs) {
        (
            CoreRowItem::Capability {
                path: lhs_path,
                operation: lhs_operation,
            },
            CoreRowItem::Capability {
                path: rhs_path,
                operation: rhs_operation,
            },
        ) => lhs_path == rhs_path && lhs_operation == rhs_operation,
        (
            CoreRowItem::Resource {
                path: lhs_path,
                mode: lhs_mode,
            },
            CoreRowItem::Resource {
                path: rhs_path,
                mode: rhs_mode,
            },
        ) => lhs_path == rhs_path && lhs_mode == rhs_mode,
        (CoreRowItem::Role { path: lhs_path }, CoreRowItem::Role { path: rhs_path }) => {
            lhs_path == rhs_path
        }
        (CoreRowItem::Policy { path: lhs_path }, CoreRowItem::Policy { path: rhs_path }) => {
            lhs_path == rhs_path
        }
        (
            CoreRowItem::Contract {
                contract: lhs_contract,
            },
            CoreRowItem::Contract {
                contract: rhs_contract,
            },
        ) => lhs_contract == rhs_contract,
        (
            CoreRowItem::Channel {
                path: lhs_path,
                mode: lhs_mode,
                payload_type: lhs_payload_type,
            },
            CoreRowItem::Channel {
                path: rhs_path,
                mode: rhs_mode,
                payload_type: rhs_payload_type,
            },
        ) => {
            lhs_path == rhs_path
                && lhs_mode == rhs_mode
                && core_types_equivalent_for_lowering(lhs_payload_type, rhs_payload_type)
        }
        (
            CoreRowItem::Process {
                operation: lhs_operation,
            },
            CoreRowItem::Process {
                operation: rhs_operation,
            },
        ) => lhs_operation == rhs_operation,
        (CoreRowItem::Failure { ty: Some(lhs_ty) }, CoreRowItem::Failure { ty: Some(rhs_ty) }) => {
            core_types_equivalent_for_lowering(lhs_ty, rhs_ty)
        }
        (CoreRowItem::Failure { ty: None }, CoreRowItem::Failure { ty: None }) => true,
        (CoreRowItem::Evidence { path: lhs_path }, CoreRowItem::Evidence { path: rhs_path }) => {
            lhs_path == rhs_path
        }
        (
            CoreRowItem::EffectGroupRef { path: lhs_path },
            CoreRowItem::EffectGroupRef { path: rhs_path },
        ) => lhs_path == rhs_path,
        _ => false,
    }
}

fn core_types_equivalent_for_lowering(lhs: &CoreType, rhs: &CoreType) -> bool {
    match (lhs, rhs) {
        (CoreType::Base(lhs), CoreType::Base(rhs))
        | (CoreType::Named(lhs), CoreType::Named(rhs))
        | (CoreType::Var(lhs), CoreType::Var(rhs)) => lhs == rhs,
        (
            CoreType::Function {
                params: lhs_params,
                result: lhs_result,
                row: lhs_row,
            },
            CoreType::Function {
                params: rhs_params,
                result: rhs_result,
                row: rhs_row,
            },
        ) => {
            lhs_params.len() == rhs_params.len()
                && lhs_params
                    .iter()
                    .zip(rhs_params)
                    .all(|(lhs_ty, rhs_ty)| core_types_equivalent_for_lowering(lhs_ty, rhs_ty))
                && core_types_equivalent_for_lowering(lhs_result, rhs_result)
                && rows_are_equivalent_for_lowering(lhs_row, rhs_row)
        }
        (
            CoreType::Refinement {
                base: lhs_base,
                predicate: lhs_predicate,
            },
            CoreType::Refinement {
                base: rhs_base,
                predicate: rhs_predicate,
            },
        ) => {
            lhs_predicate == rhs_predicate && core_types_equivalent_for_lowering(lhs_base, rhs_base)
        }
        (
            CoreType::Cont {
                input: lhs_input,
                answer: lhs_answer,
                row: lhs_row,
                multiplicity: lhs_multiplicity,
            },
            CoreType::Cont {
                input: rhs_input,
                answer: rhs_answer,
                row: rhs_row,
                multiplicity: rhs_multiplicity,
            },
        ) => {
            lhs_multiplicity == rhs_multiplicity
                && core_types_equivalent_for_lowering(lhs_input, rhs_input)
                && core_types_equivalent_for_lowering(lhs_answer, rhs_answer)
                && rows_are_equivalent_for_lowering(lhs_row, rhs_row)
        }
        (CoreType::Tuple(lhs), CoreType::Tuple(rhs)) => {
            lhs.len() == rhs.len()
                && lhs
                    .iter()
                    .zip(rhs)
                    .all(|(lhs_ty, rhs_ty)| core_types_equivalent_for_lowering(lhs_ty, rhs_ty))
        }
        (CoreType::Record(lhs), CoreType::Record(rhs)) => {
            record_fields_equivalent_for_lowering(lhs, rhs)
        }
        (
            CoreType::App {
                name: lhs_name,
                args: lhs_args,
            },
            CoreType::App {
                name: rhs_name,
                args: rhs_args,
            },
        ) => {
            lhs_name == rhs_name
                && lhs_args.len() == rhs_args.len()
                && lhs_args
                    .iter()
                    .zip(rhs_args)
                    .all(|(lhs_ty, rhs_ty)| core_types_equivalent_for_lowering(lhs_ty, rhs_ty))
        }
        _ => false,
    }
}

fn record_fields_equivalent_for_lowering(
    lhs: &[(String, CoreType)],
    rhs: &[(String, CoreType)],
) -> bool {
    if lhs.len() != rhs.len()
        || has_duplicate_record_field_name(lhs)
        || has_duplicate_record_field_name(rhs)
    {
        return false;
    }

    lhs.iter().all(|(lhs_name, lhs_ty)| {
        rhs.iter()
            .find(|(rhs_name, _)| rhs_name == lhs_name)
            .is_some_and(|(_, rhs_ty)| core_types_equivalent_for_lowering(lhs_ty, rhs_ty))
    })
}

fn has_duplicate_record_field_name(fields: &[(String, CoreType)]) -> bool {
    for i in 0..fields.len() {
        for j in i + 1..fields.len() {
            if fields[i].0 == fields[j].0 {
                return true;
            }
        }
    }
    false
}

fn rows_are_equivalent_for_lowering(lhs: &CoreRow, rhs: &CoreRow) -> bool {
    if lhs.tail != rhs.tail || lhs.items.len() != rhs.items.len() {
        return false;
    }

    let mut used = vec![false; rhs.items.len()];
    for lhs_item in &lhs.items {
        let mut found = None;
        for (index, rhs_item) in rhs.items.iter().enumerate() {
            if used[index] {
                continue;
            }
            if row_items_equivalent_for_lowering(lhs_item, rhs_item) {
                found = Some(index);
                break;
            }
        }

        let Some(index) = found else {
            return false;
        };
        used[index] = true;
    }
    true
}

fn contract_row(contract: &str) -> CoreRow {
    CoreRow {
        items: vec![CoreRowItem::Contract {
            contract: contract.to_owned(),
        }],
        tail: None,
    }
}

fn lower_row(row: &CoreRow) -> EffectRow {
    let mut items = Vec::with_capacity(row.items.len());
    for item in &row.items {
        if let Some(lowered) = lower_row_item(item)
            && !items.contains(&lowered)
        {
            items.push(lowered);
        }
    }
    EffectRow { items }
}

fn lower_row_item(item: &CoreRowItem) -> Option<EffectItem> {
    match item {
        CoreRowItem::Capability { path, operation } => Some(EffectItem {
            namespace: "cap".to_string(),
            name: dotted_name(path, operation),
            kind: EffectItemKind::Capability,
        }),
        CoreRowItem::Role { path } => Some(EffectItem {
            namespace: "role".to_string(),
            name: path.join("."),
            kind: EffectItemKind::Role,
        }),
        CoreRowItem::Policy { path } => Some(EffectItem {
            namespace: "policy".to_string(),
            name: path.join("."),
            kind: EffectItemKind::Policy,
        }),
        CoreRowItem::Contract { contract } => Some(EffectItem {
            namespace: "contract".to_string(),
            name: contract.clone(),
            kind: EffectItemKind::Contract,
        }),
        CoreRowItem::Channel { path, mode, .. } => Some(EffectItem {
            namespace: "channel".to_string(),
            name: dotted_name(path, mode),
            kind: EffectItemKind::Channel,
        }),
        CoreRowItem::Evidence { path } => Some(EffectItem {
            namespace: "evidence".to_string(),
            name: path.join("."),
            kind: EffectItemKind::Alias,
        }),
        CoreRowItem::EffectGroupRef { path } => Some(EffectItem {
            namespace: "group".to_string(),
            name: path.join("."),
            kind: EffectItemKind::Group,
        }),
        CoreRowItem::Resource { path, mode } => Some(EffectItem {
            namespace: "resource".to_string(),
            name: dotted_name(path, mode),
            kind: EffectItemKind::Alias,
        }),
        CoreRowItem::Process { operation } => Some(EffectItem {
            namespace: "proc".to_string(),
            name: operation.clone(),
            kind: EffectItemKind::Alias,
        }),
        CoreRowItem::Failure { ty } => Some(EffectItem {
            namespace: "fail".to_string(),
            name: ty
                .as_deref()
                .map(lower_type_name)
                .unwrap_or_else(|| "failure".to_string()),
            kind: EffectItemKind::Alias,
        }),
    }
}

fn dotted_name(path: &[String], leaf: &str) -> String {
    if path.is_empty() {
        return leaf.to_string();
    }
    format!("{}.{}", path.join("."), leaf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core_ash::{
        CoreCaptureSet, CoreEffectOp, CoreEvalMode, CoreExpr, CoreThunkMode, CoreType, CoreValue,
    };
    use crate::cps::{ThunkMode, Value as LoweredValue};

    fn payload_a_first() -> CoreType {
        CoreType::Record(vec![
            ("a".into(), CoreType::Base("Int".into())),
            ("b".into(), CoreType::Base("String".into())),
        ])
    }

    fn payload_b_first() -> CoreType {
        CoreType::Record(vec![
            ("b".into(), CoreType::Base("String".into())),
            ("a".into(), CoreType::Base("Int".into())),
        ])
    }

    fn sample_row_item_row() -> CoreRow {
        CoreRow {
            items: vec![CoreRowItem::Capability {
                path: vec!["db".into()],
                operation: "read".into(),
            }],
            tail: None,
        }
    }

    fn test_lowering_state() -> LoweringState {
        LoweringState::new(CoreLoweringContext::new(
            ContRef::Label("k0".to_string()),
            CoreRow::default(),
        ))
    }

    #[test]
    fn subtract_rows_structural_matches_channel_payload_rows_in_record_field_permuted_form() {
        let left = CoreRow {
            items: vec![CoreRowItem::Channel {
                path: vec!["jobs".into()],
                mode: "send".into(),
                payload_type: Box::new(payload_a_first()),
            }],
            tail: None,
        };
        let right = CoreRow {
            items: vec![CoreRowItem::Channel {
                path: vec!["jobs".into()],
                mode: "send".into(),
                payload_type: Box::new(payload_b_first()),
            }],
            tail: None,
        };
        let remaining = subtract_rows_structural(&left, &right);
        assert!(
            remaining.items.is_empty(),
            "structural subtraction should remove equivalent typed payload row item"
        );
        assert_eq!(remaining.tail, None);
    }

    #[test]
    fn lower_thunk_value_produces_thunk_closure() {
        let thunk = CoreValue::Thunk {
            mode: CoreThunkMode::Lazy,
            result_ty: CoreType::Base("Int".into()),
            body: Box::new(CoreExpr::Atom(CoreAtom::LitInt(42))),
            row: sample_row_item_row(),
            captures: CoreCaptureSet::default(),
        };

        let mut state = test_lowering_state();
        let lowered = lower_value_with_letcall_rows(&thunk, &mut state, &[], &HashMap::new())
            .expect("thunk value should lower");

        match lowered {
            LoweredValue::ThunkClosure {
                mode, body, row, ..
            } => {
                assert_eq!(mode, ThunkMode::Lazy);
                assert_eq!(row, lower_row(&sample_row_item_row()));
                match *body {
                    LoweredValue::Lam {
                        params, body, row, ..
                    } => {
                        assert!(params.is_empty());
                        assert_eq!(row, lower_row(&sample_row_item_row()));
                        assert!(matches!(
                            *body,
                            Term::Jump {
                                arg: Atom::Int(42),
                                ..
                            }
                        ));
                    }
                    other => panic!("expected thunk body lambda, got {other:?}"),
                }
            }
            other => panic!("expected thunk closure, got {other:?}"),
        }
    }

    #[test]
    fn lower_letmode_lazy_binds_thunk_value() {
        let expr = CoreExpr::LetMode {
            name: "t".to_string(),
            mode: CoreEvalMode::Lazy,
            ty: CoreType::Mode {
                mode: CoreEvalMode::Lazy,
                inner: Box::new(CoreType::Base("Int".into())),
                latent_row: Some(sample_row_item_row()),
            },
            expr: Box::new(CoreExpr::Atom(CoreAtom::LitInt(1))),
            body: Box::new(CoreExpr::Atom(CoreAtom::Var("t".to_string()))),
        };

        let mut state = test_lowering_state();
        let lowered = lower_expr_with_letcall_rows(&expr, &mut state, &[], &HashMap::new())
            .expect("letmode lazy should lower");

        match lowered {
            Term::LetVal {
                name,
                value,
                body: _body,
            } => {
                assert_eq!(name, "t");
                match value {
                    LoweredValue::ThunkClosure { mode, .. } => {
                        assert_eq!(mode, ThunkMode::Lazy);
                    }
                    other => panic!("expected thunk closure value, got {other:?}"),
                }
            }
            other => panic!("expected let-val term, got {other:?}"),
        }
    }

    #[test]
    fn lower_force_uses_force_primitive_and_preserves_row() {
        let latent_row = sample_row_item_row();
        let mut state = LoweringState::new(
            CoreLoweringContext::new(ContRef::Label("k0".to_string()), CoreRow::default())
                .with_mode_binding_latent_row("t", latent_row.clone()),
        );

        let expr = CoreExpr::Force {
            name: "v".to_string(),
            thunk: CoreAtom::Var("t".to_string()),
            body: Box::new(CoreExpr::Atom(CoreAtom::LitInt(3))),
        };

        let lowered = lower_expr_with_letcall_rows(&expr, &mut state, &[], &HashMap::new())
            .expect("force should lower");

        match lowered {
            Term::LetPrim {
                name,
                op,
                args,
                body,
            } => {
                assert_eq!(name, "v");
                assert_eq!(op, PrimOp::ForceThunk);
                assert_eq!(args, vec![Atom::Var("t".to_string())]);
                assert!(matches!(
                    *body,
                    Term::Jump {
                        arg: Atom::Int(3),
                        ..
                    }
                ));
            }
            other => panic!("expected letprim term, got {other:?}"),
        }

        let lowered_row = local_row_with_letcall_rows(&expr, &state, &[], &HashMap::new())
            .expect("force row should include latent row");
        assert_eq!(lowered_row, latent_row);
    }

    #[test]
    fn lazy_letmode_row_uses_body_row_when_initializer_is_not_forced() {
        let expr = CoreExpr::LetMode {
            name: "thunk".to_string(),
            mode: CoreEvalMode::Lazy,
            ty: CoreType::Mode {
                mode: CoreEvalMode::Lazy,
                inner: Box::new(CoreType::Base("Int".into())),
                latent_row: Some(sample_row_item_row()),
            },
            expr: Box::new(CoreExpr::Raise {
                op: CoreEffectOp::Capability {
                    path: vec!["db".into()],
                    operation: "write".into(),
                    arg_types: vec![CoreType::Base("Int".into())],
                    result_type: CoreType::Base("Unit".into()),
                },
                args: vec![],
            }),
            body: Box::new(CoreExpr::Atom(CoreAtom::LitInt(7))),
        };

        let state = test_lowering_state();
        let row = local_row_with_letcall_rows(&expr, &state, &[], &HashMap::new())
            .expect("lazy letmode local row should compute");
        assert_eq!(row, CoreRow::default());

        let total_row = total_row_with_letcall_rows(&expr, &state, &[], &HashMap::new())
            .expect("lazy letmode total row should compute");
        assert_eq!(total_row, CoreRow::default());
    }
}
