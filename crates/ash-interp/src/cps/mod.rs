//! CPS IR Interpreter
//!
//! Evaluates CPS IR terms in the Ash language.

use ash_core::cps::*;
use chrono::Utc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use thiserror::Error;

pub mod validate;

use validate::{CpsValidationError, validate_cps_program};

/// Errors that can occur during CPS evaluation
#[derive(Error, Debug, Clone, PartialEq)]
pub enum CpsError {
    #[error("unbound variable: {0}")]
    UnboundVariable(Name),
    #[error("unbound label: {0}")]
    UnboundLabel(Name),
    #[error("expected lambda, got: {0:?}")]
    ExpectedLambda(Value),
    #[error("expected continuation, got: {0:?}")]
    ExpectedContinuation(Value),
    #[error("expected thunk closure, got: {0:?}")]
    ExpectedThunk(Value),
    #[error("invalid primitive arguments for {0:?}: {1:?}")]
    InvalidPrimArgs(PrimOp, Vec<Atom>),
    #[error("unhandled effect: {0:?}")]
    UnhandledEffect(EffectOp),
    #[error("trap: {0:?}")]
    Trap(TrapReason),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CachedThunkOutcome {
    Success(Atom),
    Failure(CpsError),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemoCellState {
    Empty,
    Evaluating,
    Filled(CachedThunkOutcome),
}

/// Process-local runtime state for thunk execution.
#[derive(Debug)]
pub struct CpsRuntime {
    pub next_memo_cell: u64,
    pub memo_cells: HashMap<MemoCellId, MemoCellState>,
    pub trace: Vec<ash_core::provenance::TraceEvent>,
}

impl CpsRuntime {
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_memo_cell: 0,
            memo_cells: HashMap::new(),
            trace: Vec::new(),
        }
    }

    pub fn allocate_memo_cell(&mut self) -> MemoCellId {
        let id = MemoCellId::new(self.next_memo_cell);
        self.next_memo_cell += 1;
        self.memo_cells.insert(id, MemoCellState::Empty);
        id
    }
}

impl Default for CpsRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Result type for CPS evaluation
pub type CpsResult<T> = Result<T, CpsError>;

/// Evaluate a CPS term without validation.
///
/// # Safety
/// Assumes the caller has validated the term via `validate_cps_program()` or
/// otherwise trusts the producer. Passing malformed IR may produce undefined
/// behavior (Rust panics, incorrect results, or infinite loops).
#[allow(clippy::result_large_err)]
pub fn eval_unchecked(term: &Term, env: &Env, chain: &HandlerChain) -> CpsResult<Atom> {
    let mut runtime = CpsRuntime::new();
    eval_unchecked_with_runtime(term, env, chain, &mut runtime)
}

