# TASK-143: If-Let Expression

## Status: 🟡 Ready to Start

## Description

Implement if-let expression evaluation in Lean 4. This provides syntactic sugar for conditional pattern matching with a then-branch (on match success) and else-branch (on match failure).

## Specification Reference

- SPEC-021: Lean Reference - Section 6.5 (If-Let Expression)
- SPEC-004: Operational Semantics - Section 5.3 (IF-LET rules)

## Requirements

### Functional Requirements

1. Implement `evalIfLet` function:
   ```
   evalIfLet : Env → Pattern → Expr → Expr → Expr → Except EvalError EvalResult
   ```
2. Evaluate the expression being matched
3. Attempt to match result against pattern
4. On success: evaluate then-branch with pattern bindings
5. On failure: evaluate else-branch with original environment
6. Accumulate effects from all evaluated sub-expressions

### Property Requirements

```lean
-- If-let success case
prop_if_let_success(pattern, expr, then_branch) = 
  matchPattern pattern (eval expr).value = some bindings →
  evalIfLet env pattern expr then_branch else_branch = 
    eval (env ++ bindings) then_branch

-- If-let failure case
prop_if_let_failure(pattern, expr, else_branch) = 
  matchPattern pattern (eval expr).value = none →
  evalIfLet env pattern expr then_branch else_branch = 
    eval env else_branch

-- Effect accumulation
prop_if_let_effect = 
  result.effect = expr_effect.join (then_effect or else_effect)
```

## TDD Steps

### Step 1: Define evalIfLet (Red)

**File**: `lean_reference/Ash/Eval/IfLet.lean`

```lean
import Ash.Core.AST
import Ash.Core.Environment
import Ash.Eval.Pattern
import Ash.Eval.Expr

namespace Ash.Eval

open Ash

def evalIfLet (env : Env) (pattern : Pattern) (expr : Expr)
    (then_branch : Expr) (else_branch : Expr) : Except EvalError EvalResult := do
  -- Evaluate the expression being matched
  let expr_result ← eval env expr
  
  -- Try to match pattern
  match matchPattern pattern expr_result.value with
  | some bindings =>
      -- Success: evaluate then-branch with bindings
      let newEnv := mergeEnvs env bindings
      let then_result ← eval newEnv then_branch
      pure {
        value := then_result.value,
        effect := expr_result.effect.join then_result.effect
      }
  | none =>
      -- Failure: evaluate else-branch without bindings
      let else_result ← eval env else_branch
      pure {
        value := else_result.value,
        effect := expr_result.effect.join else_result.effect
      }

end Ash.Eval
```

**Test**:
```lean
def testIfLet : IO Unit := do
  -- if let x = 42 then x else 0
  let r := evalIfLet Env.empty 
    (.variable "x") 
    (.literal (Value.int 42))
    (.variable "x")
    (.literal (Value.int 0))
  IO.println s!"If-let (success): {r}"

-- Expected: ok { value := int 42, effect := epistemic }
```

### Step 2: Test If-Let Success with Variant (Green)

```lean
def testIfLetVariantSuccess : IO Unit := do
  -- if let Some { value = x } = Some { value = 42 } then x else 0
  let expr := Expr.constructor "Some" [("value", .literal (Value.int 42))]
  let pattern := Pattern.variant "Some" [("value", .variable "x")]
  
  let r := evalIfLet Env.empty pattern expr (.variable "x") (.literal (Value.int 0))
  IO.println s!"If-let variant (success): {r}"

-- Expected: ok { value := int 42, effect := epistemic }
```

### Step 3: Test If-Let Failure (Green)

```lean
def testIfLetFailure : IO Unit := do
  -- if let Some { value = x } = None then x else 0
  let expr := Expr.constructor "None" []
  let pattern := Pattern.variant "Some" [("value", .variable "x")]
  
  let r := evalIfLet Env.empty pattern expr (.variable "x") (.literal (Value.int 0))
  IO.println s!"If-let (failure): {r}"
  
  -- Test with literal pattern
  -- if let 42 = 43 then "yes" else "no"
  let r2 := evalIfLet Env.empty 
    (.literal (Value.int 42))
    (.literal (Value.int 43))
    (.literal (Value.string "yes"))
    (.literal (Value.string "no"))
  IO.println s!"If-let literal (failure): {r2}"

-- Expected:
-- If-let (failure): ok { value := int 0, effect := epistemic }
-- If-let literal (failure): ok { value := string "no", effect := epistemic }
```

### Step 4: Test Nested If-Let (Green)

```lean
def testNestedIfLet : IO Unit := do
  -- if let Some { value = x } = Some { value = 42 } then
  --   if let 42 = x then "matched" else "wrong value"
  -- else
  --   "none"
  
  let outerExpr := Expr.constructor "Some" [("value", .literal (Value.int 42))]
  let outerPattern := Pattern.variant "Some" [("value", .variable "x")]
  
  let innerIfLet := Expr.if_let
    (.literal (Value.int 42))
    (.variable "x")
    (.literal (Value.string "matched"))
    (.literal (Value.string "wrong value"))
  
  let r := evalIfLet Env.empty outerPattern outerExpr innerIfLet (.literal (Value.string "none"))
  IO.println s!"Nested if-let: {r}"

-- Expected: ok { value := string "matched", effect := epistemic }
```

### Step 5: Test If-Let with Environment (Green)

