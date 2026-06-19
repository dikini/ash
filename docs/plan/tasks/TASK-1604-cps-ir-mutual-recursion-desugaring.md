# TASK-1604: Support mutual recursion via tuple-of-lambdas in LetRec

## Status: 📝 Planned

## Description

Document and test the mutual recursion desugaring pattern using single `LetRec` with a tuple of lambdas. The frontend desugars mutual recursion (e.g., `letrec even odd = ...`) into a single `LetRec` binding a tuple, where each lambda extracts the other function from the tuple. This task verifies the interpreter correctly executes this pattern and adds test fixtures demonstrating it.

## Specification Reference

- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md) §2.3 — LetRec grammar
- [PLAN-160](../PLAN-160-CPS-IR-RUNTIME-EXPANSION.md)

## Dependencies

- ✅ TASK-1596: Single-binding LetRec with placeholder backfill
- ✅ TASK-1600: Record and Tuple value variants (must be complete)
- ✅ TASK-1601: Field access primitives (must be complete)

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

**Files:** `crates/ash-interp/tests/task_1604_cps_ir.rs`

```rust
#[test]
fn test_eval_mutual_recursion_even_odd() {
    // letrec pair = (tuple
    //   (lam [n] k
    //     (letprim is_zero (eq n 0)
    //       (if is_zero
    //         (jump k true {})
    //         (letprim n-1 (sub n 1)
    //           (letprim odd_fn (tuple_get 1 pair)
    //             (call odd_fn [n-1] k {}))))))
    //   (lam [n] k
    //     (letprim is_zero (eq n 0)
    //       (if is_zero
    //         (jump k false {})
    //         (letprim n-1 (sub n 1)
    //           (letprim even_fn (tuple_get 0 pair)
    //             (call even_fn [n-1] k {})))))))
    // in
    //   (letprim even (tuple_get 0 pair)
    //     (call even [4] exit {}))

    // Even lambda body
    let even_body = Term::LetPrim {
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
    };

    // Odd lambda body (similar structure, calls even)
    let odd_body = Term::LetPrim {
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
    };

    let pair_tuple = Value::Tuple {
        elems: vec![
            Atom::Var("even_unused".to_string()), // placeholder - will be backfilled
            Atom::Var("odd_unused".to_string()),  // placeholder - will be backfilled
        ],
    };

    // Actually, the tuple contains lambdas, not atoms. The lambdas capture `pair` by Var.
    let even_lam = Value::Lam {
        params: vec!["n".to_string()],
        cont: "k".to_string(),
        body: Box::new(even_body),
        row: EffectRow::default(),
    };
    let odd_lam = Value::Lam {
        params: vec!["n".to_string()],
        cont: "k".to_string(),
        body: Box::new(odd_body),
        row: EffectRow::default(),
    };

    let pair_value = Value::Tuple {
        elems: vec![
            Atom::Var("even_lam".to_string()),  // These will be resolved in env
            Atom::Var("odd_lam".to_string()),
        ],
    };

    // Hmm, this is getting complex. The actual pattern is:
    // letrec pair = (tuple even_lam odd_lam) in ...
    // where even_lam and odd_lam reference `pair` via Var.

    // Let me simplify: the tuple directly contains the lambdas as values,
    // and the lambdas' bodies reference `pair` as a variable.

    let pair_tuple = Value::Tuple {
        elems: vec![
            // We can't put Value::Lam in Atom position...
            // The tuple's elems are Atom, not Value.
            // So we need to bind the lambdas first, then put their names in the tuple.
        ],
    };
}
```

**Revised approach:** The frontend desugars to:

```rust
// letrec pair =
//   let even = (lam [n] k ... calls odd via (tuple_get 1 pair) ...)
//   let odd = (lam [n] k ... calls even via (tuple_get 0 pair) ...)
//   in (tuple even odd)
// in ...
```

But this requires nested LetVal inside LetRec, which the current IR supports.

Actually, the simplest pattern is:

```rust
// letrec pair = (tuple
//   (lam [n] k ... (tuple_get 1 pair) ...)
//   (lam [n] k ... (tuple_get 0 pair) ...))
// in ...
```

But `Value::Tuple` stores `Vec<Atom>`, and `Atom` can't hold a `Value::Lam`.

