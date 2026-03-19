# TASK-142: Match Expression Evaluation

## Status: 🟡 Ready to Start

## Description

Implement match expression evaluation in Lean 4. This enables multi-arm pattern matching with scrutinee evaluation, arm selection, and body evaluation with pattern bindings.

## Specification Reference

- SPEC-021: Lean Reference - Section 6.4 (Match Expression)
- SPEC-004: Operational Semantics - Section 5.2 (MATCH rules)

## Requirements

### Functional Requirements

1. Implement `evalMatch` function:
   ```
   evalMatch : Env → Expr → List MatchArm → Except EvalError EvalResult
   ```
2. Evaluate scrutinee expression
3. Find first matching arm using `findMatchingArm`
4. Apply pattern bindings to environment
5. Evaluate arm body in extended environment
6. Accumulate effects from scrutinee and body
7. Return `nonExhaustiveMatch` error if no arm matches

### Property Requirements

```lean
-- Exhaustive match never fails
prop_exhaustive_match(scrutinee, arms) = 
  arms_cover_all_cases(scrutinee, arms) →
  match evalMatch env scrutinee arms with
  | .error .nonExhaustiveMatch => false
  | _ => true

-- Effect accumulation
prop_match_effect_accum(env, scrutinee, arms) = 
  match evalMatch env scrutinee arms with
  | .ok result =>
      result.effect = scrutinee_effect.join body_effect
  | _ => true

-- First match wins
prop_first_match_wins(v, arm1, arm2) = 
  arm1.pattern.matches(v) →
  arm2.pattern.matches(v) →
  arm1 comes before arm2 →
  arm1 is selected
```

## TDD Steps

### Step 1: Define findMatchingArm (Red)

**File**: `lean_reference/Ash/Eval/Match.lean`

```lean
import Ash.Core.AST
import Ash.Core.Environment
import Ash.Eval.Pattern

namespace Ash.Eval

open Ash

def findMatchingArm (arms : List MatchArm) (v : Value)
    : Option (MatchArm × Env) :=
  arms.findSome? (fun arm =>
    match matchPattern arm.pattern v with
    | none => none
    | some bindings => some (arm, bindings))

end Ash.Eval
```

**Test**:
```lean
#eval findMatchingArm 
  [{ pattern := .wildcard, body := .literal (Value.int 1) }] 
  (Value.int 42)
-- Expected: some ({ pattern := wildcard, body := ... }, Env.empty)
```

### Step 2: Define evalMatch Skeleton (Red)

```lean
def evalMatch (env : Env) (scrutinee : Expr)
    (arms : List MatchArm) : Except EvalError EvalResult := do
  -- Evaluate scrutinee
  let scrutResult ← eval env scrutinee
  
  -- Find matching arm
  match findMatchingArm arms scrutResult.value with
  | none => throw .nonExhaustiveMatch
  | some (arm, bindings) =>
      -- Evaluate body with extended environment
      let newEnv := mergeEnvs env bindings
      let bodyResult ← eval newEnv arm.body
      
      -- Combine effects
      pure {
        value := bodyResult.value,
        effect := scrutResult.effect.join bodyResult.effect
      }
```

**Test** (with single wildcard arm):
```lean
def testMatch : IO Unit := do
  -- match 42 { _ => 100 }
  let scrut := Expr.literal (Value.int 42)
  let arms := [
    { pattern := .wildcard, body := .literal (Value.int 100) : MatchArm }
  ]
  let r := evalMatch Env.empty scrut arms
  IO.println s!"Simple match: {r}"

-- Expected: ok { value := int 100, effect := epistemic }
```

### Step 3: Implement Variable Binding in Match (Green)

```lean
def testVariableBinding : IO Unit := do
  -- match 42 { x => x }
  let scrut := Expr.literal (Value.int 42)
  let arms := [
    { pattern := .variable "x", body := .variable "x" : MatchArm }
  ]
  let r := evalMatch Env.empty scrut arms
  IO.println s!"Variable binding: {r}"

-- Expected: ok { value := int 42, effect := epistemic }
-- Pattern binds x=42, body returns x
```

