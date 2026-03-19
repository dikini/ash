/- # Preservation Theorem

The Preservation Theorem: Types are preserved during evaluation.

**Theorem Statement**: If Γ ⊢ e : T and e evaluates to v,
then v has type T.

In big-step semantics: If Γ ⊢ e : T and eval env e = ok result,
then result.value has type T.

Per Type Safety specification: Evaluation preserves types.
-/ 

import Ash.Core.AST
import Ash.Core.Environment
import Ash.Types.Basic
import Ash.Types.WellTyped
import Ash.Eval.Expr
import Ash.Proofs.Progress

namespace Ash.Proofs

open Ash
open Ash.Types
open Ash.Eval

/-! ## Preservation Theorem

The preservation theorem for big-step semantics.

In our context: If an expression is well-typed and evaluates successfully,
the resulting value has the expected type.
-/ 

/-! ## ValueHasType

The `ValueHasType` relation is defined in `Ash.Types.Basic` to avoid
circular dependencies between proof modules.

See `Ash.Types.Basic` for the definition.
-/

/-- Preservation Theorem

If an expression is well-typed and evaluates successfully,
the result value has the expected type.

**Theorem**: If Γ ⊢ e : T and eval env e = ok result,
then ValueHasType result.value T.

**Assumptions**:
- The expression is well-typed: `Γ ⊢ e : T`
- The environment is type-consistent: all values have their declared types
- Evaluation succeeds: `eval env e = ok result`

**Conclusion**: `ValueHasType result.value T`

TODO: Complete proof when eval is made total (see TASK-XXX). -/
theorem preservation {Γ : TyEnv} {e : Expr} {T : Ty} {env : Env} {result : EvalResult}
    (ht : WellTyped Γ e T)
    (h_env : envConsistent env Γ)
    (heval : eval env e = .ok result) :
    ValueHasType result.value T := by
  sorry

/-! ## Preservation Cases (Conceptual)

A complete preservation proof would proceed by induction on the
evaluation derivation:

### Base Cases

1. **Literal**: `eval env (literal v) = ok { value := v, effect := epistemic }`
   - By T-LITERAL, `Γ ⊢ literal v : T` where T is the literal's type
   - By ValueHasType rule for literals, `ValueHasType v T`

2. **Variable**: `eval env (variable x) = ok { value := env(x), effect := epistemic }`
   - By T-VAR, Γ(x) = T
   - By env consistency, `ValueHasType env(x) T`

### Inductive Cases

3. **Tuple**: `eval env (tuple es) = evalTuple env es`
   - By T-TUPLE, each element has the corresponding type
   - By IH, each evaluated element has the right type
   - Result tuple has tuple type with typed elements

4. **Constructor**: `eval env (constructor name fields) = evalConstructor env name fields`
   - By T-CONSTRUCTOR, each field has the expected type
   - By IH, each evaluated field has the right type
   - Result variant has variant type with typed fields

5. **If-let**: `eval env (if_let p e e1 e2)`
   - Case: Pattern matches → then-branch evaluated
     - By pattern typing, bindings have correct types
     - By IH, result of then-branch has type T
   - Case: Pattern doesn't match → else-branch evaluated
     - By IH, result of else-branch has type T

### Key Lemmas

- `env_consistency_preserved`: Pattern binding preserves consistency
- `matchPattern_preserves_types`: Pattern matching produces correctly typed bindings
- `merge_env_preserves_consistency`: Merging consistent environments
-/ 

end Ash.Proofs
