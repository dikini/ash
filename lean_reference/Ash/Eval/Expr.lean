-- Ash Expression Evaluation
-- Big-step operational semantics per SPEC-004 Section 5.1
--
-- Evaluates expressions in an environment, returning a value and accumulated effect.
-- Uses the big-step judgment form: Γ ⊢ e ⇓ v, ε
--
-- NOTE: This implements a SIMPLIFIED SUBSET of the full Ash expression language.
-- See docs/AST-Subset.md for the complete comparison with Rust.
--
-- Supported: literal, variable, constructor, tuple, match, if_let
-- Not supported: FieldAccess, IndexAccess, Unary, Binary, Call (by design)

import Ash.Core.AST
import Ash.Core.Environment
import Ash.Eval.Pattern
import Ash.Eval.Match

namespace Ash.Eval

open Ash

-- Expression Evaluation
-- Per SPEC-004 Section 5 (Big-Step Semantics):
-- The eval function implements the big-step judgment Γ ⊢ e ⇓ v, ε
--
-- - Γ (env): The evaluation environment mapping variables to values
-- - e (expr): The expression to evaluate
-- - v (result.value): The resulting value
-- - ε (result.effect): The accumulated effect
--
-- Returns Except EvalError to handle runtime errors like unbound variables.

mutual

/-- Main evaluation function

Per SPEC-004 Section 5.1:
- Literal: always returns value with epistemic effect
- Variable: looks up in environment, error if unbound
- Constructor: evaluates fields, returns variant with epistemic effect (pure)
- Tuple: evaluates elements, accumulates effects
- If-let: conditional pattern matching with then/else branches
-/ 
partial def eval (env : Env) (expr : Expr) : Except EvalError EvalResult :=
  match expr with
  | .literal v =>
      -- Literals are pure values with epistemic effect per SPEC-004
      pure { value := v, effect := .epistemic }
  | .variable x =>
      -- Variable lookup in environment
      evalVariable env x
  | .constructor name fields =>
      -- Constructor evaluation - builds variants with pure effect
      evalConstructor env name fields
  | .tuple elements =>
      -- Tuple evaluation - evaluates elements, accumulates effects
      evalTuple env elements
  | .match scrutinee arms =>
      -- Match expression evaluation per SPEC-004 Section 5.2
      evalMatch eval env scrutinee arms
  | .if_let pattern expr then_branch else_branch =>
      -- If-let expression evaluation per SPEC-004 Section 5.3
      evalIfLet env pattern expr then_branch else_branch

/-- Variable Evaluation

Per SPEC-004 Section 5.1 (VARIABLE):
Γ(x) = v
-------- [VARIABLE]
Γ ⊢ x ⇓ v, epistemic

Looks up a variable in the environment. Returns unboundVariable error if not found.
-/ 
partial def evalVariable (env : Env) (x : String) : Except EvalError EvalResult :=
  match env.lookup x with
  | none => throw (.unboundVariable x)
  | some v => pure { value := v, effect := .epistemic }

/-- Constructor Evaluation

Per SPEC-004 Section 5.1 (CONSTRUCTOR-ENUM):
∀i. Γ ⊢ eᵢ ⇓ vᵢ, εᵢ
----------------------------------------------- [CONSTRUCTOR-ENUM]
Γ ⊢ C { f₁=e₁, ..., fₙ=eₙ } ⇓ C(v₁,...,vₙ), epistemic

Constructors are pure (epistemic effect only) per SPEC-004.
Fields are evaluated left-to-right, errors propagate.
-/ 
partial def evalConstructor (env : Env) (name : String)
    (fields : List (String × Expr)) : Except EvalError EvalResult := do
  let mut field_values := []
  let mut accumulated_effect := Effect.epistemic

  for (field_name, field_expr) in fields do
    let result ← eval env field_expr
    field_values := (field_name, EvalResult.value result) :: field_values
    accumulated_effect := Effect.join accumulated_effect (EvalResult.effect result)

  pure {
    value := .variant name field_values.reverse,
    effect := accumulated_effect  -- Accumulated effect from field evaluations
  }

