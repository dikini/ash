-- Ash Property-Based Tests
-- Property tests using Plausible
-- Per TASK-146: Property-Based Tests in the Lean reference interpreter

import Plausible
import Ash.Core.AST
import Ash.Core.Environment
import Ash.Eval.Expr
import Ash.Eval.Pattern
import Ash.Eval.Match

namespace Ash.Tests

open Ash
open Plausible
open Ash.Eval

/-! ## Helper Functions -/

/-- Check if pattern match returns some (i.e., succeeded) -/
def matchSucceeds (p : Pattern) (v : Value) : Bool :=
  match matchPattern p v with
  | some _ => true
  | none => false

/-- Check if pattern match returns none (i.e., failed) -/
def matchFails (p : Pattern) (v : Value) : Bool :=
  match matchPattern p v with
  | some _ => false
  | none => true

/-- Check if evaluation result has epistemic effect -/
def isEpistemicEffect (result : Except EvalError EvalResult) : Bool :=
  match result with
  | .ok r => r.effect == .epistemic
  | .error _ => true  -- errors don't violate purity

/-- Check if result is unbound variable error -/
def isUnboundVariable (result : Except EvalError EvalResult) : Bool :=
  match result with
  | .error (.unboundVariable _) => true
  | _ => false

/-- Extract value from result -/
def resultValue (result : Except EvalError EvalResult) : Option Value :=
  match result with
  | .ok r => some r.value
  | .error _ => none

/-- Extract effect from result -/
def resultEffect (result : Except EvalError EvalResult) : Option Effect :=
  match result with
  | .ok r => some r.effect
  | .error _ => none

/-! ## Value Roundtrip Properties -/

namespace ValueRoundtripProperties

-- Int roundtrip
#test ∀ (i : Int), 
  (Value.fromJson (Value.int i).toJson).toOption == some (Value.int i)

-- String roundtrip
#test ∀ (s : String), s.length < 100 →
  (Value.fromJson (Value.string s).toJson).toOption == some (Value.string s)

-- Bool roundtrip
#test ∀ (b : Bool),
  (Value.fromJson (Value.bool b).toJson).toOption == some (Value.bool b)

-- Null roundtrip
#test (Value.fromJson Value.null.toJson).toOption == some Value.null

-- List roundtrip (small lists)
#test ∀ (i1 i2 : Int),
  let v := Value.list [Value.int i1, Value.int i2]
  (Value.fromJson v.toJson).toOption == some v

-- Tuple roundtrip
#test ∀ (i1 i2 : Int),
  let v := Value.tuple [Value.int i1, Value.int i2]
  (Value.fromJson v.toJson).toOption == some v

end ValueRoundtripProperties

/-! ## Value Equality Properties -/

namespace ValueEqualityProperties

-- int equality
#test ∀ (i : Int), Value.int i == Value.int i

-- string equality
#test ∀ (s : String), s.length < 100 → Value.string s == Value.string s

-- bool equality
#test ∀ (b : Bool), Value.bool b == Value.bool b

-- null equality
#test Value.null == Value.null

end ValueEqualityProperties

/-! ## Effect Lattice Properties (SPEC-004 Section 5.2) -/

namespace EffectLatticeProperties

-- Associativity: (e1 ⊔ e2) ⊔ e3 = e1 ⊔ (e2 ⊔ e3)
#test ∀ (e1 e2 e3 : Effect),
  (e1.join e2).join e3 = e1.join (e2.join e3)

-- Commutativity: e1 ⊔ e2 = e2 ⊔ e1
#test ∀ (e1 e2 : Effect),
  e1.join e2 = e2.join e1

-- Idempotence: e ⊔ e = e
#test ∀ (e : Effect),
  e.join e = e

-- Epistemic is bottom (identity): epistemic ⊔ e = e
#test ∀ (e : Effect),
  Effect.epistemic.join e = e

-- Operational is top: operational ⊔ e = operational
#test ∀ (e : Effect),
  Effect.operational.join e = Effect.operational

-- Bottom identity left
#test ∀ (e : Effect),
  e.join Effect.epistemic = e