**Resolution:** The tuple stores variable names that are bound to the lambdas in the environment. The `LetRec` binds `pair` to a tuple of variable references, and the lambdas are bound separately. But that's not mutual recursion — the lambdas need to see `pair`.

**Correct pattern:** The frontend generates:

```rust
// letrec pair =
//   let even = (lam [n] k ...)
//   let odd = (lam [n] k ...)
//   in (tuple even odd)
// where even and odd bodies reference `pair`
// in ...
```

But `LetRec` only binds one name. The frontend must use a single recursive binding:

```rust
// letrec pair = (tuple even odd)
// in ...
// where even and odd are NOT bound separately — they are only accessible via tuple_get
```

This means the tuple must contain lambdas directly. But `Tuple` stores `Vec<Atom>`, not `Vec<Value>`.

**Decision:** Change `Value::Tuple` to store `Vec<Value>` for the elements, not `Vec<Atom>`. This is a small change from TASK-1600. Records should also store `Vec<(Name, Value)>` for consistency.

### Step 2: Implement (Green)

**Decision from analysis above:**

1. Update `Value::Tuple` to store `Vec<Value>` (not `Vec<Atom>`)
2. Update `Value::Record` to store `Vec<(Name, Value)>` (not `Vec<(Name, Atom)>`)
3. This allows tuples/records to contain lambdas and other structured values

**Files:** `crates/ash-core/src/cps.rs`

```rust
pub enum Value {
    Atom(Atom),
    Lam { ... },
    Cont { ... },
    Record { fields: Vec<(Name, Value)> },
    Tuple { elems: Vec<Value> },
}
```

Update `eval_value` to recursively evaluate fields/elements:

```rust
Value::Record { fields } => {
    let mut new_fields = Vec::new();
    for (name, value) in fields {
        new_fields.push((name.clone(), eval_value(value, env)?));
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
```

Update `RecordGet` and `TupleGet` to work with `Value`:

```rust
// RecordGet: args[0] is variable name, resolve to Value::Record
// TupleGet: args[0] is variable name, resolve to Value::Tuple
// Return the extracted value as an Atom (if it's Atom) or bind it
```

Actually, `RecordGet`/`TupleGet` should return the extracted value, which may be a `Value::Lam`. So `LetPrim` needs to bind a `Value`, not an `Atom`.

**Current `LetPrim`:** binds `Value::Atom(result)` — this loses lambda values.

**Fix:** `eval_prim` should return `Value`, not `Atom`. Then `LetPrim` binds the `Value` directly.

This is a significant change. Let me document it properly.

### Revised Step 2: Implement

**Files:** `crates/ash-core/src/cps.rs`, `crates/ash-interp/src/cps.rs`

1. Change `Value::Record` fields to `Vec<(Name, Value)>`
2. Change `Value::Tuple` elems to `Vec<Value>`
3. Change `eval_prim` return type to `CpsResult<Value>`
4. Update `LetPrim` to bind the returned `Value` directly
5. Update all existing `PrimOp` evaluators to return `Value::Atom(...)` instead of `Atom(...)`

### Step 3: Integration

- Update all test files that construct `Value::Record` or `Value::Tuple`
- Verify existing tests still pass

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
  - cargo test -p ash-core -p ash-interp --test task_1604_cps_ir
  - cargo test -p ash-core -p ash-interp
  - cargo clippy -p ash-core -p ash-interp --all-targets -- -D warnings
  - cargo fmt --check
checklist:
  - [ ] Even/Odd mutual recursion test passes
  - [ ] State machine mutual recursion test passes
  - [ ] Existing Phase 159 tests still pass
  - [ ] No clippy warnings
```

## Dependencies for Next Task

- Provides mutual recursion pattern for TASK-1606 (speculative fixtures)

## Notes

- This task reveals that `Value::Record` and `Value::Tuple` should store `Value`, not `Atom`, to support nested lambdas. This is a correction to TASK-1600's initial design.
- The `eval_prim` return type change to `Value` is necessary for `RecordGet`/`TupleGet` to return lambdas.
- The frontend desugaring pattern for mutual recursion is documented in the test fixtures and will be referenced in operational semantics (TASK-1607).