#[allow(clippy::result_large_err)]
pub fn eval_unchecked_with_runtime(
    term: &Term,
    env: &Env,
    chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Atom> {
    match term {
        Term::LetVal { name, value, body } => eval_letval(name, value, body, env, chain, runtime),
        Term::LetPrim {
            name,
            op,
            args,
            body,
        } => eval_letprim(name, op, args, body, env, chain, runtime),
        Term::LetCont {
            name,
            param,
            cont_body,
            body,
        } => eval_letcont(name, param, cont_body, body, env, chain, runtime),
        Term::Jump { cont, arg, .. } => eval_jump(cont, arg, env, chain, runtime),
        Term::Call {
            func, args, cont, ..
        } => eval_call(func, args, cont, env, chain, runtime),
        Term::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => eval_if(cond, then_branch, else_branch, env, chain, runtime),
        Term::LetRec { name, value, body } => eval_letrec(name, value, body, env, chain, runtime),
        Term::Match {
            scrutinee,
            arms,
            default,
        } => eval_match(scrutinee, arms, default.as_deref(), env, chain, runtime),
        Term::Raise {
            op, args, resume, ..
        } => eval_raise(op, args, resume, env, chain, runtime),
        Term::Handle {
            clause, body, cont, ..
        } => eval_handle(clause, body, cont, env, chain, runtime),
        Term::RecordDischarge { body, .. } => {
            eval_unchecked_with_runtime(body, env, chain, runtime)
        }
        Term::Return { value } => Ok(eval_atom(value, env)?),
        Term::Trap { reason } => Err(CpsError::Trap(reason.clone())),
    }
}

/// Error type for checked CPS execution
#[derive(Error, Debug, Clone, PartialEq)]
pub enum CpsRunError {
    #[error("validation error: {0}")]
    Validation(#[from] CpsValidationError),
    #[error("runtime error: {0}")]
    Runtime(#[from] CpsError),
}

/// Evaluate a CPS term with validation.
///
/// First validates the term, then runs the unchecked evaluator.
/// This is the safe public entrypoint for untrusted input.
#[allow(clippy::result_large_err)]
pub fn eval_checked(term: &Term, env: &Env, chain: &HandlerChain) -> Result<Atom, CpsRunError> {
    validate_cps_program(term)?;
    let mut runtime = CpsRuntime::new();
    Ok(eval_unchecked_with_runtime(term, env, chain, &mut runtime)?)
}

/// Compatibility wrapper for code that expects `eval_term`.
///
/// # Warning
/// This does NOT validate input. Prefer `eval_checked` for untrusted IR.
/// Prefer `eval_unchecked` when the caller explicitly trusts the producer.
#[allow(clippy::result_large_err)]
pub fn eval_term(term: &Term, env: &Env, chain: &HandlerChain) -> CpsResult<Atom> {
    eval_unchecked(term, env, chain)
}

// ---------------------------------------------------------------------------
// Per-term evaluators
// ---------------------------------------------------------------------------

#[allow(clippy::result_large_err)]
fn eval_letval(
    name: &Name,
    value: &Value,
    body: &Term,
    env: &Env,
    chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Atom> {
    let evaluated_value = eval_value_with_runtime(value, env, chain, runtime)?;
    let new_env = env.clone().with_binding(name.clone(), evaluated_value);
    eval_unchecked_with_runtime(body, &new_env, chain, runtime)
}

#[allow(clippy::result_large_err)]
fn eval_letprim(
    name: &Name,
    op: &PrimOp,
    args: &[Atom],
    body: &Term,
    env: &Env,
    chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Atom> {
    eval_letprim_with_runtime(name, op, args, body, env, chain, runtime)
}

#[allow(clippy::result_large_err)]
fn eval_letprim_with_runtime(
    name: &Name,
    op: &PrimOp,
    args: &[Atom],
    body: &Term,
    env: &Env,
    chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Atom> {
    let result = if *op == PrimOp::ForceThunk {
        return eval_force_thunk_binding(name, args, body, env, chain, runtime);
    } else {
        let resolved_args: CpsResult<Vec<Value>> =
            args.iter().map(|a| eval_atom_to_value(a, env)).collect();
        eval_prim(op, &resolved_args?, env, runtime)?
    };
    let new_env = env.clone().with_binding(name.clone(), result);
    eval_unchecked_with_runtime(body, &new_env, chain, runtime)
}

#[allow(clippy::result_large_err)]
fn eval_force_thunk_binding(
    name: &Name,
    args: &[Atom],
    continuation_body: &Term,
    env: &Env,
    chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Atom> {
    if args.len() != 1 {
        return Err(CpsError::InvalidPrimArgs(PrimOp::ForceThunk, vec![]));
    }
    let maybe_thunk = eval_atom_to_value(&args[0], env)?;
    let mode = match &maybe_thunk {
        Value::ThunkClosure { mode, .. } => *mode,
        _ => return Err(CpsError::ExpectedThunk(maybe_thunk)),
    };

    runtime
        .trace
        .push(ash_core::provenance::TraceEvent::ThunkForceStarted {
            mode: thunk_mode_to_string(mode),
            timestamp: Utc::now(),
        });

    let force_result = force_thunk_value(&maybe_thunk, runtime);
    runtime
        .trace
        .push(ash_core::provenance::TraceEvent::ThunkForceCompleted {
            mode: thunk_mode_to_string(mode),
            outcome: trace_outcome_string(&force_result),
            timestamp: Utc::now(),
        });
    let forced = force_result?;

    let new_env = env.clone().with_binding(name.clone(), Value::Atom(forced));
    eval_unchecked_with_runtime(continuation_body, &new_env, chain, runtime)
}

#[allow(clippy::result_large_err)]
fn eval_letcont(
    name: &Name,
    param: &Name,
    cont_body: &Term,
    body: &Term,
    env: &Env,
    chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Atom> {
    let cont = Value::Cont {
        param: param.clone(),
        body: Box::new(cont_body.clone()),
        captured_env: env.clone(),
        captured_chain: chain.clone(),
        consumed: ConsumedFlag::new(),
        row: EffectRow::default(),
    };
    let new_env = env.clone().with_binding(name.clone(), cont);
    eval_unchecked_with_runtime(body, &new_env, chain, runtime)
}

#[allow(clippy::result_large_err)]
fn eval_jump(
    cont: &ContRef,
    arg: &Atom,
    env: &Env,
    _chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Atom> {
    let arg_value = eval_atom_to_value(arg, env)?;
    let cont_value = resolve_cont(cont, env)?;
    match cont_value {
        Value::Cont {
            param,
            body,
            captured_env,
            captured_chain,
            consumed,
            ..
        } => {
            if consumed.get() {
                return Err(CpsError::Trap(TrapReason::Custom(
                    "resume already consumed".to_string(),
                )));
            }
            consumed.set(true);
            let new_env = captured_env.clone().with_binding(param, arg_value);
            eval_unchecked_with_runtime(&body, &new_env, &captured_chain, runtime)
        }
        _ => Err(CpsError::ExpectedContinuation(cont_value)),
    }
}

#[allow(clippy::collapsible_if)]
#[allow(clippy::result_large_err)]
fn eval_call(
    func: &Atom,
    args: &[Atom],
    cont: &ContRef,
    env: &Env,
    chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Atom> {
    let func_value = resolve_value(func, env)?;
    let arg_values: CpsResult<Vec<Value>> =
        args.iter().map(|a| eval_atom_to_value(a, env)).collect();
    let arg_values = arg_values?;
    let cont_value = resolve_cont(cont, env)?;
    match func_value {
        Value::Lam {
            params,
            cont: lam_cont,
            body,
            captured_env,
            rec_binding,
            ..
        } => {
            if params.len() != arg_values.len() {
                return Err(CpsError::Trap(TrapReason::Custom(format!(
                    "call arity mismatch: expected {} args, got {}",
                    params.len(),
                    arg_values.len()
                ))));
            }
            let mut new_env = captured_env.clone();
            // Overlay recursive binding if marked
            if let Some(rec_name) = rec_binding
                && let Some(rec_value) = env.lookup(&rec_name)
            {
                new_env = new_env.with_binding(rec_name, rec_value.clone());
            }
            for (param, arg) in params.iter().zip(arg_values.iter()) {
                new_env = new_env.with_binding(param.clone(), arg.clone());
            }
            new_env = new_env.with_binding(lam_cont.clone(), cont_value);
            eval_unchecked_with_runtime(&body, &new_env, chain, runtime)
        }
        _ => Err(CpsError::ExpectedLambda(func_value)),
    }
}

#[allow(clippy::result_large_err)]
fn eval_if(
    cond: &Atom,
    then_branch: &Term,
    else_branch: &Term,
    env: &Env,
    chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Atom> {
    let cond_value = eval_atom(cond, env)?;
    match cond_value {
        Atom::Bool(true) => eval_unchecked_with_runtime(then_branch, env, chain, runtime),
        Atom::Bool(false) => eval_unchecked_with_runtime(else_branch, env, chain, runtime),
        _ => Err(CpsError::InvalidPrimArgs(PrimOp::Eq, vec![])),
    }
}

fn mark_rec_binding(value: &Value, rec_name: &Name) -> Value {
    match value {
        Value::Lam {
            params,
            cont,
            body,
            captured_env,
            rec_binding,
            row,
        } => Value::Lam {
            params: params.clone(),
            cont: cont.clone(),
            body: body.clone(),
            captured_env: captured_env.clone(),
            rec_binding: rec_binding.clone().or_else(|| Some(rec_name.clone())),
            row: row.clone(),
        },
        Value::Record { fields } => {
            let mut new_fields = Vec::new();
            for (field_name, field_value) in fields {
                new_fields.push((field_name.clone(), mark_rec_binding(field_value, rec_name)));
            }
            Value::Record { fields: new_fields }
        }
        Value::Tuple { elems } => {
            let mut new_elems = Vec::new();
            for elem in elems {
                new_elems.push(mark_rec_binding(elem, rec_name));
            }
            Value::Tuple { elems: new_elems }
        }
        other => other.clone(),
    }
}

#[allow(clippy::result_large_err)]
fn eval_letrec(
    name: &Name,
    value: &Value,
    body: &Term,
    env: &Env,
    chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Atom> {
    let mut new_env = env.clone();
    let shared_env = Rc::new(RefCell::new(new_env.clone()));
    let lam_value = match value {
        Value::Lam {
            params,
            cont,
            body: lam_body,
            ..
        } => {
            let lam = Value::Lam {
                params: params.clone(),
                cont: cont.clone(),
                body: lam_body.clone(),
                captured_env: new_env.clone(),
                rec_binding: Some(name.clone()),
                row: EffectRow::default(),
            };
            {
                let mut env_ref = shared_env.borrow_mut();
                *env_ref = env_ref.clone().with_binding(name.clone(), lam.clone());
            }
            lam
        }
        Value::Record { fields } => {
            let mut new_fields = Vec::new();
            for (field_name, field_value) in fields {
                new_fields.push((field_name.clone(), mark_rec_binding(field_value, name)));
            }
            Value::Record { fields: new_fields }
        }
        Value::Tuple { elems } => {
            let mut new_elems = Vec::new();
            for elem in elems {
                new_elems.push(mark_rec_binding(elem, name));
            }
            Value::Tuple { elems: new_elems }
        }
        other => other.clone(),
    };
    new_env = new_env.with_binding(name.clone(), lam_value);
    eval_unchecked_with_runtime(body, &new_env, chain, runtime)
}

#[allow(clippy::result_large_err)]
fn eval_raise(
    op: &EffectOp,
    args: &[Atom],
    resume: &ContRef,
    env: &Env,
    chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Atom> {
    // Check for shallow handler first
    if let Some((clause, handler_idx)) = chain.find_handler(op) {
        let arg_values: CpsResult<Vec<Atom>> = args.iter().map(|a| eval_atom(a, env)).collect();
        let arg_values = arg_values?;
        // Remove the matched shallow handler from the chain BEFORE evaluating clause body
        let mut body_chain = chain.clone();
        body_chain.frames.remove(handler_idx);
        // Build resume continuation that captures current env and chain WITHOUT the handler
        let resume_chain = body_chain.clone();
        let resume_cont = Value::Cont {
            param: clause.resume.clone(),
            body: Box::new(Term::Jump {
                cont: resume.clone(),
                arg: Atom::Var(clause.resume.clone()),
                row: EffectRow::default(),
            }),
            captured_env: env.clone(),
            captured_chain: resume_chain,
            consumed: ConsumedFlag::new(),
            row: EffectRow::default(),
        };
        let mut new_env = env.clone();
        for (param, arg) in clause.params.iter().zip(arg_values.iter()) {
            new_env = new_env.with_binding(param.clone(), Value::Atom(arg.clone()));
        }
        new_env = new_env.with_binding(clause.resume.clone(), resume_cont);
        eval_unchecked_with_runtime(&clause.body, &new_env, &body_chain, runtime)
    } else if let Some((handler_name, _provider_idx)) = chain.find_provider(op) {
        // Provider dispatch: invoke the provider handler directly
        let handler_value = env
            .lookup(&handler_name)
            .ok_or_else(|| CpsError::UnboundVariable(handler_name.clone()))?
            .clone();
        match handler_value {
            Value::Lam {
                params,
                cont: lam_cont,
                body,
                captured_env,
                rec_binding,
                ..
            } => {
                let arg_values: CpsResult<Vec<Atom>> =
                    args.iter().map(|a| eval_atom(a, env)).collect();
                let arg_values = arg_values?;
                if params.len() != arg_values.len() {
                    return Err(CpsError::Trap(TrapReason::Custom(format!(
                        "provider handler arity mismatch: expected {} args, got {}",
                        params.len(),
                        arg_values.len()
                    ))));
                }
                let resume_value = resolve_cont(resume, env)?;
                let mut new_env = captured_env.clone();
                if let Some(rec_name) = rec_binding
                    && let Some(rec_value) = env.lookup(&rec_name)
                {
                    new_env = new_env.with_binding(rec_name, rec_value.clone());
                }
                for (param, arg) in params.iter().zip(arg_values.iter()) {
                    new_env = new_env.with_binding(param.clone(), Value::Atom(arg.clone()));
                }
                new_env = new_env.with_binding(lam_cont.clone(), resume_value);
                eval_unchecked_with_runtime(&body, &new_env, chain, runtime)
            }
            _ => Err(CpsError::ExpectedLambda(handler_value)),
        }
    } else {
        Err(CpsError::UnhandledEffect(op.clone()))
    }
}

#[allow(clippy::result_large_err)]
fn eval_handle(
    clause: &HandlerClause,
    body: &Term,
    cont: &ContRef,
    env: &Env,
    chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Atom> {
    let cont_value = resolve_cont(cont, env)?;
    let mut new_chain = chain.clone();
    new_chain.push(HandlerFrame::Shallow {
        clause: clause.clone(),
    });
    let mut new_env = env.clone();
    new_env = new_env.with_binding(clause.resume.clone(), cont_value);
    eval_unchecked_with_runtime(body, &new_env, &new_chain, runtime)
}

/// Evaluate a value (atoms pass through, lambdas capture env if not already captured, conts are inert)
#[allow(clippy::result_large_err)]
fn eval_value_with_runtime(
    value: &Value,
    env: &Env,
    chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Value> {
    match value {
        Value::Atom(atom) => Ok(Value::Atom(eval_atom(atom, env)?)),
        Value::Lam {
            params,
            cont,
            body,
            captured_env,
            rec_binding,
            row,
        } => {
            // If captured_env is empty, capture current env; otherwise preserve existing capture
            let env_to_capture = if captured_env.bindings.is_empty() {
                env.clone()
            } else {
                captured_env.clone()
            };
            Ok(Value::Lam {
                params: params.clone(),
                cont: cont.clone(),
                body: body.clone(),
                captured_env: env_to_capture,
                rec_binding: rec_binding.clone(),
                row: row.clone(),
            })
        }
        Value::ThunkClosure {
            mode,
            body,
            captured_env,
            captured_chain,
            row,
            memo_cell,
        } => {
            let env_to_capture = if captured_env.bindings.is_empty() {
                env.clone()
            } else {
                captured_env.clone()
            };
            let chain_to_capture = if captured_chain.frames.is_empty() {
                chain.clone()
            } else {
                captured_chain.clone()
            };
            let allocated_cell = match mode {
                ThunkMode::Memo if memo_cell.is_none() => Some(runtime.allocate_memo_cell()),
                _ => *memo_cell,
            };

            runtime
                .trace
                .push(ash_core::provenance::TraceEvent::ThunkConstructed {
                    mode: thunk_mode_to_string(*mode),
                    row: effect_row_to_strings(row),
                    timestamp: Utc::now(),
                });

            Ok(Value::ThunkClosure {
                mode: *mode,
                body: body.clone(),
                captured_env: env_to_capture,
                captured_chain: chain_to_capture,
                row: row.clone(),
                memo_cell: allocated_cell,
            })
        }
        Value::Record { fields } => {
            let mut new_fields = Vec::new();
            for (name, field_value) in fields {
                new_fields.push((
                    name.clone(),
                    eval_value_with_runtime(field_value, env, chain, runtime)?,
                ));
            }
            Ok(Value::Record { fields: new_fields })
        }
        Value::Tuple { elems } => {
            let mut new_elems = Vec::new();
            for elem in elems {
                new_elems.push(eval_value_with_runtime(elem, env, chain, runtime)?);
            }
            Ok(Value::Tuple { elems: new_elems })
        }
        other => Ok(other.clone()),
    }
}

/// Evaluate an atom (resolve variables)
#[allow(clippy::result_large_err)]
fn eval_atom(atom: &Atom, env: &Env) -> CpsResult<Atom> {
    match atom {
        Atom::Var(name) => {
            let value = env
                .lookup(name)
                .ok_or_else(|| CpsError::UnboundVariable(name.clone()))?;
            match value {
                Value::Atom(a) => Ok(a.clone()),
                _ => Err(CpsError::UnboundVariable(name.clone())),
            }
        }
        other => Ok(other.clone()),
    }
}

/// Resolve a continuation reference to a value
#[allow(clippy::result_large_err)]
fn resolve_cont(cont: &ContRef, env: &Env) -> CpsResult<Value> {
    match cont {
        ContRef::Label(name) => env
            .lookup(name)
            .ok_or_else(|| CpsError::UnboundLabel(name.clone()))
            .cloned(),
        ContRef::Var(name) => env
            .lookup(name)
            .ok_or_else(|| CpsError::UnboundVariable(name.clone()))
            .cloned(),
    }
}

/// Resolve a value from an atom
#[allow(clippy::result_large_err)]
fn resolve_value(atom: &Atom, env: &Env) -> CpsResult<Value> {
    match atom {
        Atom::Var(name) => env
            .lookup(name)
            .ok_or_else(|| CpsError::UnboundVariable(name.clone()))
            .cloned(),
        other => Ok(Value::Atom(other.clone())),
    }
}

/// Evaluate match dispatch on constructor tags
#[allow(clippy::result_large_err)]
fn eval_match(
    scrutinee: &Atom,
    arms: &[(Name, Box<Term>)],
    default: Option<&Term>,
    env: &Env,
    chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Atom> {
    let scrut_value = resolve_value(scrutinee, env)?;
    match scrut_value {
        Value::Tuple { elems } => {
            let tag_value = elems.first().ok_or_else(|| {
                CpsError::Trap(TrapReason::Custom("empty tuple in match".to_string()))
            })?;
            match tag_value {
                Value::Atom(Atom::ConstructorName(name)) => {
                    for (arm_tag, body) in arms {
                        if arm_tag == name {
                            return eval_unchecked_with_runtime(body, env, chain, runtime);
                        }
                    }
                    if let Some(default_body) = default {
                        return eval_unchecked_with_runtime(default_body, env, chain, runtime);
                    }
                    Err(CpsError::Trap(TrapReason::Custom(
                        "no matching arm".to_string(),
                    )))
                }
                _ => Err(CpsError::Trap(TrapReason::Custom(
                    "match scrutinee tag is not a ConstructorName".to_string(),
                ))),
            }
        }
        _ => Err(CpsError::Trap(TrapReason::Custom(
            "match scrutinee is not a tuple".to_string(),
        ))),
    }
}

/// Evaluate an atom to a Value (resolve variables, pass through literals)
#[allow(clippy::result_large_err)]
fn eval_atom_to_value(atom: &Atom, env: &Env) -> CpsResult<Value> {
    match atom {
        Atom::Var(name) => env
            .lookup(name)
            .ok_or_else(|| CpsError::UnboundVariable(name.clone()))
            .cloned(),
        other => Ok(Value::Atom(other.clone())),
    }
}

/// Evaluate a primitive operation
#[allow(clippy::result_large_err)]
fn eval_prim(
    op: &PrimOp,
    args: &[Value],
    _env: &Env,
    _runtime: &mut CpsRuntime,
) -> CpsResult<Value> {
    let make_err = || CpsError::InvalidPrimArgs(op.clone(), vec![]);
    match *op {
        PrimOp::Add => {
            let a = args.first().ok_or_else(make_err)?;
            let b = args.get(1).ok_or_else(make_err)?;
            match (a, b) {
                (Value::Atom(Atom::Int(x)), Value::Atom(Atom::Int(y))) => {
                    Ok(Value::Atom(Atom::Int(x + y)))
                }
                _ => Err(make_err()),
            }
        }
        PrimOp::Sub => {
            let a = args.first().ok_or_else(make_err)?;
            let b = args.get(1).ok_or_else(make_err)?;
            match (a, b) {
                (Value::Atom(Atom::Int(x)), Value::Atom(Atom::Int(y))) => {
                    Ok(Value::Atom(Atom::Int(x - y)))
                }
                _ => Err(make_err()),
            }
        }
        PrimOp::Mul => {
            let a = args.first().ok_or_else(make_err)?;
            let b = args.get(1).ok_or_else(make_err)?;
            match (a, b) {
                (Value::Atom(Atom::Int(x)), Value::Atom(Atom::Int(y))) => {
                    Ok(Value::Atom(Atom::Int(x * y)))
                }
                _ => Err(make_err()),
            }
        }
        PrimOp::Div => {
            let a = args.first().ok_or_else(make_err)?;
            let b = args.get(1).ok_or_else(make_err)?;
            match (a, b) {
                (Value::Atom(Atom::Int(x)), Value::Atom(Atom::Int(y))) => {
                    if *y == 0 {
                        Err(make_err())
                    } else {
                        Ok(Value::Atom(Atom::Int(x / y)))
                    }
                }
                _ => Err(make_err()),
            }
        }
        PrimOp::Eq => {
            let a = args.first().ok_or_else(make_err)?;
            let b = args.get(1).ok_or_else(make_err)?;
            Ok(Value::Atom(Atom::Bool(a == b)))
        }
        PrimOp::Ne => {
            let a = args.first().ok_or_else(make_err)?;
            let b = args.get(1).ok_or_else(make_err)?;
            Ok(Value::Atom(Atom::Bool(a != b)))
        }
        PrimOp::Lt => {
            let a = args.first().ok_or_else(make_err)?;
            let b = args.get(1).ok_or_else(make_err)?;
            match (a, b) {
                (Value::Atom(Atom::Int(x)), Value::Atom(Atom::Int(y))) => {
                    Ok(Value::Atom(Atom::Bool(x < y)))
                }
                _ => Err(make_err()),
            }
        }
        PrimOp::Le => {
            let a = args.first().ok_or_else(make_err)?;
            let b = args.get(1).ok_or_else(make_err)?;
            match (a, b) {
                (Value::Atom(Atom::Int(x)), Value::Atom(Atom::Int(y))) => {
                    Ok(Value::Atom(Atom::Bool(x <= y)))
                }
                _ => Err(make_err()),
            }
        }
        PrimOp::Gt => {
            let a = args.first().ok_or_else(make_err)?;
            let b = args.get(1).ok_or_else(make_err)?;
            match (a, b) {
                (Value::Atom(Atom::Int(x)), Value::Atom(Atom::Int(y))) => {
                    Ok(Value::Atom(Atom::Bool(x > y)))
                }
                _ => Err(make_err()),
            }
        }
        PrimOp::Ge => {
            let a = args.first().ok_or_else(make_err)?;
            let b = args.get(1).ok_or_else(make_err)?;
            match (a, b) {
                (Value::Atom(Atom::Int(x)), Value::Atom(Atom::Int(y))) => {
                    Ok(Value::Atom(Atom::Bool(x >= y)))
                }
                _ => Err(make_err()),
            }
        }
        PrimOp::Neg => {
            let a = args.first().ok_or_else(make_err)?;
            match a {
                Value::Atom(Atom::Int(x)) => Ok(Value::Atom(Atom::Int(-x))),
                _ => Err(make_err()),
            }
        }
        PrimOp::Not => {
            let a = args.first().ok_or_else(make_err)?;
            match a {
                Value::Atom(Atom::Bool(x)) => Ok(Value::Atom(Atom::Bool(!x))),
                _ => Err(make_err()),
            }
        }
        PrimOp::RecordGet(ref field) => {
            let record = args.first().ok_or_else(make_err)?;
            match record {
                Value::Record { fields } => fields
                    .iter()
                    .find(|(f, _)| f == field)
                    .map(|(_, v)| v.clone())
                    .ok_or_else(make_err),
                _ => Err(make_err()),
            }
        }
        PrimOp::TupleGet(index) => {
            let tuple = args.first().ok_or_else(make_err)?;
            match tuple {
                Value::Tuple { elems } => elems.get(index).cloned().ok_or_else(make_err),
                _ => Err(make_err()),
            }
        }
        _ => Err(CpsError::InvalidPrimArgs(op.clone(), vec![])),
    }
}

#[allow(clippy::result_large_err)]
fn force_thunk_value(thunk: &Value, runtime: &mut CpsRuntime) -> CpsResult<Atom> {
    let Value::ThunkClosure {
        mode, memo_cell, ..
    } = thunk
    else {
        return Err(CpsError::ExpectedThunk(thunk.clone()));
    };

    match (mode, *memo_cell) {
        (ThunkMode::Memo, None) => Err(CpsError::Trap(TrapReason::Custom(
            "memo thunk missing memo cell".to_string(),
        ))),
        (ThunkMode::Memo, Some(cell_id)) => {
            let state = runtime.memo_cells.get(&cell_id).cloned();
            match state {
                Some(MemoCellState::Filled(outcome)) => match outcome {
                    CachedThunkOutcome::Success(atom) => {
                        runtime
                            .trace
                            .push(ash_core::provenance::TraceEvent::MemoCacheHit {
                                outcome: trace_outcome_string::<Atom>(&Ok(atom.clone())),
                                timestamp: Utc::now(),
                            });
                        Ok(atom)
                    }
                    CachedThunkOutcome::Failure(error) => {
                        runtime
                            .trace
                            .push(ash_core::provenance::TraceEvent::MemoReplayFailure {
                                reason: trace_error_outcome_string(&error),
                                timestamp: Utc::now(),
                            });
                        Err(error)
                    }
                },
                Some(MemoCellState::Evaluating) => {
                    runtime
                        .trace
                        .push(ash_core::provenance::TraceEvent::MemoReentrantRejected {
                            timestamp: Utc::now(),
                        });
                    Err(CpsError::Trap(TrapReason::Custom(
                        "re-entrant memo force".to_string(),
                    )))
                }
                Some(MemoCellState::Empty) | None => {
                    runtime
                        .memo_cells
                        .insert(cell_id, MemoCellState::Evaluating);
                    let result = run_thunk_body_with_runtime(thunk, runtime);
                    match result {
                        Ok(atom) => {
                            runtime
                                .trace
                                .push(ash_core::provenance::TraceEvent::MemoCacheFilled {
                                    outcome: trace_outcome_string(&Ok(atom.clone())),
                                    timestamp: Utc::now(),
                                });
                            runtime.memo_cells.insert(
                                cell_id,
                                MemoCellState::Filled(CachedThunkOutcome::Success(atom.clone())),
                            );
                            Ok(atom)
                        }
                        Err(error) if is_cacheable_thunk_error(&error) => {
                            runtime
                                .trace
                                .push(ash_core::provenance::TraceEvent::MemoCacheFilled {
                                    outcome: trace_outcome_string::<Atom>(&Err(error.clone())),
                                    timestamp: Utc::now(),
                                });
                            runtime.memo_cells.insert(
                                cell_id,
                                MemoCellState::Filled(CachedThunkOutcome::Failure(error.clone())),
                            );
                            Err(error)
                        }
                        Err(error) => {
                            runtime.memo_cells.insert(cell_id, MemoCellState::Empty);
                            Err(error)
                        }
                    }
                }
            }
        }
        (ThunkMode::Lazy, _) => run_thunk_body_with_runtime(thunk, runtime),
    }
}

#[allow(clippy::result_large_err)]
fn is_cacheable_thunk_error(error: &CpsError) -> bool {
    matches!(error, CpsError::Trap(_) | CpsError::UnhandledEffect(_))
}

#[allow(clippy::result_large_err)]
fn run_thunk_body_with_runtime(thunk: &Value, runtime: &mut CpsRuntime) -> CpsResult<Atom> {
    let Value::ThunkClosure {
        mode,
        body,
        captured_env,
        captured_chain,
        ..
    } = thunk
    else {
        return Err(CpsError::ExpectedThunk(thunk.clone()));
    };

    let Value::Lam {
        params,
        cont: lam_cont,
        body: lam_body,
        rec_binding,
        ..
    } = body.as_ref()
    else {
        return Err(CpsError::ExpectedThunk((**body).clone()));
    };

    if !params.is_empty() {
        return Err(CpsError::InvalidPrimArgs(PrimOp::ForceThunk, vec![]));
    }

    runtime.trace.push(
        ash_core::provenance::TraceEvent::ThunkBodyEvaluationStarted {
            mode: thunk_mode_to_string(*mode),
            timestamp: Utc::now(),
        },
    );

    let force_result_name = "__force_result";
    let cont_value = Value::Cont {
        param: force_result_name.to_string(),
        body: Box::new(Term::Return {
            value: Atom::Var(force_result_name.to_string()),
        }),
        captured_env: Env::new(),
        captured_chain: captured_chain.clone(),
        consumed: ConsumedFlag::new(),
        row: EffectRow::default(),
    };

    let mut body_env = captured_env.clone();
    if let Some(rec_name) = rec_binding
        && let Some(rec_value) = captured_env.lookup(rec_name)
    {
        body_env = body_env.with_binding(rec_name.clone(), rec_value.clone());
    }
    body_env = body_env.with_binding(lam_cont.clone(), cont_value);
    let result = eval_unchecked_with_runtime(lam_body, &body_env, captured_chain, runtime);
    runtime.trace.push(
        ash_core::provenance::TraceEvent::ThunkBodyEvaluationCompleted {
            mode: thunk_mode_to_string(*mode),
            outcome: trace_outcome_string(&result),
            timestamp: Utc::now(),
        },
    );
    result
}

fn thunk_mode_to_string(mode: ThunkMode) -> String {
    match mode {
        ThunkMode::Lazy => "lazy".to_string(),
        ThunkMode::Memo => "memo".to_string(),
    }
}

fn trace_outcome_string<T>(result: &CpsResult<T>) -> String {
    match result {
        Ok(_) => "success".to_string(),
        Err(CpsError::Trap(_)) => "trap".to_string(),
        Err(CpsError::UnhandledEffect(_)) => "unhandled-effect".to_string(),
        Err(_) => "runtime-error".to_string(),
    }
}

fn trace_error_outcome_string(error: &CpsError) -> String {
    match error {
        CpsError::Trap(_) => "trap".to_string(),
        CpsError::UnhandledEffect(_) => "unhandled-effect".to_string(),
        _ => "runtime-error".to_string(),
    }
}

fn effect_row_to_strings(row: &EffectRow) -> Vec<String> {
    row.items
        .iter()
        .map(|item| format!("{} {}", item.namespace, item.name))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn captured_effect_op() -> EffectOp {
        EffectOp {
            item: EffectItem {
                namespace: "cap".to_string(),
                name: "db.read".to_string(),
                kind: EffectItemKind::Capability,
            },
            arg_types: vec!["String".to_string()],
            result_type: "Int".to_string(),
        }
    }

    fn provider_handler(body_result: i64) -> Value {
        Value::Lam {
            params: vec!["msg".to_string()],
            cont: "k".to_string(),
            body: Box::new(Term::Jump {
                cont: ContRef::Var("k".to_string()),
                arg: Atom::Int(body_result),
                row: EffectRow::default(),
            }),
            captured_env: Env::new(),
            rec_binding: None,
            row: EffectRow::default(),
        }
    }

    fn captured_thunk(mode: ThunkMode, op: &EffectOp) -> Value {
        Value::ThunkClosure {
            mode,
            body: Box::new(Value::Lam {
                params: vec![],
                cont: "resume".to_string(),
                body: Box::new(Term::Raise {
                    op: op.clone(),
                    args: vec![Atom::String("resource".to_string())],
                    resume: ContRef::Var("resume".to_string()),
                    row: EffectRow::default(),
                }),
                captured_env: Env::new(),
                rec_binding: None,
                row: EffectRow::default(),
            }),
            captured_env: Env::new(),
            captured_chain: HandlerChain::new(),
            row: EffectRow::default(),
            memo_cell: None,
        }
    }

    #[allow(clippy::result_large_err)]
    fn force_thunk_result(
        thunk: Value,
        force_chain: HandlerChain,
        run_time: &mut CpsRuntime,
    ) -> CpsResult<Atom> {
        let env = Env::new().with_binding("thunk".to_string(), thunk);
        let body = Term::LetCont {
            name: "out".to_string(),
            param: "result".to_string(),
            cont_body: Box::new(Term::Return {
                value: Atom::Var("result".to_string()),
            }),
            body: Box::new(Term::LetPrim {
                name: "forced".to_string(),
                op: PrimOp::ForceThunk,
                args: vec![Atom::Var("thunk".to_string())],
                body: Box::new(Term::Jump {
                    cont: ContRef::Label("out".to_string()),
                    arg: Atom::Var("forced".to_string()),
                    row: EffectRow::default(),
                }),
            }),
        };

        eval_unchecked_with_runtime(&body, &env, &force_chain, run_time)
    }

    #[test]
    fn thunk_value_captures_handler_chain_for_future_force() {
        let op = captured_effect_op();
        let env = Env::new().with_binding("provider".to_string(), provider_handler(17));
        let mut chain_with_provider = HandlerChain::new();
        chain_with_provider.push(HandlerFrame::Provider {
            op: op.clone(),
            handler: "provider".to_string(),
        });

        let thunk = captured_thunk(ThunkMode::Memo, &op);
        let mut runtime = CpsRuntime::new();
        let evaluated = eval_value_with_runtime(&thunk, &env, &chain_with_provider, &mut runtime)
            .expect("memo thunk should evaluate");
        if let Value::ThunkClosure { captured_chain, .. } = &evaluated {
            assert_eq!(*captured_chain, chain_with_provider);
        } else {
            panic!("expected thunk closure");
        }
        let thunk = evaluated;

        let result = force_thunk_result(thunk, HandlerChain::new(), &mut runtime);
        assert_eq!(result, Ok(Atom::Int(17)));
    }

    #[test]
    fn lazy_thunk_uses_construction_chain_for_force_dispatch() {
        let op = captured_effect_op();
        let env = Env::new().with_binding("provider".to_string(), provider_handler(23));
        let mut chain_with_provider = HandlerChain::new();
        chain_with_provider.push(HandlerFrame::Provider {
            op: op.clone(),
            handler: "provider".to_string(),
        });

        let thunk = captured_thunk(ThunkMode::Lazy, &op);
        let mut runtime = CpsRuntime::new();
        let evaluated = eval_value_with_runtime(&thunk, &env, &chain_with_provider, &mut runtime)
            .expect("lazy thunk should evaluate");
        if let Value::ThunkClosure { captured_chain, .. } = &evaluated {
            assert_eq!(*captured_chain, chain_with_provider);
        } else {
            panic!("expected thunk closure");
        }
        let thunk = evaluated;

        let result = force_thunk_result(thunk, HandlerChain::new(), &mut runtime);
        assert_eq!(result, Ok(Atom::Int(23)));
    }
}
