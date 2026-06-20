# TASK-1611: Add RecordGet and TupleGet primitive operations

## Status: ✅ Complete

## Description

Add `PrimOp::RecordGet` and `PrimOp::TupleGet` primitive operations to the CPS IR evaluator. These enable field/element access on records and tuples, which is the runtime mechanism for destructuring structured data.

## Specification Reference

- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md) §2.2 — Value grammar
- [PLAN-160](../PLAN-160-CPS-IR-RUNTIME-EXPANSION.md)

## Dependencies

- ✅ TASK-1590: Core CPS IR data structures
- ✅ TASK-1610: Record and Tuple value variants (must be complete)

## Requirements

### Functional Requirements

1. Add `PrimOp::RecordGet(Name)` — extract field by name from `Value::Record`
2. Add `PrimOp::TupleGet(usize)` — extract element by index from `Value::Tuple`
3. Both operations fail closed if the field/index does not exist (return `CpsError::InvalidPrimArgs`)
4. Arguments to `RecordGet`/`TupleGet` are evaluated (variables resolved) before access
5. **Both operations return `Value`** (not `Atom`) — this enables extracting lambdas and other structured values

### Property Requirements

- `RecordGet` on missing field returns error
- `TupleGet` with out-of-bounds index returns error
- `TupleGet` with non-integer index returns error (type error — frontend should prevent this, but runtime defends)

## TDD Steps

### Step 1: Write Tests (Red)

**Files:** `crates/ash-interp/tests/task_1611_cps_ir.rs`

```rust
use ash_core::cps::*;
use ash_interp::cps::eval_checked;

#[test]
fn test_eval_record_get() {
    let term = Term::LetVal {
        name: "r".to_string(),
        value: Value::Record {
            fields: vec![
                ("x".to_string(), Value::Atom(Atom::Int(42))),
                ("y".to_string(), Value::Atom(Atom::Int(7))),
            ],
        },
        body: Box::new(Term::LetPrim {
            name: "x_val".to_string(),
            op: PrimOp::RecordGet("x".to_string()),
            args: vec![Atom::Var("r".to_string())],
            body: Box::new(Term::Return { value: Atom::Var("x_val".to_string()) }),
        }),
    };
    let result = eval_checked(&term, &Env::new(), &HandlerChain::new());
    assert_eq!(result, Ok(Atom::Int(42)));
}

#[test]
fn test_eval_tuple_get() {
    let term = Term::LetVal {
        name: "t".to_string(),
        value: Value::Tuple {
            elems: vec![
                Value::Atom(Atom::Int(10)),
                Value::Atom(Atom::Int(20)),
                Value::Atom(Atom::Int(30)),
            ],
        },
        body: Box::new(Term::LetPrim {
            name: "second".to_string(),
            op: PrimOp::TupleGet(1),
            args: vec![Atom::Var("t".to_string())],
            body: Box::new(Term::Return { value: Atom::Var("second".to_string()) }),
        }),
    };
    let result = eval_checked(&term, &Env::new(), &HandlerChain::new());
    assert_eq!(result, Ok(Atom::Int(20)));
}

#[test]
fn test_eval_record_get_missing_field() {
    let term = Term::LetVal {
        name: "r".to_string(),
        value: Value::Record {
            fields: vec![("x".to_string(), Value::Atom(Atom::Int(1)))],
        },
        body: Box::new(Term::LetPrim {
            name: "z".to_string(),
            op: PrimOp::RecordGet("z".to_string()),
            args: vec![Atom::Var("r".to_string())],
            body: Box::new(Term::Return { value: Atom::Var("z".to_string()) }),
        }),
    };
    let result = eval_checked(&term, &Env::new(), &HandlerChain::new());
    assert!(matches!(result, Err(CpsRunError::Runtime(CpsError::InvalidPrimArgs(_, _)))));
}

#[test]
fn test_eval_record_get_returns_lambda() {
    // RecordGet can return a lambda (trait dictionary pattern)
    let id_lam = Value::Lam {
        params: vec!["x".to_string()],
        cont: "k".to_string(),
        body: Box::new(Term::Jump {
            cont: ContRef::Var("k".to_string()),
            arg: Atom::Var("x".to_string()),
            row: EffectRow::default(),
        }),
        captured_env: Env::new(),
        rec_binding: None, // non-recursive lambda
        row: EffectRow::default(),
    };
    let term = Term::LetVal {
        name: "dict".to_string(),
        value: Value::Record {
            fields: vec![
                ("id".to_string(), id_lam),
            ],
        },
        body: Box::new(Term::LetPrim {
            name: "id_fn".to_string(),
            op: PrimOp::RecordGet("id".to_string()),
            args: vec![Atom::Var("dict".to_string())],
            body: Box::new(Term::Call {
                func: Atom::Var("id_fn".to_string()),
                args: vec![Atom::Int(42)],
                cont: ContRef::Label("exit".to_string()),
                row: EffectRow::default(),
            }),
        }),
    };
    let result = eval_checked(&term, &Env::new(), &HandlerChain::new());
    assert_eq!(result, Ok(Atom::Int(42)));
}
```

