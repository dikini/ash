# TASK-141: Pattern Matching Engine

## Status: ✅ Complete

## Description

Implement the pattern matching engine in Lean 4. This provides the core matching algorithm for destructuring values against patterns, producing environment bindings.

## Specification Reference

- SPEC-021: Lean Reference - Section 6.3 (Pattern Matching)
- SPEC-004: Operational Semantics - Section 5.2 (Pattern Binding)
- SPEC-004: Operational Semantics - Section 6.1 (Extended Pattern Binding)

## Requirements

### Functional Requirements

1. Implement `matchPattern` function:
   ```
   matchPattern : Pattern → Value → Option Env
   ```
2. Support pattern types:
   - `wildcard` - matches anything, no binding
   - `variable` - matches anything, binds to name
   - `literal` - matches equal value
   - `variant` - matches enum variant, recursively matches fields
   - `tuple` - matches tuple, element-wise matching
   - `record` - matches struct/variant fields by name
3. Return `none` on match failure
4. Return `some env` with bindings on success
5. Implement `mergeEnvs` for combining bindings

### Property Requirements

```lean
-- Pattern match determinism (for future proof)
prop_match_deterministic(p, v) = 
  matchPattern p v = env1 →
  matchPattern p v = env2 →
  env1 = env2

-- Wildcard always matches
prop_wildcard_matches(v) = 
  matchPattern .wildcard v ≠ none

-- Variable binds value
prop_variable_binds(x, v) = 
  matchPattern (.variable x) v = some (Env.empty.bind x v)

-- Literal equality required
prop_literal_eq(lit, v) = 
  lit = v ↔ matchPattern (.literal lit) v ≠ none
```

## TDD Steps

### Step 1: Define matchPattern Skeleton (Red)

**File**: `lean_reference/Ash/Eval/Pattern.lean`

```lean
import Ash.Core.AST
import Ash.Core.Environment

namespace Ash.Eval

open Ash

partial def matchPattern (p : Pattern) (v : Value) : Option Env :=
  match p, v with
  | .wildcard, _ => some Env.empty
  | .variable x, v => some (Env.empty.bind x v)
  | .literal l, v => if l = v then some Env.empty else none
  | _, _ => none  -- Placeholder for complex patterns

def mergeEnvs (env1 env2 : Env) : Env :=
  fun x =>
    match env2.lookup x with
    | some v => some v
    | none => env1.lookup x

end Ash.Eval
```

**Test**:
```lean
#eval matchPattern .wildcard (Value.int 42)
-- Expected: some (fun x => none)

#eval matchPattern (.variable "x") (Value.string "hello")
-- Expected: some (Env.empty.bind "x" (Value.string "hello"))

#eval matchPattern (.literal (Value.int 42)) (Value.int 42)
-- Expected: some Env.empty

#eval matchPattern (.literal (Value.int 42)) (Value.int 43)
-- Expected: none
```

### Step 2: Implement Variant Pattern Matching (Green)

Per SPEC-004 Section 5.2 (MATCH-VARIANT):

```lean
partial def matchPattern (p : Pattern) (v : Value) : Option Env :=
  match p, v with
  | .wildcard, _ => some Env.empty
  | .variable x, v => some (Env.empty.bind x v)
  | .literal l, v => if l = v then some Env.empty else none
  
  | .variant name fields, .variant _ vname vfields =>
      if name = vname then
        matchFields fields vfields
      else
        none
  | .variant _ _, _ => none
  
  | _, _ => none

partial def matchFields (ps : List (String × Pattern))
    (vs : List (String × Value)) : Option Env :=
  match ps with
  | [] => some Env.empty
  | (name, p) :: rest =>
      match vs.find? (fun (n, _) => n = name) with
      | none => none
      | some (_, v) =>
          match matchPattern p v with
          | none => none
          | some env1 =>
              match matchFields rest vs with
              | none => none
              | some env2 => some (mergeEnvs env1 env2)
```

