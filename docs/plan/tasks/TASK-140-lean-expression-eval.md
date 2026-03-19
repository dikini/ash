# TASK-140: Expression Evaluation

## Status: ✅ Complete

## Description

Implement expression evaluation in Lean 4 following the big-step operational semantics from SPEC-004. This includes literals, variables, constructors, and tuples with proper effect tracking.

## Specification Reference

- SPEC-021: Lean Reference - Section 6.1 (Expression Evaluation)
- SPEC-004: Operational Semantics - Section 5.1 (Constructor Evaluation)
- SPEC-004: Operational Semantics - Section 3 (Big-Step Judgment)

## Requirements

### Functional Requirements

1. Implement `eval` function with big-step semantics:
   ```
   Γ ⊢ e ⇓ v, ε
   ```
2. Handle expression types:
   - `literal` - returns value with epistemic effect
   - `variable` - looks up in environment
   - `constructor` - builds variant with epistemic effect
   - `tuple` - evaluates elements, combines effects
3. Propagate errors using `Except EvalError`
4. Track effects through evaluation

### Property Requirements

```lean
-- Constructor purity (SPEC-004 Section 5.1)
prop_constructor_pure(env, name, fields) = 
  match eval env (.constructor name fields) with
  | .ok result => result.effect = .epistemic
  | .error _ => true

-- Literal is pure
prop_literal_pure(v) = 
  match eval Env.empty (.literal v) with
  | .ok result => result.effect = .epistemic
  | .error _ => true

-- Variable lookup fails on unbound
prop_unbound_variable(x) = 
  x ∉ env → eval env (.variable x) = .error (.unboundVariable x)
```

## TDD Steps

### Step 1: Define eval Skeleton (Red)

**File**: `lean_reference/Ash/Eval/Expr.lean`

```lean
import Ash.Core.AST
import Ash.Core.Environment

namespace Ash.Eval

open Ash

-- Forward declaration for mutual recursion
def eval (env : Env) (expr : Expr) : Except EvalError EvalResult :=
  match expr with
  | .literal v => pure { value := v, effect := .epistemic }
  | .variable x => evalVariable env x
  | .constructor name fields => evalConstructor env name fields
  | .tuple elements => evalTuple env elements
  | .match scrutinee arms => throw (.unboundVariable "not implemented")
  | .if_let pattern expr then_branch else_branch => 
      throw (.unboundVariable "not implemented")

end Ash.Eval
```

**Test** (in same file):
```lean
#eval eval Env.empty (.literal (Value.int 42))
-- Expected: ok { value := int 42, effect := epistemic }
```

### Step 2: Implement Literal Evaluation (Green)

Already implemented in skeleton. Verify with test:

```lean
def testLiteral : IO Unit := do
  let result := eval Env.empty (.literal (Value.string "hello"))
  match result with
  | .ok r => IO.println s!"Value: {r.value}, Effect: {r.effect}"
  | .error e => IO.println s!"Error: {e}"

-- Expected: Value: string "hello", Effect: epistemic
```

### Step 3: Implement Variable Lookup (Green)

```lean
def evalVariable (env : Env) (x : String) : Except EvalError EvalResult :=
  match env.lookup x with
  | none => throw (.unboundVariable x)
  | some v => pure { value := v, effect := .epistemic }
```

Update eval:
```lean
  | .variable x => evalVariable env x
```

**Test**:
```lean
def testVariable : IO Unit := do
  let env := Env.empty.bind "x" (Value.int 42)
  
  -- Bound variable
  let r1 := eval env (.variable "x")
  IO.println s!"Bound: {r1}"
  
  -- Unbound variable
  let r2 := eval env (.variable "y")
  IO.println s!"Unbound: {r2}"

-- Expected:
-- Bound: ok { value := int 42, effect := epistemic }
-- Unbound: error (unboundVariable "y")
```

### Step 4: Implement Constructor Evaluation (Green)

Per SPEC-004 Section 5.1 (CONSTRUCTOR-ENUM):

```lean
def evalConstructor (env : Env) (name : String)
    (fields : List (String × Expr)) : Except EvalError EvalResult := do
  let mut field_values := []
  let mut accumulated_effect := Effect.epistemic
  
  for (field_name, field_expr) in fields do
    let result ← eval env field_expr
    field_values := (field_name, result.value) :: field_values
    accumulated_effect := accumulated_effect.join result.effect
  
  pure {
    value := .variant "" name field_values.reverse,
    effect := .epistemic  -- Constructors are pure per SPEC-004
  }
```

Update eval:
```lean
  | .constructor name fields => evalConstructor env name fields
```

**Test**:
```lean
def testConstructor : IO Unit := do
  -- Some { value = 42 }
  let ctor := Expr.constructor "Some" [("value", .literal (Value.int 42))]
  let r := eval Env.empty ctor
  IO.println s!"Constructor: {r}"
  
  -- Point { x = 1, y = 2 }
  let point := Expr.constructor "Point" [
    ("x", .literal (Value.int 1)),
    ("y", .literal (Value.int 2))
  ]
  let r2 := eval Env.empty point
  IO.println s!"Point: {r2}"

-- Expected:
-- Constructor: ok { value := variant "" "Some" [("value", int 42)], effect := epistemic }
-- Point: ok { value := variant "" "Point" [("x", int 1), ("y", int 2)], effect := epistemic }
```

