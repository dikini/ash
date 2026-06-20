# TASK-1610: Extend Value representation and primitive evaluation for structured data

## Status: ✅ Complete

## Description

This is the foundational architecture task for Phase 160. It changes the evaluator to support structured values (records, tuples) that can contain arbitrary values (including lambdas), not just atoms. This requires three coordinated changes:

1. **Change `eval_prim` to return `Value` instead of `Atom`** — so `RecordGet`/`TupleGet` can return structured values
2. **Change `LetPrim` to bind the raw `Value`** — not wrap in `Value::Atom`
3. **Add `Value::Record` and `Value::Tuple` variants** — with `Value` fields (not `Atom`), resolved at construction time

This is the first implementation task because all subsequent Phase 160 tasks depend on this API change.

## Why this change is necessary

The Phase 159 evaluator has a rigid design:
- `eval_prim` returns `Atom` (line 459 of `cps/mod.rs`)
- `LetPrim` wraps the result: `Value::Atom(result)` (line 138)
- `eval_atom` rejects variables bound to non-atom values (line 421-424)

This works for arithmetic but cannot support:
- `RecordGet` returning a lambda (for trait dictionaries)
- `TupleGet` returning a lambda (for mutual recursion)
- Any primitive returning a structured value

## Specification Reference

- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md) §2.2 — Value grammar defines `Record { fields: Vec<(Name, Atom)> }` and `Tuple { elems: Vec<Atom> }` at the IR level
- [PLAN-160](../PLAN-160-CPS-IR-RUNTIME-EXPANSION.md)

**Note:** The spec uses `Atom` for fields because that's the frontend IR grammar. The interpreter's runtime `Value` uses `Value` for fields because atoms are resolved to values during evaluation. This is the same pattern as `LetVal` — the frontend writes `Atom::Var("x")`, but the interpreter resolves it to the bound value.

## Dependencies

- ✅ TASK-1590: Core CPS IR data structures (Atom, Value, Term, Env, HandlerChain)
- ✅ TASK-1591: Core evaluator (LetVal, LetPrim, LetCont, Jump, Call)

## Requirements

### Functional Requirements

1. **Change `eval_prim` signature** from `fn eval_prim(op: PrimOp, args: &[Atom]) -> CpsResult<Atom>` to `fn eval_prim(op: &PrimOp, args: &[Value], env: &Env) -> CpsResult<Value>`
   - `env` is needed for `RecordGet`/`TupleGet` to look up the record/tuple variable and extract fields
   - `args` are now `Vec<Value>` (pre-resolved by `eval_letprim` via `eval_atom_to_value`)
   - Existing arithmetic primitives (`Add`, `Sub`, `Mul`, `Div`, `Eq`, `Gt`, etc.) match on `Value::Atom(...)` and return `Value::Atom(result)`
   - This is a pure refactoring for arithmetic — no behavioral change

2. **Change `eval_letprim` to resolve args to `Value` before calling `eval_prim`**:
   ```rust
   fn eval_letprim(
       name: &Name,
       op: &PrimOp,
       args: &[Atom],
       body: &Term,
       env: &Env,
       chain: &HandlerChain,
   ) -> CpsResult<Atom> {
       let resolved_args: Vec<Value> = args
           .iter()
           .map(|a| eval_atom_to_value(a, env))
           .collect::<CpsResult<Vec<_>>>()?;
       let result = eval_prim(op, &resolved_args, env)?;
       let new_env = env.clone().with_binding(name.clone(), result);
       eval_unchecked(body, &new_env, chain)
   }
   ```

3. **Add `Value::Record` variant** with `Value` fields:
   ```rust
   Record { fields: Vec<(Name, Value)> }
   ```
   - Derives `Clone`, `PartialEq`, `Debug`, `Serialize`, `Deserialize`

4. **Add `Value::Tuple` variant** with `Value` elements:
   ```rust
   Tuple { elems: Vec<Value> }
   ```
   - Derives `Clone`, `PartialEq`, `Debug`, `Serialize`, `Deserialize`