/-- Tuple Evaluation

Per SPEC-004 Section 5.1 (CONSTRUCTOR-TUPLE):
∀i. Γ ⊢ eᵢ ⇓ vᵢ, εᵢ
-------------------------------------- [CONSTRUCTOR-TUPLE]
Γ ⊢ (e₁, ..., eₙ) ⇓ (v₁, ..., vₙ), ⊔εᵢ

Tuples evaluate elements left-to-right and accumulate effects.
-/ 
partial def evalTuple (env : Env) (elements : List Expr) : Except EvalError EvalResult := do
  let mut values := []
  let mut accumulated_effect := Effect.epistemic

  for elem in elements do
    let result ← eval env elem
    values := EvalResult.value result :: values
    accumulated_effect := Effect.join accumulated_effect (EvalResult.effect result)

  pure {
    value := .tuple values.reverse,
    effect := accumulated_effect
  }

/-- Evaluate an if-let expression

Per SPEC-004 Section 5.3 (IF-LET rules):

IF-LET-SUCCESS:
  Γ ⊢ e ⇓ v, ε₁    bind(pat, v) = Γ'    Γ ∪ Γ' ⊢ e₁ ⇓ v₁, ε₂
  -----------------------------------------------------------
  Γ ⊢ if let pat = e then e₁ else e₂ ⇓ v₁, ε₁ ⊔ ε₂

IF-LET-FAIL:
  Γ ⊢ e ⇓ v, ε₁    bind(pat, v) = fail    Γ ⊢ e₂ ⇓ v₂, ε₂
  -----------------------------------------------------------
  Γ ⊢ if let pat = e then e₁ else e₂ ⇓ v₂, ε₁ ⊔ ε₂

Parameters:
- env: The current evaluation environment
- pattern: The pattern to match against
- expr: The expression to evaluate and match
- then_branch: Expression to evaluate if pattern matches
- else_branch: Expression to evaluate if pattern fails to match

Returns:
- ok result: Value and accumulated effect from the taken branch
- error: If evaluation fails (unbound variable, etc.)

Key properties:
- Pattern bindings are only available in the then-branch
- The else-branch uses the original environment
- Effects from the scrutinee and the taken branch are joined
- Pattern bindings do not escape to the outer scope
-/ 
partial def evalIfLet (env : Env) (pattern : Pattern) (expr : Expr)
    (then_branch : Expr) (else_branch : Expr) : Except EvalError EvalResult := do
  -- Step 1: Evaluate the expression being matched
  let expr_result ← eval env expr
  
  -- Step 2: Try to match pattern against the evaluated value
  match matchPattern pattern expr_result.value with
  | some bindings =>
      -- IF-LET-SUCCESS: Pattern matched, evaluate then-branch with bindings
      -- Merge original environment with pattern bindings (right-biased)
      let newEnv := Env.merge env bindings
      let then_result ← eval newEnv then_branch
      
      -- Return value from then-branch with joined effects
      pure {
        value := then_result.value,
        effect := Effect.join expr_result.effect then_result.effect
      }
  | none =>
      -- IF-LET-FAIL: Pattern didn't match, evaluate else-branch without bindings
      let else_result ← eval env else_branch
      
      -- Return value from else-branch with joined effects
      pure {
        value := else_result.value,
        effect := Effect.join expr_result.effect else_result.effect
      }

end

/-! ## Test Functions

These functions demonstrate and test the evaluator.
They are called from Main.lean for integration testing.
-/ 

def testLiteral : IO Unit := do
  IO.println "  Test: Literal evaluation"
  let result := eval Env.empty (.literal (Value.int 42))
  match result with
  | Except.ok r => 
      let msg := "    OK: Value=" ++ reprStr (EvalResult.value r) ++ ", Effect=" ++ toString (EvalResult.effect r)
      IO.println msg
  | Except.error e => IO.println ("    Error: " ++ toString e)

