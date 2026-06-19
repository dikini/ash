# TASK-1601: Add RecordGet and TupleGet primitive operations

## Status: 📝 Planned

## Description

Add `PrimOp::RecordGet` and `PrimOp::TupleGet` primitive operations to the CPS IR evaluator. These enable field/element access on records and tuples, which is the runtime mechanism for destructuring structured data.

## Specification Reference

- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md) §2.2 — Value grammar
- [PLAN-160](../PLAN-160-CPS-IR-RUNTIME-EXPANSION.md)

## Dependencies

- ✅ TASK-1590: Core CPS IR data structures
- ✅ TASK-1600: Record and Tuple value variants (must be complete)

## Requirements

### Functional Requirements

1. Add `PrimOp::RecordGet(Name)` — extract field by name from `Value::Record`
2. Add `PrimOp::TupleGet(usize)` — extract element by index from `Value::Tuple`
3. Both operations fail closed if the field/index does not exist (return `CpsError::InvalidPrimArgs`)
4. Arguments to `RecordGet`/`TupleGet` are evaluated (variables resolved) before access

### Property Requirements

- `RecordGet` on missing field returns error
- `TupleGet` with out-of-bounds index returns error
- `TupleGet` with non-integer index returns error (type error — frontend should prevent this, but runtime defends)

## TDD Steps

### Step 1: Write Tests (Red)

**Files:** `crates/ash-interp/tests/task_1601_cps_ir.rs`

```rust
#[test]
fn test_eval_record_get() {
    let term = Term::LetVal {
        name: "r".to_string(),
        value: Value::Record {
            fields: vec![
                ("x".to_string(), Atom::Int(42)),
                ("y".to_string(), Atom::Int(7)),
            ],
        },
        body: Box::new(Term::LetPrim {
            name: "x_val".to_string(),
            op: PrimOp::RecordGet("x".to_string()),
            args: vec![Atom::Var("r".to_string())],
            body: Box::new(Term::Return { value: Atom::Var("x_val".to_string()) }),
        }),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert_eq!(result, Ok(Atom::Int(42)));
}

#[test]
fn test_eval_tuple_get() {
    let term = Term::LetVal {
        name: "t".to_string(),
        value: Value::Tuple {
            elems: vec![Atom::Int(10), Atom::Int(20), Atom::Int(30)],
        },
        body: Box::new(Term::LetPrim {
            name: "second".to_string(),
            op: PrimOp::TupleGet(1),
            args: vec![Atom::Var("t".to_string())],
            body: Box::new(Term::Return { value: Atom::Var("second".to_string()) }),
        }),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert_eq!(result, Ok(Atom::Int(20)));
}

#[test]
fn test_eval_record_get_missing_field() {
    let term = Term::LetVal {
        name: "r".to_string(),
        value: Value::Record {
            fields: vec![("x".to_string(), Atom::Int(1))],
        },
        body: Box::new(Term::LetPrim {
            name: "z".to_string(),
            op: PrimOp::RecordGet("z".to_string()),
            args: vec![Atom::Var("r".to_string())],
            body: Box::new(Term::Return { value: Atom::Var("z".to_string()) }),
        }),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert!(matches!(result, Err(CpsError::InvalidPrimArgs(_, _))));
}
```

### Step 2: Implement (Green)

**Files:** `crates/ash-core/src/cps.rs`, `crates/ash-interp/src/cps.rs`

Add to `PrimOp` enum:

```rust
pub enum PrimOp {
    Add, Sub, Mul, Div, Eq, Ne, Lt, Le, Gt, Ge, Neg, Not,
    RecordGet(Name),   // field name
    TupleGet(usize),   // element index
}
```

Add cases to `eval_prim`:

```rust
PrimOp::RecordGet(field) => {
    let a = args.first().ok_or_else(make_err)?;
    match a {
        Atom::Var(name) => {
            let value = env.lookup(name).ok_or_else(make_err)?;
            match value {
                Value::Record { fields } => {
                    fields.iter()
                        .find(|(f, _)| f == field)
                        .map(|(_, v)| v.clone())
                        .ok_or_else(make_err)
                }
                _ => Err(make_err()),
            }
        }
        _ => Err(make_err()),
    }
}
PrimOp::TupleGet(index) => {
    let a = args.first().ok_or_else(make_err)?;
    match a {
        Atom::Var(name) => {
            let value = env.lookup(name).ok_or_else(make_err)?;
            match value {
                Value::Tuple { elems } => {
                    elems.get(*index).cloned().ok_or_else(make_err)
                }
                _ => Err(make_err()),
            }
        }
        _ => Err(make_err()),
    }
}
```

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
  - cargo test -p ash-core -p ash-interp --test task_1601_cps_ir
  - cargo clippy -p ash-core -p ash-interp --all-targets -- -D warnings
  - cargo fmt --check
checklist:
  - [ ] RecordGet extracts correct field
  - [ ] TupleGet extracts correct element
  - [ ] Missing field returns error
  - [ ] Out-of-bounds index returns error
  - [ ] No clippy warnings
```

## Dependencies for Next Task

- Provides field access for TASK-1606 (speculative fixtures)

## Notes

- `RecordGet` takes the field name as part of the `PrimOp`, not as a runtime argument. This is because field names are statically known in the lowered IR.
- The argument to `RecordGet`/`TupleGet` is the record/tuple value (as a variable reference), not the field name/index.
