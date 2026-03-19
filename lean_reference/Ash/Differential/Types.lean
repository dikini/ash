-- Ash Differential Testing Types
-- Types for comparison with Rust implementation
-- TASK-145: Differential Testing Harness

import Ash.Core.AST
import Ash.Core.Environment

namespace Ash.Differential

open Ash

/-- Type of mismatch detected during comparison

Note: We store JSON as String since Json doesn't have Repr instance -/
inductive MismatchType where
  | valueMismatch (lean : Value) (rust : String)
  | effectMismatch (lean : Effect) (rust : String)
  | errorMismatch (lean : EvalError) (rust : String)
  | unexpectedSuccess
  | unexpectedError
  deriving Repr

/-- Result of Lean evaluation as a serializable structure -/
inductive LeanResult where
  | success (result : EvalResult)
  | error (err : EvalError)
  deriving Repr

/-- Convert Except to LeanResult -/
def LeanResult.fromExcept (e : Except EvalError EvalResult) : LeanResult :=
  match e with
  | .ok r => .success r
  | .error err => .error err

/-- Comparison result between Lean and Rust -/
structure ComparisonResult where
  equivalent : Bool
  difference : Option String
  leanResult : LeanResult
  deriving Repr

/-- Mismatch report for detailed analysis -/
structure MismatchReport where
  workflow : Expr
  mismatch : MismatchType
  deriving Repr

/-- Test case for differential testing -/
structure TestCase where
  name : String
  workflow : Expr
  description : Option String := none
  deriving Repr

/-- Batch test results -/
structure BatchResults where
  passed : Nat
  failed : Nat
  reports : List MismatchReport
  deriving Repr

end Ash.Differential
