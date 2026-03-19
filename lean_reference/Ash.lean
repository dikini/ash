/-! # Ash Reference Interpreter

Main library module for the Ash reference interpreter in Lean 4.

This module re-exports all public types and functions from the core and evaluation modules.

## Quick Start

```lean
import Ash

#eval eval Env.empty (.literal (Value.int 42))
```

## Modules

- `Ash.Core.AST` - Core AST types (Value, Expr, Pattern)
- `Ash.Core.Environment` - Environment and effect types
- `Ash.Core.Serialize` - JSON serialization
- `Ash.Eval.Expr` - Expression evaluation
- `Ash.Eval.Pattern` - Pattern matching
- `Ash.Eval.Match` - Match expression evaluation
- `Ash.Eval.IfLet` - If-let expression evaluation

## References

- [SPEC-004: Operational Semantics](../../docs/spec/SPEC-004-SEMANTICS.md)
- [SPEC-021: Lean Reference](../../docs/spec/SPEC-021-LEAN-REFERENCE.md)
-/]

-- Core types
export Ash.Core.AST (Value Expr Pattern MatchArm)
export Ash.Core.Environment (Env Effect EvalResult EvalError)

-- Core operations
export Ash.Core.Environment (Env.empty mergeEnvs)
export Ash.Core.Environment (Effect.epistemic Effect.deliberative Effect.evaluative Effect.operational)
export Ash.Core.Environment (EvalResult.mk EvalResult.value EvalResult.effect)
export Ash.Core.Environment (EvalError.unboundVariable EvalError.typeMismatch EvalError.nonExhaustiveMatch EvalError.unknownConstructor EvalError.missingField)

-- Evaluation functions
export Ash.Eval.Expr (eval)
export Ash.Eval.Pattern (matchPattern)
export Ash.Eval.Match (evalMatch)
export Ash.Eval.IfLet (evalIfLet)

-- Serialization
export Ash.Core.Serialize

-- Proof modules
export Ash.Proofs.Pattern (matchPattern_deterministic)
export Ash.Proofs.Pure (constructor_purity)
export Ash.Proofs.Determinism (eval_deterministic)
