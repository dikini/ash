# TASK-1600: Add Record and Tuple value variants to CPS IR

## Status: 📝 Planned

## Description

Add `Value::Record` and `Value::Tuple` variants to the CPS IR data model in `crates/ash-core/src/cps.rs`. These are inert value constructors that enable structured data in the interpreter. Fields are atoms (inert data), not values (no nested computation).

## Specification Reference

- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md) §2.2 — Value grammar already defines `Record { fields: Vec<(Name, Atom)> }` and `Tuple { elems: Vec<Atom> }`
- [PLAN-160](../PLAN-160-CPS-IR-RUNTIME-EXPANSION.md)

## Dependencies

- ✅ TASK-1590: Core CPS IR data structures (Atom, Value, Term, Env, HandlerChain)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Record/Tuple values | SPEC-098b §2.2 | Planned for Phase 2 but not implemented | ✅ satisfied | implement now | `test_eval_record_construction` |

## Requirements

### Functional Requirements

1. Add `Value::Record { fields: Vec<(Name, Atom)> }` variant
2. Add `Value::Tuple { elems: Vec<Atom> }` variant
3. Both variants must be `Clone`, `PartialEq`, `Debug`, `Serialize`, `Deserialize`
4. `eval_value` must pass through records and tuples unchanged (they are inert)
5. `eval_atom` must resolve variables inside record/tuple fields when the record/tuple is bound to a variable and then extracted

### Property Requirements

- Record construction with duplicate field names is a frontend error, not a runtime concern (interpreter assumes well-formed input)
- Tuple element count is fixed at construction time

## TDD Steps

### Step 1: Write Tests (Red)

**Files:** `crates/ash-interp/tests/task_1600_cps_ir.rs`

```rust
#[test]
fn test_eval_record_construction() {
    let record = Value::Record {
        fields: vec![
            ("x".to_string(), Atom::Int(1)),
            ("y".to_string(), Atom::Int(2)),
        ],
    };
    let term = Term::LetVal {
        name: "r".to_string(),
        value: record,
        body: Box::new(Term::Return { value: Atom::Var("r".to_string()) }),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    // Should return the record value
    assert!(matches!(result, Ok(Atom::Var(_))));
}

#[test]
fn test_eval_tuple_construction() {
    let tuple = Value::Tuple {
        elems: vec![Atom::Int(1), Atom::Int(2), Atom::Int(3)],
    };
    let term = Term::LetVal {
        name: "t".to_string(),
        value: tuple,
        body: Box::new(Term::Return { value: Atom::Var("t".to_string()) }),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert!(matches!(result, Ok(Atom::Var(_))));
}
```

### Step 2: Implement (Green)

**Files:** `crates/ash-core/src/cps.rs`

Add variants to `Value` enum:

```rust
pub enum Value {
    Atom(Atom),
    Lam { ... },
    Cont { ... },
    Record { fields: Vec<(Name, Atom)> },
    Tuple { elems: Vec<Atom> },
}
```

Update `eval_value` in `crates/ash-interp/src/cps.rs` to pass through new variants:

```rust
fn eval_value(value: &Value, env: &Env) -> CpsResult<Value> {
    match value {
        Value::Atom(atom) => Ok(Value::Atom(eval_atom(atom, env)?)),
        Value::Record { .. } | Value::Tuple { .. } => Ok(value.clone()),
        other => Ok(other.clone()),
    }
}
```

### Step 3: Integration

- Wire through crate exports
- Ensure serde serialization works for new variants

### Step 4: Property Tests

**File:** `crates/ash-interp/tests/task_1600_cps_ir.rs`

Add round-trip tests: construct → serialize → parse → compare.

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
  - cargo test -p ash-core -p ash-interp --test task_1600_cps_ir
  - cargo clippy -p ash-core -p ash-interp --all-targets -- -D warnings
  - cargo fmt --check
checklist:
  - [ ] Record construction test passes
  - [ ] Tuple construction test passes
  - [ ] Round-trip serialization test passes
  - [ ] No clippy warnings
```

## Dependencies for Next Task

- Provides `Value::Record` and `Value::Tuple` for TASK-1601 (field access primitives)

## Notes

- Keep fields as `Atom`, not `Value`. Records are inert data in CPS IR.
- The S-expression parser/serializer will be updated in TASK-1605, not here.