**Test**:
```lean
def testVariant : IO Unit := do
  -- Create a Some { value = 42 } value
  let someVal := Value.variant "Option" "Some" [("value", Value.int 42)]
  
  -- Pattern: Some { value = x }
  let pattern := Pattern.variant "Some" [("value", .variable "x")]
  
  let result := matchPattern pattern someVal
  IO.println s!"Variant match: {result}"
  
  -- Pattern: None {}
  let noneVal := Value.variant "Option" "None" []
  let nonePattern := Pattern.variant "None" []
  let result2 := matchPattern nonePattern noneVal
  IO.println s!"None match: {result2}"

-- Expected:
-- Variant match: some (env with x = 42)
-- None match: some Env.empty
```

### Step 3: Implement Tuple Pattern Matching (Green)

```lean
partial def matchList (ps : List Pattern) (vs : List Value) : Option Env :=
  match ps, vs with
  | [], [] => some Env.empty
  | p :: prest, v :: vrest =>
      match matchPattern p v with
      | none => none
      | some env1 =>
          match matchList prest vrest with
          | none => none
          | some env2 => some (mergeEnvs env1 env2)
  | _, _ => none

-- Add to matchPattern:
  | .tuple ps, .tuple vs =>
      if ps.length = vs.length then
        matchList ps vs
      else
        none
  | .tuple _, _ => none
```

**Test**:
```lean
def testTuple : IO Unit := do
  -- Value: (1, "hello")
  let tupVal := Value.tuple [Value.int 1, Value.string "hello"]
  
  -- Pattern: (x, y)
  let pattern := Pattern.tuple [.variable "x", .variable "y"]
  
  let result := matchPattern pattern tupVal
  IO.println s!"Tuple match: {result}"
  
  -- Pattern: (_, y) - wildcard first
  let pattern2 := Pattern.tuple [.wildcard, .variable "y"]
  let result2 := matchPattern pattern2 tupVal
  IO.println s!"Tuple with wildcard: {result2}"

-- Expected:
-- Tuple match: some (env with x = 1, y = "hello")
-- Tuple with wildcard: some (env with y = "hello")
```

### Step 4: Implement Record Pattern Matching (Green)

```lean
partial def matchRecord (ps : List (String × Pattern))
    (vs : List (String × Value)) : Option Env :=
  -- Similar to matchFields but for records
  match ps with
  | [] => some Env.empty
  | (name, p) :: rest =>
      match vs.find? (fun (n, _) => n = name) with
      | none => none
      | some (_, v) =>
          match matchPattern p v with
          | none => none
          | some env1 =>
              match matchRecord rest vs with
              | none => none
              | some env2 => some (mergeEnvs env1 env2)

-- Add to matchPattern:
  | .record fields, .record vs =>
      matchRecord fields vs
  | .record fields, .variant _ _ vs =>
      -- Can match variant as record by fields
      matchRecord fields vs
  | .record _, _ => none
```

**Test**:
```lean
def testRecord : IO Unit := do
  -- Value: { x = 1, y = 2 }
  let recVal := Value.record [("x", Value.int 1), ("y", Value.int 2)]
  
  -- Pattern: { x = a }
  let pattern := Pattern.record [("x", .variable "a")]
  
  let result := matchPattern pattern recVal
  IO.println s!"Record match: {result}"

-- Expected:
-- Record match: some (env with a = 1)
```

### Step 5: Property Tests for Determinism (Green)

```lean
-- Pattern matching is deterministic (key property for proofs)
#test ∀ (p : Pattern) (v : Value),
  match matchPattern p v with
  | some env1 =>
      match matchPattern p v with
      | some env2 => env1 = env2
      | none => true  -- Shouldn't happen if first matched
  | none => true

-- Wildcard always succeeds
#test ∀ (v : Value),
  matchPattern .wildcard v ≠ none

-- Variable always succeeds and binds
#test ∀ (x : String) (v : Value),
  matchPattern (.variable x) v = some (Env.empty.bind x v)
```

### Step 6: Test Complex Nested Patterns (Green)

```lean
def testNestedPattern : IO Unit := do
  -- Value: Some { point = { x = 1, y = 2 } }
  let nestedVal := Value.variant "Option" "Some" [
    ("point", Value.record [
      ("x", Value.int 1),
      ("y", Value.int 2)
    ])
  ]
  
  -- Pattern: Some { point = { x = px } }
  let pattern := Pattern.variant "Some" [
    ("point", .record [("x", .variable "px")])
  ]
  
  let result := matchPattern pattern nestedVal
  IO.println s!"Nested: {result}"
  
  -- Verify binding
  match result with
  | some env =>
      IO.println s!"px = {env.lookup "px"}"
  | none =>
      IO.println "Match failed"

-- Expected:
-- Nested: some (env with px = 1)
-- px = some (int 1)
```