def testVariable : IO Unit := do
  IO.println "  Test: Variable lookup"
  let env := Env.empty.bind "x" (Value.int 42)

  -- Bound variable
  let r1 := eval env (.variable "x")
  match r1 with
  | Except.ok r => 
      let msg := "    OK Bound x: Value=" ++ reprStr (EvalResult.value r) ++ ", Effect=" ++ toString (EvalResult.effect r)
      IO.println msg
  | Except.error e => IO.println ("    Error: " ++ toString e)

  -- Unbound variable
  let r2 := eval env (.variable "y")
  match r2 with
  | Except.ok _ => IO.println "    FAIL: Should have failed for unbound variable"
  | Except.error _ => IO.println "    OK: Unbound y correctly fails"

def testConstructor : IO Unit := do
  IO.println "  Test: Constructor evaluation"
  -- Some with value 42
  let ctor := Expr.constructor "Some" [("value", .literal (Value.int 42))]
  let r := eval Env.empty ctor
  match r with
  | Except.ok result => 
      let msg := "    OK Some(42): Value=" ++ reprStr (EvalResult.value result) ++ ", Effect=" ++ toString (EvalResult.effect result)
      IO.println msg
  | Except.error e => IO.println ("    Error: " ++ toString e)

  -- Point with x=1, y=2
  let point := Expr.constructor "Point" [
    ("x", .literal (Value.int 1)),
    ("y", .literal (Value.int 2))
  ]
  let r2 := eval Env.empty point
  match r2 with
  | Except.ok result => 
      let msg := "    OK Point(1,2): Value=" ++ reprStr (EvalResult.value result) ++ ", Effect=" ++ toString (EvalResult.effect result)
      IO.println msg
  | Except.error e => IO.println ("    Error: " ++ toString e)

def testTuple : IO Unit := do
  IO.println "  Test: Tuple evaluation"
  -- (1, "hello", true)
  let tup := Expr.tuple [
    .literal (Value.int 1),
    .literal (Value.string "hello"),
    .literal (Value.bool true)
  ]
  let r := eval Env.empty tup
  match r with
  | Except.ok result => 
      let msg := "    OK Tuple(1, hello, true): Value=" ++ reprStr (EvalResult.value result) ++ ", Effect=" ++ toString (EvalResult.effect result)
      IO.println msg
  | Except.error e => IO.println ("    Error: " ++ toString e)

def testNested : IO Unit := do
  IO.println "  Test: Nested expression evaluation"
  -- Constructor with variable field
  let env := Env.empty.bind "x" (Value.int 100)
  let nested := Expr.constructor "Wrapper" [
    ("inner", .constructor "Some" [
      ("value", .variable "x")
    ])
  ]
  let r := eval env nested
  match r with
  | Except.ok result => 
      let msg := "    OK Nested constructor: Value=" ++ reprStr (EvalResult.value result) ++ ", Effect=" ++ toString (EvalResult.effect result)
      IO.println msg
  | Except.error e => IO.println ("    Error: " ++ toString e)

/-! ## If-Let Expression Tests (TASK-143) -/ 

def testIfLetVariable : IO Unit := do
  IO.println "  Test: If-let with variable pattern (success)"
  let result := eval Env.empty (.if_let 
    (Pattern.variable "x") 
    (Expr.literal (Value.int 42))
    (Expr.variable "x")
    (Expr.literal (Value.int 0)))
  match result with
  | Except.ok r => 
      let msg := "    OK: Value=" ++ reprStr (EvalResult.value r) ++ ", Effect=" ++ toString (EvalResult.effect r)
      IO.println msg
      if r.value == Value.int 42 then
        IO.println "    ✓ Value is correct (42)"
      else
        IO.println "    ✗ Value is incorrect"
  | Except.error e => IO.println ("    Error: " ++ toString e)

