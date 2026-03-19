-- Ash Core AST Types
-- Defines Value, Expr, Pattern, and related types

namespace Ash

/-- Placeholder for Value type - defined in TASK-138 -/
inductive Value where
  | placeholder
  deriving Repr, BEq

/-- Placeholder for Expr type - defined in TASK-138 -/
inductive Expr where
  | placeholder
  deriving Repr, BEq

/-- Placeholder for Pattern type - defined in TASK-138 -/
inductive Pattern where
  | placeholder
  deriving Repr, BEq

/-- Placeholder for MatchArm - defined in TASK-138 -/
structure MatchArm where
  pattern : Pattern
  body : Expr
  deriving Repr, BEq

end Ash