### Step 7: Test Match Failure Cases (Green)

```lean
def testFailures : IO Unit := do
  -- Mismatched variant names
  let someVal := Value.variant "Option" "Some" [("value", Value.int 42)]
  let nonePattern := Pattern.variant "None" []
  let r1 := matchPattern nonePattern someVal
  IO.println s!"Wrong variant: {r1}"  -- Expected: none
  
  -- Missing field
  let pointVal := Value.variant "Point" "Point" [("x", Value.int 1)]
  let pointPattern := Pattern.variant "Point" [
    ("x", .variable "x"),
    ("y", .variable "y")  -- y is missing from value
  ]
  let r2 := matchPattern pointPattern pointVal
  IO.println s!"Missing field: {r2}"  -- Expected: none
  
  -- Tuple length mismatch
  let tup3 := Value.tuple [Value.int 1, Value.int 2, Value.int 3]
  let tup2Pattern := Pattern.tuple [.variable "a", .variable "b"]
  let r3 := matchPattern tup2Pattern tup3
  IO.println s!"Tuple length: {r3}"  -- Expected: none

-- All should print: none
```

### Step 8: Integration with Main (Green)

**File**: `lean_reference/Main.lean`

```lean
import Ash

def runPatternTests : IO Unit := do
  IO.println "\n=== Pattern Matching Tests ==="
  
  -- Wildcard
  IO.println s!"Wildcard: {matchPattern .wildcard (Value.int 42)}"
  
  -- Variable
  IO.println s!"Variable: {matchPattern (.variable "x") (Value.string "test")}"
  
  -- Literal
  IO.println s!"Literal (match): {matchPattern (.literal (Value.int 42)) (Value.int 42)}"
  IO.println s!"Literal (fail): {matchPattern (.literal (Value.int 42)) (Value.int 43)}"
  
  -- Variant
  let someVal := Value.variant "Option" "Some" [("value", Value.int 42)]
  let somePat := Pattern.variant "Some" [("value", .variable "x")]
  IO.println s!"Variant: {matchPattern somePat someVal}"
  
  -- Tuple
  let tup := Value.tuple [Value.int 1, Value.int 2]
  let tupPat := Pattern.tuple [.variable "a", .variable "b"]
  IO.println s!"Tuple: {matchPattern tupPat tup}"

def main : IO Unit := do
  runPatternTests
```

**Run**:
```bash
lake exe ash_ref
# Expected output:
# === Pattern Matching Tests ===
# Wildcard: some ...
# Variable: some ...
# Literal (match): some ...
# Literal (fail): none
# Variant: some ...
# Tuple: some ...
```

## Completion Checklist

- [ ] `matchPattern` function with all pattern types
- [ ] `wildcard` pattern matching
- [ ] `variable` pattern binding
- [ ] `literal` pattern equality
- [ ] `variant` pattern with field matching
- [ ] `tuple` pattern with element matching
- [ ] `record` pattern with field matching
- [ ] `mergeEnvs` for combining environments
- [ ] `matchFields` for variant field matching
- [ ] `matchList` for tuple element matching
- [ ] `matchRecord` for record field matching
- [ ] Determinism property tests
- [ ] Wildcard/variable success property tests
- [ ] Complex nested pattern tests
- [ ] Match failure case tests

## Self-Review Questions

1. **Spec adherence**: Does matchPattern follow SPEC-004 Section 5.2?
   - Yes: bind(pat, v, Γ) = Γ' semantics implemented

2. **Determinism**: Is pattern matching deterministic?
   - Yes: Pure function, same input → same output

3. **Completeness**: Are all pattern types handled?
   - Yes: All 6 pattern types from SPEC-021 implemented

## Estimated Effort

16 hours

## Dependencies

- TASK-138 (AST Types)
- TASK-139 (Environment)

## Blocked By

- TASK-139

## Blocks

- TASK-142 (Match Expr)
- TASK-143 (If-Let)