```lean
def testIfLetWithEnv : IO Unit := do
  -- let y = 100 in
  -- if let Some { value = x } = Some { value = 42 } then y else 0
  -- (y from outer env, x from pattern)
  
  let env := Env.empty.bind "y" (Value.int 100)
  let expr := Expr.constructor "Some" [("value", .literal (Value.int 42))]
  let pattern := Pattern.variant "Some" [("value", .variable "x")]
  
  -- Use y from outer env, ignore x from pattern
  let r := evalIfLet env pattern expr (.variable "y") (.literal (Value.int 0))
  IO.println s!"If-let with env: {r}"
  
  -- Use x from pattern
  let r2 := evalIfLet env pattern expr (.variable "x") (.literal (Value.int 0))
  IO.println s!"If-let with pattern binding: {r2}"

-- Expected:
-- If-let with env: ok { value := int 100, effect := epistemic }
-- If-let with pattern binding: ok { value := int 42, effect := epistemic }
```

### Step 6: Test Effect Accumulation (Green)

```lean
def testIfLetEffects : IO Unit := do
  -- Effects accumulate: expr_effect ⊔ then_effect (or else_effect)
  -- For now all effects are epistemic
  
  let expr := Expr.tuple [.literal (Value.int 1), .literal (Value.int 2)]
  let pattern := Pattern.tuple [.variable "a", .variable "b"]
  
  let r := evalIfLet Env.empty pattern expr 
    (.tuple [.variable "a", .variable "b"])
    (.literal (Value.int 0))
  
  IO.println s!"If-let effects: {r}"

-- Expected: ok { value := tuple [int 1, int 2], effect := epistemic }
```

### Step 7: Property Tests (Green)

```lean
-- If-let with wildcard always takes then branch
#test ∀ (v : Value) (then_body else_body : Expr),
  match evalIfLet Env.empty .wildcard (.literal v) then_body else_body with
  | .ok result => 
      match eval Env.empty then_body with
      | .ok then_result => result.value = then_result.value
      | _ => true
  | _ => false

-- If-let with literal succeeds when equal
#test ∀ (v : Value),
  match evalIfLet Env.empty (.literal v) (.literal v)
    (.literal (Value.bool true))
    (.literal (Value.bool false))
  with
  | .ok result => result.value = Value.bool true
  | _ => false

-- If-let with literal fails when not equal
#test ∀ (v1 v2 : Value),
  v1 ≠ v2 →
  match evalIfLet Env.empty (.literal v1) (.literal v2)
    (.literal (Value.bool true))
    (.literal (Value.bool false))
  with
  | .ok result => result.value = Value.bool false
  | _ => false
```

### Step 8: Integration with Main Eval (Green)

Update `Ash/Eval/Expr.lean`:

```lean
import Ash.Eval.IfLet

-- In eval function:
  | .if_let pattern expr then_branch else_branch =>
      evalIfLet env pattern expr then_branch else_branch
```

**Integration Test**:
```lean
def runIfLetTests : IO Unit := do
  IO.println "\n=== If-Let Expression Tests ==="
  
  -- Simple variable pattern
  let r1 := eval Env.empty (.if_let
    (.variable "x")
    (.literal (Value.int 42))
    (.variable "x")
    (.literal (Value.int 0))
  )
  IO.println s!"Variable: {r1}"
  
  -- Variant success
  let r2 := eval Env.empty (.if_let
    (.variant "Some" [("value", .variable "x")])
    (.constructor "Some" [("value", .literal (Value.int 42))])
    (.variable "x")
    (.literal (Value.int 0))
  )
  IO.println s!"Variant success: {r2}"
  
  -- Variant failure
  let r3 := eval Env.empty (.if_let
    (.variant "Some" [("value", .variable "x")])
    (.constructor "None" [])
    (.variable "x")
    (.literal (Value.int 0))
  )
  IO.println s!"Variant failure: {r3}"
  
  -- Literal mismatch
  let r4 := eval Env.empty (.if_let
    (.literal (Value.int 42))
    (.literal (Value.int 43))
    (.literal (Value.bool true))
    (.literal (Value.bool false))
  )
  IO.println s!"Literal mismatch: {r4}"

def main : IO Unit := do
  runIfLetTests
```

**Run**:
```bash
lake exe ash_ref
# Expected output:
# === If-Let Expression Tests ===
# Variable: ok { value := int 42, effect := epistemic }
# Variant success: ok { value := int 42, effect := epistemic }
# Variant failure: ok { value := int 0, effect := epistemic }
# Literal mismatch: ok { value := bool false, effect := epistemic }
```

## Completion Checklist

- [ ] `evalIfLet` function
- [ ] Expression evaluation
- [ ] Pattern matching
- [ ] Success branch with bindings
- [ ] Failure branch without bindings
- [ ] Effect accumulation
- [ ] Variable pattern tests
- [ ] Variant pattern tests (success)
- [ ] Variant pattern tests (failure)
- [ ] Literal pattern tests
- [ ] Tuple pattern tests
- [ ] Nested if-let tests
- [ ] Environment capture tests
- [ ] Property tests
- [ ] Integration with main eval function

## Self-Review Questions

1. **Spec adherence**: Does evalIfLet follow SPEC-004 Section 5.3?
   - Yes: IF-LET-SUCCESS and IF-LET-FAIL rules implemented

2. **Binding scope**: Are bindings properly scoped?
   - Yes: Only in then-branch, not else-branch or outer scope

3. **Effect accumulation**: Are all effects tracked?
   - Yes: expr_effect ⊔ branch_effect

## Estimated Effort

6 hours

## Dependencies

- TASK-139 (Environment)
- TASK-140 (Expression Eval)
- TASK-141 (Pattern Match)

## Blocked By

- TASK-141

## Blocks

- TASK-145 (Differential Testing)