def testIfLetVariantSuccess : IO Unit := do
  IO.println "  Test: If-let with variant pattern (success)"
  let expr := Expr.constructor "Some" [("value", .literal (Value.int 42))]
  let result := eval Env.empty (.if_let 
    (Pattern.variant "Some" [("value", .variable "x")])
    expr
    (Expr.variable "x")
    (Expr.literal (Value.int 0)))
  match result with
  | Except.ok r => 
      let msg := "    OK: Value=" ++ reprStr (EvalResult.value r) ++ ", Effect=" ++ toString (EvalResult.effect r)
      IO.println msg
      if r.value == Value.int 42 then
        IO.println "    ✓ Value is correct (42)"
      else
        IO.println "    ✗ Value is incorrect"
  | Except.error e => IO.println ("    Error: " ++ toString e)

def testIfLetVariantFailure : IO Unit := do
  IO.println "  Test: If-let with variant pattern (failure)"
  let expr := Expr.constructor "None" []
  let result := eval Env.empty (.if_let 
    (Pattern.variant "Some" [("value", .variable "x")])
    expr
    (Expr.variable "x")
    (Expr.literal (Value.int 0)))
  match result with
  | Except.ok r => 
      let msg := "    OK: Value=" ++ reprStr (EvalResult.value r) ++ ", Effect=" ++ toString (EvalResult.effect r)
      IO.println msg
      if r.value == Value.int 0 then
        IO.println "    ✓ Else branch taken correctly (0)"
      else
        IO.println "    ✗ Wrong value"
  | Except.error e => IO.println ("    Error: " ++ toString e)

def testIfLetLiteralSuccess : IO Unit := do
  IO.println "  Test: If-let with literal pattern (success)"
  let result := eval Env.empty (.if_let 
    (Pattern.literal (Value.int 42))
    (Expr.literal (Value.int 42))
    (Expr.literal (Value.string "yes"))
    (Expr.literal (Value.string "no")))
  match result with
  | Except.ok r => 
      let msg := "    OK: Value=" ++ reprStr (EvalResult.value r) ++ ", Effect=" ++ toString (EvalResult.effect r)
      IO.println msg
      if r.value == Value.string "yes" then
        IO.println "    ✓ Then branch taken correctly"
      else
        IO.println "    ✗ Wrong value"
  | Except.error e => IO.println ("    Error: " ++ toString e)

def testIfLetLiteralFailure : IO Unit := do
  IO.println "  Test: If-let with literal pattern (failure)"
  let result := eval Env.empty (.if_let 
    (Pattern.literal (Value.int 42))
    (Expr.literal (Value.int 43))
    (Expr.literal (Value.string "yes"))
    (Expr.literal (Value.string "no")))
  match result with
  | Except.ok r => 
      let msg := "    OK: Value=" ++ reprStr (EvalResult.value r) ++ ", Effect=" ++ toString (EvalResult.effect r)
      IO.println msg
      if r.value == Value.string "no" then
        IO.println "    ✓ Else branch taken correctly"
      else
        IO.println "    ✗ Wrong value"
  | Except.error e => IO.println ("    Error: " ++ toString e)

def testIfLetWildcard : IO Unit := do
  IO.println "  Test: If-let with wildcard pattern (always succeeds)"
  let result := eval Env.empty (.if_let 
    (Pattern.wildcard)
    (Expr.literal (Value.int 42))
    (Expr.literal (Value.string "matched"))
    (Expr.literal (Value.string "not matched")))
  match result with
  | Except.ok r => 
      let msg := "    OK: Value=" ++ reprStr (EvalResult.value r) ++ ", Effect=" ++ toString (EvalResult.effect r)
      IO.println msg
      if r.value == Value.string "matched" then
        IO.println "    ✓ Then branch taken correctly"
      else
        IO.println "    ✗ Wrong value"
  | Except.error e => IO.println ("    Error: " ++ toString e)

