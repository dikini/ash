-- Ash Differential Testing Compare
-- Compare Lean and Rust results
-- TASK-145: Differential Testing Harness

import Ash.Core.AST
import Ash.Core.Environment
import Ash.Eval.Expr
import Ash.Differential.Types
import Ash.Differential.Parse

namespace Ash.Differential

open Ash
open Lean

/-- Helper type for Rust outcomes -/
inductive RustOutcome where
  | success (res : EvalResult)
  | error (errType : String)
  | noResult
  | parseError
  | parseFailed (e : String)
  deriving Repr

/-- Check if two values are structurally equivalent

Compares a Lean Value with a Rust JSON representation -/
def valuesEquivalent (lean : Value) (rustJson : Json) : Bool :=
  match Value.fromJson rustJson with
  | .error _ => false
  | .ok rust => lean == rust

/-- Check if effects are equivalent

Compares Lean Effect with Rust effect string -/
def effectsEquivalent (lean : Effect) (rust : String) : Bool :=
  match lean with
  | .epistemic => rust == "epistemic"
  | .deliberative => rust == "deliberative"
  | .evaluative => rust == "evaluative"
  | .operational => rust == "operational"

/-- Get string representation of effect from Rust JSON result -/
def getRustEffect (rustJson : Json) : Option String :=
  match rustJson.getObjValAs? String "effect" with
  | .ok s => some s
  | .error _ => none

/-- Compare error types for equivalence

Returns true if the Lean error type matches the Rust error type string -/
def errorsEquivalent (lean : EvalError) (rustErrorType : String) : Bool :=
  match lean with
  | .unboundVariable _ => rustErrorType == "unboundVariable"
  | .typeMismatch _ _ => rustErrorType == "typeMismatch"
  | .nonExhaustiveMatch => rustErrorType == "nonExhaustiveMatch"
  | .unknownConstructor _ => rustErrorType == "unknownConstructor"
  | .missingField _ _ => rustErrorType == "missingField"

/-- Compare Lean and Rust evaluation results

This is the main comparison function that:
1. Evaluates the expression in Lean
2. Parses the Rust JSON result
3. Compares values, effects, and errors
4. Returns a detailed comparison result
-/
def compareResults (expr : Expr) (rustJson : Json) : ComparisonResult :=
  let leanExcept := Ash.Eval.eval Env.empty expr
  let leanResult := LeanResult.fromExcept leanExcept
  let parseResult := parseRustResult rustJson
  let rustErrorResult := parseRustError rustJson

  -- Combine parseResult and rustErrorResult into a unified type for pattern matching
  let rustOutcome : RustOutcome :=
    match parseResult, rustErrorResult with
    | .ok (some res), _ => RustOutcome.success res
    | .ok none, .ok (some err) => RustOutcome.error err
    | .ok none, .ok none => RustOutcome.noResult
    | .ok none, .error _ => RustOutcome.parseError
    | .error e, _ => RustOutcome.parseFailed e

  match leanResult, rustOutcome with
  | LeanResult.success leanRes, RustOutcome.success rustRes =>
      -- Both succeeded - compare values and effects
      let rustEffectStr := getRustEffect rustJson |>.getD "unknown"
      if !valuesEquivalent leanRes.value (rustRes.value.toJson) then
        { equivalent := false,
          difference := some s!"Value mismatch: Lean={reprStr leanRes.value}, Rust={rustRes.value.toJson}",
          leanResult }
      else if !effectsEquivalent leanRes.effect rustEffectStr then
        { equivalent := false,
          difference := some s!"Effect mismatch: Lean={leanRes.effect}, Rust={rustEffectStr}",
          leanResult }
      else
        { equivalent := true,
          difference := none,
          leanResult }

  | LeanResult.error leanErr, RustOutcome.error rustErrType =>
      -- Both errored - check if same error type
      if errorsEquivalent leanErr rustErrType then
        { equivalent := true,
          difference := none,
          leanResult }
      else
        { equivalent := false,
          difference := some s!"Error type mismatch: Lean={leanErr.toString}, Rust={rustErrType}",
          leanResult }

  | LeanResult.success _, RustOutcome.error _ =>
      -- Lean succeeded, Rust failed
      { equivalent := false,
        difference := some "Lean succeeded but Rust failed",
        leanResult }

  | LeanResult.error _, RustOutcome.success _ =>
      -- Lean failed, Rust succeeded
      { equivalent := false,
        difference := some "Rust succeeded but Lean failed",
        leanResult }
        
  | LeanResult.success _, RustOutcome.noResult =>
      -- Lean succeeded, Rust returned no result
      { equivalent := false,
        difference := some "Lean succeeded but Rust returned no result",
        leanResult }
        
  | LeanResult.error _, RustOutcome.noResult =>
      -- Both failed, but Rust returned no error type
      { equivalent := false,
        difference := some "Both failed but Rust returned no error type",
        leanResult }
        
  | LeanResult.success _, RustOutcome.parseError =>
      -- Lean succeeded, Rust parse error
      { equivalent := false,
        difference := some "Lean succeeded but Rust had parse error",
        leanResult }
        
  | LeanResult.error _, RustOutcome.parseError =>
      -- Both failed, Rust parse error
      { equivalent := false,
        difference := some "Both failed and Rust had parse error",
        leanResult }
        
  | _, RustOutcome.parseFailed e =>
      -- Failed to parse Rust result
      { equivalent := false,
        difference := some s!"Failed to parse Rust result: {e}",
        leanResult }

/-- Compare results from JSON strings

Convenience function that takes the workflow JSON and Rust result JSON as strings -/
def compareFromJson (workflowJson : String) (rustResultJson : String) : Except String ComparisonResult := do
  let workflow ← parseWorkflowJson workflowJson
  let rustJson ← Json.parse rustResultJson
  pure (compareResults workflow rustJson)

/-- Run a single differential test and return whether it passed -/
def runSingleTest (testCase : TestCase) (rustResultJson : Json) : ComparisonResult :=
  compareResults testCase.workflow rustResultJson

/-- Create a simple test case from an expression -/
def testCaseFromExpr (name : String) (expr : Expr) : TestCase :=
  { name, workflow := expr }

end Ash.Differential