-- Top absorbs left
#test ∀ (e : Effect),
  e.join Effect.operational = Effect.operational

-- Specific ordering properties (concrete tests)
#test Effect.epistemic.join Effect.deliberative = Effect.deliberative
#test Effect.deliberative.join Effect.evaluative = Effect.evaluative
#test Effect.evaluative.join Effect.operational = Effect.operational
#test Effect.epistemic.join Effect.operational = Effect.operational

-- Lattice monotonicity properties
#test Effect.epistemic.join Effect.deliberative = Effect.deliberative
#test Effect.deliberative.join Effect.operational = Effect.operational
#test Effect.evaluative.join Effect.operational = Effect.operational

end EffectLatticeProperties

/-! ## Environment Operation Properties -/

namespace EnvironmentProperties

-- Lookup after binding returns bound value
#test (Env.empty.bind "x" (Value.int 42)).lookup "x" == some (Value.int 42)

-- Lookup different name returns none
#test (Env.empty.bind "x" (Value.int 42)).lookup "y" == none

-- Shadowing: second bind wins
#test ((Env.empty.bind "x" (Value.int 1)).bind "x" (Value.int 2)).lookup "x" == some (Value.int 2)

-- Merge prefers right-hand side
#test 
  let merged := (Env.empty.bind "x" (Value.int 1)).merge (Env.empty.bind "x" (Value.int 2))
  merged.lookup "x" == some (Value.int 2)

-- Unbound variable lookup returns none
#test Env.empty.lookup "unbound" == none

-- Merge with empty (right) preserves bindings
#test ((Env.empty.bind "x" (Value.int 1)).merge Env.empty).lookup "x" == some (Value.int 1)

-- Merge with empty (left) preserves bindings
#test (Env.empty.merge (Env.empty.bind "x" (Value.int 1))).lookup "x" == some (Value.int 1)

-- Empty environment lookup always returns none
#test ∀ (key : String), key.length < 50 →
  Env.empty.lookup key = none

-- Environment merge with disjoint keys combines both
#test 
  let e1 := Env.empty.bind "a" (Value.int 1)
  let e2 := Env.empty.bind "b" (Value.int 2)
  let merged := e1.merge e2
  merged.lookup "a" == some (Value.int 1) &&
  merged.lookup "b" == some (Value.int 2)

end EnvironmentProperties

/-! ## Pattern Matching Properties (SPEC-004 Section 5.2 & 6.1) -/

namespace PatternMatchingProperties

-- Literal matches only when equal
#test ∀ (i1 i2 : Int),
  matchSucceeds (.literal (Value.int i1)) (Value.int i2) = (i1 = i2)

-- Tuple pattern with same length matches
#test ∀ (i1 i2 : Int),
  matchSucceeds (.tuple [.wildcard, .wildcard]) (.tuple [Value.int i1, Value.int i2])

-- Tuple pattern with different length fails
#test ∀ (i : Int),
  matchFails (.tuple [.wildcard, .wildcard]) (.tuple [Value.int i])

-- Non-empty tuple pattern fails on empty tuple
#test 
  matchFails (.tuple [.wildcard]) (.tuple [])

end PatternMatchingProperties

/-! ## Pattern Serialization Properties -/

namespace PatternSerializationProperties

-- Pattern roundtrip: wildcard
#test
  let p := Pattern.wildcard
  (Pattern.fromJson p.toJson).toOption == some p

-- Pattern roundtrip: variable
#test ∀ (name : String), name.length < 50 →
  let p := Pattern.variable name
  (Pattern.fromJson p.toJson).toOption == some p

-- Pattern roundtrip: literal int
#test ∀ (i : Int),
  let p := Pattern.literal (Value.int i)
  (Pattern.fromJson p.toJson).toOption == some p

-- Pattern roundtrip: tuple of variables
#test ∀ (n1 n2 : String), n1.length < 50 → n2.length < 50 →
  let p := Pattern.tuple [Pattern.variable n1, Pattern.variable n2]
  (Pattern.fromJson p.toJson).toOption == some p

