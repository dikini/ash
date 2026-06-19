# TASK-1613: Add Match term for pattern dispatch

## Status: 📝 Planned

## Description

Add `Term::Match` for multi-way dispatch on constructor tags. This is the runtime mechanism for pattern matching on sum types. The frontend lowers pattern matching to a `Match` term that extracts the constructor tag from a tuple and dispatches to the appropriate arm.

`Term::Match` is chosen over `PrimOp::MatchTag` because match dispatch is control flow (selecting a branch), not a pure primitive operation. A dedicated term also allows for future extension (guards, nested patterns).

## Specification Reference

- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md) §2.3 — Term grammar
- [PLAN-160](../PLAN-160-CPS-IR-RUNTIME-EXPANSION.md)

## Dependencies

- ✅ TASK-1590: Core CPS IR data structures
- ✅ TASK-1612: ConstructorName atom variant (must be complete)

## Requirements

### Functional Requirements

1. Add `Term::Match` variant to the `Term` enum (rejected `PrimOp::MatchTag` — match dispatch is control flow, not a pure primitive):
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

**Files:** `crates/ash-interp/tests/task_1613_cps_ir.rs`

```rust
use ash_core::cps::*;
use ash_interp::cps::eval_checked;

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
            default: None,
        }),
    };
    let result = eval_checked(&term, &Env::new(), &HandlerChain::new());
    assert_eq!(result, Ok(Atom::Int(1)));
}

#[test]
fn test_eval_match_default() {
    let term = Term::LetVal {
        name: "shape".to_string(),
        value: Value::Tuple {
            elems: vec![
                Value::Atom(Atom::ConstructorName("Triangle".to_string())),
                Value::Atom(Atom::Int(3)),
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
    let result = eval_checked(&term, &Env::new(), &HandlerChain::new());
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
            let tag_value = elems.first().ok_or_else(|| 
                CpsError::Trap(TrapReason::Custom("empty tuple in match".to_string()))
            )?;
            match tag_value {
                Value::Atom(Atom::ConstructorName(name)) => {
                    for (arm_tag, body) in arms {
                        if arm_tag == name {
                            return eval_unchecked(body, env, chain);
                        }
                    }
                    if let Some(default_body) = default {
                        return eval_unchecked(default_body, env, chain);
                    }
                    Err(CpsError::Trap(TrapReason::Custom("no matching arm".to_string())))
                }
                _ => Err(CpsError::Trap(TrapReason::Custom(
                    "match scrutinee tag is not a ConstructorName".to_string()))),
            }
        }
        _ => Err(CpsError::Trap(TrapReason::Custom(
            "match scrutinee is not a tuple".to_string()))),
    }
}
```

**Note:** The scrutinee is a `Value::Tuple` where `elems` are `Vec<Value>`. The first element is the tag, which is `Value::Atom(Atom::ConstructorName(...))`. The error cases use `Trap` with descriptive messages rather than `InvalidPrimArgs` (which is for primitive operations).

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
  - cargo test -p ash-core -p ash-interp --test task_1613_cps_ir
  - cargo clippy -p ash-core -p ash-interp --all-targets -- -D warnings
  - cargo fmt --check
checklist:
  - [ ] Matching arm executes correctly
  - [ ] Default arm executes when no match
  - [ ] No match + no default returns error
  - [ ] Non-tuple scrutinee returns error
  - [ ] No clippy warnings
  - [ ] CHANGELOG.md entry staged
```

## Dependencies for Next Task

- Provides match dispatch for TASK-1616 (speculative fixtures)

## Notes

- The payload extraction (e.g., `radius` from `Circle(radius)`) is done by the frontend using `TupleGet` before or within the match arm body. The match itself only dispatches on the tag.
- This is intentionally minimal. Full pattern matching with nested patterns, guards, and variable bindings is a frontend concern.
