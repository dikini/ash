/-! # Basic Expression Examples

This file demonstrates basic expression evaluation in the Ash reference interpreter.

All examples use the big-step semantics from SPEC-004 Section 5.

## Prerequisites

To run these examples:
```bash
cd lean_reference
lake build Ash.Examples.Basic
```

Or interactively in your editor with the Lean 4 extension.
-/]

import Ash

namespace Ash.Examples.Basic

-- ============================================
-- Literal Expressions (SPEC-004 Section 5.1.1)
-- ============================================

/-- Evaluate a literal integer -/
#eval eval Env.empty (.literal (Value.int 42))
-- Expected: ok { value := int 42, effect := epistemic }

/-- Evaluate a literal string -/
#eval eval Env.empty (.literal (Value.string "hello"))
-- Expected: ok { value := string "hello", effect := epistemic }

/-- Evaluate a literal boolean -/
#eval eval Env.empty (.literal (Value.bool true))
-- Expected: ok { value := bool true, effect := epistemic }

/-- Evaluate the null literal -/
#eval eval Env.empty (.literal .null)
-- Expected: ok { value := null, effect := epistemic }

-- ============================================
-- Variable Expressions (SPEC-004 Section 5.1.2)
-- ============================================

/-- Variable lookup in environment -/
#eval 
  let env := Env.empty.bind "x" (Value.int 100)
  eval env (.variable "x")
-- Expected: ok { value := int 100, effect := epistemic }

/-- Error: unbound variable -/
#eval eval Env.empty (.variable "undefined")
-- Expected: error (unboundVariable "undefined")

-- ============================================
-- Constructor Expressions (SPEC-004 Section 5.1.3)
-- ============================================

/-- Constructor evaluation (pure per SPEC-004) -/
#eval eval Env.empty 
  (.constructor "Some" [("value", .literal (Value.int 42))])
-- Expected: ok { value := variant "" "Some" [("value", int 42)], effect := epistemic }

/-- Constructor with multiple fields -/
#eval eval Env.empty
  (.constructor "Point" [
    ("x", .literal (Value.int 10)),
    ("y", .literal (Value.int 20))
  ])
-- Expected: ok { value := variant "" "Point" [("x", int 10), ("y", int 20)], effect := epistemic }

/-- Constructor with nested expressions -/
#eval 
  let env := Env.empty.bind "n" (Value.int 5)
  eval env
    (.constructor "Wrapper" [
      ("inner", .variable "n")
    ])
-- Expected: ok { value := variant "" "Wrapper" [("inner", int 5)], effect := epistemic }

-- ============================================
-- Tuple Expressions (SPEC-004 Section 5.1.4)
-- ============================================

/-- Empty tuple -/
#eval eval Env.empty (.tuple [])
-- Expected: ok { value := tuple [], effect := epistemic }

/-- Tuple with multiple elements -/
#eval eval Env.empty 
  (.tuple [.literal (Value.int 1), .literal (Value.int 2)])
-- Expected: ok { value := tuple [int 1, int 2], effect := epistemic }

/-- Tuple with mixed types -/
#eval eval Env.empty
  (.tuple [
    .literal (Value.int 42),
    .literal (Value.string "hello"),
    .literal (Value.bool true)
  ])
-- Expected: ok { value := tuple [int 42, string "hello", bool true], effect := epistemic }

-- ============================================
-- Complex Nested Examples
-- ============================================

/-- Nested constructor in tuple -/
#eval eval Env.empty
  (.tuple [
    .constructor "Some" [("value", .literal (Value.int 1))],
    .constructor "None" []
  ])

/-- Variable in tuple -/
#eval
  let env := Env.empty
    |>.bind "a" (Value.int 1)
    |>.bind "b" (Value.int 2)
  eval env (.tuple [.variable "a", .variable "b"])

end Ash.Examples.Basic
