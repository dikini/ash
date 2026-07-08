---
id: ref.runtime.cps-interpreter
title: CPS IR Interpreter
kind: reference
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: runtime
last_verified: 2026-06-20
verified_against:
  git_commit: b7d6137f
  specs:
    - docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md
    - docs/plan/PLAN-159-CPS-IR-INTERPRETER.md
  tasks:
    - docs/plan/tasks/TASK-1591-cps-ir-core-evaluator.md
    - docs/plan/tasks/TASK-1593-cps-ir-raise-handle-dispatch.md
    - docs/plan/tasks/TASK-1966-docs-reference-historical-quarantine.md
  code:
    - crates/ash-interp/src/cps/mod.rs
  tests:
    - crates/ash-interp/tests/task_1591_cps_ir.rs
    - crates/ash-interp/tests/task_1592_cps_ir.rs
    - crates/ash-interp/tests/task_1593_cps_ir.rs
    - crates/ash-interp/tests/task_1594_cps_ir.rs
    - crates/ash-interp/tests/task_1595_cps_ir.rs
    - crates/ash-interp/tests/task_1596_cps_ir.rs
  examples:
    - crates/ash-interp/tests/task_1596_cps_ir.rs
refresh_trigger:
  - crates/ash-interp/src/cps/mod.rs changes
  - docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md changes
related:
  depends_on:
    - ref.language.cps-ir
    - ref.language.cps-operational-semantics
  explains: []
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md
---

# CPS IR Interpreter

## Summary

The CPS IR interpreter evaluates Ash CPS IR terms in a direct big-step semantics. It is the reference execution engine for the isolated prototype: all CPS IR terms are interpreted, not compiled to bytecode or JITed. This keeps the execution path simple, readable, and auditable.

The interpreter is in `crates/ash-interp/src/cps/mod.rs` and evaluates terms against an immutable environment and an explicit handler chain.

## Entry point

```rust
pub fn eval_term(term: &Term, env: &Env, chain: &HandlerChain) -> CpsResult<Atom>
```

- `term`: the CPS IR term to evaluate
- `env`: the current runtime environment (immutable frame stack)
- `chain`: the current handler chain (explicit frame stack)
- Returns: the final atom value, or a `CpsError`

## Architecture

The interpreter is structured as a thin dispatcher with per-term evaluators:

```rust
pub fn eval_term(term: &Term, env: &Env, chain: &HandlerChain) -> CpsResult<Atom> {
    match term {
        Term::LetVal { .. } => eval_letval(...),
        Term::LetPrim { .. } => eval_letprim(...),
        Term::LetCont { .. } => eval_letcont(...),
        Term::Jump { .. } => eval_jump(...),
        Term::Call { .. } => eval_call(...),
        Term::If { .. } => eval_if(...),
        Term::LetRec { .. } => eval_letrec(...),
        Term::Raise { .. } => eval_raise(...),
        Term::Handle { .. } => eval_handle(...),
        Term::RecordDischarge { .. } => eval_term(body, env, chain),
        Term::Trap { reason } => Err(CpsError::Trap(reason.clone())),
    }
}
```

Each `eval_*` function handles one term variant. This makes the code easy to read and easy to split into submodules if the interpreter grows.

## Per-term evaluation

### LetVal

Evaluate the value (resolving variables in atoms), bind it in the environment, and continue with the body.

```rust
fn eval_letval(name, value, body, env, chain) {
    let evaluated = eval_value(value, env)?;
    let new_env = env.clone().with_binding(name.clone(), evaluated);
    eval_term(body, &new_env, chain)
}
```

### LetPrim

Evaluate arguments (resolving variables), apply the primitive operation, bind the result, and continue.

```rust
fn eval_letprim(name, op, args, body, env, chain) {
    let resolved: Vec<Atom> = args.iter().map(|a| eval_atom(a, env)).collect()?;
    let result = eval_prim(op, &resolved)?;
    let new_env = env.clone().with_binding(name.clone(), Value::Atom(result));
    eval_term(body, &new_env, chain)
}
```

### LetCont

Create a continuation closure capturing the current environment, bind it, and continue.

