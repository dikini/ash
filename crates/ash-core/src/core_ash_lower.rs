//! Lowering from Core Ash direct-style IR into the existing CPS IR.
//!
//! Phase 161 starts with the pure/basic Core subset. Effect operations,
//! handlers, contract discharge, and traps are deliberately left to the next
//! lowering task so unsupported forms fail at this boundary.

use crate::core_ash::{
    CoreAtom, CoreContRef, CoreExpr, CorePrimOp, CoreRow, CoreRowItem, CoreType, CoreValue,
};
use crate::core_ash_validate::ValidCoreProgram;
use crate::cps::{Atom, ContRef, EffectItem, EffectItemKind, EffectRow, Env, PrimOp, Term, Value};
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
/// outside the TASK-1627 lowering subset.
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
/// outside the TASK-1627 lowering subset or a Core carrier cannot be represented
/// by the current CPS IR.
pub fn lower_core_program_with_context(
    program: ValidCoreProgram,
    context: CoreLoweringContext,
) -> Result<Term, CoreLoweringError> {
    let mut state = LoweringState::new(context);
    lower_expr(&program.into_expr(), &mut state)
}

fn lower_expr(expr: &CoreExpr, state: &mut LoweringState) -> Result<Term, CoreLoweringError> {
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
            record_function_row(name, ty, state);
            Ok(Term::LetVal {
                name: name.clone(),
                value: lower_value(value, state)?,
                body: Box::new(lower_expr(body, state)?),
            })
        }
        CoreExpr::LetRec {
            name,
            ty,
            value,
            body,
        } => {
            record_function_row(name, ty, state);
            Ok(Term::LetRec {
                name: name.clone(),
                value: lower_value(value, state)?,
                body: Box::new(lower_expr(body, state)?),
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
            body: Box::new(lower_expr(body, state)?),
        }),
        CoreExpr::LetCall {
            name,
            func,
            args,
            body,
        } => {
            let cont_name = state.fresh_cont_name();
            let cont_row = total_row(body, state)?;
            let cont_body = lower_expr(body, state)?;
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
            let then_local = local_row(then_branch, state)?;
            let else_local = local_row(else_branch, state)?;
            Ok(Term::If {
                cond: lower_atom(cond)?,
                then_branch: Box::new(lower_expr(then_branch, state)?),
                else_branch: Box::new(lower_expr(else_branch, state)?),
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
        CoreExpr::Jump { cont, arg } => {
            let row = cont_row(cont, state);
            Ok(Term::Jump {
                cont: lower_cont_ref(cont),
                arg: lower_atom(arg)?,
                row: lower_row(&row),
            })
        }
        CoreExpr::Raise { .. } => Err(CoreLoweringError::UnsupportedForm {
            detail: "Raise lowering is owned by TASK-1628".to_string(),
        }),
        CoreExpr::Handle { .. } => Err(CoreLoweringError::UnsupportedForm {
            detail: "Handle lowering is owned by TASK-1628".to_string(),
        }),
        CoreExpr::RecordDischarge { .. } => Err(CoreLoweringError::UnsupportedForm {
            detail: "RecordDischarge lowering is owned by TASK-1628".to_string(),
        }),
        CoreExpr::Trap { .. } => Err(CoreLoweringError::UnsupportedForm {
            detail: "Trap lowering is owned by TASK-1628".to_string(),
        }),
    }
}

fn lower_value(value: &CoreValue, state: &mut LoweringState) -> Result<Value, CoreLoweringError> {
    match value {
        CoreValue::Atom(atom) => Ok(Value::Atom(lower_atom(atom)?)),
        CoreValue::Lam { params, body, row } => {
            let cont = state.fresh_cont_name();
            let guard = state.with_current_cont(ContRef::Var(cont.clone()), CoreRow::default());
            let lowered_body = lower_expr(body, state);
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
        CoreValue::DischargeMarker { .. } => Err(CoreLoweringError::UnsupportedForm {
            detail: "DischargeMarker lowering is owned by TASK-1628".to_string(),
        }),
    }
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

fn record_function_row(name: &str, ty: &CoreType, state: &mut LoweringState) {
    if let CoreType::Function { row, .. } = ty {
        state
            .context
            .function_rows
            .insert(name.to_string(), row.clone());
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

fn local_row(expr: &CoreExpr, state: &LoweringState) -> Result<CoreRow, CoreLoweringError> {
    match expr {
        CoreExpr::Atom(_) | CoreExpr::Jump { .. } | CoreExpr::Trap { .. } => Ok(CoreRow::default()),
        CoreExpr::LetVal { body, .. } | CoreExpr::LetRec { body, .. } => local_row(body, state),
        CoreExpr::LetPrim { op, body, .. } => {
            lower_prim_op(op)?;
            local_row(body, state)
        }
        CoreExpr::LetCall { func, body, .. } => Ok(union_rows(
            &function_row_for_atom(func, state).unwrap_or_default(),
            &local_row(body, state)?,
        )),
        CoreExpr::If {
            then_branch,
            else_branch,
            ..
        } => Ok(union_rows(
            &local_row(then_branch, state)?,
            &local_row(else_branch, state)?,
        )),
        CoreExpr::Call { func, .. } => Ok(function_row_for_atom(func, state).unwrap_or_default()),
        CoreExpr::Raise { .. } => Err(CoreLoweringError::UnsupportedForm {
            detail: "Raise row synthesis is owned by TASK-1628".to_string(),
        }),
        CoreExpr::Handle { .. } => Err(CoreLoweringError::UnsupportedForm {
            detail: "Handle row synthesis is owned by TASK-1628".to_string(),
        }),
        CoreExpr::RecordDischarge { .. } => Err(CoreLoweringError::UnsupportedForm {
            detail: "RecordDischarge row synthesis is owned by TASK-1628".to_string(),
        }),
    }
}

fn total_row(expr: &CoreExpr, state: &LoweringState) -> Result<CoreRow, CoreLoweringError> {
    match expr {
        CoreExpr::Atom(_) => Ok(state.context.current_cont_row.clone()),
        CoreExpr::LetVal { body, .. } | CoreExpr::LetRec { body, .. } => total_row(body, state),
        CoreExpr::LetPrim { op, body, .. } => {
            lower_prim_op(op)?;
            total_row(body, state)
        }
        CoreExpr::LetCall { func, body, .. } => Ok(union_rows(
            &function_row_for_atom(func, state).unwrap_or_default(),
            &total_row(body, state)?,
        )),
        CoreExpr::If {
            then_branch,
            else_branch,
            ..
        } => Ok(union_rows(
            &total_row(then_branch, state)?,
            &total_row(else_branch, state)?,
        )),
        CoreExpr::Call { func, .. } => Ok(union_rows(
            &function_row_for_atom(func, state).unwrap_or_default(),
            &state.context.current_cont_row,
        )),
        CoreExpr::Jump { cont, .. } => Ok(cont_row(cont, state)),
        CoreExpr::Trap { .. } => Ok(CoreRow::default()),
        CoreExpr::Raise { .. } => Err(CoreLoweringError::UnsupportedForm {
            detail: "Raise row synthesis is owned by TASK-1628".to_string(),
        }),
        CoreExpr::Handle { .. } => Err(CoreLoweringError::UnsupportedForm {
            detail: "Handle row synthesis is owned by TASK-1628".to_string(),
        }),
        CoreExpr::RecordDischarge { .. } => Err(CoreLoweringError::UnsupportedForm {
            detail: "RecordDischarge row synthesis is owned by TASK-1628".to_string(),
        }),
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
        CoreRowItem::Resource { .. }
        | CoreRowItem::Process { .. }
        | CoreRowItem::Failure { .. } => None,
    }
}

fn dotted_name(path: &[String], leaf: &str) -> String {
    if path.is_empty() {
        return leaf.to_string();
    }
    format!("{}.{}", path.join("."), leaf)
}
