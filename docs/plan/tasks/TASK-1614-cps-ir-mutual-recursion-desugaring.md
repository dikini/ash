# TASK-1614: Support mutual recursion via tuple-of-lambdas in LetRec

## Status: 📝 Planned

## Description

Document and test the mutual recursion desugaring pattern using single `LetRec` with a tuple of lambdas. The frontend desugars mutual recursion (e.g., `letrec even odd = ...`) into a single `LetRec` binding a tuple, where each lambda extracts the other function from the tuple. This task verifies the interpreter correctly executes this pattern and adds test fixtures demonstrating it.

## Specification Reference

- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md) §2.3 — LetRec grammar
- [PLAN-160](../PLAN-160-CPS-IR-RUNTIME-EXPANSION.md)

## Dependencies

- ✅ TASK-1596: Single-binding LetRec with placeholder backfill
- ✅ TASK-1610: Record and Tuple value variants (must be complete)
- ✅ TASK-1611: Field access primitives (must be complete)

## Requirements

### Functional Requirements

1. The existing `LetRec` (single binding) must work when the bound value is a tuple of lambdas
2. Each lambda in the tuple must correctly reference the bound name (via `Var`) to extract sibling functions
3. The placeholder/backfill mechanism must handle tuple values correctly
4. Add test fixtures for:
   - Even/Odd mutual recursion
   - Mutually recursive state machine (e.g., locked/unlocked)

### Property Requirements

- Recursive calls through tuple extraction resolve correctly
- The placeholder `Null` is not visible after backfill
- Multiple recursive calls in a single evaluation work

## TDD Steps

### Step 1: Write Tests (Red)

**Files:** `crates/ash-interp/tests/task_1614_cps_ir.rs`

```rust
use ash_core::cps::*;
use ash_interp::cps::eval_checked;

#[test]
fn test_eval_mutual_recursion_even_odd() {
    // letrec pair = (tuple even_lam odd_lam)
    //   where even_lam = (lam [n] k ... (tuple_get 1 pair) ...)
    //         odd_lam  = (lam [n] k ... (tuple_get 0 pair) ...)
    // in (letprim even = (tuple_get 0 pair)
    //     in (call even [4] exit))

    let even_lam = Value::Lam {
        params: vec!["n".to_string()],
        cont: "k".to_string(),
        body: Box::new(Term::LetPrim {
            name: "is_zero".to_string(),
            op: PrimOp::Eq,
            args: vec![Atom::Var("n".to_string()), Atom::Int(0)],
            body: Box::new(Term::If {
                cond: Atom::Var("is_zero".to_string()),
                then_branch: Box::new(Term::Jump {
                    cont: ContRef::Var("k".to_string()),
                    arg: Atom::Bool(true),
                    row: EffectRow::default(),
                }),
                else_branch: Box::new(Term::LetPrim {
                    name: "n_minus_1".to_string(),
                    op: PrimOp::Sub,
                    args: vec![Atom::Var("n".to_string()), Atom::Int(1)],
                    body: Box::new(Term::LetPrim {
                        name: "odd_fn".to_string(),
                        op: PrimOp::TupleGet(1),
                        args: vec![Atom::Var("pair".to_string())],
                        body: Box::new(Term::Call {
                            func: Atom::Var("odd_fn".to_string()),
                            args: vec![Atom::Var("n_minus_1".to_string())],
                            cont: ContRef::Var("k".to_string()),
                            row: EffectRow::default(),
                        }),
                    }),
                }),
                row: EffectRow::default(),
            }),
        }),
        captured_env: Env::new(),
        rec_binding: Some("pair".to_string()), // recursive: needs "pair" at call time
        row: EffectRow::default(),
    };

    let odd_lam = Value::Lam {
        params: vec!["n".to_string()],
        cont: "k".to_string(),
        body: Box::new(Term::LetPrim {
            name: "is_zero".to_string(),
            op: PrimOp::Eq,
            args: vec![Atom::Var("n".to_string()), Atom::Int(0)],
            body: Box::new(Term::If {
                cond: Atom::Var("is_zero".to_string()),
                then_branch: Box::new(Term::Jump {
                    cont: ContRef::Var("k".to_string()),
                    arg: Atom::Bool(false),
                    row: EffectRow::default(),
                }),
                else_branch: Box::new(Term::LetPrim {
                    name: "n_minus_1".to_string(),
                    op: PrimOp::Sub,
                    args: vec![Atom::Var("n".to_string()), Atom::Int(1)],
                    body: Box::new(Term::LetPrim {
                        name: "even_fn".to_string(),
                        op: PrimOp::TupleGet(0),
                        args: vec![Atom::Var("pair".to_string())],
                        body: Box::new(Term::Call {
                            func: Atom::Var("even_fn".to_string()),
                            args: vec![Atom::Var("n_minus_1".to_string())],
                            cont: ContRef::Var("k".to_string()),
                            row: EffectRow::default(),
                        }),
                    }),
                }),
                row: EffectRow::default(),
            }),
        }),
        captured_env: Env::new(),
        rec_binding: Some("pair".to_string()), // recursive: needs "pair" at call time
        row: EffectRow::default(),
    };

    let pair_tuple = Value::Tuple {
        elems: vec![even_lam, odd_lam],
    };

    let term = Term::LetRec {
        name: "pair".to_string(),
        value: pair_tuple,
        body: Box::new(Term::LetPrim {
            name: "even".to_string(),
            op: PrimOp::TupleGet(0),
            args: vec![Atom::Var("pair".to_string())],
            body: Box::new(Term::Call {
                func: Atom::Var("even".to_string()),
                args: vec![Atom::Int(4)],
                cont: ContRef::Label("exit".to_string()),
                row: EffectRow::default(),
            }),
        }),
    };

    let result = eval_checked(&term, &Env::new(), &HandlerChain::new());
    assert_eq!(result, Ok(Atom::Bool(true)));
}
```

