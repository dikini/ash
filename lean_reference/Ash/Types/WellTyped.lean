/- # Ash Type System - Well-Typed Relation

Inductive definition of the well-typed relation for expressions.
Per SPEC-004 Section 4 (Type System).

## Implementation Note: Simplified Type System

This module provides a **simplified** version of the well-typed relation
due to Lean 4 technical limitations with nested inductive types.

### Current Limitations

1. **Tuple Typing**: The current definition only checks that the result type
   is a tuple type, not that element types match:
   ```lean
   | .tuple _ => ∃ Ts, T = .tuple Ts  -- Simplified
   ```
   Full version would check: `List.Forall₂ (WellTyped Γ) es Ts`

2. **Constructor Typing**: Only checks that the result is a variant type,
   not that fields match the variant definition:
   ```lean
   | .constructor name _ => ∃ vname vfields, T = .variant vname vfields
   ```
   Full version would check field types against the variant declaration.

3. **Pattern Typing**: `PatternMatches` and `extendEnvForPattern` are
   simplified - they don't fully traverse pattern structure.

4. **Match/If-let**: Always considered well-typed (simplified).

### Why These Limitations

Lean 4 has restrictions on nested inductive types (inductive types that
refer to themselves under other type constructors like `List`). The full
SPEC-004 type system would require nested inductives for:
- `WellTyped` for tuples with element type checking
- `PatternMatches` with structural pattern decomposition

### Future Work

See long-term tasks for completing the full type system using:
- Well-founded recursion
- Mutual inductives with proper structure
- Fuel-based evaluation to enable proofs

For now, this simplified version is sufficient to state and prove the
structure of type safety theorems (progress and preservation).
-/ 

import Ash.Core.AST
import Ash.Core.Environment
import Ash.Types.Basic

namespace Ash.Types

open Ash

/-! ## Type Environments

Type environments (Γ) map variables to their types.
-/ 

def TyEnv := String → Option Ty

namespace TyEnv

/-- Empty type environment -/
def empty : TyEnv := fun _ => none

/-- Bind a variable to a type in the environment -/
def bind (Γ : TyEnv) (x : String) (T : Ty) : TyEnv :=
  fun y => if x = y then some T else Γ y

/-- Look up a variable's type -/
def lookup (Γ : TyEnv) (x : String) : Option Ty :=
  Γ x

end TyEnv

/-! ## Pattern Typing (Simplified)

Helper definitions for pattern typing.

See module documentation for limitations.
-/ 

/-- PatternMatches: Pattern p matches values of type T.

Simplified version - see module documentation for full specification. -/
def PatternMatches : Pattern → Ty → Prop
  | .wildcard, _ => True
  | .variable _, _ => True
  | .literal (.int _), .int => True
  | .literal (.string _), .string => True
  | .literal (.bool _), .bool => True
  | .tuple _, .tuple _ => True  -- Simplified: should check element types
  | .variant name _, .variant vname _ => name = vname  -- Simplified: should check fields
  | .record _, .record _ => True  -- Simplified: should check field types
  | _, _ => False

/-- Extend type environment with pattern bindings (simplified).

Simplified version - see module documentation for full specification. -/
def extendEnvForPattern (Γ : TyEnv) (p : Pattern) (T : Ty) : TyEnv :=
  match p with
  | .wildcard => Γ
  | .variable x => Γ.bind x T
  | .literal _ => Γ
  | .tuple _ => Γ  -- Simplified: should extend with element bindings
  | .variant _ _ => Γ  -- Simplified: should extend with field bindings
  | .record _ => Γ  -- Simplified: should extend with field bindings

/-! ## Well-Typed Relation (Simplified)

The `WellTyped` relation defines when an expression is well-typed in a
given type environment.

Simplified version - see module documentation for full specification.

Judgment form: `Γ ⊢ e : T` means "in environment Γ, expression e has type T"
Per SPEC-004 Section 4.2: Typing rules.
-/ 

/-- WellTyped: Γ ⊢ e : T means "in environment Γ, expression e has type T".

Simplified axiomatic presentation due to Lean 4's limitations
with nested inductive types. See module documentation for details.

TODO: Complete full type system (see long-term tasks). -/
def WellTyped (Γ : TyEnv) (e : Expr) (T : Ty) : Prop :=
  match e with
  | .literal (.int _) => T = .int
  | .literal (.string _) => T = .string
  | .literal (.bool _) => T = .bool
  | .literal _ => False  -- Other literals not supported
  | .variable x => TyEnv.lookup Γ x = some T
  | .tuple _ => ∃ Ts, T = .tuple Ts  -- Simplified: should check element types
  | .constructor _ _ => ∃ vname vfields, T = .variant vname vfields  -- Simplified
  | .match _ _ => True  -- Simplified: should check arm types
  | .if_let _ _ _ _ => True  -- Simplified: should check branch types

end Ash.Types
