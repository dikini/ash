//! CPS IR Interpreter
//!
//! Evaluates CPS IR terms in the Ash language.

use ash_core::cps::*;
use std::cell::RefCell;
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
    #[error("invalid primitive arguments for {0:?}: {1:?}")]
    InvalidPrimArgs(PrimOp, Vec<Atom>),
    #[error("unhandled effect: {0:?}")]
    UnhandledEffect(EffectOp),
    #[error("trap: {0:?}")]
    Trap(TrapReason),
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
    match term {
        Term::LetVal { name, value, body } => eval_letval(name, value, body, env, chain),
        Term::LetPrim {
            name,
            op,
            args,
            body,
        } => eval_letprim(name, op, args, body, env, chain),
        Term::LetCont {
            name,
            param,
            cont_body,
            body,
        } => eval_letcont(name, param, cont_body, body, env, chain),
        Term::Jump { cont, arg, .. } => eval_jump(cont, arg, env, chain),
        Term::Call {
            func, args, cont, ..
        } => eval_call(func, args, cont, env, chain),
        Term::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => eval_if(cond, then_branch, else_branch, env, chain),
        Term::LetRec { name, value, body } => eval_letrec(name, value, body, env, chain),
        Term::Match {
            scrutinee,
            arms,
            default,
        } => eval_match(scrutinee, arms, default.as_deref(), env, chain),
        Term::Raise {
            op, args, resume, ..
        } => eval_raise(op, args, resume, env, chain),
        Term::Handle {
            clause, body, cont, ..
        } => eval_handle(clause, body, cont, env, chain),
        Term::RecordDischarge { body, .. } => eval_unchecked(body, env, chain),
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
    Ok(eval_unchecked(term, env, chain)?)
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
) -> CpsResult<Atom> {
    let evaluated_value = eval_value(value, env)?;
    let new_env = env.clone().with_binding(name.clone(), evaluated_value);
    eval_unchecked(body, &new_env, chain)
}

#[allow(clippy::result_large_err)]
fn eval_letprim(
    name: &Name,
    op: &PrimOp,
    args: &[Atom],
    body: &Term,
    env: &Env,
    chain: &HandlerChain,
) -> CpsResult<Atom> {
    let resolved_args: CpsResult<Vec<Value>> =
        args.iter().map(|a| eval_atom_to_value(a, env)).collect();
    let result = eval_prim(op, &resolved_args?, env)?;
    let new_env = env.clone().with_binding(name.clone(), result);
    eval_unchecked(body, &new_env, chain)
}

#[allow(clippy::result_large_err)]
fn eval_letcont(
    name: &Name,
    param: &Name,
    cont_body: &Term,
    body: &Term,
    env: &Env,
    chain: &HandlerChain,
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
    eval_unchecked(body, &new_env, chain)
}

#[allow(clippy::result_large_err)]
fn eval_jump(cont: &ContRef, arg: &Atom, env: &Env, _chain: &HandlerChain) -> CpsResult<Atom> {
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
            eval_unchecked(&body, &new_env, &captured_chain)
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
            if let Some(rec_name) = rec_binding {
                if let Some(rec_value) = env.lookup(&rec_name) {
                    new_env = new_env.with_binding(rec_name, rec_value.clone());
                }
            }
            for (param, arg) in params.iter().zip(arg_values.iter()) {
                new_env = new_env.with_binding(param.clone(), arg.clone());
            }
            new_env = new_env.with_binding(lam_cont.clone(), cont_value);
            eval_unchecked(&body, &new_env, chain)
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
) -> CpsResult<Atom> {
    let cond_value = eval_atom(cond, env)?;
    match cond_value {
        Atom::Bool(true) => eval_unchecked(then_branch, env, chain),
        Atom::Bool(false) => eval_unchecked(else_branch, env, chain),
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
    eval_unchecked(body, &new_env, chain)
}

#[allow(clippy::result_large_err)]
fn eval_raise(
    op: &EffectOp,
    args: &[Atom],
    resume: &ContRef,
    env: &Env,
    chain: &HandlerChain,
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
        eval_unchecked(&clause.body, &new_env, &body_chain)
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
                let resume_value = resolve_cont(resume, env)?;
                let mut new_env = captured_env.clone();
                if let Some(rec_name) = rec_binding {
                    if let Some(rec_value) = env.lookup(&rec_name) {
                        new_env = new_env.with_binding(rec_name, rec_value.clone());
                    }
                }
                for (param, arg) in params.iter().zip(arg_values.iter()) {
                    new_env = new_env.with_binding(param.clone(), Value::Atom(arg.clone()));
                }
                new_env = new_env.with_binding(lam_cont.clone(), resume_value);
                eval_unchecked(&body, &new_env, chain)
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
) -> CpsResult<Atom> {
    let cont_value = resolve_cont(cont, env)?;
    let mut new_chain = chain.clone();
    new_chain.push(HandlerFrame::Shallow {
        clause: clause.clone(),
    });
    let mut new_env = env.clone();
    new_env = new_env.with_binding(clause.resume.clone(), cont_value);
    eval_unchecked(body, &new_env, &new_chain)
}

/// Evaluate a value (atoms pass through, lambdas capture env if not already captured, conts are inert)
#[allow(clippy::result_large_err)]
fn eval_value(value: &Value, env: &Env) -> CpsResult<Value> {
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
        Value::Record { fields } => {
            let mut new_fields = Vec::new();
            for (name, field_value) in fields {
                new_fields.push((name.clone(), eval_value(field_value, env)?));
            }
            Ok(Value::Record { fields: new_fields })
        }
        Value::Tuple { elems } => {
            let mut new_elems = Vec::new();
            for elem in elems {
                new_elems.push(eval_value(elem, env)?);
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
                            return eval_unchecked(body, env, chain);
                        }
                    }
                    if let Some(default_body) = default {
                        return eval_unchecked(default_body, env, chain);
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
fn eval_prim(op: &PrimOp, args: &[Value], _env: &Env) -> CpsResult<Value> {
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
    }
}