def testIfLetTuple : IO Unit := do
  IO.println "  Test: If-let with tuple pattern"
  let expr := Expr.tuple [.literal (Value.int 1), .literal (Value.int 2)]
  let then_branch := Expr.tuple [.variable "b", .variable "a"]
  let result := eval Env.empty (.if_let 
    (Pattern.tuple [.variable "a", .variable "b"])
    expr
    then_branch
    (Expr.literal (Value.int 0)))
  match result with
  | Except.ok r => 
      let msg := "    OK: Value=" ++ reprStr (EvalResult.value r) ++ ", Effect=" ++ toString (EvalResult.effect r)
      IO.println msg
      let expected := Value.tuple [Value.int 2, Value.int 1]
      if r.value == expected then
        IO.println "    ✓ Tuple values swapped correctly"
      else
        IO.println "    ✗ Wrong value"
  | Except.error e => IO.println ("    Error: " ++ toString e)

def testIfLetNested : IO Unit := do
  IO.println "  Test: Nested if-let expressions"
  let outerExpr := Expr.constructor "Some" [("value", .literal (Value.int 42))]
  let innerIfLet := Expr.if_let
    (Pattern.literal (Value.int 42))
    (Expr.variable "x")
    (Expr.literal (Value.string "matched"))
    (Expr.literal (Value.string "wrong value"))
  let result := eval Env.empty (.if_let 
    (Pattern.variant "Some" [("value", .variable "x")])
    outerExpr
    innerIfLet
    (Expr.literal (Value.string "none")))
  match result with
  | Except.ok r => 
      let msg := "    OK: Value=" ++ reprStr (EvalResult.value r) ++ ", Effect=" ++ toString (EvalResult.effect r)
      IO.println msg
      if r.value == Value.string "matched" then
        IO.println "    ✓ Nested if-let worked correctly"
      else
        IO.println "    ✗ Wrong value"
  | Except.error e => IO.println ("    Error: " ++ toString e)

def testIfLetWithEnv : IO Unit := do
  IO.println "  Test: If-let with outer environment capture"
  let env := Env.empty.bind "y" (Value.int 100)
  let expr := Expr.constructor "Some" [("value", .literal (Value.int 42))]
  let result := eval env (.if_let 
    (Pattern.variant "Some" [("value", .variable "x")])
    expr
    (Expr.variable "y")
    (Expr.literal (Value.int 0)))
  match result with
  | Except.ok r => 
      let msg := "    OK: Value=" ++ reprStr (EvalResult.value r) ++ ", Effect=" ++ toString (EvalResult.effect r)
      IO.println msg
      if r.value == Value.int 100 then
        IO.println "    ✓ Outer environment accessible in then-branch"
      else
        IO.println "    ✗ Wrong value"
  | Except.error e => IO.println ("    Error: " ++ toString e)

def testIfLetPatternBinding : IO Unit := do
  IO.println "  Test: If-let pattern binding in then-branch"
  let env := Env.empty.bind "y" (Value.int 100)
  let expr := Expr.constructor "Some" [("value", .literal (Value.int 42))]
  let result := eval env (.if_let 
    (Pattern.variant "Some" [("value", .variable "x")])
    expr
    (Expr.variable "x")
    (Expr.literal (Value.int 0)))
  match result with
  | Except.ok r => 
      let msg := "    OK: Value=" ++ reprStr (EvalResult.value r) ++ ", Effect=" ++ toString (EvalResult.effect r)
      IO.println msg
      if r.value == Value.int 42 then
        IO.println "    ✓ Pattern binding accessible in then-branch"
      else
        IO.println "    ✗ Wrong value"
  | Except.error e => IO.println ("    Error: " ++ toString e)

