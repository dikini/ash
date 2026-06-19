//! CPS IR Interpreter
//!
//! Evaluates CPS IR terms in the Ash language.

use ash_core::cps::*;
use std::cell::RefCell;
use std::rc::Rc;
use thiserror::Error;

pub mod validate;

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

/// Evaluate a CPS term in an environment with a handler chain
pub fn eval_term(term: &Term, env: &Env, chain: &HandlerChain) -> CpsResult<Atom> {
    match term {
        Term::LetVal { name, value, body } => eval_letval(name, value, body, env, chain),
        Term::LetPrim {
            name,
            op,
            args,
            body,
        } => eval_letprim(name, *op, args, body, env, chain),
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
        Term::Raise {
            op, args, resume, ..
        } => eval_raise(op, args, resume, env, chain),
        Term::Handle {
            clause, body, cont, ..
        } => eval_handle(clause, body, cont, env, chain),
        Term::RecordDischarge { body, .. } => eval_term(body, env, chain),
        Term::Return { value } => Ok(eval_atom(value, env)?),
        Term::Trap { reason } => Err(CpsError::Trap(reason.clone())),
    }
}

// ---------------------------------------------------------------------------
// Per-term evaluators
// ---------------------------------------------------------------------------

fn eval_letval(
    name: &Name,
    value: &Value,
    body: &Term,
    env: &Env,
    chain: &HandlerChain,
) -> CpsResult<Atom> {
    let evaluated_value = eval_value(value, env)?;
    let new_env = env.clone().with_binding(name.clone(), evaluated_value);
    eval_term(body, &new_env, chain)
}

fn eval_letprim(
    name: &Name,
    op: PrimOp,
    args: &[Atom],
    body: &Term,
    env: &Env,
    chain: &HandlerChain,
) -> CpsResult<Atom> {
    let resolved_args: CpsResult<Vec<Atom>> = args.iter().map(|a| eval_atom(a, env)).collect();
    let result = eval_prim(op, &resolved_args?)?;
    let new_env = env.clone().with_binding(name.clone(), Value::Atom(result));
    eval_term(body, &new_env, chain)
}

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
    eval_term(body, &new_env, chain)
}

fn eval_jump(cont: &ContRef, arg: &Atom, env: &Env, _chain: &HandlerChain) -> CpsResult<Atom> {
    let arg_value = eval_atom(arg, env)?;
    let cont_value = resolve_cont(cont, env)?;
    match cont_value {
        Value::Cont {
            mut param,
            body,
            captured_env,
            captured_chain,
            mut consumed,
            ..
        } => {
            if consumed.get() {
                return Err(CpsError::Trap(TrapReason::Custom(
                    "resume already consumed".to_string(),
                )));
            }
            consumed.set(true);
            let new_env = captured_env
                .clone()
                .with_binding(param, Value::Atom(arg_value));
            eval_term(&body, &new_env, &captured_chain)
        }
        _ => Err(CpsError::ExpectedContinuation(cont_value)),
    }
}

fn eval_call(
    func: &Atom,
    args: &[Atom],
    cont: &ContRef,
    env: &Env,
    chain: &HandlerChain,
) -> CpsResult<Atom> {
    let func_value = resolve_value(func, env)?;
    let arg_values: CpsResult<Vec<Atom>> = args.iter().map(|a| eval_atom(a, env)).collect();
    let arg_values = arg_values?;
    let cont_value = resolve_cont(cont, env)?;
    match func_value {
        Value::Lam {
            params,
            cont: lam_cont,
            body,
            captured_env,
            ..
        } => {
            // Start from the lambda's captured definition environment
            // merged with the call-site environment for recursive references
            let mut new_env = captured_env.clone();
            // Overlay call-site bindings (for updated recursive references)
            for (key, value) in &env.bindings {
                new_env = new_env.with_binding(key.clone(), value.clone());
            }
            for (param, arg) in params.iter().zip(arg_values.iter()) {
                new_env = new_env.with_binding(param.clone(), Value::Atom(arg.clone()));
            }
            // Always bind the continuation parameter
            new_env = new_env.with_binding(lam_cont.clone(), cont_value);
            eval_term(&body, &new_env, chain)
        }
        _ => Err(CpsError::ExpectedLambda(func_value)),
    }
}