### Step 4: Test Variant Matching (Green)

```lean
def testVariantMatching : IO Unit := do
  -- match Some { value = 42 } {
  --   Some { value = x } => x,
  --   None => 0
  -- }
  let scrut := Expr.constructor "Some" [("value", .literal (Value.int 42))]
  let arms := [
    { 
      pattern := Pattern.variant "Some" [("value", .variable "x")],
      body := .variable "x"
    },
    { 
      pattern := Pattern.variant "None" [],
      body := .literal (Value.int 0)
    }
  ]
  let r := evalMatch Env.empty scrut arms
  IO.println s!"Variant match: {r}"
  
  -- Test None case
  let scrut2 := Expr.constructor "None" []
  let r2 := evalMatch Env.empty scrut2 arms
  IO.println s!"None match: {r2}"

-- Expected:
-- Variant match: ok { value := int 42, effect := epistemic }
-- None match: ok { value := int 0, effect := epistemic }
```

### Step 5: Test Non-Exhaustive Match Error (Green)

```lean
def testNonExhaustive : IO Unit := do
  -- match 42 { None => 0 }
  -- (42 doesn't match None)
  let scrut := Expr.literal (Value.int 42)
  let arms := [
    { pattern := Pattern.variant "None" [], body := .literal (Value.int 0) }
  ]
  let r := evalMatch Env.empty scrut arms
  IO.println s!"Non-exhaustive: {r}"

-- Expected: error nonExhaustiveMatch
```

### Step 6: Test Effect Accumulation (Green)

```lean
def testEffectAccumulation : IO Unit := do
  -- Effects should combine: scrutinee_effect ⊔ body_effect
  -- For now, both are epistemic, so result is epistemic
  -- This test sets up infrastructure for when we have deliberative effects
  
  let scrut := Expr.tuple [
    .literal (Value.int 1),
    .literal (Value.int 2)
  ]
  let arms := [
    { 
      pattern := Pattern.tuple [.variable "a", .variable "b"],
      body := .variable "a"
    }
  ]
  let r := evalMatch Env.empty scrut arms
  IO.println s!"Effect accumulation: {r}"

-- Expected: ok { value := int 1, effect := epistemic }
```

### Step 7: Test Nested Match (Green)

```lean
def testNestedMatch : IO Unit := do
  -- match (Some { value = (1, 2) }) {
  --   Some { value = (a, b) } => a + b
  --   None => 0
  -- }
  -- (Note: + not implemented yet, so return tuple instead)
  
  let innerTuple := Expr.tuple [.literal (Value.int 1), .literal (Value.int 2)]
  let scrut := Expr.constructor "Some" [("value", innerTuple)]
  let arms := [
    { 
      pattern := Pattern.variant "Some" [
        ("value", .tuple [.variable "a", .variable "b"])
      ],
      body := .tuple [.variable "a", .variable "b"]
    },
    { 
      pattern := Pattern.variant "None" [],
      body := .literal (Value.int 0)
    }
  ]
  let r := evalMatch Env.empty scrut arms
  IO.println s!"Nested match: {r}"

-- Expected: ok { value := tuple [int 1, int 2], effect := epistemic }
```

### Step 8: Test Match with Existing Bindings (Green)

```lean
def testMatchWithEnv : IO Unit := do
  -- let y = 100 in
  -- match 42 { x => y }
  -- Should capture y from outer environment
  
  let env := Env.empty.bind "y" (Value.int 100)
  let scrut := Expr.literal (Value.int 42)
  let arms := [
    { 
      pattern := .variable "x",
      body := .variable "y"  -- Uses outer binding, not pattern
    }
  ]
  let r := evalMatch env scrut arms
  IO.println s!"Match with env: {r}"

-- Expected: ok { value := int 100, effect := epistemic }
```

### Step 9: Property Tests (Green)

