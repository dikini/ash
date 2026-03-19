# Architecture Overview

This document describes the architecture and design of the Ash Reference Interpreter in Lean 4.

## Design Principles

1. **Executable Specification**: Code follows [SPEC-004](../../docs/spec/SPEC-004-SEMANTICS.md) exactly
2. **Provability**: Structure supports future formal proofs
3. **Testability**: Extensive property tests
4. **Compatibility**: JSON bridge for Rust interop

## Module Structure

```
Ash/
├── Core/           # Fundamental types and operations
│   ├── AST.lean    # Expression and value types
│   ├── Environment.lean  # Env, Effect, EvalResult, EvalError
│   └── Serialize.lean    # JSON serialization
│
├── Eval/           # Interpreter implementation
│   ├── Expr.lean   # Core evaluator (eval function)
│   ├── Pattern.lean  # Pattern matching (matchPattern)
│   ├── Match.lean    # Match expressions (evalMatch)
│   └── IfLet.lean    # If-let expressions (evalIfLet)
│
├── Differential/   # Testing infrastructure
│   ├── Types.lean  # Comparison types
│   ├── Parse.lean  # Parse Rust JSON output
│   └── Compare.lean  # Result comparison
│
└── Tests/          # Test suite
    ├── Properties.lean  # Property-based tests
    ├── Runner.lean      # Test runner
    └── CI.lean          # CI integration
```

## Core Design Decisions

### Environment as Function

```lean
def Env : Type := String → Option Value
```

**Why a function?**

1. **Immutable by default**: No mutation, easy to reason about
2. **Natural shadowing**: `env.bind x v` returns new function
3. **Easy to extend**: Function composition
4. **Supports proof**: Can prove properties about environment lookup

**Implementation:**
```lean
def Env.empty : Env := fun _ => none

def Env.bind (env : Env) (name : String) (value : Value) : Env :=
  fun n => if n = name then some value else env n

def Env.lookup (env : Env) (name : String) : Option Value :=
  env name
```

**Comparison with HashMap:**

| Aspect | Function | HashMap |
|--------|----------|---------|
| Lookup | O(1) apply | O(1) hash |
| Extend | O(1) closure | O(1) insert |
| Immutable | Yes (natural) | Yes (copy) |
| Provable | Yes | Harder |
| Debug | Harder | Easier |

### Effect Lattice

```lean
inductive Effect where
  | epistemic | deliberative | evaluative | operational
```

**Semilattice Structure:**

```
operational
     ▲
     │
evaluative
     ▲
     │
deliberative
     ▲
     │
epistemic
```

**Join Operation (⊔):**

```lean
def Effect.join (e1 e2 : Effect) : Effect :=
  match e1, e2 with
  | operational, _ => operational
  | _, operational => operational
  | evaluative, _ => evaluative
  | _, evaluative => evaluative
  | deliberative, _ => deliberative
  | _, deliberative => deliberative
  | epistemic, epistemic => epistemic
```

**Properties:**
- Commutative: `a ⊔ b = b ⊔ a`
- Associative: `(a ⊔ b) ⊔ c = a ⊔ (b ⊔ c)`
- Idempotent: `a ⊔ a = a`

### Except for Errors

Using `Except EvalError EvalResult` for error handling:

```lean
def eval (env : Env) (expr : Expr) : Except EvalError EvalResult
```

**Benefits:**
- Explicit error propagation
- No exceptions or panics
- Composable with `do` notation
- Type-safe error handling

**Example:**
```lean
def evalConstructor (env : Env) (name : String) (fields : List (String × Expr)) : Except EvalError EvalResult := do
  -- Evaluate each field
  let results ← fields.mapM (fun (n, e) => do
    let r ← eval env e  -- Error short-circuits here
    pure (n, r))
  -- Combine effects
  let effect := results.foldl (fun acc (_, r) => acc.join r.effect) Effect.epistemic
  pure { value := ..., effect := effect }
```

### Partial Functions

Pattern matching is marked `partial` for now:

```lean
partial def matchPattern (p : Pattern) (v : Value) : Option Env
```

**Why partial?**
- Recursive on both pattern and value structure
- Termination proof is non-trivial
- Lean can't automatically prove termination

**Future Work:**
```lean
-- Prove termination and totality
def matchPattern (p : Pattern) (v : Value) : Option Env :=
  ...
termination_by sizeOf p + sizeOf v
```

## Data Flow

```
┌─────────────┐     ┌──────────┐     ┌──────┐     ┌───────────┐     ┌────────┐     ┌──────────┐
│ Input (JSON)│────▶│  Parse   │────▶│ Expr │────▶│   Eval    │────▶│ Result │────▶│ JSON Out │
└─────────────┘     └──────────┘     └──────┘     └───────────┘     └────────┘     └──────────┘
                                                          │
                                                          ▼
                                                   ┌───────────┐
                                                   │   Env     │
                                                   │ (function)│
                                                   └───────────┘
```