def testIfLetBindingScope : IO Unit := do
  IO.println "  Test: If-let pattern bindings don't escape to else-branch"
  let expr := Expr.constructor "None" []
  let result := eval Env.empty (.if_let 
    (Pattern.variant "Some" [("value", .variable "x")])
    expr
    (Expr.variable "x")
    (Expr.variable "x"))
  match result with
  | Except.ok r => 
      let msg := "    OK: Value=" ++ reprStr (EvalResult.value r)
      IO.println msg
      IO.println "    ✗ Should have failed (x should not be bound in else-branch)"
  | Except.error e => 
      IO.println ("    OK: Error as expected: " ++ toString e)
      IO.println "    ✓ Pattern binding correctly not available in else-branch"

/-! ## Match Expression Tests (TASK-142) -/

def testMatchWildcard : IO Unit := do
  IO.println "  Test: Match with wildcard"
  let r := eval Env.empty (.match 
    (.literal (Value.int 42))
    [{ pattern := .wildcard, body := .literal (Value.int 100) }])
  match r with
  | Except.ok result =>
      let msg := "    OK: Wildcard match: Value=" ++ reprStr result.value ++ ", Effect=" ++ toString result.effect
      IO.println msg
  | Except.error e => IO.println ("    Error: " ++ toString e)

def testMatchVariable : IO Unit := do
  IO.println "  Test: Match with variable binding"
  let r := eval Env.empty (.match
    (.literal (Value.int 42))
    [{ pattern := .variable "x", body := .variable "x" }])
  match r with
  | Except.ok result =>
      let msg := "    OK: Variable bind: Value=" ++ reprStr result.value ++ ", Effect=" ++ toString result.effect
      IO.println msg
  | Except.error e => IO.println ("    Error: " ++ toString e)

def testMatchVariant : IO Unit := do
  IO.println "  Test: Match with variant patterns"
  let arms := [
    { pattern := Pattern.variant "Some" [("value", .variable "x")], body := .variable "x" },
    { pattern := Pattern.variant "None" [], body := .literal (Value.int 0) }
  ]
  -- Some case
  let r1 := eval Env.empty (.match
    (.constructor "Some" [("value", .literal (Value.int 42))])
    arms)
  match r1 with
  | Except.ok result =>
      let msg := "    OK: Some(42): Value=" ++ reprStr result.value ++ ", Effect=" ++ toString result.effect
      IO.println msg
  | Except.error e => IO.println ("    Error: " ++ toString e)
  -- None case
  let r2 := eval Env.empty (.match
    (.constructor "None" [])
    arms)
  match r2 with
  | Except.ok result =>
      let msg := "    OK: None: Value=" ++ reprStr result.value ++ ", Effect=" ++ toString result.effect
      IO.println msg
  | Except.error e => IO.println ("    Error: " ++ toString e)

def testMatchNonExhaustive : IO Unit := do
  IO.println "  Test: Non-exhaustive match error"
  let r := eval Env.empty (.match
    (.literal (Value.int 42))
    [{ pattern := Pattern.variant "None" [], body := .literal (Value.int 0) }])
  match r with
  | Except.ok _ => IO.println "    FAIL: Should have returned nonExhaustiveMatch error"
  | Except.error .nonExhaustiveMatch => IO.println "    OK: Non-exhaustive match correctly detected"
  | Except.error e => IO.println ("    Unexpected error: " ++ toString e)

def testMatchTuple : IO Unit := do
  IO.println "  Test: Match with tuple pattern"
  let r := eval Env.empty (.match
    (.tuple [.literal (Value.int 1), .literal (Value.int 2)])
    [{ pattern := Pattern.tuple [.variable "a", .variable "b"], body := .variable "a" }])
  match r with
  | Except.ok result =>
      let msg := "    OK: Tuple destructuring: Value=" ++ reprStr result.value ++ ", Effect=" ++ toString result.effect
      IO.println msg
  | Except.error e => IO.println ("    Error: " ++ toString e)