### Step 5: Implement Tuple Evaluation (Green)

Per SPEC-004 Section 5.1 (CONSTRUCTOR-TUPLE):

```lean
def evalTuple (env : Env) (elements : List Expr) : Except EvalError EvalResult := do
  let mut values := []
  let mut accumulated_effect := Effect.epistemic
  
  for elem in elements do
    let result ← eval env elem
    values := result.value :: values
    accumulated_effect := accumulated_effect.join result.effect
  
  pure {
    value := .tuple values.reverse,
    effect := accumulated_effect
  }
```

Update eval:
```lean
  | .tuple elements => evalTuple env elements
```

**Test**:
```lean
def testTuple : IO Unit := do
  -- (1, "hello", true)
  let tup := Expr.tuple [
    .literal (Value.int 1),
    .literal (Value.string "hello"),
    .literal (Value.bool true)
  ]
  let r := eval Env.empty tup
  IO.println s!"Tuple: {r}"

-- Expected:
-- Tuple: ok { value := tuple [int 1, string "hello", bool true], effect := epistemic }
```

### Step 6: Property Tests for Constructor Purity (Green)

```lean
-- SPEC-004 Section 5.1: Constructors are pure (epistemic effect)
#test ∀ (name : String) (fields : List (String × Expr)),
  name ≠ "" → fields.length < 5 →
  match eval Env.empty (.constructor name fields) with
  | .ok result => result.effect = .epistemic
  | .error _ => true

-- Literals are always pure
#test ∀ (v : Value),
  match eval Env.empty (.literal v) with
  | .ok result => result.effect = .epistemic
  | .error _ => true
```

### Step 7: Test Nested Evaluation (Green)

```lean
def testNested : IO Unit := do
  -- Constructor with variable field
  let env := Env.empty.bind "x" (Value.int 100)
  let nested := Expr.constructor "Wrapper" [
    ("inner", .constructor "Some" [
      ("value", .variable "x")
    ])
  ]
  let r := eval env nested
  IO.println s!"Nested: {r}"

-- Expected:
-- Nested: ok { 
--   value := variant "" "Wrapper" [("inner", variant "" "Some" [("value", int 100)])],
--   effect := epistemic 
-- }
```

### Step 8: Integration with Main (Green)

**File**: `lean_reference/Main.lean`

```lean
import Ash

def runEvalTests : IO Unit := do
  IO.println "=== Expression Evaluation Tests ==="
  
  -- Test literal
  let lit := Expr.literal (Value.int 42)
  IO.println s!"Literal: {eval Env.empty lit}"
  
  -- Test variable
  let env := Env.empty.bind "x" (Value.string "test")
  let var := Expr.variable "x"
  IO.println s!"Variable: {eval env var}"
  
  -- Test constructor
  let ctor := Expr.constructor "Some" [("value", .literal (Value.int 42))]
  IO.println s!"Constructor: {eval Env.empty ctor}"
  
  -- Test tuple
  let tup := Expr.tuple [.literal (Value.int 1), .literal (Value.int 2)]
  IO.println s!"Tuple: {eval Env.empty tup}"
  
  -- Test error
  let unbound := Expr.variable "undefined"
  IO.println s!"Unbound: {eval Env.empty unbound}"

def main : IO Unit := do
  runEvalTests
```

**Run**:
```bash
lake exe ash_ref
# Expected output:
# === Expression Evaluation Tests ===
# Literal: ok { value := int 42, effect := epistemic }
# Variable: ok { value := string "test", effect := epistemic }
# Constructor: ok { value := variant ... "Some" ..., effect := epistemic }
# Tuple: ok { value := tuple [int 1, int 2], effect := epistemic }
# Unbound: error (unboundVariable "undefined")
```

## Completion Checklist

- [ ] `eval` function with big-step semantics
- [ ] `literal` evaluation (epistemic effect)
- [ ] `variable` evaluation with environment lookup
- [ ] `constructor` evaluation per SPEC-004 Section 5.1
- [ ] `tuple` evaluation with effect accumulation
- [ ] Error propagation via `Except EvalError`
- [ ] Constructor purity property tests
- [ ] Literal purity property tests
- [ ] Nested expression evaluation tests
- [ ] All expression types have test cases
- [ ] Documentation comments referencing SPEC-004

## Self-Review Questions

1. **Spec adherence**: Does eval match SPEC-004 big-step semantics?
   - Yes: Γ ⊢ e ⇓ v, ε form with context and effect tracking

2. **Constructor purity**: Are constructors epistemic?
   - Yes: Per SPEC-004 Section 5.1, constructors are pure

3. **Error handling**: Are errors properly propagated?
   - Yes: Using `Except` monad for error propagation

## Estimated Effort

16 hours

## Dependencies

- TASK-138 (AST Types)
- TASK-139 (Environment)

## Blocked By

- TASK-139

## Blocks

- TASK-141 (Pattern Match)
- TASK-142 (Match Expr)
- TASK-143 (If-Let)
- TASK-145 (Differential Testing)
