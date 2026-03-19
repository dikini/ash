/-! # Pattern Matching Examples

This file demonstrates pattern matching in the Ash reference interpreter,
following SPEC-004 Section 5.2.

Pattern matching is the primary way to destructure values and extract components.

## Pattern Types

1. **Wildcard** - Matches any value, no binding
2. **Variable** - Matches any value, binds to name
3. **Literal** - Matches specific value
4. **Variant** - Matches constructor patterns
5. **Tuple** - Matches tuple patterns
6. **Record** - Matches record patterns
-/]

import Ash

namespace Ash.Examples.PatternMatching

-- ============================================
-- Basic Patterns (SPEC-004 Section 5.2.1)
-- ============================================

/-- Wildcard pattern matches anything -/
#eval matchPattern .wildcard (Value.int 42)
-- Expected: some Env.empty (no bindings, but match succeeds)

/-- Variable pattern binds the value -/
#eval matchPattern (.variable "x") (Value.string "hello")
-- Expected: some (env with x = "hello")

/-- Variable pattern binds any type -/
#eval matchPattern (.variable "y") (Value.int 100)
-- Expected: some (env with y = 100)

-- ============================================
-- Literal Patterns (SPEC-004 Section 5.2.2)
-- ============================================

/-- Literal pattern requires exact match -/
#eval matchPattern (.literal (Value.int 42)) (Value.int 42)
-- Expected: some Env.empty

/-- Literal pattern fails on mismatch -/
#eval matchPattern (.literal (Value.int 42)) (Value.int 43)
-- Expected: none

/-- String literal pattern -/
#eval matchPattern (.literal (Value.string "hello")) (Value.string "hello")
-- Expected: some Env.empty

/-- Boolean literal pattern -/
#eval matchPattern (.literal (Value.bool true)) (Value.bool true)
-- Expected: some Env.empty

-- ============================================
-- Variant Patterns (SPEC-004 Section 5.2.3)
-- ============================================

/-- Variant pattern matching success -/
#eval 
  let value := Value.variant "Option" "Some" [("value", Value.int 42)]
  let pattern := Pattern.variant "Some" [("value", .variable "x")]
  matchPattern pattern value
-- Expected: some (env with x = 42)

/-- Variant pattern with multiple fields -/
#eval
  let value := Value.variant "" "Point" [("x", Value.int 10), ("y", Value.int 20)]
  let pattern := Pattern.variant "Point" [("x", .variable "px"), ("y", .variable "py")]
  matchPattern pattern value
-- Expected: some (env with px = 10, py = 20)

/-- Variant pattern mismatch (different constructor) -/
#eval 
  let value := Value.variant "Option" "None" []
  let pattern := Pattern.variant "Some" [("value", .variable "x")]
  matchPattern pattern value
-- Expected: none

/-- Nested variant pattern -/
#eval
  let value := Value.variant "Option" "Some" [
    ("value", Value.variant "Result" "Ok" [("data", Value.int 42)])
  ]
  let pattern := Pattern.variant "Some" [
    ("value", Pattern.variant "Ok" [("data", .variable "n")])
  ]
  matchPattern pattern value
-- Expected: some (env with n = 42)

-- ============================================
-- Tuple Patterns (SPEC-004 Section 5.2.4)
-- ============================================

/-- Empty tuple pattern -/
#eval matchPattern (Pattern.tuple []) (Value.tuple [])
-- Expected: some Env.empty

/-- Tuple pattern with variables -/
#eval 
  let value := Value.tuple [Value.int 1, Value.int 2]
  let pattern := Pattern.tuple [.variable "a", .variable "b"]
  matchPattern pattern value
-- Expected: some (env with a = 1, b = 2)

/-- Tuple pattern with mixed patterns -/
#eval
  let value := Value.tuple [Value.int 42, Value.string "hello"]
  let pattern := Pattern.tuple [.literal (Value.int 42), .variable "msg"]
  matchPattern pattern value
-- Expected: some (env with msg = "hello")

/-- Tuple pattern length mismatch -/
#eval
  let value := Value.tuple [Value.int 1]
  let pattern := Pattern.tuple [.variable "a", .variable "b"]
  matchPattern pattern value
-- Expected: none

-- ============================================
-- Record Patterns (SPEC-004 Section 5.2.5)
-- ============================================

/-- Simple record pattern -/
#eval
  let value := Value.record [("name", Value.string "Alice"), ("age", Value.int 30)]
  let pattern := Pattern.record [("name", .variable "n"), ("age", .variable "a")]
  matchPattern pattern value
-- Expected: some (env with n = "Alice", a = 30)

-- ============================================
-- Match Expressions (SPEC-004 Section 5.2.6)
-- ============================================

/-- Match expression with multiple arms (first matches) -/
#eval eval Env.empty (.match
  (.constructor "Some" [("value", .literal (Value.int 42))])
  [
    { pattern := .variant "Some" [("value", .variable "x")], 
      body := .variable "x" },
    { pattern := .variant "None" [], 
      body := .literal (Value.int 0) }
  ])
-- Expected: ok { value := int 42, effect := epistemic }

/-- Match expression with multiple arms (second matches) -/
#eval eval Env.empty (.match
  (.constructor "None" [])
  [
    { pattern := .variant "Some" [("value", .variable "x")], 
      body := .variable "x" },
    { pattern := .variant "None" [], 
      body := .literal (Value.int 0) }
  ])
-- Expected: ok { value := int 0, effect := epistemic }

/-- Non-exhaustive match -/
#eval eval Env.empty (.match
  (.literal (Value.int 42))
  [
    { pattern := .literal (Value.int 0), 
      body := .literal (Value.string "zero") }
  ])
-- Expected: error nonExhaustiveMatch

/-- Match with wildcard (exhaustive) -/
#eval eval Env.empty (.match
  (.literal (Value.int 42))
  [
    { pattern := .literal (Value.int 0), 
      body := .literal (Value.string "zero") },
    { pattern := .wildcard, 
      body := .literal (Value.string "other") }
  ])
-- Expected: ok { value := string "other", effect := epistemic }

-- ============================================
-- Effect Tracking in Matches
-- ============================================

/-- Effects from scrutinee and body are combined -/
#eval eval Env.empty (.match
  (.literal (Value.int 42))
  [
    { pattern := .variable "x", 
      body := .variable "x" }
  ])
-- Expected: ok { value := int 42, effect := epistemic }
-- (scrutinee and body both have epistemic effect)

end Ash.Examples.PatternMatching