### Step 2: Implement (Green)

**Files:** `crates/ash-core/src/cps.rs`, `crates/ash-interp/src/cps.rs`

Add to `PrimOp` enum (remove `Copy` derive, keep `Clone`):

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrimOp {
    Add, Sub, Mul, Div, Eq, Ne, Lt, Le, Gt, Ge, Neg, Not,
    RecordGet(Name),   // field name — requires removing Copy
    TupleGet(usize),   // element index
}
```

**Breaking change:** `PrimOp` no longer derives `Copy`. Update all call sites that pass `PrimOp` by value to use references or clone. Specifically:
- `eval_letprim` parameter `op: PrimOp` → `op: &PrimOp` (or clone at call site)
- `eval_prim` parameter `op: PrimOp` → `op: &PrimOp`
- `CpsError::InvalidPrimArgs` currently stores `PrimOp` — since `PrimOp` is no longer `Copy`, update the error variant to store `PrimOp` (which is `Clone`) or a description string. The error constructor can clone: `CpsError::InvalidPrimArgs(op.clone(), ...)`
- Match arms in `eval_letprim` that use `*op` → use `op` directly (now reference)
- Update `eval_unchecked` match arm for `Term::LetPrim` to pass `op` by reference: `eval_letprim(name, op, args, body, env, chain)`

**Note:** `TupleGet(usize)` could theoretically keep `Copy` since `usize` is Copy, but `RecordGet(Name)` requires String which is not. For consistency, remove `Copy` from the entire enum. Use tuple variants `RecordGet(Name)` and `TupleGet(usize)` (not struct variants).

Add cases to `eval_prim` (now takes `&[Value]` and `env`):

```rust
fn eval_prim(op: &PrimOp, args: &[Value], env: &Env) -> CpsResult<Value> {
    let make_err = || CpsError::InvalidPrimArgs(op.clone(), vec![]);
    match op {
        // Existing arithmetic: args are Value::Atom, extract Atom
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
        // ... Sub, Mul, Div, Eq, etc. similarly ...
        
        PrimOp::RecordGet(field) => {
            let record = args.first().ok_or_else(make_err)?;
            match record {
                Value::Record { fields } => {
                    fields.iter()
                        .find(|(f, _)| f == field)
                        .map(|(_, v)| v.clone())
                        .ok_or_else(make_err)
                }
                _ => Err(make_err()),
            }
        }
        
        PrimOp::TupleGet(index) => {
            let tuple = args.first().ok_or_else(make_err)?;
            match tuple {
                Value::Tuple { elems } => {
                    elems.get(*index).cloned().ok_or_else(make_err)
                }
                _ => Err(make_err()),
            }
        }
    }
}
```

**Note:** `eval_prim` now receives pre-resolved `Value` arguments (from `eval_letprim`), not raw `Atom`s. For arithmetic, it matches on `Value::Atom(...)`. For `RecordGet`/`TupleGet`, the first argument is the record/tuple value itself.

### Step 3: Integration

- Ensure `PrimOp` serialization handles new variants
- Update any exhaustive match sites

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-core -p ash-interp --test task_1611_cps_ir
  - cargo clippy -p ash-core -p ash-interp --all-targets -- -D warnings
  - cargo fmt --check
checklist:
  - [ ] RecordGet extracts correct field (atom value)
  - [ ] TupleGet extracts correct element (atom value)
  - [ ] RecordGet extracts lambda from record (structured value)
  - [ ] Missing field returns error
  - [ ] Out-of-bounds index returns error
  - [ ] No clippy warnings
  - [ ] CHANGELOG.md entry staged
```

## Dependencies for Next Task

- Provides field access for TASK-1616 (speculative fixtures)

## Notes

- `RecordGet` takes the field name as part of the `PrimOp`, not as a runtime argument. This is because field names are statically known in the lowered IR.
- The argument to `RecordGet`/`TupleGet` is the record/tuple value (as a variable reference), not the field name/index.