### Step 2: Implement (Green)

**Required: Add `rec_binding` marker to `Value::Lam` and update `eval_call` to overlay recursive bindings.**

The current `eval_letrec` (line 258 of `cps/mod.rs`) only special-cases `Value::Lam`:
```rust
let lam_value = match value {
    Value::Lam { ... } => { ... backfill ... }
    other => other.clone(),  // Tuple falls through — no backfill!
};
```

For mutual recursion via tuple-of-lambdas, the lambdas inside the tuple capture the environment at construction time. If that environment contains a placeholder for the recursive binding, the lambdas will forever see the placeholder.

**Solution: Add `rec_binding` marker to `Value::Lam` for scoped call-time overlay.**

Instead of mutating captured environments or broadly overlaying all call-site bindings, we extend `eval_call` to overlay **only the specific recursive binding** that the lambda needs. This preserves lexical scoping while enabling mutual recursion.

**How it works:**

1. `LetRec` evaluates as usual: bind placeholder → evaluate value → backfill with actual value
2. When a lambda is called, `eval_call` checks if the call-site environment contains a recursive binding that the lambda's captured environment is missing
3. If so, only that specific recursive binding is overlaid — not all missing bindings

**Implementation (Option A: minimal marker):**

Add `rec_binding: Option<Name>` to `Value::Lam`:

```rust
pub enum Value {
    Atom(Atom),
    Lam {
        params: Vec<Name>,
        cont: Name,
        body: Box<Term>,
        captured_env: Env,
        rec_binding: Option<Name>, // NEW: Some(name) for recursive lambdas, None otherwise
        row: EffectRow,
    },
    Cont { ... },
}
```

Update `eval_letrec` to set `rec_binding` when constructing lambdas inside the recursive value:

```rust
fn eval_letrec(
    name: &Name,
    value: &Value,
    body: &Term,
    env: &Env,
    chain: &HandlerChain,
) -> CpsResult<Atom> {
    let mut new_env = env.clone();
    
    // Phase 1: Bind placeholder
    let placeholder = Value::Atom(Atom::Null);
    new_env = new_env.with_binding(name.clone(), placeholder);
    
    // Phase 2: Evaluate value — lambdas capture env with placeholder
    let evaluated_value = eval_value_with_rec_binding(value, &new_env, name)?;
    
    // Phase 3: Backfill with actual value
    new_env = new_env.with_binding(name.clone(), evaluated_value);
    
    // Phase 4: Evaluate body
    eval_unchecked(body, &new_env, chain)
}

/// Evaluate value, marking any constructed lambdas with the recursive binding name.
fn eval_value_with_rec_binding(
    value: &Value,
    env: &Env,
    rec_name: &Name,
) -> CpsResult<Value> {
    match value {
        Value::Atom(atom) => Ok(Value::Atom(eval_atom(atom, env)?)),
        Value::Lam { params, cont, body, row, .. } => {
            Ok(Value::Lam {
                params: params.clone(),
                cont: cont.clone(),
                body: body.clone(),
                captured_env: env.clone(),
                rec_binding: Some(rec_name.clone()), // MARK: this lambda needs rec_name at call time
                row: row.clone(),
            })
        }
        Value::Cont { param, body, captured_chain, consumed, row } => {
            Ok(Value::Cont { ... }) // continuations don't need rec_binding
        }
        Value::Record { fields } => { ... }
        Value::Tuple { elems } => { ... }
    }
}
```

