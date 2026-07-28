//! Engine-private CPS evaluation kernel.
//!
//! The Engine owns validation and execution of checked CPS.  No non-Engine
//! crate can name this module or its evaluator surface.

// This preserves the migrated CPS kernel and its regression suite without
// widening TASK-2037 into a lint-driven refactor of historical semantics.
#![allow(
    clippy::doc_markdown,
    clippy::implicit_clone,
    clippy::manual_string_new,
    clippy::match_same_arms,
    clippy::match_wildcard_for_single_variants,
    clippy::missing_const_for_fn,
    clippy::needless_pass_by_ref_mut,
    clippy::needless_pass_by_value,
    clippy::redundant_clone,
    clippy::too_many_lines,
    clippy::trivially_copy_pass_by_ref,
    clippy::uninlined_format_args,
    clippy::unnested_or_patterns,
    clippy::use_self,
    clippy::wildcard_imports
)]

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
    #[cfg(test)]
    #[error("expected an atomic CPS result, got: {0:?}")]
    ExpectedAtomicResult(Value),
    #[error("invalid primitive arguments for {0:?}: {1:?}")]
    InvalidPrimArgs(PrimOp, Vec<Atom>),
    #[error("unhandled effect: {0:?}")]
    UnhandledEffect(EffectOp),
    #[error("trap: {0:?}")]
    Trap(TrapReason),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CachedThunkOutcome {
    Success(Value),
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
#[cfg(test)]
pub fn eval_unchecked(term: &Term, env: &Env, chain: &HandlerChain) -> CpsResult<Atom> {
    let mut runtime = CpsRuntime::new();
    eval_unchecked_with_runtime(term, env, chain, &mut runtime).and_then(value_to_atom)
}

#[allow(clippy::result_large_err)]
pub fn eval_unchecked_with_runtime(
    term: &Term,
    env: &Env,
    chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Value> {
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
            row,
            multiplicity,
        } => eval_letcont(
            name,
            param,
            cont_body,
            row,
            multiplicity,
            body,
            env,
            chain,
            runtime,
        ),
        Term::LetContCall {
            name,
            cont,
            arg,
            row,
            body,
        } => eval_letcontcall(name, cont, arg, row, body, env, chain, runtime),
        Term::Jump { cont, arg, .. } => eval_jump(cont, arg, env, chain, runtime),
        Term::JumpValue { cont, arg, .. } => eval_jump_value(cont, arg, env, chain, runtime),
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
        Term::Return { value } => eval_value_with_runtime(value, env, chain, runtime),
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

/// Observable terminal result of checked CPS evaluation.
///
/// This is a prototype projection boundary for the canonical CPS kernel:
/// `Return` and `Trap` are distinct terminal observations. It does not make
/// the existing CPS evaluator a complete production semantics.
#[derive(Debug, Clone, PartialEq)]
pub enum CpsTerminalOutcome {
    /// Successful terminal completion with its observed value.
    Return(Value),
    /// Structured terminal failure with its declared reason.
    Trap(TrapReason),
}

/// Evaluate a CPS term with validation.
///
/// First validates the term, then runs the unchecked evaluator.
/// This is the safe public entrypoint for untrusted input.
#[allow(clippy::result_large_err)]
#[cfg(test)]
pub fn eval_checked(term: &Term, env: &Env, chain: &HandlerChain) -> Result<Atom, CpsRunError> {
    validate_cps_program(term)?;
    let mut runtime = CpsRuntime::new();
    Ok(value_to_atom(eval_unchecked_with_runtime(
        term,
        env,
        chain,
        &mut runtime,
    )?)?)
}

/// Evaluate checked CPS and project its terminal observable.
///
/// Validation runs before evaluation. A structured CPS trap is projected as
/// [`CpsTerminalOutcome::Trap`]; validation and other runtime failures remain
/// [`CpsRunError`] values. The atom-only test helper retains its established
/// `Result<Atom, CpsRunError>` contract.
#[allow(clippy::result_large_err)]
pub fn eval_checked_terminal(
    term: &Term,
    env: &Env,
    chain: &HandlerChain,
) -> Result<CpsTerminalOutcome, CpsRunError> {
    validate_cps_program(term)?;
    let mut runtime = CpsRuntime::new();
    match eval_unchecked_with_runtime(term, env, chain, &mut runtime) {
        Ok(value) => Ok(CpsTerminalOutcome::Return(value)),
        Err(CpsError::Trap(reason)) => Ok(CpsTerminalOutcome::Trap(reason)),
        Err(error) => Err(CpsRunError::Runtime(error)),
    }
}

/// Compatibility wrapper for code that expects `eval_term`.
///
/// # Warning
/// This does NOT validate input. Prefer `eval_checked` for untrusted IR.
/// Prefer `eval_unchecked` when the caller explicitly trusts the producer.
#[allow(clippy::result_large_err)]
#[cfg(test)]
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
) -> CpsResult<Value> {
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
) -> CpsResult<Value> {
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
) -> CpsResult<Value> {
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
) -> CpsResult<Value> {
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

    let new_env = env.clone().with_binding(name.clone(), forced);
    eval_unchecked_with_runtime(continuation_body, &new_env, chain, runtime)
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::result_large_err)]
fn eval_letcont(
    name: &Name,
    param: &Name,
    cont_body: &Term,
    row: &EffectRow,
    multiplicity: &ContMultiplicity,
    body: &Term,
    env: &Env,
    chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Value> {
    let cont = Value::Cont {
        param: param.clone(),
        body: Box::new(cont_body.clone()),
        captured_env: env.clone(),
        captured_chain: chain.clone(),
        consumed: ConsumedFlag::new(),
        row: row.clone(),
        multiplicity: *multiplicity,
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
) -> CpsResult<Value> {
    let arg_value = eval_atom_to_value(arg, env)?;
    let cont_value = resolve_cont(cont, env)?;
    invoke_cont(&cont_value, &arg_value, runtime)
}

#[allow(clippy::result_large_err)]
fn eval_jump_value(
    cont: &ContRef,
    arg: &Value,
    env: &Env,
    chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Value> {
    let arg_value = eval_value_with_runtime(arg, env, chain, runtime)?;
    let cont_value = resolve_cont(cont, env)?;
    invoke_cont(&cont_value, &arg_value, runtime)
}

/// Invoke a continuation value with an argument, branching on multiplicity.
///
/// - Affine: reject if already consumed, mark consumed, then evaluate.
/// - MultiShotPure: evaluate without inspecting or setting consumed state.
///
/// Shared by `Jump` and `LetContCall` so both invocation forms obey the same
/// multiplicity discipline.
#[allow(clippy::result_large_err)]
fn invoke_cont(
    cont_value: &Value,
    arg_value: &Value,
    runtime: &mut CpsRuntime,
) -> CpsResult<Value> {
    let Value::Cont {
        param,
        body,
        captured_env,
        captured_chain,
        consumed,
        row,
        multiplicity,
    } = cont_value
    else {
        return Err(CpsError::ExpectedContinuation(cont_value.clone()));
    };

    match multiplicity {
        ContMultiplicity::Affine => {
            if consumed.get() {
                return Err(CpsError::Trap(TrapReason::Custom(
                    "resume already consumed".to_string(),
                )));
            }
            consumed.set(true);
            let new_env = captured_env
                .clone()
                .with_binding(param.clone(), arg_value.clone());
            eval_unchecked_with_runtime(body, &new_env, captured_chain, runtime)
        }
        ContMultiplicity::MultiShotPure => {
            // Runtime fail-closed: multi-shot-pure with a non-empty row is
            // invalid even if validation was bypassed (SPEC-102 §6.4.1).
            if !row.items.is_empty() {
                return Err(CpsError::Trap(TrapReason::Custom(format!(
                    "multi-shot-pure continuation has non-empty declared row {row:?}"
                ))));
            }
            // Multi-shot continuations must not inspect or set the consumed flag.
            // Each invocation uses the captured environment and handler chain
            // independently.
            let new_env = captured_env
                .clone()
                .with_binding(param.clone(), arg_value.clone());
            eval_unchecked_with_runtime(body, &new_env, captured_chain, runtime)
        }
    }
}

/// Answer-binding continuation invocation.
///
/// Invokes the continuation, then binds the returned answer to `name` before
/// evaluating `body`. Affine continuations are consumed; multi-shot-pure
/// continuations remain reusable.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::result_large_err)]
fn eval_letcontcall(
    name: &Name,
    cont: &Name,
    arg: &Atom,
    _row: &EffectRow,
    body: &Term,
    env: &Env,
    chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Value> {
    let arg_value = eval_atom_to_value(arg, env)?;
    let cont_value = env
        .lookup(cont)
        .cloned()
        .ok_or_else(|| CpsError::UnboundVariable(cont.clone()))?;
    let answer = invoke_cont(&cont_value, &arg_value, runtime)?;
    let new_env = env.clone().with_binding(name.clone(), answer);
    eval_unchecked_with_runtime(body, &new_env, chain, runtime)
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
) -> CpsResult<Value> {
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
) -> CpsResult<Value> {
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
        Value::ThunkClosure {
            mode,
            body,
            captured_env,
            captured_chain,
            row,
            memo_cell,
        } => Value::ThunkClosure {
            mode: *mode,
            body: Box::new(mark_rec_binding(body.as_ref(), rec_name)),
            captured_env: captured_env.clone(),
            captured_chain: captured_chain.clone(),
            row: row.clone(),
            memo_cell: *memo_cell,
        },
        Value::Tuple { elems } => {
            let mut new_elems = Vec::new();
            for elem in elems {
                new_elems.push(mark_rec_binding(elem, rec_name));
            }
            Value::Tuple { elems: new_elems }
        }
        Value::Constructor { name, fields } => Value::Constructor {
            name: name.clone(),
            fields: fields
                .iter()
                .map(|(field_name, field_value)| {
                    (field_name.clone(), mark_rec_binding(field_value, rec_name))
                })
                .collect(),
        },
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
) -> CpsResult<Value> {
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
            let marked_record = Value::Record { fields: new_fields };
            let mut recursive_env = new_env.clone();
            recursive_env = recursive_env.with_binding(name.clone(), marked_record.clone());
            eval_value_with_runtime(&marked_record, &recursive_env, chain, runtime)?
        }
        Value::Tuple { elems } => {
            let mut new_elems = Vec::new();
            for elem in elems {
                new_elems.push(mark_rec_binding(elem, name));
            }
            let marked_tuple = Value::Tuple { elems: new_elems };
            let mut recursive_env = new_env.clone();
            recursive_env = recursive_env.with_binding(name.clone(), marked_tuple.clone());
            eval_value_with_runtime(&marked_tuple, &recursive_env, chain, runtime)?
        }
        Value::Constructor {
            name: constructor,
            fields,
        } => {
            let marked_constructor = Value::Constructor {
                name: constructor.clone(),
                fields: fields
                    .iter()
                    .map(|(field_name, field_value)| {
                        (field_name.clone(), mark_rec_binding(field_value, name))
                    })
                    .collect(),
            };
            let recursive_env = new_env
                .clone()
                .with_binding(name.clone(), marked_constructor.clone());
            eval_value_with_runtime(&marked_constructor, &recursive_env, chain, runtime)?
        }
        Value::ThunkClosure {
            mode,
            body,
            captured_env,
            captured_chain,
            row,
            memo_cell,
        } => {
            let memo_cell = match (mode, memo_cell) {
                (ThunkMode::Memo, None) => Some(runtime.allocate_memo_cell()),
                (_, memo_cell) => *memo_cell,
            };

            let marked_body = mark_rec_binding(body.as_ref(), name);
            let marked_body = Box::new(marked_body);

            let placeholder = Value::ThunkClosure {
                mode: *mode,
                body: marked_body.clone(),
                captured_env: captured_env.clone(),
                captured_chain: captured_chain.clone(),
                row: row.clone(),
                memo_cell,
            };

            let mut recursive_env = new_env.clone();
            recursive_env = recursive_env.with_binding(name.clone(), placeholder);

            let marked_thunk = Value::ThunkClosure {
                mode: *mode,
                body: marked_body,
                captured_env: captured_env.clone(),
                captured_chain: captured_chain.clone(),
                row: row.clone(),
                memo_cell,
            };

            eval_value_with_runtime(&marked_thunk, &recursive_env, chain, runtime)?
        }
        other => other.clone(),
    };
    new_env = new_env.with_binding(name.clone(), lam_value);
    eval_unchecked_with_runtime(body, &new_env, chain, runtime)
}

/// Resolve the dynamic resume continuation row and multiplicity from the
/// handler clause metadata and the `Raise.resume` target.
///
/// This is the runtime fail-closed check required by SPEC-102 §5/§7.
///
/// - For `ResumeRowMetadata::Known(known_row)`: compare the known row with the
///   resolved target row (from the `Raise.resume` continuation). If the target
///   row cannot be resolved or differs from the known row, fail closed.
/// - For `ResumeRowMetadata::InheritFromTarget`: derive the affine resume row
///   from the resolved target row. This path is valid only for affine resumes;
///   multi-shot-pure resumes require a known row.
#[allow(clippy::result_large_err)]
fn resolve_resume_metadata(
    clause: &HandlerClause,
    resume: &ContRef,
    env: &Env,
) -> CpsResult<(EffectRow, ContMultiplicity)> {
    // The resolved target row comes from the continuation value already bound
    // for the Raise.resume target.
    let resolved_target_row = resolve_resume_target_row(resume, env);

    match &clause.resume_row {
        ResumeRowMetadata::Known(known_row) => {
            // Compare the known row with the resolved target row.
            match resolved_target_row {
                Some(target_row) => {
                    if &target_row != known_row {
                        return Err(CpsError::Trap(TrapReason::Custom(format!(
                            "resume row mismatch: handler clause declares {known_row:?}, \
                             target requires {target_row:?}"
                        ))));
                    }
                    Ok((target_row, clause.resume_multiplicity))
                }
                None => Err(CpsError::Trap(TrapReason::Custom(
                    "resume row mismatch: handler clause declares known row but target \
                     row cannot be resolved"
                        .to_string(),
                ))),
            }
        }
        ResumeRowMetadata::InheritFromTarget => {
            // Derive the affine resume row from the resolved target row.
            // Multi-shot-pure resumes require a known row.
            if clause.resume_multiplicity == ContMultiplicity::MultiShotPure {
                return Err(CpsError::Trap(TrapReason::Custom(
                    "multi-shot-pure resume requires a known row; inherited target row is not valid"
                        .to_string(),
                )));
            }
            let row = resolved_target_row.unwrap_or_default();
            Ok((row, clause.resume_multiplicity))
        }
    }
}

/// Best-effort resolution of the resume target row from a `ContRef`.
///
/// The target continuation's declared row is the static resume row to compare
/// with checked handler metadata. A missing or non-continuation target remains
/// unresolved and therefore fails closed for known rows.
fn resolve_resume_target_row(resume: &ContRef, env: &Env) -> Option<EffectRow> {
    match resolve_cont(resume, env).ok()? {
        Value::Cont { row, .. } => Some(row),
        _ => None,
    }
}

#[allow(clippy::result_large_err)]
fn eval_raise(
    op: &EffectOp,
    args: &[Atom],
    resume: &ContRef,
    env: &Env,
    chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Value> {
    match chain.find_operation_frame(op) {
        Some(HandlerFrameMatch::Shallow {
            clause,
            frame_index,
        })
        | Some(HandlerFrameMatch::Deep {
            clause,
            frame_index,
        }) => {
            let arg_values: CpsResult<Vec<Atom>> = args.iter().map(|a| eval_atom(a, env)).collect();
            let arg_values = arg_values?;
            // Remove the matched handler from the chain before evaluating its clause body.
            let mut body_chain = chain.clone();
            body_chain.frames.remove(frame_index);
            // A deep frame restores itself around its captured continuation;
            // a shallow frame keeps the historical one-shot scope.
            let resume_chain = match chain.frames.get(frame_index) {
                Some(HandlerFrame::Deep { .. }) => chain.clone(),
                Some(HandlerFrame::Shallow { .. }) => body_chain.clone(),
                Some(HandlerFrame::Provider { .. }) | None => {
                    return Err(CpsError::Trap(TrapReason::Custom(
                        "handler frame lookup changed during dispatch".to_string(),
                    )));
                }
            };

            // Resolve the dynamic resume row and copy multiplicity from the clause.
            // This is the runtime fail-closed check required by SPEC-102 §5/§7.
            let (resume_row, resume_multiplicity) = resolve_resume_metadata(clause, resume, env)?;

            let resume_cont = Value::Cont {
                param: clause.resume.clone(),
                body: Box::new(Term::Jump {
                    cont: resume.clone(),
                    arg: Atom::Var(clause.resume.clone()),
                    row: resume_row.clone(),
                }),
                captured_env: env.clone(),
                captured_chain: resume_chain,
                consumed: ConsumedFlag::new(),
                row: resume_row,
                multiplicity: resume_multiplicity,
            };
            let mut new_env = env.clone();
            for (param, arg) in clause.params.iter().zip(arg_values.iter()) {
                new_env = new_env.with_binding(param.clone(), Value::Atom(arg.clone()));
            }
            new_env = new_env.with_binding(clause.resume.clone(), resume_cont);
            eval_unchecked_with_runtime(&clause.body, &new_env, &body_chain, runtime)
        }
        Some(HandlerFrameMatch::Provider { handler, .. }) => {
            // Provider dispatch: invoke the provider handler directly.
            let handler_value = env
                .lookup(handler)
                .ok_or_else(|| CpsError::UnboundVariable(handler.to_string()))?
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
        }
        None => Err(CpsError::UnhandledEffect(op.clone())),
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
) -> CpsResult<Value> {
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
        Value::Atom(Atom::Var(name)) => env
            .lookup(name)
            .cloned()
            .ok_or_else(|| CpsError::UnboundVariable(name.clone())),
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
        Value::Constructor { name, fields } => {
            let mut new_fields = Vec::new();
            for (field_name, field_value) in fields {
                new_fields.push((
                    field_name.clone(),
                    eval_value_with_runtime(field_value, env, chain, runtime)?,
                ));
            }
            Ok(Value::Constructor {
                name: name.clone(),
                fields: new_fields,
            })
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

/// Preserves the historical atom-only evaluator API for callers that do not
/// use the canonical terminal-value projection.
#[allow(clippy::result_large_err)]
#[cfg(test)]
fn value_to_atom(value: Value) -> CpsResult<Atom> {
    match value {
        Value::Atom(atom) => Ok(atom),
        value => Err(CpsError::ExpectedAtomicResult(value)),
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
) -> CpsResult<Value> {
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
                        Err(CpsError::Trap(TrapReason::Custom(
                            "division by zero".to_string(),
                        )))
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
fn force_thunk_value(thunk: &Value, runtime: &mut CpsRuntime) -> CpsResult<Value> {
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
                                outcome: trace_outcome_string::<Value>(&Ok(atom.clone())),
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
fn run_thunk_body_with_runtime(thunk: &Value, runtime: &mut CpsRuntime) -> CpsResult<Value> {
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
            value: Value::Atom(Atom::Var(force_result_name.to_string())),
        }),
        captured_env: Env::new(),
        captured_chain: captured_chain.clone(),
        consumed: ConsumedFlag::new(),
        row: EffectRow::default(),
        multiplicity: ContMultiplicity::Affine,
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
                value: Value::Atom(Atom::Var("result".to_string())),
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
            row: EffectRow::default(),
            multiplicity: ContMultiplicity::Affine,
        };

        eval_unchecked_with_runtime(&body, &env, &force_chain, run_time).and_then(value_to_atom)
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

    #[test]
    fn letrec_memo_thunk_allocates_memo_cell_before_binding() {
        let op = captured_effect_op();
        let env = Env::new().with_binding("provider".to_string(), provider_handler(41));
        let mut chain_with_provider = HandlerChain::new();
        chain_with_provider.push(HandlerFrame::Provider {
            op: op.clone(),
            handler: "provider".to_string(),
        });

        let letrec_term = Term::LetRec {
            name: "memoized".to_string(),
            value: Value::ThunkClosure {
                mode: ThunkMode::Memo,
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
            },
            body: Box::new(Term::LetPrim {
                name: "forced".to_string(),
                op: PrimOp::ForceThunk,
                args: vec![Atom::Var("memoized".to_string())],
                body: Box::new(Term::Return {
                    value: Value::Atom(Atom::Var("forced".to_string())),
                }),
            }),
        };

        let mut runtime = CpsRuntime::new();
        let result =
            eval_unchecked_with_runtime(&letrec_term, &env, &chain_with_provider, &mut runtime);
        assert_eq!(result, Ok(Value::Atom(Atom::Int(41))));
    }

    #[test]
    fn recursive_memo_force_rejects_reentrant_force() {
        let letrec_term = Term::LetRec {
            name: "memoized".to_string(),
            value: Value::ThunkClosure {
                mode: ThunkMode::Memo,
                body: Box::new(Value::Lam {
                    params: vec![],
                    cont: "resume".to_string(),
                    body: Box::new(Term::LetPrim {
                        name: "self_forced".to_string(),
                        op: PrimOp::ForceThunk,
                        args: vec![Atom::Var("memoized".to_string())],
                        body: Box::new(Term::Return {
                            value: Value::Atom(Atom::Int(7)),
                        }),
                    }),
                    captured_env: Env::default(),
                    rec_binding: None,
                    row: EffectRow::default(),
                }),
                captured_env: Env::default(),
                captured_chain: HandlerChain::new(),
                row: EffectRow::default(),
                memo_cell: None,
            },
            body: Box::new(Term::LetPrim {
                name: "forced".to_string(),
                op: PrimOp::ForceThunk,
                args: vec![Atom::Var("memoized".to_string())],
                body: Box::new(Term::Return {
                    value: Value::Atom(Atom::Int(0)),
                }),
            }),
        };

        let mut runtime = CpsRuntime::new();
        let result = eval_unchecked_with_runtime(
            &letrec_term,
            &Env::default(),
            &HandlerChain::default(),
            &mut runtime,
        );
        assert_eq!(
            result,
            Err(CpsError::Trap(TrapReason::Custom(
                "re-entrant memo force".to_string()
            )))
        );
    }

    #[test]
    fn composite_recursive_memo_thunk_preserves_enclosing_binding() {
        let letrec_term = Term::LetRec {
            name: "pair".to_string(),
            value: Value::Tuple {
                elems: vec![
                    Value::ThunkClosure {
                        mode: ThunkMode::Memo,
                        body: Box::new(Value::Lam {
                            params: vec![],
                            cont: "resume".to_string(),
                            body: Box::new(Term::LetPrim {
                                name: "pair_second".to_string(),
                                op: PrimOp::TupleGet(1),
                                args: vec![Atom::Var("pair".to_string())],
                                body: Box::new(Term::Return {
                                    value: Value::Atom(Atom::Var("pair_second".to_string())),
                                }),
                            }),
                            captured_env: Env::new(),
                            rec_binding: None,
                            row: EffectRow::default(),
                        }),
                        captured_env: Env::default(),
                        captured_chain: HandlerChain::new(),
                        row: EffectRow::default(),
                        memo_cell: None,
                    },
                    Value::Atom(Atom::Int(9)),
                ],
            },
            body: Box::new(Term::LetPrim {
                name: "pair0".to_string(),
                op: PrimOp::TupleGet(0),
                args: vec![Atom::Var("pair".to_string())],
                body: Box::new(Term::LetPrim {
                    name: "forced".to_string(),
                    op: PrimOp::ForceThunk,
                    args: vec![Atom::Var("pair0".to_string())],
                    body: Box::new(Term::Return {
                        value: Value::Atom(Atom::Var("forced".to_string())),
                    }),
                }),
            }),
        };

        let mut runtime = CpsRuntime::new();
        let result = eval_unchecked_with_runtime(
            &letrec_term,
            &Env::default(),
            &HandlerChain::default(),
            &mut runtime,
        );
        assert_eq!(result, Ok(Value::Atom(Atom::Int(9))));
    }
}

#[cfg(test)]
mod migrated_regression_tests;