```lean
-- Wildcard arm is exhaustive for any value
#test ∀ (v : Value),
  match evalMatch Env.empty (.literal v) [
    { pattern := .wildcard, body := .literal (Value.int 0) }
  ] with
  | .error .nonExhaustiveMatch => false
  | _ => true

-- Variable arm binds correctly
#test ∀ (v : Value),
  match evalMatch Env.empty (.literal v) [
    { pattern := .variable "x", body := .variable "x" }
  ] with
  | .ok result => result.value = v
  | _ => false

-- Non-exhaustive patterns fail appropriately
#test ∀ (v : Value),
  v ≠ Value.null →
  match evalMatch Env.empty (.literal v) [
    { pattern := .literal .null, body := .literal (Value.int 0) }
  ] with
  | .error .nonExhaustiveMatch => true
  | _ => false
```

### Step 10: Integration with Main Eval (Green)

Update `Ash/Eval/Expr.lean` to call evalMatch:

```lean
import Ash.Eval.Match

-- In eval function:
  | .match scrutinee arms => evalMatch env scrutinee arms
```

**Integration Test**:
```lean
def runMatchTests : IO Unit := do
  IO.println "\n=== Match Expression Tests ==="
  
  -- Simple wildcard
  let r1 := eval Env.empty (.match 
    (.literal (Value.int 42))
    [{ pattern := .wildcard, body := .literal (Value.int 100) }]
  )
  IO.println s!"Wildcard: {r1}"
  
  -- Variable binding
  let r2 := eval Env.empty (.match
    (.literal (Value.int 42))
    [{ pattern := .variable "x", body := .variable "x" }]
  )
  IO.println s!"Variable: {r2}"
  
  -- Variant matching
  let r3 := eval Env.empty (.match
    (.constructor "Some" [("value", .literal (Value.int 42))])
    [
      { pattern := .variant "Some" [("value", .variable "x")], body := .variable "x" },
      { pattern := .variant "None" [], body := .literal (Value.int 0) }
    ]
  )
  IO.println s!"Variant: {r3}"
  
  -- Non-exhaustive
  let r4 := eval Env.empty (.match
    (.literal (Value.int 42))
    [{ pattern := .literal (Value.int 0), body := .literal (Value.int 100) }]
  )
  IO.println s!"Non-exhaustive: {r4}"

def main : IO Unit := do
  runMatchTests
```

**Run**:
```bash
lake exe ash_ref
# Expected output:
# === Match Expression Tests ===
# Wildcard: ok { value := int 100, effect := epistemic }
# Variable: ok { value := int 42, effect := epistemic }
# Variant: ok { value := int 42, effect := epistemic }
# Non-exhaustive: error nonExhaustiveMatch
```

## Completion Checklist

- [ ] `findMatchingArm` function
- [ ] `evalMatch` function with big-step semantics
- [ ] Scrutinee evaluation
- [ ] First-match arm selection
- [ ] Pattern binding application
- [ ] Body evaluation in extended environment
- [ ] Effect accumulation (scrutinee ⊔ body)
- [ ] Non-exhaustive match error
- [ ] Wildcard arm tests
- [ ] Variable binding tests
- [ ] Variant matching tests
- [ ] Tuple matching tests
- [ ] Nested match tests
- [ ] Non-exhaustive error tests
- [ ] Property tests for exhaustiveness
- [ ] Integration with main eval function

## Self-Review Questions

1. **Spec adherence**: Does evalMatch follow SPEC-004 Section 5.2?
   - Yes: MATCH-VARIANT and MATCH-WILDCARD rules implemented

2. **Effect tracking**: Are effects properly accumulated?
   - Yes: scrutinee_effect.join body_effect

3. **First-match semantics**: Is order preserved?
   - Yes: findSome? returns first matching arm

## Estimated Effort

12 hours

## Dependencies

- TASK-139 (Environment)
- TASK-140 (Expression Eval)
- TASK-141 (Pattern Match)

## Blocked By

- TASK-141

## Blocks

- TASK-143 (If-Let)
- TASK-145 (Differential Testing)