Update `eval_call` to overlay only the marked recursive binding:

```rust
fn eval_call(
    func: &Atom,
    args: &[Atom],
    cont: &ContRef,
    env: &Env,
    chain: &HandlerChain,
) -> CpsResult<Atom> {
    let func_value = resolve_value(func, env)?;
    let arg_values: CpsResult<Vec<Value>> = args.iter().map(|a| eval_atom_to_value(a, env)).collect();
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
            let mut new_env = captured_env.clone();
            
            // NEW: Overlay only the specific recursive binding, if any
            if let Some(rec_name) = rec_binding {
                if let Some(rec_value) = env.lookup(&rec_name) {
                    new_env = new_env.with_binding(rec_name, rec_value.clone());
                }
            }
            
            // Bind arguments and continuation
            for (param, arg) in params.iter().zip(arg_values.iter()) {
                new_env = new_env.with_binding(param.clone(), arg.clone());
            }
            new_env = new_env.with_binding(lam_cont.clone(), cont_value);
            eval_unchecked(&body, &new_env, chain)
        }
        _ => Err(CpsError::ExpectedLambda(func_value)),
    }
}
```

**Why this is correct:**
- The `rec_binding` marker precisely identifies which binding to overlay
- Non-recursive lambdas have `rec_binding: None` — no overlay, no dynamic scoping
- Recursive lambdas have `rec_binding: Some(name)` — only `name` is overlaid
- The test `test_eval_call_no_dynamic_scope` (from the analysis above) passes because caller-local "x" is not the marked recursive binding

**Files:** `crates/ash-core/src/cps.rs` (add `rec_binding` to `Value::Lam`), `crates/ash-interp/src/cps/mod.rs` (update `eval_letrec` and `eval_call`)

**Implementation sequencing:**
1. First, add `rec_binding: Option<Name>` to `Value::Lam` in `crates/ash-core/src/cps.rs`
2. Fix all compiler errors from the new field (update existing `Value::Lam` constructors in tests, add `rec_binding: None` or `Some(...)`)
3. Verify serde roundtrip still works (run `cargo test -p ash-core --test task_1599_cps_ir`)
4. Only then proceed to `eval_letrec` and `eval_call` changes

### Step 3: Integration

- Run the test to verify the pattern works end-to-end
- Add the test to the Phase 160 test suite
- Verify existing Phase 159 LetRec tests still pass (factorial, etc.)

## Dispatch

```yaml
agent: hermes
reasoning: high
max_turns: 20
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-core -p ash-interp --test task_1614_cps_ir
  - cargo test -p ash-core -p ash-interp
  - cargo clippy -p ash-core -p ash-interp --all-targets -- -D warnings
  - cargo fmt --check
checklist:
  - [ ] Even/Odd mutual recursion test passes
  - [ ] State machine mutual recursion test passes
  - [ ] Existing Phase 159 tests still pass
  - [ ] No clippy warnings
  - [ ] CHANGELOG.md entry staged
```

## Dependencies for Next Task

- Provides mutual recursion pattern for TASK-1616 (speculative fixtures)

## Notes

- The `Value::Record` and `Value::Tuple` types store `Value` (not `Atom`) for fields/elements, enabling nested lambdas. This is the design from TASK-1610.
- The `eval_prim` return type is `Value` (not `Atom`), enabling `RecordGet`/`TupleGet` to return lambdas. This is the design from TASK-1610.
- The frontend desugaring pattern for mutual recursion is documented in the test fixtures and will be referenced in operational semantics (TASK-1617).
