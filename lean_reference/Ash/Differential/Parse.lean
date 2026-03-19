-- Ash Differential Testing Parse
-- Parse Rust JSON output
-- TASK-145: Differential Testing Harness

import Ash.Core.AST
import Ash.Core.Environment
import Ash.Differential.Types

namespace Ash.Differential

open Ash
open Lean

/-- Parse Effect from string representation -/
def parseEffect (s : String) : Except String Effect :=
  match s with
  | "epistemic" => pure .epistemic
  | "deliberative" => pure .deliberative
  | "evaluative" => pure .evaluative
  | "operational" => pure .operational
  | _ => throw s!"Unknown effect: {s}"

/-- Parse a Rust evaluation result from JSON

Expected JSON format for success:
{
  "status": "ok",
  "value": <value_json>,
  "effect": "epistemic" | "deliberative" | "evaluative" | "operational"
}

Expected JSON format for error:
{
  "status": "error",
  "error": {
    "type": "unboundVariable" | "typeMismatch" | ...,
    ...
  }
}
-/
def parseRustResult (json : Json) : Except String (Option EvalResult) := do
  let status ← json.getObjValAs? String "status"
  match status with
  | "ok" => do
      let value ← json.getObjValAs? Value "value"
      let effectStr ← json.getObjValAs? String "effect"
      let effect ← parseEffect effectStr
      pure (some { value, effect })
  | "error" =>
      -- Error case returns none - the caller should check for error type match separately
      pure none
  | _ => throw s!"Unknown status: {status}"

/-- Parse error result from Rust JSON

Returns the error type string if this is an error result
-/
def parseRustError (json : Json) : Except String (Option String) := do
  let status ← json.getObjValAs? String "status"
  match status with
  | "error" =>
      match json.getObjVal? "error" with
      | .error _ => pure none
      | .ok ej =>
          match ej.getObjValAs? String "type" with
          | .ok errorType => pure (some errorType)
          | .error _ => pure none
  | _ => pure none

/-- Parse workflow expression from JSON file content -/
def parseWorkflowJson (jsonStr : String) : Except String Expr := do
  let json ← Json.parse jsonStr
  Expr.fromJson json

/-- Parse a test case from JSON

Expected format:
{
  "name": "test_name",
  "description": "optional description",
  "workflow": <expr_json>
}
-/
def parseTestCase (json : Json) : Except String TestCase := do
  let name ← json.getObjValAs? String "name"
  let description : Option String :=
    match json.getObjVal? "description" with
    | .ok (Json.str s) => some s
    | _ => none
  let workflow ← json.getObjValAs? Expr "workflow"
  pure { name, workflow, description }

end Ash.Differential
