-- Ash Core Environment and Effect Types
-- Defines Env, Effect, EvalResult, EvalError per SPEC-021 and SPEC-004

import Ash.Core.AST
import Plausible

namespace Ash

open Lean
open Plausible

/-! ## Effect Lattice

Effect tracking per SPEC-004 Section 5.2.
The effect lattice orders computational power:
  epistemic < deliberative < evaluative < operational

- epistemic: Read-only, pure computations
- deliberative: Analysis and inspection
- evaluative: Decision making
- operational: Side effects, IO, mutation
-/

/-- Effect lattice tracking computational power per SPEC-004 Section 5.2 -/
inductive Effect where
  | epistemic      -- Read-only, pure
  | deliberative   -- Analysis
  | evaluative     -- Decision
  | operational    -- Side effects
  deriving Repr, BEq, DecidableEq, Inhabited

/-- Pluggable instance for Effect - generates random effects -/
instance : Arbitrary Effect where
  arbitrary := 
    Gen.elements [Effect.epistemic, Effect.deliberative, Effect.evaluative, Effect.operational]
      (by decide)

/-- Shrinkable instance for Effect (no shrinking for finite domain) -/
instance : Shrinkable Effect where
  shrink _ := []

/-- Convert Effect to String -/
def Effect.toString : Effect → String
  | .epistemic => "epistemic"
  | .deliberative => "deliberative"
  | .evaluative => "evaluative"
  | .operational => "operational"

instance : ToString Effect where
  toString := Effect.toString

/-- Convert Effect to JSON (as string) -/
def Effect.toJson (e : Effect) : Json :=
  toString e

instance : ToJson Effect where
  toJson := Effect.toJson

/-- Parse Effect from JSON -/
def Effect.fromJson (json : Json) : Except String Effect := do
  let s ← json.getStr?
  match s with
  | "epistemic" => pure .epistemic
  | "deliberative" => pure .deliberative
  | "evaluative" => pure .evaluative
  | "operational" => pure .operational
  | _ => throw s!"Unknown effect: {s}"

instance : FromJson Effect where
  fromJson? := Effect.fromJson

/-- Effect join (lattice supremum)

Per SPEC-004 Section 5.2:
- epistemic ⊔ e = e  (epistemic is bottom)
- operational ⊔ e = operational  (operational is top)
-/
def Effect.join (e1 e2 : Effect) : Effect :=
  match e1, e2 with
  | _, .operational | .operational, _ => .operational
  | _, .evaluative | .evaluative, _ => .evaluative
  | _, .deliberative | .deliberative, _ => .deliberative
  | _, _ => .epistemic

instance : Max Effect where
  max := Effect.join

/-! ## Environment

Variable binding environment as function String → Option Value.
Provides lexical scoping semantics.
-/

/-- Environment as function from names to values -/
def Env : Type := String → Option Value

/-- Empty environment -/
def Env.empty : Env := fun _ => none

/-- Bind a variable in the environment

Returns a new environment where `x` is bound to `v`.
Shadows any existing binding for `x`. -/
def Env.bind (env : Env) (x : String) (v : Value) : Env :=
  fun y => if x = y then some v else env y

/-- Lookup a variable in the environment -/
def Env.lookup (env : Env) (x : String) : Option Value :=
  env x

/-- Merge two environments (right-biased)

Bindings in env2 take precedence over env1.
This supports lexical scoping for nested let bindings. -/
def Env.merge (env1 env2 : Env) : Env :=
  fun x =>
    match env2.lookup x with
    | some v => some v
    | none => env1.lookup x

/-! ## Evaluation Result and Errors -/

/-- Evaluation result with value and accumulated effect -/
structure EvalResult where
  value : Value
  effect : Effect
  deriving Repr, BEq, Inhabited

/-- Evaluation errors -/
inductive EvalError where
  | unboundVariable (name : String)
  | typeMismatch (expected : String) (actual : String)
  | nonExhaustiveMatch
  | unknownConstructor (name : String)
  | missingField (constructor : String) (field : String)
  deriving Repr, BEq, DecidableEq, Inhabited

/-- Convert EvalError to human-readable String -/
def EvalError.toString : EvalError → String
  | .unboundVariable name => s!"Unbound variable: {name}"
  | .typeMismatch expected actual => 
      s!"Type mismatch: expected {expected}, got {actual}"
  | .nonExhaustiveMatch => "Non-exhaustive pattern match"
  | .unknownConstructor name => s!"Unknown constructor: {name}"
  | .missingField ctor field => 
      s!"Missing field '{field}' in constructor '{ctor}'"

instance : ToString EvalError where
  toString := EvalError.toString

/-- Convert EvalError to JSON -/
def EvalError.toJson : EvalError → Json
  | .unboundVariable name =>
      Json.mkObj [("type", "unboundVariable"), ("name", name)]
  | .typeMismatch expected actual =>
      Json.mkObj [("type", "typeMismatch"), ("expected", expected), ("actual", actual)]
  | .nonExhaustiveMatch =>
      Json.mkObj [("type", "nonExhaustiveMatch")]
  | .unknownConstructor name =>
      Json.mkObj [("type", "unknownConstructor"), ("name", name)]
  | .missingField ctor field =>
      Json.mkObj [("type", "missingField"), ("constructor", ctor), ("field", field)]

instance : ToJson EvalError where
  toJson := EvalError.toJson

/-- Parse EvalError from JSON -/
def EvalError.fromJson (json : Json) : Except String EvalError := do
  let type ← json.getObjValAs? String "type"
  match type with
  | "unboundVariable" =>
      let name ← json.getObjValAs? String "name"
      pure (.unboundVariable name)
  | "typeMismatch" =>
      let expected ← json.getObjValAs? String "expected"
      let actual ← json.getObjValAs? String "actual"
      pure (.typeMismatch expected actual)
  | "nonExhaustiveMatch" =>
      pure .nonExhaustiveMatch
  | "unknownConstructor" =>
      let name ← json.getObjValAs? String "name"
      pure (.unknownConstructor name)
  | "missingField" =>
      let ctor ← json.getObjValAs? String "constructor"
      let field ← json.getObjValAs? String "field"
      pure (.missingField ctor field)
  | _ => throw s!"Unknown EvalError type: {type}"

instance : FromJson EvalError where
  fromJson? := EvalError.fromJson

/-- ToString instance for Option Value using Repr -/
instance : ToString (Option Value) where
  toString
    | some v => s!"some {reprStr v}"
    | none => "none"

/-- ToString instance for EvalResult -/
instance : ToString EvalResult where
  toString r := "{ value := " ++ reprStr r.value ++ ", effect := " ++ toString r.effect ++ " }"

/-- Convert EvalResult to JSON -/
def EvalResult.toJson (r : EvalResult) : Json :=
  Json.mkObj [("value", r.value.toJson), ("effect", r.effect.toJson)]

instance : ToJson EvalResult where
  toJson := EvalResult.toJson

/-- Parse EvalResult from JSON -/
def EvalResult.fromJson (json : Json) : Except String EvalResult := do
  let value ← json.getObjValAs? Value "value"
  let effect ← json.getObjValAs? Effect "effect"
  pure { value, effect }

instance : FromJson EvalResult where
  fromJson? := EvalResult.fromJson

end Ash