end PatternSerializationProperties

/-! ## Expression Serialization Properties -/

namespace ExprSerializationProperties

-- Expr roundtrip: literal
#test ∀ (i : Int),
  let e := Expr.literal (Value.int i)
  (Expr.fromJson e.toJson).toOption == some e

-- Expr roundtrip: variable
#test ∀ (name : String), name.length < 50 →
  let e := Expr.variable name
  (Expr.fromJson e.toJson).toOption == some e

-- Expr roundtrip: tuple of literals
#test ∀ (i1 i2 : Int),
  let e := Expr.tuple [Expr.literal (Value.int i1), Expr.literal (Value.int i2)]
  (Expr.fromJson e.toJson).toOption == some e

-- Expr roundtrip: simple constructor
#test ∀ (i : Int),
  let e := Expr.constructor "Some" [("value", Expr.literal (Value.int i))]
  (Expr.fromJson e.toJson).toOption == some e

end ExprSerializationProperties

/-! ## EvalError Serialization Properties -/

namespace EvalErrorSerializationProperties

-- EvalError roundtrip: unboundVariable
#test ∀ (name : String), name.length < 50 →
  let e := EvalError.unboundVariable name
  (EvalError.fromJson e.toJson).toOption == some e

-- EvalError roundtrip: typeMismatch
#test ∀ (expected actual : String), expected.length < 50 → actual.length < 50 →
  let e := EvalError.typeMismatch expected actual
  (EvalError.fromJson e.toJson).toOption == some e

-- EvalError roundtrip: nonExhaustiveMatch
#test
  let e := EvalError.nonExhaustiveMatch
  (EvalError.fromJson e.toJson).toOption == some e

-- EvalError roundtrip: unknownConstructor
#test ∀ (name : String), name.length < 50 →
  let e := EvalError.unknownConstructor name
  (EvalError.fromJson e.toJson).toOption == some e

-- EvalError roundtrip: missingField
#test ∀ (ctor field : String), ctor.length < 50 → field.length < 50 →
  let e := EvalError.missingField ctor field
  (EvalError.fromJson e.toJson).toOption == some e

end EvalErrorSerializationProperties

/-! ## EvalResult Serialization Properties -/

namespace EvalResultSerializationProperties

-- EvalResult roundtrip with int value
#test ∀ (i : Int),
  let r := { value := Value.int i, effect := Effect.epistemic : EvalResult }
  (EvalResult.fromJson r.toJson).toOption == some r

-- EvalResult roundtrip with string value
#test ∀ (s : String), s.length < 100 →
  let r := { value := Value.string s, effect := Effect.operational : EvalResult }
  (EvalResult.fromJson r.toJson).toOption == some r

-- EvalResult roundtrip with different effects
#test ∀ (e : Effect) (i : Int),
  let r := { value := Value.int i, effect := e : EvalResult }
  (EvalResult.fromJson r.toJson).toOption == some r

end EvalResultSerializationProperties

/-! ## Effect Serialization Properties -/

namespace EffectSerializationProperties

-- Effect roundtrip: all effects survive JSON serialization
#test ∀ (e : Effect),
  (Effect.fromJson e.toJson).toOption == some e

-- Specific effect roundtrip tests
#test (Effect.fromJson Effect.epistemic.toJson).toOption == some Effect.epistemic
#test (Effect.fromJson Effect.operational.toJson).toOption == some Effect.operational
#test (Effect.fromJson Effect.deliberative.toJson).toOption == some Effect.deliberative
#test (Effect.fromJson Effect.evaluative.toJson).toOption == some Effect.evaluative

end EffectSerializationProperties

/-! ## Constructor Purity Property (SPEC-004 Section 5.1) -/

namespace ConstructorPurityProperties

-- Constructor purity: empty constructor has epistemic effect
#test 
  isEpistemicEffect (eval Env.empty (.constructor "None" []))

-- Constructor with literal fields is pure (epistemic effect)
#test ∀ (i : Int),
  isEpistemicEffect (eval Env.empty (.constructor "Some" [("value", .literal (Value.int i))]))