5. **Update `eval_value`** to recursively evaluate record/tuple fields:
   ```rust
   fn eval_value(value: &Value, env: &Env) -> CpsResult<Value> {
       match value {
           Value::Atom(atom) => Ok(Value::Atom(eval_atom(atom, env)?)),
           Value::Record { fields } => {
               let mut new_fields = Vec::new();
               for (name, field_value) in fields {
                   // Recursively evaluate each field value
                   new_fields.push((name.clone(), eval_value(field_value, env)?));
               }
               Ok(Value::Record { fields: new_fields })
           }
           Value::Tuple { elems } => {
               let mut new_elems = Vec::new();
               for elem in elems {
                   // Recursively evaluate each element
                   new_elems.push(eval_value(elem, env)?);
               }
               Ok(Value::Tuple { elems: new_elems })
           }
           Value::Lam {
               params,
               cont,
               body,
               row,
               ..
           } => {
               // Ordinary lambdas capture the current environment and are not recursive.
               Ok(Value::Lam {
                   params: params.clone(),
                   cont: cont.clone(),
                   body: body.clone(),
                   captured_env: env.clone(),
                   rec_binding: None,
                   row: row.clone(),
               })
           }
           Value::Cont { .. } => Ok(value.clone()),
       }
   }
   ```
   
   **Note:** Since `Value::Record` and `Value::Tuple` now store `Value` (not `Atom`), `eval_value` recursively evaluates each field/element. For `Value::Atom(...)` fields, this calls `eval_atom`. For `Value::Lam` fields, it captures the current environment with `rec_binding: None` (ordinary lambdas). For nested `Value::Record`/`Value::Tuple`, it recurses.

6. **Update `eval_atom`** or add `eval_atom_to_value`:
   - The existing `eval_atom` returns `Atom` and rejects non-atom values
   - For `Jump`/`Call` arguments that may be structured values, add a new function:
     ```rust
     fn eval_atom_to_value(atom: &Atom, env: &Env) -> CpsResult<Value> {
         match atom {
             Atom::Var(name) => env.lookup(name)
                 .ok_or_else(|| CpsError::UnboundVariable(name.clone()))
             .cloned(),
             other => Ok(Value::Atom(other.clone())),
         }
     }
     ```
   - Update `eval_jump` to use `eval_atom_to_value` for the argument (so continuations can receive structured values)
   - Update `eval_call` to use `eval_atom_to_value` for arguments

7. **Update `Term::Return` handling**: The current `Return { value }` evaluates the atom via `eval_atom`, which rejects non-atom bindings. For Phase 160, structured values are returned through continuations (Jump/Call), not via `Return`. Tests that construct records/tuples should verify them via `Jump` to a continuation that receives the structured value, or via `LetPrim` with `RecordGet`/`TupleGet` that extracts an atom.
   - Keep `Return` as atom-only for now; the evaluator return type stays `CpsResult<Atom>`
   - Structured value tests should use `Jump` with `eval_atom_to_value` or extract atoms via `RecordGet`/`TupleGet`

### API Changes

- `eval_prim` now takes `(&PrimOp, &[Value], &Env)` and returns `CpsResult<Value>` (was `(PrimOp, &[Atom]) -> CpsResult<Atom>`)
- `eval_letprim` resolves args to `Vec<Value>` via `eval_atom_to_value` before calling `eval_prim`
- `LetPrim` binds the returned `Value` directly (was `Value::Atom(result)`)
- `eval_jump` argument resolution uses `eval_atom_to_value` (was `eval_atom`)
- `eval_call` argument resolution uses `eval_atom_to_value` (was `eval_atom`)

## TDD Steps

### Step 1: Write Tests (Red)

**Files:** `crates/ash-interp/tests/task_1610_cps_ir.rs`

```rust
use ash_core::cps::*;
use ash_interp::cps::{eval_term, eval_checked};

#[test]
fn test_eval_record_construction_with_atoms() {
    // Record with literal atoms — resolves to Value::Atom fields
    // Verify by extracting a field and returning the atom
    let record = Value::Record {
        fields: vec![
            ("x".to_string(), Value::Atom(Atom::Int(42))),
            ("y".to_string(), Value::Atom(Atom::Int(7))),
        ],
    };
    let term = Term::LetVal {
        name: "r".to_string(),
        value: record,
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
fn test_eval_tuple_construction_with_atoms() {
    // Verify by extracting an element and returning the atom
    let tuple = Value::Tuple {
        elems: vec![
            Value::Atom(Atom::Int(10)),
            Value::Atom(Atom::Int(20)),
            Value::Atom(Atom::Int(30)),
        ],
    };
    let term = Term::LetVal {
        name: "t".to_string(),
        value: tuple,
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
fn test_eval_letprim_still_binds_atoms() {
    // Existing arithmetic should still work
    let term = Term::LetPrim {
        name: "y".to_string(),
        op: PrimOp::Add,
        args: vec![Atom::Int(1), Atom::Int(2)],
        body: Box::new(Term::Return { value: Atom::Var("y".to_string()) }),
    };
    let result = eval_checked(&term, &Env::new(), &HandlerChain::new());
    assert_eq!(result, Ok(Atom::Int(3)));
}

#[test]
fn test_eval_jump_with_structured_value() {
    // Jump with a record argument — continuation receives structured value
    // Then extract atom from it and return
    let record = Value::Record {
        fields: vec![
            ("x".to_string(), Value::Atom(Atom::Int(42))),
        ],
    };
    let term = Term::LetVal {
        name: "r".to_string(),
        value: record,
        body: Box::new(Term::LetCont {
            name: "k".to_string(),
            param: "v".to_string(),
            cont_body: Box::new(Term::LetPrim {
                name: "x_val".to_string(),
                op: PrimOp::RecordGet("x".to_string()),
                args: vec![Atom::Var("v".to_string())],
                body: Box::new(Term::Return { value: Atom::Var("x_val".to_string()) }),
            }),
            body: Box::new(Term::Jump {
                cont: ContRef::Label("k".to_string()),
                arg: Atom::Var("r".to_string()),
                row: EffectRow::default(),
            }),
        }),
    };
    let result = eval_checked(&term, &Env::new(), &HandlerChain::new());
    assert_eq!(result, Ok(Atom::Int(42)));
}
```

