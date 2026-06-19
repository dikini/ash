# TASK-1605: Update S-expression parser/serializer for new forms

## Status: 📝 Planned

## Description

Update the S-expression parser and serializer (from TASK-1599/TASK-1600) to handle the new CPS IR forms: `Value::Record`, `Value::Tuple`, `Atom::ConstructorName`, `Term::Match`, and the updated `PrimOp::RecordGet`/`TupleGet`.

## Specification Reference

- [PLAN-160](../PLAN-160-CPS-IR-RUNTIME-EXPANSION.md)
- TASK-1599: S-expression parser hardening
- TASK-1600: Record/Tuple values
- TASK-1601: Field access primitives
- TASK-1602: Constructor tags
- TASK-1603: Match dispatch

## Dependencies

- ✅ TASK-1599: S-expression parser hardening
- ✅ TASK-1600: Record/Tuple values (must be complete)
- ✅ TASK-1601: Field access primitives (must be complete)
- ✅ TASK-1602: Constructor tags (must be complete)
- ✅ TASK-1603: Match dispatch (must be complete)

## Requirements

### Functional Requirements

1. Parser recognizes new S-expression syntax:
   - `(record ((name value) ...))` → `Value::Record`
   - `(tuple (value ...))` → `Value::Tuple`
   - `(constructor "Name")` → `Atom::ConstructorName`
   - `(record_get name record)` → `PrimOp::RecordGet`
   - `(tuple_get index tuple)` → `PrimOp::TupleGet`
   - `(match scrutinee ("Tag1" body) ("Tag2" body) ...)` → `Term::Match`
   - `(match scrutinee ("Tag1" body) ... (default default_body))` → `Term::Match` with default
2. Serializer outputs the same syntax for all new forms
3. Round-trip tests: parse → serialize → parse produces identical IR

### S-expression Syntax Reference

```lisp
;; Record value
(record ((x 42) (y "hello")))

;; Tuple value
(tuple (1 2 3))

;; Constructor name atom
(constructor "Circle")

;; Record field access
(letprim x_val (record_get x r)
  ...)

;; Tuple element access
(letprim second (tuple_get 1 t)
  ...)

;; Match dispatch
(match shape
  ("Circle" (return 1))
  ("Rect" (return 2))
  (default (return 99)))
```

## TDD Steps

### Step 1: Write Tests (Red)

**Files:** `crates/ash-interp/tests/task_1605_cps_ir.rs`

```rust
#[test]
fn test_roundtrip_record() {
    let term = Term::LetVal {
        name: "r".to_string(),
        value: Value::Record {
            fields: vec![
                ("x".to_string(), Value::Atom(Atom::Int(42))),
                ("y".to_string(), Value::Atom(Atom::String("hello".to_string()))),
            ],
        },
        body: Box::new(Term::Return { value: Atom::Var("r".to_string()) }),
    };
    let sexpr = serialize_term(&term);
    let parsed = parse_term(&sexpr).unwrap();
    assert_eq!(term, parsed);
}

#[test]
fn test_roundtrip_match() {
    let term = Term::LetVal {
        name: "shape".to_string(),
        value: Value::Tuple {
            elems: vec![
                Value::Atom(Atom::ConstructorName("Circle".to_string())),
                Value::Atom(Atom::Float(5.0)),
            ],
        },
        body: Box::new(Term::Match {
            scrutinee: Atom::Var("shape".to_string()),
            arms: vec![
                ("Circle".to_string(), Box::new(Term::Return { value: Atom::Int(1) })),
                ("Rect".to_string(), Box::new(Term::Return { value: Atom::Int(2) })),
            ],
            default: Some(Box::new(Term::Return { value: Atom::Int(99) })),
        }),
    };
    let sexpr = serialize_term(&term);
    let parsed = parse_term(&sexpr).unwrap();
    assert_eq!(term, parsed);
}
```

### Step 2: Implement (Green)

**Files:** `crates/ash-core/src/sexp.rs` (or wherever parser/serializer lives)

Add parser cases for new syntax forms. Add serializer cases for new IR variants.

### Step 3: Integration

- Ensure all new forms serialize to valid S-expressions
- Ensure parser rejects malformed syntax (e.g., `record_get` with missing field name)

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
  - cargo test -p ash-core -p ash-interp --test task_1605_cps_ir
  - cargo test -p ash-core -p ash-interp --test task_1599_cps_ir
  - cargo clippy -p ash-core -p ash-interp --all-targets -- -D warnings
  - cargo fmt --check
checklist:
  - [ ] Record round-trip test passes
  - [ ] Tuple round-trip test passes
  - [ ] Constructor name round-trip test passes
  - [ ] Match round-trip test passes
  - [ ] Existing round-trip tests still pass
  - [ ] No clippy warnings
```

## Dependencies for Next Task

- Provides S-expression support for TASK-1606 (speculative fixtures)

## Notes

- The parser/serializer code location may vary. Check `crates/ash-core/src/sexp.rs` or similar.
- If the parser uses a recursive descent approach, add new parse functions for each form.
- If the serializer uses pattern matching on the IR, add new match arms for each variant.