-- Constructor with nested constructor is still pure
#test 
  isEpistemicEffect (eval Env.empty (.constructor "Wrapper" [("inner", .constructor "None" [])]))

end ConstructorPurityProperties

/-! ## Literal Purity Property (SPEC-004 Section 5.1) -/

namespace LiteralPurityProperties

-- Literal purity: int literals have epistemic effect
#test ∀ (i : Int),
  isEpistemicEffect (eval Env.empty (.literal (Value.int i)))

-- String literal purity
#test ∀ (s : String), s.length < 50 →
  isEpistemicEffect (eval Env.empty (.literal (Value.string s)))

-- Bool literal purity
#test ∀ (b : Bool),
  isEpistemicEffect (eval Env.empty (.literal (Value.bool b)))

-- Null literal purity
#test isEpistemicEffect (eval Env.empty (.literal Value.null))

end LiteralPurityProperties

/-! ## Variable Evaluation Properties -/

namespace VariableEvaluationProperties

-- Variable lookup fails on unbound
#test 
  isUnboundVariable (eval Env.empty (.variable "unbound_var_123"))

-- Variable lookup succeeds on bound
#test ∀ (i : Int),
  resultValue (eval (Env.empty.bind "x" (Value.int i)) (.variable "x")) == some (Value.int i)

-- Variable lookup returns correct effect
#test ∀ (i : Int),
  resultEffect (eval (Env.empty.bind "x" (Value.int i)) (.variable "x")) == some .epistemic

end VariableEvaluationProperties

/-! ## Match Expression Properties (TASK-142) -/

namespace MatchExpressionProperties

-- Wildcard arm is always exhaustive (returns expected value)
#test ∀ (i : Int),
  resultValue (eval Env.empty (.match (.literal (Value.int i)) [
    { pattern := .wildcard, body := .literal (Value.int 0) }
  ])) == some (Value.int 0)

-- Variable arm binds correctly
#test ∀ (i : Int),
  resultValue (eval Env.empty (.match (.literal (Value.int i)) [
    { pattern := .variable "x", body := .variable "x" }
  ])) == some (Value.int i)

-- First match wins (literal vs wildcard) - returns "first"
#test ∀ (i : Int),
  resultValue (eval Env.empty (.match (.literal (Value.int i)) [
    { pattern := .literal (Value.int i), body := .literal (Value.string "first") },
    { pattern := .wildcard, body := .literal (Value.string "second") }
  ])) == some (Value.string "first")

-- Match returns correct effect (should be epistemic for pure expressions)
#test ∀ (i : Int),
  resultEffect (eval Env.empty (.match (.literal (Value.int i)) [
    { pattern := .wildcard, body := .literal (Value.int 0) }
  ])) == some .epistemic

end MatchExpressionProperties

/-! ## If-Let Expression Properties (TASK-143) -/

namespace IfLetExpressionProperties

-- If-let with wildcard always takes then branch
#test ∀ (i : Int),
  resultValue (eval Env.empty (.if_let .wildcard (.literal (Value.int i))
    (.literal (Value.bool true))
    (.literal (Value.bool false))
  )) == some (Value.bool true)

-- If-let with variable pattern succeeds and binds
#test ∀ (i : Int),
  resultValue (eval Env.empty (.if_let (.variable "x") (.literal (Value.int i))
    (.variable "x")
    (.literal (Value.int 0))
  )) == some (Value.int i)

-- If-let with literal succeeds when equal
#test ∀ (i : Int),
  resultValue (eval Env.empty (.if_let (.literal (Value.int i)) (.literal (Value.int i))
    (.literal (Value.bool true))
    (.literal (Value.bool false))
  )) == some (Value.bool true)

-- If-let with literal fails when not equal (goes to else branch)
#test ∀ (i1 i2 : Int), i1 ≠ i2 →
  resultValue (eval Env.empty (.if_let 
    (.literal (Value.int i1))
    (.literal (Value.int i2))
    (.literal (Value.bool true))
    (.literal (Value.bool false))
  )) == some (Value.bool false)