```rust
fn eval_letcont(name, param, cont_body, body, env, chain) {
    let cont = Value::Cont {
        param: param.clone(),
        body: Box::new(cont_body.clone()),
        captured_env: env.clone(),
        row: EffectRow::default(),
    };
    let new_env = env.clone().with_binding(name.clone(), cont);
    eval_term(body, &new_env, chain)
}
```

The `captured_env` is essential: when the continuation is later invoked via `Jump`, it runs in this captured environment, not the environment at the jump site. This preserves lexical scoping.

### Jump

Evaluate the argument, resolve the continuation, and execute the continuation body in the captured environment.

```rust
fn eval_jump(cont, arg, env, chain) {
    let arg_value = eval_atom(arg, env)?;
    let cont_value = resolve_cont(cont, env)?;
    match cont_value {
        Value::Cont { param, body, captured_env, .. } => {
            let new_env = captured_env.clone().with_binding(param, Value::Atom(arg_value));
            eval_term(&body, &new_env, chain)
        }
        _ => Err(CpsError::ExpectedContinuation(cont_value)),
    }
}
```

### Call

Evaluate the function and arguments, bind parameters in a new environment, and execute the lambda body.

```rust
fn eval_call(func, args, cont, env, chain) {
    let func_value = resolve_value(func, env)?;
    let arg_values: Vec<Atom> = args.iter().map(|a| eval_atom(a, env)).collect()?;
    let cont_value = resolve_cont(cont, env)?;
    match func_value {
        Value::Lam { params, cont: lam_cont, body, .. } => {
            let mut new_env = env.clone();
            for (param, arg) in params.iter().zip(arg_values.iter()) {
                new_env = new_env.with_binding(param.clone(), Value::Atom(arg.clone()));
            }
            // Only bind continuation parameter if not already present
            // to preserve outer continuation references in nested continuations
            if !new_env.bindings.contains_key(&lam_cont) {
                new_env = new_env.with_binding(lam_cont.clone(), cont_value);
            }
            eval_term(&body, &new_env, chain)
        }
        _ => Err(CpsError::ExpectedLambda(func_value)),
    }
}
```

The non-overwriting binding rule is critical for recursive functions with nested continuations. When a recursive call overwrites the lambda's continuation parameter `k`, any nested continuation that references `k` must still find the outer `k`, not the recursive call's `k`.

### If

Evaluate the condition and choose the branch.

```rust
fn eval_if(cond, then_branch, else_branch, env, chain) {
    match eval_atom(cond, env)? {
        Atom::Bool(true) => eval_term(then_branch, env, chain),
        Atom::Bool(false) => eval_term(else_branch, env, chain),
        other => Err(CpsError::InvalidPrimArgs(PrimOp::Eq, vec![other])),
    }
}
```

### LetRec

Bind the recursive name to a placeholder, evaluate the value in the environment with the placeholder, then backfill.

```rust
fn eval_letrec(name, value, body, env, chain) {
    let mut new_env = env.clone();
    new_env = new_env.with_binding(name.clone(), Value::Atom(Atom::Null));
    let evaluated = eval_value(value, &new_env)?;
    new_env = new_env.with_binding(name.clone(), evaluated);
    eval_term(body, &new_env, chain)
}
```

This is a standard placeholder/backfill pattern. The lambda body references the recursive name, which is initially `Null` but gets backfilled with the actual lambda before the body is executed.

### Raise

Find the innermost handler for the effect, bind parameters and resume continuation, and execute the handler body.

```rust
fn eval_raise(op, args, resume, env, chain) {
    match chain.find_handler(op) {
        Some(clause) => {
            let arg_values: Vec<Atom> = args.iter().map(|a| eval_atom(a, env)).collect()?;
            let resume_value = resolve_cont(resume, env)?;
            let mut new_env = env.clone();
            for (param, arg) in clause.params.iter().zip(arg_values.iter()) {
                new_env = new_env.with_binding(param.clone(), Value::Atom(arg.clone()));
            }
            new_env = new_env.with_binding(clause.resume.clone(), resume_value);
            eval_term(&clause.body, &new_env, chain)
        }
        None => Err(CpsError::UnhandledEffect(op.clone())),
    }
}
```

### Handle

Install a shallow handler frame on the chain, bind the resume continuation, and execute the body.