### Step 2: Implement (Green)

**Implementation sequencing:**
1. Add `Value::Record` and `Value::Tuple` variants to `crates/ash-core/src/cps.rs`
2. Fix compiler errors from new variants (update any existing code that constructs `Value` exhaustively)
3. Verify serde roundtrip: `cargo test -p ash-core --test task_1599_cps_ir`
4. Only then proceed to evaluator changes

**Files:**
- `crates/ash-core/src/cps.rs` — Add `Value::Record` and `Value::Tuple` variants
- `crates/ash-interp/src/cps/mod.rs` — Change `eval_prim`, `eval_letprim`, `eval_jump`, `eval_call`, `eval_value`

### Step 3: Integration

- Ensure all existing Phase 159 tests still pass
- Verify no behavioral changes for arithmetic primitives

### Step 4: Property Tests

**File:** `crates/ash-interp/tests/task_1610_cps_ir.rs`

```rust
#[test]
fn test_roundtrip_record_value() {
    let record = Value::Record {
        fields: vec![
            ("x".to_string(), Value::Atom(Atom::Int(42))),
        ],
    };
    let s = serde_lexpr::to_string(&record).unwrap();
    let parsed: Value = serde_lexpr::from_str(&s).unwrap();
    assert_eq!(record, parsed);
}

#[test]
fn test_roundtrip_tuple_value() {
    let tuple = Value::Tuple {
        elems: vec![Value::Atom(Atom::Int(1)), Value::Atom(Atom::Int(2))],
    };
    let s = serde_lexpr::to_string(&tuple).unwrap();
    let parsed: Value = serde_lexpr::from_str(&s).unwrap();
    assert_eq!(tuple, parsed);
}
```

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
  - cargo test -p ash-core -p ash-interp --test task_1610_cps_ir
  - cargo test -p ash-interp --test task_1590_cps_ir  # existing tests still pass
  - cargo test -p ash-interp --test task_1591_cps_ir  # existing tests still pass
  - cargo test -p ash-interp --test task_1596_cps_ir  # existing tests still pass
  - cargo clippy -p ash-core -p ash-interp --all-targets -- -D warnings
  - cargo fmt --check
checklist:
  - [ ] Record construction test passes
  - [ ] Tuple construction test passes
  - [ ] Existing arithmetic primitives still work (LetPrim binds atoms correctly)
  - [ ] Jump with structured value argument works
  - [ ] Call with structured value arguments works
  - [ ] Round-trip serialization test passes
  - [ ] All existing Phase 159 tests still pass
  - [ ] No clippy warnings
  - [ ] CHANGELOG.md entry staged
```

## Dependencies for Next Task

- Provides `Value::Record` and `Value::Tuple` for TASK-1611 (field access primitives)
- Provides `eval_prim -> Value` for TASK-1611 (RecordGet/TupleGet can return structured values)
- Provides `eval_atom_to_value` for TASK-1614 (mutual recursion via tuple-of-lambdas)

## Notes

- **This is a breaking API change to the evaluator.** All existing tests must still pass.
- The spec's `Atom` field type is the IR grammar; the interpreter's `Value` field type is the runtime representation. This is not a spec violation — it's the same pattern as `LetVal` (frontend writes `Atom::Var`, interpreter resolves to `Value`).
- `eval_atom` is kept for places that genuinely need atoms (e.g., `If` condition). New code paths use `eval_atom_to_value` where structured values are expected.
- Serde serialization for new variants is verified in this task; TASK-1615 adds round-trip file tests.
