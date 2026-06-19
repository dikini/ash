# TASK-1606: Write speculative test fixtures for upper-language patterns

## Status: 📝 Planned

## Description

Write `.cps` test fixtures that demonstrate how upper-language features (mutual recursion, records, sum types, trait dictionary passing) lower to the expanded CPS IR. These fixtures serve as:
1. **Proof of concept** for frontend lowering strategies
2. **Regression tests** for the interpreter's new features
3. **Documentation** for the lowering contract between frontend and runtime

## Specification Reference

- [PLAN-160](../PLAN-160-CPS-IR-RUNTIME-EXPANSION.md)
- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md)

## Dependencies

- ✅ TASK-1604: Mutual recursion desugaring (must be complete)
- ✅ TASK-1605: S-expression parser updates (must be complete)

## Requirements

### Functional Requirements

Write test fixtures for each pattern:

1. **Mutual recursion (even/odd)** — demonstrates tuple-of-lambdas desugaring
2. **Record construction and access** — demonstrates product type lowering
3. **Sum type construction and pattern matching** — demonstrates ADT lowering with ConstructorName + Match
4. **Trait dictionary passing** — demonstrates monomorphized trait method call

### Fixture Format

Each fixture is a `.cps` file that:
- Is self-contained (no external dependencies)
- Uses only the expanded CPS IR forms
- Includes expected output as a comment
- Can be executed by the interpreter

## TDD Steps

### Step 1: Write Fixtures (Red — they fail before implementation)

**Files:** `crates/ash-interp/tests/fixtures/phase160/`

```
crates/ash-interp/tests/fixtures/phase160/
├── even_odd.cps
├── record_person.cps
├── shape_adt.cps
├── trait_show_dict.cps
└── README.md
```

**even_odd.cps:**
```lisp
;; Mutual recursion: even and odd
;; Expected: even(4) = true

(letrec pair
  (tuple
    (lam [n] k
      (letprim is_zero (eq n 0)
        (if is_zero
          (jump k true {})
          (letprim n-1 (sub n 1)
            (letprim odd_fn (tuple_get 1 pair)
              (call odd_fn [n-1] k {}))))))
    (lam [n] k
      (letprim is_zero (eq n 0)
        (if is_zero
          (jump k false {})
          (letprim n-1 (sub n 1)
            (letprim even_fn (tuple_get 0 pair)
              (call even_fn [n-1] k {})))))))
  (letprim even (tuple_get 0 pair)
    (letcont exit [v] (return v)
      (call even [4] exit {}))))
```

**record_person.cps:**
```lisp
;; Record: Person { name: String, age: Int }
;; Expected: person.name = "Alice"

(letval person (record ((name "Alice") (age 30)))
  (letprim name_val (record_get name person)
    (return name_val)))
```

**shape_adt.cps:**
```lisp
;; Sum type: Shape = Circle { radius: Float } | Rect { width: Float, height: Float }
;; Expected: area(Circle(5.0)) = 78.54...

(letval shape (tuple ((constructor "Circle") 5.0))
  (match shape
    ("Circle"
      (letprim radius (tuple_get 1 shape)
        (letprim r_squared (mul radius radius)
          (letprim pi 3.14159
            (letprim area (mul pi r_squared)
              (return area))))))
    ("Rect"
      (letprim width (tuple_get 1 shape)
        (letprim height (tuple_get 2 shape)
          (letprim area (mul width height)
            (return area))))
    (default
      (trap InvalidShape))))
```

**trait_show_dict.cps:**
```lisp
;; Trait Show<T> with method show(T) -> String
;; Lowered: dictionary record with show field
;; Expected: show_int(42) = "42"

(letval show_dict (record ((show show_int)))
  (letprim show_fn (record_get show show_dict)
    (call show_fn [42] exit {})))
```

### Step 2: Write Execution Tests

**Files:** `crates/ash-interp/tests/task_1606_cps_ir.rs`

```rust
use std::path::PathBuf;

#[test]
fn test_fixture_even_odd() {
    let path = PathBuf::from("tests/fixtures/phase160/even_odd.cps");
    let term = read_term_from_file(&path).unwrap();
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert_eq!(result, Ok(Atom::Bool(true)));
}

#[test]
fn test_fixture_record_person() {
    let path = PathBuf::from("tests/fixtures/phase160/record_person.cps");
    let term = read_term_from_file(&path).unwrap();
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert_eq!(result, Ok(Atom::String("Alice".to_string())));
}

#[test]
fn test_fixture_shape_adt() {
    let path = PathBuf::from("tests/fixtures/phase160/shape_adt.cps");
    let term = read_term_from_file(&path).unwrap();
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    // area = pi * 5.0 * 5.0 = 78.53975
    assert!(matches!(result, Ok(Atom::Float(f)) if (f - 78.53975).abs() < 0.001));
}
```

### Step 3: Verify Fixtures Execute

Run all fixture tests. Fix any issues in the fixtures or implementation.

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
  - cargo test -p ash-interp --test task_1606_cps_ir
  - cargo clippy -p ash-core -p ash-interp --all-targets -- -D warnings
  - cargo fmt --check
checklist:
  - [ ] even_odd fixture executes correctly
  - [ ] record_person fixture executes correctly
  - [ ] shape_adt fixture executes correctly
  - [ ] trait_show_dict fixture executes correctly (or is documented as needing future work)
  - [ ] All fixtures have expected output comments
  - [ ] No clippy warnings
```

## Dependencies for Next Task

- Provides concrete examples for TASK-1607 (operational semantics) and TASK-1608 (reference docs)

## Notes

- The `trait_show_dict.cps` fixture may need to be simplified or deferred if trait dictionary lowering is not yet fully designed.
- Fixtures should be written as `.cps` files (not inline Rust) to demonstrate the actual lowering format.
- The `README.md` in the fixtures directory should explain each pattern and its lowering strategy.
