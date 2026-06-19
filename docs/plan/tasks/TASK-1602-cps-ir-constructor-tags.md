# TASK-1602: Add ConstructorName atom variant for sum types

## Status: 📝 Planned

## Description

Add `Atom::ConstructorName(Name)` variant to the CPS IR data model. Constructor names are tags used for sum type discrimination — they enable pattern matching on algebraic data types by carrying a static tag that can be compared for equality.

## Specification Reference

- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md) §2.1 — Atom grammar
- [PLAN-160](../PLAN-160-CPS-IR-RUNTIME-EXPANSION.md)

## Dependencies

- ✅ TASK-1590: Core CPS IR data structures

## Requirements

### Functional Requirements

1. Add `Atom::ConstructorName(Name)` variant
2. Constructor names are inert — they evaluate to themselves
3. Constructor names support equality comparison (`==`, `!=`)
4. Constructor names serialize/deserialize correctly

### Property Requirements

- Two constructor names with the same text are equal
- Two constructor names with different text are not equal
- Constructor names are distinct from string literals (different namespace)

## TDD Steps

### Step 1: Write Tests (Red)

**Files:** `crates/ash-interp/tests/task_1602_cps_ir.rs`

```rust
#[test]
fn test_constructor_name_equality() {
    let a = Atom::ConstructorName("Circle".to_string());
    let b = Atom::ConstructorName("Circle".to_string());
    let c = Atom::ConstructorName("Rect".to_string());
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn test_constructor_name_eval() {
    let atom = Atom::ConstructorName("Circle".to_string());
    let result = eval_atom(&atom, &Env::new());
    assert_eq!(result, Ok(Atom::ConstructorName("Circle".to_string())));
}
```

### Step 2: Implement (Green)

**Files:** `crates/ash-core/src/cps.rs`

Add to `Atom` enum:

```rust
pub enum Atom {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    Var(Name),
    ConstructorName(Name),  // NEW
}
```

`eval_atom` already passes through non-`Var` atoms unchanged, so no evaluator change needed.

### Step 3: Integration

- Ensure `Eq`/`PartialEq` derive handles new variant correctly
- Verify serde serialization round-trips

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
  - cargo test -p ash-core -p ash-interp --test task_1602_cps_ir
  - cargo clippy -p ash-core -p ash-interp --all-targets -- -D warnings
  - cargo fmt --check
checklist:
  - [ ] Constructor name equality works
  - [ ] Constructor name evaluation is identity
  - [ ] Serialization round-trip works
  - [ ] No clippy warnings
```

## Dependencies for Next Task

- Provides constructor tags for TASK-1603 (match dispatch)

## Notes

- Constructor names are not function pointers. They are purely tags for discrimination.
- In the lowered IR, a sum type constructor like `Circle(radius)` becomes:
  ```rust
  Value::Tuple {
      elems: vec![
          Atom::ConstructorName("Circle".to_string()),
          Atom::Var("radius".to_string()),
      ],
  }
  ```
- The frontend is responsible for generating the correct tag + payload structure.