```rust
fn eval_handle(clause, body, cont, env, chain) {
    let cont_value = resolve_cont(cont, env)?;
    let mut new_chain = chain.clone();
    new_chain.push(HandlerFrame::Shallow { clause: clause.clone() });
    let mut new_env = env.clone();
    new_env = new_env.with_binding(clause.resume.clone(), cont_value);
    eval_term(body, &new_env, &new_chain)
}
```

Shallow handlers are removed after handling a single effect. Provider frames persist across resumes.

## Primitive operations

The interpreter supports 12 primitive operations:

| Operation | Arity | Types | Description |
|-----------|-------|-------|-------------|
| `Add` | 2 | `Int, Int` | Integer addition |
| `Sub` | 2 | `Int, Int` | Integer subtraction |
| `Mul` | 2 | `Int, Int` | Integer multiplication |
| `Div` | 2 | `Int, Int` | Integer division (div by zero errors) |
| `Eq` | 2 | Any | Structural equality |
| `Ne` | 2 | Any | Structural inequality |
| `Lt` | 2 | `Int, Int` | Less than |
| `Le` | 2 | `Int, Int` | Less than or equal |
| `Gt` | 2 | `Int, Int` | Greater than |
| `Ge` | 2 | `Int, Int` | Greater than or equal |
| `Neg` | 1 | `Int` | Integer negation |
| `Not` | 1 | `Bool` | Boolean negation |

## Error handling

The interpreter uses a typed error enum:

```rust
pub enum CpsError {
    UnboundVariable(Name),
    UnboundLabel(Name),
    ExpectedLambda(Value),
    ExpectedContinuation(Value),
    InvalidPrimArgs(PrimOp, Vec<Atom>),
    UnhandledEffect(EffectOp),
    Trap(TrapReason),
}
```

`Trap` is the final result type: when a term reaches an exit continuation, it traps with the result value. In the test harness, this is caught and returned as the test result.

## Handler chain semantics

The handler chain is searched innermost-first (top of stack to bottom). There are two frame types:

- **Shallow**: handles one effect, then is removed
- **Provider**: persists across resumes; does not have clauses directly

When `Raise` is evaluated:
1. Search the chain from innermost to outermost
2. Skip provider frames (they don't have clauses)
3. If a shallow handler matches, execute its clause body
4. If no handler matches, return `UnhandledEffect`

When a handler clause invokes `resume` with a value:
1. The resume continuation is the continuation captured at the `Raise` point
2. Execution continues from after the `Raise` with the provided value
3. The handler frame is not re-pushed (shallow semantics)

## Example: factorial execution trace

```rust
// (call fact [5] exit)
// -> n=5, k=exit
// -> is_zero = eq 5 0 = false
// -> n_minus_1 = sub 5 1 = 4
// -> k_mul bound with captured env (n=5, k=exit)
// -> (call fact [4] k_mul)
// -> n=4, k=k_mul (but exit preserved in k_mul's captured env)
// ... recursion continues ...
// -> base case: n=0, (jump k 1)
// -> k is innermost continuation
// -> continuations unwind, multiplying results
// -> final: (jump exit 120)
```

## Testing

The interpreter has comprehensive tests covering:

- Core evaluation (LetVal, LetPrim, LetCont, Jump, Call)
- Conditionals (If with bool/non-bool conditions)
- Effect raising and handling (Raise, Handle)
- Handler chain semantics (ordering, shallow removal, provider persistence)
- Resume continuations (restore chain, one-shot)
- Recursion (LetRec with factorial)
- Row validation (duplicate detection)
- Serialization round-trips (JSON and S-expressions)

All tests use TDD: failing tests were written first, then minimal implementation to make them pass.

## Known limitations

- No bytecode compilation or JIT (interpreter-only)
- No legacy AST lowering
- No Lean 4 differential testing
- Single-binding `LetRec` only; Phase 160 supports mutual recursion through the tuple-of-lambdas desugaring pattern, not native multi-binding `LetRec`
- Row validation is limited to duplicate checking
- Effect aliases not supported
- Full contract discharge not implemented

## See also

- [CPS IR](../language/cps-ir.md) — the intermediate representation
- [SPEC-099b: Target Operational Semantics](../../docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md) — formal semantics
- [PLAN-159: CPS IR Interpreter](../../docs/plan/PLAN-159-CPS-IR-INTERPRETER.md) — implementation plan
