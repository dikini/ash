# TASK-1603: Add Match dispatch for pattern matching

## Status: 📝 Planned

## Description

Add `PrimOp::MatchTag` primitive operation for multi-way dispatch on constructor tags. This is the runtime mechanism for pattern matching on sum types. The frontend lowers pattern matching to a sequence of `If` + `Eq` comparisons or a single `MatchTag` primitive.

## Specification Reference

- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md) §2.3 — Term grammar
- [PLAN-160](../PLAN-160-CPS-IR-RUNTIME-EXPANSION.md)

## Dependencies

- ✅ TASK-1590: Core CPS IR data structures
- ✅ TASK-1602: ConstructorName atom variant (must be complete)

## Requirements

### Functional Requirements

1. Add `PrimOp::MatchTag { scrutinee: Atom, arms: Vec<(Name, Term)> }` or implement as a `Term::Match` variant
2. Decision: Use `Term::Match` for clarity and extensibility:
   ```rust
   Term::Match {
       scrutinee: Atom,
       arms: Vec<(Name, Name, Box<Term>)>, // (constructor_tag, binding_name, body)
       default: Option<Box<Term>>,
   }
   ```
   Actually, simpler: the payload is already bound. The match just dispatches on tag:
   ```rust
   Term::Match {
       scrutinee: Atom,           // variable bound to the tuple (tag, payload...)
       arms: Vec<(Name, Box<Term>)>, // (constructor_tag, body)
       default: Option<Box<Term>>,
   }
   ```
3. Evaluator extracts tag via `TupleGet(0)`, compares to each arm's tag, executes matching body
4. If no arm matches and no default, trap with `MatchError`

### Property Requirements

- Match with matching arm executes that arm's body
- Match with no matching arm and no default returns error
- Match with default executes default when no arm matches
- Scrutinee must be a tuple with ConstructorName at index 0

## TDD Steps

### Step 1: Write Tests (Red)

**Files:** `crates/ash-interp/tests/task_1603_cps_ir.rs`

```rust
#[test]
fn test_eval_match_circle() {
    // let shape = (tuple ("Circle" 5.0)) in
    //   match shape with
    //     "Circle" -> (return 1)
    //     "Rect" -> (return 2)
    let term = Term::LetVal {
        name: "shape".to_string(),
        value: Value::Tuple {
            elems: vec![
                Atom::ConstructorName("Circle".to_string()),
                Atom::Float(5.0),
            ],
        },
        body: Box::new(Term::Match {
            scrutinee: Atom::Var("shape".to_string()),
            arms: vec![
                ("Circle".to_string(), Box::new(Term::Return { value: Atom::Int(1) })),
                ("Rect".to_string(), Box::new(Term::Return { value: Atom::Int(2) })),
            ],
            default: None,
        }),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert_eq!(result, Ok(Atom::Int(1)));
}

#[test]
fn test_eval_match_default() {
    let term = Term::LetVal {
        name: "shape".to_string(),
        value: Value::Tuple {
            elems: vec![
                Atom::ConstructorName("Triangle".to_string()),
                Atom::Int(3),
            ],
        },
        body: Box::new(Term::Match {
            scrutinee: Atom::Var("shape".to_string()),
            arms: vec![
                ("Circle".to_string(), Box::new(Term::Return { value: Atom::Int(1) })),
            ],
            default: Some(Box::new(Term::Return { value: Atom::Int(99) })),
        }),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert_eq!(result, Ok(Atom::Int(99)));
}
```

### Step 2: Implement (Green)

**Files:** `crates/ash-core/src/cps.rs`, `crates/ash-interp/src/cps.rs`

Add to `Term` enum:

```rust
Match {
    scrutinee: Atom,
    arms: Vec<(Name, Box<Term>)>,
    default: Option<Box<Term>>,
}
```

Add to `eval_term`:

```rust
Term::Match { scrutinee, arms, default } => {
    let scrut_value = resolve_value(scrutinee, env)?;
    match scrut_value {
        Value::Tuple { elems } => {
            let tag = elems.first().ok_or_else(|| CpsError::InvalidPrimArgs(PrimOp::Eq, vec![]))?;
            match tag {
                Atom::ConstructorName(name) => {
                    for (arm_tag, body) in arms {
                        if &arm_tag == name {
                            return eval_term(body, env, chain);
                        }
                    }
                    if let Some(default_body) = default {
                        return eval_term(default_body, env, chain);
                    }
                    Err(CpsError::Trap(TrapReason::Custom("no matching arm".to_string())))
                }
                _ => Err(CpsError::InvalidPrimArgs(PrimOp::Eq, vec![tag.clone()])),
            }
        }
        _ => Err(CpsError::InvalidPrimArgs(PrimOp::Eq, vec![])),
    }
}
```

### Step 3: Integration

- Wire through `eval_term` match arm
- Update serde serialization

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
  - cargo test -p ash-core -p ash-interp --test task_1603_cps_ir
  - cargo clippy -p ash-core -p ash-interp --all-targets -- -D warnings
  - cargo fmt --check
checklist:
  - [ ] Matching arm executes correctly
  - [ ] Default arm executes when no match
  - [ ] No match + no default returns error
  - [ ] Non-tuple scrutinee returns error
  - [ ] No clippy warnings
```

## Dependencies for Next Task

- Provides match dispatch for TASK-1606 (speculative fixtures)

## Notes

- The payload extraction (e.g., `radius` from `Circle(radius)`) is done by the frontend using `TupleGet` before or within the match arm body. The match itself only dispatches on the tag.
- This is intentionally minimal. Full pattern matching with nested patterns, guards, and variable bindings is a frontend concern.