fn eval_if(
    cond: &Atom,
    then_branch: &Term,
    else_branch: &Term,
    env: &Env,
    chain: &HandlerChain,
) -> CpsResult<Atom> {
    let cond_value = eval_atom(cond, env)?;
    match cond_value {
        Atom::Bool(true) => eval_term(then_branch, env, chain),
        Atom::Bool(false) => eval_term(else_branch, env, chain),
        _ => Err(CpsError::InvalidPrimArgs(PrimOp::Eq, vec![cond_value])),
    }
}

fn eval_letrec(
    name: &Name,
    value: &Value,
    body: &Term,
    env: &Env,
    chain: &HandlerChain,
) -> CpsResult<Atom> {
    let mut new_env = env.clone();
    // Step 1: Bind placeholder
    new_env = new_env.with_binding(name.clone(), Value::Atom(Atom::Null));
    // Step 2: Construct lambda with placeholder env
    let lam_value = match value {
        Value::Lam {
            params,
            cont,
            body: lam_body,
            ..
        } => Value::Lam {
            params: params.clone(),
            cont: cont.clone(),
            body: lam_body.clone(),
            captured_env: new_env.clone(),
            row: EffectRow::default(),
        },
        other => other.clone(),
    };
    // Step 3: Backfill with the lambda
    new_env = new_env.with_binding(name.clone(), lam_value.clone());
    // Step 4: Create a NEW lambda with the backfilled env
    let final_value = match &lam_value {
        Value::Lam {
            params,
            cont,
            body: lam_body,
            ..
        } => Value::Lam {
            params: params.clone(),
            cont: cont.clone(),
            body: lam_body.clone(),
            captured_env: new_env.clone(),
            row: EffectRow::default(),
        },
        other => other.clone(),
    };
    // Step 5: Re-bind with the final lambda
    new_env = new_env.with_binding(name.clone(), final_value);
    eval_term(body, &new_env, chain)
}

fn eval_raise(
    op: &EffectOp,
    args: &[Atom],
    resume: &ContRef,
    env: &Env,
    chain: &HandlerChain,
) -> CpsResult<Atom> {
    match chain.find_handler(op) {
        Some(clause) => {
            let arg_values: CpsResult<Vec<Atom>> = args.iter().map(|a| eval_atom(a, env)).collect();
            let arg_values = arg_values?;
            // Build resume continuation that captures current env and chain WITHOUT the handler
            let mut resume_chain = chain.clone();
            // Remove the shallow handler that matched
            if let Some(idx) = resume_chain.frames.iter().rposition(
                |f| matches!(f, HandlerFrame::Shallow { clause: c } if c.op.item == op.item),
            ) {
                resume_chain.frames.remove(idx);
            }
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
            eval_term(&clause.body, &new_env, chain)
        }
        None => Err(CpsError::UnhandledEffect(op.clone())),
    }
}

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
    eval_term(body, &new_env, &new_chain)
}

/// Evaluate a value (atoms pass through, lambdas capture env if not already captured, conts are inert)
fn eval_value(value: &Value, env: &Env) -> CpsResult<Value> {
    match value {
        Value::Atom(atom) => Ok(Value::Atom(eval_atom(atom, env)?)),
        Value::Lam {
            params,
            cont,
            body,
            captured_env,
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
                row: row.clone(),
            })
        }
        other => Ok(other.clone()),
    }
}

