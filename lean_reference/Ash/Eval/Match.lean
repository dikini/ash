-- Ash Match Expression Evaluation
-- Implements match expression evaluation per SPEC-004 Section 5.2
--
-- Match expressions evaluate a scrutinee and select the first matching arm,
-- then evaluate the arm's body in an environment extended with pattern bindings.
--
-- NOTE: This is part of the SIMPLIFIED SUBSET implementation.
-- See docs/AST-Subset.md for the relationship with Rust's full AST.

import Ash.Core.AST
import Ash.Core.Environment
import Ash.Eval.Pattern

namespace Ash.Eval

open Ash

/-! ## Match Expression Evaluation

Per SPEC-004 Section 5.2 (MATCH rules):
The match expression evaluation follows big-step semantics:

```
Γ ⊢ e ⇓ v, ε₁    bind(pat, v, Γ) = Γ'    Γ' ⊢ e' ⇓ v', ε₂
--------------------------------------------------------- [MATCH]
Γ ⊢ match e { pat => e', ... } ⇓ v', ε₁ ⊔ ε₂
```

Key properties:
- Scrutinee is evaluated first
- First matching arm is selected (first-match wins)
- Pattern bindings extend the environment
- Effects accumulate (scrutinee ⊔ body)
- Non-exhaustive matches return an error

-/

/-- Find the first matching arm for a value

Iterates through arms in order, returning the first arm whose pattern
matches the value, along with the bindings produced by the match.

Returns `none` if no arm matches (non-exhaustive match).

Per SPEC-004 Section 5.2: First-match semantics
-/
def findMatchingArm (arms : List MatchArm) (v : Value) : Option (MatchArm × Env) :=
  arms.findSome? (fun arm =>
    match matchPattern arm.pattern v with
    | none => none
    | some bindings => some (arm, bindings))

/-- Evaluate a match expression

Per SPEC-004 Section 5.2:
1. Evaluate scrutinee expression
2. Find first matching arm using `findMatchingArm`
3. Apply pattern bindings to environment (merged with current env)
4. Evaluate arm body in extended environment
5. Accumulate effects from scrutinee and body (scrutinee_effect ⊔ body_effect)
6. Return `nonExhaustiveMatch` error if no arm matches

Arguments:
- eval: The expression evaluator function (passed to avoid circular imports)
- env: Current evaluation environment
- scrutinee: Expression to evaluate and match against
- arms: List of match arms (pattern => body)

Returns:
- `ok result` with value and accumulated effect on success
- `error nonExhaustiveMatch` if no arm matches the scrutinee value
- Other errors from evaluating scrutinee or body expressions
-/
def evalMatch (eval : Env → Expr → Except EvalError EvalResult)
    (env : Env) (scrutinee : Expr) (arms : List MatchArm) : Except EvalError EvalResult := do
  -- Step 1: Evaluate scrutinee
  let scrutResult ← eval env scrutinee

  -- Step 2: Find first matching arm
  match findMatchingArm arms scrutResult.value with
  | none =>
      -- No arm matches - non-exhaustive match
      throw .nonExhaustiveMatch
  | some (arm, bindings) =>
      -- Step 3: Merge pattern bindings with current environment
      -- Bindings from pattern take precedence (right-biased merge)
      let newEnv := Env.merge env bindings

      -- Step 4: Evaluate body in extended environment
      let bodyResult ← eval newEnv arm.body

      -- Step 5: Combine effects (scrutinee_effect ⊔ body_effect)
      pure {
        value := bodyResult.value,
        effect := scrutResult.effect.join bodyResult.effect
      }

end Ash.Eval