def testMatchNested : IO Unit := do
  IO.println "  Test: Nested match patterns"
  let innerTuple := Expr.tuple [.literal (Value.int 1), .literal (Value.int 2)]
  let r := eval Env.empty (.match
    (.constructor "Some" [("value", innerTuple)])
    [
      { 
        pattern := Pattern.variant "Some" [
          ("value", .tuple [.variable "a", .variable "b"])
        ],
        body := .tuple [.variable "a", .variable "b"]
      },
      { pattern := Pattern.variant "None" [], body := .literal (Value.int 0) }
    ])
  match r with
  | Except.ok result =>
      let msg := "    OK: Nested pattern: Value=" ++ reprStr result.value ++ ", Effect=" ++ toString result.effect
      IO.println msg
  | Except.error e => IO.println ("    Error: " ++ toString e)

def testMatchWithEnv : IO Unit := do
  IO.println "  Test: Match with existing environment bindings"
  let env := Env.empty.bind "y" (Value.int 100)
  let r := eval env (.match
    (.literal (Value.int 42))
    [{ pattern := .variable "x", body := .variable "y" }])
  match r with
  | Except.ok result =>
      let msg := "    OK: Env capture: Value=" ++ reprStr result.value ++ ", Effect=" ++ toString result.effect
      IO.println msg
  | Except.error e => IO.println ("    Error: " ++ toString e)

def testMatchFirstWins : IO Unit := do
  IO.println "  Test: First-match wins semantics"
  let r := eval Env.empty (.match
    (.literal (Value.int 42))
    [
      { pattern := .variable "x", body := .literal (Value.int 1) },
      { pattern := .variable "y", body := .literal (Value.int 2) }
    ])
  match r with
  | Except.ok result =>
      if result.value == Value.int 1 then
        IO.println "    OK: First arm selected (value = 1)"
      else
        IO.println ("    FAIL: Expected 1, got " ++ reprStr result.value)
  | Except.error e => IO.println ("    Error: " ++ toString e)

def testMatchLiteral : IO Unit := do
  IO.println "  Test: Match with literal pattern"
  let arms := [
    { pattern := .literal (Value.int 42), body := .literal (Value.string "yes") },
    { pattern := .wildcard, body := .literal (Value.string "no") }
  ]
  -- Matching literal
  let r1 := eval Env.empty (.match (.literal (Value.int 42)) arms)
  match r1 with
  | Except.ok result =>
      if result.value == Value.string "yes" then
        IO.println "    OK: Literal pattern matched"
      else
        IO.println ("    FAIL: Expected 'yes', got " ++ reprStr result.value)
  | Except.error e => IO.println ("    Error: " ++ toString e)
  -- Non-matching literal
  let r2 := eval Env.empty (.match (.literal (Value.int 99)) arms)
  match r2 with
  | Except.ok result =>
      if result.value == Value.string "no" then
        IO.println "    OK: Literal non-match fell through to wildcard"
      else
        IO.println ("    FAIL: Expected 'no', got " ++ reprStr result.value)
  | Except.error e => IO.println ("    Error: " ++ toString e)

def runAllMatchTests : IO Unit := do
  IO.println "\n=== Match Expression Tests (TASK-142) ==="
  testMatchWildcard
  testMatchVariable
  testMatchVariant
  testMatchNonExhaustive
  testMatchTuple
  testMatchNested
  testMatchWithEnv
  testMatchFirstWins
  testMatchLiteral
  IO.println "============================================"

def runAllExprTests : IO Unit := do
  IO.println "\n=== Expression Evaluation Tests ==="
  testLiteral
  testVariable
  testConstructor
  testTuple
  testNested
  IO.println "===================================="

def runAllIfLetTests : IO Unit := do
  IO.println "\n=== If-Let Expression Tests (TASK-143) ==="
  testIfLetVariable
  testIfLetVariantSuccess
  testIfLetVariantFailure
  testIfLetLiteralSuccess
  testIfLetLiteralFailure
  testIfLetWildcard
  testIfLetTuple
  testIfLetNested
  testIfLetWithEnv
  testIfLetPatternBinding
  testIfLetBindingScope
  IO.println "============================================"

end Ash.Eval