/// Evaluate an atom (resolve variables)
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
fn resolve_value(atom: &Atom, env: &Env) -> CpsResult<Value> {
    match atom {
        Atom::Var(name) => env
            .lookup(name)
            .ok_or_else(|| CpsError::UnboundVariable(name.clone()))
            .cloned(),
        other => Ok(Value::Atom(other.clone())),
    }
}

/// Evaluate a primitive operation
fn eval_prim(op: PrimOp, args: &[Atom]) -> CpsResult<Atom> {
    let make_err = || CpsError::InvalidPrimArgs(op, args.to_vec());
    match op {
        PrimOp::Add => {
            let a = args.first().ok_or_else(make_err)?;
            let b = args.get(1).ok_or_else(make_err)?;
            match (a, b) {
                (Atom::Int(x), Atom::Int(y)) => Ok(Atom::Int(x + y)),
                _ => Err(make_err()),
            }
        }
        PrimOp::Sub => {
            let a = args.first().ok_or_else(make_err)?;
            let b = args.get(1).ok_or_else(make_err)?;
            match (a, b) {
                (Atom::Int(x), Atom::Int(y)) => Ok(Atom::Int(x - y)),
                _ => Err(make_err()),
            }
        }
        PrimOp::Mul => {
            let a = args.first().ok_or_else(make_err)?;
            let b = args.get(1).ok_or_else(make_err)?;
            match (a, b) {
                (Atom::Int(x), Atom::Int(y)) => Ok(Atom::Int(x * y)),
                _ => Err(make_err()),
            }
        }
        PrimOp::Div => {
            let a = args.first().ok_or_else(make_err)?;
            let b = args.get(1).ok_or_else(make_err)?;
            match (a, b) {
                (Atom::Int(x), Atom::Int(y)) => {
                    if *y == 0 {
                        Err(make_err())
                    } else {
                        Ok(Atom::Int(x / y))
                    }
                }
                _ => Err(make_err()),
            }
        }
        PrimOp::Eq => {
            let a = args.first().ok_or_else(make_err)?;
            let b = args.get(1).ok_or_else(make_err)?;
            Ok(Atom::Bool(a == b))
        }
        PrimOp::Ne => {
            let a = args.first().ok_or_else(make_err)?;
            let b = args.get(1).ok_or_else(make_err)?;
            Ok(Atom::Bool(a != b))
        }
        PrimOp::Lt => {
            let a = args.first().ok_or_else(make_err)?;
            let b = args.get(1).ok_or_else(make_err)?;
            match (a, b) {
                (Atom::Int(x), Atom::Int(y)) => Ok(Atom::Bool(x < y)),
                _ => Err(make_err()),
            }
        }
        PrimOp::Le => {
            let a = args.first().ok_or_else(make_err)?;
            let b = args.get(1).ok_or_else(make_err)?;
            match (a, b) {
                (Atom::Int(x), Atom::Int(y)) => Ok(Atom::Bool(x <= y)),
                _ => Err(make_err()),
            }
        }
        PrimOp::Gt => {
            let a = args.first().ok_or_else(make_err)?;
            let b = args.get(1).ok_or_else(make_err)?;
            match (a, b) {
                (Atom::Int(x), Atom::Int(y)) => Ok(Atom::Bool(x > y)),
                _ => Err(make_err()),
            }
        }
        PrimOp::Ge => {
            let a = args.first().ok_or_else(make_err)?;
            let b = args.get(1).ok_or_else(make_err)?;
            match (a, b) {
                (Atom::Int(x), Atom::Int(y)) => Ok(Atom::Bool(x >= y)),
                _ => Err(make_err()),
            }
        }
        PrimOp::Neg => {
            let a = args.first().ok_or_else(make_err)?;
            match a {
                Atom::Int(x) => Ok(Atom::Int(-x)),
                _ => Err(make_err()),
            }
        }
        PrimOp::Not => {
            let a = args.first().ok_or_else(make_err)?;
            match a {
                Atom::Bool(x) => Ok(Atom::Bool(!x)),
                _ => Err(make_err()),
            }
        }
    }
}
