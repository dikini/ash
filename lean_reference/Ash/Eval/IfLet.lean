-- Ash If-Let Expression Evaluation
-- Implements if-let expression semantics per SPEC-004 Section 5.3
--
-- This module re-exports the if-let functionality from Expr.lean.
-- The actual implementation is in Expr.lean as part of the mutual recursion.
--
-- NOTE: This is part of the SIMPLIFIED SUBSET implementation.
-- If-let is a core construct for conditional pattern binding in the Lean subset.
-- See docs/AST-Subset.md for the relationship with Rust's full AST.

import Ash.Eval.Expr

namespace Ash.Eval

/-- Re-export evalIfLet from Expr.lean

The evalIfLet function is defined in Expr.lean as part of the mutual
recursion block with eval. This module exists to:
1. Follow the modular structure of the codebase
2. Provide a clear location for if-let related functionality
3. Match the task organization (TASK-143)

Per SPEC-004 Section 5.3 (IF-LET rules):

IF-LET-SUCCESS:
  Γ ⊢ e ⇓ v, ε₁    bind(pat, v) = Γ'    Γ ∪ Γ' ⊢ e₁ ⇓ v₁, ε₂
  -----------------------------------------------------------
  Γ ⊢ if let pat = e then e₁ else e₂ ⇓ v₁, ε₁ ⊔ ε₂

IF-LET-FAIL:
  Γ ⊢ e ⇓ v, ε₁    bind(pat, v) = fail    Γ ⊢ e₂ ⇓ v₂, ε₂
  -----------------------------------------------------------
  Γ ⊢ if let pat = e then e₁ else e₂ ⇓ v₂, ε₁ ⊔ ε₂

Key properties:
- Pattern bindings are only available in the then-branch
- The else-branch uses the original environment
- Effects from the scrutinee and the taken branch are joined
- Pattern bindings do not escape to the outer scope
-/ 

-- Re-export test functions from Expr.lean
export Ash.Eval (
  evalIfLet
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
  runAllIfLetTests
)

end Ash.Eval