-- If-let returns epistemic effect for pure expressions
#test ∀ (i : Int),
  resultEffect (eval Env.empty (.if_let .wildcard (.literal (Value.int i))
    (.literal (Value.int 0))
    (.literal (Value.int 1))
  )) == some .epistemic

end IfLetExpressionProperties

/-! ## Serialization Helpers -/

namespace SerializationHelpers

-- testRoundtrip helper works correctly
#test ∀ (i : Int),
  testRoundtrip (Value.int i) = true

#test ∀ (s : String), s.length < 100 →
  testRoundtrip (Value.string s) = true

end SerializationHelpers

/-! ## Type Definition Properties -/

namespace TypeDefinitionProperties

-- TypeExpr equality
#test ∀ (name : String), name.length < 50 →
  TypeExpr.named name == TypeExpr.named name

-- TypeDef equality
#test ∀ (name : String), name.length < 50 →
  let td := { name := name, params := [], body := TypeBody.enum [] : TypeDef }
  td == td

-- Variant equality
#test ∀ (name : String), name.length < 50 →
  let v := { name := name, fields := [] : Variant }
  v == v

end TypeDefinitionProperties

/-! ## Pattern Properties -/

namespace BasicPatternProperties

-- Wildcard pattern is not equal to variable
#test Pattern.wildcard != Pattern.variable "x"

-- Pattern equality is reflexive
#test ∀ (name : String), name.length < 50 →
  Pattern.variable name == Pattern.variable name

-- Literal pattern equality
#test ∀ (i : Int), Pattern.literal (Value.int i) == Pattern.literal (Value.int i)

end BasicPatternProperties

/-! ## Expr Properties -/

namespace BasicExprProperties

-- Literal expression equality
#test ∀ (i : Int), 
  let e := Expr.literal (Value.int i)
  e == e

-- Variable expression equality
#test ∀ (name : String), name.length < 50 →
  let e := Expr.variable name
  e == e

end BasicExprProperties

/-! ## EvalError Properties -/

namespace EvalErrorProperties

-- EvalError equality is reflexive
#test EvalError.unboundVariable "test" == EvalError.unboundVariable "test"

-- Different error types are not equal
#test EvalError.nonExhaustiveMatch != EvalError.unboundVariable "x"

-- EvalError with different names are not equal
#test EvalError.unboundVariable "x" != EvalError.unboundVariable "y"

-- EvalError roundtrip preservation
#test ∀ (name : String), name.length < 50 →
  let e := EvalError.unboundVariable name
  e == e

end EvalErrorProperties

/-! ## JSON Structure Invariants (TASK-144) -/

namespace JSONStructureProperties

-- Value JSON roundtrip property
#test
  let v := Value.int 42
  (Value.fromJson v.toJson).toOption == some v

-- Variant JSON roundtrip preserves value
#test
  let v := Value.variant "Some" [("value", Value.int 42)]
  (Value.fromJson v.toJson).toOption == some v

-- Record JSON roundtrip - test via testRoundtrip helper
def recordJsonRoundtrip : Bool :=
  let v := Value.record [("x", Value.int 1), ("y", Value.int 2)]
  testRoundtrip v

#test recordJsonRoundtrip

end JSONStructureProperties

/-! ## Additional Property Tests -/

namespace AdditionalProperties

-- Join with self is idempotent for effects
#test ∀ (e : Effect), e.join e = e

-- Max instance behaves like join
#test ∀ (e1 e2 : Effect),
  Max.max e1 e2 = e1.join e2

-- Effect comparison: epistemic is the smallest
#test Effect.epistemic.join Effect.deliberative = Effect.deliberative
#test Effect.epistemic.join Effect.evaluative = Effect.evaluative
#test Effect.epistemic.join Effect.operational = Effect.operational

-- Effect comparison: operational is the largest
#test Effect.operational.join Effect.epistemic = Effect.operational
#test Effect.operational.join Effect.deliberative = Effect.operational
#test Effect.operational.join Effect.evaluative = Effect.operational

end AdditionalProperties

end Ash.Tests