## Evaluation Semantics

### Big-Step Semantics

The evaluator implements big-step operational semantics:

```
           (Rule name)
─────────────────────────────
  env ⊢ expr ⇓ result
```

**Example - Literal:**
```
─────────────────────── (E-Literal)
  env ⊢ literal v ⇓ ⟨v, ε⟩
```

**Example - Variable:**
```
  env(x) = v
─────────────────────── (E-Var)
  env ⊢ x ⇓ ⟨v, ε⟩
```

**Example - Match:**
```
  env ⊢ scrutinee ⇓ ⟨v, σ⟩
  matchPattern(p₁, v) = some env'
  env' ⊢ body₁ ⇓ r
────────────────────────────────── (E-Match-1)
  env ⊢ match scrutinee [p₁⇒body₁, ...] ⇓ r ⊔ σ
```

See [SPEC-004](../../docs/spec/SPEC-004-SEMANTICS.md) for complete semantics.

## Testing Strategy

### 1. Unit Tests

Basic functionality via `#eval`:

```lean
#eval eval Env.empty (.literal (Value.int 42))
-- ok { value := int 42, effect := epistemic }
```

### 2. Property Tests

Invariants via Plausible:

```lean
#test ∀ (e1 e2 : Effect), e1.join e2 = e2.join e1
#test ∀ (v : Value), matchPattern .wildcard v ≠ none
```

### 3. Differential Tests

Compare with Rust:

```bash
./scripts/differential_test.sh --count 1000
```

## Serialization

### JSON Bridge

For differential testing, we need compatible JSON:

```lean
-- Lean to JSON
(Value.int 42).toJson  -- {"type": "int", "value": 42}

-- JSON to Lean
Value.fromJson json    -- Except String Value
```

**Format Agreement:**
- Both Lean and Rust use same JSON schema
- Field names match exactly
- Effect strings are lowercase

## Future Extensions

### Type Safety Proofs

```lean
theorem preservation {env : Env} {e : Expr} {τ : Type}
  (h : WellTyped env e τ) :
  eval env e = some v → typeof v = τ
```

### Pattern Match Proofs

```lean
theorem match_deterministic {p : Pattern} {v : Value} {env1 env2 : Env}
  (h1 : matchPattern p v = some env1)
  (h2 : matchPattern p v = some env2) :
  env1 = env2
```

theorem match_total {p : Pattern} {v : Value}
  (h : patternCovers p (typeof v)) :
  ∃ env, matchPattern p v = some env
```

### Workflow Semantics

Extend to full workflow constructs:

```lean
inductive Workflow where
  | observe (name : String) (expr : Expr)
  | act (name : String) (expr : Expr)
  | decide (name : String) (expr : Expr)
  | sequence (w1 w2 : Workflow)
  | parallel (ws : List Workflow)
```

## Performance Notes

- **Execution**: Uses Lean's native execution (compiled to C)
- **Environment**: O(1) lookup via function application
- **Pattern matching**: Eager evaluation
- **Memory**: No GC pressure from immutability

## Comparison with Rust Implementation

| Aspect | Lean | Rust |
|--------|------|------|
| **Style** | Functional | Imperative |
| **Effects** | Tracked in types | Runtime tracking |
| **Errors** | Except monad | Result type |
| **Environment** | Pure function | HashMap |
| **Pattern match** | Recursive | Iterator-based |
| **Provable** | Yes | No |
| **Speed** | Slower | Faster |
| **Lines of code** | ~500 | ~1500 |

The Lean version prioritizes clarity and provability over raw speed.

## File Organization

Each file has a clear purpose:

- **AST.lean**: Type definitions only, no logic
- **Environment.lean**: Environment operations and effect lattice
- **Serialize.lean**: JSON serialization, isolated from core logic
- **Expr.lean**: Main evaluator, depends on other Eval modules
- **Pattern.lean**: Pattern matching, pure function
- **Match.lean**: Match expression evaluation
- **IfLet.lean**: If-let expression evaluation
- **Differential/**: Testing infrastructure, separate from core
- **Tests/**: Test suite, depends on all core modules

## Dependencies

```
Main.lean ─────▶ Ash ─────▶ Core, Eval, Differential, Tests
                  │
                  ├──▶ Core (AST, Environment, Serialize)
                  │
                  ├──▶ Eval (Expr, Pattern, Match, IfLet)
                  │       │
                  │       └──▶ Core
                  │
                  ├──▶ Differential (Types, Parse, Compare)
                  │       │
                  │       └──▶ Core
                  │
                  └──▶ Tests (Properties, Runner, CI)
                          │
                          └──▶ Core, Eval
```

External dependencies:
- `std`: Standard library extensions
- `plausible`: Property-based testing
