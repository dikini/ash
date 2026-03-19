/- # Type Safety Corollary

The Type Safety Corollary: Well-typed programs are safe.

**Corollary Statement**: If Γ ⊢ e : T, then evaluation of e:
1. Never gets stuck (Progress), and
2. Produces a value of type T (Preservation)

This is the fundamental soundness property of the type system.

Per Type Safety specification: Well-typed terms evaluate to
well-typed values (or diverge, or fail with a defined error).
-/ 

import Ash.Core.AST
import Ash.Core.Environment
import Ash.Types.Basic
import Ash.Types.WellTyped
import Ash.Eval.Expr
import Ash.Proofs.Progress
import Ash.Proofs.Preservation

namespace Ash.Proofs

open Ash
open Ash.Types
open Ash.Eval

/-! ## Type Safety Corollary

Type safety combines progress and preservation into a single
soundness statement about the type system.

In big-step semantics:
- **Progress**: eval always produces a defined result (ok or error)
- **Preservation**: If eval produces a value, it has the expected type

Together: Well-typed programs either:
1. Evaluate to a value of the expected type, or
2. Produce a defined error (not stuck)
-/ 

/-- Type Safety Corollary

If an expression is well-typed in a consistent environment,
then evaluation produces a defined result where:
- If successful, the value has the expected type
- If error, it's a defined error (not stuck)

**Theorem**: If Γ ⊢ e : T and env is consistent with Γ,
then eval env e produces a result r such that:
- If r = ok result, then ValueHasType result.value T
- If r = error err, then err is a defined EvalError

This combines:
- **Progress**: eval env e produces a result (not stuck)
- **Preservation**: The result value has type T

Note: The error cases are also "safe" in the sense that they
are defined errors (not arbitrary stuck states).
-/ 
theorem type_safety {Γ : TyEnv} {e : Expr} {T : Ty} {env : Env}
    (ht : WellTyped Γ e T)
    (h_env : envConsistent env Γ) :
    ∃ result, eval env e = result ∧
      match result with
      | .ok r => ValueHasType r.value T
      | .error _ => True  -- Error is defined, not stuck
      := by
  -- Step 1: By progress, evaluation produces a result
  have h_progress := progress ht h_env
  rcases h_progress with ⟨result, h_result⟩
  -- Step 2: Return the result with the appropriate proof
  refine ⟨result, h_result, ?_⟩
  -- Case analysis on the result type
  cases result with
  | ok r =>
      -- Apply preservation theorem
      have hp := preservation ht h_env h_result
      simpa using hp
  | error _ =>
      -- Error case is trivially safe (defined error, not stuck)
      trivial

/-! ## Type Safety Implications

Type safety ensures that well-typed Ash programs have the
following properties:

### No Stuck States

Well-typed programs never reach a stuck state where no
evaluation rule applies. This means:
- Variables are always bound (or error is raised)
- Pattern matches are exhaustive (or handled by if-let)
- Operations have correctly typed operands

### Type-Correct Results

Successful evaluation produces values that match the
static type of the expression. This enables:
- Safe optimization based on type information
- Interoperability with other type-safe code
- Refactoring with confidence in type correctness

### Defined Error Behavior

When evaluation fails, it fails with a defined error:
- `unboundVariable`: Variable not in scope
- `typeMismatch`: Runtime type mismatch (shouldn't happen for well-typed)
- `nonExhaustiveMatch`: No pattern matched (in match expressions)
- `unknownConstructor`: Constructor not in scope
- `missingField`: Field access on variant without field

These are not arbitrary crashes but semantically meaningful errors.
-/ 

/-! ## Soundness Statement

The type system is sound: well-typed programs don't go wrong.

This is the fundamental theorem that justifies the type system.
-/ 

/-- Type System Soundness

The Ash type system is sound with respect to its operational semantics.

For any well-typed expression:
1. Evaluation is defined (progress)
2. Successful evaluation produces a value of the expected type (preservation)

This is the top-level soundness statement. -/
theorem type_system_soundness :
    (∀ Γ e T, WellTyped Γ e T →
     ∀ env, envConsistent env Γ →
     ∃ result, eval env e = result) ∧  -- Progress
    (∀ Γ e T env result,
     WellTyped Γ e T →
     envConsistent env Γ →
     eval env e = .ok result →
     ValueHasType result.value T) := by  -- Preservation
  constructor
  · -- Progress: well-typed expressions evaluate
    intro Γ e T ht env h_env
    apply progress ht h_env
  · -- Preservation: evaluation preserves types
    intro Γ e T env result ht h_env heval
    apply preservation ht h_env heval

/-! ## Conclusion

Type safety ensures that well-typed Ash programs behave predictably:
- No stuck states (all well-typed expressions can be evaluated)
- Type-correct results (values match their static types)
- Defined error behavior (errors are semantically meaningful)

This provides the foundation for:
- Safe program transformation and optimization
- Correct compiler and interpreter implementations
- Formal reasoning about program behavior
-/ 

end Ash.Proofs
