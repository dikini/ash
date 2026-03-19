/-! # If-Let Expression Examples

This file demonstrates if-let expressions in the Ash reference interpreter,
following SPEC-004 Section 5.3.

If-let expressions combine pattern matching with conditional execution,
allowing you to match a value against a pattern and execute different
branches based on whether the match succeeds.

## Syntax

```
if_let <pattern> := <expr> then <then_branch> else <else_branch>
```

## Semantics

1. Evaluate `<expr>` to get a value
2. Match the value against `<pattern>`
3. If match succeeds: evaluate `<then_branch>` with pattern bindings
4. If match fails: evaluate `<else_branch>` with original environment

## Effects

The combined effect is: expr_effect ⊔ then_effect (or else_effect)
-/]

import Ash

namespace Ash.Examples.IfLet

-- ============================================
-- Basic If-Let (SPEC-004 Section 5.3)
-- ============================================

/-- If-let with variable pattern (always succeeds) -/
#eval eval Env.empty (.if_let
  (.variable "x")
  (.literal (Value.int 42))
  (.variable "x")           -- then branch
  (.literal (Value.int 0))) -- else branch
-- Expected: ok { value := int 42, effect := epistemic }

/-- If-let with literal pattern (success case) -/
#eval eval Env.empty (.if_let
  (.literal (Value.int 42))
  (.literal (Value.int 42))
  (.literal (Value.string "matched"))
  (.literal (Value.string "not matched")))
-- Expected: ok { value := string "matched", effect := epistemic }

/-- If-let with literal pattern (failure case) -/
#eval eval Env.empty (.if_let
  (.literal (Value.int 42))
  (.literal (Value.int 99))
  (.literal (Value.string "matched"))
  (.literal (Value.string "not matched")))
-- Expected: ok { value := string "not matched", effect := epistemic }

-- ============================================
-- Variant Patterns in If-Let
-- ============================================

/-- If-let with variant pattern (success case) -/
#eval eval Env.empty (.if_let
  (.variant "Some" [("value", .variable "x")])
  (.constructor "Some" [("value", .literal (Value.int 42))])
  (.variable "x")           -- then branch: returns 42
  (.literal (Value.int 0))) -- else branch
-- Expected: ok { value := int 42, effect := epistemic }

/-- If-let with variant pattern (failure case) -/
#eval eval Env.empty (.if_let
  (.variant "Some" [("value", .variable "x")])
  (.constructor "None" [])
  (.variable "x")           -- then branch (not taken)
  (.literal (Value.int 0))) -- else branch: returns 0
-- Expected: ok { value := int 0, effect := epistemic }

/-- If-let extracting nested value -/
#eval eval Env.empty (.if_let
  (.variant "Some" [("value", .variable "inner")])
  (.constructor "Some" [("value", .literal (Value.string "hello"))])
  (.variable "inner")
  (.literal (Value.string "default")))
-- Expected: ok { value := string "hello", effect := epistemic }

-- ============================================
-- Environment Handling
-- ============================================

/-- If-let capturing outer environment (else branch uses outer) -/
#eval 
  let env := Env.empty.bind "default" (Value.int 100)
  eval env (.if_let
    (.variant "Some" [("value", .variable "x")])
    (.constructor "None" [])
    (.variable "x")           -- not reached
    (.variable "default"))    -- uses outer binding
-- Expected: ok { value := int 100, effect := epistemic }

/-- If-let with shadowing -/
#eval
  let env := Env.empty.bind "x" (Value.int 999)
  eval env (.if_let
    (.variable "x")           -- binds new x
    (.literal (Value.int 42))
    (.variable "x")           -- refers to bound x (42)
    (.variable "x"))          -- would refer to outer x (999)
-- Expected: ok { value := int 42, effect := epistemic }

-- ============================================
-- Nested If-Let
-- ============================================

/-- Nested if-let for deep destructuring -/
#eval eval Env.empty (.if_let
  (.variant "Some" [("value", .variable "outer")])
  (.constructor "Some" [("value", .literal (Value.int 42))])
  (.if_let
    (.literal (Value.int 42))
    (.variable "outer")
    (.literal (Value.string "matched both"))
    (.literal (Value.string "inner failed")))
  (.literal (Value.string "outer failed")))
-- Expected: ok { value := string "matched both", effect := epistemic }

/-- Nested if-let with variant patterns -/
#eval eval Env.empty (.if_let
  (.variant "Ok" [("result", .variable "r")])
  (.constructor "Ok" [("result", .constructor "Some" [("value", .literal (Value.int 10))])])
  (.if_let
    (.variant "Some" [("value", .variable "v")])
    (.variable "r")
    (.variable "v")
    (.literal (Value.int (-1))))
  (.literal (Value.int (-2))))
-- Expected: ok { value := int 10, effect := epistemic }

-- ============================================
-- Common Patterns
-- ============================================

/-- Safe unwrapping with default -/
#eval
  let someValue := Value.variant "Option" "Some" [("value", Value.int 42)]
  eval Env.empty (.if_let
    (.variant "Some" [("value", .variable "n")])
    (.literal someValue)
    (.variable "n")
    (.literal (Value.int 0)))
-- Expected: ok { value := int 42, effect := epistemic }

/-- Default for None -/
#eval
  let noneValue := Value.variant "Option" "None" []
  eval Env.empty (.if_let
    (.variant "Some" [("value", .variable "n")])
    (.literal noneValue)
    (.variable "n")
    (.literal (Value.int 0)))
-- Expected: ok { value := int 0, effect := epistemic }

-- ============================================
-- Effect Tracking
-- ============================================

/-- Effects combine: scrutinee ⊔ then_branch -/
#eval eval Env.empty (.if_let
  (.variable "x")
  (.literal (Value.int 1))
  (.literal (Value.int 2))
  (.literal (Value.int 3)))
-- Both scrutinee and branches are pure (epistemic)
-- Expected: ok { value := int 2, effect := epistemic }

end Ash.Examples.IfLet
